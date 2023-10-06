#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::media::payload::{PayloadCodec, PayloadCodecFactory, PayloadCodecFeatures};
use crate::result::Result;

use crate::SteganoError;
use byteorder::ReadBytesExt;
use std::default::Default;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

pub struct Message {
    pub codec_factory: PayloadCodecFactory,
    pub files: Vec<(String, Vec<u8>)>,
    pub text: Option<String>,
}

// TODO implement Result returning
impl Message {
    pub fn of(dec: &mut dyn Read, codec_factory: PayloadCodecFactory) -> Result<Self> {
        let version = dec.read_u8()?;

        let codec: Box<dyn PayloadCodec> = codec_factory.create_codec(version)?;
        let content = codec.decode(dec)?;

        if codec.has_feature(PayloadCodecFeatures::TextOnly) {
            let text = String::from_utf8(content)?;

            Ok(Self {
                codec_factory,
                files: Default::default(),
                text: Some(text),
            })
        } else if codec.has_feature(PayloadCodecFeatures::TextAndDocuments) {
            Ok(Self {
                codec_factory,
                ..Self::new_of(content)?
            })
        } else {
            Err(SteganoError::UnsupportedMessageFormat(
                codec.version().into(),
            ))
        }
    }

    pub fn new_of_files(files: &[String]) -> Result<Self> {
        let mut m = Self::new();

        for f in files.iter() {
            //            .map(|f| (f, File::open(f).expect("Data file was not readable.")))
            //            // TODO instead of filtering, accepting directories would be nice
            //            .filter(|(name, f)| f.metadata().unwrap().is_file())
            m.add_file(f)?;
        }

        Ok(m)
    }

    pub fn add_file(&mut self, file: &str) -> Result<&mut Self> {
        let mut fd = File::open(file)?;
        let mut fb: Vec<u8> = Vec::new();

        fd.read_to_end(&mut fb)?;

        let file = Path::new(file)
            .file_name()
            .ok_or_else(|| SteganoError::InvalidFileName)?
            .to_str()
            .ok_or_else(|| SteganoError::InvalidFileName)?;

        self.add_file_data(file, fb);
        // self.files.push((file.to_owned(), fb));

        Ok(self)
    }

    pub fn add_file_data(&mut self, file: &str, data: Vec<u8>) -> &mut Self {
        self.files.push((file.to_owned(), data));

        self
    }

    pub fn empty() -> Self {
        Self::new()
    }

    fn new() -> Self {
        Message {
            // todo: injection all the way up!!
            codec_factory: PayloadCodecFactory,
            files: Vec::new(),
            text: None,
        }
    }

    fn new_of(buf: Vec<u8>) -> Result<Message> {
        let mut files = Vec::new();
        let mut buf = Cursor::new(buf);

        while let Ok(zip) = zip::read::read_zipfile_from_stream(&mut buf) {
            match zip {
                None => {}
                Some(mut file) => {
                    let mut writer = Vec::new();
                    file.read_to_end(&mut writer)?;

                    files.push((file.name().to_string(), writer));
                }
            }
        }

        let mut m = Message::new();
        m.files.append(&mut files);

        Ok(m)
    }
}

impl TryFrom<&mut Vec<u8>> for Message {
    type Error = SteganoError;

    fn try_from(buf: &mut Vec<u8>) -> std::result::Result<Self, Self::Error> {
        let mut c = Cursor::new(buf);
        Message::of(&mut c, PayloadCodecFactory)
    }
}

impl TryFrom<&Message> for Vec<u8> {
    type Error = SteganoError;

    fn try_from(m: &Message) -> std::result::Result<Self, Self::Error> {
        let mut buf = Vec::new();

        let codec = if m.files.is_empty() {
            m.codec_factory.create_codec_for_text()
        } else {
            m.codec_factory.create_codec_for_documents()
        };

        {
            let w = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(w);

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for (name, buf) in (m.files).iter().map(|(name, buf)| (name, buf)) {
                zip.start_file(name, options)?;

                let mut r = Cursor::new(buf);
                std::io::copy(&mut r, &mut zip)?;
            }

            zip.finish()?;
        }

        Ok(codec.encode(&mut Cursor::new(buf))?)
    }
}

#[cfg(test)]
mod message_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use std::io::copy;
    use zip::write::FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn should_convert_into_vec_of_bytes() {
        let files = vec!["../resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files).unwrap();

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

        let b: Vec<u8> = (&m).try_into().unwrap();
        assert_ne!(b.len(), 0, "File buffer was empty");
    }

    #[test]
    fn should_convert_from_vec_of_bytes() {
        let files = vec!["../resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files).unwrap();
        let mut b: Vec<u8> = (&m).try_into().unwrap();

        let m = Message::try_from(&mut b).unwrap();
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
        let files = vec!["../resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files).unwrap();
        let mut b: Vec<u8> = (&m).try_into().unwrap();
        let mut r = Cursor::new(&mut b);

        let m = Message::of(&mut r, PayloadCodecFactory).unwrap();
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
        use std::io::BufReader;

        // todo: Question: this layer here expects somehow valid message buffers,
        //       that is why it's failing right now.
        //       If this layer would be more robust, the termination checking must be implemented better.
        const BUF: [u8; 6] = [0x1, b'H', b'e', 0xff, 0xff, 0xcd];

        let mut r = BufReader::new(&BUF[..]);
        let m = Message::of(&mut r, PayloadCodecFactory).unwrap();
        assert_eq!(m.text.unwrap(), "He", "Message.text was not as expected");
        assert_eq!(m.files.len(), 0, "Message.files were not empty.");
    }

    #[test]
    fn should_create_zip_that_is_windows_compatible() -> std::io::Result<()> {
        let mut file = File::open("../resources/with_text/hello_world.png")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let mut out_buf = Vec::new();

        let w = Cursor::new(&mut out_buf);
        let mut zip = ZipWriter::new(w);

        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("hello_world.png", options)
            .unwrap_or_else(|_| panic!("processing file '{}' failed.", "hello_world.png"));

        let mut r = Cursor::new(buf);
        copy(&mut r, &mut zip).expect("Failed to copy data to the zip entry.");

        zip.finish().expect("finish zip failed.");

        Ok(())
    }
}
