use bitstream_io::{BitRead, BitReader, LittleEndian};
use std::io::{Cursor, Result, Write};

use crate::{MediaPrimitive, MediaPrimitiveMut};

/// abstracting write back of a carrier item
pub trait WriteCarrierItem {
    fn write_carrier_item(&mut self, item: &MediaPrimitive) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

/// generic hiding algorithm, used for specific ones like LSB
pub trait HideAlgorithm<T> {
    /// encodes one bit onto a carrier T e.g. u8 or i16
    fn encode(&self, carrier: T, information: &Result<bool>);
}

/// generic stegano encoder
pub struct Encoder<'c, C>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
{
    pub carrier: C,
    pub algorithm: Box<dyn HideAlgorithm<MediaPrimitiveMut<'c>>>,
}

impl<'c, C> Encoder<'c, C>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
{
    pub fn new(carrier: C, algorithm: Box<dyn HideAlgorithm<MediaPrimitiveMut<'c>>>) -> Self {
        Encoder { carrier, algorithm }
    }
}

impl<'c, C> Write for Encoder<'c, C>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // TODO better let the algorithm determine the density of encoding
        let items_to_take = buf.len() << 3; // 1 bit per sample <=> * 8 <=> << 3
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);
        let mut bit_written: usize = 0;
        let enc = self.algorithm.as_ref();
        for s in self.carrier.by_ref().take(items_to_take) {
            enc.encode(s, &bit_iter.read_bit());
            bit_written += 1;
        }

        Ok(bit_written >> 3)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// default 1 bit hiding strategy
#[derive(Debug)]
pub struct OneBitHide;
impl<'c> HideAlgorithm<MediaPrimitiveMut<'c>> for OneBitHide {
    fn encode(&self, carrier: MediaPrimitiveMut<'c>, information: &Result<bool>) {
        if let Ok(bit) = information {
            match carrier {
                MediaPrimitiveMut::ImageColorChannel(b) => {
                    *b = ((*b) & (u8::MAX - 1)) | if *bit { 1 } else { 0 }
                }
                MediaPrimitiveMut::AudioSample(b) => {
                    *b = ((*b) & (i16::MAX - 1)) | if *bit { 1 } else { 0 }
                }
                _ => {}
            }
        }
    }
}

/// 1 bit hiding strategy, but
#[derive(Debug)]
pub struct OneBitInLowFrequencyHide;
impl<'c> HideAlgorithm<MediaPrimitiveMut<'c>> for OneBitInLowFrequencyHide {
    fn encode(&self, carrier: MediaPrimitiveMut<'c>, information: &Result<bool>) {
        if let Ok(bit) = information {
            match carrier {
                MediaPrimitiveMut::ImageColorChannel(b) => {
                    *b = ((*b) & 0b11110000) | if *bit { 0b00001111 } else { 0 }
                }
                MediaPrimitiveMut::AudioSample(b) => {
                    *b = ((*b) & (0b11111111 << 8)) | if *bit { 0b000000011111111 } else { 0 }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_in_lower_frequencies() {
        let encoder = OneBitInLowFrequencyHide;
        let mut data = 0b11001101;
        {
            let mp = MediaPrimitiveMut::ImageColorChannel(&mut data);
            encoder.encode(mp, &Ok(true));
        }
        assert_eq!(data, 0b11001111);
    }

    #[test]
    fn should_not_harm_on_error() {
        let encoder = OneBitHide;
        let mut data = 0b00001110;
        {
            let mp = MediaPrimitiveMut::ImageColorChannel(&mut data);
            encoder.encode(mp, &Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)));
        }
        assert_eq!(
            data,
            0b00001110
        );
    }

    #[test]
    fn should_encode_one_bit() {
        let encoder = OneBitHide;
        let mut data = 0b00001110;
        {
            let mp = MediaPrimitiveMut::ImageColorChannel(&mut data);
            encoder.encode(mp, &Ok(true));
        }
        assert_eq!(data, 0b00001111);

        let mut data = 0b00001110;
        {
            let mp = MediaPrimitiveMut::AudioSample(&mut data);
            encoder.encode(mp, &Ok(true));
        }
        assert_eq!(data, 0b00001111);

        let mut data = 0b00001110;
        {
            let mp = MediaPrimitiveMut::ImageColorChannel(&mut data);
            encoder.encode(mp, &Ok(false));
        }
        assert_eq!(data, 0b00001110);

        let mut data = 0b00001110;
        {
            let mp = MediaPrimitiveMut::AudioSample(&mut data);
            encoder.encode(mp, &Ok(false));
        }
        assert_eq!(data, 0b00001110);
    }
}
