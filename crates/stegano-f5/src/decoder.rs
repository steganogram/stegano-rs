//! F5 Decoder - extracts data from DCT coefficients.
//!
//! The decoder reverses the embedding process, using the same permutation
//! and matrix encoding to recover the original message.

use crate::error::{F5Error, Result};
use crate::matrix::CheckMatrix;
use crate::permutation::Permutation;

/// Header size in bits: 4 bits for w + 28 bits for data length.
const HEADER_BITS: usize = 32;

/// Maximum valid w value.
const MAX_W: u8 = 9;

/// F5 Decoder for extracting data from DCT coefficients.
///
/// # Note
///
/// F5 does NOT handle decryption. Data returned from `extract()` is:
/// - Plain raw bytes, OR
/// - Encrypted bytes (decryption handled by outer layer)
#[derive(Debug, Default)]
pub struct F5Decoder;

impl F5Decoder {
    /// Create a new F5 decoder.
    pub fn new() -> Self {
        F5Decoder
    }

    /// Extract message from DCT coefficients.
    ///
    /// # Arguments
    /// * `coefficients` - Slice of quantized DCT coefficients
    /// * `permutation_seed` - Optional seed for coefficient shuffling (must match embed)
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Extracted raw bytes
    /// * `Err(F5Error::NoDataFound)` if no valid F5 header found
    pub fn extract(
        &self,
        coefficients: &[i16],
        permutation_seed: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        // Create permutation
        let permutation = match permutation_seed {
            Some(seed) => Permutation::from_seed(seed, coefficients.len()),
            None => Permutation::identity(coefficients.len()),
        };

        // Create iterator over usable coefficients
        let mut coeff_iter = UsableCoefficients::new(coefficients, &permutation);

        // First, extract header using w=1 (1 bit per coefficient)
        // Header: 4 bits w, 28 bits message length
        let header_bits = self.extract_bits(&mut coeff_iter, HEADER_BITS, coefficients, 1)?;

        // Parse header
        let w = bits_to_usize(&header_bits[0..4]) as u8;
        let message_len = bits_to_usize(&header_bits[4..32]);

        // Validate w
        if w == 0 || w > MAX_W {
            return Err(F5Error::NoDataFound {
                reason: format!("invalid w parameter: {}", w),
            });
        }

        // Validate message length
        if message_len > coefficients.len() {
            return Err(F5Error::NoDataFound {
                reason: format!("message length {} exceeds coefficient count", message_len),
            });
        }

        // Extract message bits using the w from header
        let message_bits_count = message_len * 8;

        // Reset iterator and skip header
        let mut coeff_iter = UsableCoefficients::new(coefficients, &permutation);

        // Skip header (32 bits at w=1 means 32 coefficients)
        for _ in 0..HEADER_BITS {
            if coeff_iter.next().is_none() {
                return Err(F5Error::NoDataFound {
                    reason: "not enough coefficients for header".to_string(),
                });
            }
        }

        // Extract message using matrix encoding with w
        let matrix = CheckMatrix::new(w);
        let n = matrix.n();
        let mut message_bits = Vec::with_capacity(message_bits_count);

        while message_bits.len() < message_bits_count {
            // Collect n coefficients
            let mut group = Vec::with_capacity(n);
            for _ in 0..n {
                match coeff_iter.next() {
                    Some(idx) => group.push(idx),
                    None => {
                        return Err(F5Error::NoDataFound {
                            reason: "not enough coefficients for message".to_string(),
                        });
                    }
                }
            }

            // Extract bits using matrix multiplication
            let lsbs: Vec<bool> = group
                .iter()
                .map(|&idx| (coefficients[idx].abs() & 1) == 1)
                .collect();

            let extracted = matrix.multiply(&lsbs);

            // Take only the bits we need
            let bits_remaining = message_bits_count - message_bits.len();
            let bits_to_take = bits_remaining.min(w as usize);
            message_bits.extend(&extracted[0..bits_to_take]);
        }

        // Convert bits to bytes (LSB first per byte, matching encoder)
        let mut message = Vec::with_capacity(message_len);
        for chunk in message_bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << i;
                }
            }
            message.push(byte);
        }

        Ok(message)
    }

    /// Extract bits using w=1 (for header).
    fn extract_bits(
        &self,
        coeff_iter: &mut UsableCoefficients,
        count: usize,
        coefficients: &[i16],
        _w: u8,
    ) -> Result<Vec<bool>> {
        let mut bits = Vec::with_capacity(count);

        for _ in 0..count {
            match coeff_iter.next() {
                Some(idx) => {
                    let bit = (coefficients[idx].abs() & 1) == 1;
                    bits.push(bit);
                }
                None => {
                    return Err(F5Error::NoDataFound {
                        reason: "not enough coefficients".to_string(),
                    });
                }
            }
        }

        Ok(bits)
    }
}

