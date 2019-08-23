use image::{GenericImageView, ImageBuffer, Pixel, Rgba};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub struct Steganogramm {
    target: Option<String>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
}

pub trait Encoder {
    fn hide(self) -> Self;
}

pub trait Decoder {
    fn unhide(self) -> Self;
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

    pub fn hide(&mut self) -> &mut Steganogramm {
        let mut source = self.source.take().unwrap();
        let carrier = self.carrier.take().unwrap();
        let (width, heigh) = carrier.dimensions();

        let mut target: image::RgbaImage = image::ImageBuffer::new(width, heigh);

        for (x, y, pixel) in target.enumerate_pixels_mut() {
            let other = carrier.get_pixel(x, y);
            let r = (0.3 * x as f32) as u8;
            let b = (0.3 * y as f32) as u8;
            // TODO feat(core:hide) implement the basic functionality here
            *pixel = Rgba([r, 0, b, 0xff]);
            // *pixel = other;
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
