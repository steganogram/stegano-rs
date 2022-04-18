use clap::{crate_authors, crate_description, crate_version, Arg, ArgMatches, Command};

use std::path::Path;
use stegano_core::commands::{unveil, unveil_raw};
use stegano_core::media::image::lsb_codec::CodecOptions;
use stegano_core::*;

fn main() -> Result<()> {
    let matches = Command::new("Stegano CLI")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg_required_else_help(true)
        .subcommand(Command::new("hide")
            .about("Hides data in PNG images and WAV audio files")
            .arg(
                Arg::new("media")
                    .short('i')
                    .long("in")
                    .value_name("media file")
                    .takes_value(true)
                    .required(true)
                    .help("Media file such as PNG image or WAV audio file, used readonly."),
            )
            .arg(
                Arg::new("write_to_file")
                    .short('o')
                    .long("out")
                    .value_name("output image file")
                    .takes_value(true)
                    .required(true)
                    .help("Final image will be stored as file"),
            )
            .arg(
                Arg::new("data_file")
                    .short('d')
                    .long("data")
                    .value_name("data file")
                    .takes_value(true)
                    .required_unless_present("message")
                    .min_values(1)
                    .max_values(100)
                    .help("File(s) to hide in the image"),
            )
            .arg(
                Arg::new("message")
                    .short('m')
                    .long("message")
                    .value_name("text message")
                    .takes_value(true)
                    .required(false)
                    .help("A text message that will be hidden"),
            )
            .arg(
                Arg::new("force_content_version2")
                    .long("x-force-content-version-2")
                    .value_name("text message")
                    .takes_value(false)
                    .required(false)
                    .help("Experimental: enforce content version 2 encoding (for backwards compatibility)"),
            )
        )
        .subcommand(Command::new("unveil")
        .about("Unveils data from PNG images")
        .arg(
            Arg::new("input_image")
                .short('i')
                .long("in")
                .value_name("image source file")
                .takes_value(true)
                .required(true)
                .help("Source image that contains secret data"),
        )
        .arg(
            Arg::new("output_folder")
                .short('o')
                .long("out")
                .value_name("output folder")
                .takes_value(true)
                .required(true)
                .help("Final data will be stored in that folder"),
        )
    ).subcommand(Command::new("unveil-raw")
        .about("Unveils raw data in PNG images")
        .arg(
            Arg::new("input_image")
                .short('i')
                .long("in")
                .value_name("image source file")
                .takes_value(true)
                .required(true)
                .help("Source image that contains secret data"),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("out")
                .value_name("output file")
                .takes_value(true)
                .required(true)
                .help("Raw data will be stored as binary file"),
        )
    )
        .arg(
            Arg::new("color_step_increment")
                .long("x-color-step-increment")
                .value_name("color channel step increment")
                .takes_value(true)
                .default_value("1")
                .required(false)
                .help("Experimental: image color channel step increment"),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("hide", m)) => {
            let mut s = SteganoCore::encoder_with_options(get_options(m));

            s.use_media(m.value_of("media").unwrap())?
                .write_to(m.value_of("write_to_file").unwrap());

            match m.value_of("message") {
                None => {}
                Some(msg) => {
                    s.hide_message(msg);
                }
            }

            match m.values_of("data_file") {
                None => {}
                Some(files) => {
                    s.hide_files(files.collect());
                }
            }

            if m.is_present("force_content_version2") {
                s.force_content_version(ContentVersion::V2);
            }

            s.hide();
        }
        Some(("unveil", m)) => {
            unveil(
                Path::new(m.value_of("input_image").unwrap()),
                Path::new(m.value_of("output_folder").unwrap()),
                &get_options(m),
            )?;
        }
        Some(("unveil-raw", m)) => {
            unveil_raw(
                Path::new(m.value_of("input_image").unwrap()),
                Path::new(m.value_of("output_folder").unwrap()),
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn get_options(args: &ArgMatches) -> CodecOptions {
    let mut c = CodecOptions::default();
    if args.is_present("color_step_increment") {
        c.color_channel_step_increment = args
            .value_of("color_step_increment")
            .unwrap()
            .parse()
            .unwrap();
    }
    c
}
