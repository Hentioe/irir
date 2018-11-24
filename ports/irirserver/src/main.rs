#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate env_logger;
extern crate failure;
extern crate irirserver;
extern crate libcore;
extern crate libresizer;
extern crate regex;

use actix_web::middleware::Logger;
use actix_web::{
    fs, http, middleware::ErrorHandlers, middleware::Response, server, App, HttpRequest,
    HttpResponse, Result as AtxResult,
};
use irirserver::cli;
use irirserver::errors::*;
use libcore::errors::*;
use libresizer::{ImageInfo, ImageOption};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

fn render_display_404<S>(_: &HttpRequest<S>, mut resp: HttpResponse) -> AtxResult<Response> {
    resp.set_body(WebError::NotFound.to_string());
    Ok(Response::Done(resp))
}

type WebResult<T> = std::result::Result<T, WebError>;

struct DisplayAppState {
    options: ImageOption,
}

lazy_static! {
    static ref MATCHES: clap::ArgMatches<'static> = cli::build_cli().get_matches();
    static ref Cache: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
}

fn display_resize(req: &HttpRequest<DisplayAppState>) -> WebResult<fs::NamedFile> {
    let (name, format) = get_file_params(req).map_err(WebError::internal)?;
    let (width, height) = get_size_params(req).map_err(WebError::internal)?;
    check_size(&width, &height).map_err(WebError::internal)?;
    let img_info = ImageInfo::new(name.as_str(), format.as_str(), width, height);
    display(&req.state().options, &img_info)
}

fn display_blur(req: &HttpRequest<DisplayAppState>) -> WebResult<fs::NamedFile> {
    let params = req.match_info();
    let level: u32 = params
        .get("level")
        .unwrap_or("5")
        .parse()
        .map_err(WebError::parse)?;
    let (name, format) = get_file_params(req).map_err(WebError::internal)?;
    let (width, height) = get_size_params(req).map_err(WebError::internal)?;
    let mut img_info = ImageInfo::new(name.as_str(), format.as_str(), width, height);
    img_info.blur(level);
    display(&req.state().options, &img_info)
}

fn display_crop(req: &HttpRequest<DisplayAppState>) -> WebResult<fs::NamedFile> {
    let blur_l = get_blur_params(req).map_err(WebError::internal)?;
    let (name, format) = get_file_params(req).map_err(WebError::internal)?;
    let (width, height) = get_size_params(req).map_err(WebError::internal)?;
    let mut img_info = ImageInfo::new(name.as_str(), format.as_str(), width, height);
    if let Some(l) = blur_l {
        img_info.blur(l);
    };
    let crop_params = get_crop_params(req).map_err(WebError::internal)?;
    img_info.crop(crop_params.0, crop_params.1, crop_params.2, crop_params.3);
    display(&req.state().options, &img_info)
}

