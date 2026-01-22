# F5 Steganography Implementation Plan (POC)

## Overview

This document describes the implementation plan for F5 steganography in `stegano-rs`. F5 is a JPEG-domain steganographic algorithm developed by Andreas Westfeld that embeds data into quantized DCT (Discrete Cosine Transform) coefficients using matrix encoding and permutative straddling.

**POC Goal**: Prove a round-trip of encoding "Hello World" into a JPEG and decoding it back.

---

## 1. F5 Algorithm Technical Summary

### 1.1 Core Concepts

F5 operates on **quantized DCT coefficients** within JPEG images. Unlike simple LSB replacement, F5 uses:

1. **Matrix Encoding (1, n, k)**: Embeds `k` message bits into `n = 2^k - 1` non-zero AC coefficients with at most **one coefficient modification**
2. **Permutative Straddling**: Pseudo-random shuffling of coefficients to spread modifications uniformly
3. **Shrinkage Handling**: When a coefficient with absolute value 1 becomes 0 after modification, the algorithm re-embeds at the next coefficient

### 1.2 Check Matrix Construction

#### Purpose

The **check matrix** `H_w` is the core of F5's matrix encoding. It serves two purposes:
1. **Encoding**: Determines which single coefficient to modify to embed a group of message bits
2. **Decoding**: Extracts embedded message bits by multiplying the matrix with coefficient LSBs

#### Parameters

- **`w`** (width): Number of message bits embedded per group (encoding parameter)
- **`n = 2^w - 1`**: Number of coefficients needed per group
- The matrix has dimensions **`w` rows × `n` columns**

The trade-off: larger `w` = more efficient (fewer modifications) but requires more coefficients per group.

#### Construction Formula

Each column `j` (where `j` = 1 to n) represents the binary encoding of `j`:

```
H_w[i, j] = bit(j, w - i + 1)
```

This means: column `j` contains the binary representation of number `j`, read from top to bottom.

#### Example: w = 2

With `w = 2`, we embed **2 message bits** into `n = 2² - 1 = 3` coefficients.

The matrix columns are the binary representations of 1, 2, 3:
```
        col 1   col 2   col 3
        (j=1)   (j=2)   (j=3)
        -----   -----   -----
row 1:    0       1       1      ← bit 2 (MSB) of j
row 2:    1       0       1      ← bit 1 (LSB) of j

Binary:   01      10      11     (decimal: 1, 2, 3)
```

So `H_2`:
```
H_2 = | 0 1 1 |
      | 1 0 1 |
```

**Interpretation**: Each column indicates which message bits are affected by that coefficient position. Column 3 (`11`) means coefficient 3 affects both message bits.

#### Example: w = 3

With `w = 3`, we embed **3 message bits** into `n = 2³ - 1 = 7` coefficients.

```
        col 1  col 2  col 3  col 4  col 5  col 6  col 7
        (j=1)  (j=2)  (j=3)  (j=4)  (j=5)  (j=6)  (j=7)
        -----  -----  -----  -----  -----  -----  -----
row 1:    0      0      0      1      1      1      1   ← bit 3 (MSB)
row 2:    0      1      1      0      0      1      1   ← bit 2
row 3:    1      0      1      0      1      0      1   ← bit 1 (LSB)

Binary:  001    010    011    100    101    110    111  (decimal: 1-7)
```

So `H_3`:
```
H_3 = | 0 0 0 1 1 1 1 |
      | 0 1 1 0 0 1 1 |
      | 1 0 1 0 1 0 1 |
```

#### How It Works (Encoding)

To embed message bits `M` into coefficient LSBs `C`:
1. Compute current hash: `s = H_w × C` (matrix multiply mod 2)
2. Compute difference: `d = M XOR s`
3. If `d = 0`: no change needed
4. If `d ≠ 0`: flip coefficient at position `d` (interpreting `d` as column index)

The column structure guarantees that flipping coefficient `d` changes the hash by exactly `d`.

### 1.3 Embedding Algorithm

```
Input: Cover JPEG, Message M (raw bytes - encryption handled externally), Permutation Seed
Output: Stego JPEG

1. Decode JPEG → obtain quantized DCT coefficients
2. Generate permutation from seed → shuffle coefficients
3. Count non-zero AC coefficients (excluding DC at index % 64 == 0)
4. Calculate parameter w based on capacity and message length:
   - (w-1)/(2^(w-1) - 1) < r ≤ w/(2^w - 1)  where r = message_length / capacity
5. Embed 31-bit metadata header:
   - Encoding parameter w (bits to determine)
   - Message length l (bits to determine)
6. For each group of n = 2^w - 1 non-zero AC coefficients:
   a. Read k = w message bits M_i
   b. Extract LSBs of coefficients → C_i
   c. Calculate: a_i = bin2dec(M_i ⊕ (H_w × C_i))
   d. If a_i == 0: no modification needed
   e. If a_i != 0: decrement |coefficient[a_i]| by 1
   f. If coefficient becomes 0 (shrinkage): discard, read next coefficient, retry
7. Inverse shuffle coefficients
8. Re-encode JPEG with modified coefficients
```

### 1.4 Extraction Algorithm

```
Input: Stego JPEG, Permutation Seed
Output: Message M (raw bytes - decryption handled externally)

1. Decode JPEG → obtain quantized DCT coefficients
2. Generate permutation from seed → shuffle coefficients
3. Extract 31-bit metadata header → get w and message length l
4. For each group of n = 2^w - 1 non-zero AC coefficients:
   a. Extract LSBs → S_i
   b. Calculate: M_i = H_w × S_i
   c. Append k = w bits to message
5. Return message (truncated to length l)
```

### 1.5 Modification Rule

