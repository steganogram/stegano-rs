use crate::media::payload::{PayloadCodec, PayloadCodecFactory, PayloadCodecFeatures};
use crate::result::Result;
use crate::SteganoError;

use byteorder::ReadBytesExt;
use image::EncodableLayout;
use std::default::Default;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub files: Vec<(String, Vec<u8>)>,
    pub text: Option<String>,
}

impl Message {
    /// Creates a new message with the content based on the message serialization format.
    pub fn from_raw_data(
        dec: &mut dyn Read,
        codec_factory: &dyn PayloadCodecFactory,
    ) -> Result<Self> {
        let version = dec.read_u8()?;
        let codec: Box<dyn PayloadCodec> =
            codec_factory.create_codec(PayloadCodecFeatures::MixedFeatures(version))?;

        let message = decode_message(&*codec, dec)?;

        Ok(message)
    }

    /// Creates a new message with the given text.
    fn from_utf8(content: Vec<u8>) -> Result<Self> {
        let text = String::from_utf8(content)?;

        Ok(Self {
            files: Default::default(),
            text: Some(text),
        })
    }

    /// Creates a new message with the given files.
    pub fn from_files<P: AsRef<Path>>(files: &[P]) -> Result<Self> {
        let mut m = Self::new();

        for f in files.iter() {
            m.add_file(f)?;
        }

        Ok(m)
    }

    pub fn add_file<P: AsRef<Path> + ?Sized>(&mut self, file: &P) -> Result<&mut Self> {
        let mut fd = File::open(file)?;
        let mut fb: Vec<u8> = Vec::new();

        fd.read_to_end(&mut fb)?;
        self.add_file_data(file, fb)?;

        Ok(self)
    }

    pub fn add_file_data<P: AsRef<Path> + ?Sized>(
        &mut self,
        file: &P,
        data: Vec<u8>,
    ) -> Result<&mut Self> {
        let file = file
            .as_ref()
            .file_name()
            .ok_or_else(|| SteganoError::InvalidFileName)?
            .to_str()
            .ok_or_else(|| SteganoError::InvalidFileName)?;

        self.files.push((file.to_owned(), data));

        Ok(self)
    }

    pub fn features(&self) -> PayloadCodecFeatures {
        if self.files.is_empty() {
            PayloadCodecFeatures::TextOnly
        } else {
            PayloadCodecFeatures::TextAndDocuments
        }
    }

    pub fn empty() -> Self {
        Self::new()
    }

    fn new() -> Self {
        Message {
            files: Vec::new(),
            text: None,
        }
    }

    fn from_documents_data(buf: Vec<u8>) -> Result<Message> {
        // todo: thinking about refactoring that, so that the this whole logic is actually ankered in the codec, or at least in the codec factory
        let mut buf = Cursor::new(buf);
        let mut m = Message::new();

        let mut zip = ZipArchive::new(&mut buf)?;
        if !zip.comment().is_empty() {
            m.text = Some(String::from_utf8_lossy(zip.comment().as_bytes()).to_string())
        }

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let mut writer = Vec::new();
            file.read_to_end(&mut writer)?;

            m.files.push((
                file.mangled_name()
                    .to_str()
                    .unwrap_or("--no-file-name--")
                    .to_string(),
                writer,
            ));
        }

        Ok(m)
    }

    pub fn to_raw_data(&self, codec_factory: &dyn PayloadCodecFactory) -> Result<Vec<u8>> {
        let codec = codec_factory.create_codec(self.features())?;
        encode_message(&*codec, self)
    }
}

// impl TryFrom<&mut Vec<u8>> for Message {
//     type Error = SteganoError;

//     fn try_from(buf: &mut Vec<u8>) -> std::result::Result<Self, Self::Error> {
//         let mut c = Cursor::new(buf);
//         Message::from_raw_data(&mut c, PayloadCodecFactory)
//     }
// }

// impl TryFrom<&Message> for Vec<u8> {
//     type Error = SteganoError;

//     fn try_from(m: &Message) -> std::result::Result<Self, Self::Error> {
//         let codec = FabA.create_codec(m.features())?;
//         encode_message(&*codec, m)
//     }
// }

pub(crate) fn encode_message(encoder: &dyn PayloadCodec, msg: &Message) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    {
        let w = Cursor::new(&mut buf);
        let mut zip = ZipWriter::new(w);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (name, buf) in (msg.files).iter().map(|(name, buf)| (name, buf)) {
            zip.start_file(name, options)?;

            let mut r = Cursor::new(buf);
            std::io::copy(&mut r, &mut zip)?;
        }

        zip.finish()?;
    }

    encoder.encode(&mut Cursor::new(buf))
}

