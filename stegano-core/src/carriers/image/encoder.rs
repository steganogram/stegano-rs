use crate::universal_encoder::WriteCarrierItem;
use crate::CarrierItem;
use image::{Rgba, RgbaImage};
use std::io::{Error, ErrorKind, Result};

/// Keeps track of the state when writing to a `RgbaImage`
pub struct ImagePngTarget<'i> {
    pub target: &'i mut RgbaImage,
    max_x: u32,
    max_y: u32,
    max_c: u8,
    x: u32,
    y: u32,
    c: u8,
}

impl<'i> ImagePngTarget<'i> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(target: &'i mut RgbaImage) -> Self {
        let (max_x, max_y) = target.dimensions();
        Self {
            target,
            max_x,
            max_y,
            max_c: 3,
            x: 0,
            y: 0,
            c: 0,
        }
    }
}

/// writes a carrier item to the target
impl<'i> WriteCarrierItem for ImagePngTarget<'i> {
    fn write_carrier_item(&mut self, item: &CarrierItem) -> Result<usize> {
        if self.x == self.max_x {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        if let CarrierItem::ImageColorChannel(b) = item {
            let pixel = self.target.get_pixel_mut(self.x, self.y);
            pixel.0[self.c as usize] = *b;
        } else {
            panic!("Unsupported carrier item for images.");
        }

        self.c += 1;
        if self.c == self.max_c {
            self.c = 0;
            self.y += 1;
        }
        if self.y == self.max_y {
            self.y = 0;
            self.x += 1;
        }

        Ok(1)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod image_encoder_tests {
    use super::*;

    const TARGET_PNG: &str = "../resources/plain/carrier-image.png";

    #[test]
    #[should_panic]
    fn it_should_panic_on_non_u8_carrier_items() {
        let mut img = image::open(TARGET_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let mut target = ImagePngTarget::new(&mut img);
        target
            .write_carrier_item(&CarrierItem::AudioSample(i16::MAX))
            .expect("this should panic");
    }

    #[test]
    fn it_should_write_a_u8_carrier_item() {
        let mut img = image::open(TARGET_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let mut target = ImagePngTarget::new(&mut img);
        target
            .write_carrier_item(&CarrierItem::ImageColorChannel(0xAE))
            .expect("this should panic");
    }

    #[test]
    #[should_panic]
    fn it_should_return_error_on_eof() {
        let mut img = image::open(TARGET_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let (max_x, max_y) = img.dimensions();
        let mut target = ImagePngTarget::new(&mut img);
        let max_writes = max_x * max_y * 3;
        for _ in 0..max_writes {
            target
                .write_carrier_item(&CarrierItem::ImageColorChannel(0xAE))
                .expect("error on writing a carrier item");
        }
        target
            .write_carrier_item(&CarrierItem::ImageColorChannel(0xAE))
            .expect("this is the last write, so it should error, it's ok");
    }
}
