//! Permutation for coefficient shuffling.
//!
//! Permutative straddling spreads embedded data uniformly across the coefficient
//! array, making it harder to detect and improving security.

use fastrand::Rng;

/// Pseudo-random permutation for coefficient shuffling.
///
/// The permutation is deterministic given the same seed, allowing the decoder
/// to reconstruct the same ordering used during embedding.
///
/// # Note
///
/// The permutation seed controls WHERE data is hidden (coefficient ordering).
/// Data encryption is handled externally by the caller (WHAT is hidden).
#[derive(Debug, Clone)]
pub struct Permutation {
    /// Shuffled indices: indices[i] = where original index i maps to.
    indices: Vec<usize>,
    /// Inverse mapping: inverse[i] = original index that maps to i.
    inverse: Vec<usize>,
}

impl Permutation {
    /// Create a permutation from seed bytes.
    ///
    /// # Arguments
    /// * `seed` - Seed bytes for deterministic shuffling
    /// * `length` - Number of elements to permute
    ///
    /// # Returns
    /// A new permutation that shuffles indices 0..length.
    pub fn from_seed(seed: &[u8], length: usize) -> Self {
        let seed_u64 = hash_seed(seed);
        let mut rng = Rng::with_seed(seed_u64);

        // Initialize identity permutation
        let mut indices: Vec<usize> = (0..length).collect();

        // Fisher-Yates shuffle
        for i in (1..length).rev() {
            let j = rng.usize(0..=i);
            indices.swap(i, j);
        }

        // Build inverse mapping
        let mut inverse = vec![0usize; length];
        for (original, &shuffled) in indices.iter().enumerate() {
            inverse[shuffled] = original;
        }

        Permutation { indices, inverse }
    }

    /// Create an identity permutation (no shuffling).
    ///
    /// Useful for testing or when permutation is disabled.
    pub fn identity(length: usize) -> Self {
        let indices: Vec<usize> = (0..length).collect();
        let inverse = indices.clone();
        Permutation { indices, inverse }
    }

    /// Get the shuffled index for an original index.
    ///
    /// # Arguments
    /// * `original` - Original (unshuffled) index
    ///
    /// # Returns
    /// The shuffled position.
    #[inline]
    pub fn shuffled(&self, original: usize) -> usize {
        self.indices[original]
    }

    /// Get the original index for a shuffled index.
    ///
    /// # Arguments
    /// * `shuffled` - Shuffled position
    ///
    /// # Returns
    /// The original (unshuffled) index.
    #[inline]
    pub fn unshuffled(&self, shuffled: usize) -> usize {
        self.inverse[shuffled]
    }

    /// Get the length of the permutation.
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Check if the permutation is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Apply permutation to reorder a slice.
    ///
    /// Returns a new vector with elements in shuffled order.
    pub fn shuffle<T: Clone>(&self, data: &[T]) -> Vec<T> {
        assert_eq!(data.len(), self.len());
        let mut result = Vec::with_capacity(data.len());
        for i in 0..data.len() {
            result.push(data[self.unshuffled(i)].clone());
        }
        result
    }

    /// Apply inverse permutation to restore original order.
    ///
    /// Returns a new vector with elements in original order.
    pub fn unshuffle<T: Clone>(&self, data: &[T]) -> Vec<T> {
        assert_eq!(data.len(), self.len());
        let mut result = Vec::with_capacity(data.len());
        for i in 0..data.len() {
            result.push(data[self.shuffled(i)].clone());
        }
        result
    }
}

/// Hash seed bytes to u64 for RNG seeding.
///
/// Uses a simple FNV-1a inspired hash for deterministic conversion.
fn hash_seed(seed: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    seed.iter().fold(FNV_OFFSET, |hash, &byte| {
        (hash ^ (byte as u64)).wrapping_mul(FNV_PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_deterministic() {
        // Same seed should produce same permutation
        let p1 = Permutation::from_seed(b"test_seed", 100);
        let p2 = Permutation::from_seed(b"test_seed", 100);

        for i in 0..100 {
            assert_eq!(p1.shuffled(i), p2.shuffled(i));
        }
    }

    #[test]
    fn test_permutation_different_seeds() {
        // Different seeds should produce different permutations
        let p1 = Permutation::from_seed(b"seed_a", 100);
        let p2 = Permutation::from_seed(b"seed_b", 100);

        let mut differences = 0;
        for i in 0..100 {
            if p1.shuffled(i) != p2.shuffled(i) {
                differences += 1;
            }
        }
        // Most indices should differ
        assert!(
            differences > 50,
            "Only {} differences, expected > 50",
            differences
        );
    }

    #[test]
    fn test_permutation_bijective() {
        // Permutation must be a bijection (one-to-one mapping)
        let p = Permutation::from_seed(b"test", 100);

        let mut seen = vec![false; 100];
        for i in 0..100 {
            let shuffled = p.shuffled(i);
            assert!(!seen[shuffled], "Duplicate shuffled index {}", shuffled);
            seen[shuffled] = true;
        }

        // All indices should be covered
        assert!(seen.iter().all(|&x| x), "Not all indices covered");
    }

    #[test]
    fn test_permutation_inverse() {
        // unshuffled(shuffled(i)) == i
        let p = Permutation::from_seed(b"test", 100);

        for i in 0..100 {
            let shuffled = p.shuffled(i);
            let restored = p.unshuffled(shuffled);
            assert_eq!(restored, i);
        }
    }

    #[test]
    fn test_permutation_inverse_other_direction() {
        // shuffled(unshuffled(i)) == i
        let p = Permutation::from_seed(b"test", 100);

        for i in 0..100 {
            let unshuffled = p.unshuffled(i);
            let restored = p.shuffled(unshuffled);
            assert_eq!(restored, i);
        }
    }

    #[test]
    fn test_identity_permutation() {
        let p = Permutation::identity(10);

        for i in 0..10 {
            assert_eq!(p.shuffled(i), i);
            assert_eq!(p.unshuffled(i), i);
        }
    }

    #[test]
    fn test_shuffle_data() {
        let p = Permutation::from_seed(b"test", 5);
        let data = vec!['a', 'b', 'c', 'd', 'e'];

        let shuffled = p.shuffle(&data);
        let restored = p.unshuffle(&shuffled);

        assert_eq!(restored, data);
    }

    #[test]
    fn test_empty_permutation() {
        let p = Permutation::from_seed(b"test", 0);
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn test_single_element() {
        let p = Permutation::from_seed(b"test", 1);
        assert_eq!(p.shuffled(0), 0);
        assert_eq!(p.unshuffled(0), 0);
    }

    #[test]
    fn test_hash_seed_deterministic() {
        assert_eq!(hash_seed(b"test"), hash_seed(b"test"));
        assert_ne!(hash_seed(b"test1"), hash_seed(b"test2"));
    }

    #[test]
    fn test_hash_seed_empty() {
        // Empty seed should still produce valid hash
        let hash = hash_seed(b"");
        assert_ne!(hash, 0);
    }
}