pub(crate) fn decode_message(decoder: &dyn PayloadCodec, data: &mut dyn Read) -> Result<Message> {
    let content = decoder.decode(data)?;

    if decoder.has_feature(PayloadCodecFeatures::TextOnly) {
        Message::from_utf8(content)
    } else if decoder.has_feature(PayloadCodecFeatures::TextAndDocuments) {
        Message::from_documents_data(content)
    } else {
        Err(SteganoError::UnsupportedMessageFormat(
            decoder.version().into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use coverage_helper::test;

    use crate::media::payload::{legacy, FabA, HasFeature, TEXT_ONLY};

    use super::*;
    use std::io::{copy, BufReader};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn should_convert_into_vec_of_bytes() {
        let files = vec!["tests/images/with_text/hello_world.png".to_string()];
        let m = Message::from_files(&files).unwrap();

        assert_eq!(
            m.files.len(),
            1,
            "One file was not there, buffer was broken"
        );
        let (name, _buf) = &m.files[0];
        assert_eq!(
            name, "hello_world.png",
            "One file was not there, buffer was broken"
        );

        let b: Vec<u8> = m.to_raw_data(&FabA).unwrap();
        assert_ne!(b.len(), 0, "File buffer was empty");
    }

    #[test]
    fn should_convert_from_vec_of_bytes() {
        let files = vec!["tests/images/with_text/hello_world.png".to_string()];
        let m = Message::from_files(&files).unwrap();
        let b: Vec<u8> = m.to_raw_data(&FabA).unwrap();

        let m = Message::from_raw_data(&mut Cursor::new(b), &FabA).unwrap();
        assert_eq!(
            m.files.len(),
            1,
            "One file was not there, buffer was broken"
        );
        let (name, _buf) = &m.files[0];
        assert_eq!(
            name, "hello_world.png",
            "One file was not there, buffer was broken"
        );
    }

    #[test]
    fn should_instantiate_from_read_trait() {
        let files = &["tests/images/with_text/hello_world.png"];
        let m = Message::from_files(files).unwrap();
        let mut b: Vec<u8> = m.to_raw_data(&FabA).unwrap();
        let mut r = Cursor::new(&mut b);

        let m = Message::from_raw_data(&mut r, &FabA).unwrap();
        assert_eq!(
            m.files.len(),
            1,
            "One file was not there, buffer was broken"
        );
        let (name, _buf) = &m.files[0];
        assert_eq!(
            name, "hello_world.png",
            "One file was not there, buffer was broken"
        );
    }

    #[test]
    fn should_instantiate_from_read_trait_from_message_buffer() {
        // todo: Question: this layer here expects somehow valid message buffers,
        //       that is why it's failing right now.
        //       If this layer would be more robust, the termination checking must be implemented better.
        const BUF: [u8; 6] = [TEXT_ONLY, b'H', b'e', 0xff, 0xff, 0xcd];

        let m = Message::from_raw_data(&mut Cursor::new(BUF), &legacy::FabTextOnly).unwrap();
        assert_eq!(m.text.unwrap(), "He", "Message.text was not as expected");
        assert_eq!(m.files.len(), 0, "Message.files were not empty.");
    }

    #[test]
    fn should_create_zip_that_is_windows_compatible() -> std::io::Result<()> {
        let mut file = File::open("tests/images/with_text/hello_world.png")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let mut out_buf = Vec::new();

        let w = Cursor::new(&mut out_buf);
        let mut zip = ZipWriter::new(w);

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("hello_world.png", options)
            .unwrap_or_else(|_| panic!("processing file '{}' failed.", "hello_world.png"));

        let mut r = Cursor::new(buf);
        copy(&mut r, &mut zip).expect("Failed to copy data to the zip entry.");

        zip.finish().expect("finish zip failed.");

        Ok(())
    }

    #[test]
    fn should_error_on_unsupported_message() {
        let features = PayloadCodecFeatures::MixedFeatures(0b11000000);
        assert!(!features.has_feature(PayloadCodecFeatures::TextOnly));
        assert!(!features.has_feature(PayloadCodecFeatures::TextAndDocumentsTerminated));
        assert!(!features.has_feature(PayloadCodecFeatures::TextAndDocuments));
        assert!(!features.has_feature(PayloadCodecFeatures::LengthHeader));

        let mut buf = [features.into()];
        let mut r = Cursor::new(&mut buf);
        let mut reader = BufReader::new(&mut r);
        let message_result = Message::from_raw_data(&mut reader, &FabA);

        match message_result.err().unwrap() {
            SteganoError::UnsupportedMessageFormat(0b11000000) => {
                // expected
            }
            err => panic!("Error was not of type UnsupportedMessageFormat, but was of {err:?}"),
        }
    }
}
