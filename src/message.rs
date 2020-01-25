use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io::{Read, Cursor};
use std::fs::File;

#[derive(PartialEq, Debug)]
pub enum ContentVersion {
    V1,
    V2,
    V4,
    Unsupported(u8),
}

impl ContentVersion {
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::V1 => 0x01,
            Self::V2 => 0x02,
            Self::V4 => 0x04,
            Self::Unsupported(v) => *v,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => Self::V1,
            0x02 => Self::V2,
            0x04 => Self::V4,
            b => Self::Unsupported(b),
        }
    }
}

pub struct Message {
    pub header: ContentVersion,
    pub files: Vec<(String, Vec<u8>)>,
    pub text: Option<String>
}

impl Message {
    pub fn of(dec: &mut dyn Read) -> Self {
        let version = dec.read_u8()
            .expect("Failed to read version header");

        let version = ContentVersion::from_u8(version);

        match version {
            ContentVersion::V1 => Self::new_of_v1(dec),
            ContentVersion::V2 => Self::new_of_v2(dec),
            ContentVersion::V4 => Self::new_of_v4(dec),
            ContentVersion::Unsupported(v) => unimplemented!("Version {} is not implemented.", v)
        }
    }

    pub fn new_of_files(files: &[String]) -> Self {
        let mut m = Self::new(ContentVersion::V4);

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

    pub fn add_file(&mut self, file: &str) -> &mut Self{
        let mut fd = File::open(file)
            .expect("File was not readable");
        let mut fb: Vec<u8> = Vec::new();

        fd.read_to_end(&mut fb)
            .expect("Failed buffer whole file.");

        self.files.push((file.to_owned(), fb));

        self
    }

    pub fn empty() -> Self {
        Self::new(ContentVersion::V4)
    }

    fn new(version: ContentVersion) -> Self {
        Message {
            header: version,
            files: Vec::new(),
            text: None,
        }
    }

    fn new_of_v4(r: &mut dyn Read) -> Self {
        let payload_size = r.read_u32::<BigEndian>()
            .expect("Failed to read payload size header");

        let mut buf = Vec::new();
        r.take(payload_size as u64)
            .read_to_end(&mut buf)
            .expect("Message read of content version 0x04 failed.");

        Self::new_of(buf)
    }

    fn new_of_v2(r: &mut dyn Read) -> Self {
        const EOF: u8 = 0xff;
        let mut buf = Vec::new();

        while let Ok(b) = r.read_u8() {
            if b == EOF {
                if let Ok(bn) = r.read_u8() {
                    if bn == EOF {
                        break;
                    } else {
                        buf.push(b);
                        buf.push(bn);
                        continue;
                    }
                }
            }
            buf.push(b);
        }

        Self::new_of(buf)
    }

    fn new_of(buf: Vec<u8>) -> Message {
        let mut files = Vec::new();
        let buf = Cursor::new(buf);
        let mut zip = zip::ZipArchive::new(buf)
            .expect("FileContent was invalid.");
        if zip.len() > 1 {
            unimplemented!("Only one target file is supported right now.");
        }
        for i in 0..zip.len() {
            let mut writer = Vec::new();
            let mut file = zip.by_index(i).unwrap();
            file.read_to_end(&mut writer)
                .expect("Failed to read data from inner message structure.");

            files.push((file.name().to_string(), writer));
        }
        let mut m = Message::new(ContentVersion::V4);
        m.files.append(&mut files);
        m
    }

    fn new_of_v1(r: &mut dyn Read) -> Self {
        const EOF: u8 = 0xff;
        let mut buf = Vec::new();

        while let Ok(b) = r.read_u8() {
            if b == EOF {
                break;
            }
            buf.push(b);
        }

        // TODO shall we upgrade all v1 to v4, to get rid of the legacy?
        let mut m = Message::new(ContentVersion::V1);
        m.text = Some(String::from_utf8(buf)
                .expect("Message failed to decode from image")
            );

        m
    }
}

impl From<&mut Vec<u8>> for Message {
    fn from(buf: &mut Vec<u8>) -> Self {
        let mut c = Cursor::new(buf);
        Message::of(&mut c)
    }
}

impl Into<Vec<u8>> for &Message {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(self.header.to_u8());

        {
            let mut buf = Vec::new();

            {
                let w = std::io::Cursor::new(&mut buf);
                let mut zip = zip::ZipWriter::new(w);

                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);

                (&self.files)
                    .iter()
                    .map(|b| b)
                    .for_each(|(name, buf)| {
                        zip.start_file(name, options).
                            unwrap_or_else(|_| panic!("processing file '{}' failed.", name));

                        let mut r = std::io::Cursor::new(buf);
                        std::io::copy(&mut r, &mut zip)
                            .expect("Failed to copy data to the zip entry");
                    });

                zip.finish().expect("finish zip failed.");
            }

            if self.header == ContentVersion::V4 {
                v.write_u32::<BigEndian>(buf.len() as u32)
                    .expect("Failed to write the inner message size.");
            }

            v.append(&mut buf);

            if self.header == ContentVersion::V2 {
                v.write_u16::<BigEndian>(0xffff)
                    .expect("Failed to write content format 2 termination.");
            }
        }

        v
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn should_convert_into_vec_of_bytes() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files);

        assert_eq!(m.files.len(), 1, "One file was not there, buffer was broken");
        let (name, _buf) = &m.files[0];
        assert_eq!(name, &files[0], "One file was not there, buffer was broken");

        let b: Vec<u8> = (&m).into();
        assert_ne!(b.len(), 0, "File buffer was empty");
    }

    #[test]
    fn should_convert_from_vec_of_bytes() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files);
        let mut b: Vec<u8> = (&m).into();

        let m = Message::from(&mut b);
        assert_eq!(m.files.len(), 1, "One file was not there, buffer was broken");
        let (name, _buf) = &m.files[0];
        assert_eq!(name, &files[0], "One file was not there, buffer was broken");
    }

    #[test]
    fn should_instantiate_from_read_trait() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let m = Message::new_of_files(&files);
        let mut b: Vec<u8> = (&m).into();
        let mut r = Cursor::new(&mut b);

        let m = Message::of(&mut r);
        assert_eq!(m.files.len(), 1, "One file was not there, buffer was broken");
        let (name, _buf) = &m.files[0];
        assert_eq!(name, &files[0], "One file was not there, buffer was broken");
    }

    #[test]
    fn should_instantiate_from_read_trait_from_message_buffer() {
        use std::io::BufReader;
        const BUF: [u8; 6] = [0x1, b'H', b'e', 0xff, 0xff, 0xcd];

        let mut r = BufReader::new(&BUF[..]);
        let m = Message::of(&mut r);
        assert_eq!(m.text.unwrap(), "He", "Message.text was not as expected");
        assert_eq!(m.files.len(), 0, "Message.files were not empty.");
    }
}