use bitstream_io::{BitReader, BitWriter, LittleEndian};
use hound::{WavReader, WavWriter};
use std::io::{BufWriter, Cursor, Read, Result, Seek, Write};

#[derive(Default)]
pub struct AudioPosition {}
impl AudioPosition {}

pub struct LSBDecoder<'i, I, P> {
    input: &'i mut I,
    position: P,
}

impl<'i, I> LSBDecoder<'i, WavReader<I>, AudioPosition> where I: Read {}

impl<'a, A> Read for LSBDecoder<'a, WavReader<A>, AudioPosition>
where
    A: Read,
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
            let bit = sample & 0x0001;
            bit_buffer.write_bit(bit > 0).expect("Cannot write bit n");
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

pub struct LSBEncoder<'i, 'o, I, O, P> {
    input: &'i mut I,
    output: &'o mut O,
    position: P,
}

impl<'i, 'o, I, O> LSBEncoder<'i, 'o, WavReader<I>, WavWriter<O>, AudioPosition>
where
    I: Read,
    O: Write + Seek,
{
}

impl<'a, 'c, A, C> Write for LSBEncoder<'a, 'c, WavReader<C>, WavWriter<A>, AudioPosition>
where
    A: Write + Seek,
    C: Read,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        #[inline]
        fn bit_wave(byte: i16, bit: Result<bool>) -> i16 {
            let byt = match bit {
                Err(_) => byte,
                Ok(byt) => {
                    if byt {
                        1
                    } else {
                        0
                    }
                }
            };
            (byte & (i16::MAX - 1)) | byt
        }

        let bits_to_write = buf.len() * 8;
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);
        let mut bits_written = 0;
        for (_i, s) in self.input.samples::<i16>().enumerate() {
            if bits_written == bits_to_write {
                break;
            }
            let sample = s.unwrap();
            let sample = bit_wave(sample, bit_iter.read_bit());
            self.output.write_sample(sample).unwrap();
            bits_written += 1;
        }
        Ok(bits_written / 8)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

type LSBAudioDecoder<'i, I> = LSBDecoder<'i, WavReader<I>, AudioPosition>;
type LSBAudioEncoder<'i, 'o, I, O> = LSBEncoder<'i, 'o, WavReader<I>, WavWriter<O>, AudioPosition>;

pub struct LSBCodec;

impl LSBCodec {
    /// builds a LSB Audio Decoder that implements Read
    pub fn decoder<I: Read>(input: &mut WavReader<I>) -> LSBAudioDecoder<I> {
        LSBDecoder {
            input,
            position: AudioPosition::default(),
        }
    }

    /// builds a LSB Audio Encoder that implements Write
    pub fn encoder<'i, 'o, I: Read, O: Write + Seek>(
        input: &'i mut WavReader<I>,
        output: &'o mut WavWriter<O>,
    ) -> LSBAudioEncoder<'i, 'o, I, O> {
        LSBEncoder {
            input,
            output,
            position: AudioPosition::default(),
        }
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
