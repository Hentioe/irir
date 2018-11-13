#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate env_logger;
extern crate libcore;
extern crate libresizer;
extern crate regex;

use actix_web::middleware::Logger;
use actix_web::{fs, server, App, HttpRequest};
use irirserver::cli;
use libcore::errors::*;
use libresizer::{ImageInfo, ImageOption};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

struct DisplayAppState {
    options: ImageOption,
}

lazy_static! {
    static ref MATCHES: clap::ArgMatches<'static> = cli::build_cli().get_matches();
    static ref Cache: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
}

fn display_resize(req: &HttpRequest<DisplayAppState>) -> Result<fs::NamedFile> {
    let (name, format) = get_file_params(req)?;
    let (width, height) = get_size_params(req)?;
    check_size(&width, &height)?;
    let img_info = ImageInfo::new(name.as_str(), format.as_str(), width, height);
    display(&req.state().options, &img_info)
}

fn display_blur(req: &HttpRequest<DisplayAppState>) -> Result<fs::NamedFile> {
    let params = req.match_info();
    let level: u32 = params.get("level").unwrap_or("10").parse()?;
    let (name, format) = get_file_params(req)?;
    let (width, height) = get_size_params(req)?;
    let mut img_info = ImageInfo::new(name.as_str(), format.as_str(), width, height);
    img_info.blur(level);
    display(&req.state().options, &img_info)
}

fn display(opts: &ImageOption, img_info: &ImageInfo) -> Result<fs::NamedFile> {
    let info_hash = img_info.to_hash();
    let mut cache = Cache.lock().unwrap();
    // In the cache
    if cache.contains_key(&info_hash) {
        let hash = cache
            .get(&info_hash)
            .ok_or(err_msg("Cache temporary error"))?;
        let mut opath = PathBuf::from(&opts.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(img_info.format());
        Ok(fs::NamedFile::open(
            opath.to_str().ok_or(err_msg("No output file found"))?,
        )?)
    } else {
        // Handle & Add to cache
        let hash = {
            if img_info.blur_level() != None {
                libresizer::more::blur(&opts, &img_info)?
            } else {
                libresizer::resize(&opts, &img_info)?
            }
        };
        let mut opath = PathBuf::from(&opts.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(img_info.format());
        let nf = fs::NamedFile::open(opath.to_str().ok_or(err_msg("No output file found"))?)?;
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
        vec![App::with_state(DisplayAppState {
            options: ImageOption::new(originals, outputs, filter_type),
        })
        .middleware(Logger::default())
        .middleware(Logger::new("%a %{User-Agent}i"))
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
        .resource("/{name}.{format}", |r| r.f(display_resize))
        .resource("/{size_s}/{name}.{format}", |r| r.f(display_resize))
        .finish()]
    };
    server::new(apps)
        .bind(format!("0.0.0.0:{}", port))
        .unwrap()
        .run();
}
