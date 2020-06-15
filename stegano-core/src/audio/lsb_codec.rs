use bitstream_io::{BitReader, BitWriter, LittleEndian};
use hound::{WavReader, WavWriter};
use std::io::{BufWriter, Cursor, Read, Result, Seek, Write};

#[derive(Default)]
pub struct AudioPosition {}
impl AudioPosition {}

pub struct Decoder<'i, I, P, A> {
    input: &'i mut I,
    position: P,
    algorithm: A,
}

impl<'i, I, A> Decoder<'i, WavReader<I>, AudioPosition, A>
where
    I: Read,
    A: UnveilAlgorithm<i16>,
{
}

impl<'i, I, A> Read for Decoder<'i, WavReader<I>, AudioPosition, A>
where
    I: Read,
    A: UnveilAlgorithm<i16>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bits_to_write = buf.len() * 8;
        let buf_writer = BufWriter::new(buf);
        let mut bit_buffer = BitWriter::endian(buf_writer, LittleEndian);

        let mut bit_read = 0;
        for (_i, s) in self.input.samples::<i16>().enumerate() {
            if bit_read == bits_to_write {
                break;
            }
            let sample = s.unwrap();
            let bit = self.algorithm.decode(sample);
            bit_buffer.write_bit(bit).expect("Cannot write bit n");
            bit_read += 1;
        }

        if !bit_buffer.byte_aligned() {
            bit_buffer
                .byte_align()
                .expect("Failed to align the last byte read from carrier.");
        }

        Ok(bit_read / 8)
    }
}

pub struct Encoder<'i, I, O, P, A> {
    input: &'i mut I,
    output: &'i mut O,
    position: P,
    algorithm: A,
}

impl<'i, I, O, A> Encoder<'i, WavReader<I>, WavWriter<O>, AudioPosition, A>
where
    I: Read,
    O: Write + Seek,
    A: HideAlgorithm<i16>,
{
}

impl<'i, I, O, A> Write for Encoder<'i, WavReader<I>, WavWriter<O>, AudioPosition, A>
where
    I: Read,
    O: Write + Seek,
    A: HideAlgorithm<i16>,
{
    /// algorithm for LSB manipulation on audio data
    /// TODO keep track of position state so that write_all that operates in chunks works
    ///     add support for sequential writes
    ///     refactor the essence small strategy pattern that is injected on constructor
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let bits_to_write = buf.len() * 8;
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);
        let mut bits_written = 0;
        for (_i, s) in self.input.samples::<i16>().enumerate() {
            if bits_written == bits_to_write {
                break;
            }
            let sample = self.algorithm.encode(s.unwrap(), &bit_iter.read_bit());
            self.output.write_sample(sample).unwrap();
            bits_written += 1;
        }
        Ok(bits_written / 8)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Factory for decoder and encoder
pub struct LSBCodec;

impl LSBCodec {
    /// builds a LSB Audio Decoder that implements Read
    /// Example how to retrieve a decoder:
    ///
    /// ```rust
    /// use std::path::Path;
    /// use hound::{WavReader, WavWriter};
    /// use stegano_core::audio::LSBCodec;
    ///
    /// let audio_with_secret: &Path = "../resources/secrets/audio-with-secrets.wav".as_ref();
    /// let mut reader = WavReader::open(audio_with_secret).expect("Cannot create reader");
    ///
    /// let mut buf = vec![0; 12];
    /// LSBCodec::decoder(&mut reader)
    ///     .read_exact(&mut buf[..])
    ///     .expect("Cannot read 12 bytes from codec");
    /// let msg = String::from_utf8(buf).expect("Cannot convert result to string");
    /// assert_eq!("Hello World!", msg);
    /// ```
    pub fn decoder<'i, I: Read>(input: &'i mut WavReader<I>) -> Box<dyn Read + 'i> {
        Box::new(Decoder {
            input,
            algorithm: LSBAlgorithm::default(),
            position: AudioPosition::default(),
        })
    }

    /// builds a LSB Audio Encoder that implements Write
    /// Example how to retrieve an encoder:
    ///
    /// ```rust
    /// use std::path::Path;
    /// use tempdir::TempDir;
    /// use hound::{WavReader, WavWriter};
    /// use stegano_core::audio::LSBCodec;
    ///
    /// let input: &Path = "../resources/plain/carrier-audio.wav".as_ref();
    /// let out_dir = TempDir::new("audio-temp").expect("Cannot create temp dir");
    /// let audio_with_secret = out_dir.path().join("audio-with-secret.wav");
    ///
    /// let mut reader = WavReader::open(input).expect("Cannot create reader");
    /// let mut writer = WavWriter::create(audio_with_secret.as_path(), reader.spec())
    ///     .expect("Cannot create writer");
    /// let secret_message = "Hello World!".as_bytes();
    ///
    /// LSBCodec::encoder(&mut reader, &mut writer)
    ///     .write_all(&secret_message[..])
    ///     .expect("Cannot write to codec");
    /// ```
    pub fn encoder<'i, I: Read, O: Write + Seek>(
        input: &'i mut WavReader<I>,
        output: &'i mut WavWriter<O>,
    ) -> Box<dyn Write + 'i> {
        Box::new(Encoder {
            input,
            output,
            algorithm: LSBAlgorithm::default(),
            position: AudioPosition::default(),
        })
    }
}

