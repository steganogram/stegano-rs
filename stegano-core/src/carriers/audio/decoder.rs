use crate::universal_decoder::CarrierItem;
use hound::WavReader;
use std::io::Read;

/// stegano source for wav audio files, based on `WavReader` by `hound` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use hound::WavReader;
/// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
/// use stegano_core::carriers::audio::decoder::AudioWavSource;
///
/// // create a `WavReader` from an audio file
/// let mut reader = WavReader::open("../resources/secrets/audio-with-secrets.wav")
///     .expect("Cannot open secret audio file with WavReader");
/// let mut secret = vec![0; 12];
///
/// // create a `Decoder` based on an `AudioWavSource` based on the `WavReader`
/// Decoder::new(AudioWavSource::new(&mut reader), OneBitUnveil)
///     .read_exact(&mut secret)
///     .expect("Cannot read 12 bytes from decoder");
///
/// let msg = String::from_utf8(secret).expect("Cannot convert result to string");
/// assert_eq!("Hello World!", msg);
/// ```
pub struct AudioWavSource<'i, I> {
    pub input: &'i mut WavReader<I>,
}

impl<'i, I> AudioWavSource<'i, I>
where
    I: Read,
{
    /// constructor for a given `WavReader` that lives somewhere
    pub fn new(input: &'i mut WavReader<I>) -> Self {
        Self { input }
    }
}

/// iteratoes over the audio samples and returns each wrapped into a `CarrierItem`
impl<'i, I> Iterator for AudioWavSource<'i, I>
where
    I: Read,
{
    type Item = CarrierItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Ok(sample)) = self.input.samples::<i16>().next() {
            Some(CarrierItem::SignedTwoByte(sample))
        } else {
            None
        }
    }
}
