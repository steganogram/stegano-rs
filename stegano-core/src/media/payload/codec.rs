use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Read;

pub(super) const TEXT_ONLY: u8 = 1 << 0;
pub(super) const TEXT_AND_DOCUMENTS_TERMINATED: u8 = 1 << 1;
pub(super) const TEXT_AND_DOCUMENTS: u8 = 1 << 2;
pub(super) const LENGTH_HEADER: u8 = 1 << 3;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PayloadCodecFeatures {
    TextOnly,
    TextAndDocumentsTerminated,
    TextAndDocuments,
    LengthHeader,
    MixedFeatures(u8),
}

impl From<PayloadCodecFeatures> for u8 {
    fn from(value: PayloadCodecFeatures) -> Self {
        match value {
            PayloadCodecFeatures::TextOnly => TEXT_ONLY,
            PayloadCodecFeatures::TextAndDocumentsTerminated => TEXT_AND_DOCUMENTS_TERMINATED,
            PayloadCodecFeatures::TextAndDocuments => TEXT_AND_DOCUMENTS,
            PayloadCodecFeatures::LengthHeader => LENGTH_HEADER,
            PayloadCodecFeatures::MixedFeatures(other) => other,
        }
    }
}

pub trait PayloadEncoder {
    fn version(&self) -> PayloadCodecFeatures;

    fn encode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>>;
}

pub trait PayloadDecoder {
    fn decode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>>;
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

impl<E> HasFeature for E
where
    E: PayloadEncoder,
{
    fn has_feature(&self, feature: PayloadCodecFeatures) -> bool {
        let v: u8 = self.version().into();
        v.has_feature(feature)
    }
}

#[derive(Default)]
pub struct PayloadDecoderLegacyVersion1;
impl PayloadDecoder for PayloadDecoderLegacyVersion1 {
    fn decode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        while let Ok(b) = content.read_u8() {
            // very naive, only one terminator
            if b == 0xff {
                break;
            }
            buffer.write_u8(b)?;
        }

        Ok(buffer)
    }
}

#[derive(Default)]
pub struct PayloadDecoderLegacyVersion2;
impl PayloadDecoder for PayloadDecoderLegacyVersion2 {
    fn decode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        content.read_to_end(&mut buffer)?;

        let zeros = buffer.iter().rev().take_while(|x| x == &&0x0).count();
        buffer.truncate(buffer.len() - zeros);
        if buffer[buffer.len() - 1] == 0xff {
            buffer.truncate(buffer.len() - 1);
        }
        if buffer[buffer.len() - 1] == 0xff {
            buffer.truncate(buffer.len() - 1);
        }

        Ok(buffer)
    }
}

#[derive(Default)]
pub struct PayloadDecoderWithLengthHeader;
impl PayloadDecoder for PayloadDecoderWithLengthHeader {
    fn decode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
        let len = content.read_u32::<BigEndian>()? as usize;
        let mut buffer = Vec::new();
        content.read_to_end(&mut buffer)?;
        if len > buffer.len() {
            todo!("read len value cannot be bigger than the actual buffer");
        }
        buffer.truncate(len);

        Ok(buffer)
    }
}

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

    fn encode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
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

    fn encode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
        self.encoder.encode(content)
    }
}

impl PayloadDecoder for PayloadFlexCodec {
    fn decode(&self, content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
        self.decoder.decode(content)
    }
}

impl PayloadCodec for PayloadFlexCodec {}
