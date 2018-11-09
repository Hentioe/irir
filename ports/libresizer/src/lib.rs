extern crate image;
extern crate libcore;

use image::DynamicImage;
pub use image::FilterType;
use libcore::errors::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::u32::MAX;

pub mod more;

#[derive(Clone)]
pub struct ImageOption {
    input_dir: String,
    output_dir: String,
    filter_type: FilterType,
}

impl ImageOption {
    pub fn new(input: &str, output: &str, filter_type: &FilterType) -> ImageOption {
        ImageOption {
            input_dir: input.to_string(),
            output_dir: output.to_string(),
            filter_type: *filter_type,
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

#[derive(Debug, Clone, Hash, Default)]
pub struct ImageInfo {
    name: String,
    format: String,
    width: Option<u32>,
    height: Option<u32>,
    blur_level: Option<u32>,
}

impl ImageInfo {
    pub fn new(name: &str, format: &str, width: Option<u32>, height: Option<u32>) -> ImageInfo {
        ImageInfo {
            name: name.to_string(),
            format: format.to_string(),
            width,
            height,
            ..Default::default()
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

    pub fn blur_level(&self) -> Option<u32> {
        self.blur_level
    }

    pub fn blur(&mut self, level: u32) -> &ImageInfo {
        self.blur_level = Some(level);
        self
    }
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

fn pipeline(opts: &ImageOption, img_info: &ImageInfo, handlers: Vec<&ImageHandler>) -> Result<u64> {
    // Load original image
    let mut fpath = PathBuf::from(&opts.input_dir());
    fpath.push(img_info.fname());
    let img = image::open(&fpath)?;
    // Recursive call handler
    let result_img = pipeline_each(img, 0, handlers)?;
    // Get hash
    let mut hasher = DefaultHasher::new();
    hasher.write(&result_img.raw_pixels());
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
        result_img.save(opath)?;
    }
    Ok(hash)
}

fn pipeline_each(
    result: DynamicImage,
    begin: usize,
    handlers: Vec<&ImageHandler>,
) -> Result<DynamicImage> {
    if begin >= handlers.len() {
        Ok(result)
    } else {
        let handler = handlers
            .get(begin)
            .ok_or(err_msg(format!("No image handler found, index: {}", begin)))?;
        let next = begin + 1;
        pipeline_each(handler.handle(result)?, next, handlers)
    }
}

trait ImageHandler {
    fn handle(&self, img: DynamicImage) -> Result<DynamicImage>;
}

struct Resizer {
    width: Option<u32>,
    height: Option<u32>,
    filter_type: FilterType,
}

impl ImageHandler for Resizer {
    fn handle(&self, img: DynamicImage) -> Result<DynamicImage> {
        let width = self.width.unwrap_or(MAX);
        let height = self.height.unwrap_or(MAX);

        let resized = {
            if width == MAX && height == MAX {
                // Original size
                img
            } else if width == MAX || height == MAX {
                // Preserve aspect ratio
                img.resize(width, height, self.filter_type)
            } else {
                // Does not preserve aspect ratio
                img.resize_exact(width, height, self.filter_type)
            }
        };
        Ok(resized)
    }
}

pub fn resize(opts: &ImageOption, img_info: &ImageInfo) -> Result<u64> {
    pipeline(
        opts,
        img_info,
        vec![&Resizer {
            width: img_info.width,
            height: img_info.height,
            filter_type: opts.filter_type(),
        }],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let opts = ImageOption::new("../../originals", "../../outputs", FilterType::Lanczos3);
        let img_info = ImageInfo::new("ferris", "png", None, Some(350));
        println!("hash: {}", resize(&opts, &img_info).unwrap());
    }
}
