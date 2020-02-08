use clap::{App, Arg, SubCommand, crate_version, AppSettings};

use stegano::*;

fn main() -> std::io::Result<()> {
    let matches = App::new("Stegano CLI")
        .version(crate_version!())
        .author("Sven Assmann <sven.assmann.it@gmail.com>")
        .about("Hiding secrets with steganography in PNG images.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("hide")
            .about("Hides data in PNG images")
            .arg(
                Arg::with_name("carrier_image")
                    .short("i")
                    .long("in")
                    .value_name("image source file")
                    .takes_value(true)
                    .required(true)
                    .help("Source image to hide data in (wont't manipulate this)"),
            )
            .arg(
                Arg::with_name("write_to_file")
                    .short("o")
                    .long("out")
                    .value_name("output image file")
                    .takes_value(true)
                    .required(true)
                    .help("Final image will be stored as file"),
            )
            .arg(
                Arg::with_name("data_file")
                    .short("d")
                    .long("data")
                    .value_name("data file")
                    .takes_value(true)
                    .required_unless("message")
                    .min_values(1)
                    .max_values(100)
                    .help("File(s) to hide in the image"),
            )
            .arg(
                Arg::with_name("message")
                    .short("m")
                    .long("message")
                    .value_name("text message")
                    .takes_value(true)
                    .required(false)
                    .help("A text message that will be hidden"),
            )
            .arg(
                Arg::with_name("force_content_version2")
                    .long("x-force-content-version-2")
                    .value_name("text message")
                    .takes_value(false)
                    .required(false)
                    .help("Experimental: enforce content version 2 encoding (for backwards compatibility)"),
            )
        ).subcommand(SubCommand::with_name("unveil")
            .about("Unveils data from PNG images")
            .arg(
                Arg::with_name("input_image")
                    .short("i")
                    .long("in")
                    .value_name("image source file")
                    .takes_value(true)
                    .required(true)
                    .help("Source image that contains secret data"),
            )
            .arg(
                Arg::with_name("output_folder")
                    .short("o")
                    .long("out")
                    .value_name("output folder")
                    .takes_value(true)
                    .required(true)
                    .help("Final data will be stored in that folder"),
            )
        ).subcommand(SubCommand::with_name("unveil-raw")
            .about("Unveils raw data in PNG images")
            .arg(
                Arg::with_name("input_image")
                    .short("i")
                    .long("in")
                    .value_name("image source file")
                    .takes_value(true)
                    .required(true)
                    .help("Source image that contains secret data"),
            )
            .arg(
                Arg::with_name("output_file")
                    .short("o")
                    .long("out")
                    .value_name("output file")
                    .takes_value(true)
                    .required(true)
                    .help("Raw data will be stored as binary file"),
            )
        ).get_matches();

    match matches.subcommand() {
        ("hide", Some(m)) => {
            let mut s = SteganoCore::encoder();

            s.use_carrier_image(m.value_of("carrier_image").unwrap())
             .write_to(m.value_of("write_to_file").unwrap());

            match m.value_of("message") {
                None => {}
                Some(msg) => {
                    s.hide_message(msg);
                },
            }

            match m.values_of("data_file") {
                None => {},
                Some(files) => {
                    s.hide_files(files.collect());
                },
            }

            if m.is_present("force_content_version2") {
                s.force_content_version(ContentVersion::V2);
            }

            s.hide();
        }
        ("unveil", Some(m)) => {
            SteganoCore::decoder()
                .use_source_image(m.value_of("input_image").unwrap())
                .write_to_folder(m.value_of("output_folder").unwrap())
                .unveil();
        }
        ("unveil-raw", Some(m)) => {
            SteganoCore::raw_decoder()
                .use_source_image(m.value_of("input_image").unwrap())
                .write_to_file(m.value_of("output_file").unwrap())
                .unveil();
        }
        _ => {
        }
    }

    Ok(())
}
