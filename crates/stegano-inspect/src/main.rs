use argh::FromArgs;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use stegano_f5::parse_quantization_tables;

#[derive(FromArgs)]
/// Inspecting jpeg image files
struct SteganoInspectArgs {
    /// inspect the quantization tables of the given jpeg
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Quantization(QuantizationArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Shows the quantization tables of jpeg files
#[argh(subcommand, name = "quantization")]
struct QuantizationArgs {
    /// the jpeg image file to inspect
    #[argh(positional)]
    jpeg_files: Vec<String>,
}

fn main() {
    let args: SteganoInspectArgs = argh::from_env();

    match &args.command {
        Command::Quantization(args) => {
            for file_name in args.jpeg_files.iter() {
                let file = File::open(file_name.as_str()).expect("cannot open file");
                let mut reader = BufReader::new(file);

                let tables = parse_quantization_tables(&mut reader)
                    .expect("Failed to parse quantization tables");

                println!(
                    "# Quantization Tables of `{}`",
                    fs::canonicalize(file_name.as_str())
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                );
                println!();
                for table in &tables {
                    println!(
                        "## Table {} (precision: {}-bit)",
                        table.id,
                        if table.precision == 0 { 8 } else { 16 }
                    );
                    print!("{}", table.to_ascii_table());
                    println!();
                }
            }
        }
    }
}
