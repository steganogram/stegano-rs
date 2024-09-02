use clap::{Parser, Subcommand};

use crate::commands::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
    /// Experimental: image color channel step increment
    #[arg(long = "x-color-step-increment", default_value = "1")]
    pub color_step_increment: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Hide(hide::HideArgs),
    Unveil(unveil::UnveilArgs),
    UnveilRaw(unveil_raw::UnveilRawArgs),
}
