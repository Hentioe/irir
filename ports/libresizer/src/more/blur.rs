extern crate image;

use super::super::*;
use image::DynamicImage;

pub struct BlurHandler {
    sigma: f32,
}

impl ImageHandler for BlurHandler {
    fn handle(&self, img: DynamicImage) -> Result<DynamicImage> {
        if self.sigma != 0.0 {
            Ok(img.blur(self.sigma))
        } else {
            Ok(img)
        }
    }
}

impl BlurHandler {
    pub fn new(level: Option<u32>) -> Self {
        BlurHandler {
            sigma: level.unwrap_or(0) as f32,
        }
    }
}

pub fn blur(opts: &ImageOption, img_info: &ImageInfo) -> Result<u64> {
    let resizer = Resizer {
        width: img_info.width,
        height: img_info.height,
        filter_type: opts.filter_type(),
    };
    let blur_handler = BlurHandler::new(img_info.blur_level());
    let handlers: Vec<&ImageHandler> = vec![&resizer, &blur_handler];
    pipeline(opts, img_info, handlers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur() {
        let opts = ImageOption::new("../../originals", "../../outputs", FilterType::Lanczos3);
        let mut img_info = ImageInfo::new("ferris", "png", None, None);
        img_info.blur(10);
        println!("hash: {}", blur(&opts, &img_info).unwrap());
    }
}
