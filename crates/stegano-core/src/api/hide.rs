use std::path::{Path, PathBuf};

use crate::{CodecOptions, SteganoCore, SteganoError};

pub fn prepare() -> HideApi {
    HideApi::default()
}

#[derive(Default, Debug)]
pub struct HideApi {
    message: Option<String>,
    files: Option<Vec<PathBuf>>,
    image: Option<PathBuf>,
    output: Option<PathBuf>,
    password: Option<String>,
    options: CodecOptions,
}

impl HideApi {
    pub fn with_options(mut self, options: CodecOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn use_message<S: AsRef<str>>(mut self, message: Option<S>) -> Self {
        self.message = message.map(|s| s.as_ref().to_string());
        self
    }

    pub fn use_files(mut self, data_files: Option<Vec<PathBuf>>) -> Self {
        self.files = data_files;
        self
    }

    pub fn with_files(mut self, data_files: Vec<PathBuf>) -> Self {
        self.files = Some(data_files);
        self
    }

    pub fn with_file<A: AsRef<Path>>(mut self, data_file: A) -> Self {
        let data_file = data_file.as_ref().to_path_buf();
        if let Some(files) = &mut self.files {
            files.push(data_file);
        } else {
            self.files = Some(vec![data_file]);
        }
        self
    }

    pub fn with_image<A: AsRef<Path>>(mut self, image: A) -> Self {
        self.image = Some(image.as_ref().to_path_buf());
        self
    }

    pub fn with_output<A: AsRef<Path>>(mut self, output: A) -> Self {
        self.output = Some(output.as_ref().to_path_buf());
        self
    }

    /// Set the password
    pub fn with_password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    /// Set the password
    /// If `None` is passed, no password will be used, leads to no de-/encryption used
    pub fn use_password<S: AsRef<str>>(mut self, password: Option<S>) -> Self {
        self.password = password.map(|s| s.as_ref().to_string());
        self
    }

    pub fn execute(self) -> Result<(), SteganoError> {
        self.validate()?;
        let Some(image) = self.image else {
            return Err(SteganoError::CarrierNotSet);
        };
        let Some(output) = self.output else {
            return Err(SteganoError::TargetNotSet);
        };

        let mut s = SteganoCore::encoder_with_options(self.options);
        s.use_media(&image)?.save_as(&output);

        if let Some(password) = self.password {
            s.with_encryption(password);
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
            .with_password("SuperSecret42")
            .with_output(temp_dir.path().join("image-with-secret.png"))
            .execute()
            .expect("Failed to hide message in image");
    }
}
