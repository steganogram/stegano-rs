use bitstream_io::{BitRead, BitReader, LittleEndian};
use std::io::{Cursor, Result, Write};

use crate::{HideBit, MediaPrimitive, MediaPrimitiveMut};

/// abstracting write back of a carrier item
pub trait WriteCarrierItem {
    fn write_carrier_item(&mut self, item: &MediaPrimitive) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

/// generic hiding algorithm, used for specific ones like LSB
pub trait HideAlgorithm<T> {
    /// encodes one bit onto a carrier T e.g. u8 or i16
    fn encode(&self, carrier: T, information: &Result<bool>) -> T;
}

/// generic stegano encoder
pub struct Encoder<'c, C>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
{
    pub carrier: C,
}

impl<'c, C> Encoder<'c, C>
where
    C: Iterator<Item = MediaPrimitiveMut<'c>>,
{
    pub fn new(carrier: C) -> Self {
        Encoder { carrier }
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
        for s in self.carrier.by_ref().take(items_to_take) {
            s.hide_bit(bit_iter.read_bit().unwrap()).unwrap();
            bit_written += 1;
        }

        Ok(bit_written >> 3)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// default 1 bit hiding strategy
pub struct OneBitHide;
impl HideAlgorithm<MediaPrimitive> for OneBitHide {
    fn encode(&self, carrier: MediaPrimitive, information: &Result<bool>) -> MediaPrimitive {
        match information {
            Err(_) => carrier,
            Ok(bit) => match carrier {
                MediaPrimitive::ImageColorChannel(b) => MediaPrimitive::ImageColorChannel(
                    (b & (u8::MAX - 1)) | if *bit { 1 } else { 0 },
                ),
                MediaPrimitive::AudioSample(b) => {
                    MediaPrimitive::AudioSample((b & (i16::MAX - 1)) | if *bit { 1 } else { 0 })
                }
            },
        }
    }
}
