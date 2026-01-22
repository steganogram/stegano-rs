//! Check Matrix for F5 matrix encoding.
//!
//! The check matrix H_w is the core of F5's (1, n, k) matrix encoding scheme.
//! It enables embedding `w` message bits into `n = 2^w - 1` coefficients with
//! at most one coefficient modification.

/// F5 Check Matrix for matrix encoding.
///
/// The matrix has dimensions `w` rows × `n` columns where `n = 2^w - 1`.
/// Each column `j` (1-indexed) contains the binary representation of `j`.
///
/// # Example
///
/// For `w = 2`, the matrix is:
/// ```text
/// H_2 = | 0 1 1 |
///       | 1 0 1 |
/// ```
/// Columns represent binary: 01, 10, 11 (decimal: 1, 2, 3)
#[derive(Debug, Clone)]
pub struct CheckMatrix {
    /// The encoding parameter (number of message bits per group).
    w: u8,
    /// Number of coefficients per group: n = 2^w - 1.
    n: usize,
}

impl CheckMatrix {
    /// Create a new check matrix for parameter `w`.
    ///
    /// # Arguments
    /// * `w` - Number of message bits to embed per group (1-15, practically 1-9)
    ///
    /// # Panics
    /// Panics if `w` is 0 or greater than 15.
    pub fn new(w: u8) -> Self {
        assert!(w > 0 && w <= 15, "w must be between 1 and 15, got {}", w);
        let n = (1usize << w) - 1; // 2^w - 1
        CheckMatrix { w, n }
    }

    /// Get the encoding parameter `w`.
    #[inline]
    pub fn w(&self) -> u8 {
        self.w
    }

    /// Get the number of coefficients per group `n = 2^w - 1`.
    #[inline]
    pub fn n(&self) -> usize {
        self.n
    }

    /// Get matrix element H_w[row, col].
    ///
    /// # Arguments
    /// * `row` - Row index (0-based, 0 to w-1)
    /// * `col` - Column index (0-based, 0 to n-1)
    ///
    /// # Returns
    /// The bit value at position (row, col).
    ///
    /// # Formula
    /// H_w[i, j] = bit(j+1, w-i) where j+1 is the 1-indexed column number.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> bool {
        debug_assert!(row < self.w as usize, "row {} >= w {}", row, self.w);
        debug_assert!(col < self.n, "col {} >= n {}", col, self.n);

        // Column j (0-indexed) represents binary of (j+1)
        // Row i (0-indexed) represents bit position (w - i - 1) from LSB
        let column_value = col + 1; // 1-indexed column number
        let bit_position = self.w as usize - row - 1; // MSB first
        (column_value >> bit_position) & 1 == 1
    }

    /// Multiply H_w × vector (mod 2).
    ///
    /// # Arguments
    /// * `bits` - Vector of n bits (coefficient LSBs)
    ///
    /// # Returns
    /// Result vector of w bits.
    ///
    /// # Panics
    /// Panics if `bits.len() != n`.
    pub fn multiply(&self, bits: &[bool]) -> Vec<bool> {
        assert_eq!(
            bits.len(),
            self.n,
            "bits length {} != n {}",
            bits.len(),
            self.n
        );

        let mut result = vec![false; self.w as usize];

        for row in 0..self.w as usize {
            let mut sum = false;
            for col in 0..self.n {
                if bits[col] && self.get(row, col) {
                    sum = !sum; // XOR
                }
            }
            result[row] = sum;
        }

        result
    }

    /// Compute the hash of coefficient LSBs: H_w × C (mod 2).
    ///
    /// This is a convenience method that extracts LSBs and multiplies.
    ///
    /// # Arguments
    /// * `coefficients` - Slice of n coefficients
    ///
    /// # Returns
    /// Hash as a usize (interpreting the w-bit result as a number).
    pub fn hash_coefficients(&self, coefficients: &[i16]) -> usize {
        assert_eq!(coefficients.len(), self.n);

        let bits: Vec<bool> = coefficients.iter().map(|&c| (c.abs() & 1) == 1).collect();

        let hash_bits = self.multiply(&bits);
        bits_to_usize(&hash_bits)
    }

    /// Find which coefficient to modify to achieve target hash.
    ///
    /// Given current hash `s` and target message bits `m`, computes
    /// the coefficient index to flip (1-indexed), or 0 if no change needed.
    ///
    /// # Arguments
    /// * `current_hash` - Current hash value (H_w × C)
    /// * `target` - Target message bits to embed
    ///
    /// # Returns
    /// - `0` if no modification needed (current_hash == target)
    /// - `1..=n` indicating which coefficient to modify
    pub fn find_modification(&self, current_hash: usize, target: usize) -> usize {
        current_hash ^ target // XOR gives the column index (or 0 if equal)
    }

    /// Determine optimal `w` given capacity and message length.
    ///
    /// Larger `w` is more efficient (fewer modifications) but requires
    /// more coefficients per group. This finds the largest `w` where
    /// the message still fits.
    ///
    /// # Arguments
    /// * `usable_coefficients` - Number of usable (non-zero AC) coefficients
    /// * `message_bits` - Number of message bits to embed
    ///
    /// # Returns
    /// Optimal `w` parameter (1-9).
    pub fn optimal_w(usable_coefficients: usize, message_bits: usize) -> u8 {
        if message_bits == 0 || usable_coefficients == 0 {
            return 1;
        }

        // Try w from high to low, find largest that fits
        for w in (1u8..=9).rev() {
            let n = (1usize << w) - 1; // coefficients per group
            let groups_available = usable_coefficients / n;
            let bits_embeddable = groups_available * (w as usize);

            if bits_embeddable >= message_bits {
                return w;
            }
        }

        // Fall back to w=1 (least efficient but maximum capacity)
        1
    }
}

