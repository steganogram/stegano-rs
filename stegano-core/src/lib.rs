//! # Stegano Core API
//!
//! There are 3 main structures exposed via [`SteganoCore`][core] that is
//! - [`SteganoEncoder`][enc] for writing data into an image
//! - [`SteganoDecoder`][dec] for reading data from an image
//! - [`SteganoRawDecoder`][raw] for reading the plain raw bytes from an image
//!
//! # Usage Examples
//!
//! ## Hide data inside an image
//!
//! ```rust
//! use stegano_core::{SteganoCore, SteganoEncoder};
//!
//! SteganoCore::encoder()
//!     .hide_file("Cargo.toml").unwrap()
//!     .use_media("../resources/plain/carrier-image.png").unwrap()
//!     .write_to("image-with-a-file-inside.png")
//!     .hide();
//! ```
//!
//! ## Unveil data from an image
//!
//! ```rust
//! use stegano_core::{SteganoCore, SteganoEncoder, CodecOptions};
//! use stegano_core::commands::unveil;
//! use std::path::Path;
//!
//! SteganoCore::encoder()
//!     .hide_file("Cargo.toml").unwrap()
//!     .use_media("../resources/plain/carrier-image.png").unwrap()
//!     .write_to("image-with-a-file-inside.png")
//!     .hide();
//!
//! unveil(
//!     &Path::new("image-with-a-file-inside.png"),
//!     &Path::new("./"),
//!     &CodecOptions::default()).unwrap();
//! ```
//!
//! [core]: ./struct.SteganoCore.html
//! [enc]: ./struct.SteganoEncoder.html
//! [dec]: ./struct.SteganoDecoder.html
//! [raw]: ./struct.SteganoRawDecoder.html

#![warn(
// clippy::unwrap_used,
// clippy::cargo_common_metadata,
// clippy::branches_sharing_code,
// clippy::cast_lossless,
// clippy::cognitive_complexity,
// clippy::get_unwrap,
// clippy::if_then_some_else_none,
// clippy::inefficient_to_string,
// clippy::match_bool,
// clippy::missing_const_for_fn,
// clippy::missing_panics_doc,
// clippy::option_if_let_else,
// clippy::redundant_closure,
clippy::redundant_else,
// clippy::redundant_pub_crate,
// clippy::ref_binding_to_reference,
// clippy::ref_option_ref,
// clippy::same_functions_in_if_condition,
// clippy::unneeded_field_pattern,
// clippy::unnested_or_patterns,
// clippy::use_self,
)]

pub mod bit_iterator;
pub use bit_iterator::BitIterator;

pub mod message;
pub use message::*;

pub mod raw_message;
pub use raw_message::*;

pub mod commands;
pub mod error;
pub mod media;
pub mod result;
pub mod universal_decoder;
pub mod universal_encoder;

use hound::{WavReader, WavSpec, WavWriter};
use image::RgbaImage;
use std::default::Default;
use std::fs::File;
use std::path::Path;

pub use crate::error::SteganoError;
pub use crate::media::image::CodecOptions;
pub use crate::result::Result;

/// wrap the low level data types that carries information
#[derive(Debug, Eq, PartialEq)]
pub enum MediaPrimitive {
    ImageColorChannel(u8),
    AudioSample(i16),
}

/// mutable primitive for storing stegano data
#[derive(Debug, Eq, PartialEq)]
pub enum MediaPrimitiveMut<'a> {
    ImageColorChannel(&'a mut u8),
    AudioSample(&'a mut i16),
    None,
}

pub trait HideBit {
    fn hide_bit(self, bit: bool) -> Result<()>;
}

impl HideBit for MediaPrimitiveMut<'_> {
    fn hide_bit(self, bit: bool) -> Result<()> {
        match self {
            MediaPrimitiveMut::ImageColorChannel(c) => {
                *c = (*c & (u8::MAX - 1)) | if bit { 1 } else { 0 };
            }
            MediaPrimitiveMut::AudioSample(s) => {
                *s = (*s & (i16::MAX - 1)) | if bit { 1 } else { 0 };
            }
            MediaPrimitiveMut::None => {}
        }
        Ok(())
    }
}

pub type WavAudio = (WavSpec, Vec<i16>);

/// a media container for steganography
pub enum Media {
    Image(RgbaImage),
    Audio(WavAudio),
}

pub struct SteganoCore {}

