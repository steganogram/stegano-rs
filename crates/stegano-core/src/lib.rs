#![cfg_attr(feature = "benchmarks", feature(test))]
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

//! # Stegano Core API
//!
//! There are 3 main API entry points:
//! - `hide`
//! - `unveil`
//! - `unveil_raw`
//!
//! Each of these APIs offer a builder pattern to configure the operation.
//!
//! ## Usage Examples
//!
//! ### Hide data inside an image
//!
//! ```rust
//! use tempfile::tempdir;
//!
//! let temp_dir = tempdir().expect("Failed to create temporary directory");
//!
//! stegano_core::api::hide::prepare()   // prepares the hide operation with some default options
//!     .with_message("Hello, World!")   // this message will be hidden inside the image
//!     .with_file("Cargo.toml")         // this file will be hidden inside the image
//!     .using_password("SuperSecret42") // encrypts all the data with this password
//!     .with_image("tests/images/plain/carrier-image.png")     // this is the carrier image, it's read-only
//!     .with_output(temp_dir.path().join("image-with-a-file-inside.png"))  // this is the output image
//!     .execute()
//!     .expect("Failed to hide file in image");
//! ```
//!
//! ### Unveil data from an image
//!
//! ```rust
//! use tempfile::tempdir;
//!
//! let temp_dir = tempdir().expect("Failed to create temporary directory");
//!
//! stegano_core::api::unveil::prepare()
//!     .from_secret_file("tests/images/encrypted/hello_world.png")
//!     .using_password("Secret42")
//!     .into_output_folder(temp_dir.path())
//!     .execute()
//!     .expect("Failed to unveil message from image");
//! ```

#[cfg(feature = "benchmarks")]
extern crate test;

mod error;
mod message;
mod raw_message;
mod result;
mod universal_decoder;
mod universal_encoder;

pub(crate) mod media;

pub mod api;

pub use crate::error::SteganoError;
pub use crate::result::Result;

use std::default::Default;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::media::payload::{FabA, FabS, PayloadCodecFactory};
use crate::media::{CodecOptions, F5CodecOptions, LsbCodecOptions, Media, Persist, DEFAULT_JPEG_QUALITY};
use crate::message::Message;
use crate::raw_message::RawMessage;

pub struct SteganoEncoder {
    codec_factory: Box<dyn PayloadCodecFactory>,
    target: Option<PathBuf>,
    carrier: Option<Media>,
    message: Message,
    color_channel_step_increment: Option<usize>,
    jpeg_quality: Option<u8>,
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
            codec_factory: Box::new(FabA),
            target: None,
            carrier: None,
            message: Message::empty(),
            color_channel_step_increment: None,
            jpeg_quality: None,
        }
    }
}

// todo: check if this layer of abstraction is necessary, the api::hide module could be the entry point
impl SteganoEncoder {
    pub fn new() -> Self {
        Self::default()
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

    pub fn with_color_step_increment(&mut self, step: usize) -> &mut Self {
        self.color_channel_step_increment = Some(step);
        self
    }

    pub fn with_jpeg_quality(&mut self, quality: u8) -> &mut Self {
        self.jpeg_quality = Some(quality);
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
        let Some(ref media) = self.carrier else {
            return Err(SteganoError::CarrierNotSet);
        };

        let Some(ref target) = self.target else {
            return Err(SteganoError::TargetNotSet);
        };

        // Determine codec from target path extension
        let codec_opts = self.determine_codec_options(target)?;

        // Encode the message
        let data = self.message.to_raw_data(&*self.codec_factory)?;

        // Hide data and save
        let mut encoded = media.hide_data(data, &codec_opts)?;
        encoded.save_as(target)?;

        Ok(self)
    }

    /// Determine codec options based on target file extension
    fn determine_codec_options(&self, target: &Path) -> Result<CodecOptions> {
        let ext = target
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "png" => {
                let mut opts = LsbCodecOptions::default();
                if let Some(step) = self.color_channel_step_increment {
                    opts.color_channel_step_increment = step;
                }
                Ok(CodecOptions::Lsb(opts))
            }
            "jpg" | "jpeg" => {
                // Derive F5 seed from password if available
                let seed = self.codec_factory.password().map(|p| p.as_bytes().to_vec());
                let quality = self.jpeg_quality.unwrap_or(DEFAULT_JPEG_QUALITY);
                Ok(CodecOptions::F5(F5CodecOptions { seed, quality }))
            }
            "wav" => Ok(CodecOptions::AudioLsb),
            _ => Err(SteganoError::UnsupportedMedia),
        }
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use crate::api;
    use api::unveil;
    use media::MediaPrimitiveMut;
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

        SteganoEncoder::new()
            .add_file("Cargo.toml")?
            .use_media("tests/audio/plain/carrier-audio.wav")?
            .save_as(&secret_media_p)
            .hide_and_save()?;

        let l = fs::metadata(&secret_media_p)
            .expect("Secret media was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil::prepare()
            .from_secret_file(&secret_media_p)
            .into_output_folder(&out_dir)
            .execute()?;

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

        SteganoEncoder::new()
            .add_file("Cargo.toml")?
            .use_media("tests/images/with_text/hello_world.png")?
            .save_as(&image_with_secret_path)
            .hide_and_save()?;

        let l = fs::metadata(&image_with_secret_path)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil::prepare()
            .from_secret_file(&image_with_secret_path)
            .into_output_folder(&out_dir)
            .execute()?;

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

        api::unveil_raw::prepare()
            .from_secret_file("tests/images/with_text/hello_world.png")
            .into_raw_file(&expected_file)
            .execute()?;

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
        let expected_file = out_dir.path().join("random_1666_byte.bin");

        SteganoEncoder::new()
            .add_file(secret_to_hide)?
            .use_media(BASE_IMAGE)?
            .save_as(&image_with_secret_path)
            .hide_and_save()?;

        let l = fs::metadata(&image_with_secret_path)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        unveil::prepare()
            .from_secret_file(&image_with_secret_path)
            .into_output_folder(&out_dir)
            .execute()?;

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

        unveil::prepare()
            .from_secret_file(&image_with_secret_path)
            .into_output_folder(&out_dir)
            .execute()?;

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

        unveil::prepare()
            .from_secret_file("tests/images/with_attachment/Blah.txt.png")
            .into_output_folder(&out_dir)
            .execute()?;

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

        unveil::prepare()
            .from_secret_file("tests/images/with_attachment/Blah.txt__and__Blah-2.txt.png")
            .into_output_folder(&out_dir)
            .execute()?;

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
        let secret_to_hide = "tests/images/secrets/Blah.txt";

        SteganoEncoder::new()
            .use_media(BASE_IMAGE)?
            .add_file(secret_to_hide)?
            .save_as(&image_with_secret_path)
            .hide_and_save()?;

        assert_file_not_empty(&image_with_secret_path);

        unveil::prepare()
            .from_secret_file(&image_with_secret_path)
            .into_output_folder(&out_dir)
            .execute()?;

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

    fn assert_file_not_empty(image_with_secret: impl AsRef<Path>) {
        let l = fs::metadata(image_with_secret.as_ref())
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
