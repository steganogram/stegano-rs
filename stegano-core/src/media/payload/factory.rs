use super::*;
use crate::error::SteganoError;
use crate::result::Result;

#[derive(Default)]
pub struct PayloadCodecFactory;
impl PayloadCodecFactory {
    /// returns the current text and documents codec, with length-header feature
    pub fn create_codec_for_documents(&self) -> Box<dyn PayloadCodec> {
        Box::new(PayloadFlexCodec::new(
            Self::create_text_and_documents_encoder(),
            PayloadDecoderWithLengthHeader,
        ))
    }

    /// returns the current text only codec, with length-header feature
    pub fn create_codec_for_text(&self) -> Box<dyn PayloadCodec> {
        Box::new(PayloadFlexCodec::new(
            Self::create_text_encoder(),
            PayloadDecoderWithLengthHeader,
        ))
    }

    pub fn create_codec<T: Into<u8>>(&self, version: T) -> Result<Box<dyn PayloadCodec>> {
        let version: u8 = version.into();
        match version {
            TEXT_ONLY => Ok(Box::new(PayloadFlexCodec::new(
                Self::create_text_encoder(),
                PayloadDecoderLegacyVersion1,
            ))),
            TEXT_AND_DOCUMENTS_TERMINATED => Ok(Box::new(PayloadFlexCodec::new(
                Self::create_text_and_documents_encoder(),
                PayloadDecoderLegacyVersion2::default(),
            ))),
            TEXT_AND_DOCUMENTS => Ok(self.create_codec_for_documents()),
            version if version.has_feature(PayloadCodecFeatures::LengthHeader) => {
                if version.has_feature(PayloadCodecFeatures::TextOnly)
                    || version.has_feature(PayloadCodecFeatures::TextAndDocuments)
                {
                    Ok(Box::new(PayloadFlexCodec::new(
                        // here we explicitly keep the specific version as is
                        PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
                            version,
                        )),
                        PayloadDecoderWithLengthHeader,
                    )))
                } else {
                    // imagine a future content format, then this else case will fire back!
                    Err(SteganoError::UnsupportedMessageFormat(version))
                }
            }
            version => Err(SteganoError::UnsupportedMessageFormat(version)),
        }
    }

    fn create_text_and_documents_encoder() -> PayloadEncoderWithLengthHeader {
        PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
            u8::from(PayloadCodecFeatures::TextAndDocuments)
                | u8::from(PayloadCodecFeatures::LengthHeader),
        ))
    }

    fn create_text_encoder() -> PayloadEncoderWithLengthHeader {
        PayloadEncoderWithLengthHeader::new(PayloadCodecFeatures::MixedFeatures(
            u8::from(PayloadCodecFeatures::TextOnly) | u8::from(PayloadCodecFeatures::LengthHeader),
        ))
    }
}
