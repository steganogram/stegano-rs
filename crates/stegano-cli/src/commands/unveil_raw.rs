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
    pub fn run(self, color_step_increment: usize) -> crate::CliResult<()> {
        let password = if self.password.is_none() {
            crate::cli::ask_for_password(false)
        } else {
            self.password
        };

        stegano_core::api::unveil_raw::prepare()
            .with_color_step_increment(color_step_increment)
            .from_secret_file(self.media)
            .into_raw_file(self.output_file)
            .using_password(password)
            .execute()
    }
}
