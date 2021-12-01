use bitstream_io::{BitWrite, BitWriter, LittleEndian};
use enum_dispatch::enum_dispatch;
use std::io::{BufWriter, Read, Result};

use crate::MediaPrimitive;

#[enum_dispatch]
pub enum UnveilAlgorithms {
    OneBitUnveil,
}

/// generic unveil algorithm
#[enum_dispatch(UnveilAlgorithms)]
pub trait UnveilAlgorithm {
    fn decode(&self, carrier: MediaPrimitive) -> bool;
}

/// generic stegano decoder
pub struct Decoder<I, A>
where
    I: Iterator<Item = MediaPrimitive>,
    A: UnveilAlgorithm,
{
    pub input: I,
    pub algorithm: A,
}

/// generic stegano decoder constructor method
impl<I, A> Decoder<I, A>
where
    I: Iterator<Item = MediaPrimitive>,
    A: UnveilAlgorithm,
{
    pub fn new(input: I, algorithm: A) -> Self {
        Decoder { input, algorithm }
    }
}

impl<I, A> Read for Decoder<I, A>
where
    I: Iterator<Item = MediaPrimitive>,
    A: UnveilAlgorithm,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // TODO better let the algorithm determine the density of decoding
        let items_to_take = buf.len() << 3; // 1 bit per carrier item
        let buf_writer = BufWriter::new(buf);
        let mut bit_buffer = BitWriter::endian(buf_writer, LittleEndian);

        let mut bit_read: usize = 0;
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

        Ok(bit_read >> 3)
    }
}

/// default 1 bit unveil strategy
#[derive(Debug)]
pub struct OneBitUnveil;
impl UnveilAlgorithm for OneBitUnveil {
    #[inline]
    fn decode(&self, carrier: MediaPrimitive) -> bool {
        match carrier {
            MediaPrimitive::ImageColorChannel(b) => (b & 0x1) > 0,
            MediaPrimitive::AudioSample(b) => (b & 0x1) > 0,
        }
    }
}
