use clap::Parser;

use stegano_core::commands::{unveil, unveil_raw};
use stegano_core::media::image::lsb_codec::CodecOptions;
use stegano_core::*;

mod cli;
use cli::*;

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    match handle_subcommands(args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_subcommands(args: Cli) -> std::result::Result<(), SteganoError> {
    let options = get_options(&args);
    match args.command {
        Commands::Hide(hide_args) => {
            let mut s = SteganoCore::encoder_with_options(options);

            s.use_media(hide_args.media)?
                .save_as(hide_args.write_to_file);

            if let Some(msg) = hide_args.message {
                s.add_message(msg.as_str())?;
            }

            if let Some(password) = hide_args.password {
                s.with_encryption(password);
            }

            if let Some(files) = hide_args.data_file {
                s.add_files(&files)?;
            }

            s.hide_and_save().map(|_| ())
        }
        Commands::Unveil(unveil_args) => unveil(
            &unveil_args.media,
            &unveil_args.output_folder,
            &options,
            unveil_args.password,
        ),
        Commands::UnveilRaw(args) => unveil_raw(&args.input_image, &args.output_file),
    }
}

fn get_options(args: &Cli) -> CodecOptions {
    CodecOptions {
        color_channel_step_increment: args.color_step_increment as _,
        ..Default::default()
    }
}
