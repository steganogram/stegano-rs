//! JPEG transcoding module for F5 steganography.
//!
//! This module provides coefficient-level access to JPEG files without
//! full decode/re-encode. It extracts DCT coefficients via Huffman decoding
//! and re-encodes them after F5 modification.
//!
//! # Architecture
//!
//! ```text
//! JPEG → parse → huffman decode → [i16] coefficients → F5 → huffman encode → JPEG
//! ```
//!
//! Code is adapted from:
//! - [jpeg-decoder](https://github.com/image-rs/jpeg-decoder) - parsing, Huffman decode
//! - [jpeg-encoder](https://github.com/vstroebel/jpeg-encoder) - Huffman encode, writing

pub mod huffman;
pub mod marker;
pub mod parser;
pub mod scan;
pub mod writer;

pub use huffman::{encode_coefficient, BitReader, BitWriter, HuffmanEncoder, HuffmanLookup};
pub use marker::Marker;
pub use parser::{
    parse_jpeg, parse_quantization_tables, Component, FrameInfo, HuffmanTable, JpegSegments,
    QuantizationTable, Segment, NATURAL_TO_ZIGZAG, ZIGZAG_TO_NATURAL,
};
pub use scan::{decode_scan, encode_scan, ScanCoefficients};
pub use writer::write_jpeg;

use crate::error::Result;
use crate::{F5Decoder, F5Encoder};

/// Embed data into a JPEG image using F5 steganography.
///
/// This is the high-level API that combines JPEG transcoding with F5 embedding.
///
/// # Arguments
/// * `jpeg_data` - Original JPEG file data
/// * `message` - Data to embed
/// * `seed` - Optional permutation seed for spreading data across coefficients
///
/// # Returns
/// New JPEG file with embedded data.
///
/// # Example
/// ```ignore
/// let cover_jpeg = std::fs::read("cover.jpg")?;
/// let message = b"Secret message";
/// let stego_jpeg = embed_in_jpeg(&cover_jpeg, message, Some(b"seed"))?;
/// std::fs::write("stego.jpg", stego_jpeg)?;
/// ```
pub fn embed_in_jpeg(jpeg_data: &[u8], message: &[u8], seed: Option<&[u8]>) -> Result<Vec<u8>> {
    // Parse JPEG structure
    let segments = parse_jpeg(jpeg_data)?;

    // Decode coefficients
    let mut coefficients = decode_scan(&segments)?;

    // Embed message using F5
    let encoder = F5Encoder::new();
    encoder.embed(coefficients.as_mut_slice(), message, seed)?;

    // Re-encode coefficients
    let new_scan_data = encode_scan(&coefficients, &segments)?;

    // Write new JPEG
    Ok(write_jpeg(&segments, &new_scan_data))
}

/// Extract data from a JPEG image using F5 steganography.
///
/// This is the high-level API that combines JPEG transcoding with F5 extraction.
///
/// # Arguments
/// * `jpeg_data` - JPEG file data containing embedded message
/// * `seed` - Optional permutation seed (must match the one used for embedding)
///
/// # Returns
/// Extracted message data.
///
/// # Example
/// ```ignore
/// let stego_jpeg = std::fs::read("stego.jpg")?;
/// let message = extract_from_jpeg(&stego_jpeg, Some(b"seed"))?;
/// println!("Extracted: {:?}", message);
/// ```
pub fn extract_from_jpeg(jpeg_data: &[u8], seed: Option<&[u8]>) -> Result<Vec<u8>> {
    // Parse JPEG structure
    let segments = parse_jpeg(jpeg_data)?;

    // Decode coefficients
    let coefficients = decode_scan(&segments)?;

    // Extract message using F5
    let decoder = F5Decoder::new();
    decoder.extract(coefficients.as_slice(), seed)
}

/// Calculate the embedding capacity of a JPEG image.
///
/// Returns the maximum number of bytes that can be embedded using F5.
///
/// # Arguments
/// * `jpeg_data` - JPEG file data
///
/// # Returns
/// Maximum embedding capacity in bytes.
pub fn jpeg_capacity(jpeg_data: &[u8]) -> Result<usize> {
    // Parse and decode
    let segments = parse_jpeg(jpeg_data)?;
    let coefficients = decode_scan(&segments)?;

    // Count usable coefficients (non-zero, non-DC)
    let mut usable_count = 0;
    for block_idx in 0..coefficients.total_blocks {
        let block = coefficients.block(block_idx);
        // Skip DC (index 0), count non-zero AC coefficients
        for &coeff in &block[1..] {
            if coeff != 0 {
                usable_count += 1;
            }
        }
    }

    // F5 capacity depends on k parameter (bits per change)
    // With optimal k, capacity is approximately usable_count / (2^k - 1) bits
    // For a conservative estimate, assume k=1 (1 bit per coefficient)
    // Actual capacity is typically higher due to matrix encoding
    Ok(usable_count / 8)
}

