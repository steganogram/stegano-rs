use crate::result::Result;

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct FabTextOnly;

impl PayloadCodecFactory for FabTextOnly {
    fn create_codec(&self, features: PayloadCodecFeatures) -> Result<Box<dyn PayloadCodec>> {
        let version: u8 = features.into();
        match version {
            TEXT_ONLY => Ok(Box::new(PayloadFlexCodec::new(
                // this is not the legacy encoder, FWIW we should not have the legacy encoders anylonger, we migrate to newer formats
                PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
                    u8::from(PayloadCodecFeatures::TextOnly)
                        | u8::from(PayloadCodecFeatures::LengthHeader),
                )),
                v1::PayloadDecoderLegacyVersion1,
            ))),
            TEXT_AND_DOCUMENTS_TERMINATED => Ok(Box::new(PayloadFlexCodec::new(
                PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
                    u8::from(PayloadCodecFeatures::TextAndDocuments)
                        | u8::from(PayloadCodecFeatures::LengthHeader),
                )),
                v2::PayloadDecoderWithTerminator,
            ))),
            _ => Err(crate::SteganoError::UnsupportedMessageFormat(version)),
        }
    }
}

pub mod v1 {
    use std::io::Read;

    use byteorder::{ReadBytesExt, WriteBytesExt};

    use super::*;

    #[derive(Debug, Default)]
    pub struct PayloadDecoderLegacyVersion1;
    impl PayloadDecoder for PayloadDecoderLegacyVersion1 {
        fn decode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
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
}

pub mod v2 {
    use std::io::Read;

    use super::*;

    #[derive(Debug, Default)]
    pub struct PayloadDecoderWithTerminator;
    impl PayloadDecoder for PayloadDecoderWithTerminator {
        fn decode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
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
}