/// Iterator over usable (non-zero AC) coefficient indices in permuted order.
struct UsableCoefficients<'a> {
    coefficients: &'a [i16],
    permutation: &'a Permutation,
    current: usize,
}

impl<'a> UsableCoefficients<'a> {
    fn new(coefficients: &'a [i16], permutation: &'a Permutation) -> Self {
        UsableCoefficients {
            coefficients,
            permutation,
            current: 0,
        }
    }
}

impl<'a> Iterator for UsableCoefficients<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.permutation.len() {
            let idx = self.permutation.unshuffled(self.current);
            self.current += 1;

            if is_usable(self.coefficients[idx], idx) {
                return Some(idx);
            }
        }
        None
    }
}

/// Check if a coefficient is usable for embedding/extraction.
#[inline]
fn is_usable(coeff: i16, index: usize) -> bool {
    coeff != 0 && !is_dc_coefficient(index)
}

/// Check if an index is a DC coefficient (first of each 8x8 block).
#[inline]
fn is_dc_coefficient(index: usize) -> bool {
    index.is_multiple_of(64)
}

/// Convert bits to usize (MSB first).
fn bits_to_usize(bits: &[bool]) -> usize {
    bits.iter()
        .fold(0usize, |acc, &b| (acc << 1) | (b as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::F5Encoder;

    fn generate_test_coefficients(block_count: usize) -> Vec<i16> {
        let mut rng = fastrand::Rng::with_seed(12345);
        let mut coeffs = Vec::with_capacity(block_count * 64);

        for _ in 0..block_count {
            // DC coefficient - larger value
            coeffs.push(rng.i16(-500..500));

            // AC coefficients - mostly small, many zeros
            for _ in 1..64 {
                let val = match rng.usize(0..10) {
                    0..=5 => 0,
                    6..=7 => rng.i16(-2..=2),
                    8 => rng.i16(-10..=10),
                    _ => rng.i16(-50..=50),
                };
                coeffs.push(val);
            }
        }
        coeffs
    }

    #[test]
    fn test_roundtrip_simple() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"Hello World";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, None).unwrap();

        let decoder = F5Decoder::new();
        let extracted = decoder.extract(&coeffs, None).unwrap();

        assert_eq!(extracted, message);
    }

    #[test]
    fn test_roundtrip_with_permutation() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"Secret message with permutation";
        let seed = b"my_secret_seed";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, Some(seed)).unwrap();

        let decoder = F5Decoder::new();
        let extracted = decoder.extract(&coeffs, Some(seed)).unwrap();

        assert_eq!(extracted, message);
    }

    #[test]
    fn test_roundtrip_empty_message() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, None).unwrap();

        let decoder = F5Decoder::new();
        let extracted = decoder.extract(&coeffs, None).unwrap();

        assert_eq!(extracted, message);
    }

    #[test]
    fn test_roundtrip_various_sizes() {
        for size in [1, 10, 50, 100, 200] {
            let mut coeffs = generate_test_coefficients(200);
            let message: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

            let encoder = F5Encoder::new();
            if encoder.embed(&mut coeffs, &message, None).is_ok() {
                let decoder = F5Decoder::new();
                let extracted = decoder.extract(&coeffs, None).unwrap();
                assert_eq!(extracted, message, "Failed for size {}", size);
            }
        }
    }

    #[test]
    fn test_wrong_seed_fails() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"Secret";
        let seed = b"correct_seed";
        let wrong_seed = b"wrong_seed";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, Some(seed)).unwrap();

        let decoder = F5Decoder::new();
        let result = decoder.extract(&coeffs, Some(wrong_seed));

        // Should either fail or return different data
        match result {
            Err(_) => { /* Expected: extraction failed */ }
            Ok(extracted) => assert_ne!(extracted, message, "Should not match with wrong seed"),
        }
    }

    #[test]
    fn test_binary_data() {
        let mut coeffs = generate_test_coefficients(100);
        // All byte values
        let message: Vec<u8> = (0..=127).collect();

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, &message, None).unwrap();

        let decoder = F5Decoder::new();
        let extracted = decoder.extract(&coeffs, None).unwrap();

        assert_eq!(extracted, message);
    }
}
