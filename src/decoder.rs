use std::fs::*;
use std::io::prelude::*;
use super::{Decoder, ByteReader, FilterReader};
use super::Codec;
//use std::io::{Stdout, stdout, StdoutLock};

pub type FileOutputDecoder = SteganoDecoder<File, Codec<ByteReader>>;
pub type FileOutputRawDecoder = SteganoDecoder<File, ByteReader>;

pub struct SteganoDecoder<O, I>
    where O: Write + 'static,
          I: Read + Sized
{
    output: Option<O>,
    input: Option<I>,
}

impl<O, I> Default for SteganoDecoder<O, I>
    where O: Write + 'static,
          I: Read + Sized
{
    fn default() -> Self {
        Self {
            output: None,
            input: None,
        }
    }
}

impl<O, I> SteganoDecoder<O, I>
    where O: Write + 'static,
          I: Read + Sized
{
    pub fn new() -> Self {
    Self::default()
}
}

impl<O> SteganoDecoder<O, ByteReader>
    where O: Write + 'static
{
    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.input = Some(ByteReader::new(input_file));

        self
    }
}

impl<O> SteganoDecoder<O, Codec<ByteReader>>
    where O: Write + 'static
{
    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.input = Some(
            Codec::decoder(
                ByteReader::new(input_file)));

        self
    }
}

impl<O> SteganoDecoder<O, FilterReader<ByteReader>>
    where O: Write + 'static
{
    pub fn use_source_image(&mut self, input_file: &str) -> &mut Self {
        self.input = Some(FilterReader::new(ByteReader::new(input_file)));

        self
    }
}

impl<I> SteganoDecoder<File, I>
    where I: Read + Sized
{
    pub fn write_to_file(&mut self, output_file: &str) -> &mut Self {
        let file = File::create(output_file.to_string())
            .expect("Target file should be writeable");
        self.output = Some(file);

        self
    }
}

//impl<'a, I> SteganoDecoder<StdoutLock<'a>, I>
//    where I: Read + Sized
//{
//    pub fn write_to_stdout(&mut self, stdout: StdoutLock<'a>) -> &mut Self {
//        self.output = Some(stdout);
//
//        self
//    }
//}

impl<O, I> Decoder for SteganoDecoder<O, I>
    where O: Write,
          I: Read + Sized
{
    fn unveil(&mut self) -> &mut Self {
        let mut reader = self.input.take().unwrap();
        let mut writer = self.output.take().unwrap();
        std::io::copy(&mut reader, &mut writer)
            .expect("Data was not transferred to output file");

        self
    }
}