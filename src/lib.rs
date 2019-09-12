use image::*;
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Bytes;
use std::io::*;
use std::path::Path;
use std::slice;

pub struct Steganogramm {
    target: Option<String>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
}

pub trait Encoder {
    fn hide(&mut self) -> &mut Self;
}

pub trait Decoder {
    fn unhide(self) -> Self;
}

pub struct BitIterator<I> {
    n: u32,
    i: u32,
    iter: I,
    byte: Option<u8>,
}

impl<I> BitIterator<I> {
    pub fn new(s: I) -> Self {
        BitIterator {
            n: 8,
            i: 0,
            iter: s,
            byte: None,
        }
    }
}

impl<I: Read> Iterator for BitIterator<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let bit = self.i % self.n;
            self.i = self.i + 1;
            if bit == 0 {
                self.byte = None;
            }
            if self.byte == None {
                let mut b = 0;
                match self.iter.read(slice::from_mut(&mut b)) {
                    Ok(0) => None,
                    Ok(..) => {
                        self.byte = Some(b);
                        self.byte
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(_) => None,
                };
            }
            return match self.byte {
                None => None,
                Some(b) => Some((b >> bit) & 1),
            };
        }
    }
}

impl Encoder for Steganogramm {
    fn hide(&mut self) -> &mut Steganogramm {
        let source = self.source.take().unwrap();
        let carrier = self.carrier.take().unwrap();
        let (width, heigh) = carrier.dimensions();
        let bit_iter = BitIterator::new(source);

        let mut target: RgbaImage = ImageBuffer::new(width, heigh);

        for (x, y, pixel) in target.enumerate_pixels_mut() {
            let other = carrier.get_pixel(x, y);
            // let r = (0.3 * x as f32) as u8;
            // let b = (0.3 * y as f32) as u8;
            // TODO feat(core:hide) implement the basic functionality here
            let image::Rgba(data) = other;
            *pixel = Rgba([data[0], data[1], data[2], data[3]]);
        }
        target.save(self.target.take().unwrap()).unwrap();

        // loop {
        //   let mut buffer = [0; 512];
        //   let bytes_read = source.read(&mut buffer).unwrap();

        //   if bytes_read != buffer.len() {
        //     break;
        //   }
        // }

        // match self.encoder {
        //   None => {},
        //   Some(ref _enc) => {
        //     // let data = [0x15; 512*512]; // An array containing a RGBA sequence. First pixel is red and second pixel is black.
        //     // enc.write_header().unwrap().write_image_data(&data).unwrap(); // Save
        //     // self.encoder = Some(*enc)
        //   }
        // }

        self
    }
}

impl Steganogramm {
    pub fn new() -> Steganogramm {
        Steganogramm {
            carrier: None,
            source: None,
            target: None,
        }
    }

    pub fn use_carrier_image(&mut self, input_file: &str) -> &mut Steganogramm {
        self.carrier =
            Some(image::open(Path::new(input_file)).expect("Carrier image was not readable."));
        self
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Steganogramm {
        self.target = Some(output_file.to_string());
        self
    }

    pub fn take_data_to_hide_from(&mut self, input_file: &str) -> &mut Steganogramm {
        self.source = Some(File::open(input_file).expect("Source file was not readable."));
        self
    }
}