F5 **decrements the absolute value** of a coefficient:
- Positive coefficient `c > 0`: `c → c - 1`
- Negative coefficient `c < 0`: `c → c + 1`

This differs from LSB replacement and creates the characteristic "shrinkage" when |c| = 1 becomes 0.

### 1.6 Capacity Estimation

```
L = h_DCT - (h_DCT / 64) - h(0) - 0.51 × h(1)
```

Where:
- `h_DCT` = total DCT coefficients
- `h_DCT / 64` = number of DC coefficients (one per 8×8 block)
- `h(0)` = count of zero AC coefficients
- `h(1)` = count of AC coefficients with |value| = 1
- `0.51 × h(1)` = estimated shrinkage loss

---

## 2. Implementation Architecture

### 2.1 Crate Structure

```
stegano-rs/
├── crates/
│   ├── stegano-f5/                      # F5 algorithm implementation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── encoder.rs               # F5 embedding logic
│   │   │   ├── decoder.rs               # F5 extraction logic
│   │   │   ├── matrix.rs                # Check matrix construction
│   │   │   ├── permutation.rs           # Coefficient shuffling
│   │   │   └── error.rs                 # Error types
│   │   └── Cargo.toml
│   ├── stegano-f5-jpeg-decoder/         # Forked & renamed jpeg-decoder
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── stegano-f5-jpeg-encoder/         # Forked & renamed jpeg-encoder
│   │   ├── src/
│   │   └── Cargo.toml
│   └── stegano-f5-image/                # (Optional) Forked & renamed image
│       ├── src/
│       └── Cargo.toml
```

See [Section 4: JPEG Crate Fork Strategy](#4-jpeg-crate-fork-strategy) for fork procedure details.

### 2.2 Required JPEG Crate Modifications

The forked JPEG crates need modifications to expose DCT coefficient access:

#### 2.2.1 stegano-f5-jpeg-encoder

```rust
// Required: callback-based DCT coefficient access during encoding
pub fn encode_with_dct_callback<F>(
    image: &[u8],
    width: u16,
    height: u16,
    quality: u8,
    callback: F,
) -> Result<Vec<u8>>
where
    F: FnMut(&mut [i16]) -> Result<()>;  // Modifies DCT coefficients in-place
```

#### 2.2.2 stegano-f5-jpeg-decoder

```rust
// Required: access to quantized DCT coefficients during decoding
pub fn decode_with_coefficients(&mut self) -> Result<(Vec<u8>, Vec<i16>)> {
    // Returns (pixel_data, dct_coefficients)
}

// Or callback-based:
pub fn decode_with_dct_callback<F>(&mut self, callback: F) -> Result<Vec<u8>>
where
    F: FnMut(usize, &[i16]);  // (block_index, coefficients)
```

### 2.3 JPEG Codec Integration Points

The JPEG encoding/decoding pipeline has these key stages where we need hooks:

```
ENCODING:
RGB Image → Color Space (YCbCr) → 8x8 Block Split → DCT → Quantization → [F5 HOOK] → Entropy Coding → JPEG File

DECODING:
JPEG File → Entropy Decoding → [F5 HOOK] → Dequantization → IDCT → Block Merge → Color Space (RGB) → Image
```

**Critical**: F5 operates on **quantized** DCT coefficients (after quantization, before entropy coding).

---

## 3. Implementation Components

### 3.1 Check Matrix Module (`matrix.rs`)

```rust
/// Generates the F5 check matrix H_w for parameter w
/// Returns a w × (2^w - 1) matrix
pub struct CheckMatrix {
    w: u8,
    n: usize,  // 2^w - 1
    // Matrix stored as bit vectors for efficiency
}

impl CheckMatrix {
    pub fn new(w: u8) -> Self;

    /// H_w[i, j] = bit(j, w - i + 1)
    pub fn get(&self, i: usize, j: usize) -> bool;

    /// Compute H_w × vector (mod 2)
    pub fn multiply(&self, bits: &[bool]) -> Vec<bool>;

    /// Determine optimal w given capacity and message length
    pub fn optimal_w(capacity: usize, message_len: usize) -> u8;
}
```

#### Unit Tests for CheckMatrix

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_construction_w2() {
        // H_2 should be: | 0 1 1 |
        //                | 1 0 1 |
        let m = CheckMatrix::new(2);
        assert_eq!(m.n, 3);  // 2^2 - 1

        // Column 1 = binary 01 → [0, 1]
        assert_eq!(m.get(0, 0), false);
        assert_eq!(m.get(1, 0), true);

        // Column 2 = binary 10 → [1, 0]
        assert_eq!(m.get(0, 1), true);
        assert_eq!(m.get(1, 1), false);

        // Column 3 = binary 11 → [1, 1]
        assert_eq!(m.get(0, 2), true);
        assert_eq!(m.get(1, 2), true);
    }

    #[test]
    fn test_matrix_construction_w3() {
        let m = CheckMatrix::new(3);
        assert_eq!(m.n, 7);  // 2^3 - 1

        // Column 5 = binary 101 → [1, 0, 1]
        assert_eq!(m.get(0, 4), true);   // bit 3 of 5
        assert_eq!(m.get(1, 4), false);  // bit 2 of 5
        assert_eq!(m.get(2, 4), true);   // bit 1 of 5
    }

    #[test]
    fn test_matrix_multiply() {
        let m = CheckMatrix::new(2);
        // H_2 × [1, 0, 1] = [0⊕0⊕1, 1⊕0⊕1] = [1, 0] (XOR of columns 1 and 3)
        let bits = vec![true, false, true];
        let result = m.multiply(&bits);
        assert_eq!(result, vec![true, false]);
    }

    #[test]
    fn test_optimal_w_selection() {
        // High capacity, small message → high w (more efficient)
        assert!(CheckMatrix::optimal_w(10000, 100) >= 3);

        // Low capacity, large message → low w (need more coefficients)
        assert!(CheckMatrix::optimal_w(1000, 500) <= 2);
    }
}
```

### 3.2 Permutation Module (`permutation.rs`)

```rust
/// Generates a pseudo-random permutation for coefficient shuffling
///
/// Note: The permutation seed is separate from any data encryption.
/// - Permutation seed: Controls WHERE data is hidden (coefficient ordering)
/// - Data encryption: Handled externally before calling F5 (WHAT is hidden)
pub struct Permutation {
    indices: Vec<usize>,
    inverse: Vec<usize>,
}

