use std::io::{Read, Write};
use std::path::Path;

use hound::{WavReader, WavSpec};

use crate::media::audio::wav_iter::{AudioWavIter, AudioWavIterMut};
use crate::universal_decoder::{Decoder, OneBitUnveil};
use crate::universal_encoder::Encoder2;

/// convenient wrapper for `WavReader::open`
pub fn read_samples(file: &Path) -> (Vec<i16>, WavSpec) {
    let mut reader = WavReader::open(file).expect("Cannot create reader");
    (
        reader.samples().map(|s| s.unwrap()).collect(),
        reader.spec(),
    )
}

/// Factory for decoder and encoder
pub struct LsbCodec;

impl LsbCodec {
    /// builds a LSB Audio Decoder that implements Read
    ///
    /// ## Example how to retrieve a decoder:
    /// ```rust
    /// use std::path::Path;
    /// use hound::WavReader;
    /// use stegano_core::media::audio::LsbCodec;
    ///
    /// let audio_with_secret: &Path = "../resources/secrets/audio-with-secrets.wav".as_ref();
    /// let mut reader = WavReader::open(audio_with_secret).expect("Cannot create reader");
    ///
    /// let mut buf = vec![0; 12];
    /// LsbCodec::decoder(&mut reader)
    ///     .read_exact(&mut buf[..])
    ///     .expect("Cannot read 12 bytes from codec");
    /// let msg = String::from_utf8(buf).expect("Cannot convert result to string");
    /// assert_eq!("Hello World!", msg);
    /// ```
    pub fn decoder<'i, I: Read>(input: &'i mut WavReader<I>) -> Box<dyn Read + 'i> {
        Box::new(Decoder::new(
            AudioWavIter::new(input.samples::<i16>().map(|s| s.unwrap())),
            OneBitUnveil,
        ))
    }

    /// ## Example how to retrieve an encoder
    /// builds a LSB Audio Encoder that implements Write
    ///
    /// ```rust
    /// use std::path::Path;
    /// use tempfile::TempDir;
    /// use hound::{WavReader, WavWriter};
    /// use stegano_core::media::audio::LsbCodec;
    /// use stegano_core::media::audio::read_samples;
    ///
    /// let input: &Path = "../resources/plain/carrier-audio.wav".as_ref();
    /// let out_dir = TempDir::new().expect("Cannot create temp dir");
    /// let audio_with_secret = out_dir.path().join("audio-with-secret.wav");
    ///
    /// let (mut samples, spec) = read_samples(input);
    /// let mut writer = WavWriter::create(audio_with_secret.as_path(), spec)
    ///     .expect("Cannot create writer");
    /// let secret_message = "Hello World!".as_bytes();
    ///
    /// LsbCodec::encoder(&mut samples)
    ///     .write_all(&secret_message[..])
    ///     .expect("Cannot write to codec");
    /// ```
    pub fn encoder<'i>(input: &'i mut Vec<i16>) -> Box<dyn Write + 'i> {
        Box::new(Encoder2::new(AudioWavIterMut::new(input.iter_mut())))
    }
}

#[cfg(test)]
mod tests {
    use hound::WavWriter;

    use tempfile::TempDir;

    use crate::Result;

    use super::*;

    const SOME_WAV: &str = "../resources/plain/carrier-audio.wav";
    const SECRET: &str = "../README.md";

    #[test]
    fn it_should_encode_and_decode_in_chunks_by_using_read_to_end() -> Result<()> {
        let out_dir = TempDir::new()?;
        let audio_with_secret_p = out_dir.path().join("audio-with-secret.wav");
        let audio_with_secret = audio_with_secret_p.as_path();

        let secret_to_hide_origin = std::fs::read(SECRET)?;
        let secret_to_hide = secret_to_hide_origin.clone();
        let (mut samples, spec) = read_samples(SOME_WAV.as_ref());
        {
            let mut codec = LsbCodec::encoder(&mut samples);
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
                WavWriter::create(audio_with_secret, spec).expect("Cannot create writer");
            samples
                .iter()
                .for_each(|s| writer.write_sample(*s).unwrap());
            writer.finalize().expect("Cannot finalize");
        }

        let mut reader =
            WavReader::open(audio_with_secret).expect("carrier audio file was not readable");
        let mut codec = LsbCodec::decoder(&mut reader);
        let mut unveiled_secret = Vec::new();
        let total_read = codec
            .read_to_end(&mut unveiled_secret)
            .expect("Cannot read all data from codec");
        assert!(
            total_read > secret_to_hide.len(),
            "Total read should be way more than the original secret"
        );
        let unveiled_secret = unveiled_secret[..secret_to_hide.len()].to_vec();
        assert_eq!(
            String::from_utf8(secret_to_hide_origin),
            String::from_utf8(unveiled_secret)
        );

        Ok(())
    }
}
