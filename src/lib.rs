pub mod bit_iterator;

pub use bit_iterator::BitIterator;
use image::*;
use std::fs::File;
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

        // TODO there is not yet a way to write to to stdout
        // let stdout = std::io::stdout();
        // let mut handle = stdout.lock();
        // target.write(&mut handle);
        // let b = &buf[..];
        // let b = target.image_to_bytes();
        // handle.write_all(&target.to_vec());

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