impl Permutation {
    /// Create permutation from seed bytes
    pub fn from_seed(seed: &[u8], length: usize) -> Self;

    /// Get shuffled index
    pub fn shuffled(&self, original: usize) -> usize;

    /// Get original index from shuffled
    pub fn unshuffled(&self, shuffled: usize) -> usize;
}
```

#### Unit Tests for Permutation

```rust
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
        assert!(differences > 50);
    }

    #[test]
    fn test_permutation_bijective() {
        // Permutation must be a bijection (one-to-one mapping)
        let p = Permutation::from_seed(b"test", 100);

        let mut seen = vec![false; 100];
        for i in 0..100 {
            let shuffled = p.shuffled(i);
            assert!(!seen[shuffled], "Duplicate shuffled index");
            seen[shuffled] = true;
        }
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
}
```

**Decision - PRNG**: Use `fastrand` crate (already in dependency tree) with Fisher-Yates shuffle:
- Seed bytes hashed to `u64` for `Rng::seed()`
- WyRand algorithm: fast, deterministic, sufficient for spreading data (not cryptographic, but encryption is handled externally)
- Note: Not compatible with original F5 Java implementation's java.util.Random

**Decision - Optional permutation**: Support both - allow `Option<&[u8]>` seed. `None` skips permutation (useful for testing), `Some(seed)` enables shuffling.

### 3.3 Encoder Module (`encoder.rs`)

```rust
/// F5 encoder - embeds raw data into DCT coefficients
///
/// Note: F5 does NOT handle encryption. Data passed to embed() should be:
/// - Plain raw bytes, OR
/// - Pre-encrypted bytes (encryption handled by outer layer)
pub struct F5Encoder;

impl F5Encoder {
    pub fn new() -> Self;

    /// Embed message into DCT coefficients
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
    ) -> Result<(), F5Error>;

    /// Calculate embedding capacity in bytes
    pub fn capacity(coefficients: &[i16]) -> usize;
}

// Internal helpers
fn is_dc_coefficient(index: usize) -> bool {
    index % 64 == 0
}

fn is_embeddable(coeff: i16, index: usize) -> bool {
    coeff != 0 && !is_dc_coefficient(index)
}
```

### 3.4 Decoder Module (`decoder.rs`)

```rust
/// F5 decoder - extracts raw data from DCT coefficients
///
/// Note: F5 does NOT handle decryption. Data returned from extract() is:
/// - Plain raw bytes, OR
/// - Encrypted bytes (decryption handled by outer layer)
pub struct F5Decoder;

impl F5Decoder {
    pub fn new() -> Self;

    /// Extract message from DCT coefficients
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
    ) -> Result<Vec<u8>, F5Error>;
}
```

#### Unit Tests for Encoder/Decoder

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Generate synthetic DCT coefficients for testing
    /// Mimics real JPEG coefficient distribution (many zeros, decreasing frequency for larger values)
    fn generate_test_coefficients(block_count: usize) -> Vec<i16> {
        let mut coeffs = Vec::with_capacity(block_count * 64);
        let mut rng = /* seeded RNG */;

        for block in 0..block_count {
            // DC coefficient (index 0 of each block) - larger values
            coeffs.push(rng.gen_range(-500..500));

            // AC coefficients (indices 1-63) - mostly small, many zeros
            for _ in 1..64 {
                let val = match rng.gen_range(0..10) {
                    0..=5 => 0,           // 60% zeros
                    6..=7 => rng.gen_range(-2..=2),  // 20% small
                    8 => rng.gen_range(-10..=10),    // 10% medium
                    _ => rng.gen_range(-50..=50),    // 10% larger
                };
                coeffs.push(val);
            }
        }
        coeffs
    }

    #[test]
    fn test_capacity_calculation() {
        let coeffs = generate_test_coefficients(100);  // 100 blocks = 6400 coefficients
        let capacity = F5Encoder::capacity(&coeffs);

        // Capacity should be positive and less than total non-zero AC coefficients
        assert!(capacity > 0);
        assert!(capacity < coeffs.len());
    }

    #[test]
    fn test_embed_extract_roundtrip_simple() {
        let mut coeffs = generate_test_coefficients(100);
        let original_coeffs = coeffs.clone();
        let message = b"Hello World";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, None).unwrap();

        // Coefficients should be modified
        assert_ne!(coeffs, original_coeffs);

        let decoder = F5Decoder::new();
        let extracted = decoder.extract(&coeffs, None).unwrap();

        assert_eq!(extracted, message);
    }

    #[test]
    fn test_embed_extract_with_permutation() {
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
    fn test_wrong_seed_fails_extraction() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"Secret";
        let seed = b"correct_seed";
        let wrong_seed = b"wrong_seed";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, Some(seed)).unwrap();

        let decoder = F5Decoder::new();
        let result = decoder.extract(&coeffs, Some(wrong_seed));

        // Should either fail or return garbage (not the original message)
        match result {
            Err(_) => { /* Expected: extraction failed */ }
            Ok(extracted) => assert_ne!(extracted, message),
        }
    }

    #[test]
    fn test_capacity_exceeded_error() {
        let mut coeffs = generate_test_coefficients(10);  // Small capacity
        let message = vec![0u8; 10000];  // Large message

        let encoder = F5Encoder::new();
        let result = encoder.embed(&mut coeffs, &message, None);

        assert!(matches!(result, Err(F5Error::CapacityExceeded { .. })));
    }

    #[test]
    fn test_empty_message() {
        let mut coeffs = generate_test_coefficients(100);
        let message = b"";

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, message, None).unwrap();

        let decoder = F5Decoder::new();
        let extracted = decoder.extract(&coeffs, None).unwrap();

        assert_eq!(extracted, message);
    }

    #[test]
    fn test_various_message_sizes() {
        for size in [1, 10, 100, 500, 1000] {
            let mut coeffs = generate_test_coefficients(500);
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
    fn test_dc_coefficients_unchanged() {
        let mut coeffs = generate_test_coefficients(100);
        let dc_before: Vec<i16> = coeffs.iter().step_by(64).cloned().collect();

        let encoder = F5Encoder::new();
        encoder.embed(&mut coeffs, b"test message", None).unwrap();

        let dc_after: Vec<i16> = coeffs.iter().step_by(64).cloned().collect();
        assert_eq!(dc_before, dc_after, "DC coefficients should not be modified");
    }

    #[test]
    fn test_shrinkage_handling() {
        // Create coefficients with many ±1 values to trigger shrinkage
        let mut coeffs: Vec<i16> = (0..6400)
            .map(|i| if i % 64 == 0 { 100 } else { if i % 3 == 0 { 1 } else { -1 } })
            .collect();

        let encoder = F5Encoder::new();
        let result = encoder.embed(&mut coeffs, b"test", None);

        // Should handle shrinkage gracefully (either succeed or fail cleanly)
        assert!(result.is_ok() || matches!(result, Err(F5Error::CapacityExceeded { .. })));
    }
}
```

