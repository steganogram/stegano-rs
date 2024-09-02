use std::io::Read;

pub struct RawMessage {
    pub content: Vec<u8>,
}

impl RawMessage {
    // TODO: implement password support as in the `Message` struct
    //       By passing in the `codec_factory: &dyn PayloadCodecFactory` as parameter
    //       Then using decipher the message data but keeping here the raw data only
    pub fn of(dec: &mut dyn Read) -> Self {
        let mut m = Self::new();
        dec.read_to_end(&mut m.content)
            .expect("Failed to read raw message contents.");

        m
    }

    fn new() -> Self {
        RawMessage {
            content: Vec::new(),
        }
    }
}

#[cfg(test)]
mod raw_message_tests {
    use super::*;

    #[test]
    fn should_instantiate_from_read_trait_from_message_buffer() {
        use std::io::BufReader;
        const BUF: [u8; 6] = [0x1, b'H', b'e', 0xff, 0xff, 0xcd];

        let mut r = BufReader::new(&BUF[..]);
        let m = RawMessage::of(&mut r);
        assert_eq!(m.content, BUF, "RawMessage.content was not as expected");
    }
}
