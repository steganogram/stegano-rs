pub mod bit_iterator;

pub use bit_iterator::BitIterator;
use bitstream_io::{BitReader, BitWriter, LittleEndian};
use image::*;
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;

pub struct Steganogramm {
    target: Option<String>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
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

pub trait Encoder {
    fn hide<'a>(&'a self) -> &'a Self;
}

pub trait Decoder {
    fn unhide(&mut self) -> &mut Self;
}

impl Encoder for Steganogramm {
    fn hide<'a>(&'a self) -> &'a Self {
        let carrier = self.carrier.as_ref().unwrap();
        let (width, heigh) = carrier.dimensions();
        let mut reader = BitReader::endian(self.source.as_ref().unwrap(), LittleEndian);
        // let mut bit_iter = BitIterator::new(self.source.as_ref().unwrap());
        let mut target: RgbaImage = ImageBuffer::new(width, heigh);

        #[inline]
        fn bit_wave(byte: &u8, bit: &Result<bool>) -> u8 {
            match bit {
                Err(_) => *byte,
                Ok(b) => (*byte & 0xFE) | (if *b == true { 1 } else { 0 }),
            }
        }

        for (x, y, pixel) in target.enumerate_pixels_mut() {
            let image::Rgba(data) = carrier.get_pixel(x, y);
            *pixel = Rgba([
                bit_wave(&data[0], &reader.read_bit()),
                bit_wave(&data[1], &reader.read_bit()),
                bit_wave(&data[2], &reader.read_bit()),
                data[3],
            ]);
        }
        target.save(self.target.as_ref().unwrap()).unwrap();

        self
    }
}

pub struct SteganoDecode {
    target: Option<File>,
    source: Option<String>,
}

impl SteganoDecode {
    pub fn new() -> Self {
        SteganoDecode {
            source: None,
            target: None,
        }
    }

    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.source = Some(input_file.to_string());

        self
    }

    pub fn write_to(&mut self, output_file: &str) -> &mut Self {
        self.target =
            Some(File::create(output_file.to_string()).expect("Target should be write able"));
        self
    }
}

impl Decoder for SteganoDecode {
    fn unhide(&mut self) -> &mut Self {
        let img = image::open(Path::new(self.source.as_ref().unwrap().as_str()))
            .expect("Carrier image was not readable.");
        let source = img.as_rgba8().unwrap();
        let t = self.target.take().unwrap();
        let mut bit_buffer = BitWriter::endian(t, LittleEndian);

        for (_x, _y, pixel) in source.enumerate_pixels() {
            let image::Rgba(data) = pixel;
            bit_buffer
                .write_bit((data[0] & 0x01) == 1)
                .expect("Bit R on Pixel({}, {})");
            bit_buffer
                .write_bit((data[1] & 0x01) == 1)
                .expect("Bit G on Pixel({}, {})");
            bit_buffer
                .write_bit((data[2] & 0x01) == 1)
                .expect("Bit B on Pixel({}, {})");
        }

        self
    }
}