### 3.5 Data Format & Layer Responsibilities

#### Message Format (NOT F5's concern)

The message format is handled by the **outer layer** (`stegano-core`), not by F5. The `stegano-core` crate defines:

```rust
// stegano-core/src/message.rs
pub struct Message {
    pub header: ContentVersion,  // V1=0x01, V2=0x02, V4=0x04
    pub files: Vec<(String, Vec<u8>)>,
    pub text: Option<String>,
}
```

This `Message` is serialized to `Vec<u8>` before being passed to F5:
- **V4 format**: `[version:1] [payload_size:4] [compressed_zip_data:N]`
- **V2 format**: `[version:1] [compressed_zip_data:N] [0xFF 0xFF]`
- **V1 format**: `[version:1] [text_bytes:N] [0xFF]`

**F5 receives this serialized `Vec<u8>` as opaque raw bytes.** F5 does not interpret, parse, or modify this data - it simply embeds/extracts it.

#### F5 Internal Encoding Metadata

F5 needs minimal internal metadata to function (this is encoding-level, not message-level):

```
| w (4 bits) | data_length (28 bits) |
Total: 32 bits (4 bytes)
```

- `w`: Matrix encoding parameter (1-15, practically 1-9) - decoder needs this to know the encoding scheme
- `data_length`: Length of embedded data in **bytes** - decoder needs this to know when to stop

This is embedded at the start of the coefficient stream, before the actual data.

#### Layer Separation

```
┌─────────────────────────────────────────────────────────────┐
│  Application Layer (stegano-cli, etc.)                      │
│  - User files, text messages                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Transport Layer (stegano-core)                             │
│  - Message struct serialization (ContentVersion header)     │
│  - Compression (zip)                                        │
│  - Encryption (if needed, handled here NOT in F5)           │
│  - Output: Vec<u8> raw bytes                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Encoding Layer (stegano-f5)                                │
│  - Receives: raw Vec<u8> (opaque data)                      │
│  - Adds: minimal encoding metadata (w, length)              │
│  - Embeds into DCT coefficients via matrix encoding         │
│  - Output: modified JPEG                                    │
└─────────────────────────────────────────────────────────────┘
```

#### API Contract

```rust
// F5 receives raw bytes - doesn't care about content
let raw_data: Vec<u8> = message.into();  // stegano-core serializes
f5_encoder.embed(&mut coeffs, &raw_data, seed)?;  // F5 embeds opaquely

// F5 returns raw bytes - doesn't interpret content
let raw_data = f5_decoder.extract(&coeffs, seed)?;
let message = Message::from(&mut raw_data);  // stegano-core deserializes
```

---

## 4. JPEG Crate Fork Strategy

### 4.1 Fork Approach

We fork the required JPEG crates into the workspace with renamed package names to:
- Avoid naming collisions when publishing
- Keep modifications isolated from upstream
- Allow JPEG-only focus (removing unrelated code if needed)

#### Fork Targets

| Upstream | Location | Fork Location | Package Name |
|----------|----------|---------------|--------------|
| `jpeg-decoder` | Separate repo: `image-rs/jpeg-decoder` | `crates/stegano-f5-jpeg-decoder/` | `stegano-f5-jpeg-decoder` |
| JPEG encoder | Embedded in `image` crate: `src/codecs/jpeg/encoder.rs` | `crates/stegano-f5-jpeg-encoder/` | `stegano-f5-jpeg-encoder` |

**Note**: There is no standalone `jpeg-encoder` crate. The encoder lives inside the `image` crate and must be **extracted** rather than forked directly.

#### Fork Procedure: jpeg-decoder (straightforward)

