use crate::media::image::decoder::ImagePngSource;
use crate::media::image::encoder::ImagePngMut;
use crate::universal_decoder::{Decoder, OneBitUnveil};
use crate::universal_encoder::Encoder2;
use image::RgbaImage;
use std::io::{Read, Write};

/// Factory for decoder and encoder
pub struct LSBCodec;

impl LSBCodec {
    /// builds a LSB Image Decoder that implements Read
    ///
    /// ## Example how to retrieve a decoder:
    /// ```rust
    /// use stegano_core::media::image::LSBCodec;
    /// use image::RgbaImage;
    ///
    /// let mut image_with_secret = image::open("../resources/secrets/image-with-hello-world.png")
    ///     .expect("Cannot open secret image")
    ///     .to_rgba();
    ///
    /// let mut buf = vec![0; 13];
    /// LSBCodec::decoder(&mut image_with_secret)
    ///     .read_exact(&mut buf[..])
    ///     .expect("Cannot read 13 bytes from codec");
    ///
    /// let msg = String::from_utf8(buf).expect("Cannot convert result to string");
    /// assert_eq!(msg, "\u{1}Hello World!");
    /// ```
    pub fn decoder<'i>(input: &'i RgbaImage) -> Box<dyn Read + 'i> {
        Box::new(Decoder::new(ImagePngSource::new(input), OneBitUnveil))
    }

    /// builds a LSB Image Encoder that implements Write
    /// ## Example how to retrieve an encoder:
    ///
    /// ```rust
    /// use stegano_core::media::image::LSBCodec;
    /// use image::{RgbaImage, open};
    ///
    /// let mut plain_image = open("../resources/plain/carrier-image.png")
    ///     .expect("Cannot open carrier image")
    ///     .to_rgba();
    /// let (width, height) = plain_image.dimensions();
    /// let secret_message = "Hello World!".as_bytes();
    ///
    /// {
    ///     LSBCodec::encoder(&mut plain_image)
    ///         .write_all(&secret_message[..])
    ///         .expect("Cannot write to codec");
    /// }
    /// let mut buf = vec![0; secret_message.len()];
    /// LSBCodec::decoder(&mut plain_image.into())
    ///     .read_exact(&mut buf[..])
    ///     .expect("Cannot read 12 bytes from codec");
    ///
    /// let msg = String::from_utf8(buf).expect("Cannot convert result to string");
    /// assert_eq!(msg, "Hello World!");
    /// ```
    pub fn encoder<'i>(carrier: &'i mut RgbaImage) -> Box<dyn Write + 'i> {
        Box::new(Encoder2::new(ImagePngMut::new(carrier)))
    }
}

#[cfg(test)]
mod audio_e2e_tests {
    // use super::*;
    // TODO implement them
}
