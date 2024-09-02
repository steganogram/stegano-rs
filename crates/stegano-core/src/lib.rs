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
//! use tempfile::tempdir;
//!
//! let temp_dir = tempdir().expect("Failed to create temporary directory");
//!
//! stegano_core::api::hide::prepare()
//!     .with_file("Cargo.toml")        // will hide this file inside the image
//!     .with_message("Hello, World!")  // will hide this message inside the image too
//!     .with_password("SuperSecret42") // will encrypt all the data with this password
//!     .with_image("tests/images/plain/carrier-image.png")
//!     .with_output(temp_dir.path().join("image-with-a-file-inside.png"))
//!     .execute()
//!     .expect("Failed to hide file in image");
//! ```
//!
//! ## Unveil data from an image
//!
//! ```rust
//! use tempfile::tempdir;
//!
//! let temp_dir = tempdir().expect("Failed to create temporary directory");
//!
//! stegano_core::api::unveil::prepare()
//!     .with_secret_image("tests/images/encrypted/hello_world.png")
//!     .with_password("Secret42")
//!     .with_output_folder(temp_dir.path())
//!     .execute()
//!     .expect("Failed to unveil message from image");
//! ```
//!
//! [core]: ./struct.SteganoCore.html
//! [enc]: ./struct.SteganoEncoder.html
//! [dec]: ./struct.SteganoDecoder.html
//! [raw]: ./struct.SteganoRawDecoder.html

#![warn(
    // clippy::unwrap_used,
    // clippy::expect_used,
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
use media::payload::{FabA, FabS, PayloadCodecFactory};
use media::types::Media;
pub use message::*;

pub mod raw_message;
pub use raw_message::*;

pub mod api;
pub mod commands;
pub mod error;
pub mod media;
pub mod result;
pub mod universal_decoder;
pub mod universal_encoder;

use std::default::Default;
use std::fs::File;
use std::path::{Path, PathBuf};

pub use crate::error::SteganoError;
pub use crate::media::image::CodecOptions;
pub use crate::result::Result;

/// wrap the low level data types that carries information
#[derive(Debug, Eq, PartialEq)]
pub enum MediaPrimitive {
    ImageColorChannel(u8),
    AudioSample(i16),
}

impl From<u8> for MediaPrimitive {
    fn from(value: u8) -> Self {
        MediaPrimitive::ImageColorChannel(value)
    }
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

pub struct SteganoCore;

impl SteganoCore {
    pub fn encoder() -> SteganoEncoder {
        SteganoEncoder::with_options(CodecOptions::default())
    }

    pub fn encoder_with_options(opts: CodecOptions) -> SteganoEncoder {
        SteganoEncoder::with_options(opts)
    }
}

pub trait Persist {
    fn save_as(&mut self, _: &Path) -> Result<()>;
}

pub struct SteganoEncoder {
    options: CodecOptions,
    codec_factory: Box<dyn PayloadCodecFactory>,
    // todo: change to Path
    target: Option<PathBuf>,
    carrier: Option<Media>,
    message: Message,
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
            options: CodecOptions::default(),
            codec_factory: Box::new(FabA),
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

    pub fn use_media(&mut self, input_file: impl AsRef<Path>) -> Result<&mut Self> {
        let path = input_file.as_ref();
        self.carrier = Some(Media::from_file(path)?);

        Ok(self)
    }

    pub fn save_as(&mut self, output_file: impl AsRef<Path>) -> &mut Self {
        self.target = Some(output_file.as_ref().to_owned());
        self
    }

    pub fn with_encryption<S: Into<String>>(&mut self, password: S) -> &mut Self {
        self.codec_factory = Box::new(FabS::new(password));
        self
    }

    pub fn add_message(&mut self, msg: &str) -> Result<&mut Self> {
        self.message
            .add_file_data("secret-message.txt", msg.as_bytes().to_vec())?;

        Ok(self)
    }

    pub fn add_file<P: AsRef<Path> + ?Sized>(&mut self, input_file: &P) -> Result<&mut Self> {
        {
            let _f = File::open(input_file).expect("Data file was not readable.");
        }
        self.message.add_file(input_file)?;

        Ok(self)
    }

    pub fn add_files<P: AsRef<Path>>(&mut self, input_files: &[P]) -> Result<&mut Self> {
        self.message.files = Vec::new();
        for f in input_files.iter() {
            self.add_file(f)?;
        }

        Ok(self)
    }

