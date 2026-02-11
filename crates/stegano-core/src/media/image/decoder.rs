use super::iterators::{ColorIter, Transpose};
use crate::media::codec_options::LsbCodecOptions;
use crate::media::MediaPrimitive;

use image::{Rgba, RgbaImage};

/// stegano source for image files, based on `RgbaImage` by `image` crate
pub struct ImageRgbaColor<'i> {
    i: usize,
    steps: usize,
    pixel: ColorIter<'i, Rgba<u8>>,
}

impl<'i> ImageRgbaColor<'i> {
    /// constructor for a given `RgbaImage` that lives somewhere
    #[cfg(test)]
    pub fn new(input: &'i RgbaImage) -> Self {
        Self::new_with_options(input, &LsbCodecOptions::default())
    }

    pub fn new_with_options(input: &'i RgbaImage, options: &LsbCodecOptions) -> Self {
        let w = input.width();
        Self {
            i: 0,
            steps: options.get_color_channel_step_increment(),
            pixel: ColorIter::from_transpose(
                Transpose::from_rows(input.rows(), w, true),
                options.skip_alpha_channel,
            ),
        }
    }
}

/// iterates over the image and returns single color channels of each pixel wrapped into a `CarrierItem`
impl Iterator for ImageRgbaColor<'_> {
    type Item = MediaPrimitive;

    #[inline(always)]
    fn next(&'_ mut self) -> Option<Self::Item> {
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
    use std::io::Read;

    use crate::universal_decoder::{OneBitUnveil, UniversalDecoder};

    use super::*;

    const HELLO_WORLD_PNG: &str = "tests/images/with_text/hello_world.png";

    #[test]
    fn it_should_iterate_over_all_colors_of_an_image() {
        let img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut media_primitive_iter = ImageRgbaColor::new(&img);

        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let expected_pixel = img.get_pixel(x, y);
                for color_idx in 0..3 {
                    let expected_color = *expected_pixel.0.get(color_idx).unwrap();
                    let given_color = media_primitive_iter.next().unwrap_or_else(|| {
                        panic!("MediaPrimitive at ({x}, {y}) was not even existing!")
                    });

                    assert_eq!(
                        given_color,
                        expected_color.into(),
                        "MediaPrimitive at ({x}, {y}) does not match"
                    );
                }
            }
        }
        // ensure iterator is exhausted
        assert!(media_primitive_iter.next().is_none());
    }

    #[test]
    fn it() {
        // create a `RgbaImage` from a png image file
        let image = image::open("tests/images/with_text/hello_world.png")
            .expect("Cannot open secret image file")
            .to_rgba8();
        let mut secret = vec![0; 13];

        // create a `Decoder` based on an `ImagePngSource` based on the `RgbaImage`
        UniversalDecoder::new(ImageRgbaColor::new(&image), OneBitUnveil)
            .read_exact(&mut secret)
            .expect("Cannot read 13 bytes from decoder");

        let msg = String::from_utf8(secret).expect("Cannot convert result to string");
        assert_eq!("\u{1}Hello World!", msg);
    }
}