#[derive(Default)]
struct LSBAlgorithm;

trait HideAlgorithm<T> {
    #[inline(always)]
    fn encode(&self, carrier: T, information: &Result<bool>) -> T;
}

impl HideAlgorithm<i16> for LSBAlgorithm {
    #[inline(always)]
    fn encode(&self, carrier: i16, information: &Result<bool>) -> i16 {
        match information {
            Err(_) => carrier,
            Ok(bit) => {
                (carrier & 0x7FFE) | {
                    match *bit {
                        true => 1,
                        false => 0,
                    }
                }
            }
        }
    }
}

impl HideAlgorithm<u8> for LSBAlgorithm {
    fn encode(&self, carrier: u8, information: &Result<bool>) -> u8 {
        match information {
            Err(_) => carrier,
            Ok(bit) => {
                (carrier & (u8::MAX - 1)) | {
                    if *bit {
                        1
                    } else {
                        0
                    }
                }
            }
        }
    }
}

pub trait UnveilAlgorithm<T> {
    fn decode(&self, carrier: T) -> bool;
}

impl UnveilAlgorithm<i16> for LSBAlgorithm {
    #[inline(always)]
    fn decode(&self, carrier: i16) -> bool {
        (carrier & 0x1) > 0
    }
}

impl UnveilAlgorithm<u8> for LSBAlgorithm {
    fn decode(&self, carrier: u8) -> bool {
        (carrier & 0x1) > 0
    }
}

#[cfg(test)]
mod audio_decoder_tests {
    use super::*;
    use std::path::Path;
    use tempdir::TempDir;

    const SOME_WAV: &str = "../resources/plain/carrier-audio.wav";

    #[test]
    fn it_should_encode_and_decode_a_string() -> Result<()> {
        let secret_message: &str = "Hello World!";
        let input: &Path = SOME_WAV.as_ref();
        let out_dir = TempDir::new("audio-temp")?;
        let audio_with_secret = out_dir.path().join("audio-with-secret.wav");

        {
            // Block is important so that writer is dropped, so that it persists the file
            let mut reader = WavReader::open(input).expect("Cannot create reader");
            let mut writer = WavWriter::create(audio_with_secret.as_path(), reader.spec())
                .expect("Cannot create writer");
            let buf = secret_message.as_bytes();
            let mut codec = LSBCodec::encoder(&mut reader, &mut writer);
            codec.write_all(&buf[..]).expect("Cannot write to codec");
        }

        let mut reader = WavReader::open(audio_with_secret.as_path())
            .expect("carrier audio file was not readable");
        let mut codec = LSBCodec::decoder(&mut reader);
        let mut buf = vec![0; 12];
        codec.read_exact(&mut buf).expect("Failed to read 12 bytes");
        let msg = String::from_utf8(buf).expect("Failed to convert result to string");
        assert_eq!(secret_message, msg);
        Ok(())
    }
}