```bash
# 1. Clone upstream into crates directory
git clone https://github.com/image-rs/jpeg-decoder.git crates/stegano-f5-jpeg-decoder

# 2. Remove .git to make it a regular directory (not a submodule)
rm -rf crates/stegano-f5-jpeg-decoder/.git

# 3. Rename package in Cargo.toml
# Change: name = "jpeg-decoder"
# To:     name = "stegano-f5-jpeg-decoder"

# 4. Add to workspace members in root Cargo.toml
# members = [..., "crates/stegano-f5-jpeg-decoder"]
```

#### Extraction Procedure: jpeg-encoder (from image crate)

The JPEG encoder must be extracted from the `image` crate since it's not a standalone crate:

```bash
# 1. Clone image crate temporarily
git clone https://github.com/image-rs/image.git /tmp/image-rs

# 2. Create new crate structure
mkdir -p crates/stegano-f5-jpeg-encoder/src

# 3. Copy relevant encoder files
cp /tmp/image-rs/src/codecs/jpeg/encoder.rs crates/stegano-f5-jpeg-encoder/src/
# Also copy any shared types/utilities the encoder depends on

# 4. Create Cargo.toml with extracted dependencies
# 5. Refactor to be standalone (remove image crate dependencies)
# 6. Add to workspace members
```

**Extraction considerations**:
- The encoder likely depends on image crate types (`DynamicImage`, `ColorType`, etc.)
- May need to define minimal local types or accept raw pixel data directly
- DCT and quantization logic is the critical part to preserve
- Review dependencies: may pull in only what's needed for DCT coefficient access

#### Cleanup (remove existing submodules)

The existing git submodules at `crates/jpeg-decoder` and `crates/image` should be removed:

```bash
# Remove submodule entries
git submodule deinit -f crates/jpeg-decoder crates/image
git rm -f crates/jpeg-decoder crates/image
rm -rf .git/modules/crates/jpeg-decoder .git/modules/crates/image

# Update .gitmodules (remove entries)
# Update root Cargo.toml (remove from workspace members)
```

#### Image Crate Consideration

Since the JPEG encoder is embedded in the `image` crate, we have two approaches:

**Option A: Extract encoder only** (preferred for POC)
- Extract `src/codecs/jpeg/encoder.rs` and its dependencies
- Create minimal standalone `stegano-f5-jpeg-encoder` crate
- Pros: Smaller footprint, focused scope
- Cons: May require significant refactoring to decouple from image types

**Option B: Fork entire image crate**
- Fork full `image` crate as `stegano-f5-image`
- Remove non-JPEG format support (PNG, GIF, BMP, etc.)
- Keep: `src/codecs/jpeg/`, core image types, minimal dependencies
- Pros: Less refactoring, maintains internal consistency
- Cons: Larger codebase to maintain

**Recommendation**: Start with Option A. If extraction proves too complex due to tight coupling, fall back to Option B.

### 4.2 Required Modifications

#### In `stegano-f5-jpeg-decoder`:

```rust
// Add to Decoder struct - expose quantized DCT coefficients
pub fn decode_with_coefficients(&mut self) -> Result<(Vec<u8>, Vec<i16>), Error> {
    // Returns (pixel_data, dct_coefficients)
}

// Alternative: callback-based access during decoding
pub fn decode_with_dct_hook<F>(&mut self, hook: F) -> Result<Vec<u8>, Error>
where
    F: FnMut(usize, &[i16]);  // (block_index, coefficients)
```

#### In `stegano-f5-jpeg-encoder`:

```rust
// Add coefficient interception point after quantization, before entropy coding
pub fn encode_with_dct_hook<F>(
    &mut self,
    image: &[u8],
    width: u16,
    height: u16,
    hook: F,
) -> Result<Vec<u8>, Error>
where
    F: FnMut(usize, &mut [i16]);  // (block_index, coefficients)
```

### 4.3 Resulting Crate Structure

```
stegano-rs/
├── crates/
│   ├── stegano-f5/                      # F5 algorithm implementation
│   │   └── Cargo.toml                   # depends on stegano-f5-jpeg-*
│   ├── stegano-f5-jpeg-decoder/         # Forked jpeg-decoder
│   │   ├── Cargo.toml                   # name = "stegano-f5-jpeg-decoder"
│   │   └── src/
│   ├── stegano-f5-jpeg-encoder/         # Forked jpeg-encoder
│   │   ├── Cargo.toml                   # name = "stegano-f5-jpeg-encoder"
│   │   └── src/
│   └── stegano-f5-image/                # (Optional) Forked image crate
│       ├── Cargo.toml                   # name = "stegano-f5-image"
│       └── src/
```

#### Dependency Graph

```
stegano-f5
    ├── stegano-f5-jpeg-decoder (for extraction)
    └── stegano-f5-jpeg-encoder (for embedding)
            └── (may depend on stegano-f5-jpeg-decoder internally)
```

---

## 5. POC Implementation Plan

### 5.1 Phase 1: Core Algorithm (No JPEG Integration)

**Goal**: Implement and test F5 matrix encoding/decoding on synthetic coefficient arrays.

1. Implement `CheckMatrix` with unit tests
2. Implement `Permutation` with unit tests
3. Implement `F5Encoder::embed()` on `&mut [i16]`
4. Implement `F5Decoder::extract()` on `&[i16]`
5. Write round-trip test with synthetic coefficients

