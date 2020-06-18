use crate::lsb::{HideAlgorithm, UnveilAlgorithm};
use bitstream_io::{BitWriter, LittleEndian};
use hound::{Error, WavReader};
use std::io::{BufWriter, Read, Result};

pub enum CarrierItem {
    UnsignedByte(u8),
    SignedTwoByte(i16),
}

/// generic stegano decoder
pub struct Decoder<I, A>
where
    I: Iterator<Item = CarrierItem>,
    A: UnveilAlgorithm<CarrierItem>,
{
    pub input: I,
    pub algorithm: A,
}

/// generic stegano decoder constructor method
impl<I, A> Decoder<I, A>
where
    I: Iterator<Item = CarrierItem>,
    A: UnveilAlgorithm<CarrierItem>,
{
    pub fn new(input: I, algorithm: A) -> Self {
        Decoder { input, algorithm }
    }
}

impl<I, A> Read for Decoder<I, A>
where
    I: Iterator<Item = CarrierItem>,
    A: UnveilAlgorithm<CarrierItem>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // 1 bit per sample
        let items_to_take = buf.len() << 3;
        let buf_writer = BufWriter::new(buf);
        let mut bit_buffer = BitWriter::endian(buf_writer, LittleEndian);

        let mut bit_read = 0;
        for carrier in self.input.by_ref().take(items_to_take) {
            let bit = self.algorithm.decode(carrier);
            bit_buffer.write_bit(bit).expect("Cannot write bit n");
            bit_read += 1;
        }

        if !bit_buffer.byte_aligned() {
            bit_buffer
                .byte_align()
                .expect("Failed to align the last byte read from carrier.");
        }

        Ok(bit_read >> 3 as usize)
    }
}
