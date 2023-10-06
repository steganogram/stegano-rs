use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::min;
use std::io::{Read, Write};

const TEXT_ONLY: u8 = 1 << 0;
const TEXT_AND_DOCUMENTS_TERMINATED: u8 = 1 << 1;
const TEXT_AND_DOCUMENTS: u8 = 1 << 2;
const LENGTH_HEADER: u8 = 1 << 3;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum PayloadCodecFeatures {
    TextOnly = TEXT_ONLY,
    TextAndDocumentsTerminated = TEXT_AND_DOCUMENTS_TERMINATED,
    TextAndDocuments = TEXT_AND_DOCUMENTS,
    LengthHeader = LENGTH_HEADER,

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

pub trait PayloadCodec: PayloadEncoder + PayloadDecoder {
    fn has_feature(&self, feature: PayloadCodecFeatures) -> bool {
        let v: u8 = self.version().into();
        let f: u8 = feature.into();

        (v & f) != 0
    }
}

#[derive(Default)]
pub struct PayloadCodecFactory;
impl PayloadCodecFactory {
    pub fn create_codec<T: Into<u8>>(&self, version: T) -> Box<dyn PayloadCodec> {
        let version: u8 = version.into();
        match version {
            TEXT_ONLY => Box::new(PayloadFlexCodec::new(
                PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
                    u8::from(PayloadCodecFeatures::TextOnly)
                        | u8::from(PayloadCodecFeatures::LengthHeader),
                )),
                PayloadDecoderLegacyVersion1::default(),
            )),
            TEXT_AND_DOCUMENTS_TERMINATED => Box::new(PayloadFlexCodec::new(
                Self::create_text_and_documents_encoder(),
                PayloadDecoderLegacyVersion2::default(),
            )),
            TEXT_AND_DOCUMENTS => Box::new(PayloadFlexCodec::new(
                Self::create_text_and_documents_encoder(),
                PayloadDecoderWithLengthHeader::default(),
            )),
            version if version & u8::from(PayloadCodecFeatures::LengthHeader) != 0 => {
                if version & u8::from(PayloadCodecFeatures::TextOnly) != 0 {
                    Box::new(PayloadFlexCodec::new(
                        // here we explicitly keep the specific version as is
                        PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
                            version,
                        )),
                        PayloadDecoderWithLengthHeader::default(),
                    ))
                } else if version & u8::from(PayloadCodecFeatures::TextAndDocuments) != 0 {
                    Box::new(PayloadFlexCodec::new(
                        // here we explicitly keep the specific version as is
                        PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
                            version,
                        )),
                        PayloadDecoderWithLengthHeader::default(),
                    ))
                } else {
                    unimplemented!("totally unsupported version: {version}")
                }
            }
            version => {
                unimplemented!("yet unsupported version: {version}")
            }
        }
    }

    fn create_text_and_documents_encoder() -> PayloadEncoderWithLengthHeader {
        PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
            u8::from(PayloadCodecFeatures::TextAndDocuments)
                | u8::from(PayloadCodecFeatures::LengthHeader),
        ))
    }
}

#[derive(Default)]
pub struct PayloadDecoderLegacyVersion1;

impl PayloadDecoder for PayloadDecoderLegacyVersion1 {
    fn decode(&self, mut content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        loop {
            if let Ok(b) = content.read_u8() {
                // very naive, only one terminator
                if b == 0xff {
                    break;
                }
                buffer.write_u8(b)?;
            } else {
                break;
            }
        }

        Ok(buffer)
    }
}

/// This alias exists only to illustrate the specific use case at the client code
pub type PayloadDecoderLegacyVersion2 = PayloadDecoderLegacyVersion1;

#[derive(Default)]
pub struct PayloadDecoderWithLengthHeader;
impl PayloadDecoder for PayloadDecoderWithLengthHeader {
    fn decode(&self, mut content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
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

    fn encode(&self, mut content: &mut dyn Read) -> std::io::Result<Vec<u8>> {
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
