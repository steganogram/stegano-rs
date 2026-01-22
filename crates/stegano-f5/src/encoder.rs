//! F5 Encoder - embeds data into DCT coefficients.
//!
//! The encoder uses matrix encoding to embed message bits with minimal
//! coefficient modifications, and permutative straddling to spread
//! modifications uniformly.

use crate::error::{F5Error, Result};
use crate::matrix::{usize_to_bits, CheckMatrix};
use crate::permutation::Permutation;

/// Header size in bits: 4 bits for w + 28 bits for data length.
const HEADER_BITS: usize = 32;

/// Maximum message length in bytes (2^28 - 1).
const MAX_MESSAGE_LEN: usize = (1 << 28) - 1;

/// F5 Encoder for embedding data into DCT coefficients.
///
/// # Note
///
/// F5 does NOT handle encryption. Data passed to `embed()` should be:
/// - Plain raw bytes, OR
/// - Pre-encrypted bytes (encryption handled by outer layer)
#[derive(Debug, Default)]
pub struct F5Encoder {
    /// Optional fixed w parameter. If None, optimal w is calculated.
    fixed_w: Option<u8>,
}

impl F5Encoder {
    /// Create a new F5 encoder with automatic w selection.
    pub fn new() -> Self {
        F5Encoder { fixed_w: None }
    }

    /// Create a new F5 encoder with a fixed w parameter.
    ///
    /// # Arguments
    /// * `w` - Matrix encoding parameter (1-9)
    pub fn with_w(w: u8) -> Self {
        assert!(w >= 1 && w <= 9, "w must be between 1 and 9");
        F5Encoder { fixed_w: Some(w) }
    }

