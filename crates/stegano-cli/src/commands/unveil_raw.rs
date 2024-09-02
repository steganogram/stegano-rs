use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct UnveilRawArgs {
    /// Password used to encrypt the data
    #[arg(short, long, value_name = "password")]
    pub password: Option<String>,

    /// Source media that contains secret data
    #[arg(
        short = 'i',
        long = "in",
        value_name = "media source file",
        required = true
    )]
    pub media: PathBuf,

    /// Raw data will be stored as binary file
    #[arg(short = 'o', long = "out", value_name = "output file", required = true)]
    pub output_file: PathBuf,
}

impl UnveilRawArgs {
    pub fn run(self, _options: stegano_core::CodecOptions) -> crate::CliResult<()> {
        stegano_core::commands::unveil_raw(&self.media, &self.output_file, self.password)
    }
}
