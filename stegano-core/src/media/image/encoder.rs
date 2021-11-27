use image::{Rgba, RgbaImage};

use crate::media::image::iterators::{ColorIter, TransposeMut};
use crate::MediaPrimitiveMut;

/// stegano source for image files, based on `RgbaImage` by `image` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::{Read, Write};
/// use image::{RgbaImage};
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::media::image::encoder::ImagePngMut;
/// use stegano_core::universal_encoder::Encoder;
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
///     let mut encoder = Encoder::new(ImagePngMut::new(&mut image).into_iter());
///     encoder.write_all(secret_message)
///         .expect("Cannot write secret message");
/// }
/// assert_ne!(image_original.get_pixel(0, 0), image.get_pixel(0, 0));
/// ```
pub struct ImagePngMut<'a> {
    i: usize,
    pixel: ColorIter<'a, Rgba<u8>>,
}

impl<'a> ImagePngMut<'a> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(input: &'a mut RgbaImage) -> Self {
        let h = input.height();
        Self {
            i: 0,
            pixel: ColorIter::from_transpose(TransposeMut::from_rows_mut(input.rows_mut(), h)),
        }
    }
}

impl<'i> Iterator for ImagePngMut<'i> {
    type Item = MediaPrimitiveMut<'i>;

    fn next(&'_ mut self) -> Option<Self::Item> {
        let is_alpha = (self.i + 1) % 4 == 0;
        if is_alpha {
            self.pixel.next();
            self.i += 1;
        }
        self.i += 1;
        self.pixel.next().map(MediaPrimitiveMut::ImageColorChannel)
    }
}

#[cfg(test)]
mod decoder_tests {
    use super::*;

    const HELLO_WORLD_PNG: &str = "../resources/with_text/hello_world.png";

    #[test]
    fn it_should_iterate_columns_first_and_only_3_color_channels() {
        let img_ro = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut carrier = ImagePngMut::new(&mut img);

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
            let mut carrier = ImagePngMut::new(&mut img);
            if let MediaPrimitiveMut::ImageColorChannel(color) = carrier.next().unwrap() {
                *color += 0x2;
            }
        }
        let first_pixel_changed = *img.get_pixel(0, 0);
        assert_ne!(
            first_pixel.0.get(0),
            first_pixel_changed.0.get(0),
            "First Color (Red-Channel) should have been changed."
        );
        assert_eq!(
            first_pixel.0.get(1),
            first_pixel_changed.0.get(1),
            "Second Color (Green-Channel) should be equal."
        );
    }
}