#[cfg(test)]
mod api_tests {
    use super::*;

    #[test]
    fn test_embed_extract_roundtrip() {
        let cover = include_bytes!("../../../../resources/NoSecrets.jpg");
        let message = b"Hello, F5 steganography!";
        let seed = b"test_seed_123";

        // Parse and decode original
        let segments = parse_jpeg(cover).expect("Failed to parse cover");
        println!("Restart interval: {}", segments.restart_interval);
        let mut coefficients = decode_scan(&segments).expect("Failed to decode cover");
        let original_coeffs = coefficients.data.clone();

        println!("Original coefficient count: {}", original_coeffs.len());
        println!("Original non-zero count: {}", original_coeffs.iter().filter(|&&c| c != 0).count());

        // Embed using F5
        let encoder = crate::F5Encoder::new();
        encoder.embed(coefficients.as_mut_slice(), message, Some(seed)).expect("Failed to embed");

        // Check how many coefficients changed
        let mut dc_changed = 0;
        let mut ac_changed = 0;
        let mut shrunk = 0;
        for (i, (&orig, &new)) in original_coeffs.iter().zip(coefficients.data.iter()).enumerate() {
            if orig != new {
                let is_dc = (i % 64) == 0;
                if is_dc {
                    dc_changed += 1;
                    println!("DC coefficient {} changed: {} -> {}", i / 64, orig, new);
                } else {
                    ac_changed += 1;
                }
                if orig != 0 && new == 0 {
                    shrunk += 1;
                }
            }
        }
        println!("DC coefficients changed: {}", dc_changed);
        println!("AC coefficients changed: {}", ac_changed);
        println!("Coefficients shrunk to 0: {}", shrunk);

        // Encode to scan data
        let new_scan = encode_scan(&coefficients, &segments).expect("Failed to encode");
        println!("Original scan data: {} bytes", segments.scan_data.len());
        println!("New scan data: {} bytes", new_scan.len());

        // Decode back
        let mut segments2 = segments.clone();
        segments2.scan_data = new_scan.clone();

        // Try to decode, but catch the error to get more info
        let coefficients2 = match decode_scan(&segments2) {
            Ok(c) => c,
            Err(e) => {
                // Compare raw bytes to see where the difference is
                let min_len = segments.scan_data.len().min(new_scan.len());
                let mut first_diff = None;
                for i in 0..min_len {
                    if segments.scan_data[i] != new_scan[i] {
                        first_diff = Some(i);
                        break;
                    }
                }
                if let Some(pos) = first_diff {
                    println!(
                        "First byte difference at position {}: orig={:02X} new={:02X}",
                        pos, segments.scan_data[pos], new_scan[pos]
                    );
                } else if new_scan.len() != segments.scan_data.len() {
                    println!(
                        "No byte differences in first {} bytes, but sizes differ",
                        min_len
                    );
                }
                panic!("Failed to decode re-encoded: {:?}", e);
            }
        };

        // Verify coefficients match
        assert_eq!(coefficients.data.len(), coefficients2.data.len());
        let mismatches = coefficients.data.iter().zip(coefficients2.data.iter())
            .filter(|(&a, &b)| a != b)
            .count();
        assert_eq!(mismatches, 0, "Encode/decode should preserve coefficients");

        // Extract
        let decoder = crate::F5Decoder::new();
        let extracted = decoder.extract(coefficients2.as_slice(), Some(seed)).expect("Failed to extract");

        assert!(
            extracted.starts_with(message),
            "Extracted message should start with original: got {:?}",
            String::from_utf8_lossy(&extracted[..message.len().min(extracted.len())])
        );
    }

    #[test]
    fn test_jpeg_capacity() {
        let jpeg = include_bytes!("../../../../resources/NoSecrets.jpg");
        let capacity = jpeg_capacity(jpeg).expect("Failed to calculate capacity");

        println!("JPEG capacity: {} bytes", capacity);

        // Should have reasonable capacity for a large image
        assert!(capacity > 10000, "Large image should have significant capacity");
    }

    #[test]
    fn test_embed_without_seed() {
        let cover = include_bytes!("../../../../resources/NoSecrets.jpg");
        let message = b"No seed message";

        // Embed without seed
        let stego = embed_in_jpeg(cover, message, None).expect("Failed to embed");

        // Extract without seed
        let extracted = extract_from_jpeg(&stego, None).expect("Failed to extract");

        assert!(
            extracted.starts_with(message),
            "Should extract without seed"
        );
    }
}
