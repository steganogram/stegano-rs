# stegano-f5

F5 steganography algorithm for JPEG images.

## Overview

This crate implements the F5 steganographic algorithm developed by Andreas Westfeld. F5 embeds data into quantized DCT coefficients using matrix encoding and permutative straddling.

The implementation uses two forked JPEG crates for codec access:
- `stegano-f5-jpeg-encoder` — provides a coefficient hook after DCT+quantization
- `stegano-f5-jpeg-decoder` — provides raw coefficient access (skipping IDCT)

```
EMBED:   Pixels → DCT → Quantize → [F5 hook] → Huffman encode → JPEG
EXTRACT: JPEG → Huffman decode → [raw coefficients] → F5 extract
```

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

// Embed a message (decodes JPEG, re-encodes with F5 data)
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
use stegano_f5::{F5Encoder, F5Decoder};

// Embed into pre-obtained coefficients (zigzag order, flat i16 slice)
let encoder = F5Encoder::new();
encoder.embed(&mut coefficients, message, Some(seed))?;

// Extract from coefficients
let decoder = F5Decoder::new();
let extracted = decoder.extract(&coefficients, Some(seed))?;
```


## Running the Tests

```bash
cargo test -p stegano-f5
```

### Run the roundtrip tests with output

```bash
cargo test -p stegano-f5 test_embed_extract_roundtrip -- --nocapture
```

## Architecture

```
crates/stegano-f5/                    # Orchestration layer
├── examples/
│   ├── hide.rs                       # CLI: embed message into JPEG
│   └── unveil.rs                     # CLI: extract message from JPEG
└── src/
    ├── lib.rs                        # Public exports
    ├── jpeg_ops.rs                   # JPEG orchestration (uses both forks)
    ├── encoder.rs                    # F5 embedding algorithm
    ├── decoder.rs                    # F5 extraction algorithm
    ├── matrix.rs                     # Check matrix for matrix encoding
    ├── permutation.rs                # Coefficient permutation
    └── error.rs                      # Error types

crates/stegano-f5-jpeg-encoder/       # Forked jpeg-encoder (with hook)
└── src/encoder.rs                    # Modified: set_coefficient_hook()

crates/stegano-f5-jpeg-decoder/       # Forked jpeg-decoder (with coeff access)
└── src/decoder.rs                    # Modified: decode_raw_coefficients()
```

## References

- [F5 Algorithm Paper](https://link.springer.com/chapter/10.1007/3-540-45496-9_21) - Original F5 paper by Andreas Westfeld
- [JPEG Specification (ITU T.81)](https://www.w3.org/Graphics/JPEG/itu-t81.pdf) - Huffman coding details
