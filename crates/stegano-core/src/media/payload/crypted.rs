use std::io::Read;

use stegano_seasmoke::decrypt_data;
use stegano_seasmoke::encrypt_data;

use super::FabA;
use super::PayloadCodecFactory;
use super::PayloadCodecFeatures;
use super::PayloadEncoder;
use super::{PayloadCodec, PayloadDecoder};
use crate::result::Result;
use crate::SteganoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabS {
    pub password: String,
}

impl FabS {
    pub fn new<I: Into<String>>(password: I) -> Self {
        FabS {
            password: password.into(),
        }
    }
}

impl PayloadCodecFactory for FabS {
    fn create_codec(&self, features: PayloadCodecFeatures) -> Result<Box<dyn PayloadCodec>> {
        let features = features
            .add_feature(PayloadCodecFeatures::ChaCrypto)
            .add_feature(PayloadCodecFeatures::LengthHeader);
        let codec = FabA.create_codec(features)?;

        Ok(Box::new(CryptedPayloadCodec::new(
            codec,
            self.password.clone(),
        )))
    }

    fn password(&self) -> Option<&str> {
        Some(&self.password)
    }
}

pub struct CryptedPayloadCodec {
    inner_encoder: Box<dyn PayloadCodec>,
    password: String,
}

impl CryptedPayloadCodec {
    pub fn new(inner_encoder: Box<dyn PayloadCodec>, password: String) -> Self {
        Self {
            inner_encoder,
            password,
        }
    }
}

/// Encryption overhead: 16 bytes (Poly1305 auth tag) + 24 bytes (nonce) + 32 bytes (salt)
#[allow(dead_code)]
const ENCRYPTION_OVERHEAD: usize = 16 + 24 + 32;

impl PayloadEncoder for CryptedPayloadCodec {
    fn version(&self) -> PayloadCodecFeatures {
        self.inner_encoder.version()
    }

    fn encode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        // let's collect all data first
        content.read_to_end(&mut data)?;

        // now we encrypt the data
        let data = encrypt_data(&self.password, &data).expect("todo");

        // now we encode the encrypted data with the inner encoder
        let mut cursor = std::io::Cursor::new(data);
        self.inner_encoder.encode(&mut cursor)
    }

    fn encoded_size(&self, content_len: usize) -> usize {
        // Encrypted size = content + encryption overhead (auth tag + nonce + salt)
        let encrypted_len = content_len + ENCRYPTION_OVERHEAD;
        // Then pass to inner encoder
        self.inner_encoder.encoded_size(encrypted_len)
    }
}

impl PayloadDecoder for CryptedPayloadCodec {
    fn decode(&self, content: &mut dyn Read) -> Result<Vec<u8>> {
        // let's collect all data first, but from the decoder that is smarter than us
        let data = self.inner_encoder.decode(content)?;
        let decrypted_data =
            decrypt_data(&self.password, &data).map_err(SteganoError::DecryptionError)?;

        Ok(decrypted_data)
    }
}

impl PayloadCodec for CryptedPayloadCodec {}

#[cfg(test)]
mod tests {
    use crate::{media::payload::HasFeature, Message};

    use super::*;

    #[test]
    fn test_encryption_codec() {
        // imagine we have a message with a text and a file
        // and we want to encode it with a codec that encrypts the data
        // and then decode it again

        let cipher = FabS::new("password42".to_owned());
        let msg = Message::from_files(&["LICENSE"]).unwrap();
        let encrypted_data = msg.to_raw_data(&cipher).unwrap();
        assert!(!encrypted_data.is_empty());

        let features = PayloadCodecFeatures::MixedFeatures(encrypted_data[0]);
        assert!(features.has_feature(PayloadCodecFeatures::TextAndDocuments));
        assert!(features.has_feature(PayloadCodecFeatures::LengthHeader));
        // thats the major part there!
        assert!(features.has_feature(PayloadCodecFeatures::ChaCrypto));

        let msg_decrypted =
            Message::from_raw_data(&mut std::io::Cursor::new(encrypted_data), &cipher).unwrap();

        assert_eq!(msg_decrypted, msg);
    }
}