/// Convert a slice of bits to a usize (MSB first).
#[inline]
fn bits_to_usize(bits: &[bool]) -> usize {
    bits.iter()
        .fold(0usize, |acc, &b| (acc << 1) | (b as usize))
}

/// Convert a usize to a vector of bits (MSB first).
#[inline]
pub(crate) fn usize_to_bits(value: usize, num_bits: usize) -> Vec<bool> {
    (0..num_bits).rev().map(|i| (value >> i) & 1 == 1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_construction_w2() {
        // H_2 should be: | 0 1 1 |
        //                | 1 0 1 |
        let m = CheckMatrix::new(2);
        assert_eq!(m.n(), 3); // 2^2 - 1

        // Column 1 (index 0) = binary 01 → [0, 1]
        assert_eq!(m.get(0, 0), false); // bit 2 of 1 = 0
        assert_eq!(m.get(1, 0), true); // bit 1 of 1 = 1

        // Column 2 (index 1) = binary 10 → [1, 0]
        assert_eq!(m.get(0, 1), true); // bit 2 of 2 = 1
        assert_eq!(m.get(1, 1), false); // bit 1 of 2 = 0

        // Column 3 (index 2) = binary 11 → [1, 1]
        assert_eq!(m.get(0, 2), true); // bit 2 of 3 = 1
        assert_eq!(m.get(1, 2), true); // bit 1 of 3 = 1
    }

    #[test]
    fn test_matrix_construction_w3() {
        let m = CheckMatrix::new(3);
        assert_eq!(m.n(), 7); // 2^3 - 1

        // Column 5 (index 4) = binary 101 → [1, 0, 1]
        assert_eq!(m.get(0, 4), true); // bit 3 of 5 = 1
        assert_eq!(m.get(1, 4), false); // bit 2 of 5 = 0
        assert_eq!(m.get(2, 4), true); // bit 1 of 5 = 1

        // Column 7 (index 6) = binary 111 → [1, 1, 1]
        assert_eq!(m.get(0, 6), true);
        assert_eq!(m.get(1, 6), true);
        assert_eq!(m.get(2, 6), true);
    }

    #[test]
    fn test_matrix_multiply() {
        let m = CheckMatrix::new(2);

        // H_2 × [1, 0, 1] (columns 1 and 3)
        // Column 1 = [0, 1], Column 3 = [1, 1]
        // XOR: [0^1, 1^1] = [1, 0]
        let bits = vec![true, false, true];
        let result = m.multiply(&bits);
        assert_eq!(result, vec![true, false]);
    }

    #[test]
    fn test_matrix_multiply_all_zeros() {
        let m = CheckMatrix::new(2);
        let bits = vec![false, false, false];
        let result = m.multiply(&bits);
        assert_eq!(result, vec![false, false]);
    }

    #[test]
    fn test_matrix_multiply_all_ones() {
        let m = CheckMatrix::new(2);
        // H_2 × [1, 1, 1] = XOR of all columns
        // [0, 1, 1] XOR [1, 0, 1] XOR [1, 1, 1] = [0^1^1, 1^0^1] = [0, 0]
        let bits = vec![true, true, true];
        let result = m.multiply(&bits);
        assert_eq!(result, vec![false, false]);
    }

    #[test]
    fn test_hash_coefficients() {
        let m = CheckMatrix::new(2);

        // Coefficients with LSBs [1, 0, 1]
        let coeffs: Vec<i16> = vec![3, 4, 5]; // LSBs: 1, 0, 1
        let hash = m.hash_coefficients(&coeffs);

        // Expected: H_2 × [1, 0, 1] = [1, 0] = 2
        assert_eq!(hash, 2);
    }

    #[test]
    fn test_find_modification() {
        let m = CheckMatrix::new(2);

        // If current hash is 2 and target is 3, need to flip column (2 XOR 3) = 1
        assert_eq!(m.find_modification(2, 3), 1);

        // If current hash equals target, no modification needed
        assert_eq!(m.find_modification(2, 2), 0);

        // If current hash is 0 and target is 3, need to flip column 3
        assert_eq!(m.find_modification(0, 3), 3);
    }

    #[test]
    fn test_optimal_w_selection() {
        // High capacity, small message → high w (more efficient)
        let w = CheckMatrix::optimal_w(10000, 100);
        assert!(w >= 3, "Expected w >= 3 for high capacity, got {}", w);

        // Low capacity, large message → low w (need more capacity)
        let w = CheckMatrix::optimal_w(100, 80);
        assert!(w <= 2, "Expected w <= 2 for low capacity, got {}", w);

        // Edge case: zero message
        assert_eq!(CheckMatrix::optimal_w(1000, 0), 1);

        // Edge case: zero capacity
        assert_eq!(CheckMatrix::optimal_w(0, 100), 1);
    }

    #[test]
    fn test_bits_conversion() {
        // Test usize_to_bits
        let bits = usize_to_bits(5, 3); // 101
        assert_eq!(bits, vec![true, false, true]);

        let bits = usize_to_bits(0, 4);
        assert_eq!(bits, vec![false, false, false, false]);

        // Test bits_to_usize
        assert_eq!(bits_to_usize(&[true, false, true]), 5);
        assert_eq!(bits_to_usize(&[false, false]), 0);
    }

    #[test]
    #[should_panic(expected = "w must be between 1 and 15")]
    fn test_invalid_w_zero() {
        CheckMatrix::new(0);
    }

    #[test]
    #[should_panic(expected = "w must be between 1 and 15")]
    fn test_invalid_w_too_large() {
        CheckMatrix::new(16);
    }
}
