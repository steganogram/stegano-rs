pub mod bit_iterator;

pub use bit_iterator::BitIterator;
use bitstream_io::{BitWriter, LittleEndian};
use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::path::Path;
use image::*;
use std::io;
use std::borrow::BorrowMut;

pub struct SteganoEncoder {
    target: Option<String>,
    carrier: Option<image::DynamicImage>,
    source: Option<std::fs::File>,
}

pub trait Encoder {
    fn hide(&self) -> &Self;
}

pub trait Decoder {
    fn unveil(&mut self) -> &mut Self;
}

impl Default for SteganoEncoder {
    fn default() -> Self {
        Self {
            target: None,
            carrier: None,
            source: None
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
    fn hide(&self) -> &Self {
        let carrier = self.carrier.as_ref().unwrap();
        let (width, height) = carrier.dimensions();
        let mut bit_iter = BitIterator::new(self.source.as_ref().unwrap());
        let mut target: RgbaImage = ImageBuffer::new(width, height);

        #[inline]
        fn bit_wave(byte: u8, bit: Option<u8>) -> u8 {
            let mut b = 0;
            match bit {
                None => {}
                Some(byt) => b = byt,
            }

            (byte & 0xFE) | b
        }

        for x in 0..width {
            for y in 0..height {
                let image::Rgba(data) = carrier.get_pixel(x, y);
                target.put_pixel(x, y,  Rgba([
                    bit_wave(data[0], bit_iter.next()),
                    bit_wave(data[1], bit_iter.next()),
                    bit_wave(data[2], bit_iter.next()),
                    data[3],
                ]));
            }
        }

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

impl<T> Default for SteganoDecoder<T>
where T: Write + 'static
{
    fn default() -> Self {
        Self {
            output: None,
            input: None
        }
    }
}

impl<T> SteganoDecoder<T>
where T: Write + 'static
{
    pub fn new() -> Self {
        Self::default()
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

impl<T> SteganoDecoder<T>
where T: Filter<File> + Write
{
    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        self.output = Some(
            T::decorate(File::create(output_file.to_string())
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
        let source_image = self.input.as_ref().unwrap();
        let mut bit_buffer = BitWriter::endian(
            self.output.take().unwrap(),
            LittleEndian
        );

        for x in 0..source_image.width() {
            for y in 0..source_image.height() {
                let image::Rgba(data) = source_image.get_pixel(x, y);
                bit_buffer
                    .write_bit((data[0] & 1) == 1)
                    .unwrap_or_else(|_| panic!("Color R on Pixel({}, {})", x, y));
                bit_buffer
                    .write_bit((data[1] & 1) == 1)
                    .unwrap_or_else(|_| panic!("Color G on Pixel({}, {})", x, y));
                bit_buffer
                    .write_bit((data[2] & 1) == 1)
                    .unwrap_or_else(|_| panic!("Color B on Pixel({}, {})", x, y));
            }
        }

        self
    }
}

pub struct ZeroFilter<T> {
    inner: T
}

pub trait Filter<T>
where T: Write + 'static
{
    fn decorate(inner: T) -> Self;
}

impl<T> Filter<T> for ZeroFilter<T>
where T: Write + 'static
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
                match self.inner.borrow_mut().write(&[*b]) {
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

pub struct FFFilter<T> {
    inner: T,
    terminators: Vec<u8>,
    bof: bool,
}

impl<T> Filter<T> for FFFilter<T>
where T: Write + 'static
{
    fn decorate(inner: T) -> Self {
        FFFilter { bof: false, inner, terminators: Vec::with_capacity(2) }
    }
}

impl<T> Write for FFFilter<T>
where T: Write
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for b in buf {
            if !self.bof && *b == 0x01 {
                self.bof = true;
                continue;
            }
            if self.terminators.len() >= 2 {
                continue;
            }
            if *b == 0xff {
                self.terminators.push(*b);
                continue;
            } else {
                match self.inner.borrow_mut().write(&self.terminators.to_vec()) {
                    Ok(_) => {},
                    Err(e) => return Err(e)
                }
                self.terminators.clear();
            }
            if self.terminators.len() < 2 {
                match self.inner.borrow_mut().write(&[*b]) {
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

pub type SteganoDecoderV2 = SteganoDecoder<FFFilter<File>>;