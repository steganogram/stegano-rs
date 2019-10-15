pub mod bit_iterator;

pub use bit_iterator::BitIterator;
use bitstream_io::{BitReader, BitWriter, LittleEndian, Numeric};
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use std::slice::Chunks;
use std::vec::Vec;
use image::*;
use std::io;

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
    fn unveil(&mut self) -> &mut Self;
}

impl Encoder for Steganogramm {
    fn hide<'a>(&'a self) -> &'a Self {
        let carrier = self.carrier.as_ref().unwrap();
        let (width, heigh) = carrier.dimensions();
        let mut reader = BitReader::endian(
            self.source.as_ref().unwrap(),
            LittleEndian
        );
        // let mut bit_iter = BitIterator::new(self.source.as_ref().unwrap());
        let mut target: RgbaImage = ImageBuffer::new(width, heigh);

        #[inline]
        fn bit_wave(byte: &u8, bit: &Result<bool>) -> u8 {
            let mut b = 0;
            match bit {
                Err(_) => {}
                Ok(byt) => b = if *byt == true { 1 } else { 0 },
            }

            (*byte & 0xFE) | b
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

        // let mut output = File::create(self.target.unwrap()).unwrap();
        // target.write_to(&mut output, PNG).unwrap();

        target.save(self.target.as_ref().unwrap()).unwrap();

        self
    }
}

pub struct SteganoDecoder<T>
where T: Write + 'static
{
    output: Option<T>,
    input: Option<RgbaImage>,
}

impl<T> SteganoDecoder<T>
where T: Write + 'static
{
    pub fn new() -> Self {
        SteganoDecoder {
            output: None,
            input: None,
        }
    }

    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.input = Some(
            image::open(Path::new(input_file))
                .expect("Carrier image was not readable.")
                .to_rgba()
        );

        self
    }
}

impl SteganoDecoder<ZeroFilter<File>> {
    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        self.output = Some(
            ZeroFilter::decorate(File::create(output_file.to_string())
                .expect("Target should be write able"))
        );

        self
    }
}

impl SteganoDecoder<ZeroFilter<Stdout>> {
    pub fn write_to_stdout(&mut self, stdout: Stdout) -> &mut Self {
        self.output = Some(
            ZeroFilter::decorate(stdout)
        );

        self
    }
}

impl<T> Decoder for SteganoDecoder<T>
where T: Write
{
    fn unveil(&mut self) -> &mut Self {
        let source_image = self.input.as_ref().unwrap().pixels();
        let mut bit_buffer = BitWriter::endian(
            self.output.take().unwrap(),
            LittleEndian
        );

        for pixel in source_image {
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

pub struct ZeroFilter<T> {
    inner: T
}

impl<T> ZeroFilter<T>
where T: Write
{
    fn decorate(inner: T) -> Self {
        ZeroFilter { inner }
    }
}

impl<T> Write for ZeroFilter<T>
where T: Write
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for b in buf {
            if *b != 0 {
                match self.inner.write(&[*b]) {
                    Ok(_) => {},
                    Err(e) => return Err(e)
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}