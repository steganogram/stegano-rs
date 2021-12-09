use bitstream_io::{BitRead, BitReader, LittleEndian};
use enum_dispatch::enum_dispatch;
use std::io::{Cursor, Result, Write};

use crate::{MediaPrimitive, MediaPrimitiveMut};

/// abstracting write back of a carrier item
pub trait WriteCarrierItem {
    fn write_carrier_item(&mut self, item: &MediaPrimitive) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

#[enum_dispatch]
pub enum HideAlgorithms {
    OneBitHide,
    OneBitInLowFrequencyHide,
}

/// generic hiding algorithm, used for specific ones like LSB
#[enum_dispatch(HideAlgorithms)]
pub trait HideAlgorithm {
    /// encodes one bit onto a carrier T e.g. u8 or i16
    fn encode<'c>(&self, carrier: MediaPrimitiveMut<'c>, information: &Result<bool>);
}

/// generic stegano encoder
pub struct Encoder<'c, C, A>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
    A: HideAlgorithm,
{
    pub carrier: C,
    pub algorithm: A,
}

impl<'c, C, A> Encoder<'c, C, A>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
    A: HideAlgorithm,
{
    pub fn new(carrier: C, algorithm: A) -> Self {
        Encoder { carrier, algorithm }
    }
}

impl<'c, C, A> Write for Encoder<'c, C, A>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
    A: HideAlgorithm,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // TODO better let the algorithm determine the density of encoding
        let items_to_take = buf.len() << 3; // 1 bit per sample <=> * 8 <=> << 3
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);
        let mut bit_written: usize = 0;
        for s in self.carrier.by_ref().take(items_to_take) {
            self.algorithm.encode(s, &bit_iter.read_bit());
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
impl HideAlgorithm for OneBitHide {
    #[inline(always)]
    fn encode<'c>(&self, carrier: MediaPrimitiveMut<'c>, information: &Result<bool>) {
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
impl HideAlgorithm for OneBitInLowFrequencyHide {
    #[inline(always)]
    fn encode<'c>(&self, carrier: MediaPrimitiveMut<'c>, information: &Result<bool>) {
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
            encoder.encode(
                mp,
                &Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)),
            );
        }
        assert_eq!(data, 0b00001110);
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
