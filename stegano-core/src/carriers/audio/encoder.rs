use crate::universal_encoder::WriteCarrierItem;
use crate::CarrierItem;
use hound::WavWriter;
use std::io::{Result, Seek, Write};

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
    fn write_carrier_item(&mut self, carrier_item: &CarrierItem) -> Result<usize> {
        match carrier_item {
            CarrierItem::ImageColorChannel(_) => {
                Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
            }
            CarrierItem::AudioSample(b) => match self.target.write_sample(*b) {
                Ok(_) => Ok(2),
                // TODO map the error somehow to std::io::ErrorKind
                Err(_) => Err(std::io::Error::from(std::io::ErrorKind::Other)),
            },
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.target
            .flush()
            .unwrap_or_else(|_| panic!("Flushing the WavWriter failed."));
        Ok(())
    }
}
