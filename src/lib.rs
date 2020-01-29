extern crate hex_literal;

pub mod bit_iterator;

pub use bit_iterator::BitIterator;

pub mod lsb_codec;

pub use lsb_codec::LSBCodec;

pub mod message;

pub use message::*;

pub mod raw_message;

pub use raw_message::*;

use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use image::*;

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
            let _f = File::open(input_file)
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

    pub fn force_content_version(&mut self, c: ContentVersion) -> &mut Self {
        self.message.header = c;

        self
    }
}

impl Hide for SteganoEncoder {
    fn hide(&mut self) -> &Self {
        let mut img = self.carrier.as_mut().unwrap();
        let mut dec = LSBCodec::new(&mut img);

        let buf: Vec<u8> = (&self.message).into();
        dec.write_all(&buf[..])
            .expect("Failed to hide data in carrier image.");

        self.carrier.as_mut()
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
        let img = image::open(input_file)
            .expect("Input image is not readable.")
            .to_rgba();

        self.input = Some(img);

        self
    }

    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        {
            let _f = File::create(output_file.to_string())
                .expect("Output cannot be created.");
        }
        self.output = Some(output_file.to_string());

        self
    }

    pub fn write_to_folder(&mut self, output_folder: &str) -> &mut Self {
        match DirBuilder::new()
            .recursive(true)
            .create(output_folder) {
            Ok(_) => {},
            Err(ref e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    eprintln!("Cannot create output folder: {}", e);
                }
            },
        }

        self.output = Some(output_folder.to_string());

        self
    }
}

impl Unveil for SteganoDecoder {
    fn unveil(&mut self) -> &mut Self {
        let mut dec = LSBCodec::new(self.input.as_mut().unwrap());
        let msg = Message::of(&mut dec);

        (&msg.files)
            .iter()
            .map(|b| b)
            .for_each(|(file_name, buf)| {
                let file_name = file_name.split('/')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .to_string();
                let target_file = format!("{}/{}", self.output.as_ref().unwrap(), file_name);
                let mut target_file = File::create(target_file)
                    .expect("Cannot create target output file");

                let mut c = Cursor::new(buf);
                std::io::copy(&mut c, &mut target_file).
                    expect("Failed to write data to final target file.");
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
        let target_file = self.inner.output.as_ref().unwrap();
        let mut target_file = File::create(target_file)
            .expect("Cannot open output file.");

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
            .write_to_folder("/tmp/Cargo.toml.d")
            .unveil();

        let expected = fs::metadata("Cargo.toml")
            .expect("Source file is not available.")
            .len();
        let given = fs::metadata("/tmp/Cargo.toml.d/Cargo.toml")
            .expect("Output image was not written.")
            .len();

        assert_eq!(given, expected, "Unveiled file size differs to the original");
    }

    #[test]
    fn should_raw_unveil_a_message() {
        // FIXME: there no zip, just plain raw string is contained
        SteganoRawDecoder::new()
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
    fn should_hide_and_unveil_a_binary_file() -> Result<()> {
        let out_dir = TempDir::new("random_1666_byte.bin.png")?;
        let secret_to_hide = "resources/secrets/random_1666_byte.bin";
        let image_with_secret_path = out_dir.path().join("random_1666_byte.bin.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();

        SteganoEncoder::new()
            .hide_file(secret_to_hide)
            .use_carrier_image("resources/Base.png")
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

        let expected_file_size = fs::metadata(secret_to_hide)
            .expect("Source file is not available.")
            .len();

        let expected_file = out_dir.path().join("random_1666_byte.bin");
        let given = fs::metadata(expected_file.to_str().unwrap())
            .expect("Unveiled file was not written.")
            .len();
        assert_eq!(expected_file_size - given, 0, "Unveiled file size differs to the original");
        // TODO: implement content matching

        Ok(())
    }

    #[test]
    fn should_hide_and_unveil_a_zip_file() -> Result<()> {
        let out_dir = TempDir::new("zip_with_2_files.zip.png")?;
        let secret_to_hide = "resources/secrets/zip_with_2_files.zip";
        let image_with_secret_path = out_dir.path().join("zip_with_2_files.zip.png");
        let image_with_secret = image_with_secret_path.to_str().unwrap();

        SteganoEncoder::new()
            .hide_file(secret_to_hide)
            .use_carrier_image("resources/Base.png")
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

        let expected = fs::metadata(secret_to_hide)
            .expect("Source file is not available.")
            .len();

        let expected_file = out_dir.path().join("zip_with_2_files.zip");
        let given = fs::metadata(expected_file.to_str().unwrap())
            .expect("Unveiled file was not written.")
            .len();
        assert_eq!(expected - given, 0, "Unveiled file size differs to the original");

        Ok(())
    }

    #[test]
    fn should_ensure_content_v2_compatibility() {
        SteganoDecoder::new()
            .use_source_image("resources/with_attachment/Blah.txt.png")
            .write_to_folder("/tmp")
            .unveil();

        let mut given_content = Vec::new();
        File::open("/tmp/Blah.txt")
            .expect("Output file was not openable.")
            .read_to_end(&mut given_content)
            .expect("Output file was not readable.");

        let mut expected_content = Vec::new();
        File::open("resources/secrets/Blah.txt")
            .expect("Fixture file was not openable.")
            .read_to_end(&mut expected_content)
            .expect("Fixture file was not readable.");

        assert_eq!(given_content, expected_content, "Unveiled data did not match expected");
    }

    #[test]
    fn should_ensure_content_v2_compatibility_with_2_files() {
        let output_folder = "/tmp/Blah.txt__and__Blah-2.txt.d";
        SteganoDecoder::new()
            .use_source_image("resources/with_attachment/Blah.txt__and__Blah-2.txt.png")
            .write_to_folder(output_folder)
            .unveil();

        let mut given_content = Vec::new();
        File::open(format!("{}/Blah.txt", output_folder))
            .expect("Output file #1 was not openable.")
            .read_to_end(&mut given_content)
            .expect("Output file #1 was not readable.");

        let mut expected_content = Vec::new();
        File::open("resources/secrets/Blah.txt")
            .expect("Fixture file was not openable.")
            .read_to_end(&mut expected_content)
            .expect("Fixture file was not readable.");

        assert_eq!(given_content, expected_content, "Unveiled data file #1 did not match expected");

        let mut given_content = Vec::new();
        File::open(format!("{}/Blah-2.txt", output_folder))
            .expect("Output file #1 was not openable.")
            .read_to_end(&mut given_content)
            .expect("Output file #1 was not readable.");

        let mut expected_content = Vec::new();
        File::open("resources/secrets/Blah-2.txt")
            .expect("Fixture file was not openable.")
            .read_to_end(&mut expected_content)
            .expect("Fixture file was not readable.");

        assert_eq!(given_content, expected_content, "Unveiled data file #2 did not match expected");
    }
}