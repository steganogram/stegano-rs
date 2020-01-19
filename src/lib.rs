#[macro_use]
extern crate hex_literal;

pub mod bit_iterator;

pub use bit_iterator::BitIterator;

pub mod lsb_codec;

pub use lsb_codec::LSBCodec;

pub mod message;

pub use message::*;

pub mod raw_message;

pub use raw_message::*;

use bitstream_io::{LittleEndian, BitReader};
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use image::*;
use std::io;
use std::borrow::BorrowMut;

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
    target_image: Option<RgbaImage>,
    carrier: Option<RgbaImage>,
    message: Message,
    x: u32,
    y: u32,
    c: usize,
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
            target: None,
            target_image: None,
            carrier: None,
            message: Message::empty(),
            x: 0,
            y: 0,
            c: 0,
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
                .expect("Carrier image was not readable.")
                .to_rgba()
        );

        self
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Self {
        self.target = Some(output_file.to_string());
        self
    }

    pub fn hide_message(&mut self, msg: &str) -> &mut Self {
        self.message.text = Some(msg.to_string());

        self
    }

    pub fn hide_file(&mut self, input_file: &str) -> &mut Self {
        {
            let f = File::open(input_file)
                .expect("Data file was not readable.");
        }
        self.message.add_file(&input_file.to_string());

        self
    }

    pub fn hide_files(&mut self, input_files: Vec<&str>) -> &mut Self {
        self.message.files = Vec::new();
        input_files
            .iter()
            .for_each(|&f| {
                self.hide_file(f);
            });

        self
    }
}

impl Hide for SteganoEncoder {
    fn hide(&mut self) -> &Self {
        let mut img = self.carrier.as_mut().unwrap();
        let mut dec = LSBCodec::new(&mut img);

        let mut buf: Vec<u8> = (&self.message).into();
        dec.write_all(&buf[..]);

        self.carrier.as_mut()
            .expect("Image was not there for saving.")
            .save(self.target.as_ref().unwrap());

        self
    }
}

pub struct SteganoDecoder {
    input: Option<RgbaImage>,
    output: Option<File>,
}

impl Default for SteganoDecoder
{
    fn default() -> Self {
        Self {
            output: None,
            input: None,
        }
    }
}

impl SteganoDecoder
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        let mut img = image::open(input_file)
            .expect("Input image is not readable.")
            .to_rgba();

        self.input = Some(img);

        self
    }

    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        let file = File::create(output_file.to_string())
            .expect("Output cannot be created.");
        self.output = Some(file);

        self
    }
}

impl Unveil for SteganoDecoder {
    fn unveil(&mut self) -> &mut Self {
        let mut dec = LSBCodec::new(self.input.as_mut().unwrap());
        let mut msg = Message::of(&mut dec);

        if msg.files.len() > 1 {
            unimplemented!("More than one content file is not yet supported.")
        }

        (&msg.files)
            .iter()
            .map(|b| b.as_ref())
            .for_each(|(file_name, buf)| {
                // TODO for now we have only one target file
//                        let mut target_file = File::create(format!("/tmp/{}", file_name))
//                            .expect("File was not writeable");
                let mut target_file = self.output.as_mut().unwrap();

                let mut c = Cursor::new(buf);
                std::io::copy(&mut c, &mut target_file);
            });

        self
    }
}

pub struct SteganoRawDecoder {
    inner: SteganoDecoder,
}

impl Default for SteganoRawDecoder
{
    fn default() -> Self {
        Self {
            inner: SteganoDecoder::new(),
        }
    }
}

impl SteganoRawDecoder
{
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
        let mut target_file = self.inner.output.as_mut().unwrap();

        let mut c = Cursor::new(&mut msg.content);
        std::io::copy(&mut c, &mut target_file);

        self
    }
}


#[cfg(test)]
mod e2e_tests {
    use super::*;
    use std::fs;

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
    #[should_panic(expected = "Carrier image was not readable.")]
    fn should_panic_for_invalid_carrier_image_file() {
        SteganoEncoder::new().use_carrier_image("random_file.png");
    }

    #[test]
    fn should_accecpt_a_png_as_target_file() {
        SteganoEncoder::new().write_to("/tmp/out-test-image.png");
    }

    #[test]
    fn should_hide_and_unveil_one_text_file() {
        SteganoEncoder::new()
            .hide_file("Cargo.toml")
            .use_carrier_image("resources/with_text/hello_world.png")
            .write_to("/tmp/out-test-image.png")
            .hide();

        let l = fs::metadata("/tmp/out-test-image.png")
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        SteganoDecoder::new()
            .use_source_image("/tmp/out-test-image.png")
            .write_to_file("/tmp/Cargo.toml")
            .unveil();

        let expected = fs::metadata("Cargo.toml")
            .expect("Source file is not available.")
            .len();
        let given = fs::metadata("/tmp/Cargo.toml")
            .expect("Output image was not written.")
            .len();

        assert_eq!(given, expected, "Unveiled file size differs to the original");
    }

    #[test]
    fn should_raw_unveil_a_message() {
        // FIXME: there no zip, just plain raw string is contained
        let dec = SteganoRawDecoder::new()
            .use_source_image("resources/with_text/hello_world.png")
            .write_to_file("/tmp/HelloWorld.bin")
            .unveil();

        let l = fs::metadata("/tmp/HelloWorld.bin")
            .expect("Output file was not written.")
            .len();

        // TODO content verification needs to be done as well
        assert_ne!(l, 0, "Output raw data file was empty.");
    }

    #[test]
    fn should_hide_and_unveil_a_binary_file() {
        let out = "/tmp/random_1666_byte.bin.png";
        let input = "resources/secrets/random_1666_byte.bin";
        SteganoEncoder::new()
            .hide_file(input)
            .use_carrier_image("resources/Base.png")
            .write_to(out)
            .hide();

        let l = fs::metadata(out)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");
        let target = "/tmp/random_1666_byte.bin.decoded";

        SteganoDecoder::new()
            .use_source_image(out)
            .write_to_file(target)
            .unveil();

        let expected = fs::metadata(input)
            .expect("Source file is not available.")
            .len();

        let given = fs::metadata(target)
            .expect("Unveiled file was not written.")
            .len();
        assert_eq!(expected - given, 0, "Unveiled file size differs to the original");
        // TODO: implement content matching
    }

    #[test]
    fn should_hide_and_unveil_a_zip_file() {
        let input = "resources/secrets/zip_with_2_files.zip";
        let out = "/tmp/zip_with_2_files.zip.png";
        let target = "/tmp/zip_with_2_files.zip.decoded";

        SteganoEncoder::new()
            .hide_file(input)
            .use_carrier_image("resources/Base.png")
            .write_to(out)
            .hide();

        let l = fs::metadata(out)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        SteganoDecoder::new()
            .use_source_image(out)
            .write_to_file(target)
            .unveil();

        let expected = fs::metadata(input)
            .expect("Source file is not available.")
            .len();

        let given = fs::metadata(target)
            .expect("Unveiled file was not written.")
            .len();
        assert_eq!(expected - given, 0, "Unveiled file size differs to the original");
        // TODO: implement content matching
    }
}