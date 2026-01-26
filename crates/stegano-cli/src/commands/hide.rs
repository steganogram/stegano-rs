use std::path::PathBuf;

use clap::Args;

use crate::CliResult;

/// Hides data in PNG images and WAV audio files
#[derive(Args, Debug)]
pub struct HideArgs {
    /// Password used to encrypt the data
    #[arg(long, value_name = "password")]
    pub password: Option<String>,

    /// Media file such as PNG image or WAV audio file, used readonly.
    #[arg(short = 'i', long = "in", value_name = "media file", required = true)]
    pub media: PathBuf,

    /// Final image will be stored as file
    #[arg(
        short = 'o',
        long = "out",
        value_name = "output image file",
        required = true
    )]
    pub write_to_file: PathBuf,

    /// File(s) to hide in the image
    #[arg(
        short = 'd',
        long = "data",
        value_name = "data files",
        required_unless_present = "message"
    )]
    pub data_files: Option<Vec<PathBuf>>,

    /// A text message that will be hidden
    #[arg(
        short,
        long,
        value_name = "text message",
        required_unless_present = "data_files"
    )]
    pub message: Option<String>,

    /// JPEG quality for F5 steganography (1-100). Only applies to JPEG output.
    #[arg(
        long,
        value_name = "quality",
        value_parser = clap::value_parser!(u8).range(1..=100)
    )]
    pub jpeg_quality: Option<u8>,
}

impl HideArgs {
    pub fn run(self, color_step_increment: usize) -> CliResult<()> {
        let password = if self.password.is_none() {
            crate::cli::ask_for_password(true)
        } else {
            self.password
        };

        // Note: Codec is determined by target file extension:
        // - .png → LSB encoding (uses color_step_increment)
        // - .jpg/.jpeg → F5 encoding (uses jpeg_quality)
        let mut api = stegano_core::api::hide::prepare()
            .with_color_step_increment(color_step_increment)
            .with_image(self.media)
            .with_output(self.write_to_file)
            .using_password(password)
            .use_files(self.data_files)
            .use_message(self.message);

        if let Some(quality) = self.jpeg_quality {
            api = api.with_jpeg_quality(quality);
        }

        api.execute()
    }
}
