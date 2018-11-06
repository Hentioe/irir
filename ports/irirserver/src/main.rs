#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate libcore;
extern crate libresizer;

use actix_web::{fs, server, App, HttpRequest};
use libcore::errors::*;
use libresizer::{ImageInfo, ImageOption};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    static ref IOPTS: ImageOption = ImageOption::new("./originals", "./outputs");
    static ref Cache: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
}

fn display_img(req: &HttpRequest) -> Result<fs::NamedFile> {
    let params = req.match_info();
    let name = params
        .get("name")
        .ok_or(err_msg("Missing name parameter"))?;
    let format = params
        .get("format")
        .ok_or(err_msg("Missing format parameter"))?;
    let query = req.query();
    let width = if let Some(w) = query.get("w") {
        let w: u32 = w.parse()?;
        if w > 1280 {
            return Err(err_msg("Width out of range"));
        }
        Some(w)
    } else {
        None
    };
    let height = if let Some(h) = query.get("h") {
        let h: u32 = h.parse()?;
        if h > 1280 {
            return Err(err_msg("Height out of range"));
        }
        Some(h)
    } else {
        None
    };
    let img_info = ImageInfo::new(name, format, width, height);
    let info_hash = img_info.to_hash();
    let mut cache = Cache.lock().unwrap();
    // In cache
    if cache.contains_key(&info_hash) {
        let hash = cache
            .get(&info_hash)
            .ok_or(err_msg("Cache temporary error"))?;
        let mut opath = PathBuf::from(&IOPTS.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(format);
        Ok(fs::NamedFile::open(
            opath.to_str().ok_or(err_msg("No output file found"))?,
        )?)
    } else {
        // Resize & Add cache
        let hash = libresizer::resize(&IOPTS, &img_info)?;
        let mut opath = PathBuf::from(&IOPTS.output_dir());
        opath.push(hash.to_string());
        opath.set_extension(format);
        let nf = fs::NamedFile::open(opath.to_str().ok_or(err_msg("No output file found"))?)?;
        cache.insert(img_info.to_hash(), hash);
        Ok(nf)
    }
}

fn main() {
    let apps = || {
        vec![App::new()
            .prefix("/display")
            .resource("/", |r| r.f(|_req| "/{file_name} access image"))
            .resource("/{name}.{format}", |r| r.f(display_img))
            .finish()]
    };
    server::new(apps).bind("127.0.0.1:8080").unwrap().run();
}
