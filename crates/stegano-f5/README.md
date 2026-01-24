# stegano-f5

F5 steganography algorithm for JPEG images.

## Overview

This crate implements the F5 steganographic algorithm developed by Andreas Westfeld. F5 embeds data into quantized DCT coefficients using matrix encoding and permutative straddling.

The implementation uses a **transcode-only** approach: it decodes JPEG scan data to DCT coefficients via Huffman decoding, applies F5 modifications, and re-encodes without full decode/re-encode. This preserves image quality since we never dequantize or requantize.

```
JPEG → Huffman decode → [i16] coefficients → F5 → Huffman encode → JPEG
```

## Usage

## Command-Line Examples

The crate includes two examples for quick command-line usage.

### Hide a message

```bash
cargo run -p stegano-f5 --example hide -- <input.jpg> <seed> <message>
```

Example:
```bash
cargo run -p stegano-f5 --example hide -- cover.jpg mysecret "Hello from F5!"
```

Output:
```
Cover image: cover.jpg
Capacity: 1053578 bytes
Message length: 14 bytes
Output: cover_stegano.jpg
Size: 6089809 -> 6089517 bytes (-292 bytes)
Done!
```

### Extract a message

```bash
cargo run -p stegano-f5 --example unveil -- <stego.jpg> <seed>
```

Example:
```bash
cargo run -p stegano-f5 --example unveil -- cover_stegano.jpg mysecret
```

Output:
```
Hello from F5!
```

Use an empty string `""` for the seed if no seed was used during embedding.


### High-Level API

```rust
use stegano_f5::{embed_in_jpeg, extract_from_jpeg, jpeg_capacity};

// Check capacity
let cover = std::fs::read("cover.jpg")?;
let capacity = jpeg_capacity(&cover)?;
println!("Can embed up to {} bytes", capacity);

// Embed a message
let message = b"Secret message";
let seed = b"optional_seed";
let stego = embed_in_jpeg(&cover, message, Some(seed))?;
std::fs::write("stego.jpg", stego)?;

// Extract the message
let stego = std::fs::read("stego.jpg")?;
let extracted = extract_from_jpeg(&stego, Some(seed))?;
assert_eq!(&extracted[..message.len()], message);
```

### Low-Level API

```rust
use stegano_f5::{F5Encoder, F5Decoder, jpeg};

// Parse JPEG and decode coefficients
let segments = jpeg::parse_jpeg(&jpeg_data)?;
let mut coefficients = jpeg::decode_scan(&segments)?;

// Embed using F5
let encoder = F5Encoder::new();
encoder.embed(coefficients.as_mut_slice(), message, Some(seed))?;

// Re-encode and write JPEG
let new_scan = jpeg::encode_scan(&coefficients, &segments)?;
let output = jpeg::write_jpeg(&segments, &new_scan);
```


## Running the Roundtrip Tests

The crate includes tests that verify the full embed/extract roundtrip works correctly.

### Run all tests

```bash
cargo test -p stegano-f5
```

### Run the roundtrip test with output

```bash
cargo test -p stegano-f5 test_embed_extract_roundtrip -- --nocapture
```

Expected output:
```
Restart interval: 0
Original coefficient count: 10469376
Original non-zero count: 8590700
DC coefficients changed: 0
AC coefficients changed: 66
Coefficients shrunk to 0: 24
Original scan data: 6076253 bytes
New scan data: 6075955 bytes
test jpeg::api_tests::test_embed_extract_roundtrip ... ok
```

### Run specific test suites

```bash
# Huffman encode/decode tests
cargo test -p stegano-f5 huffman

# Scan encode/decode tests
cargo test -p stegano-f5 scan

# Full API tests
cargo test -p stegano-f5 api_tests
```

## Verifying the Implementation

### 1. Coefficient Roundtrip

The scan roundtrip test verifies that decoding and re-encoding produces identical coefficients:

```bash
cargo test -p stegano-f5 test_encode_decode_roundtrip -- --nocapture
```

### 2. F5 Embed/Extract Roundtrip

The API roundtrip test verifies the full F5 pipeline:

```bash
cargo test -p stegano-f5 test_embed_extract_roundtrip -- --nocapture
```

This test:
1. Parses a cover JPEG
2. Decodes DCT coefficients
3. Embeds a message using F5
4. Re-encodes to scan data
5. Decodes again to verify coefficients preserved
6. Extracts the message and verifies it matches

### 3. Shrinkage Handling

F5 can "shrink" coefficients (change ±1 to 0). The implementation handles this correctly:

```bash
cargo test -p stegano-f5 test_encode_decode_with_shrinkage -- --nocapture
cargo test -p stegano-f5 test_encode_decode_with_multiple_shrinkages -- --nocapture
```

## Architecture

```
crates/stegano-f5/
├── examples/
│   ├── hide.rs         # CLI: embed message into JPEG
│   └── unveil.rs       # CLI: extract message from JPEG
│
└── src/
    ├── lib.rs          # Public exports
    ├── encoder.rs      # F5 embedding algorithm
    ├── decoder.rs      # F5 extraction algorithm
    ├── matrix.rs       # Check matrix for matrix encoding
    ├── permutation.rs  # Coefficient permutation
    ├── error.rs        # Error types
    │
    └── jpeg/           # JPEG transcode module
        ├── mod.rs      # High-level API (embed_in_jpeg, extract_from_jpeg)
        ├── marker.rs   # JPEG marker constants
        ├── parser.rs   # JPEG structure parsing
        ├── huffman.rs  # Huffman encode/decode
        ├── scan.rs     # Scan data encode/decode
        └── writer.rs   # JPEG file assembly
```

## References

- [F5 Algorithm Paper](https://link.springer.com/chapter/10.1007/3-540-45496-9_21) - Original F5 paper by Andreas Westfeld
- [JPEG Specification (ITU T.81)](https://www.w3.org/Graphics/JPEG/itu-t81.pdf) - Huffman coding details
