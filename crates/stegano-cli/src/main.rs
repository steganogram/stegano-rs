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
                    .required(true)
                    .help("Media file such as PNG image or WAV audio file, used readonly."),
            )
            .arg(
                Arg::new("write_to_file")
                    .short('o')
                    .long("out")
                    .value_name("output image file")
                    .required(true)
                    .help("Final image will be stored as file"),
            )
            .arg(
                Arg::new("data_file")
                    .short('d')
                    .long("data")
                    .value_name("data file")
                    .required_unless_present("message")
                    .num_args(1..100)
                    .help("File(s) to hide in the image"),
            )
            .arg(
                Arg::new("message")
                    .short('m')
                    .long("message")
                    .value_name("text message")
                    .required(false)
                    .help("A text message that will be hidden"),
            )
            .arg(
                Arg::new("force_content_version2")
                    .long("x-force-content-version-2")
                    .value_name("text message")
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
                .required(true)
                .help("Source image that contains secret data"),
        )
        .arg(
            Arg::new("output_folder")
                .short('o')
                .long("out")
                .value_name("output folder")
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
                .required(true)
                .help("Source image that contains secret data"),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("out")
                .value_name("output file")
                .required(true)
                .help("Raw data will be stored as binary file"),
        )
    )
        .arg(
            Arg::new("color_step_increment")
                .long("x-color-step-increment")
                .value_name("color channel step increment")
                .default_value("1")
                .required(false)
                .help("Experimental: image color channel step increment"),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("hide", m)) => {
            let mut s = SteganoCore::encoder_with_options(get_options(&matches));

            s.use_media(m.get_one::<String>("media").unwrap())?
                .write_to(m.get_one::<String>("write_to_file").unwrap());

            if let Some(msg) = m.get_one::<String>("message") {
                s.hide_message(msg);
            }

            if let Some(files) = m.get_many::<String>("data_file") {
                s.hide_files(files.map(|f| &**f).collect());
            }

            if m.contains_id("force_content_version2") {
                s.force_content_version(ContentVersion::V2);
            }

            s.hide();
        }
        Some(("unveil", m)) => {
            unveil(
                Path::new(m.get_one::<String>("input_image").unwrap()),
                Path::new(m.get_one::<String>("output_folder").unwrap()),
                &get_options(&matches),
            )?;
        }
        Some(("unveil-raw", m)) => {
            unveil_raw(
                Path::new(m.get_one::<String>("input_image").unwrap()),
                Path::new(m.get_one::<String>("output_folder").unwrap()),
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn get_options(args: &ArgMatches) -> CodecOptions {
    let mut c = CodecOptions::default();
    if args.contains_id("color_step_increment") {
        c.color_channel_step_increment = args
            .get_one::<String>("color_step_increment")
            .unwrap()
            .parse()
            .unwrap();
    }
    c
}
