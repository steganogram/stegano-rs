use crate::CarrierItem;
use bitstream_io::{BitReader, LittleEndian};
use std::io::{Cursor, Result, Write};

/// abstracting write back of a carrier item
pub trait WriteCarrierItem {
    fn write_carrier_item(&mut self, item: &CarrierItem) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

/// generic hiding algorithm, used for specific ones like LSB
pub trait HideAlgorithm<T> {
    /// encodes one bit onto a carrier T e.g. u8 or i16
    fn encode(&self, carrier: T, information: &Result<bool>) -> T;
}

/// generic stegano encoder
pub struct Encoder<I, O, A>
where
    I: Iterator<Item = CarrierItem>,
    O: WriteCarrierItem,
    A: HideAlgorithm<CarrierItem>,
{
    pub input: I,
    pub output: O,
    pub algorithm: A,
}

/// generic stegano encoder constructor method
impl<I, O, A> Encoder<I, O, A>
where
    I: Iterator<Item = CarrierItem>,
    O: WriteCarrierItem,
    A: HideAlgorithm<CarrierItem>,
{
    pub fn new(input: I, output: O, algorithm: A) -> Self {
        Encoder {
            input,
            output,
            algorithm,
        }
    }
}

impl<I, O, A> Write for Encoder<I, O, A>
where
    I: Iterator<Item = CarrierItem>,
    O: WriteCarrierItem,
    A: HideAlgorithm<CarrierItem>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // TODO better let the algorithm determine the density of encoding
        let items_to_take = buf.len() << 3; // 1 bit per sample <=> * 8 <=> << 3
        let mut bit_iter = BitReader::endian(Cursor::new(buf), LittleEndian);
        let mut bit_written = 0;
        for s in self.input.by_ref().take(items_to_take) {
            let item: CarrierItem = self.algorithm.encode(s, &bit_iter.read_bit());
            self.output.write_carrier_item(&item).unwrap();
            bit_written += 1;
        }

        Ok(bit_written >> 3 as usize)
    }

    fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }
}

/// default 1 bit hiding strategy
pub struct OneBitHide;
impl HideAlgorithm<CarrierItem> for OneBitHide {
    fn encode(&self, carrier: CarrierItem, information: &Result<bool>) -> CarrierItem {
        match information {
            Err(_) => carrier,
            Ok(bit) => match carrier {
                CarrierItem::UnsignedByte(b) => {
                    CarrierItem::UnsignedByte((b & (u8::MAX - 1)) | if *bit { 1 } else { 0 })
                }
                CarrierItem::SignedTwoByte(b) => {
                    CarrierItem::SignedTwoByte((b & (i16::MAX - 1)) | if *bit { 1 } else { 0 })
                }
            },
        }
    }
}
