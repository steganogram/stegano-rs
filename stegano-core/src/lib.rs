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
//! use stegano_core::{SteganoCore, SteganoEncoder, Hide};
//!
//! SteganoCore::encoder()
//!     .hide_file("Cargo.toml")
//!     .use_carrier_image("../resources/plain/carrier-image.png")
//!     .write_to("/tmp/image-with-a-file-inside.png")
//!     .hide();
//! ```
//!
//! ## Unveil data from an image
//!
//! ```rust
//! use stegano_core::{SteganoCore, SteganoEncoder, SteganoDecoder, Hide, Unveil};
//!
//! SteganoCore::encoder()
//!     .hide_file("Cargo.toml")
//!     .use_carrier_image("../resources/plain/carrier-image.png")
//!     .write_to("/tmp/image-with-a-file-inside.png")
//!     .hide();
//!
//! SteganoCore::decoder()
//!     .use_source_image("/tmp/image-with-a-file-inside.png")
//!     .write_to_folder("/tmp/")
//!     .unveil();
//! ```
//!
//! [core]: ./struct.SteganoCore.html
//! [enc]: ./struct.SteganoEncoder.html
//! [dec]: ./struct.SteganoDecoder.html
//! [raw]: ./struct.SteganoRawDecoder.html

pub mod bit_iterator;
pub use bit_iterator::BitIterator;
pub mod lsb_codec;
pub use lsb_codec::LSBCodec;
pub mod message;
pub use message::*;
pub mod raw_message;
pub use raw_message::*;

pub mod carriers;
// pub mod encoder;
// pub mod lsb;
pub mod universal_decoder;
pub mod universal_encoder;
use image::*;
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;

/// wrap the low level data types that carries information
#[derive(Debug, PartialEq)]
pub enum CarrierItem {
    UnsignedByte(u8),
    SignedTwoByte(i16),
}

pub struct SteganoCore {}

impl SteganoCore {
    pub fn encoder() -> SteganoEncoder {
        SteganoEncoder::new()
    }

    pub fn decoder() -> SteganoDecoder {
        SteganoDecoder::new()
    }

    pub fn raw_decoder() -> SteganoRawDecoder {
        SteganoRawDecoder::new()
    }
}

pub trait Hide {
    // TODO should return Result<()>
    fn hide(&mut self) -> &Self;
}

pub trait Unveil {
    // TODO should return Result<()>
    fn unveil(&mut self) -> &mut Self;
}

pub struct SteganoEncoder {
    target: Option<String>,
    carrier: Option<RgbaImage>,
    message: Message,
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
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

    pub fn use_carrier_image(&mut self, input_file: &str) -> &mut Self {
        self.carrier = Some(
            image::open(Path::new(input_file))
                .unwrap_or_else(|_| {
                    panic!(
                        "Carrier image '{}' was not readable in {}.",
                        input_file,
                        std::env::current_dir()
                            .expect("CWD was not set")
                            .to_str()
                            .expect("Path could not be unwrapped")
                    )
                })
                .to_rgba(),
        );

        self
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

    pub fn hide_file(&mut self, input_file: &str) -> &mut Self {
        {
            let _f = File::open(input_file).expect("Data file was not readable.");
        }
        self.message.add_file(&input_file.to_string());

        self
    }

    pub fn hide_files(&mut self, input_files: Vec<&str>) -> &mut Self {
        self.message.files = Vec::new();
        input_files.iter().for_each(|&f| {
            self.hide_file(f);
        });

        self
    }

    pub fn force_content_version(&mut self, c: ContentVersion) -> &mut Self {
        self.message.header = c;

        self
    }
}

impl Hide for SteganoEncoder {
    fn hide(&mut self) -> &Self {
        let (x, y) = (&self.carrier).as_ref().unwrap().dimensions();
        let mut img = self.carrier.as_mut().unwrap();
        let mut dec = LSBCodec::new(&mut img);

        let buf: Vec<u8> = (&self.message).into();
        dec.write_all(&buf[..])
            .expect("Failed to hide data in carrier image.");

        if self.message.header == ContentVersion::V2 {
            let mut space_to_fill = ((x * y * 3) / 8) as usize;
            space_to_fill -= buf.len();

            for _ in 0..space_to_fill {
                dec.write_all(&[0])
                    .expect("Failed to terminate version 2 content.");
            }
        }

        self.carrier
            .as_mut()
            .expect("Image was not there for saving.")
            .save(self.target.as_ref().unwrap())
            .expect("Failed to save final image");

        self
    }
}

pub struct SteganoDecoder {
    input: Option<RgbaImage>,
    output: Option<String>,
}

impl Default for SteganoDecoder {
    fn default() -> Self {
        Self {
            output: None,
            input: None,
        }
    }
}

impl SteganoDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        let img = image::open(input_file)
            .expect("Input image is not readable.")
            .to_rgba();

