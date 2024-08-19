use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Read;

use crate::result::Result;

pub(crate) const TEXT_ONLY: u8 = 1 << 0;
pub(crate) const TEXT_AND_DOCUMENTS_TERMINATED: u8 = 1 << 1;
pub(crate) const TEXT_AND_DOCUMENTS: u8 = 1 << 2;
pub(crate) const LENGTH_HEADER: u8 = 1 << 3;
pub(crate) const AES_CRYPTO: u8 = 1 << 4;
pub(crate) const CHA_CRYPTO: u8 = 1 << 5;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PayloadCodecFeatures {
    TextOnly,
    TextAndDocumentsTerminated,
    TextAndDocuments,
    LengthHeader,
    AesCrypto,
    ChaCrypto,
    MixedFeatures(u8),
}

impl PayloadCodecFeatures {
    /// Adds other features, but also consumes self.
    pub fn add_feature(self, other: PayloadCodecFeatures) -> Self {
        let v: u8 = self.into();
        let f: u8 = other.into();

        PayloadCodecFeatures::MixedFeatures(v | f)
    }
}

impl From<PayloadCodecFeatures> for u8 {
    fn from(value: PayloadCodecFeatures) -> Self {
        match value {
            PayloadCodecFeatures::TextOnly => TEXT_ONLY,
            PayloadCodecFeatures::TextAndDocumentsTerminated => TEXT_AND_DOCUMENTS_TERMINATED,
            PayloadCodecFeatures::TextAndDocuments => TEXT_AND_DOCUMENTS,
            PayloadCodecFeatures::LengthHeader => LENGTH_HEADER,
            PayloadCodecFeatures::AesCrypto => AES_CRYPTO,
            PayloadCodecFeatures::ChaCrypto => CHA_CRYPTO,
            PayloadCodecFeatures::MixedFeatures(other) => other,
        }
    }
}

pub trait PayloadEncoder {
    fn version(&self) -> PayloadCodecFeatures;

    fn encode(&self, content: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait PayloadDecoder {
    fn decode(&self, content: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait HasFeature {
    fn has_feature(&self, feature: PayloadCodecFeatures) -> bool;
}

pub trait PayloadCodec: PayloadEncoder + PayloadDecoder + HasFeature {}

impl HasFeature for u8 {
    fn has_feature(&self, feature: PayloadCodecFeatures) -> bool {
        let v: u8 = *self;
        let f: u8 = feature.into();

        (v & f) != 0
    }
}

impl HasFeature for PayloadCodecFeatures {
    fn has_feature(&self, feature: PayloadCodecFeatures) -> bool {
        let v: u8 = (*self).into();
        v.has_feature(feature)
    }
}

impl<E> HasFeature for E
where
    E: PayloadEncoder,
{
    fn has_feature(&self, feature: PayloadCodecFeatures) -> bool {
        let v: u8 = self.version().into();
        v.has_feature(feature)
    }
}

#[derive(Debug, Default)]
pub struct PayloadDecoderWithLengthHeader;
impl PayloadDecoder for PayloadDecoderWithLengthHeader {
    fn decode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
        let len = content.read_u32::<BigEndian>()? as usize;
        let mut buffer = Vec::new();
        content.read_to_end(&mut buffer)?;
        if len > buffer.len() {
            panic!(
                "read len value cannot be bigger `{len}` than the actual buffer `{}`",
                buffer.len()
            );
        }
        buffer.truncate(len);

        Ok(buffer)
    }
}

#[derive(Debug)]
pub struct PayloadEncoderWithLengthHeader {
    version: PayloadCodecFeatures,
}

impl PayloadEncoderWithLengthHeader {
    pub fn new(version: PayloadCodecFeatures) -> Self {
        Self { version }
    }
}

impl PayloadEncoder for PayloadEncoderWithLengthHeader {
    fn version(&self) -> PayloadCodecFeatures {
        self.version
    }

    fn encode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
        let mut src = Vec::new();
        content.read_to_end(&mut src)?;

        let mut buffer = Vec::with_capacity(src.len() + 6);
        buffer.write_u8(self.version().into())?;
        buffer.write_u32::<BigEndian>(src.len() as u32)?;
        buffer.extend_from_slice(&src[..]);
        buffer.write_u8(0xff)?;

        Ok(buffer)
    }
}

pub struct PayloadFlexCodec {
    encoder: Box<dyn PayloadEncoder>,
    decoder: Box<dyn PayloadDecoder>,
}

impl PayloadFlexCodec {
    pub fn new<'a, E: PayloadEncoder + 'static, D: PayloadDecoder + 'static>(
        encoder: E,
        decoder: D,
    ) -> Self
    where
        Self: 'a,
    {
        Self {
            encoder: Box::new(encoder),
            decoder: Box::new(decoder),
        }
    }
}

impl PayloadEncoder for PayloadFlexCodec {
    fn version(&self) -> PayloadCodecFeatures {
        self.encoder.version()
    }

    fn encode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
        self.encoder.encode(content)
    }
}

impl PayloadDecoder for PayloadFlexCodec {
    fn decode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
        self.decoder.decode(content)
    }
}

impl PayloadCodec for PayloadFlexCodec {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_adding() {
        let f = PayloadCodecFeatures::TextAndDocuments
            .add_feature(PayloadCodecFeatures::LengthHeader)
            .add_feature(PayloadCodecFeatures::ChaCrypto);

        assert!(f.has_feature(PayloadCodecFeatures::TextAndDocuments));
        assert!(f.has_feature(PayloadCodecFeatures::LengthHeader));
        assert!(f.has_feature(PayloadCodecFeatures::ChaCrypto));
    }
}
