use std::slice::IterMut;

use crate::media::{MediaPrimitive, MediaPrimitiveMut};

/// Iterating wav audio samples, based on `WavReader` by `hound` crate
pub struct AudioWavIter<T> {
    samples: T,
}

impl<T> AudioWavIter<T>
where
    T: Iterator<Item = i16>,
{
    pub fn new(samples: T) -> Self {
        Self { samples }
    }
}

/// Audio samples iterator that yields immutable MediaPrimitives `MediaPrimitive`
impl<T> Iterator for AudioWavIter<T>
where
    T: Iterator<Item = i16>,
{
    type Item = MediaPrimitive;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.next().map(MediaPrimitive::AudioSample)
    }
}

/// Iterating mutable wav audio samples, based on `WavReader` by `hound` crate
pub struct AudioWavIterMut<'a, T> {
    samples: IterMut<'a, T>,
}

impl<'a> AudioWavIterMut<'a, i16> {
    pub fn new(samples: IterMut<'a, i16>) -> Self {
        Self { samples }
    }
}

/// Audio samples iterator that yields mutable MediaPrimitives `MediaPrimitiveMut`
impl<'a> Iterator for AudioWavIterMut<'a, i16> {
    type Item = MediaPrimitiveMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.next().map(MediaPrimitiveMut::AudioSample)
    }
}

#[cfg(test)]
mod tests {
    use hound::{WavReader, WavSpec};
    use std::{io::Read, path::Path};

    use crate::universal_decoder::{OneBitUnveil, UniversalDecoder};

    use super::*;

    /// convenient wrapper for `WavReader::open`
    fn read_samples(file: &Path) -> (Vec<i16>, WavSpec) {
        let mut reader = WavReader::open(file).expect("Cannot create reader");
        (
            reader.samples().map(|s| s.unwrap()).collect(),
            reader.spec(),
        )
    }

    #[test]
    fn test_wav_iter_with_decoder() {
        // create a `WavReader` from an audio file
        let mut reader = WavReader::open("tests/audio/secrets/audio-with-secrets.wav")
            .expect("Cannot open secret audio file with WavReader");
        let mut secret = vec![0; 12];

        // create a `Decoder` based on a Iterator
        UniversalDecoder::new(
            AudioWavIter::new(reader.samples::<i16>().map(|s| s.unwrap())),
            OneBitUnveil,
        )
        .read_exact(&mut secret)
        .expect("Cannot read 12 bytes from decoder");

        let msg = String::from_utf8(secret).expect("Cannot convert result to string");
        assert_eq!("Hello World!", msg);
    }

    #[test]
    fn test_wav_iter_mut() {
        let secret_file = Path::new("tests/audio/secrets/audio-with-secrets.wav");
        let mut secret = vec![0; 12];

        // collect all samples in a Vec<i16>
        let (samples, _) = read_samples(secret_file);

        UniversalDecoder::new(AudioWavIter::new(samples.into_iter()), OneBitUnveil)
            .read_exact(&mut secret)
            .expect("Cannot read 12 bytes from decoder");

        let msg = String::from_utf8(secret).expect("Cannot convert result to string");
        assert_eq!("Hello World!", msg);
    }
}
