use std::path::PathBuf;

use clap::Args;
use stegano_core::CodecOptions;

use crate::CliResult;

/// Hides data in PNG images and WAV audio files
#[derive(Args, Debug)]
pub struct HideArgs {
    /// Password used to encrypt the data
    #[arg(short, long, value_name = "password")]
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
}

impl HideArgs {
    pub fn run(self, options: CodecOptions) -> CliResult<()> {
        stegano_core::commands::hide(
            &self.media,
            &self.write_to_file,
            self.data_files,
            self.message,
            self.password,
            options,
        )
    }
}
