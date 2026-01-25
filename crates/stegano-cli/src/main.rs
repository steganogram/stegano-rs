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
    let color_step = args.color_step_increment as usize;
    match args.command {
        Commands::Hide(hide) => hide.run(color_step),
        Commands::Unveil(unveil) => unveil.run(color_step),
        Commands::UnveilRaw(unveil_raw) => unveil_raw.run(color_step),
    }
}
