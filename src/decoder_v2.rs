use std::fs::*;
use std::io::prelude::*;
use std::io::*;
use std::io;
use std::borrow::{BorrowMut};
use super::{Decoder, ByteReader};

pub type SteganoDecoderV2 = SteganoDecoder<TerminatorFilter<File>>;
pub type SteganoRawDecoder = SteganoDecoder<File>;

pub struct SteganoDecoder<T>
    where T: Write + 'static
{
    output: Option<T>,
    input: Option<String>,
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
        self.input = Some(input_file.to_string());

        self
    }
}

impl<T> SteganoDecoder<T>
    where T: Filter<File> + Write
{
    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        self.output = Some(
            T::decorate(File::create(output_file.to_string())
                .expect("Target file should be writeable"))
        );

        self
    }
}

impl SteganoDecoder<File> {
    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        self.output = Some(
            File::create(output_file.to_string())
                .expect("Target file should be writeable")
        );

        self
    }
}

impl SteganoDecoder<ByteFilter<Stdout>> {
    pub fn write_to_stdout(&mut self, stdout: Stdout) -> &mut Self {
        self.output = Some(
            ByteFilter::decorate(stdout)
        );

        self
    }
}

impl<T> Decoder for SteganoDecoder<T>
    where T: Write
{
    fn unveil(&mut self) -> &mut Self {
        let mut reader = ByteReader::new(self.input.take().unwrap().as_str());
        let mut writer = self.output.take().unwrap();
        std::io::copy(&mut reader, &mut writer)
            .expect("Data was not transferred to output file");

        self
    }
}

pub struct ByteFilter<T> {
    inner: T,
    filter_byte: u8
}

pub trait Filter<T>
    where T: Write + 'static
{
    fn decorate(inner: T) -> Self;
}

impl<T> Filter<T> for ByteFilter<T>
    where T: Write + 'static
{
    fn decorate(inner: T) -> Self {
        ByteFilter { inner, filter_byte: 0x0 }
    }
}

impl<T> Write for ByteFilter<T>
    where T: Write
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for b in buf {
            if *b != self.filter_byte {
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

pub struct TerminatorFilter<T> {
    inner: T,
    terminators: Vec<u8>,
    terminator: u8,
    bof: bool,
}

impl<T> Filter<T> for TerminatorFilter<T>
    where T: Write + 'static
{
    fn decorate(inner: T) -> Self {
        TerminatorFilter {
            bof: false,
            inner,
            terminators: Vec::with_capacity(2),
            terminator: 0xff,
        }
    }
}

impl<T> Write for TerminatorFilter<T>
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
            if *b == self.terminator {
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

pub struct RijndaelFilter<T> {
    inner: T
}

impl<T> Filter<T> for RijndaelFilter<T>
    where T: Write + 'static
{
    fn decorate(inner: T) -> Self {
        RijndaelFilter { inner }
    }
}