    /// Embed message into DCT coefficients.
    ///
    /// # Arguments
    /// * `coefficients` - Mutable slice of quantized DCT coefficients
    /// * `message` - Raw bytes to embed (plain or pre-encrypted)
    /// * `permutation_seed` - Optional seed for coefficient shuffling (None = no shuffle)
    ///
    /// # Returns
    /// * `Ok(())` on success (coefficients modified in-place)
    /// * `Err(F5Error::CapacityExceeded)` if message too large
    pub fn embed(
        &self,
        coefficients: &mut [i16],
        message: &[u8],
        permutation_seed: Option<&[u8]>,
    ) -> Result<()> {
        if message.len() > MAX_MESSAGE_LEN {
            return Err(F5Error::InvalidParameter {
                param: "message_len",
                value: message.len().to_string(),
                reason: format!("exceeds maximum of {} bytes", MAX_MESSAGE_LEN),
            });
        }

        // Count usable coefficients (non-zero AC)
        let usable_count = count_usable_coefficients(coefficients);

        // Determine w parameter
        let message_bits = message.len() * 8 + HEADER_BITS;
        let w = self
            .fixed_w
            .unwrap_or_else(|| CheckMatrix::optimal_w(usable_count, message_bits));

        // Check capacity
        let capacity = self.capacity_with_w(coefficients, w);
        if message.len() > capacity {
            return Err(F5Error::CapacityExceeded {
                required: message.len(),
                available: capacity,
            });
        }

        // Create permutation
        let permutation = match permutation_seed {
            Some(seed) => Permutation::from_seed(seed, coefficients.len()),
            None => Permutation::identity(coefficients.len()),
        };

        // Pre-collect usable coefficient indices in permuted order
        // This avoids borrow conflicts during embedding
        let usable_indices: Vec<usize> = (0..permutation.len())
            .map(|i| permutation.unshuffled(i))
            .filter(|&idx| is_usable(coefficients[idx], idx))
            .collect();

        let mut coeff_pos = 0; // Position in usable_indices

        // === PHASE 1: Embed header using w=1 (direct LSB) ===
        // Header: 4 bits w, 28 bits message length = 32 bits total
        let mut header_bits = Vec::with_capacity(HEADER_BITS);
        header_bits.extend(usize_to_bits(w as usize, 4));
        header_bits.extend(usize_to_bits(message.len(), 28));

        // Embed header bits directly into coefficient LSBs (no matrix encoding)
        for bit in header_bits {
            loop {
                if coeff_pos >= usable_indices.len() {
                    return Err(F5Error::CapacityExceeded {
                        required: message.len(),
                        available: 0,
                    });
                }

                let idx = usable_indices[coeff_pos];
                coeff_pos += 1;

                // Skip coefficients that shrunk to 0
                if coefficients[idx] == 0 {
                    continue;
                }

                // Set LSB to the bit value
                let current_lsb = (coefficients[idx].abs() & 1) == 1;
                if current_lsb != bit {
                    // Need to change LSB - decrement absolute value
                    if coefficients[idx] > 0 {
                        coefficients[idx] -= 1;
                    } else {
                        coefficients[idx] += 1;
                    }

                    // Check for shrinkage
                    if coefficients[idx] == 0 {
                        // Shrinkage - retry with next coefficient
                        continue;
                    }
                }
                break;
            }
        }

        // === PHASE 2: Embed message using matrix encoding with parameter w ===
        let matrix = CheckMatrix::new(w);
        let n = matrix.n();

        // Build message bit stream (LSB first per byte)
        let mut message_bits = Vec::with_capacity(message.len() * 8);
        for &byte in message {
            for i in 0..8 {
                message_bits.push((byte >> i) & 1 == 1);
            }
        }

        let mut bit_index = 0;

        while bit_index < message_bits.len() {
            // Get w bits to embed
            let bits_remaining = message_bits.len() - bit_index;
            let bits_to_embed = bits_remaining.min(w as usize);

            // Pad with zeros if needed (last group)
            let mut target_bits = vec![false; w as usize];
            for i in 0..bits_to_embed {
                target_bits[i] = message_bits[bit_index + i];
            }
            let target = bits_to_usize(&target_bits);

            // Try to embed this group (with shrinkage handling)
            loop {
                // Collect n usable coefficients (skip zeros due to shrinkage)
                let mut group = Vec::with_capacity(n);
                let start_pos = coeff_pos;

                while group.len() < n && coeff_pos < usable_indices.len() {
                    let idx = usable_indices[coeff_pos];
                    coeff_pos += 1;

                    // Check if still usable (might have shrunk to 0)
                    if coefficients[idx] != 0 {
                        group.push(idx);
                    }
                }

                if group.len() < n {
                    return Err(F5Error::CapacityExceeded {
                        required: message.len(),
                        available: self.capacity(coefficients),
                    });
                }

                // Compute current hash
                let current_hash = compute_hash(&matrix, &group, coefficients);

                // Find which coefficient to modify
                let modification = matrix.find_modification(current_hash, target);

                if modification == 0 {
                    // No modification needed
                    break;
                }

                // Modify the coefficient (decrement absolute value)
                let coeff_idx = group[modification - 1];

                if coefficients[coeff_idx] > 0 {
                    coefficients[coeff_idx] -= 1;
                } else {
                    coefficients[coeff_idx] += 1;
                }

                // Check for shrinkage (coefficient became 0)
                if coefficients[coeff_idx] == 0 {
                    // Shrinkage occurred - retry embedding the same bits
                    // Reset position to re-collect coefficients (excluding the shrunk one)
                    coeff_pos = start_pos;
                    continue;
                }

                // Successfully embedded
                break;
            }

            bit_index += w as usize;
        }

        Ok(())
    }

    /// Calculate embedding capacity in bytes.
    ///
    /// # Arguments
    /// * `coefficients` - Slice of quantized DCT coefficients
    ///
    /// # Returns
    /// Maximum number of message bytes that can be embedded.
    pub fn capacity(&self, coefficients: &[i16]) -> usize {
        let usable = count_usable_coefficients(coefficients);
        if usable == 0 {
            return 0;
        }

        // Use w=1 for maximum capacity estimate (least efficient but most capacity)
        let w = self.fixed_w.unwrap_or(1);
        self.capacity_with_w(coefficients, w)
    }

    /// Calculate capacity for a specific w value.
    fn capacity_with_w(&self, coefficients: &[i16], w: u8) -> usize {
        let usable = count_usable_coefficients(coefficients);
        let shrinkage_estimate = count_shrinkable(coefficients);

        // Estimate effective usable coefficients after shrinkage
        // Approximately 51% of |1| coefficients will shrink
        let effective_usable = usable.saturating_sub((shrinkage_estimate * 51) / 100);

        if effective_usable == 0 {
            return 0;
        }

        let n = (1usize << w) - 1;
        let groups = effective_usable / n;
        let total_bits = groups * (w as usize);

        // Subtract header bits
        let message_bits = total_bits.saturating_sub(HEADER_BITS);
        message_bits / 8
    }
}

/// Check if a coefficient is usable for embedding.
#[inline]
fn is_usable(coeff: i16, index: usize) -> bool {
    coeff != 0 && !is_dc_coefficient(index)
}

