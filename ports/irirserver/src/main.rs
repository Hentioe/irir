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

lazy_static! {
    static ref IOPTS: ImageOption = {
        let matches = cli::build_cli().get_matches();
        let originals = matches.value_of("origin_path").unwrap();
        let outputs = matches.value_of("output_path").unwrap();
        let filter_type =
            libresizer::gen_filter_type(matches.value_of("filter_type").unwrap()).unwrap();
        ImageOption::new(originals, outputs, filter_type)
    };
    static ref Cache: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
}
fn display_by_path(req: &HttpRequest) -> Result<fs::NamedFile> {
    let params = req.match_info();
    let name = params
        .get("name")
        .ok_or(err_msg("Missing name parameter"))?;
    let format = params
        .get("format")
        .ok_or(err_msg("Missing format parameter"))?;
    let size_s = params
        .get("size_s")
        .ok_or(err_msg("Missing size parameter"))?;

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
    check_size(&width, &height)?;
    let img_info = ImageInfo::new(name, format, width, height);
    display(&img_info)
}

fn display_by_query(req: &HttpRequest) -> Result<fs::NamedFile> {
    let params = req.match_info();
    let name = params
        .get("name")
        .ok_or(err_msg("Missing name parameter"))?;
    let format = params
        .get("format")
        .ok_or(err_msg("Missing format parameter"))?;
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
    check_size(&width, &height)?;
    let img_info = ImageInfo::new(name, format, width, height);
    display(&img_info)
}

fn display(img_info: &ImageInfo) -> Result<fs::NamedFile> {
    let info_hash = img_info.to_hash();
    let mut cache = Cache.lock().unwrap();
    // In cache
    if cache.contains_key(&info_hash) {
        let hash = cache
            .get(&info_hash)
            .ok_or(err_msg("Cache temporary error"))?;
        let mut opath = PathBuf::from(&IOPTS.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(img_info.format());
        Ok(fs::NamedFile::open(
            opath.to_str().ok_or(err_msg("No output file found"))?,
        )?)
    } else {
        // Resize & Add cache
        let hash = libresizer::resize(&IOPTS, &img_info)?;
        let mut opath = PathBuf::from(&IOPTS.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(img_info.format());
        let nf = fs::NamedFile::open(opath.to_str().ok_or(err_msg("No output file found"))?)?;
        cache.insert(img_info.to_hash(), hash);
        Ok(nf)
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
    let matches = cli::build_cli().get_matches();
    let port = matches.value_of("port").unwrap();
    libresizer::gen_filter_type(matches.value_of("filter_type").unwrap()).unwrap();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let apps = || {
        vec![App::new()
            .middleware(Logger::default())
            .middleware(Logger::new("%a %{User-Agent}i"))
            .prefix("/display")
            .resource("/", |r| {
                r.f(|_req| "/Use '{file_name}?w={num}&h={num}' to access images")
            })
            .resource("/{name}.{format}", |r| r.f(display_by_query))
            .resource("/{size_s}/{name}.{format}", |r| r.f(display_by_path))
            .finish()]
    };
    server::new(apps)
        .bind(format!("0.0.0.0:{}", port))
        .unwrap()
        .run();
}
