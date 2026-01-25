use std::path::{Path, PathBuf};

use crate::{SteganoEncoder, SteganoError};

use super::Password;

/// Prepares the hide API for further configuration
pub fn prepare() -> HideApi {
    HideApi::default()
}

#[derive(Default, Debug)]
pub struct HideApi {
    message: Option<String>,
    files: Option<Vec<PathBuf>>,
    image: Option<PathBuf>,
    output: Option<PathBuf>,
    password: Password,
    color_channel_step_increment: Option<usize>,
}

impl HideApi {
    /// This is the message that will be hidden
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// This is the message that will be hidden
    /// If `None` is passed, no message will be hidden
    pub fn use_message<S: AsRef<str>>(mut self, message: Option<S>) -> Self {
        self.message = message.map(|s| s.as_ref().to_string());
        self
    }

    /// This are the files that will be hidden
    /// If `None` is passed, no files will be hidden
    pub fn use_files(mut self, data_files: Option<Vec<PathBuf>>) -> Self {
        self.files = data_files;
        self
    }

    /// This are the files that will be hidden
    /// Note: this will overwrite any previously set files
    pub fn with_files(mut self, data_files: Vec<PathBuf>) -> Self {
        self.files = Some(data_files);
        self
    }

    /// This is the file that will be hidden
    /// Note: this will add the file to the list of files to hide
    pub fn with_file<A: AsRef<Path>>(mut self, data_file: A) -> Self {
        let data_file = data_file.as_ref().to_path_buf();
        if let Some(files) = &mut self.files {
            files.push(data_file);
        } else {
            self.files = Some(vec![data_file]);
        }
        self
    }

    /// This is the carrier image
    pub fn with_image<A: AsRef<Path>>(mut self, image: A) -> Self {
        self.image = Some(image.as_ref().to_path_buf());
        self
    }

    /// This is the output image/audio
    pub fn with_output<A: AsRef<Path>>(mut self, output: A) -> Self {
        self.output = Some(output.as_ref().to_path_buf());
        self
    }

    /// Set the password used for encrypting all data
    /// If `None` is passed, no password will be used, leads to no de-/encryption used
    pub fn using_password<P: Into<Password>>(mut self, password: P) -> Self {
        self.password = password.into();
        self
    }

    /// Set the color channel step increment for LSB encoding.
    ///
    /// This controls how pixels are traversed during encoding.
    /// Only applies to PNG output files using LSB steganography.
    /// For JPEG output files (F5 steganography), this setting is ignored.
    pub fn with_color_step_increment(mut self, step: usize) -> Self {
        self.color_channel_step_increment = Some(step);
        self
    }

    /// Execute the hiding process and blocks until it is finished
    pub fn execute(self) -> Result<(), SteganoError> {
        self.validate()?;
        let Some(ref image) = self.image else {
            return Err(SteganoError::CarrierNotSet);
        };
        let Some(ref output) = self.output else {
            return Err(SteganoError::TargetNotSet);
        };

        let mut s = SteganoEncoder::new();
        s.use_media(image)?.save_as(output);

        if let Some(password) = self.password.as_ref() {
            s.with_encryption(password);
        }

        if let Some(step) = self.color_channel_step_increment {
            s.with_color_step_increment(step);
        }

        if let Some(message) = self.message {
            s.add_message(message.as_str())?;
        }

        if let Some(files) = self.files {
            s.add_files(&files)?;
        }

        s.hide_and_save()?;

        Ok(())
    }

