use clap::{App, Arg, SubCommand};

use stegano::*;

fn main() -> std::io::Result<()> {
    let matches = App::new("Stegano App")
        .version("0.1")
        .author("Sven Assmann <sven.assmann.it@gmail.com>")
        .about("Implements LSB steganography for PNG image files in rust-lang. Aims for a command line version only.")
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
                    .required(true)
                    .help("Data of that file that will be hidden"),
            )
        ).subcommand(SubCommand::with_name("unveil")
            .about("Unveils data in PNG images")
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
                    .help("Final data will be stored as file"),
            )
        ).get_matches();

    match matches.subcommand() {
        ("hide", Some(m)) => {
            SteganoEncoder::new()
                .use_carrier_image(m.value_of("carrier_image").unwrap())
                .take_data_to_hide_from(m.value_of("data_file").unwrap())
                .write_to(m.value_of("write_to_file").unwrap())
                .hide();
        }
        ("unveil", Some(m)) => {
            SteganoDecoderV2::new()
                .use_source_image(m.value_of("input_image").unwrap())
                .write_to_file(m.value_of("output_file").unwrap())
                .unveil();
        }
        _ => {
            // TODO consider to show the sub command list
        }
    }

    Ok(())
}
