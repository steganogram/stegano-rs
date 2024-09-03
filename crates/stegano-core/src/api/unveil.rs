use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    media::{
        audio::wav_iter::AudioWavIter,
        image::LsbCodec,
        payload::{FabA, FabS, PayloadCodecFactory},
        types::Media,
    },
    universal_decoder::{Decoder, OneBitUnveil},
    CodecOptions, Message, SteganoError,
};

use super::Password;

pub fn prepare() -> UnveilApi {
    UnveilApi::default()
}

#[derive(Default, Debug)]
pub struct UnveilApi {
    secret_media: Option<PathBuf>,
    output_folder: Option<PathBuf>,
    password: Password,
    options: CodecOptions,
}

impl UnveilApi {
    /// Use the given codec options
    pub fn with_options(mut self, options: CodecOptions) -> Self {
        self.options = options;
        self
    }

    /// This is the secret image that contains the data to be unveiled
    pub fn from_secret_file(mut self, secret_image: impl AsRef<Path>) -> Self {
        self.secret_media = Some(secret_image.as_ref().to_path_buf());
        self
    }

    /// This is the secret audio that contains the data to be unveiled
    pub fn with_secret_audio(mut self, secret_audio: impl AsRef<Path>) -> Self {
        self.secret_media = Some(secret_audio.as_ref().to_path_buf());
        self
    }

    /// This is the folder where the data will be saved to
    pub fn into_output_folder(mut self, output_folder: impl AsRef<Path>) -> Self {
        self.output_folder = Some(output_folder.as_ref().to_path_buf());
        self
    }

    /// Set the password used for encrypting all data
    /// If `None` is passed, no password will be used, leads to no de-/encryption used
    pub fn using_password<P: Into<Password>>(mut self, password: P) -> Self {
        self.password = password.into();
        self
    }

    /// Execute the unveil process and blocks until it is finished
    pub fn execute(self) -> Result<(), SteganoError> {
        let Some(secret_media) = self.secret_media else {
            return Err(SteganoError::CarrierNotSet);
        };
        let Some(output_folder) = self.output_folder else {
            return Err(SteganoError::TargetNotSet);
        };

        let media = Media::from_file(&secret_media)?;
        let fab: Box<dyn PayloadCodecFactory> = if let Some(password) = self.password.as_ref() {
            Box::new(FabS::new(password))
        } else {
            Box::new(FabA)
        };

        let msg = match media {
            Media::Image(image) => {
                let mut decoder = LsbCodec::decoder(&image, &self.options);
                Message::from_raw_data(&mut decoder, &*fab)?
            }
            Media::Audio(audio) => {
                let mut decoder =
                    Decoder::new(AudioWavIter::new(audio.1.into_iter()), OneBitUnveil);
                Message::from_raw_data(&mut decoder, &*fab)?
            }
        };

        let mut files = msg.files;
        if let Some(text) = msg.text {
            files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
        }

        if files.is_empty() {
            return Err(SteganoError::NoSecretData);
        }

        for (file_name, buf) in files.iter().map(|(file_name, buf)| {
            let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

            (file, buf)
        }) {
            let target_file = output_folder.join(file_name);
            let mut target_file =
                File::create(target_file).map_err(|source| SteganoError::WriteError { source })?;

            target_file
                .write_all(buf.as_slice())
                .map_err(|source| SteganoError::WriteError { source })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::read_to_string;

    use tempfile::tempdir;

    #[test]
    fn illustrate_api_usage() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        crate::api::unveil::prepare()
            .from_secret_file("tests/images/encrypted/hello_world.png")
            .using_password("Secret42")
            .into_output_folder(temp_dir.path())
            .execute()
            .expect("Failed to unveil message from image");

        assert_eq!(temp_dir.path().read_dir().unwrap().count(), 1);
        let secret_message = read_to_string(
            std::fs::File::open(temp_dir.path().join("secret-message.txt"))
                .expect("Failed to open file"),
        )
        .expect("Failed to read file");
        assert_eq!(secret_message, "Hello World");
    }
}
