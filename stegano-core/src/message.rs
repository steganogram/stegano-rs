use crate::media::payload::PayloadCodec;
use crate::media::payload::{
    PayloadCodecFactory, PayloadCodecFeatures, PayloadDecoder, PayloadEncoder,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::default::Default;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

pub struct Message {
    pub codec_factory: PayloadCodecFactory,
    pub files: Vec<(String, Vec<u8>)>,
    pub text: Option<String>,
}

// TODO implement Result returning
impl Message {
    pub fn of(dec: &mut dyn Read, codec_factory: PayloadCodecFactory) -> Self {
        let version = dec.read_u8().expect("Failed to read version header");

        let codec: Box<dyn PayloadCodec> = codec_factory.create_codec(version);

        let content = codec
            .decode(dec)
            .expect("The surrounding method here should be fallible .. ");
        if codec.has_feature(PayloadCodecFeatures::TextOnly) {
            let text =
                String::from_utf8(content).expect("the surrounding method needs to be fallible..");

            Self {
                codec_factory,
                files: Default::default(),
                text: Some(text),
            }
        } else if codec.has_feature(PayloadCodecFeatures::TextAndDocuments) {
            Self {
                codec_factory,
                ..Self::new_of(content)
            }
        } else {
            unimplemented!("well.. maybe an own error..")
        }
        // match version {
        //     PayloadCodecFeatures::TextOnly => Self::new_of_v1(dec),
        //     PayloadCodecFeatures::TextOnlyAndTerminated => Self::new_of_v2(dec),
        //     PayloadCodecFeatures::TextAndDocuments => Self::new_of_v4(dec),
        //     PayloadCodecFeatures::Unsupported(_) => {
        //         panic!("Seems like you've got an invalid stegano file")
        //     }
        // }
    }

    pub fn new_of_files(files: &[String]) -> Self {
        let mut m = Self::new();

        files
            .iter()
            //            .map(|f| (f, File::open(f).expect("Data file was not readable.")))
            //            // TODO instead of filtering, accepting directories would be nice
            //            .filter(|(name, f)| f.metadata().unwrap().is_file())
            .for_each(|f| {
                m.add_file(f);
            });

        m
    }

    pub fn add_file(&mut self, file: &str) -> &mut Self {
        let mut fd = File::open(file).expect("File was not readable");
        let mut fb: Vec<u8> = Vec::new();

        fd.read_to_end(&mut fb).expect("Failed buffer whole file.");

        let file = Path::new(file).file_name().unwrap().to_str().unwrap();

        self.add_file_data(file, fb);
        // self.files.push((file.to_owned(), fb));

        self
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
            codec_factory: PayloadCodecFactory::default(),
            files: Vec::new(),
            text: None,
        }
    }

    fn new_of(buf: Vec<u8>) -> Message {
        let mut files = Vec::new();
        let mut buf = Cursor::new(buf);

        while let Ok(zip) = zip::read::read_zipfile_from_stream(&mut buf) {
            match zip {
                None => {}
                Some(mut file) => {
                    let mut writer = Vec::new();
                    file.read_to_end(&mut writer)
                        .expect("Failed to read data from inner message structure.");

                    files.push((file.name().to_string(), writer));
                }
            }
        }

        let mut m = Message::new();
        m.files.append(&mut files);

        m
    }
}

impl From<&mut Vec<u8>> for Message {
    fn from(buf: &mut Vec<u8>) -> Self {
        let mut c = Cursor::new(buf);
        Message::of(&mut c, PayloadCodecFactory::default())
    }
}

impl From<&Message> for Vec<u8> {
    fn from(m: &Message) -> Vec<u8> {
        let mut buf = Vec::new();

        let codec = m.codec_factory.create_codec(if m.files.is_empty() {
            PayloadCodecFeatures::TextOnly
        } else {
            PayloadCodecFeatures::TextAndDocuments
        });
        {
            let w = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(w);

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            (m.files)
                .iter()
                .map(|(name, buf)| (name, buf))
                .for_each(|(name, buf)| {
                    zip.start_file(name, options)
                        .unwrap_or_else(|_| panic!("processing file '{name}' failed."));

                    let mut r = std::io::Cursor::new(buf);
                    std::io::copy(&mut r, &mut zip).expect("Failed to copy data to the zip entry.");
                });

            zip.finish().expect("finish zip failed.");
        }

        return codec.encode(&mut Cursor::new(buf)).unwrap();
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;
    use std::io::copy;
    use zip::write::FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn should_convert_into_vec_of_bytes() {
        let files = vec!["../resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files);

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

        let b: Vec<u8> = (&m).into();
        assert_ne!(b.len(), 0, "File buffer was empty");
    }

    #[test]
    fn should_convert_from_vec_of_bytes() {
        let files = vec!["../resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files);
        let mut b: Vec<u8> = (&m).into();

        let m = Message::from(&mut b);
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
        let m = Message::new_of_files(&files);
        let mut b: Vec<u8> = (&m).into();
        let mut r = Cursor::new(&mut b);

        let m = Message::of(&mut r, PayloadCodecFactory::default());
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
        let m = Message::of(&mut r, PayloadCodecFactory::default());
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
