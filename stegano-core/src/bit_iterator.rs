use std::io::prelude::*;
use std::io::*;
use std::slice;

pub struct BitIterator<I> {
    n: u32,
    i: u32,
    iter: I,
    byte: Option<u8>,
}

impl<I> BitIterator<I> {
    pub fn new(s: I) -> Self {
        BitIterator {
            n: 8,
            i: 0,
            iter: s,
            byte: None,
        }
    }
}

impl<I> Iterator for BitIterator<I>
where
    I: Read,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let bit = (self.i % self.n) as u8;
            self.i += 1;
            if bit == 0 {
                self.byte = None;
            }
            if self.byte == None {
                let mut b = 0;
                match self.iter.read(slice::from_mut(&mut b)) {
                    Ok(0) => None,
                    Ok(..) => {
                        self.byte = Some(b);
                        self.byte
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(_) => None,
                };
            }
            return match self.byte {
                None => None,
                Some(b) => Some((b >> bit) & 1),
            };
        }
    }
}
