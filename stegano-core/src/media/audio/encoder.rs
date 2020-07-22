use crate::universal_encoder::WriteCarrierItem;
use crate::MediaPrimitive;
use hound::WavWriter;
use std::io;
use std::io::{Error, ErrorKind, Seek, Write};

pub struct AudioWavTarget<'t, T>
where
    T: Write + Seek,
{
    pub target: &'t mut WavWriter<T>,
}

impl<'t, T> AudioWavTarget<'t, T>
where
    T: Write + Seek,
{
    pub fn new(target: &'t mut WavWriter<T>) -> Self {
        AudioWavTarget { target }
    }
}

impl<'t, T> WriteCarrierItem for AudioWavTarget<'t, T>
where
    T: Write + Seek,
{
    fn write_carrier_item(&mut self, carrier_item: &MediaPrimitive) -> io::Result<usize> {
        match carrier_item {
            MediaPrimitive::ImageColorChannel(_) => {
                Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
            }
            MediaPrimitive::AudioSample(b) => match self.target.write_sample(*b) {
                Ok(_) => Ok(2),
                // TODO map the error somehow to std::io::ErrorKind
                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::Other)),
            },
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.target.flush().map_err(|e| {
            eprintln!(
                "{}\nchannels: {}\nbits per sample: {}",
                e,
                self.target.spec().channels,
                self.target.spec().bits_per_sample,
            );

            Error::from(ErrorKind::Other)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::WavReader;
    use std::fs::File;
    use std::io::{Cursor, Read};
    use std::path::Path;

    const SOME_WAV: &str = "../resources/plain/carrier-audio.wav";

    #[test]
    fn it_should_write_primitives() {
        let mut buffer = Vec::new();
        File::open(SOME_WAV.as_ref() as &Path)
            .expect("cannot read wav file")
            .read_to_end(&mut buffer)
            .expect("cannot read the whole file to buffer");
        let mut reader = WavReader::open(SOME_WAV.as_ref() as &Path).expect("Cannot create reader");
        {
            let mut buffer = Cursor::new(&mut buffer);
            let mut writer =
                WavWriter::new(&mut buffer, reader.spec()).expect("Cannot create writer");
            let mut target = AudioWavTarget::new(&mut writer);
            target
                .write_carrier_item(&MediaPrimitive::AudioSample(0x00b1))
                .expect("Cannot write audio sample");
            target
                .write_carrier_item(&MediaPrimitive::AudioSample(0x00b1))
                .expect("Cannot write audio sample");
            target.flush().expect("Flush failed");
        }
        let b1 = *buffer.get(0).expect("cannot read from buffer");
        let b2 = *buffer.get(1).expect("cannot read from buffer");
        let s1 = reader.samples::<i16>().next().unwrap().unwrap();
        assert_eq!(s1, ((b1 << 1) & (b2)) as i16);
    }
}
