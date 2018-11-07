extern crate image;
extern crate libcore;

pub use image::FilterType;
use libcore::errors::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ImageOption {
    input_dir: String,
    output_dir: String,
    filter_type: FilterType,
}

impl ImageOption {
    pub fn new(input: &str, output: &str, filter_type: FilterType) -> ImageOption {
        ImageOption {
            input_dir: input.to_string(),
            output_dir: output.to_string(),
            filter_type,
        }
    }

    pub fn input_dir(&self) -> &str {
        self.input_dir.as_str()
    }
    pub fn output_dir(&self) -> &str {
        self.output_dir.as_str()
    }
    pub fn filter_type(&self) -> FilterType {
        self.filter_type
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

    pub fn format(&self) -> &str {
        self.format.as_str()
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
    let resized = img.resize(width, height, opts.filter_type());
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
    // Check if the file exists
    if !Path::new(&opath).exists() {
        resized.save(opath)?;
    }
    Ok(hash)
}

pub fn gen_filter_type(filter_type_s: &str) -> Result<FilterType> {
    match filter_type_s.to_lowercase().as_str() {
        "nearest" => Ok(FilterType::Nearest),
        "gaussian" => Ok(FilterType::Gaussian),
        "catmullrom" => Ok(FilterType::CatmullRom),
        "lanczos3" => Ok(FilterType::Lanczos3),
        "triangle" => Ok(FilterType::Triangle),
        _ => Err(err_msg("Unknown FilterType")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let opts = ImageOption::new("../../originals", "../../outputs", FilterType::Lanczos3);
        let img_info = ImageInfo::new("ember", "png", None, Some(55));
        resize(&opts, &img_info).unwrap();
    }
}
