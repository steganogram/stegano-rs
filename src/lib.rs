#[macro_use]
extern crate hex_literal;

pub mod bit_iterator;

pub use bit_iterator::BitIterator;

pub mod decoder;

pub use decoder::*;

pub mod byte_reader;

pub use byte_reader::*;

pub mod filter_reader;

pub use filter_reader::*;

pub mod codec;

pub use codec::Codec;

pub mod decipher;

use bitstream_io::{LittleEndian, BitReader};
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use image::*;
use std::io;
use std::borrow::BorrowMut;

pub struct SteganoCore {
    target: Option<String>,
    target_image: Option<RgbaImage>,
    carrier: Option<image::DynamicImage>,
    files_to_hide: Vec<String>,
    x: u32,
    y: u32,
    c: usize,
}

pub trait Hide {
    fn hide(&mut self) -> &Self;
}

pub trait Unveil {
    fn unveil(&mut self) -> &mut Self;
}

impl Default for SteganoCore {
    fn default() -> Self {
        Self {
            target: None,
            target_image: None,
            carrier: None,
            files_to_hide: Vec::new(),
            x: 0,
            y: 0,
            c: 0,
        }
    }
}

impl SteganoCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_carrier_image(&mut self, input_file: &str) -> &mut Self {
        self.carrier = Some(
            image::open(Path::new(input_file))
                .expect("Carrier image was not readable."));
        self
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Self {
        self.target = Some(output_file.to_string());
        self
    }

    pub fn hide_message(&mut self, msg: &str) -> &mut Self {
        unimplemented!("TODO hide_message not implemented");
        self
    }

    pub fn hide_file(&mut self, input_file: &str) -> &mut Self {
        {
            let f = File::open(input_file)
                .expect("Data file was not readable.");
        }
        self.files_to_hide.push(input_file.to_string());

        self
    }

    pub fn hide_files(&mut self, input_files: Vec<&str>) -> &mut Self {
        self.files_to_hide = Vec::new();
        input_files
            .iter()
            .for_each(|&f| {
                self.hide_file(f);
            });

        self
    }
}

impl Hide for SteganoCore {
    fn hide(&mut self) -> &Self {
        let mut files = self.files_to_hide.clone();
        let mut codec = Codec::encoder(self.borrow_mut());
        let mut buf = Vec::new();

        {
            let mut w = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(w);

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            files
                .iter()
                .map(|f| (f, File::open(f).expect("Data file was not readable.")))
                // TODO instead of filtering, accepting directories would be nice
                .filter(|(name, f)| f.metadata().unwrap().is_file())
                .for_each(|(name, mut f)| {
                    zip.start_file(name, options).
                        expect("start zip file failed.");

                    std::io::copy(&mut f, &mut zip)
                        .expect("Failed to copy data to the zip entry");
                });

            zip.finish().expect("finish zip failed.");
        }

        let mut w = std::io::Cursor::new(&mut buf);
        std::io::copy(&mut w, &mut codec)
            .expect("Failed to copy from zip to codec.");

        codec.flush()
            .expect("Failed to flush the codec.");

        self
    }
}

impl Write for SteganoCore {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        #[inline]
        fn bit_wave(byte: u8, bit: io::Result<bool>) -> u8 {
            let byt = match bit {
                // TODO here we need some configurability, to prevent 0 writing on demand
//                Err(_) => 0,
                Err(_) => byte,
                Ok(byt) => if byt { 1 } else { 0 }
            };
            (byte & 0xFE) | byt
        }

        let carrier = self.carrier.as_ref().unwrap();
        let (width, height) = carrier.dimensions();
        let bytes_to_write = buf.len();
        match self.target_image {
            None => {
                self.target_image = Some(ImageBuffer::new(width, height));
            }
            _ => {}
        }
        let mut bit_iter = BitReader::endian(
            Cursor::new(buf),
            LittleEndian,
        );

        let mut bits_written = 0;
        let mut bytes_written = 0;
        for x in self.x..width {
            for y in self.y..height {
                let image::Rgba(mut rgba) = carrier.get_pixel(x, y);
                for c in self.c..3 as usize {
                    if bytes_written >= bytes_to_write {
                        self.x = x;
                        self.y = y;
                        self.c = c;
                        self.target_image.as_mut()
                            .expect("Target Image was not present.")
                            .put_pixel(x, y, Rgba(rgba));
                        return Ok(bytes_written);
                    }

                    rgba[c] = bit_wave(rgba[c], bit_iter.read_bit());
                    bits_written += 1;
                    if bits_written % 8 == 0 {
                        bytes_written = (bits_written / 8) as usize;
                    }
                }
                self.target_image.as_mut()
                    .unwrap()
                    .put_pixel(x, y, Rgba(rgba));
                if self.c > 0 {
                    self.c = 0;
                }
            }
            if self.y > 0 {
                self.y = 0;
            }
        }
        self.x = width;

        Ok(bytes_written)
    }

    fn flush(&mut self) -> Result<()> {
        // copy the other pixel as they are..
        {
            let (width, height) = self.carrier.as_ref().unwrap().dimensions();
            for x in self.x..width {
                for y in self.y..height {
                    let pixel = self.carrier.as_ref().unwrap().get_pixel(x, y);
                    self.target_image.as_mut()
                        .unwrap()
                        .put_pixel(x, y, pixel);
                }
                if self.y > 0 {
                    self.y = 0;
                }
            }
        }

        self.target_image.as_mut()
            .expect("Image was not there for saving.")
            .save(self.target.as_ref().unwrap())
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use std::fs;

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file() {
        SteganoCore::new().hide_file("foofile");
    }

    #[test]
    #[should_panic(expected = "Data file was not readable.")]
    fn should_panic_on_invalid_data_file_among_valid() {
        SteganoCore::new().hide_files(vec!["Cargo.toml", "foofile"]);
    }

    #[test]
    #[should_panic(expected = "Carrier image was not readable.")]
    fn should_panic_for_invalid_carrier_image_file() {
        SteganoCore::new().use_carrier_image("random_file.png");
    }

    #[test]
    fn should_accecpt_a_png_as_target_file() {
        SteganoCore::new().write_to("/tmp/out-test-image.png");
    }

    #[test]
    fn should_hide_and_unveil_one_text_file() {
        SteganoCore::new()
            .hide_file("Cargo.toml")
            .use_carrier_image("resources/with_text/hello_world.png")
            .write_to("/tmp/out-test-image.png")
            .hide();

        let l = fs::metadata("/tmp/out-test-image.png")
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");

        FileOutputDecoder::new()
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
    #[ignore]
    fn should_raw_unveil_a_message() {
        // FIXME: there no zip, just plain raw string is contained
        let dec = FileOutputRawDecoder::new()
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
    fn should_encode_decode_a_binary_file() {
        let out = "/tmp/foo.zip.png";
        let input = "resources/secrets/random_1666_byte.bin";
        SteganoCore::new()
            .hide_file(input)
            .use_carrier_image("resources/Base.png")
            .write_to(out)
            .hide();

        let l = fs::metadata(out)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");
        let target = "/tmp/foo.bin.decoded";

        FileOutputDecoder::new()
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