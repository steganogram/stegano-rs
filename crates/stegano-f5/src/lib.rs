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
//! // Embed a message (transcode existing JPEG)
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
//! For more control, use the F5Encoder/F5Decoder directly on coefficient slices:
//!
//! ```ignore
//! use stegano_f5::{F5Encoder, F5Decoder};
//!
//! // Embed into pre-obtained coefficients (zigzag order, flat i16 slice)
//! let encoder = F5Encoder::new();
//! encoder.embed(&mut coefficients, message, Some(seed))?;
//!
//! // Extract from coefficients
//! let decoder = F5Decoder::new();
//! let extracted = decoder.extract(&coefficients, Some(seed))?;
//! ```

mod decoder;
mod encoder;
mod error;
mod jpeg_ops;
mod matrix;
mod permutation;

pub use decoder::F5Decoder;
pub use encoder::F5Encoder;
pub use error::{F5Error, Result};
pub use jpeg_ops::{embed_in_jpeg, embed_in_jpeg_from_image, extract_from_jpeg, jpeg_capacity};
pub use matrix::CheckMatrix;
pub use permutation::Permutation;
