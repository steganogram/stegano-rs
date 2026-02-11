//! Hide a message in a JPEG file using F5 steganography.
//!
//! Usage: cargo run -p stegano-f5 --example hide -- <input.jpg> <seed> <message>
//!
//! The output file will be named <input>_stegano.jpg

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <input.jpg> <seed> <message>", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  input.jpg  - Cover JPEG image");
        eprintln!("  seed       - Password/seed for permutation (use \"\" for no seed)");
        eprintln!("  message    - Message to hide");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run -p stegano-f5 --example hide -- cover.jpg mysecret \"Hello World\"");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let seed = &args[2];
    let message = &args[3];

    // Read cover image
    let cover_data = match fs::read(input_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_path, e);
            std::process::exit(1);
        }
    };

    // Check capacity
    let capacity = match stegano_f5::jpeg_capacity(&cover_data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error analyzing JPEG: {}", e);
            std::process::exit(1);
        }
    };

    println!("Cover image: {}", input_path);
    println!("Capacity: {} bytes", capacity);
    println!("Message length: {} bytes", message.len());

    if message.len() > capacity {
        eprintln!(
            "Error: Message too large ({} bytes) for image capacity ({} bytes)",
            message.len(),
            capacity
        );
        std::process::exit(1);
    }

    // Embed message
    let seed_bytes = if seed.is_empty() {
        None
    } else {
        Some(seed.as_bytes())
    };

    let stego_data = match stegano_f5::embed_in_jpeg(&cover_data, message.as_bytes(), seed_bytes) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error embedding message: {}", e);
            std::process::exit(1);
        }
    };

    // Generate output filename
    let input = Path::new(input_path);
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let output_path = input
        .parent()
        .unwrap_or(Path::new("."))
        .join(format!("{}_stegano.jpg", stem));

    // Write output
    if let Err(e) = fs::write(&output_path, &stego_data) {
        eprintln!("Error writing {}: {}", output_path.display(), e);
        std::process::exit(1);
    }

    println!("Output: {}", output_path.display());
    println!(
        "Size: {} -> {} bytes ({:+} bytes)",
        cover_data.len(),
        stego_data.len(),
        stego_data.len() as i64 - cover_data.len() as i64
    );
    println!("Done!");
}
