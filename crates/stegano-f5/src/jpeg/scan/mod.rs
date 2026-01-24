//! JPEG scan data encoding and decoding.
//!
//! Encodes and decodes entropy-coded scan data to/from DCT coefficients
//! without performing dequantization or IDCT. This is the key module for
//! F5 coefficient access and modification.
//!
//! # Module Structure
//!
//! - `baseline` - Baseline (sequential) JPEG encode/decode
//! - `progressive` - Progressive JPEG encode/decode (future)
//! - `utils` - Shared utilities (future, if needed)

mod baseline;

use super::parser::{JpegSegments, ZIGZAG_TO_NATURAL};
use crate::error::{F5Error, Result};

pub use baseline::{decode_scan_baseline, encode_scan_baseline};

/// Decoded scan coefficients.
#[derive(Debug, Clone)]
pub struct ScanCoefficients {
    /// All DCT coefficients in scan order.
    /// Organized as blocks of 64 i16 values in zigzag order.
    /// Block order follows JPEG interleaving rules.
    pub data: Vec<i16>,

    /// Number of 8x8 blocks per component.
    pub blocks_per_component: Vec<usize>,

    /// Total number of blocks.
    pub total_blocks: usize,

    /// Image dimensions.
    pub width: u16,
    pub height: u16,
}

impl ScanCoefficients {
    /// Get coefficients as a flat slice (for F5 algorithm).
    #[inline]
    pub fn as_slice(&self) -> &[i16] {
        &self.data
    }

    /// Get coefficients as a mutable flat slice (for F5 embedding).
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [i16] {
        &mut self.data
    }

    /// Get a specific block's coefficients.
    #[inline]
    pub fn block(&self, index: usize) -> &[i16] {
        let start = index * 64;
        &self.data[start..start + 64]
    }

    /// Get a specific block's coefficients mutably.
    #[inline]
    pub fn block_mut(&mut self, index: usize) -> &mut [i16] {
        let start = index * 64;
        &mut self.data[start..start + 64]
    }

    /// Convert coefficients from zigzag to natural order.
    pub fn to_natural_order(&self) -> Vec<i16> {
        let mut result = vec![0i16; self.data.len()];
        for block_idx in 0..self.total_blocks {
            let src_start = block_idx * 64;
            let dst_start = block_idx * 64;
            for i in 0..64 {
                result[dst_start + ZIGZAG_TO_NATURAL[i]] = self.data[src_start + i];
            }
        }
        result
    }
}

/// Decode scan data from a parsed JPEG.
///
/// Extracts all DCT coefficients from the entropy-coded scan data.
/// Automatically dispatches to baseline or progressive decoder based on JPEG type.
///
/// # Arguments
/// * `segments` - Parsed JPEG segments containing Huffman tables and scan data
///
/// # Returns
/// Decoded DCT coefficients in scan order.
pub fn decode_scan(segments: &JpegSegments) -> Result<ScanCoefficients> {
    let frame = segments.frame.as_ref().ok_or_else(|| F5Error::InvalidCoefficients {
        reason: "missing frame info (SOF)".to_string(),
    })?;

    if frame.is_progressive() {
        // TODO: progressive::decode_scan_progressive(segments)
        return Err(F5Error::InvalidCoefficients {
            reason: "progressive JPEGs not yet supported".to_string(),
        });
    }

    baseline::decode_scan_baseline(segments)
}

/// Encode DCT coefficients back to scan data.
///
/// Re-encodes coefficients using the same Huffman tables from the original JPEG.
/// Automatically dispatches to baseline or progressive encoder based on JPEG type.
///
/// # Arguments
/// * `coefficients` - DCT coefficients to encode
/// * `segments` - Parsed JPEG segments containing Huffman tables
///
/// # Returns
/// Entropy-coded scan data with byte stuffing applied.
/// For baseline: single Vec<u8>
/// For progressive: Vec<Vec<u8>> with one buffer per scan (wrapped in outer Vec for uniform API)
pub fn encode_scan(coefficients: &ScanCoefficients, segments: &JpegSegments) -> Result<Vec<u8>> {
    let frame = segments.frame.as_ref().ok_or_else(|| F5Error::InvalidCoefficients {
        reason: "missing frame info (SOF)".to_string(),
    })?;

    if frame.is_progressive() {
        // TODO: progressive::encode_scan_progressive(coefficients, segments)
        return Err(F5Error::InvalidCoefficients {
            reason: "progressive JPEGs not yet supported".to_string(),
        });
    }

    baseline::encode_scan_baseline(coefficients, segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpeg::parse_jpeg;

    #[test]
    fn test_progressive_jpeg_rejected() {
        let jpeg_data = include_bytes!("../../../../../resources/f5/tryout.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        let result = decode_scan(&segments);
        assert!(
            result.is_err(),
            "Progressive JPEG should be rejected"
        );

        if let Err(e) = result {
            let msg = format!("{:?}", e);
            assert!(msg.contains("progressive"), "Error should mention progressive");
        }
    }
}
