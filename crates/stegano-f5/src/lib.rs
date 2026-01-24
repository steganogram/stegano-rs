//! F5 Steganography Algorithm for JPEG Images
//!
//! This crate implements the F5 steganographic algorithm developed by Andreas Westfeld.
//! F5 embeds data into quantized DCT coefficients using matrix encoding and permutative
//! straddling.
//!
//! # High-Level API
//!
//! The simplest way to use this crate is with the high-level JPEG functions:
//!
//! ```ignore
//! use stegano_f5::{embed_in_jpeg, extract_from_jpeg, jpeg_capacity};
//!
//! // Check capacity
//! let cover = std::fs::read("cover.jpg")?;
//! let capacity = jpeg_capacity(&cover)?;
//! println!("Can embed up to {} bytes", capacity);
//!
//! // Embed a message
//! let message = b"Secret message";
//! let seed = b"optional_seed";
//! let stego = embed_in_jpeg(&cover, message, Some(seed))?;
//! std::fs::write("stego.jpg", stego)?;
//!
//! // Extract the message
//! let stego = std::fs::read("stego.jpg")?;
//! let extracted = extract_from_jpeg(&stego, Some(seed))?;
//! ```
//!
//! # Low-Level API
//!
//! For more control, use the F5Encoder/F5Decoder directly with coefficient access:
//!
//! ```ignore
//! use stegano_f5::{F5Encoder, F5Decoder, jpeg};
//!
//! // Parse JPEG and decode coefficients
//! let segments = jpeg::parse_jpeg(&jpeg_data)?;
//! let mut coefficients = jpeg::decode_scan(&segments)?;
//!
//! // Embed using F5
//! let encoder = F5Encoder::new();
//! encoder.embed(coefficients.as_mut_slice(), message, Some(seed))?;
//!
//! // Re-encode and write JPEG
//! let new_scan = jpeg::encode_scan(&coefficients, &segments)?;
//! let output = jpeg::write_jpeg(&segments, &new_scan);
//! ```

mod decoder;
mod encoder;
mod error;
pub mod jpeg;
mod matrix;
mod permutation;

pub use decoder::F5Decoder;
pub use encoder::F5Encoder;
pub use error::{F5Error, Result};
pub use jpeg::{
    embed_in_jpeg, extract_from_jpeg, jpeg_capacity, parse_jpeg, parse_quantization_tables,
    JpegSegments, QuantizationTable,
};
pub use matrix::CheckMatrix;
pub use permutation::Permutation;