impl SteganoCore {
    pub fn encoder() -> SteganoEncoder {
        SteganoEncoder::with_options(CodecOptions::default())
    }

    pub fn encoder_with_options(opts: CodecOptions) -> SteganoEncoder {
        SteganoEncoder::with_options(opts)
    }
}

pub trait Hide {
    fn hide_message(&mut self, message: &Message) -> Result<&mut Media>;
    fn hide_message_with_options(
        &mut self,
        message: &Message,
        opts: &CodecOptions,
    ) -> Result<&mut Media>;
}

impl Media {
    pub fn from_file(f: &Path) -> Result<Self> {
        if let Some(ext) = f.extension() {
            let ext = ext.to_str().unwrap().to_lowercase();
            match ext.as_str() {
                "png" => Ok(Self::Image(
                    image::open(f)
                        .map_err(|_e| SteganoError::InvalidImageMedia)?
                        .to_rgba8(),
                )),
                "wav" => {
                    let mut reader =
                        WavReader::open(f).map_err(|_e| SteganoError::InvalidAudioMedia)?;
                    let spec = reader.spec();
                    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();

                    Ok(Self::Audio((spec, samples)))
                }
                _ => Err(SteganoError::UnsupportedMedia),
            }
        } else {
            Err(SteganoError::UnsupportedMedia)
        }
    }
}

pub trait Persist {
    fn save_as(&mut self, _: &Path) -> Result<()>;
}

impl Persist for Media {
    fn save_as(&mut self, file: &Path) -> Result<()> {
        match self {
            Media::Image(i) => i.save(file).map_err(|_e| SteganoError::ImageEncodingError),
            Media::Audio((spec, samples)) => {
                let mut writer =
                    WavWriter::create(file, *spec).map_err(|_| SteganoError::AudioCreationError)?;
                if let Some(error) = samples
                    .iter()
                    .map(|s| {
                        writer
                            .write_sample(*s)
                            .map_err(|_| SteganoError::AudioEncodingError)
                    })
                    .filter_map(Result::err)
                    .next()
                {
                    return Err(error);
                }

                Ok(())
            }
        }
    }
}

impl Hide for Media {
    fn hide_message(&mut self, message: &Message) -> Result<&mut Self> {
        self.hide_message_with_options(message, &CodecOptions::default())
    }

    fn hide_message_with_options(
        &mut self,
        message: &Message,
        opts: &CodecOptions,
    ) -> Result<&mut Media> {
        let buf: Vec<u8> = message.try_into()?;

        match self {
            Media::Image(i) => {
                let (width, height) = i.dimensions();
                let _space_to_fill = (width * height * 3) / 8;
                let mut encoder = media::image::LsbCodec::encoder(i, opts);

                encoder
                    .write_all(buf.as_ref())
                    .map_err(|_e| SteganoError::ImageEncodingError)?
            }
            Media::Audio((_spec, samples)) => {
                let mut encoder = media::audio::LsbCodec::encoder(samples);

                encoder
                    .write_all(buf.as_ref())
                    .map_err(|_e| SteganoError::AudioEncodingError)?
            }
        }

        Ok(self)
    }
}

pub struct SteganoEncoder {
    options: CodecOptions,
    target: Option<String>,
    carrier: Option<Media>,
    message: Message,
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
            options: CodecOptions::default(),
            target: None,
            carrier: None,
            message: Message::empty(),
        }
    }
}

