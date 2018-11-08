extern crate image;

use super::super::*;
use image::DynamicImage;

struct BlurHandler {
    sigma: f32,
}

impl ImageHandler for BlurHandler {
    fn handle(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.blur(self.sigma))
    }
}

impl BlurHandler {
    pub fn sigma(&mut self, sigma: f32) -> &BlurHandler {
        self.sigma = sigma;
        self
    }
}

pub fn blur(opts: &ImageOption, img_info: &ImageInfo) -> Result<u64> {
    let resizer = Resizer {
        width: img_info.width,
        height: img_info.height,
        filter_type: opts.filter_type(),
    };
    let blur_handler = BlurHandler {
        sigma: img_info.blur_level().unwrap_or(0) as f32,
    };
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
