use crate::MediaPrimitive;
use image::RgbaImage;

/// stegano source for image files, based on `RgbaImage` by `image` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use image::{RgbaImage};
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::media::image::decoder::ImagePngSource;
///
/// // create a `RgbaImage` from a png image file
/// let mut image = image::open("../resources/with_text/hello_world.png")
///     .expect("Cannot open secret image file")
///     .to_rgba();
/// let mut secret = vec![0; 13];
///
/// // create a `Decoder` based on an `ImagePngSource` based on the `RgbaImage`
/// Decoder::new(ImagePngSource::new(&mut image), OneBitUnveil)
///     .read_exact(&mut secret)
///     .expect("Cannot read 13 bytes from decoder");
///
/// let msg = String::from_utf8(secret).expect("Cannot convert result to string");
/// assert_eq!("\u{1}Hello World!", msg);
/// ```
pub struct ImagePngSource<'i> {
    pub input: &'i RgbaImage,
    max_x: u32,
    max_y: u32,
    max_c: u8,
    pub x: u32,
    pub y: u32,
    pub c: u8,
}

impl<'i> ImagePngSource<'i> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(input: &'i RgbaImage) -> Self {
        let (max_x, max_y) = input.dimensions();
        Self {
            input,
            max_x,
            max_y,
            max_c: 3,
            x: 0,
            y: 0,
            c: 0,
        }
    }
}

/// iterates over the image and returns single color channels of each pixel wrapped into a `CarrierItem`
impl<'i> Iterator for ImagePngSource<'i> {
    type Item = MediaPrimitive;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.max_x {
            return None;
        }
        let pixel = self.input.get_pixel(self.x, self.y);
        let result = Some(MediaPrimitive::ImageColorChannel(pixel.0[self.c as usize]));
        self.c += 1;
        if self.c == self.max_c {
            self.c = 0;
            self.y += 1;
        }
        if self.y == self.max_y {
            self.y = 0;
            self.x += 1;
        }
        result
    }
}

#[cfg(test)]
mod decoder_tests {
    use super::*;

    const HELLO_WORLD_PNG: &str = "../resources/with_text/hello_world.png";

    #[test]
    fn it_should_iterate_over_all_colors_of_an_image() {
        let img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let (_, height) = img.dimensions();
        let first_pixel = *img.get_pixel(0, 0);
        let second_pixel = *img.get_pixel(0, 1);
        let second_row_first_pixel = *img.get_pixel(1, 0);
        let mut source = ImagePngSource::new(&img);
        assert_eq!(
            source.next().unwrap(),
            MediaPrimitive::ImageColorChannel(first_pixel.0[0]),
            "pixel(0, 0) color 1 does not match"
        );
        source.next();
        assert_eq!(
            source.next().unwrap(),
            MediaPrimitive::ImageColorChannel(first_pixel.0[2]),
            "pixel(0, 0) color 3 does not match"
        );
        assert_eq!(
            source.next().unwrap(),
            MediaPrimitive::ImageColorChannel(second_pixel.0[0]),
            "pixel(0, 1) color 1 does not match"
        );
        assert_eq!(
            source.nth(((height * 3) - 4) as usize).unwrap(),
            MediaPrimitive::ImageColorChannel(second_row_first_pixel.0[0]),
            "pixel(1, 0) color 1 does not match"
        );
    }

    #[test]
    fn it_should_yield_none_after_last_pixel_last_color() {
        let img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let (width, height) = img.dimensions();
        let mut source = ImagePngSource::new(&img);
        assert_ne!(
            source.nth(((height * width * 3) - 1) as usize),
            None,
            "last pixel color 3 should not be None"
        );
        assert_eq!(
            source.nth(((height * width * 3) + 1) as usize),
            None,
            "last pixel after color 3 should be none"
        );
    }
}
