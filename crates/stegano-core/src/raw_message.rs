use std::io::Read;

use byteorder::ReadBytesExt;

use crate::media::payload::{PayloadCodec, PayloadCodecFactory, PayloadCodecFeatures};
use crate::result::Result;

#[derive(Debug, Default)]
pub struct RawMessage {
    pub content: Vec<u8>,
}

impl RawMessage {
    pub fn from_raw_data(
        dec: &mut dyn Read,
        codec_factory: &dyn PayloadCodecFactory,
    ) -> Result<Self> {
        let version = dec.read_u8()?;
        let codec: Box<dyn PayloadCodec> =
            codec_factory.create_codec(PayloadCodecFeatures::MixedFeatures(version))?;

        Ok(Self {
            content: codec.decode(dec)?,
        })
    }
}

#[cfg(test)]
mod raw_message_tests {
    use std::io::BufReader;

    use super::*;
    use crate::media::payload::FabA;

    #[test]
    fn should_instantiate_from_read_trait_from_message_buffer() {
        const BUF: [u8; 6] = [0x1, b'H', b'e', 0xff, 0xff, 0xcd];
        // ------------------------^^^^^^^^^^--------------------
        //                         | this is the message content

        let mut r = BufReader::new(&BUF[..]);
        let m = RawMessage::from_raw_data(&mut r, &FabA)
            .expect("Failed to create RawMessage from buffer");

        assert_eq!(
            m.content,
            &[b'H', b'e'],
            "RawMessage.content should contain `He` ascii bytes"
        );
    }
}
