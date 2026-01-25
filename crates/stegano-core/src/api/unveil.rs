use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    media::{
        audio, image,
        payload::{FabA, FabS, PayloadCodecFactory},
        LsbCodecOptions, Media,
    },
    Message, SteganoError,
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
    options: LsbCodecOptions,
}

impl UnveilApi {
    /// Use the given LSB codec options
    pub fn with_options(mut self, options: LsbCodecOptions) -> Self {
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
        let Some(ref secret_media) = self.secret_media else {
            return Err(SteganoError::CarrierNotSet);
        };
        let Some(ref output_folder) = self.output_folder else {
            return Err(SteganoError::TargetNotSet);
        };

        let msg = self.unveil(secret_media)?;

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

    fn unveil(&self, secret_media: &Path) -> Result<Message, SteganoError> {
        let media = Media::from_file(secret_media)?;
        let fab: Box<dyn PayloadCodecFactory> = if let Some(password) = self.password.as_ref() {
            Box::new(FabS::new(password))
        } else {
            Box::new(FabA)
        };

        match media {
            Media::Image(img) => {
                let mut decoder = image::LsbCodec::decoder(&img, &self.options);
                Message::from_raw_data(&mut decoder, &*fab)
            }
            Media::ImageJpeg { source, .. } => {
                // F5 extraction - derive seed from password
                let seed: Option<Vec<u8>> = self
                    .password
                    .as_ref()
                    .as_ref()
                    .map(|p| p.as_bytes().to_vec());

                let mut decoder = image::F5JpegDecoder::new(&source, seed.as_deref())?;
                Message::from_raw_data(&mut decoder, &*fab)
            }
            Media::Audio(audio) => {
                let mut decoder = audio::LsbCodec::decoder(&audio.1);
                Message::from_raw_data(&mut decoder, &*fab)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::read_to_string;

    use tempfile::tempdir;

    #[test]
    fn should_hide_and_unveil_text_in_jpeg_without_password() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("secret.jpg");

        crate::api::hide::prepare()
            .with_message("Hello JPEG!")
            .with_image("tests/images/NoSecrets.jpg")
            .with_output(&output)
            .execute()
            .expect("Failed to hide");

        crate::api::unveil::prepare()
            .from_secret_file(&output)
            .into_output_folder(temp_dir.path())
            .execute()
            .expect("Failed to unveil");

        let msg = std::fs::read_to_string(temp_dir.path().join("secret-message.txt")).unwrap();
        assert_eq!(msg, "Hello JPEG!");
    }

    #[test]
    fn should_hide_and_unveil_text_in_jpeg_with_password() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("secret.jpg");

        crate::api::hide::prepare()
            .with_message("Encrypted JPEG!")
            .with_image("tests/images/NoSecrets.jpg")
            .using_password("TestPass123")
            .with_output(&output)
            .execute()
            .expect("Failed to hide");

        crate::api::unveil::prepare()
            .from_secret_file(&output)
            .using_password("TestPass123")
            .into_output_folder(temp_dir.path())
            .execute()
            .expect("Failed to unveil");

        let msg = std::fs::read_to_string(temp_dir.path().join("secret-message.txt")).unwrap();
        assert_eq!(msg, "Encrypted JPEG!");
    }

    #[test]
    fn should_fail_unveil_jpeg_with_wrong_password() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("secret.jpg");

        crate::api::hide::prepare()
            .with_message("Secret!")
            .with_image("tests/images/NoSecrets.jpg")
            .using_password("CorrectPassword")
            .with_output(&output)
            .execute()
            .expect("Failed to hide");

        let result = crate::api::unveil::prepare()
            .from_secret_file(&output)
            .using_password("WrongPassword")
            .into_output_folder(temp_dir.path())
            .execute();

        // Should fail: wrong seed produces wrong coefficient order, wrong password fails decryption
        assert!(result.is_err());
    }

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
