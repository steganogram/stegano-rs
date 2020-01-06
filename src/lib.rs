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

pub struct SteganoEncoder {
    target: Option<String>,
    target_image: Option<RgbaImage>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
    x: u32,
    y: u32,
    c: usize,
}

pub trait Encoder {
    fn hide(&mut self) -> &Self;
}

pub trait Decoder {
    fn unveil(&mut self) -> &mut Self;
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
            target: None,
            target_image: None,
            carrier: None,
            source: None,
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
                .expect("Carrier image was not readable."));
        self
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Self {
        self.target = Some(output_file.to_string());
        self
    }

    pub fn take_data_to_hide_from(&mut self, input_file: &str) -> &mut Self {
        self.source = Some(
            File::open(input_file)
                .expect("Source file was not readable."));
        self
    }
}

impl Encoder for SteganoEncoder {
    fn hide(&mut self) -> &Self {
        {
            let mut reader = self.source.take().unwrap();
            let mut codec = Codec::encoder(self.borrow_mut());

            std::io::copy(&mut reader, &mut codec)
                .expect("Failed to copy data to the codec");

            codec.flush()
                .expect("Failed to flush the codec.");
        }

        self
    }
}

impl Write for SteganoEncoder {
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
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn should_encode_decode_a_binary_file() {
        let out = "/tmp/foo.zip.png";
        let input = "tmp/foo.zip";
        SteganoEncoder::new()
            .take_data_to_hide_from(input)
            .use_carrier_image("resources/Base.png")
            .write_to(out)
            .hide();

        let l = fs::metadata(out)
            .expect("Output image was not written.")
            .len();
        assert!(l > 0, "File is not supposed to be empty");
        let target = "/tmp/foo.decoded.zip";

        FileOutputDecoder::new()
            .use_source_image(out)
            .write_to_file(target)
            .unveil();

        let expected = fs::metadata(input)
            .expect("Source file is not available.")
            .len();

        let mut buf = Vec::new();
        let mut file = File::open(target)
            .expect("output file is not readbale");
        let r = file.read_to_end(&mut buf).unwrap();

        let mut reader = std::io::Cursor::new(&buf[..]);
        let mut zip = zip::ZipArchive::new(reader)
            .expect("zip archive was not readable");
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            println!("Filename: {}", file.name());
            let first_byte = file.bytes().next().unwrap()
                .expect("not able to read next byte");
            println!("{}", first_byte);
        }

        let mut zeros = 0;
        for b in buf.iter().rev() {
            let b = *b;
            if b == 0 {
                zeros += 1;
            } else {
                break;
            }
        }

        let given = fs::metadata(target)
            .expect("Output image was not written.")
            .len();

        let given = given - zeros - 2;
//        assert_eq!(given, expected, "Unveiled file size differs to the original");
    }
}