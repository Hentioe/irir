extern crate image;
extern crate libcore;

use libcore::errors::*;
use image::FilterType;
use std::path::PathBuf;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone)]
pub struct ImageOption {
    input_dir: String,
    output_dir: String,
}

impl ImageOption {
    fn new(input: &str, output: &str) -> ImageOption {
        ImageOption { input_dir: input.to_string(), output_dir: output.to_string() }
    }
}

pub fn resize(opts: &ImageOption, fname: &str, nwidth: Option<u32>, nheight: Option<u32>) -> Result<u64> {
    let width = if let Some(w) = nwidth {
        w
    } else {
        std::u32::MAX
    };
    let height = if let Some(h) = nheight {
        h
    } else {
        std::u32::MAX
    };
    let mut fpath = PathBuf::from(&opts.input_dir);
    fpath.push(fname);
    let img = image::open(&fpath)?;
    println!("{}", fpath.to_str().unwrap());
    let resized = img.resize(width, height, FilterType::Gaussian);
    let mut hasher = DefaultHasher::new();
    hasher.write(&resized.raw_pixels());
    let hash = hasher.finish();
    let mut opath = PathBuf::from(&opts.output_dir);
    opath.push(hash.to_string());
    opath.set_extension(&fpath.extension().ok_or(err_msg("Did not get the extension of the input file"))?);
    resized.save(opath)?;
    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let opts = ImageOption::new("../../originals", "../../outputs");
        resize(&opts, "jojo_01.jpg", Some(81), None).unwrap();
    }
}