```rust
#[test]
fn test_f5_roundtrip_synthetic() {
    let mut coefficients = generate_synthetic_coefficients(10000);
    let message = b"Hello World";  // Raw bytes (could be pre-encrypted externally)
    let seed = b"test_seed";        // Permutation seed (not encryption key)

    let encoder = F5Encoder::new();
    encoder.embed(&mut coefficients, message, Some(seed)).unwrap();

    let decoder = F5Decoder::new();
    let extracted = decoder.extract(&coefficients, Some(seed)).unwrap();

    assert_eq!(extracted, message);
}

#[test]
fn test_f5_roundtrip_no_permutation() {
    // Test without permutation (simpler, less secure, good for debugging)
    let mut coefficients = generate_synthetic_coefficients(10000);
    let message = b"Hello World";

    let encoder = F5Encoder::new();
    encoder.embed(&mut coefficients, message, None).unwrap();  // No shuffle

    let decoder = F5Decoder::new();
    let extracted = decoder.extract(&coefficients, None).unwrap();

    assert_eq!(extracted, message);
}
```

### 5.2 Phase 2: JPEG Integration

**Goal**: Integrate F5 with actual JPEG files.

1. Implement `JpegProcessor` (or fork jpeg-decoder/encoder)
2. Expose DCT coefficient access
3. Wire up F5 encoder/decoder to JPEG processor
4. Write integration test with real JPEG

```rust
#[test]
fn test_f5_roundtrip_jpeg() {
    let cover_path = "test_data/cover.jpg";
    let stego_path = "test_data/stego.jpg";
    let message = b"Hello World";  // Raw bytes
    let seed = b"test_seed";        // Permutation seed

    // Embed
    let mut jpeg = JpegProcessor::load(cover_path).unwrap();
    let encoder = F5Encoder::new();
    encoder.embed(jpeg.coefficients_mut(), message, Some(seed)).unwrap();
    jpeg.save(stego_path).unwrap();

    // Extract
    let jpeg = JpegProcessor::load(stego_path).unwrap();
    let decoder = F5Decoder::new();
    let extracted = decoder.extract(jpeg.coefficients(), Some(seed)).unwrap();

    assert_eq!(extracted, b"Hello World");
}
```

### 5.3 Phase 3: Public API

**Goal**: Provide clean public API for stegano-f5 crate.

```rust
// High-level API
//
// Note: These functions handle raw bytes only.
// Encryption/decryption is the caller's responsibility.

/// Embed raw data into a JPEG file
///
/// # Arguments
/// * `cover_path` - Path to cover JPEG
/// * `stego_path` - Path to write stego JPEG
/// * `data` - Raw bytes to embed (plain or pre-encrypted)
/// * `permutation_seed` - Optional seed for coefficient shuffling
pub fn embed_in_jpeg(
    cover_path: &Path,
    stego_path: &Path,
    data: &[u8],
    permutation_seed: Option<&[u8]>,
) -> Result<(), F5Error>;

/// Extract raw data from a stego JPEG file
///
/// # Arguments
/// * `stego_path` - Path to stego JPEG
/// * `permutation_seed` - Optional seed (must match embed call)
///
/// # Returns
/// Raw bytes (caller handles decryption if needed)
pub fn extract_from_jpeg(
    stego_path: &Path,
    permutation_seed: Option<&[u8]>,
) -> Result<Vec<u8>, F5Error>;

// Capacity check
pub fn jpeg_capacity(path: &Path) -> Result<usize, F5Error>;
```

---

## 6. Testing Strategy

Testing is critical for this POC. We organize tests into three layers: unit tests (internal logic), integration tests (API boundaries), and end-to-end tests (full JPEG round-trip).

### 6.1 Test Pyramid

```
                    ┌─────────────────┐
                    │   E2E Tests     │  ← Full JPEG round-trip
                    │   (few, slow)   │
                    └────────┬────────┘
                             │
               ┌─────────────┴─────────────┐
               │    Integration Tests      │  ← API boundaries
               │    (moderate count)       │
               └─────────────┬─────────────┘
                             │
    ┌────────────────────────┴────────────────────────┐
    │              Unit Tests                         │  ← Internal logic
    │              (many, fast)                       │
    └─────────────────────────────────────────────────┘
```

### 6.2 Unit Tests (per module)

Located in each module's `#[cfg(test)]` block. See Section 3 for detailed test code.

| Module | Test Focus |
|--------|------------|
| `matrix.rs` | Matrix construction, multiplication, optimal w selection |
| `permutation.rs` | Determinism, bijectivity, inverse correctness |
| `encoder.rs` | Capacity calculation, coefficient modification rules |
| `decoder.rs` | Header parsing, message extraction |

### 6.3 Integration Tests (API boundaries)

Located in `stegano-f5/tests/` directory. Test interactions between components.

#### 6.3.1 Encoder ↔ Decoder Integration

```rust
// tests/encoder_decoder_integration.rs

#[test]
fn test_encoder_decoder_contract() {
    // Verify encoder output is valid input for decoder
    let mut coeffs = generate_test_coefficients(200);
    let message = b"Integration test message";
    let seed = b"integration_seed";

    F5Encoder::new().embed(&mut coeffs, message, Some(seed)).unwrap();

    // Decoder should accept encoder's output
    let extracted = F5Decoder::new().extract(&coeffs, Some(seed)).unwrap();
    assert_eq!(extracted, message);
}

#[test]
fn test_header_format_compatibility() {
    // Verify header written by encoder is readable by decoder
    let mut coeffs = generate_test_coefficients(200);

    F5Encoder::new().embed(&mut coeffs, b"test", None).unwrap();

    // Decoder should correctly parse header (w parameter, message length)
    let extracted = F5Decoder::new().extract(&coeffs, None).unwrap();
    assert_eq!(extracted.len(), 4);  // "test" = 4 bytes
}
```

#### 6.3.2 F5 ↔ JPEG Codec Integration

