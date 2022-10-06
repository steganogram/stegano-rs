use image::{Rgba, RgbaImage};

use crate::media::image::iterators::{ColorIterMut, TransposeMut};
use crate::media::image::lsb_codec::CodecOptions;
use crate::MediaPrimitiveMut;

/// stegano source for image files, based on `RgbaImage` by `image` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::{Read, Write};
/// use image::{RgbaImage};
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::media::image::encoder::ImageRgbaColorMut;
/// use stegano_core::universal_encoder::{Encoder, OneBitHide};
///
/// // create a `RgbaImage` from a png image file
/// let image_original = image::open("../resources/plain/carrier-image.png")
///     .expect("Cannot open carrier image")
///     .to_rgba8();
/// let mut image = image::open("../resources/plain/carrier-image.png")
///     .expect("Cannot open carrier image")
///     .to_rgba8();
/// let secret_message = "Hello World!".as_bytes();
/// {
///     let mut encoder = Encoder::new(ImageRgbaColorMut::new(&mut image).into_iter(), OneBitHide);
///     encoder.write_all(secret_message)
///         .expect("Cannot write secret message");
/// }
/// assert_ne!(image_original.get_pixel(0, 0), image.get_pixel(0, 0));
/// ```
pub struct ImageRgbaColorMut<'a> {
    i: usize,
    steps: usize,
    skip_alpha: bool,
    pixel: ColorIterMut<'a, Rgba<u8>>,
}

impl<'a> ImageRgbaColorMut<'a> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(input: &'a mut RgbaImage) -> Self {
        Self::new_with_options(input, &CodecOptions::default())
    }

    pub fn new_with_options(input: &'a mut RgbaImage, options: &CodecOptions) -> Self {
        let h = input.height();
        Self {
            i: 0,
            steps: options.get_color_channel_step_increment(),
            skip_alpha: options.get_skip_alpha_channel(),
            pixel: ColorIterMut::from_transpose(TransposeMut::from_rows_mut(input.rows_mut(), h)),
        }
    }
}

impl<'i> Iterator for ImageRgbaColorMut<'i> {
    type Item = MediaPrimitiveMut<'i>;

    fn next(&'_ mut self) -> Option<Self::Item> {
        if self.skip_alpha && self.i > 0 {
            let is_next_alpha = (self.i + 1) % 4 == 0;
            if is_next_alpha {
                self.pixel.next();
                self.i += 1;
            }
        }
        let res = self.pixel.next().map(MediaPrimitiveMut::ImageColorChannel);
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
    use crate::media::image::lsb_codec::Concealer;
    use crate::test_utils::{prepare_small_image, HELLO_WORLD_PNG};

    #[test]
    fn it_should_step_in_increments_smaller_than_one_pixel() {
        let img_ro = prepare_small_image();
        let mut img = img_ro.clone();
        let mut carrier = ImageRgbaColorMut::new_with_options(
            &mut img,
            &CodecOptions {
                skip_alpha_channel: true,
                color_channel_step_increment: 2,
                concealer: Concealer::LeastSignificantBit,
            },
        );

        if let Some(MediaPrimitiveMut::ImageColorChannel(b)) = carrier.nth(1) {
            let (x, y, c) = (0, 0, 2);
            let pixel = img_ro.get_pixel(x, y);
            let expected_color = *pixel.0.get(c).unwrap();

            let actual_color = *b;

            assert_eq!(
                expected_color, actual_color,
                "Pixel at (x={}, y={}) @ color {} mismatched expected={:?}",
                x, y, c, pixel.0
            );
        }
    }

    #[test]
    fn it_should_step_in_increments_bigger_than_one_pixel() {
        let img_ro = prepare_small_image();
        let mut img = img_ro.clone();
        let mut carrier = ImageRgbaColorMut::new_with_options(
            &mut img,
            &CodecOptions {
                skip_alpha_channel: true,
                color_channel_step_increment: 3,
                concealer: Concealer::LeastSignificantBit,
            },
        );

        if let Some(MediaPrimitiveMut::ImageColorChannel(b)) = carrier.nth(1) {
            let (x, y, c) = (0, 1, 0);
            let pixel = img_ro.get_pixel(x, y);
            let expected_color = *pixel.0.get(c).unwrap();

            let actual_color = *b;

            assert_eq!(
                expected_color, actual_color,
                "Pixel at (x={}, y={}) @ color {} mismatched expected={:?}",
                x, y, c, pixel.0
            );
        }
    }

    #[test]
    fn it_should_iterate_columns_first_and_only_3_color_channels() {
        let img_ro = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut carrier = ImageRgbaColorMut::new(&mut img);

        for x in 0..width {
            for y in 0..height {
                let pixel = img_ro.get_pixel(x, y);
                for c in 0..3 {
                    let expected_color = pixel.0.get(c).unwrap();
                    if let Some(MediaPrimitiveMut::ImageColorChannel(b)) = carrier.next() {
                        let actual_color = *b;

                        assert_eq!(
                            *expected_color, actual_color,
                            "Pixel at (x={}, y={}) @ color {} mismatched current={:?}",
                            x, y, c, pixel.0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn it_should_be_possible_to_mutate_colors() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let first_pixel = *img.get_pixel(0, 0);
        {
            let mut carrier = ImageRgbaColorMut::new(&mut img);
            if let MediaPrimitiveMut::ImageColorChannel(color) = carrier.next().unwrap() {
                *color += 0x2;
            }
        }
        let first_pixel_changed = *img.get_pixel(0, 0);
        assert_ne!(
            first_pixel.0.first(),
            first_pixel_changed.0.first(),
            "First Color (Red-Channel) should have been changed."
        );
        assert_eq!(
            first_pixel.0.get(1),
            first_pixel_changed.0.get(1),
            "Second Color (Green-Channel) should be equal."
        );
    }
}
