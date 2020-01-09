use std::io::prelude::Write;
use futures::io::Error;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::io::{BufWriter, Cursor, Read, Take};
use std::borrow::BorrowMut;

pub struct Codec<W> {
    inner: Option<W>,
    payload_reader: Option<Take<W>>,
    buffer: BufWriter<Vec<u8>>,
    byte_count: u32,
}

impl<W> Codec<W>
    where W: Write
{
    pub fn encoder(writer: W) -> Self {
        Codec {
            inner: Some(writer),
            payload_reader: None,
            buffer: BufWriter::new(Vec::new()),
            byte_count: 0,
        }
    }
}

impl<W> Write for Codec<W>
    where W: Write
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let r = self.buffer.write(buf);
        match r {
            Ok(bytes) => {
                self.byte_count += bytes as u32;
                Ok(bytes)
            }
            Err(e) => Err(e)
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.buffer.flush()
            .expect("Cannot flush inner buffer.");

        self.inner.as_mut().unwrap().write_u8(0x04)
            .expect("Cannot write version header.");

        self.inner.as_mut().unwrap().write_u32::<BigEndian>(self.byte_count)
            .expect("Cannot write payload size header.");

        let mut target = self.inner.as_mut().unwrap();
        std::io::copy(&mut Cursor::new(self.buffer.get_mut()),
                      target)
            .expect("Error during copy the payload from buffer to target");

        self.inner.as_mut().unwrap().flush()
            .expect("Error during flush of the inner writer of Codec");

        self.buffer = BufWriter::new(Vec::new());
        self.byte_count = 0;

        Ok(())
    }
}

impl<R> Codec<R>
    where R: Read
{
    pub fn decoder(reader: R) -> Self {
        Codec {
            inner: Some(reader),
            payload_reader: None,
            buffer: BufWriter::new(Vec::new()),
            byte_count: 0,
        }
    }
}

impl<R> Read for Codec<R>
    where R: Read
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.payload_reader {
            None => {
                let version = self.inner.as_mut().unwrap().read_u8()
                    .expect("Failed to read version header");

                match version {
                    0x04 => {
                        let payload_size = self.inner.as_mut().unwrap().read_u32::<BigEndian>()
                            .expect("Failed to read payload size header");

                        // TODO find a way to limit the reading on self.inner without hacks
                        let mut i = self.inner.take().unwrap();
                        self.inner = None;

                        let payload = i
                            .take(payload_size as u64);
                        self.payload_reader = Some(payload);
                    }
                    _ => unimplemented!("only format version 0x04 is supported")
                }
                self.byte_count += 5;
            }
            _ => {}
        }

        self.payload_reader.as_mut().unwrap().read(buf)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Write, Cursor};
    use byteorder::ReadBytesExt;
    use super::*;

    #[test]
    fn should_write_version_and_size_header() {
        let some_data = vec![b'S', b'o', b'm', b'e', 0x3c, 0xab, 0x0c];
        let mut buf = Vec::new();

        write_to_codec(&mut buf, &some_data);

        let mut s = Cursor::new(&mut buf[1..]);
        let payload_size = s.read_u32::<BigEndian>().unwrap();

        assert_eq!(buf[0], 0x04, "1st Byte should contain version header");
        assert_eq!(payload_size, some_data.len() as u32, "2nd 4 Bytes should contain size");
        assert_eq!(&buf[5..], &some_data[..], "Payload should equal");
    }

    #[test]
    fn should_read_payload() {
        let mut some_data = vec![0x04, 0x0, 0x0, 0x0, 0x02, b'H', 0xab, 0x3e, 0xff];
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut some_data[..]);

        let mut codec = Codec::decoder(&mut cursor);
        let r = codec.read_to_end(&mut buf)
            .expect("Reading from Codec failed.");

        buf.shrink_to_fit();
        assert_eq!(r, 2, "Read bytes not as expected");
        assert_eq!(buf.len(), 2, "Payload was not as long as expected");
    }

    #[inline]
    fn write_to_codec(buf: &mut Vec<u8>, some_data: &[u8]) {
        let mut writer = BufWriter::new(buf);
        let mut codec = Codec::encoder(&mut writer);
        codec.write(&some_data[..])
            .expect("Codec::write_all failed.");
        codec.flush();
    }
}