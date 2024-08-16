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
/// let image_original = image::open("tests/images/plain/carrier-image.png")
///     .expect("Cannot open carrier image")
///     .to_rgba8();
/// let mut image = image::open("tests/images/plain/carrier-image.png")
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
    pixel: ColorIterMut<'a, Rgba<u8>>,
}

impl<'a> ImageRgbaColorMut<'a> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(input: &'a mut RgbaImage) -> Self {
        Self::new_with_options(input, &CodecOptions::default())
    }

    pub fn new_with_options(input: &'a mut RgbaImage, options: &CodecOptions) -> Self {
        let w = input.width();
        Self {
            i: 0,
            steps: options.color_channel_step_increment,
            pixel: ColorIterMut::from_transpose(
                TransposeMut::from_rows_mut(input.rows_mut(), w, options.skip_last_row_and_column),
                options.skip_alpha_channel,
            ),
        }
    }
}

impl<'i> Iterator for ImageRgbaColorMut<'i> {
    type Item = MediaPrimitiveMut<'i>;

    fn next(&'_ mut self) -> Option<Self::Item> {
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
    use crate::test_utils::{
        prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha, prepare_5x5_image,
        HELLO_WORLD_PNG,
    };

    #[test]
    fn it_should_iterate_over_all_colors_of_an_image() {
        let img_ro = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let mut img = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let (width, height) = img.dimensions();
        let mut media_primitive_iter = ImageRgbaColorMut::new(&mut img);

        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let expected_pixel = img_ro.get_pixel(x, y);
                for color_idx in 0..3 {
                    let mut expected_color = *expected_pixel.0.get(color_idx).unwrap();
                    let given_color = media_primitive_iter.next().unwrap_or_else(|| {
                        panic!("MediaPrimitive at ({x}, {y}) was not even existing!")
                    });

                    assert_eq!(
                        given_color,
                        MediaPrimitiveMut::ImageColorChannel(&mut expected_color),
                        "MediaPrimitive at ({x}, {y}) does not match"
                    );
                }
            }
        }
        // ensure iterator is exhausted
        assert!(media_primitive_iter.next().is_none());
    }

    #[test]
    fn it_should_step_in_increments_smaller_than_one_pixel() {
        let img_ro = prepare_5x5_image();
        let mut img = img_ro.clone();
        let mut carrier = ImageRgbaColorMut::new_with_options(
            &mut img,
            &CodecOptions {
                skip_alpha_channel: true,
                color_channel_step_increment: 2,
                ..CodecOptions::default()
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
        let img_ro = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let mut img = img_ro.clone();
        let mut carrier = ImageRgbaColorMut::new_with_options(
            &mut img,
            &CodecOptions {
                skip_alpha_channel: true,
                color_channel_step_increment: 3,
                ..CodecOptions::default()
            },
        );

        if let Some(MediaPrimitiveMut::ImageColorChannel(actual_color)) = carrier.nth(1) {
            let (x, y, expected_color) = (0, 1, 0);
            let pixel = img_ro.get_pixel(x, y);
            let expected_color = pixel.0.get(expected_color).unwrap();

            assert_eq!(
                actual_color, expected_color,
                "Pixel at (x={}, y={}) @ color {} mismatched expected={:?}",
                x, y, expected_color, pixel.0
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

        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let pixel = img_ro.get_pixel(x, y);
                for color_idx in 0..3 {
                    let expected_color = pixel.0.get(color_idx).unwrap();
                    if let Some(MediaPrimitiveMut::ImageColorChannel(actual_color)) = carrier.next()
                    {
                        assert_eq!(
                            actual_color, expected_color,
                            "Pixel at (x={}, y={}) @ color {} mismatched current={:?}",
                            x, y, color_idx, pixel.0
                        );
                    } else {
                        panic!("There should always be a color for the 3 loops here..")
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
