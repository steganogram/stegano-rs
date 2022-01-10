use argh::FromArgs;
use std::fs;
use std::fs::File;
use std::io::BufReader;

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
    Quantization(QantizationArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Shows the quantization tables of jpeg files
#[argh(subcommand, name = "quantization")]
struct QantizationArgs {
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
                let mut decoder = jpeg_decoder::Decoder::new(BufReader::new(file));
                decoder.decode().expect("Decoding failed. If other software can successfully decode the specified JPEG image, then it's likely that there is a bug in jpeg-decoder");

                println!(
                    "# Quantization Tables of {}",
                    fs::canonicalize(file_name.as_str())
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                );
                println!();
                for table in decoder.quantization_tables.iter().flatten() {
                    print_quantization_table(table);
                }
            }
        }
    }
}

fn print_quantization_table(table: &[u16; 64]) {
    const W: usize = 8;
    print!("|    |");
    for x in 0..W {
        print!("   x{x} |", x = x);
    }
    println!();
    print!("|----");
    for _ in 0..W {
        print!("|------");
    }
    println!("|");
    for y in 0..W {
        print!("| y{y} ", y = y);
        for x in 0..W {
            let px = x;
            let py = y * W;

            print!("| {:4} ", table[px + py]);
        }
        println!("|");
    }
    println!();
}
