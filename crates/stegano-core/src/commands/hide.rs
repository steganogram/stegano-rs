use std::path::{Path, PathBuf};

use crate::{CodecOptions, SteganoError};

pub fn hide(
    media: &Path,
    write_to_file: &Path,
    data_files: Option<Vec<PathBuf>>,
    message: Option<String>,
    password: Option<String>,
    options: CodecOptions,
) -> Result<(), SteganoError> {
    crate::api::hide::prepare()
        .with_options(options)
        .with_image(media)
        .with_output(write_to_file)
        .use_password(password)
        .use_files(data_files)
        .use_message(message)
        .execute()
}
