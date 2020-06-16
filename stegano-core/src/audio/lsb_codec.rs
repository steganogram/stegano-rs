use bitstream_io::{BitReader, BitWriter, LittleEndian};
use hound::{WavReader, WavWriter};
use std::io::{BufWriter, Cursor, Error, ErrorKind, Read, Result, Seek, Write};

#[derive(Default)]
pub struct AudioPosition {
    current: usize,
}
impl AudioPosition {
    pub fn next(&mut self) {
        self.current += 1
    }
}

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
        // 1 bit per sample
        let samples_to_take = buf.len() << 3;
        let buf_writer = BufWriter::new(buf);
        let mut bit_buffer = BitWriter::endian(buf_writer, LittleEndian);

        let mut bit_read = 0;
        for s in self
            .input
            .samples::<i16>()
            // NOTE:
            // `.skip(self.position.current)` is not required because the state of the iterator
            // is persisted at the underlying reader, and what is consumed remains consumed.
            .take(samples_to_take)
        {
            let sample = s.unwrap();
            let bit = self.algorithm.decode(sample);
            bit_buffer.write_bit(bit).expect("Cannot write bit n");
            bit_read += 1;
            self.position.next();
        }

        if !bit_buffer.byte_aligned() {
            bit_buffer
                .byte_align()
                .expect("Failed to align the last byte read from carrier.");
        }

        Ok(bit_read >> 3 as usize)
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
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let samples_to_take = buf.len() << 3; // 1 bit per sample <=> * 8 <=> << 3
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);
        let mut bit_written = 0;
        for s in self
            .input
            .samples::<i16>()
            // NOTE:
            // `.skip(self.position.current)` is not required because the state of the iterator
            // is persisted at the underlying reader, and what is consumed remains consumed.
            .take(samples_to_take)
        {
            let sample = self.algorithm.encode(s.unwrap(), &bit_iter.read_bit());
            self.output.write_sample(sample).unwrap();
            bit_written += 1;
            self.position.next();
        }
        Ok(bit_written >> 3)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
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
    #[inline(always)]
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
    #[inline(always)]
    fn decode(&self, carrier: T) -> bool;
}

impl UnveilAlgorithm<i16> for LSBAlgorithm {
    #[inline(always)]
    fn decode(&self, carrier: i16) -> bool {
        (carrier & 0x1) > 0
    }
}

impl UnveilAlgorithm<u8> for LSBAlgorithm {
    #[inline(always)]
    fn decode(&self, carrier: u8) -> bool {
        (carrier & 0x1) > 0
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

#[cfg(test)]
mod audio_decoder_tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use tempdir::TempDir;

    const SOME_WAV: &str = "../resources/plain/carrier-audio.wav";
    const BIG_SECRET: &str = "../LICENSE";

    #[test]
    fn it_should_encode_and_decode_in_chunks_by_using_read_to_end() -> Result<()> {
        let mut secret_to_hide = Vec::new();
        let file = File::open(BIG_SECRET)?;
        let mut buf_reader = BufReader::new(file);
        buf_reader.read_to_end(&mut secret_to_hide)?;
        secret_to_hide.shrink_to_fit();

        let input: &Path = SOME_WAV.as_ref();
        let out_dir = TempDir::new("audio-temp")?;
        let audio_with_secret = out_dir.path().join("audio-with-secret.wav");
        {
            // Block is important so that writer is dropped, so that it persists the file
            let mut reader = WavReader::open(input).expect("Cannot create reader");
            let mut writer = WavWriter::create(audio_with_secret.as_path(), reader.spec())
                .expect("Cannot create writer");
            let mut codec = LSBCodec::encoder(&mut reader, &mut writer);
            let half_the_buffer = secret_to_hide.len() / 2;
            codec
                .write(&secret_to_hide[..half_the_buffer])
                .expect("Cannot write half the buffer to codec");
            codec
                .write(&secret_to_hide[half_the_buffer..])
                .expect("Cannot write the other half of the buffer to codec");
        }

        let mut reader = WavReader::open(audio_with_secret.as_path())
            .expect("carrier audio file was not readable");
        let mut codec = LSBCodec::decoder(&mut reader);
        let mut unveiled_secret = Vec::new();
        let total_read = codec
            .read_to_end(&mut unveiled_secret)
            .expect("Cannot read all data from codec");
        assert_eq!(secret_to_hide.len(), total_read);

        let unveiled_secret_file_path = out_dir.path().join("LICENSE");
        let mut unveiled_secret_file = File::create(unveiled_secret_file_path.as_path())
            .expect("Cannot create the file for the unveiled data.");
        unveiled_secret_file.write_all(&unveiled_secret[..])?;
        assert_eq!(
            unveiled_secret_file.metadata().unwrap().len() as usize,
            secret_to_hide.len()
        );
        Ok(())
    }
}
