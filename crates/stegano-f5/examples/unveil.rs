//! Extract a hidden message from a JPEG file using F5 steganography.
//!
//! Usage: cargo run -p stegano-f5 --example unveil -- <stego.jpg> <seed>

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <stego.jpg> <seed>", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  stego.jpg  - JPEG image with hidden message");
        eprintln!("  seed       - Password/seed used during embedding (use \"\" for no seed)");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run -p stegano-f5 --example unveil -- stego.jpg mysecret");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let seed = &args[2];

    // Read stego image
    let stego_data = match fs::read(input_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_path, e);
            std::process::exit(1);
        }
    };

    // Extract message
    let seed_bytes = if seed.is_empty() {
        None
    } else {
        Some(seed.as_bytes())
    };

    let message = match stegano_f5::extract_from_jpeg(&stego_data, seed_bytes) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error extracting message: {}", e);
            std::process::exit(1);
        }
    };

    // Try to print as UTF-8, fall back to hex if not valid
    match String::from_utf8(message.clone()) {
        Ok(text) => {
            println!("{}", text);
        }
        Err(_) => {
            eprintln!("Message is not valid UTF-8, showing as hex:");
            for byte in &message {
                print!("{:02x}", byte);
            }
            println!();
        }
    }
}
