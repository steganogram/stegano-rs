use std::io::Result;

/// trait for keep track of position on encode and decode
pub trait Position {
    /// returns the current position
    fn current(&self) -> usize;
    /// moves to the next position
    fn next(&mut self);
    /// moves to the previous position
    fn prev(&mut self);
}

/// generic hiding algorithms, used for specific ones like LSB
pub trait HideAlgorithm<T> {
    /// encodes one bit onto a carrier T e.g. u8 or i16
    fn encode(&self, carrier: T, information: &Result<bool>) -> T;
}

#[derive(Default)]
pub(crate) struct LSBAlgorithm;

impl HideAlgorithm<i16> for LSBAlgorithm {
    #[inline(always)]
    fn encode(&self, carrier: i16, information: &Result<bool>) -> i16 {
        match information {
            Err(_) => dbg!(carrier),
            Ok(bit) => (carrier & 0x7FFE) | if *bit { 1 } else { 0 },
        }
    }
}

impl HideAlgorithm<u8> for LSBAlgorithm {
    #[inline(always)]
    fn encode(&self, carrier: u8, information: &Result<bool>) -> u8 {
        match information {
            Err(_) => carrier,
            Ok(bit) => (carrier & (u8::MAX - 1)) | if *bit { 1 } else { 0 },
        }
    }
}

/// generic unveil algorithm
pub trait UnveilAlgorithm<T> {
    fn decode(&self, carrier: T) -> bool;
}

/// unveil algorithm for i16
impl UnveilAlgorithm<i16> for LSBAlgorithm {
    #[inline(always)]
    fn decode(&self, carrier: i16) -> bool {
        (carrier & 0x1) > 0
    }
}

/// unveil algorithm for u8
impl UnveilAlgorithm<u8> for LSBAlgorithm {
    #[inline(always)]
    fn decode(&self, carrier: u8) -> bool {
        (carrier & 0x1) > 0
    }
}
