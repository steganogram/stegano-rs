use clap::{Parser, Subcommand};
use dialoguer::Password;

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

pub fn ask_for_password() -> Option<String> {
    eprintln!("Warning: No password provided. We recommend always using encryption.");
    eprintln!("         Skip on your own risk.");
    let password = Password::new()
        .with_prompt("Password")
        .allow_empty_password(true)
        .interact()
        .expect("Failed to read password");

    if password.is_empty() {
        None
    } else {
        Some(password)
    }
}

pub fn ask_for_password_twice() -> Option<String> {
    eprintln!("Warning: No password provided. We recommend always using encryption.");
    eprintln!("         Skip on your own risk.");
    let password = Password::new()
        .with_prompt("Password")
        .with_confirmation("Confirm password", "Passwords mismatching")
        .allow_empty_password(true)
        .interact()
        .expect("Failed to read password");

    if password.is_empty() {
        None
    } else {
        Some(password)
    }
}
