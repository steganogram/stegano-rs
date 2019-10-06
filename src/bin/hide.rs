use clap::{App, Arg};

use stegano::*;

fn main() -> std::io::Result<()> {
    let matches = App::new("Stegano App")
        .version("0.1")
        .author("Sven Assmann <sven.assmann.it@gmail.com>")
        // .about("Implements LSB steganography for PNG image files in rust-lang. Aims for a command line version only.")
        // .subcommand(SubCommand::with_name("test")
        .about("Hides data in PNG images")
        .arg(
            Arg::with_name("carrier_image")
                .short("i")
                .long("in")
                .value_name("image source file")
                .takes_value(true)
                .required(true)
                .help("Source image to hide data in (wont't mainipulate this)"),
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
        .get_matches();

    Steganogramm::new()
        .use_carrier_image(matches.value_of("carrier_image").unwrap())
        .take_data_to_hide_from(matches.value_of("data_file").unwrap())
        .write_to(matches.value_of("write_to_file").unwrap())
        .hide();

    Ok(())
}
