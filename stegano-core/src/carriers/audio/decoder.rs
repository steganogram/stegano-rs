use crate::lsb::{HideAlgorithm, UnveilAlgorithm};
use crate::universal_decoder::CarrierItem;
use bitstream_io::{BitWriter, LittleEndian};
use hound::{Error, WavReader};
use std::io::{BufWriter, Read, Result};

/// stegano source for wav audio files, based on `WavReader` by `hound` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use hound::{WavReader, WavWriter};
/// use stegano_core::carriers::audio::decoder::{AudioWavSource, WavUnveil};
/// use stegano_core::universal_decoder::Decoder;
///
/// // create a `WavReader` from an audio file
/// let mut reader = WavReader::open("../resources/secrets/audio-with-secrets.wav")
///     .expect("Cannot open secret audio file with WavReader");
/// let mut secret = vec![0; 12];
///
/// // create a `Decoder` based on an `AudioWavSource` based on the `WavReader`
/// Decoder::new(AudioWavSource::new(&mut reader), WavUnveil)
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
    pub fn new(input: &'i mut WavReader<I>) -> Self {
        Self { input }
    }
}

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

/// wav specific implementation for unveil of data
pub struct WavUnveil;
impl UnveilAlgorithm<CarrierItem> for WavUnveil {
    #[inline(always)]
    fn decode(&self, carrier: CarrierItem) -> bool {
        match carrier {
            CarrierItem::UnsignedByte(b) => (b & 0x1) > 0,
            CarrierItem::SignedTwoByte(b) => (b & 0x1) > 0,
        }
    }
}
