use std::path::Path;

use crate::{CodecOptions, SteganoError};

pub fn unveil(
    secret_media: &Path,
    output_folder: &Path,
    password: Option<String>,
    options: CodecOptions,
) -> Result<(), SteganoError> {
    crate::api::unveil::prepare()
        .with_options(options)
        .from_secret_file(secret_media)
        .into_output_folder(output_folder)
        .using_password(password)
        .execute()
}
