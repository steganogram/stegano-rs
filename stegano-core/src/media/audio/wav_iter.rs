use crate::{MediaPrimitive, MediaPrimitiveMut};
use std::slice::IterMut;

/// iterating wav audio samples, based on `WavReader` by `hound` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use hound::WavReader;
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::media::audio::wav_iter::{AudioWavIterMut, AudioWavIter};
///
/// // create a `WavReader` from an audio file
/// let mut reader = WavReader::open("../resources/secrets/audio-with-secrets.wav")
///     .expect("Cannot open secret audio file with WavReader");
/// let mut secret = vec![0; 12];
///
/// // create a `Decoder` based on a Iterator
/// Decoder::new(AudioWavIter::new(reader.samples::<i16>().map(|s|s.unwrap())), OneBitUnveil)
///     .read_exact(&mut secret)
///     .expect("Cannot read 12 bytes from decoder");
///
/// let msg = String::from_utf8(secret).expect("Cannot convert result to string");
/// assert_eq!("Hello World!", msg);
/// ```
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
        self.samples.next().map(|s| MediaPrimitive::AudioSample(s))
    }
}

/// iterating mutable wav audio samples, based on `WavReader` by `hound` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use hound::WavReader;
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::media::audio::wav_iter::AudioWavIter;
/// use stegano_core::media::audio::read_samples;
///
/// let secret_file = Path::new("../resources/secrets/audio-with-secrets.wav");
/// // create a `WavReader` from an audio file
/// let mut reader = WavReader::open(&secret_file)
///     .expect("Cannot open secret audio file with WavReader");
/// let mut secret = vec![0; 12];
///
/// // collect all samples in a Vec<i16>
/// let (mut samples, _) = read_samples(&secret_file);
///
/// Decoder::new(AudioWavIter::new(samples.into_iter()), OneBitUnveil)
///     .read_exact(&mut secret)
///     .expect("Cannot read 12 bytes from decoder");
///
/// let msg = String::from_utf8(secret).expect("Cannot convert result to string");
/// assert_eq!("Hello World!", msg);
/// ```
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
        self.samples
            .next()
            .map(|s| MediaPrimitiveMut::AudioSample(s))
    }
}
