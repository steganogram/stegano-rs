use super::decoder::ImageRgbaColor;
use super::encoder::ImageRgbaColorMut;
use crate::media::codec_options::{Concealer, LsbCodecOptions};
use crate::universal_decoder::{OneBitUnveil, UniversalDecoder};
use crate::universal_encoder::{
    HideAlgorithms, OneBitHide, OneBitInLowFrequencyHide, UniversalEncoder,
};

use image::RgbaImage;
use std::io::{Read, Write};

/// Factory for decoder and encoder
pub struct LsbCodec;

impl LsbCodec {
    /// builds a LSB Image Decoder that implements Read
    pub fn decoder<'i>(input: &'i RgbaImage, opts: &LsbCodecOptions) -> Box<dyn Read + 'i> {
        Box::new(UniversalDecoder::new(
            ImageRgbaColor::new_with_options(input, opts),
            match opts.concealer {
                Concealer::LeastSignificantBit => OneBitUnveil,
                Concealer::LowFrequencies => OneBitUnveil,
            },
        ))
    }

    /// builds a LSB Image Encoder that implements Write
    pub fn encoder<'i>(carrier: &'i mut RgbaImage, opts: &LsbCodecOptions) -> Box<dyn Write + 'i> {
        let algorithm: HideAlgorithms = match opts.concealer {
            Concealer::LeastSignificantBit => OneBitHide.into(),
            Concealer::LowFrequencies => OneBitInLowFrequencyHide.into(),
        };
        Box::new(UniversalEncoder::new(
            ImageRgbaColorMut::new_with_options(carrier, opts),
            algorithm,
        ))
    }
}

#[cfg(feature = "benchmarks")]
#[allow(unused_imports)] // clippy false positive, on nightly when `cargo bench` is called
mod benchmarks {
    use super::LsbCodec;
    use super::*;

    /// Benchmark for decoding an image
    #[bench]
    fn image_decoding(b: &mut test::Bencher) {
        let img = image::open("tests/images/with_text/hello_world.png")
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut buf = [0; 13];

        b.iter(|| {
            UniversalDecoder::new(ImageRgbaColor::new(&img), OneBitUnveil)
                .read_exact(&mut buf)
                .expect("Failed to read 13 bytes");
        })
    }

    /// Benchmark for encoding an image
    #[bench]
    fn image_encoding(b: &mut test::Bencher) {
        let mut plain_image = image::open("tests/images/plain/carrier-image.png")
            .expect("Input image is not readable.")
            .to_rgba8();
        let secret_message = b"Hello World!";

        b.iter(|| {
            UniversalEncoder::new(ImageRgbaColorMut::new(&mut plain_image), OneBitHide)
                .write_all(&secret_message[..])
                .expect("Cannot write secret message");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode() {
        let image_with_secret = image::open("tests/images/secrets/image-with-hello-world.png")
            .expect("Cannot open secret image")
            .to_rgba8();

        let mut buf = vec![0; 13];
        LsbCodec::decoder(&image_with_secret, &LsbCodecOptions::default())
            .read_exact(&mut buf[..])
            .expect("Cannot read 13 bytes from codec");

        let msg = String::from_utf8(buf).expect("Cannot convert result to string");
        assert_eq!(msg, "\u{1}Hello World!");
    }

    #[test]
    fn should_encode() {
        let mut plain_image = image::open("tests/images/plain/carrier-image.png")
            .expect("Cannot open carrier image")
            .to_rgba8();
        let secret_message = "Hello World!".as_bytes();

        {
            LsbCodec::encoder(&mut plain_image, &LsbCodecOptions::default())
                .write_all(secret_message)
                .expect("Cannot write to codec");
        }
        let mut buf = vec![0; secret_message.len()];
        LsbCodec::decoder(&plain_image, &LsbCodecOptions::default())
            .read_exact(&mut buf[..])
            .expect("Cannot read 12 bytes from codec");

        let msg = String::from_utf8(buf).expect("Cannot convert result to string");
        assert_eq!(msg, "Hello World!");
    }
}