        self.input = Some(img);

        self
    }

    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        {
            let _f = File::create(output_file.to_string()).expect("Output cannot be created.");
        }
        self.output = Some(output_file.to_string());

        self
    }

    pub fn write_to_folder(&mut self, output_folder: &str) -> &mut Self {
        match DirBuilder::new().recursive(true).create(output_folder) {
            Ok(_) => {}
            Err(ref e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    eprintln!("Cannot create output folder: {}", e);
                }
            }
        }

        self.output = Some(output_folder.to_string());

        self
    }
}

impl Unveil for SteganoDecoder {
    fn unveil(&mut self) -> &mut Self {
        let mut dec = LSBCodec::new(self.input.as_mut().unwrap());
        let msg = Message::of(&mut dec);
        let mut files = msg.files;

        if let Some(text) = msg.text {
            files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
        }

        (&files)
            .iter()
            .map(|(file_name, buf)| {
                let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

                (file, buf)
            })
            .for_each(|(file_name, buf)| {
                let target_file = Path::new(self.output.as_ref().unwrap()).join(file_name);
                let mut target_file =
                    File::create(target_file).expect("Cannot create target output file");

                let mut c = Cursor::new(buf);
                std::io::copy(&mut c, &mut target_file)
                    .expect("Failed to write data to final target file.");
            });

        self
    }
}

pub struct SteganoRawDecoder {
    inner: SteganoDecoder,
}

impl Default for SteganoRawDecoder {
    fn default() -> Self {
        Self {
            inner: SteganoDecoder::new(),
        }
    }
}

impl SteganoRawDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.inner.use_source_image(input_file);

        self
    }

    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        self.inner.write_to_file(output_file);

        self
    }
}

