use image::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use std::slice;

pub struct Steganogramm {
    target: Option<String>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
}

pub trait Encoder {
    fn hide<'a>(&'a self) -> &'a Self;
}

pub trait Decoder {
    fn unhide(&mut self) -> &mut Self;
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

impl<I> Iterator for BitIterator<I>
where
    I: Read,
{
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
    fn hide<'a>(&'a self) -> &'a Self {
        let carrier = self.carrier.as_ref().unwrap();
        let (width, heigh) = carrier.dimensions();
        let mut bit_iter = BitIterator::new(self.source.as_ref().unwrap());
        let mut target: RgbaImage = ImageBuffer::new(width, heigh);

        #[inline]
        fn bit_wave(byte: &u8, bit: &Option<u8>) -> u8 {
            match bit {
                None => *byte,
                Some(b) => (*byte & 0xFE) | (b & 1),
            }
        }

        for (x, y, pixel) in target.enumerate_pixels_mut() {
            let image::Rgba(data) = carrier.get_pixel(x, y);
            *pixel = Rgba([
                bit_wave(&data[0], &bit_iter.next()),
                bit_wave(&data[1], &bit_iter.next()),
                bit_wave(&data[2], &bit_iter.next()),
                data[3],
            ]);
        }
        target.save(self.target.as_ref().unwrap()).unwrap();

        self
    }
}

impl Steganogramm {
    pub fn new() -> Self {
        Steganogramm {
            carrier: None,
            source: None,
            target: None,
        }
    }

    pub fn use_carrier_image(&mut self, input_file: &str) -> &mut Self {
        self.carrier =
            Some(image::open(Path::new(input_file)).expect("Carrier image was not readable."));
        self
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Self {
        self.target = Some(output_file.to_string());
        self
    }

    pub fn take_data_to_hide_from(&mut self, input_file: &str) -> &mut Self {
        self.source = Some(File::open(input_file).expect("Source file was not readable."));
        self
    }
}

impl Steganogramm {
    pub fn decoder() -> Self {
        Steganogramm {
            carrier: None,
            source: None,
            target: None,
        }
    }

    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.source = Some(File::open(input_file).expect("Source file was not readable."));

        self
    }
}

impl Decoder for Steganogramm {
    fn unhide(&mut self) -> &mut Self {
        self
    }
}