```rust
// tests/jpeg_codec_integration.rs

use stegano_f5_jpeg_decoder::Decoder;
use stegano_f5_jpeg_encoder::Encoder;

#[test]
fn test_jpeg_decoder_exposes_coefficients() {
    let jpeg_bytes = include_bytes!("fixtures/test_image.jpg");
    let mut decoder = Decoder::new(jpeg_bytes.as_slice());

    let (pixels, coefficients) = decoder.decode_with_coefficients().unwrap();

    // Verify coefficients structure
    assert!(!coefficients.is_empty());
    assert_eq!(coefficients.len() % 64, 0);  // Multiple of block size

    // Verify DC coefficients exist (first of each block)
    let dc_count = coefficients.len() / 64;
    assert!(dc_count > 0);
}

#[test]
fn test_jpeg_encoder_accepts_modified_coefficients() {
    // Load, modify, re-encode should produce valid JPEG
    let jpeg_bytes = include_bytes!("fixtures/test_image.jpg");
    let mut decoder = Decoder::new(jpeg_bytes.as_slice());
    let (_, mut coefficients) = decoder.decode_with_coefficients().unwrap();

    // Modify some AC coefficients (simulating F5 embedding)
    for (i, coeff) in coefficients.iter_mut().enumerate() {
        if i % 64 != 0 && *coeff != 0 {  // Skip DC, skip zeros
            *coeff = coeff.saturating_sub(1);
        }
    }

    // Re-encode should succeed
    let encoded = Encoder::encode_with_coefficients(&coefficients, /* params */).unwrap();
    assert!(!encoded.is_empty());

    // Result should be valid JPEG (decodable)
    let mut verify_decoder = Decoder::new(encoded.as_slice());
    assert!(verify_decoder.decode().is_ok());
}

#[test]
fn test_coefficient_roundtrip_without_f5() {
    // Decode → Re-encode → Decode should preserve coefficients (baseline)
    let jpeg_bytes = include_bytes!("fixtures/test_image.jpg");

    let (_, coeffs_original) = Decoder::new(jpeg_bytes.as_slice())
        .decode_with_coefficients().unwrap();

    let reencoded = Encoder::encode_with_coefficients(&coeffs_original, /* params */).unwrap();

    let (_, coeffs_after) = Decoder::new(reencoded.as_slice())
        .decode_with_coefficients().unwrap();

    // Coefficients should be identical (no F5 modification)
    assert_eq!(coeffs_original, coeffs_after);
}
```

### 6.4 End-to-End Tests

Located in `stegano-f5/tests/` directory. Full round-trip with actual JPEG files.

```rust
// tests/e2e_jpeg_roundtrip.rs

use std::fs;
use tempfile::tempdir;

#[test]
fn test_e2e_hello_world() {
    let dir = tempdir().unwrap();
    let cover_path = Path::new("tests/fixtures/cover.jpg");
    let stego_path = dir.path().join("stego.jpg");

    let message = b"Hello World";
    let seed = b"e2e_test_seed";

    // Embed
    embed_in_jpeg(cover_path, &stego_path, message, Some(seed)).unwrap();

    // Verify stego file exists and is valid JPEG
    assert!(stego_path.exists());
    assert!(is_valid_jpeg(&stego_path));

    // Extract
    let extracted = extract_from_jpeg(&stego_path, Some(seed)).unwrap();

    assert_eq!(extracted, message);
}

#[test]
fn test_e2e_binary_data() {
    let dir = tempdir().unwrap();
    let cover_path = Path::new("tests/fixtures/cover.jpg");
    let stego_path = dir.path().join("stego.jpg");

    // Binary data with all byte values
    let message: Vec<u8> = (0..=255).collect();
    let seed = b"binary_test";

    embed_in_jpeg(cover_path, &stego_path, &message, Some(seed)).unwrap();
    let extracted = extract_from_jpeg(&stego_path, Some(seed)).unwrap();

    assert_eq!(extracted, message);
}

#[test]
fn test_e2e_different_jpeg_qualities() {
    for quality in [50, 75, 90, 95] {
        let dir = tempdir().unwrap();
        let cover_path = format!("tests/fixtures/cover_q{}.jpg", quality);
        let stego_path = dir.path().join("stego.jpg");

        let message = format!("Quality {} test", quality);

        embed_in_jpeg(&cover_path, &stego_path, message.as_bytes(), None).unwrap();
        let extracted = extract_from_jpeg(&stego_path, None).unwrap();

        assert_eq!(extracted, message.as_bytes(), "Failed for quality {}", quality);
    }
}

#[test]
fn test_e2e_different_image_sizes() {
    for size in ["64x64", "256x256", "512x512", "1024x768"] {
        let dir = tempdir().unwrap();
        let cover_path = format!("tests/fixtures/cover_{}.jpg", size);
        let stego_path = dir.path().join("stego.jpg");

        let message = b"Size test";

        let result = embed_in_jpeg(&cover_path, &stego_path, message, None);
        if result.is_ok() {
            let extracted = extract_from_jpeg(&stego_path, None).unwrap();
            assert_eq!(extracted, message, "Failed for size {}", size);
        }
        // Small images may not have enough capacity - that's OK
    }
}

#[test]
fn test_e2e_stego_image_visually_similar() {
    let dir = tempdir().unwrap();
    let cover_path = Path::new("tests/fixtures/cover.jpg");
    let stego_path = dir.path().join("stego.jpg");

    embed_in_jpeg(cover_path, &stego_path, b"Hidden message", None).unwrap();

    // Load both images and compare
    let cover_pixels = load_jpeg_pixels(cover_path);
    let stego_pixels = load_jpeg_pixels(&stego_path);

    // PSNR (Peak Signal-to-Noise Ratio) should be high (> 40 dB is imperceptible)
    let psnr = calculate_psnr(&cover_pixels, &stego_pixels);
    assert!(psnr > 40.0, "PSNR {} dB is too low - visible artifacts", psnr);
}
```

