use clap::Parser;

use stegano_core::*;

mod cli;
use cli::*;
mod commands;

pub type CliResult<T> = std::result::Result<T, SteganoError>;

fn main() -> Result<()> {
    env_logger::init();

    let args = CliArgs::parse();
    if let Err(err) = handle_subcommands(args) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    Ok(())
}

fn handle_subcommands(args: CliArgs) -> CliResult<()> {
    let options = get_options(&args);
    match args.command {
        Commands::Hide(hide) => hide.run(options),
        Commands::Unveil(unveil) => unveil.run(options),
        Commands::UnveilRaw(unveil_raw) => unveil_raw.run(options),
    }
}

fn get_options(args: &CliArgs) -> LsbCodecOptions {
    LsbCodecOptions {
        color_channel_step_increment: args.color_step_increment as _,
        ..Default::default()
    }
}
