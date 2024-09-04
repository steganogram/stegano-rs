use std::io::{Read, Write};

use super::wav_iter::AudioWavIterMut;
use crate::media::MediaPrimitive;
use crate::universal_decoder::{OneBitUnveil, UniversalDecoder};
use crate::universal_encoder::{OneBitHide, UniversalEncoder};

/// Factory for decoder and encoder
pub struct LsbCodec;

impl LsbCodec {
    /// builds a LSB Audio Decoder that implements Read
    pub fn decoder<'i>(input: &'i [i16]) -> Box<dyn Read + 'i> {
        Box::new(UniversalDecoder::new(
            input
                .iter()
                .map(Clone::clone)
                .map(MediaPrimitive::AudioSample),
            OneBitUnveil,
        ))
    }

    /// builds a LSB Audio Encoder that implements Write
    pub fn encoder<'i>(input: &'i mut [i16]) -> Box<dyn Write + 'i> {
        Box::new(UniversalEncoder::new(
            AudioWavIterMut::new(input.iter_mut()),
            OneBitHide,
        ))
    }
}

#[cfg(feature = "benchmarks")]
#[allow(unused_imports)]    // clippy false positive
mod benchmarks {
    use super::LsbCodec;
    use crate::media::WavReader;

    /// Benchmark for audio decoding
    #[bench]
    pub fn audio_decoding(b: &mut test::Bencher) {
        let mut reader = WavReader::open("tests/audio/secrets/audio-with-secrets.wav")
            .expect("Cannot create reader");
        let samples = reader.samples().map(|s| s.unwrap()).collect::<Vec<i16>>();
        let mut buf = [0; 12];

        b.iter(|| {
            LsbCodec::decoder(&samples)
                .read_exact(&mut buf)
                .expect("Cannot read 12 bytes from decoder");
        })
    }

    /// Benchmark for audio encoding
    #[bench]
    pub fn audio_encoding(b: &mut test::Bencher) {
        let mut reader =
            WavReader::open("tests/audio/plain/carrier-audio.wav").expect("Cannot create reader");
        let mut samples = reader.samples().map(|s| s.unwrap()).collect::<Vec<i16>>();
        let secret_message = b"Hello World!";

        b.iter(|| {
            LsbCodec::encoder(&mut samples)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use hound::{WavReader, WavSpec, WavWriter};
    use tempfile::TempDir;

    const SOME_WAV: &str = "tests/audio/plain/carrier-audio.wav";

    /// convenient wrapper for `WavReader::open`
    fn read_samples(file: &Path) -> (Vec<i16>, WavSpec) {
        let mut reader = WavReader::open(file).expect("Cannot create reader");
        (
            reader.samples().map(|s| s.unwrap()).collect(),
            reader.spec(),
        )
    }

    #[test]
    fn it_should_encode_and_decode_in_chunks_by_using_read_to_end() {
        let out_dir = TempDir::new().unwrap();
        let audio_with_secret_p = out_dir.path().join("audio-with-secret.wav");
        let audio_with_secret = audio_with_secret_p.as_path();

        let secret_to_hide_origin = include_bytes!("lsb_codec.rs").to_vec();
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
        let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();
        let mut codec = LsbCodec::decoder(&samples);
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
    }
}
