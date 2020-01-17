use crate::LSBCodec;
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io::{Read, Cursor};
use std::fs::File;

pub struct Message {
    pub header: u8,
    pub files: Vec<Box<(String, Vec<u8>)>>,
    pub text: Option<String>
}

impl Message {
    const VERSION: u8 = 0x04;

    pub fn of(dec: &mut dyn Read) -> Self {
        let version = dec.read_u8()
            .expect("Failed to read version header");

        match version {
            Self::VERSION => Self::new_of_v4(dec),
            _ => unimplemented!("Other than version 4 is not implemented yet")
        }
    }

    pub fn new_of_files(files: &Vec<String>) -> Self {
        let mut buf: Vec<Box<(String, Vec<u8>)>> = Vec::new();

        files
            .iter()
//            .map(|f| (f, File::open(f).expect("Data file was not readable.")))
//            // TODO instead of filtering, accepting directories would be nice
//            .filter(|(name, f)| f.metadata().unwrap().is_file())
            .for_each(|f| {
                let mut fd = File::open(f)
                    .expect("File was not readable");
                let mut fb: Vec<u8> = Vec::new();

                fd.read_to_end(&mut fb);
                buf.push(Box::new((f.to_owned(), fb)));
            });

        let mut m = Self::new(Self::VERSION);
        m.files.append(&mut buf);

        m
    }

    fn new(version: u8) -> Self {
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
        let mut i = r.take(payload_size as u64)
            .read_to_end(&mut buf)
            .expect("Message read of convent version 0x04 failed.");

        let mut files = Vec::new();

        let mut buf = Cursor::new(buf);
        let mut zip = zip::ZipArchive::new(buf)
            .expect("FileContent was invalid.");

        if zip.len() > 1 {
            unimplemented!("Only one target file is supported right now");
        }
        for i in 0..zip.len() {
            let mut writer = Vec::new();
            let mut file = zip.by_index(i).unwrap();
            file.read_to_end(&mut writer);
            files.push(Box::new((file.name().to_string(), writer)));
        }

        let mut m = Message::new(Self::VERSION);
        m.files.append(&mut files);

        m
    }
}

impl From<&mut Vec<u8>> for Message {
    fn from(buf: &mut Vec<u8>) -> Self {
        let mut c = Cursor::new(buf);
        Message::of(&mut c)
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(self.header);

        {
            let mut buf = Vec::new();

            {
                let mut w = std::io::Cursor::new(&mut buf);
                let mut zip = zip::ZipWriter::new(w);

                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored);

                (&self.files)
                    .iter()
                    .map(|b| b.as_ref())
                    .for_each(|(name, buf)| {
                        zip.start_file(name, options).
                            expect(format!("processing file '{}' failed.", name).as_str());

                        let mut r = std::io::Cursor::new(buf);
                        std::io::copy(&mut r, &mut zip)
                            .expect("Failed to copy data to the zip entry");
                    });

                zip.finish().expect("finish zip failed.");
            }

            v.write_u32::<BigEndian>(buf.len() as u32);
            v.append(&mut buf);
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
        let mut m = Message::new_of_files(&files);

        assert_eq!(m.files.len(), 1, "One file was not there, buffer was broken");
        let (name, buf) = m.files[0].as_ref();
        assert_eq!(name, &files[0], "One file was not there, buffer was broken");

        let mut b: Vec<u8> = m.into();
        assert_ne!(b.len(), 0, "File buffer was empty");
    }

    #[test]
    fn should_convert_from_vec_of_bytes() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let mut m = Message::new_of_files(&files);
        let mut b: Vec<u8> = m.into();

        let m = Message::from(&mut b);
        assert_eq!(m.files.len(), 1, "One file was not there, buffer was broken");
        let (name, buf) = m.files[0].as_ref();
        assert_eq!(name, &files[0], "One file was not there, buffer was broken");
    }

    #[test]
    fn should_instantiate_from_read_trait() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let mut m = Message::new_of_files(&files);
        let mut b: Vec<u8> = m.into();
        let mut r = Cursor::new(&mut b);

        let m = Message::of(&mut r);
        assert_eq!(m.files.len(), 1, "One file was not there, buffer was broken");
        let (name, buf) = m.files[0].as_ref();
        assert_eq!(name, &files[0], "One file was not there, buffer was broken");
    }

        // reverse way now
        let fc = FileContent::from(buf);
        assert_eq!(fc.files().len(), 1, "One file was not there, buffer was broken");
        let (name, buf) = fc.files()[0].as_ref();
        assert_eq!(name, "resources/with_text/hello_world.png", "One file was not there, buffer was broken")
    }
}