### 6.5 Test Fixtures

```
stegano-f5/
└── tests/
    └── fixtures/
        ├── cover.jpg              # Standard test image (e.g., 512x512)
        ├── cover_q50.jpg          # Low quality
        ├── cover_q75.jpg          # Medium quality
        ├── cover_q90.jpg          # High quality
        ├── cover_q95.jpg          # Very high quality
        ├── cover_64x64.jpg        # Small image
        ├── cover_256x256.jpg      # Medium image
        ├── cover_512x512.jpg      # Standard size
        └── cover_1024x768.jpg     # Large image
```

### 6.6 Test Utilities Module

```rust
// tests/common/mod.rs

/// Generate synthetic DCT coefficients mimicking real JPEG distribution
pub fn generate_test_coefficients(block_count: usize) -> Vec<i16>;

/// Check if file is valid JPEG
pub fn is_valid_jpeg(path: &Path) -> bool;

/// Load JPEG as raw pixel values
pub fn load_jpeg_pixels(path: &Path) -> Vec<u8>;

/// Calculate PSNR between two images
pub fn calculate_psnr(original: &[u8], modified: &[u8]) -> f64;

/// Generate random message of given size
pub fn random_message(size: usize) -> Vec<u8>;
```

### 6.7 Running Tests

```bash
# Run all tests
cargo test -p stegano-f5

# Run only unit tests (fast)
cargo test -p stegano-f5 --lib

# Run integration tests
cargo test -p stegano-f5 --test '*'

# Run with verbose output
cargo test -p stegano-f5 -- --nocapture

# Run specific test
cargo test -p stegano-f5 test_e2e_hello_world
```

---

## 7. Resolved Design Decisions

### 7.1 Algorithm Decisions

| Decision | Resolution |
|----------|------------|
| **PRNG for permutation** | Use `fastrand` (already in dependency tree) with Fisher-Yates shuffle. Not compatible with original F5's java.util.Random. |
| **Shrinkage handling** | Follow original F5 behavior exactly (re-embed same message bits after shrinkage). |
| **Matrix encoding parameter `w`** | Allow user to specify `w`, but auto-calculate optimal value if not specified. |

### 7.2 Integration Decisions

| Decision | Resolution |
|----------|------------|
| **JPEG library approach** | Fork `jpeg-encoder` and `jpeg-decoder` directly. Only fork `image` crate if necessary. Keep it simple. |
| **Coefficient ordering** | Process in zigzag order as stored. Follow F5 algorithm/paper semantics. |
| **Color component handling** | Embed in all components (Y, Cb, Cr) as original F5 does. |

### 7.3 Testing Decisions

| Decision | Resolution |
|----------|------------|
| **Test images** | Use existing `resources/` images. F5-specific test images go in `resources/f5/`. Base images: `resources/Base.png`, `resources/NoSecrets.jpg`. Generate synthetic images if systematic testing requires it. |
| **Compatibility testing** | Not compatible with original F5 Java implementation (different PRNG, own header format). Acceptable for POC. |

---

## 8. Dependencies

### 8.1 Required Crates

```toml
[dependencies]
# Core
thiserror = "1.0"           # Error handling
bitstream-io = "2.0"        # Bit-level I/O

# PRNG (for permutation) - already in dependency tree
fastrand = "2"              # Simple, fast, seedable PRNG (Fisher-Yates shuffle)

# JPEG processing (forked crates)
stegano-f5-jpeg-decoder = { path = "../stegano-f5-jpeg-decoder" }
stegano-f5-jpeg-encoder = { path = "../stegano-f5-jpeg-encoder" }
# Note: image crate fork only if needed (prefer working with jpeg crates directly)

[dev-dependencies]
tempfile = "3.0"            # Temporary files for tests
```

### 8.2 Feature Flags (Future)

```toml
[features]
default = ["std"]
std = []
original-f5-compat = []  # Use original F5 PRNG and header format
```

---

## 9. Success Criteria for POC

1. **Round-trip test passes**: Embed "Hello World" into a JPEG, extract it back, verify match
2. **Multiple message sizes**: Test with messages of varying lengths
3. **Capacity calculation**: Accurately predict if message will fit
4. **Error handling**: Graceful failure when message exceeds capacity
5. **Basic documentation**: API documentation and usage examples

---

## 10. References

1. Westfeld, A. (2001). "F5—A Steganographic Algorithm: High Capacity Despite Better Steganalysis." Information Hiding Workshop, LNCS 2137.

2. Liu et al. (2020). "Stego key recovery method for F5 steganography with matrix encoding." EURASIP Journal on Image and Video Processing.

3. Original F5 Java implementation: https://github.com/matthewgao/F5-steganography

4. JavaScript port: https://github.com/desudesutalk/f5stegojs

---

## 11. Appendix: Matrix Encoding Performance

| w | n (coefficients) | Embedding ratio | Efficiency |
|---|------------------|-----------------|------------|
| 1 | 1                | 100.00%         | 2.00       |
| 2 | 3                | 66.67%          | 2.67       |
| 3 | 7                | 42.86%          | 3.43       |
| 4 | 15               | 26.67%          | 4.27       |
| 5 | 31               | 16.13%          | 5.16       |
| 6 | 63               | 9.52%           | 6.09       |
| 7 | 127              | 5.51%           | 7.06       |
| 8 | 255              | 3.14%           | 8.03       |
| 9 | 511              | 1.76%           | 9.02       |

*Embedding efficiency = bits embedded per coefficient change*