    fn validate(&self) -> Result<(), SteganoError> {
        if self.message.is_none() && self.files.is_none() {
            if self.message.is_none() {
                return Err(SteganoError::MissingMessage);
            }
            if self.files.is_none() {
                return Err(SteganoError::MissingFiles);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    #[test]
    fn illustrate_api_usage() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        crate::api::hide::prepare()
            .with_message("Hello, World!")
            .with_image("tests/images/plain/carrier-image.png")
            .using_password("SuperSecret42")
            .with_output(temp_dir.path().join("image-with-secret.png"))
            .execute()
            .expect("Failed to hide message in image");
    }

    #[test]
    fn tests_validation_message_is_set() {
        assert!(matches!(
            crate::api::hide::prepare().execute().unwrap_err(),
            crate::SteganoError::MissingMessage
        ));
    }

    #[test]
    fn tests_validation_carrier_is_set() {
        assert!(matches!(
            crate::api::hide::prepare()
                .with_message("foo")
                .execute()
                .unwrap_err(),
            crate::SteganoError::CarrierNotSet
        ));
    }

    #[test]
    fn tests_validation_target_is_set() {
        assert!(matches!(
            crate::api::hide::prepare()
                .with_message("foo")
                .with_image("foo")
                .execute()
                .unwrap_err(),
            crate::SteganoError::TargetNotSet
        ));
    }

    #[test]
    fn should_hide_in_jpeg_without_password() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("secret.jpg");

        crate::api::hide::prepare()
            .with_message("Hello from JPEG!")
            .with_image("tests/images/NoSecrets.jpg")
            .with_output(&output)
            .execute()
            .expect("Failed to hide in JPEG");

        let data = std::fs::read(&output).unwrap();
        assert_eq!(&data[0..2], &[0xFF, 0xD8], "Should be a valid JPEG");
        assert!(data.len() > 100, "Output should not be trivially small");
    }

    #[test]
    fn should_hide_in_jpeg_with_password() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("secret.jpg");

        crate::api::hide::prepare()
            .with_message("Secret JPEG message")
            .with_image("tests/images/NoSecrets.jpg")
            .using_password("MyPassword42")
            .with_output(&output)
            .execute()
            .expect("Failed to hide in JPEG with password");

        let data = std::fs::read(&output).unwrap();
        assert_eq!(&data[0..2], &[0xFF, 0xD8], "Should be a valid JPEG");
    }

    #[test]
    fn should_hide_in_png_source_and_output_jpeg() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("from_png.jpg");

        crate::api::hide::prepare()
            .with_message("Cross-format test")
            .with_image("tests/images/plain/carrier-image.png")
            .using_password("CrossFormat42")
            .with_output(&output)
            .execute()
            .expect("Failed to hide PNG to JPEG");

        crate::api::unveil::prepare()
            .from_secret_file(&output)
            .using_password("CrossFormat42")
            .into_output_folder(temp_dir.path())
            .execute()
            .expect("Failed to unveil from JPEG");

        let msg = std::fs::read_to_string(temp_dir.path().join("secret-message.txt")).unwrap();
        assert_eq!(msg, "Cross-format test");
    }

    #[test]
    fn should_hide_and_unveil_binary_file_in_jpeg() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output = temp_dir.path().join("stegano.jpg");
        let secret = "tests/images/secrets/random_1666_byte.bin";

        crate::api::hide::prepare()
            .with_file(secret)
            .with_image("tests/images/NoSecrets.jpg")
            .using_password("BinaryFilePass")
            .with_output(&output)
            .execute()
            .expect("Failed to hide file in JPEG");

        crate::api::unveil::prepare()
            .from_secret_file(&output)
            .using_password("BinaryFilePass")
            .into_output_folder(temp_dir.path())
            .execute()
            .expect("Failed to unveil from JPEG");

        let expected = std::fs::read(secret).unwrap();
        let actual = std::fs::read(temp_dir.path().join("random_1666_byte.bin")).unwrap();
        assert_eq!(actual, expected, "Unveiled binary data did not match");
    }

    // create some tests for the files methods
    #[test]
    fn test_adding_files() {
        let api = crate::api::hide::prepare()
            .with_message("foo")
            .with_image("foo");

        let api = api.with_file("x").with_file("y").with_file("z");
        assert_eq!(api.files.as_ref().unwrap().len(), 3);

        let api = api.with_files(vec!["a".into(), "b".into()]);
        assert_eq!(api.files.as_ref().unwrap().len(), 2);

        let api = api.use_files(None);
        assert!(api.files.as_ref().is_none());
    }
}