fn display(opts: &ImageOption, img_info: &ImageInfo) -> WebResult<fs::NamedFile> {
    let info_hash = img_info.to_hash();
    let mut cache = Cache.lock().unwrap();
    // In the cache
    if cache.contains_key(&info_hash) {
        let hash = cache
            .get(&info_hash)
            .ok_or(err_msg("Cache temporary error"))
            .map_err(WebError::internal)?;
        let mut opath = PathBuf::from(&opts.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(img_info.format());
        Ok(fs::NamedFile::open(
            opath
                .to_str()
                .ok_or(err_msg("No output file found"))
                .map_err(WebError::internal)?,
        )
        .map_err(WebError::io)?)
    } else {
        // Handle & Add to cache
        let hash = {
            if img_info.croped() {
                libresizer::more::crop(&opts, &img_info).map_err(WebError::internal)?
            } else if img_info.blur_level() != None {
                libresizer::more::blur(&opts, &img_info).map_err(WebError::internal)?
            } else {
                libresizer::resize(&opts, &img_info).map_err(WebError::internal)?
            }
        };
        let mut opath = PathBuf::from(&opts.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(img_info.format());
        let nf = fs::NamedFile::open(
            opath
                .to_str()
                .ok_or(err_msg("No output file found"))
                .map_err(WebError::internal)?,
        )
        .map_err(WebError::io)?;
        cache.insert(img_info.to_hash(), hash);
        Ok(nf)
    }
}

fn get_file_params(req: &HttpRequest<DisplayAppState>) -> Result<(String, String)> {
    let params = req.match_info();
    let name = params
        .get("name")
        .ok_or(err_msg("Missing name parameter"))?;
    let format = params
        .get("format")
        .ok_or(err_msg("Missing format parameter"))?;
    Ok((name.to_string(), format.to_string()))
}

fn get_crop_params(
    req: &HttpRequest<DisplayAppState>,
) -> Result<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)> {
    let params = req.match_info();
    let crop = if let Some(crop_s) = params.get("crop_s") {
        let re_x = Regex::new(r"x(?P<x>\d+)")?;
        let re_y = Regex::new(r"y(?P<y>\d+)")?;
        let re_w = Regex::new(r"w(?P<w>\d+)")?;
        let re_h = Regex::new(r"h(?P<h>\d+)")?;

        let x = if let Some(caps) = re_x.captures(crop_s) {
            Some(caps["x"].parse::<u32>()?)
        } else {
            None
        };

        let y = if let Some(caps) = re_y.captures(crop_s) {
            Some(caps["y"].parse::<u32>()?)
        } else {
            None
        };

        let w = if let Some(caps) = re_w.captures(crop_s) {
            Some(caps["w"].parse::<u32>()?)
        } else {
            None
        };

        let h = if let Some(caps) = re_h.captures(crop_s) {
            Some(caps["h"].parse::<u32>()?)
        } else {
            None
        };
        (x, y, w, h)
    } else {
        let query = req.query();
        let x = if let Some(x_s) = query.get("c_x") {
            Some(x_s.parse::<u32>()?)
        } else {
            None
        };
        let y = if let Some(y_s) = query.get("c_y") {
            Some(y_s.parse::<u32>()?)
        } else {
            None
        };
        let w = if let Some(w_s) = query.get("c_w") {
            Some(w_s.parse::<u32>()?)
        } else {
            None
        };
        let h = if let Some(h_s) = query.get("c_h") {
            Some(h_s.parse::<u32>()?)
        } else {
            None
        };
        (x, y, w, h)
    };
    Ok(crop)
}

fn get_blur_params(req: &HttpRequest<DisplayAppState>) -> Result<Option<u32>> {
    let params = req.match_info();
    let blur_l = if let Some(level_s) = params.get("level") {
        Some(level_s.parse()?)
    } else {
        let query = req.query();
        if let Some(bl_s) = query.get("bl") {
            Some(bl_s.parse()?)
        } else {
            None
        }
    };
    Ok(blur_l)
}

fn get_size_params(req: &HttpRequest<DisplayAppState>) -> Result<(Option<u32>, Option<u32>)> {
    let params = req.match_info();
    if let Some(size_s) = params.get("size_s") {
        let re_w = Regex::new(r"w(?P<width>\d+)")?;
        let re_h = Regex::new(r"h(?P<height>\d+)")?;
        let width = if let Some(caps) = re_w.captures(size_s) {
            Some(caps["width"].parse()?)
        } else {
            None
        };
        let height = if let Some(caps) = re_h.captures(size_s) {
            Some(caps["height"].parse()?)
        } else {
            None
        };
        Ok((width, height))
    } else {
        let query = req.query();
        let width = if let Some(w) = query.get("w") {
            Some(w.parse()?)
        } else {
            None
        };
        let height = if let Some(h) = query.get("h") {
            Some(h.parse()?)
        } else {
            None
        };
        Ok((width, height))
    }
}

fn check_size(width: &Option<u32>, height: &Option<u32>) -> Result<()> {
    let width = width.unwrap_or(0);
    let height = height.unwrap_or(0);
    if width > 1280 || height > 1280 {
        Err(err_msg("Illegal size parameter"))
    } else {
        Ok(())
    }
}

fn main() {
    let port = MATCHES.value_of("port").unwrap();

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let apps = || {
        let originals = MATCHES.value_of("origin_path").unwrap();
        let outputs = MATCHES.value_of("output_path").unwrap();
        let filter_type =
            libresizer::gen_filter_type(MATCHES.value_of("filter_type").unwrap()).unwrap();
        vec![
            App::with_state(DisplayAppState {
                options: ImageOption::new(originals, outputs, filter_type),
            })
            .middleware(Logger::default())
            .middleware(Logger::new("%a %{User-Agent}i"))
            .middleware(
                ErrorHandlers::new().handler(http::StatusCode::NOT_FOUND, render_display_404),
            )
            .prefix("/display")
            .resource("", |r| {
                r.f(|_req| {
                    "Access images via /display/w{num}/{file_name} or /display/h{num}/{file_name}"
                })
            })
            .resource("/bl/{name}.{format}", |r| r.f(display_blur))
            .resource("/bl/{size_s}/{name}.{format}", |r| r.f(display_blur))
            .resource("/bl{level}/{name}.{format}", |r| r.f(display_blur))
            .resource("/bl{level}/{size_s}/{name}.{format}", |r| r.f(display_blur))
            .resource("/cr/{crop_s}/{name}.{format}", |r| r.f(display_crop))
            .resource("/cr/{crop_s}/bl{level}/{name}.{format}", |r| {
                r.f(display_crop)
            })
            .resource("/cr/{crop_s}/bl{level}/{size_s}/{name}.{format}", |r| {
                r.f(display_crop)
            })
            .resource("/cr/{crop_s}/{size_s}/{name}.{format}", |r| {
                r.f(display_crop)
            })
            .resource("/{name}.{format}", |r| r.f(display_resize))
            .resource("/{size_s}/{name}.{format}", |r| r.f(display_resize))
            .finish(),
            App::with_state(DisplayAppState {
                options: ImageOption::new(originals, outputs, filter_type),
            })
            .handler(
                "/",
                fs::StaticFiles::new("www")
                    .unwrap()
                    .index_file("index.html"),
            )
            .finish(),
        ]
    };
    server::new(apps)
        .bind(format!("0.0.0.0:{}", port))
        .unwrap()
        .run();
}
