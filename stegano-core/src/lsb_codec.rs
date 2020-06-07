use std::io::{BufWriter, Cursor, Read, Result, Write};

use bitstream_io::{BitReader, BitWriter, LittleEndian};
use image::{Rgba, RgbaImage};

pub struct LSBCodec<'img> {
    subject: &'img mut RgbaImage,
    x: u32,
    y: u32,
    c: usize,
}

impl<'img> LSBCodec<'img> {
    pub fn new(image: &'img mut RgbaImage) -> Self {
        LSBCodec {
            subject: image,
            x: 0,
            y: 0,
            c: 0,
        }
    }
}

impl<'img> Read for LSBCodec<'img> {
    fn read(&mut self, b: &mut [u8]) -> Result<usize> {
        #[inline]
        #[cfg(debug_assertions)]
        fn update_progress(total_progress: u32, progress: &mut u8, x: u32, y: u32) {
            let p = ((x * y * 100) / total_progress) as u8;
            if p > *progress {
                *progress = p;
                print!("\rProgress: {}%", p);
                if p == 99 {
                    println!("\rDone                    ");
                }
            }
        }
        #[inline]
        #[cfg(not(debug_assertions))]
        fn update_progress(total_progress: u32, progress: &mut u8, x: u32, y: u32) {
            let p = ((x * y * 100) / total_progress) as u8;
            if p > *progress {
                *progress = p;
            }
        }

        let source_image = &self.subject;
        let (width, height) = source_image.dimensions();
        let bytes_to_read = b.len();
        let total_progress = width * height;
        let buf_writer = BufWriter::new(b);
        let mut bit_buffer = BitWriter::endian(buf_writer, LittleEndian);

        let mut progress: u8 = ((self.x * self.y * 100) / total_progress) as u8;
        let mut bits_read = 0;
        let mut bytes_read = 0;
        for x in self.x..width {
            for y in self.y..height {
                let image::Rgba(rgba) = source_image.get_pixel(x, y);
                for (c, color) in rgba.iter().enumerate().take(3).skip(self.c) {
                    if bytes_read >= bytes_to_read {
                        self.x = x;
                        self.y = y;
                        self.c = c;
                        return Ok(bytes_read);
                    }
                    let bit = color & 0x01;
                    bit_buffer
                        .write_bit(bit > 0)
                        .unwrap_or_else(|_| panic!("Color {} on Pixel({}, {})", c, x, y));
                    bits_read += 1;

                    if bits_read % 8 == 0 {
                        bytes_read = (bits_read / 8) as usize;
                        update_progress(total_progress, &mut progress, x, y);
                    }
                }
                if self.c > 0 {
                    self.c = 0;
                }
            }
            if self.y > 0 {
                self.y = 0;
            }
        }
        self.x = width;
        if !bit_buffer.byte_aligned() {
            bit_buffer
                .byte_align()
                .expect("Failed to align the last byte read from image.");
        }

        Ok(bytes_read)
    }
}

impl<'img> Write for LSBCodec<'img> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        #[inline]
        fn bit_wave(byte: u8, bit: Result<bool>) -> u8 {
            let byt = match bit {
                // TODO here we need some configurability, to prevent 0 writing on demand
                Err(_) => byte,
                Ok(byt) => {
                    if byt {
                        1
                    } else {
                        0
                    }
                }
            };
            (byte & 0xFE) | byt
        }

        let carrier = &mut self.subject;
        let (width, height) = carrier.dimensions();
        let bytes_to_write = buf.len();
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);

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
                        carrier.put_pixel(x, y, Rgba(rgba));
                        return Ok(bytes_written);
                    }

                    rgba[c] = bit_wave(rgba[c], bit_iter.read_bit());
                    bits_written += 1;
                    if bits_written % 8 == 0 {
                        bytes_written = (bits_written / 8) as usize;
                    }
                }
                carrier.put_pixel(x, y, Rgba(rgba));
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
        Ok(())
    }
}

#[cfg(test)]
mod decoder_tests {
    use super::*;

    const H: u8 = b'H';
    const E: u8 = b'e';
    const L: u8 = b'l';
    const O: u8 = b'o';
    const HELLO_WORLD_PNG: &str = "resources/with_text/hello_world.png";
    const ZIP_PNG: &str = "resources/with_attachment/Blah.txt.png";

    #[test]
    fn test_read_trait_behaviour_for_read_once() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let mut dec = LSBCodec::new(&mut img);

        let mut buf = [0 as u8; 13];
        let r = dec.read(&mut buf).unwrap();
        assert_eq!(r, 13, "bytes should have been read");
        assert_eq!(buf[0], 0x1, "1st byte does not match");
        assert_eq!(buf[1], H, "2nd byte is not a 'H'");
        assert_eq!(buf[2], E, "3rd byte is not a 'e'");
        assert_eq!(buf[3], L, "4th byte is not a 'l'");

        println!("{}", std::str::from_utf8(&buf).unwrap());
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\u{1}Hello World!");
    }

    #[test]
    fn test_read_trait_behaviour_for_read_multiple_times() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let mut dec = LSBCodec::new(&mut img);

        let mut buf = [0 as u8; 3];
        let r = dec.read(&mut buf).unwrap();
        assert_eq!(r, 3, "bytes should have been read");
        assert_eq!(buf[0], 0x1, "1st byte does not match");
        assert_eq!(buf[1], H, "2nd byte is not a 'H'");
        assert_eq!(buf[2], E, "3rd byte is not a 'e'");
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\u{1}He");

        let r = dec.read(&mut buf).unwrap();
        assert_eq!(r, 3, "bytes should have been read");
        assert_eq!(buf[0], L, "4th byte is not a 'l'");
        assert_eq!(buf[1], L, "5th byte is not a 'l'");
        assert_eq!(buf[2], O, "6th byte is not a 'o'");
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "llo");
    }

    #[test]
    fn should_not_contain_noise_bytes() {
        let mut img = image::open(ZIP_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let mut dec = LSBCodec::new(&mut img);
        let expected_bytes = ((515 * 443 * 3) / 8) as usize;

        let mut buf = Vec::new();
        let r = dec.read_to_end(&mut buf).unwrap();
        assert_eq!(r, expected_bytes, "bytes should have been read"); // filesize
    }
}

#[cfg(test)]
mod bit_writer_tests {
    use super::*;

    #[test]
    fn test_bit_writer() {
        let _b = vec![0b0100_1000, 0b0110_0001, 0b0110_1100];
        let mut buf = Vec::with_capacity(3);

        {
            let mut buf_writer = BufWriter::new(&mut buf);
            let mut bit_buffer = BitWriter::endian(&mut buf_writer, LittleEndian);

            bit_buffer.write_bit(0 == 1).expect("1 failed");
            bit_buffer.write_bit(0 == 1).expect("2 failed");
            bit_buffer.write_bit(0 == 1).expect("3 failed");
            bit_buffer.write_bit(true).expect("4 failed");
            bit_buffer.write_bit(0 == 1).expect("5 failed");
            bit_buffer.write_bit(0 == 1).expect("6 failed");
            bit_buffer.write_bit(true).expect("7 failed");
            bit_buffer.write_bit(0 == 1).expect("8 failed");
        }

        assert_eq!(*buf.first().unwrap(), b'H');
    }
}
