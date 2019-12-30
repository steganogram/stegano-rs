use std::io::prelude::*;
use std::io::{Read, Result, Cursor, SeekFrom, BufReader, BufWriter};

struct FilterReader<T> {
    inner: T,
    first_byte: Option<u8>,
    first_read: bool,
}

impl<T> Read for FilterReader<T>
    where T: Read + Sized
{
    fn read(&mut self, b: &mut [u8]) -> Result<usize> {
        match self.first_byte {
            Some(0x1) => self.read_until_eof(b),
            Some(byt) => self.read_like_raw(b, &byt),
            None => self.read_first_byte_and_further(b),
        }
    }
}


impl<T> FilterReader<T>
    where T: Read + Sized
{
    fn new(reader: T) -> Self {
        FilterReader {
            inner: reader,
            first_byte: None,
            first_read: true,
        }
    }

    fn read_until_eof(&mut self, b: &mut [u8]) -> Result<usize> {
        /// there are some issues with Vec, so vec! to the rescue
        /// https://www.reddit.com/r/rust/comments/4m1snt/issues_with_read_and_vecu8/
        ///
        let mut buf = vec![0; b.len()];
        let mut bytes = 0;
        let mut eof = 0;
        let s = self.inner.read(&mut buf).unwrap();
        for i in 0..s {
            if eof == 2 {
                return Ok(bytes);
            }
            let byt = buf[i];
            if byt == 0xff {
                eof += 1;
                continue;
            }
            eof = 0;
            bytes += 1;
            b[i] = byt;
        }
        Ok(bytes)
    }

    fn read_first_byte_and_further(&mut self, b: &mut [u8]) -> Result<usize> {
        let mut buf = [0 as u8; 1];
        let r = self.inner.read(&mut buf)
            .expect("Failed to read first byte from inner");
        if r < 1 {
            return Ok(r);   // done reading
        }
        self.first_byte = Some(buf[0]);

        self.read(b)
    }

    fn read_like_raw(&mut self, b: &mut [u8], byt: &u8) -> Result<usize> {
        let mut bytes = 0;
        if self.first_read {
            let mut buf = vec![0; b.len() - 1];
            bytes = self.inner.read(&mut buf).unwrap() + 1;
            let mut w = BufWriter::new(b);
            w.write(&[*byt]).expect("cannot write first byte");
            w.write_all(&mut buf).expect("cannot write other bytes");
            w.flush().expect("cannot flush buffer");

            self.first_read = false;
        } else {
            bytes = self.inner.read(b).unwrap();
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    const buf: [u8; 6] = [0x1, b'H', b'e', 0xff, 0xff, 0xcd];

    #[test]
    fn should_skip_the_first_byte_when_first_byte_is_0x01() {
        let r = BufReader::new(&buf[..]);
        let mut f = FilterReader::new(r);

        let mut b = [0; 2];
        let read = f.read(&mut b).unwrap();
        assert_eq!(read, 2);
        assert_eq!(b[0], b'H', "first byte was not 'H'");
        assert_eq!(b[1], b'e', "2nd byte was not 'e'");
    }

    #[test]
    fn should_find_eof_at_0xffff_when_first_byte_is_0x01() {
        let r = BufReader::new(&buf[..]);
        let mut f = FilterReader::new(r);

        let mut b = [0; 4];
        let read = f.read(&mut b).unwrap();
        assert_eq!(read, 2);
        assert_eq!(b[0], b'H', "first byte was not 'H'");
        assert_eq!(b[1], b'e', "2nd byte was not 'e'");
        assert_eq!(b[2], 0, "3nd byte was not kept as 0");
    }

    #[test]
    fn should_read_plain_when_no_special_fist_byte_given() {
        let b = [0xef, b'H', b'e', 0xcd, 0xab];
        let r = BufReader::new(&b[..]);
        let mut f = FilterReader::new(r);

        let mut b = [0; 6];
        let read = f.read(&mut b).unwrap();
        assert_eq!(read, 5);
        assert_eq!(b[0], 0xef, "first byte was not 0xef");
        assert_eq!(b[1], b'H', "2nd byte was not 'H'");
        assert_eq!(b[2], b'e', "3rd byte was not 'e'");
        assert_eq!(b[3], 0xcd, "4th byte was not 0xcd");
        assert_eq!(b[4], 0xab, "5th byte was not 0xad");
    }

    #[test]
    fn should_read_plain_when_no_special_fist_byte_given_and_continue() {
        let b = [0xef, b'H', b'e', 0xcd, 0xab];
        let r = BufReader::new(&b[..]);
        let mut f = FilterReader::new(r);

        let mut b = [0; 3];
        let read = f.read(&mut b).unwrap();
        assert_eq!(read, 3);
        assert_eq!(b[0], 0xef, "first byte was not 0xef");
        assert_eq!(b[1], b'H', "2nd byte was not 'H'");
        assert_eq!(b[2], b'e', "3rd byte was not 'e'");
        let mut b = [0; 3];
        let read = f.read(&mut b).unwrap();
        assert_eq!(read, 2);
        assert_eq!(b[0], 0xcd, "4th byte was not 0xcd");
        assert_eq!(b[1], 0xab, "5th byte was not 0xad");
    }
}
