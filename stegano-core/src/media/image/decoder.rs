use crate::media::image::iterators::{ColorIter, Transpose};
use crate::media::image::lsb_codec::CodecOptions;
use crate::MediaPrimitive;
use image::{Rgba, RgbaImage};

/// stegano source for image files, based on `RgbaImage` by `image` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use image::{RgbaImage};
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::media::image::decoder::ImageRgbaColor;
///
/// // create a `RgbaImage` from a png image file
/// let mut image = image::open("../resources/with_text/hello_world.png")
///     .expect("Cannot open secret image file")
///     .to_rgba8();
/// let mut secret = vec![0; 13];
///
/// // create a `Decoder` based on an `ImagePngSource` based on the `RgbaImage`
/// Decoder::new(ImageRgbaColor::new(&mut image), OneBitUnveil)
///     .read_exact(&mut secret)
///     .expect("Cannot read 13 bytes from decoder");
///
/// let msg = String::from_utf8(secret).expect("Cannot convert result to string");
/// assert_eq!("\u{1}Hello World!", msg);
/// ```
pub struct ImageRgbaColor<'i> {
    i: usize,
    steps: usize,
    skip_alpha: bool,
    pixel: ColorIter<'i, Rgba<u8>>,
}

impl<'i> ImageRgbaColor<'i> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(input: &'i RgbaImage) -> Self {
        Self::new_with_options(input, &CodecOptions::default())
    }

    pub fn new_with_options(input: &'i RgbaImage, options: &CodecOptions) -> Self {
        let h = input.height();
        Self {
            i: 0,
            steps: options.get_color_channel_step_increment(),
            skip_alpha: options.get_skip_alpha_channel(),
            pixel: ColorIter::from_transpose(Transpose::from_rows(input.rows(), h)),
        }
    }
}

/// iterates over the image and returns single color channels of each pixel wrapped into a `CarrierItem`
impl<'i> Iterator for ImageRgbaColor<'i> {
    type Item = MediaPrimitive;

    #[inline(always)]
    fn next(&'_ mut self) -> Option<Self::Item> {
        if self.skip_alpha && self.i > 0 {
            let is_next_alpha = (self.i + 1) % 4 == 0;
            if is_next_alpha {
                self.pixel.next();
                self.i += 1;
            }
        }
        let res = self
            .pixel
            .next()
            .map(|c| MediaPrimitive::ImageColorChannel(*c));
        self.i += 1;
        for _ in 0..self.steps - 1 {
            self.pixel.next();
            self.i += 1;
        }
        res
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
            .to_rgba8();
        let (_, height) = img.dimensions();
        let first_pixel = *img.get_pixel(0, 0);
        let second_pixel = *img.get_pixel(0, 1);
        let second_row_first_pixel = *img.get_pixel(1, 0);
        let mut source = ImageRgbaColor::new(&img);
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
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut source = ImageRgbaColor::new(&img);
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
