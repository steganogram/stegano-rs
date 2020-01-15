use crate::LSBCodec;
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io::{Read, Cursor};
use std::fs::File;

pub struct Message<T> {
    pub header: u8,
    pub content: Option<T>,
}

impl Message<FileContent> {
    const VERSION: u8 = 0x04;

    pub fn of(dec: &mut LSBCodec) -> Self {
        let version = dec.read_u8()
            .expect("Failed to read version header");

        match version {
            Self::VERSION => {
                Self::from(dec)
            }
            _ => {
                Message {
                    header: version,
                    content: None,
                }
            }
        }
    }

    pub fn new(fc: FileContent) -> Self {
        Message {
            header: 0x04,
            content: Some(fc)
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T> From<T> for Message<FileContent>
where T: Read
{
    fn from(mut dec: T) -> Self {
        let payload_size = dec.read_u32::<BigEndian>()
            .expect("Failed to read payload size header");

        let mut buf = Vec::new();
        let mut i = dec.take(payload_size as u64)
            .read_to_end(&mut buf)
            .expect("Message read of convent version 0x04 failed.");

        Message {
            header: Self::VERSION,
            content: Some(FileContent::from(buf))
        }
    }
}

impl Into<Vec<u8>> for Message<FileContent> {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(self.header);
        let mut c: Vec<u8> = self.content.as_ref().unwrap().into();
        v.write_u32::<BigEndian>(c.len() as u32);
        v.append(&mut c);
        v
    }
}

pub struct FileContent {
    content: Vec<Box<(String, Vec<u8>)>>
}

impl FileContent {
    pub fn new(files: &Vec<String>) -> Self {

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

        FileContent {
            content: buf
        }
    }

    pub fn files(&self) -> &Vec<Box<(String, Vec<u8>)>> {
        &self.content
    }
}

impl From<Vec<u8>> for FileContent {
    fn from(buf: Vec<u8>) -> Self {
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

        FileContent {
            content: files
        }
    }
}

impl Into<Vec<u8>> for &FileContent {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::new();

        {
            let mut w = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(w);

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            self.files()
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

        buf
    }
}

pub struct TextContent {
    content: String
}

impl TextContent {
    fn new(c: String) -> Self {
        TextContent {
            content: c
        }
    }

    fn text(&self) -> &str {
        &self.content[..]
    }
}

impl From<Vec<u8>> for TextContent {
    fn from(buf: Vec<u8>) -> Self {
        TextContent::new(String::from_utf8(buf)
            .expect("Failed to convert from buf into TextContent"))
    }
}

impl Into<Vec<u8>> for &TextContent {
    fn into(self) -> Vec<u8> {
        self.content.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod text_content_tests {
    use super::*;

    #[test]
    fn should_create_a_new_text_content() {
        let t = TextContent::new("Hello World!".to_string());
        assert_eq!(t.text(), "Hello World!", "TextContent.text() was wrong.");
    }

    #[test]
    fn should_convert_into_buffer() {
        let t = TextContent::new("Hello Wörld!".to_string());
        let v: Vec<u8> = (&t).into();
        assert_eq!(v, "Hello Wörld!".as_bytes(), "Conversion to Vec<u8> was wrong.");
    }

    #[test]
    fn should_convert_from_buffer() {
        let hw = "Hello Wörld!".as_bytes().to_vec();
        let t = TextContent::from(hw);
        assert_eq!(t.text(), "Hello Wörld!", "Conversion from Vec<u8> was wrong.");
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn should_convert_file_content_into_vec_and_back() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let fc = FileContent::new(&files);
        let mut m = Message::new(fc);

        let mut b: Vec<u8> = m.into();
        assert_ne!(b.len(), 0, "File buffer was empty");

        // reverse way
        let mut c = Cursor::new(&mut b);
        let m = Message::from(c);
        let fc = FileContent::new(&files);
        assert_eq!(fc.files().len(), 1, "One file was not there, buffer was broken");
        let (name, buf) = fc.files()[0].as_ref();
        assert_eq!(name, "resources/with_text/hello_world.png", "One file was not there, buffer was broken")
    }
}

#[cfg(test)]
mod file_content_tests {
    use super::*;

    #[test]
    fn should_convert_files_into_vec_and_back() {
        let files = vec!["resources/with_text/hello_world.png".to_string()];
        let fc = FileContent::new(&files);

        let mut buf: Vec<u8> = (&fc).into();
        assert_ne!(buf.len(), 0, "File buffer was empty");

        // reverse way now
        let fc = FileContent::from(buf);
        assert_eq!(fc.files().len(), 1, "One file was not there, buffer was broken");
        let (name, buf) = fc.files()[0].as_ref();
        assert_eq!(name, "resources/with_text/hello_world.png", "One file was not there, buffer was broken")
    }
}

