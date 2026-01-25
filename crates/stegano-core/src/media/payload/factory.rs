use std::fmt::Debug;

use super::*;
use crate::error::SteganoError;
use crate::result::Result;

pub trait PayloadCodecFactory {
    fn create_codec(&self, features: PayloadCodecFeatures) -> Result<Box<dyn PayloadCodec>>;
    /// Returns the password if one is set (for deriving F5 seed)
    fn password(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FabA;
impl PayloadCodecFactory for FabA {
    fn create_codec(&self, features: PayloadCodecFeatures) -> Result<Box<dyn PayloadCodec>>
    where
        Self: Sized,
    {
        let version: u8 = features.into();
        match version {
            TEXT_ONLY | TEXT_AND_DOCUMENTS_TERMINATED => legacy::FabTextOnly.create_codec(features),
            TEXT_AND_DOCUMENTS => Ok(Box::new(PayloadFlexCodec::new(
                PayloadEncoderWithLengthHeader::new(
                    PayloadCodecFeatures::TextAndDocuments
                        .add_feature(PayloadCodecFeatures::LengthHeader),
                ),
                PayloadDecoderWithLengthHeader,
            ))),
            version if version.has_feature(PayloadCodecFeatures::LengthHeader) => {
                let codec: Box<dyn PayloadCodec> = Box::new(PayloadFlexCodec::new(
                    // here we explicitly keep the specific version as is
                    PayloadEncoderWithLengthHeader::new(features),
                    PayloadDecoderWithLengthHeader,
                ));
                Ok(codec)
            }
            version => Err(SteganoError::UnsupportedMessageFormat(version)),
        }
    }
}
