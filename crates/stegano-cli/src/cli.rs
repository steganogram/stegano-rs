use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Experimental: image color channel step increment
    #[arg(long = "x-color-step-increment", default_value = "1")]
    pub color_step_increment: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Hides data in PNG images and WAV audio files
    Hide(HideArgs),
    Unveil(UnveilArgs),
    UnveilRaw(UnveilRawArgs),
}

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
        value_name = "data file",
        required_unless_present = "message"
    )]
    pub data_file: Option<Vec<PathBuf>>,

    /// A text message that will be hidden
    #[arg(
        short,
        long,
        value_name = "text message",
        required_unless_present = "data_file"
    )]
    pub message: Option<String>,
}

#[derive(Args, Debug)]
pub struct UnveilArgs {
    /// Password used to encrypt the data
    #[arg(short, long, value_name = "password")]
    pub password: Option<String>,

    /// Source image that contains secret data
    #[arg(
        short = 'i',
        long = "in",
        value_name = "media source file",
        required = true
    )]
    pub media: PathBuf,

    /// Final data will be stored in that folder
    #[arg(
        short = 'o',
        long = "out",
        value_name = "output folder",
        required = true
    )]
    pub output_folder: PathBuf,
}

#[derive(Args, Debug)]
pub struct UnveilRawArgs {
    /// Source media that contains secret data
    #[arg(
        short = 'i',
        long = "in",
        value_name = "media source file",
        required = true
    )]
    pub input_image: PathBuf,

    /// Raw data will be stored as binary file
    #[arg(short = 'o', long = "out", value_name = "output file", required = true)]
    pub output_file: PathBuf,
}
