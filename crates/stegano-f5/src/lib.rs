//! F5 Steganography Algorithm for JPEG Images
//!
//! This crate implements the F5 steganographic algorithm developed by Andreas Westfeld.
//! F5 embeds data into quantized DCT coefficients using matrix encoding and permutative
//! straddling.
//!
//! # Layer Responsibilities
//!
//! This crate handles **encoding-level** concerns only:
//! - Embedding raw bytes into DCT coefficients
//! - Extracting raw bytes from DCT coefficients
//! - Matrix encoding with parameter `w`
//! - Coefficient permutation for uniform spreading
//!
//! Message format, compression, and encryption are handled by outer layers (e.g., `stegano-core`).
//!
//! # Example
//!
//! ```ignore
//! use stegano_f5::{F5Encoder, F5Decoder};
//!
//! // Embed data into DCT coefficients
//! let mut coefficients: Vec<i16> = /* from JPEG decoder */;
//! let message = b"Hello World";
//! let seed = b"permutation_seed";
//!
//! let encoder = F5Encoder::new();
//! encoder.embed(&mut coefficients, message, Some(seed))?;
//!
//! // Extract data from DCT coefficients
//! let decoder = F5Decoder::new();
//! let extracted = decoder.extract(&coefficients, Some(seed))?;
//! assert_eq!(extracted, message);
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
pub use jpeg::{parse_quantization_tables, QuantizationTable};
pub use matrix::CheckMatrix;
pub use permutation::Permutation;