    pub fn hide_and_save(&mut self) -> Result<&mut Self> {
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
        if self.carrier.is_none() {
            return Err(SteganoError::CarrierNotSet);
        }

        if self.target.is_none() {
            return Err(SteganoError::TargetNotSet);
        }

        if let (Some(media), Some(target)) = (self.carrier.as_mut(), self.target.as_ref()) {
            let data = self.message.to_raw_data(&*self.codec_factory)?;
            media
                .hide_data(data, &self.options)?
                .save_as(Path::new(target))?;
        }

        Ok(self)
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use crate::commands::{unveil, unveil_raw};
    use std::fs;
    use std::io::Read;
    use tempfile::TempDir;

    const BASE_IMAGE: &str = "tests/images/Base.png";

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file() {
        SteganoEncoder::new().add_file("foofile").unwrap();
    }

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file_among_valid() {
        SteganoEncoder::new()
            .add_files(&["Cargo.toml", "foofile"])
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
        SteganoEncoder::new().save_as("/tmp/out-test-image.png");
    }

    #[test]
    fn should_hide_and_unveil_one_text_file_in_wav() -> Result<()> {
        let out_dir = TempDir::new()?;
        let secret_media_p = out_dir.path().join("secret.wav");
        let secret_media_f = secret_media_p.to_str().unwrap();

        SteganoEncoder::new()
            .add_file("Cargo.toml")?
            .use_media("tests/audio/plain/carrier-audio.wav")?
            .save_as(secret_media_f)
            .hide_and_save()?;

        let l = fs::metadata(secret_media_p.as_path())
            .expect("Secret media was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil(
            secret_media_p.as_path(),
            out_dir.path(),
            None,
            CodecOptions::default(),
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
            .add_file("Cargo.toml")?
            .use_media("tests/images/with_text/hello_world.png")?
            .save_as(image_with_secret)
            .hide_and_save()?;

        let l = fs::metadata(image_with_secret)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            None,
            CodecOptions::default(),
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
            Path::new("tests/images/with_text/hello_world.png"),
            expected_file.as_path(),
            None,
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
        let secret_to_hide = "tests/images/secrets/random_1666_byte.bin";
        let image_with_secret_path = out_dir.path().join("random_1666_byte.bin.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let expected_file = out_dir.path().join("random_1666_byte.bin");

        SteganoEncoder::new()
            .add_file(secret_to_hide)?
            .use_media(BASE_IMAGE)?
            .save_as(image_with_secret)
            .hide_and_save()?;

        let l = fs::metadata(image_with_secret)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            None,
            CodecOptions::default(),
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
        let secret_to_hide = "tests/images/secrets/zip_with_2_files.zip";
        let image_with_secret_path = out_dir.path().join("zip_with_2_files.zip.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let expected_file = out_dir.path().join("zip_with_2_files.zip");

        SteganoEncoder::new()
            .add_file(secret_to_hide)?
            .use_media(BASE_IMAGE)?
            .save_as(image_with_secret)
            .hide_and_save()?;

        assert_file_not_empty(image_with_secret);

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            None,
            CodecOptions::default(),
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
            Path::new("tests/images/with_attachment/Blah.txt.png"),
            out_dir.path(),
            None,
            CodecOptions::default(),
        )?;

        assert_eq_file_content(
            &decoded_secret,
            "tests/images/secrets/Blah.txt".as_ref(),
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
            Path::new("tests/images/with_attachment/Blah.txt__and__Blah-2.txt.png"),
            out_dir.path(),
            None,
            CodecOptions::default(),
        )?;
        assert_eq_file_content(
            &decoded_secret_1,
            "tests/images/secrets/Blah.txt".as_ref(),
            "Unveiled data file #1 did not match expected",
        );

        assert_eq_file_content(
            &decoded_secret_2,
            "tests/images/secrets/Blah-2.txt".as_ref(),
            "Unveiled data file #2 did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility_with_2_files_writing() -> Result<()> {
        let out_dir = TempDir::new()?;
        let image_with_secret_path = out_dir.path().join("Blah.txt.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let secret_to_hide = "tests/images/secrets/Blah.txt";

        SteganoEncoder::new()
            .use_media(BASE_IMAGE)?
            .add_file(secret_to_hide)?
            .save_as(image_with_secret)
            .hide_and_save()?;

        assert_file_not_empty(image_with_secret);

        unveil(
            image_with_secret_path.as_path(),
            out_dir.path(),
            None,
            CodecOptions::default(),
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

    pub const HELLO_WORLD_PNG: &str = "tests/images/with_text/hello_world.png";

    /// This image has some traits:
    /// --------------y-------------
    /// | 0,0 -> (0, 1, 2, 3 ) | 0,1 -> (4, 5, 6, 7 ) | ...
    /// | 1,0 -> (20,21,22,23) | 1,1 -> (24,25,26,27) | ...
    /// | 2,0 -> (40,41,42,43) | 2,1 -> (44,45,46,47) | ...
    /// x ...
    /// | ..
    /// | ..
    pub fn prepare_5x5_image() -> RgbaImage {
        ImageBuffer::from_fn(5, 5, |x, y| {
            let i = (4 * x + 20 * y) as u8;
            image::Rgba([i, i + 1, i + 2, i + 3])
        })
    }

    pub fn prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha() -> RgbaImage {
        let mut img = ImageBuffer::new(4, 6);
        let mut i = 0;
        for x in 0..(img.width() - 1) {
            for y in 0..(img.height() - 1) {
                let pi = img.get_pixel_mut(x, y);
                *pi = image::Rgba([i, i + 1, i + 2, 255]);
                i += 3;
            }
        }

        img
    }

    pub fn prepare_4x6_linear_growing_colors_regular_skipped_alpha() -> RgbaImage {
        let mut img = ImageBuffer::new(4, 6);
        let mut i = 0;
        for x in 0..img.width() {
            for y in 0..img.height() {
                let pi = img.get_pixel_mut(x, y);
                *pi = image::Rgba([i, i + 1, i + 2, 255]);
                i += 3;
            }
        }

        img
    }
}