/// Check if an index is a DC coefficient (first of each 8x8 block).
#[inline]
fn is_dc_coefficient(index: usize) -> bool {
    index % 64 == 0
}

/// Count usable (non-zero AC) coefficients.
fn count_usable_coefficients(coefficients: &[i16]) -> usize {
    coefficients
        .iter()
        .enumerate()
        .filter(|(i, &c)| is_usable(c, *i))
        .count()
}

/// Count coefficients that might shrink (|value| == 1).
fn count_shrinkable(coefficients: &[i16]) -> usize {
    coefficients
        .iter()
        .enumerate()
        .filter(|(i, &c)| !is_dc_coefficient(*i) && c.abs() == 1)
        .count()
}

/// Compute hash of a coefficient group.
fn compute_hash(matrix: &CheckMatrix, group: &[usize], coefficients: &[i16]) -> usize {
    let bits: Vec<bool> = group
        .iter()
        .map(|&idx| (coefficients[idx].abs() & 1) == 1)
        .collect();
    bits_to_usize(&matrix.multiply(&bits))
}

/// Convert bits to usize (MSB first).
fn bits_to_usize(bits: &[bool]) -> usize {
    bits.iter()
        .fold(0usize, |acc, &b| (acc << 1) | (b as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_coefficients(block_count: usize) -> Vec<i16> {
        let mut rng = fastrand::Rng::with_seed(12345);
        let mut coeffs = Vec::with_capacity(block_count * 64);

        for _ in 0..block_count {
            // DC coefficient - larger value
            coeffs.push(rng.i16(-500..500));

            // AC coefficients - mostly small, many zeros
            for _ in 1..64 {
                let val = match rng.usize(0..10) {
                    0..=5 => 0,               // 60% zeros
                    6..=7 => rng.i16(-2..=2), // 20% small
                    8 => rng.i16(-10..=10),   // 10% medium
                    _ => rng.i16(-50..=50),   // 10% larger
                };
                coeffs.push(val);
            }
        }
        coeffs
    }

    #[test]
    fn test_count_usable_coefficients() {
        let coeffs = generate_test_coefficients(10);
        let usable = count_usable_coefficients(&coeffs);

        // Should have some usable coefficients but not all
        assert!(usable > 0);
        assert!(usable < coeffs.len());
    }

    #[test]
    fn test_is_dc_coefficient() {
        assert!(is_dc_coefficient(0));
        assert!(is_dc_coefficient(64));
        assert!(is_dc_coefficient(128));
        assert!(!is_dc_coefficient(1));
        assert!(!is_dc_coefficient(63));
        assert!(!is_dc_coefficient(65));
    }

    #[test]
    fn test_capacity_calculation() {
        let coeffs = generate_test_coefficients(100);
        let encoder = F5Encoder::new();
        let capacity = encoder.capacity(&coeffs);

        // Should have positive capacity
        assert!(capacity > 0, "Expected positive capacity");
    }

    #[test]
    fn test_embed_basic() {
        let mut coeffs = generate_test_coefficients(100);
        let original = coeffs.clone();
        let message = b"Hello";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, None).unwrap();

        // Coefficients should be modified
        assert_ne!(coeffs, original);
    }

    #[test]
    fn test_embed_with_permutation() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"Test message";
        let seed = b"test_seed";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, Some(seed)).unwrap();

        // Should complete without error
    }

    #[test]
    fn test_embed_empty_message() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, None).unwrap();
    }

    #[test]
    fn test_capacity_exceeded() {
        let mut coeffs = generate_test_coefficients(5); // Small capacity
        let message = vec![0u8; 10000]; // Large message

        let encoder = F5Encoder::new();
        let result = encoder.embed(&mut coeffs, &message, None);

        assert!(matches!(result, Err(F5Error::CapacityExceeded { .. })));
    }

    #[test]
    fn test_dc_coefficients_unchanged() {
        let mut coeffs = generate_test_coefficients(100);
        let dc_before: Vec<i16> = coeffs.iter().step_by(64).cloned().collect();

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, b"test message", None).unwrap();

        let dc_after: Vec<i16> = coeffs.iter().step_by(64).cloned().collect();
        assert_eq!(
            dc_before, dc_after,
            "DC coefficients should not be modified"
        );
    }

    #[test]
    fn test_fixed_w() {
        let mut coeffs = generate_test_coefficients(100);

        let encoder = F5Encoder::with_w(3);
        encoder.embed(&mut coeffs, b"test", None).unwrap();
    }
}
