use image::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use bitstream_io::{BitWriter, LittleEndian};
use crate::BitIterator;
use std::cmp::min;

pub type SteganoDecoderV3 = SteganoDecoder;

pub struct SteganoDecoder {
    input: Option<RgbaImage>,
    x: u32,
    y: u32,
    c: usize,
}

impl SteganoDecoder {
    pub fn new(input_file: &str) -> Self {
        SteganoDecoder {
            input: Some(image::open(Path::new(input_file))
                .expect("Input image is not readable.")
                .to_rgba()
            ),
            x: 0,
            y: 0,
            c: 0,
        }
    }
}

impl Read for SteganoDecoder {
    fn read(&mut self, b: &mut [u8]) -> Result<usize> {
        let source_image = self.input.as_ref().unwrap();
        let width = source_image.width();
        let height = source_image.height();
        const COLORS: [usize; 3] = [0, 1, 2];
        let bytes_to_read = b.len();
        let total_progress = width * height;
        let mut buf_writer = BufWriter::with_capacity(b.len(), b);
//        let mut bit_buffer = BitWriter::endian(
//            &mut buf_writer,
//            LittleEndian,
//        );

        let mut bits_read = 0;
        let mut bytes_read = 0;
        let mut byte: u8 = 0;
        let mut progress: u8 = ((self.x * self.y * 100) / total_progress) as u8;
        'outer: for x in self.x..width {
            for y in self.y..height {
                let p = ((x * y * 100) / total_progress) as u8;
                if p > progress {
                    progress = p;
                    println!("progress: {}%", progress);
                    std::io::stdout().flush();
                }

                let image::Rgba(rgba) = source_image.get_pixel(x, y);
                for c in self.c..3 {    // TODO better iterate over COLORS
                    if bytes_read >= bytes_to_read {
                        self.x = x;
                        self.y = y;
                        self.c = c;
                        buf_writer.flush();
                        return Ok(bytes_read);
                    }
                    let bit_position = (bits_read % 8) as u8;
                    byte = ((rgba[c] & 1) << bit_position) | (byte);
                    bits_read += 1;
                    if bits_read % 8 == 0 {
                        bytes_read = bits_read / 8;
                        buf_writer.write(&[byte]);
                        byte = 0;
                    }
                }
                if self.c > 0 {
                    self.c = 0;
                }
            }
            if self.y > 0 {
                self.y = 0;
            }
        };
        self.x = width;

        buf_writer.flush();
        return Ok(bytes_read);
    }
}

#[cfg(test)]
mod tests {

    #![feature(test)]
    extern crate test;

    use super::*;
    use test::Bencher;

    const H: u8 = 'H' as u8;
    const e: u8 = 'e' as u8;
    const l: u8 = 'l' as u8;

    #[test]
    fn test_read_trait_behaviour_for_read_once() {
        let mut dec = SteganoDecoderV3::new("resources/HelloWorld_no_passwd_v2.x.png");

        let mut buf = [0 as u8; 13];
        let r = dec.read(&mut buf).unwrap();
        assert_eq!(r, 13, "bytes should have been read");
        assert_eq!(buf[0], 0x1, "1st byte does not match");
        assert_eq!(buf[1], H, "2nd byte is not a 'H'");
        assert_eq!(buf[2], e, "3rd byte is not a 'e'");
        assert_eq!(buf[3], l, "4th byte is not a 'l'");

        println!("{}", std::str::from_utf8(&buf).unwrap());
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\u{1}Hello World!");
    }

    #[test]
    fn test_read_trait_behaviour_for_read_multiple_times() {
        let mut dec = SteganoDecoderV3::new("resources/HelloWorld_no_passwd_v2.x.png");

        let mut buf = [0 as u8; 3];
        let r = dec.read(&mut buf).unwrap();
        assert_eq!(r, 3, "bytes should have been read");
        assert_eq!(buf[0], 0x1, "1st byte does not match");
        assert_eq!(buf[1], H, "2nd byte is not a 'H'");
        assert_eq!(buf[2], e, "3rd byte is not a 'e'");
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\u{1}He");

        let r = dec.read(&mut buf).unwrap();
        assert_eq!(r, 3, "bytes should have been read");
        assert_eq!(buf[0], l, "4th byte is not a 'l'");
        assert_eq!(buf[1], l, "5th byte is not a 'l'");
        assert_eq!(buf[2], 'o' as u8, "6th byte is not a 'o'");
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "llo");
    }

    #[test]
    fn test_read_trait_behaviour_for_read_all() {
        let mut dec = SteganoDecoderV3::new("resources/HelloWorld_no_passwd_v2.x.png");
        let expected_bytes = ((515 * 443 * 3) / 8) as usize;

        let mut buf = Vec::new();
        let r = dec.read_to_end(&mut buf).unwrap();
        assert_eq!(r, expected_bytes, "bytes should have been read"); // filesize
        assert_eq!(buf[0], 0x1, "1st byte does not match");
        assert_eq!(buf[1], H, "2nd byte is not a 'H'");
        assert_eq!(buf[2], e, "3rd byte is not a 'e'");
    }

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        b.iter(|| {
            let mut dec = SteganoDecoderV3::new("resources/HelloWorld_no_passwd_v2.x.png");
            let mut buf = Vec::new();
            let r = dec.read_to_end(&mut buf).unwrap();
        });
    }


    #[test]
    fn test_bit_writer() {
        let b = vec![0b0100_1000, 0b0110_0001, 0b0110_1100];
        let mut buf = Vec::with_capacity(3);

        {
            let mut buf_writer = BufWriter::new(&mut buf);
            let mut bit_buffer = BitWriter::endian(
                &mut buf_writer,
                LittleEndian,
            );

            bit_buffer.write_bit((0 & 1) == 1);
            bit_buffer.write_bit((0 & 1) == 1);
            bit_buffer.write_bit((0 & 1) == 1);
            bit_buffer.write_bit((1 & 1) == 1);
            bit_buffer.write_bit((0 & 1) == 1);
            bit_buffer.write_bit((0 & 1) == 1);
            bit_buffer.write_bit((1 & 1) == 1);
            bit_buffer.write_bit((0 & 1) == 1);
            buf_writer.flush();
        }

        assert_eq!(*buf.first().unwrap(), 'H' as u8);
    }
}