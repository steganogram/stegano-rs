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
        .with_secret_image(secret_media)
        .with_output_folder(output_folder)
        .use_password(password)
        .execute()
}
