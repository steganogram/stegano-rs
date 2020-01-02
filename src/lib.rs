#[macro_use] extern crate hex_literal;

pub mod bit_iterator;
pub use bit_iterator::BitIterator;

pub mod decoder;
pub use decoder::*;

pub mod byte_reader;
pub use byte_reader::*;

pub mod filter_reader;
pub use filter_reader::*;

pub mod decipher;

use bitstream_io::{LittleEndian, BitReader};
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use image::*;
use std::io;

pub struct SteganoEncoder {
    target: Option<String>,
    target_image: Option<RgbaImage>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
    x: u32,
    y: u32,
    c: u8,
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
            c: 0
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
        let mut buf = Vec::new();
        self.source.as_ref().unwrap().read_to_end(&mut buf)
            .expect("File was probably too big");
        self.write(&buf);
        self.flush();

        self
    }
}

impl Write for SteganoEncoder {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        #[inline]
        fn bit_wave(byte: u8, bit: io::Result<bool>) -> u8 {
            let byt = match bit {
                Err(_) => 0,
                Ok(byt) => if byt { 1 } else { 0 }
            };
            (byte & 0xFE) | byt
        }

        let carrier = self.carrier.as_ref().unwrap();
        let (width, height) = carrier.dimensions();
        let mut buf = Vec::from(buf);
        match self.target_image {
            None => {
                self.target_image = Some(ImageBuffer::new(width, height));
                buf.insert(0, 0x02);
                buf.push(0xff);
            }
            _ => {}
        }
        let mut bit_iter = BitReader::endian(
            Cursor::new(buf),
            LittleEndian,
        );

        let mut bits_written = 0;
        for x in self.x..width {
            for y in self.y..height {
                // TODO check if there are full bytes left to write,
                //      half bytes must be written on flush or on next iteration of write

                let image::Rgba(data) = carrier.get_pixel(x, y);
                let new_rgba = Rgba([
                    bit_wave(data[0], bit_iter.read_bit()),
                    bit_wave(data[1], bit_iter.read_bit()),
                    bit_wave(data[2], bit_iter.read_bit()),
                    data[3],
                ]);
                self.target_image.as_mut()
                    .expect("Target Image was not present.")
                    .put_pixel(x, y, new_rgba);
                bits_written += 3;
            }
        }

        Ok(bits_written / 8)
    }

    fn flush(&mut self) -> Result<()> {
        // TODO this can only be called once the state awareness is given in the write function
//        self.write(&[0xff]);
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

        let mut zeros = 0;
        for b in buf.iter().rev() {
            let b = *b;
            if b == 0 {
                zeros += 1;
            } else {
                break
            }
        }

        let given = fs::metadata(target)
            .expect("Output image was not written.")
            .len();

        let given = given - zeros - 2;
        assert_eq!(given, expected, "Unveiled file size differs to the original");
    }
}