use std::path::PathBuf;

use clap::Args;
use stegano_core::CodecOptions;

use crate::CliResult;

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

impl UnveilArgs {
    pub fn run(self, options: CodecOptions) -> CliResult<()> {
        let password = if self.password.is_none() {
            crate::cli::ask_for_password()
        } else {
            self.password
        };

        stegano_core::api::unveil::prepare()
            .with_options(options)
            .from_secret_file(self.media)
            .into_output_folder(self.output_folder)
            .using_password(password)
            .execute()
    }
}
