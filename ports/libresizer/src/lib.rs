extern crate image;
extern crate libcore;

use image::FilterType;
use libcore::errors::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ImageOption {
    input_dir: String,
    output_dir: String,
}

impl ImageOption {
    pub fn new(input: &str, output: &str) -> ImageOption {
        ImageOption {
            input_dir: input.to_string(),
            output_dir: output.to_string(),
        }
    }

    pub fn input_dir(&self) -> &str {
        self.input_dir.as_str()
    }
    pub fn output_dir(&self) -> &str {
        self.output_dir.as_str()
    }
}

#[derive(Debug, Clone, Hash)]
pub struct ImageInfo {
    name: String,
    format: String,
    width: Option<u32>,
    height: Option<u32>,
}

impl ImageInfo {
    pub fn new(name: &str, format: &str, width: Option<u32>, height: Option<u32>) -> ImageInfo {
        ImageInfo {
            name: name.to_string(),
            format: format.to_string(),
            width,
            height,
        }
    }

    pub fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn fname(&self) -> String {
        format!("{}.{}", self.name, self.format)
    }
}

pub fn resize(opts: &ImageOption, img_info: &ImageInfo) -> Result<u64> {
    let width = if let Some(w) = img_info.width {
        w
    } else {
        std::u32::MAX
    };
    let height = if let Some(h) = img_info.height {
        h
    } else {
        std::u32::MAX
    };
    if width == std::u32::MAX && height == width {
        return Err(err_msg("Please add at least one parameter"));
    }
    let mut fpath = PathBuf::from(&opts.input_dir());
    fpath.push(img_info.fname());
    let img = image::open(&fpath)?;
    let resized = img.resize(width, height, FilterType::Lanczos3);
    let mut hasher = DefaultHasher::new();
    hasher.write(&resized.raw_pixels());
    let hash = hasher.finish();
    let mut opath = PathBuf::from(&opts.output_dir());
    opath.push(hash.to_string());
    opath.set_extension(
        &fpath
            .extension()
            .ok_or(err_msg("Did not get the extension of the input file"))?,
    );
    resized.save(opath)?;
    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let opts = ImageOption::new("../../originals", "../../outputs");
        let img_info = ImageInfo::new("jojo_01", "jpg", Some(80), None);
        resize(&opts, &img_info).unwrap();
    }
}
