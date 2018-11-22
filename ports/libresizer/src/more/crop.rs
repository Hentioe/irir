extern crate image;

use super::super::*;
use super::blur::BlurHandler;
use image::DynamicImage;

struct CropHandler {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl CropHandler {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        CropHandler {
            x,
            y,
            width,
            height,
        }
    }
}

impl ImageHandler for CropHandler {
    fn handle(&self, img: DynamicImage) -> Result<DynamicImage> {
        let mut nimg = img.clone();
        Ok(nimg.crop(self.x, self.y, self.width, self.height))
    }
}

pub fn crop(opts: &ImageOption, img_info: &ImageInfo) -> Result<u64> {
    let resizer = Resizer {
        width: img_info.width,
        height: img_info.height,
        filter_type: opts.filter_type(),
    };
    let crop_handler = CropHandler::new(
        img_info.crop_x.unwrap_or(0),
        img_info.crop_y.unwrap_or(0),
        img_info.crop_w.unwrap_or(MAX),
        img_info.crop_h.unwrap_or(MAX),
    );
    let blur_handler = BlurHandler::new(img_info.blur_level());
    let handlers: Vec<&ImageHandler> = if let Some(_blur) = img_info.blur_level() {
        vec![&crop_handler, &resizer, &blur_handler]
    } else {
        vec![&crop_handler, &resizer]
    };
    pipeline(opts, img_info, handlers)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_crop() {
        let opts = ImageOption::new("../../originals", "../../outputs", FilterType::Lanczos3);
        let mut img_info = ImageInfo::new("ferris", "png", None, None);
        img_info.crop(Some(500), None, None, Some(200));
        img_info.blur(15);
        println!("hash: {}", crop(&opts, &img_info).unwrap());
    }
}