impl SteganoEncoder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_options(opts: CodecOptions) -> Self {
        Self {
            options: opts,
            ..Self::default()
        }
    }

    pub fn use_media(&mut self, input_file: &str) -> Result<&mut Self> {
        let path = Path::new(input_file);
        self.carrier = Some(Media::from_file(path)?);

        Ok(self)
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Self {
        self.target = Some(output_file.to_owned());
        self
    }

    pub fn hide_message(&mut self, msg: &str) -> &mut Self {
        self.message
            .add_file_data("secret-message.txt", msg.as_bytes().to_vec());

        self
    }

    pub fn hide_file(&mut self, input_file: &str) -> Result<&mut Self> {
        {
            let _f = File::open(input_file).expect("Data file was not readable.");
        }
        self.message.add_file(input_file)?;

        Ok(self)
    }

    pub fn hide_files(&mut self, input_files: Vec<&str>) -> Result<&mut Self> {
        self.message.files = Vec::new();
        for f in input_files.iter() {
            self.hide_file(f)?;
        }

        Ok(self)
    }

    pub fn hide(&mut self) -> &Self {
        {
            // TODO this hack needs to be implemented as well :(
            // if self.message.header == ContentVersion::V2 {
            //     space_to_fill -= buf.len();
            //
            //     for _ in 0..space_to_fill {
            //         dec.write_all(&[0])
            //             .expect("Failed to terminate version 2 content.");
            //     }
            // }
        }

        if let Some(media) = self.carrier.as_mut() {
            media
                // .hide_message(&self.message)
                .hide_message_with_options(&self.message, &self.options)
                .expect("Failed to hide message in media")
                .save_as(Path::new(self.target.as_ref().unwrap()))
                .expect("Failed to save media");
        }

        self
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use crate::commands::{unveil, unveil_raw};
    use std::fs;
    use std::io::Read;
    use tempfile::TempDir;

    const BASE_IMAGE: &str = "../resources/Base.png";

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file() {
        SteganoEncoder::new().hide_file("foofile").unwrap();
    }

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file_among_valid() {
        SteganoEncoder::new()
            .hide_files(vec!["Cargo.toml", "foofile"])
            .unwrap();
    }

    #[test]
    fn should_panic_for_invalid_carrier_image_file() {
        let mut encoder = SteganoEncoder::new();
        let result = encoder.use_media("some_random_file.png");
        match result.err() {
            Some(SteganoError::InvalidImageMedia) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn should_panic_for_invalid_media_file() {
        let mut encoder = SteganoEncoder::new();
        let result = encoder.use_media("Cargo.toml");
        match result.err() {
            Some(SteganoError::UnsupportedMedia) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn carrier_item_mut_should_allow_to_mutate_colors() {
        let mut color: u8 = 8;
        let c = MediaPrimitiveMut::ImageColorChannel(&mut color);

        if let MediaPrimitiveMut::ImageColorChannel(i) = c {
            *i = 9;
        }

        assert_eq!(color, 9);
    }

    #[test]
    fn should_accept_a_png_as_target_file() {
        SteganoEncoder::new().write_to("/tmp/out-test-image.png");
    }

    #[test]
    fn should_hide_and_unveil_one_text_file_in_wav() -> Result<()> {
        let out_dir = TempDir::new()?;
        let secret_media_p = out_dir.path().join("secret.wav");
        let secret_media_f = secret_media_p.to_str().unwrap();

        SteganoEncoder::new()
            .hide_file("Cargo.toml")?
            .use_media("../resources/plain/carrier-audio.wav")?
            .write_to(secret_media_f)
            .hide();

        let l = fs::metadata(secret_media_p.as_path())
            .expect("Secret media was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil(
            secret_media_p.as_path(),
            out_dir.path(),
            &CodecOptions::default(),
        )?;

        let given_decoded_secret = out_dir.path().join("Cargo.toml");
        assert_eq_file_content(
            &given_decoded_secret,
            "Cargo.toml".as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_hide_and_unveil_one_text_file() -> Result<()> {
        let out_dir = TempDir::new()?;
        let image_with_secret_path = out_dir.path().join("secret.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();

        SteganoEncoder::new()
            .hide_file("Cargo.toml")?
            .use_media("../resources/with_text/hello_world.png")?
            .write_to(image_with_secret)
            .hide();

        let l = fs::metadata(image_with_secret)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            &CodecOptions::default(),
        )?;

        let given_decoded_secret = out_dir.path().join("Cargo.toml");
        assert_eq_file_content(
            &given_decoded_secret,
            "Cargo.toml".as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_raw_unveil_a_message() -> Result<()> {
        let out_dir = TempDir::new()?;
        let expected_file = out_dir.path().join("hello_world.bin");
        let raw_decoded_secret = expected_file.to_str().unwrap();

        unveil_raw(
            Path::new("../resources/with_text/hello_world.png"),
            expected_file.as_path(),
        )?;

        let l = fs::metadata(raw_decoded_secret)
            .expect("Output file was not written.")
            .len();

        // TODO content verification needs to be done as well
        assert_ne!(l, 0, "Output raw data file was empty.");

        Ok(())
    }

    #[test]
    fn should_hide_and_unveil_a_binary_file() -> Result<()> {
        let out_dir = TempDir::new()?;
        let secret_to_hide = "../resources/secrets/random_1666_byte.bin";
        let image_with_secret_path = out_dir.path().join("random_1666_byte.bin.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let expected_file = out_dir.path().join("random_1666_byte.bin");

        SteganoEncoder::new()
            .hide_file(secret_to_hide)?
            .use_media(BASE_IMAGE)?
            .write_to(image_with_secret)
            .hide();

        let l = fs::metadata(image_with_secret)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            &CodecOptions::default(),
        )?;
        assert_eq_file_content(
            &expected_file,
            secret_to_hide.as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_hide_and_unveil_a_zip_file() -> Result<()> {
        let out_dir = TempDir::new()?;
        let secret_to_hide = "../resources/secrets/zip_with_2_files.zip";
        let image_with_secret_path = out_dir.path().join("zip_with_2_files.zip.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let expected_file = out_dir.path().join("zip_with_2_files.zip");

        SteganoEncoder::new()
            .hide_file(secret_to_hide)?
            .use_media(BASE_IMAGE)?
            .write_to(image_with_secret)
            .hide();

        assert_file_not_empty(image_with_secret);

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            &CodecOptions::default(),
        )?;

        assert_eq_file_content(
            &expected_file,
            secret_to_hide.as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility() -> Result<()> {
        let out_dir = TempDir::new()?;
        let decoded_secret = out_dir.path().join("Blah.txt");

        unveil(
            Path::new("../resources/with_attachment/Blah.txt.png"),
            out_dir.path(),
            &CodecOptions::default(),
        )?;

        assert_eq_file_content(
            &decoded_secret,
            "../resources/secrets/Blah.txt".as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility_with_2_files_reading() -> Result<()> {
        let out_dir = TempDir::new()?;
        let decoded_secret_1 = out_dir.path().join("Blah.txt");
        let decoded_secret_2 = out_dir.path().join("Blah-2.txt");

        unveil(
            Path::new("../resources/with_attachment/Blah.txt__and__Blah-2.txt.png"),
            out_dir.path(),
            &CodecOptions::default(),
        )?;
        assert_eq_file_content(
            &decoded_secret_1,
            "../resources/secrets/Blah.txt".as_ref(),
            "Unveiled data file #1 did not match expected",
        );

        assert_eq_file_content(
            &decoded_secret_2,
            "../resources/secrets/Blah-2.txt".as_ref(),
            "Unveiled data file #2 did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility_with_2_files_writing() -> Result<()> {
        let out_dir = TempDir::new()?;
        let image_with_secret_path = out_dir.path().join("Blah.txt.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let secret_to_hide = "../resources/secrets/Blah.txt";

        // todo: this whole test case.. we don't want to support that any longer
        SteganoEncoder::new()
            // .force_content_version(PayloadCodecFeatures::V2)
            .use_media(BASE_IMAGE)?
            .hide_file(secret_to_hide)?
            .write_to(image_with_secret)
            .hide();

        assert_file_not_empty(image_with_secret);

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            &CodecOptions::default(),
        )?;

        let decoded_secret = out_dir.path().join("Blah.txt");
        assert_eq_file_content(
            decoded_secret.as_ref(),
            secret_to_hide.as_ref(),
            "Unveiled data file did not match expected",
        );

        Ok(())
    }

    // TODO test for hide_message

    fn assert_eq_file_content(file1: &Path, file2: &Path, msg: &str) {
        let mut content1 = Vec::new();
        File::open(file1)
            .expect("file left was not openable.")
            .read_to_end(&mut content1)
            .expect("file left was not readable.");

        let mut content2 = Vec::new();
        File::open(file2)
            .expect("file right was not openable.")
            .read_to_end(&mut content2)
            .expect("file right was not readable.");

        assert_eq!(content1, content2, "{}", msg);
    }

    fn assert_file_not_empty(image_with_secret: &str) {
        let l = fs::metadata(image_with_secret)
            .expect("image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");
    }
}

#[cfg(test)]
mod test_utils {
    use image::{ImageBuffer, RgbaImage};

    pub const HELLO_WORLD_PNG: &str = "../resources/with_text/hello_world.png";

    pub fn prepare_small_image() -> RgbaImage {
        ImageBuffer::from_fn(5, 5, |x, y| {
            let i = (4 * x + 20 * y) as u8;
            image::Rgba([i, i + 1, i + 2, i + 3])
        })
    }
}
