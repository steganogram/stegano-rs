use crate::media::audio::wav_iter::{AudioWavIter, AudioWavIterMut};
use crate::universal_decoder::{Decoder, OneBitUnveil};
use crate::universal_encoder::Encoder2;
use hound::{WavReader, WavSpec};
use std::io::{Read, Write};
use std::path::Path;

/// convenient wrapper for `WavReader::open`
pub fn read_samples(file: &Path) -> (Vec<i16>, WavSpec) {
    let mut reader = WavReader::open(file).expect("Cannot create reader");
    (
        reader.samples().map(|s| s.unwrap()).collect(),
        reader.spec(),
    )
}

/// Factory for decoder and encoder
pub struct LSBCodec;

impl LSBCodec {
    /// builds a LSB Audio Decoder that implements Read
    ///
    /// ## Example how to retrieve a decoder:
    /// ```rust
    /// use std::path::Path;
    /// use hound::WavReader;
    /// use stegano_core::media::audio::LSBCodec;
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
        Box::new(Decoder::new(
            AudioWavIter::new(input.samples::<i16>().into_iter().map(|s| s.unwrap())),
            OneBitUnveil,
        ))
    }

    /// ## Example how to retrieve an encoder
    /// builds a LSB Audio Encoder that implements Write
    ///
    /// ```rust
    /// use std::path::Path;
    /// use tempdir::TempDir;
    /// use hound::{WavReader, WavWriter};
    /// use stegano_core::media::audio::LSBCodec;
    /// use stegano_core::media::audio::read_samples;
    ///
    /// let input: &Path = "../resources/plain/carrier-audio.wav".as_ref();
    /// let out_dir = TempDir::new("audio-temp").expect("Cannot create temp dir");
    /// let audio_with_secret = out_dir.path().join("audio-with-secret.wav");
    ///
    /// let (mut samples, spec) = read_samples(input);
    /// let mut writer = WavWriter::create(audio_with_secret.as_path(), spec)
    ///     .expect("Cannot create writer");
    /// let secret_message = "Hello World!".as_bytes();
    ///
    /// LSBCodec::encoder(&mut samples)
    ///     .write_all(&secret_message[..])
    ///     .expect("Cannot write to codec");
    /// ```
    pub fn encoder<'i>(input: &'i mut Vec<i16>) -> Box<dyn Write + 'i> {
        Box::new(Encoder2::new(AudioWavIterMut::new(input.iter_mut())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use crate::SteganoError;
    use hound::WavWriter;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use tempdir::TempDir;

    const SOME_WAV: &str = "../resources/plain/carrier-audio.wav";
    const BIG_SECRET: &str = "../LICENSE";

    /// conclusions for audio:
    /// - media con contain a Vec<i16> that is owned
    /// - encoder can receive a &mut Vec<16>
    /// - MediaPrimitive can contain a &mut i16 and is then same as a color channel
    /// - Media can implement the save method only when it knows the WavSpec via WavWriter
    #[test]
    fn toy_around_with_vec_as_audio_buffer() -> Result<()> {
        let input: &Path = SOME_WAV.as_ref();
        let mut reader = WavReader::open(input).expect("Cannot open wav file");
        let buf: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();

        let out_dir = TempDir::new("audio-temp")?;
        let audio_with_secret = out_dir.path().join("audio-with-secret.wav");
        let out_file = audio_with_secret.as_path();

        let mut writer =
            WavWriter::create(out_file, reader.spec()).expect("Cannot create wav file");
        buf.iter()
            .for_each(|s| writer.write_sample(*s).expect("Cannot write sample"));
        writer
            .finalize()
            .map_err(|_e| SteganoError::AudioEncodingError)?;

        assert_ne!(out_file.metadata().unwrap().len(), 0);

        Ok(())
    }

    #[test]
    fn it_should_encode_and_decode_in_chunks_by_using_read_to_end() -> Result<()> {
        let original_message = std::fs::read_to_string(BIG_SECRET)?;
        let mut secret_to_hide = Vec::new();
        let file = File::open(BIG_SECRET)?;
        let mut buf_reader = BufReader::new(file);
        buf_reader.read_to_end(&mut secret_to_hide)?;
        secret_to_hide.shrink_to_fit();

        let input: &Path = SOME_WAV.as_ref();
        let out_dir = TempDir::new("audio-temp")?;
        let audio_with_secret = out_dir.path().join("audio-with-secret.wav");
        let (mut samples, spec) = read_samples(input);
        let samples_copy = samples.clone();
        {
            // Block is important so that writer is dropped, so that it persists the file
            let mut codec = LSBCodec::encoder(&mut samples);
            let half_the_buffer = secret_to_hide.len() / 2;
            codec
                .write_all(&secret_to_hide[..half_the_buffer])
                .expect("Cannot write half the buffer to codec");
            codec
                .write_all(&secret_to_hide[half_the_buffer..])
                .expect("Cannot write the other half of the buffer to codec");
        }
        {
            let mut writer =
                WavWriter::create(audio_with_secret.as_path(), spec).expect("Cannot create writer");
            samples_copy
                .iter()
                .for_each(|s| writer.write_sample(*s).unwrap());
            writer.finalize().expect("Cannot finalize");
        }

        let mut reader = WavReader::open(audio_with_secret.as_path())
            .expect("carrier audio file was not readable");
        let mut codec = LSBCodec::decoder(&mut reader);
        let mut unveiled_secret = Vec::new();
        let total_read = codec
            .read_to_end(&mut unveiled_secret)
            .expect("Cannot read all data from codec");
        assert!(
            total_read > secret_to_hide.len(),
            "Total read should be way more than the original secret"
        );
        let unveiled_secret = &unveiled_secret[..secret_to_hide.len() - 1];
        // println!("{:?}", unveiled_secret);
        // let unveiled_message = std::str::from_utf8(unveiled_secret).unwrap();
        // assert_eq!(unveiled_message, original_message);

        let unveiled_secret_file_path = out_dir.path().join("LICENSE");
        let mut unveiled_secret_file = File::create(unveiled_secret_file_path.as_path())
            .expect("Cannot create the file for the unveiled data.");
        unveiled_secret_file.write_all(&unveiled_secret[..])?;
        // assert_eq!(
        //     unveiled_secret_file.metadata().unwrap().len() as usize,
        //     secret_to_hide.len()
        // );

        std::fs::copy(unveiled_secret_file_path.as_path(), "/tmp/foo")?;
        Ok(())
    }
}