impl Unveil for SteganoRawDecoder {
    fn unveil(&mut self) -> &mut Self {
        let mut dec = LSBCodec::new(self.inner.input.as_mut().unwrap());
        let mut msg = RawMessage::of(&mut dec);
        let target_file = self.inner.output.as_ref().unwrap();
        let mut target_file = File::create(target_file).expect("Cannot open output file.");

        let mut c = Cursor::new(&mut msg.content);
        std::io::copy(&mut c, &mut target_file)
            .expect("Failed to write RawMessage to target file.");

        self
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    const BASE_IMAGE: &str = "../resources/Base.png";

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file() {
        SteganoEncoder::new().hide_file("foofile");
    }

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file_among_valid() {
        SteganoEncoder::new().hide_files(vec!["Cargo.toml", "foofile"]);
    }

    #[test]
    #[should_panic(expected = "Carrier image 'random_file.png' was not readable in")]
    fn should_panic_for_invalid_carrier_image_file() {
        SteganoEncoder::new().use_carrier_image("random_file.png");
    }

    #[test]
    fn should_accept_a_png_as_target_file() {
        SteganoEncoder::new().write_to("/tmp/out-test-image.png");
    }

    #[test]
    fn should_hide_and_unveil_one_text_file() -> Result<()> {
        let out_dir = TempDir::new("hello_world.png")?;
        let image_with_secret_path = out_dir.path().join("secret.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();

        SteganoEncoder::new()
            .hide_file("Cargo.toml")
            .use_carrier_image("../resources/with_text/hello_world.png")
            .write_to(image_with_secret)
            .hide();

        let l = fs::metadata(image_with_secret)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        SteganoDecoder::new()
            .use_source_image(image_with_secret)
            .write_to_folder(out_dir.path().to_str().unwrap())
            .unveil();

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
        let out_dir = TempDir::new("hello_world.png")?;
        let expected_file = out_dir.path().join("hello_world.bin");
        let raw_decoded_secret = expected_file.to_str().unwrap();

        SteganoRawDecoder::new()
            .use_source_image("../resources/with_text/hello_world.png")
            .write_to_file(raw_decoded_secret)
            .unveil();

        let l = fs::metadata(raw_decoded_secret)
            .expect("Output file was not written.")
            .len();

        // TODO content verification needs to be done as well
        assert_ne!(l, 0, "Output raw data file was empty.");

        Ok(())
    }

    #[test]
    fn should_hide_and_unveil_a_binary_file() -> Result<()> {
        let out_dir = TempDir::new("random_1666_byte.bin.png")?;
        let secret_to_hide = "../resources/secrets/random_1666_byte.bin";
        let image_with_secret_path = out_dir.path().join("random_1666_byte.bin.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let expected_file = out_dir.path().join("random_1666_byte.bin");

        SteganoEncoder::new()
            .hide_file(secret_to_hide)
            .use_carrier_image(BASE_IMAGE)
            .write_to(image_with_secret)
            .hide();

        let l = fs::metadata(image_with_secret)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        SteganoDecoder::new()
            .use_source_image(image_with_secret)
            .write_to_folder(out_dir.path().to_str().unwrap())
            .unveil();

        assert_eq_file_content(
            &expected_file,
            secret_to_hide.as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_hide_and_unveil_a_zip_file() -> Result<()> {
        let out_dir = TempDir::new("zip_with_2_files.zip.png")?;
        let secret_to_hide = "../resources/secrets/zip_with_2_files.zip";
        let image_with_secret_path = out_dir.path().join("zip_with_2_files.zip.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let expected_file = out_dir.path().join("zip_with_2_files.zip");

        SteganoEncoder::new()
            .hide_file(secret_to_hide)
            .use_carrier_image(BASE_IMAGE)
            .write_to(image_with_secret)
            .hide();

        assert_file_not_empty(image_with_secret);

        SteganoDecoder::new()
            .use_source_image(image_with_secret)
            .write_to_folder(out_dir.path().to_str().unwrap())
            .unveil();

        assert_eq_file_content(
            &expected_file,
            secret_to_hide.as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility() -> Result<()> {
        let out_dir = TempDir::new("Blah.txt.png")?;
        let decoded_secret = out_dir.path().join("Blah.txt");

        SteganoDecoder::new()
            .use_source_image("../resources/with_attachment/Blah.txt.png")
            .write_to_folder(out_dir.path().to_str().unwrap())
            .unveil();

        assert_eq_file_content(
            &decoded_secret,
            "../resources/secrets/Blah.txt".as_ref(),
            "Unveiled data did not match expected",
        );

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility_with_2_files_reading() -> Result<()> {
        let out_dir = TempDir::new("Blah.txt__and__Blah-2.txt.png")?;
        let output_folder = out_dir.path().to_str().unwrap();
        let decoded_secret_1 = out_dir.path().join("Blah.txt");
        let decoded_secret_2 = out_dir.path().join("Blah-2.txt");

        SteganoDecoder::new()
            .use_source_image("../resources/with_attachment/Blah.txt__and__Blah-2.txt.png")
            .write_to_folder(output_folder)
            .unveil();

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
        let out_dir = TempDir::new("out-dir")?;
        let output_folder = out_dir.path().to_str().unwrap();
        let image_with_secret_path = out_dir.path().join("Blah.txt.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();
        let secret_to_hide = "../resources/secrets/Blah.txt";

        SteganoEncoder::new()
            .force_content_version(ContentVersion::V2)
            .use_carrier_image(BASE_IMAGE)
            .hide_file(secret_to_hide)
            .write_to(image_with_secret)
            .hide();

        assert_file_not_empty(image_with_secret);

        SteganoDecoder::new()
            .use_source_image(image_with_secret)
            .write_to_folder(output_folder)
            .unveil();

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
