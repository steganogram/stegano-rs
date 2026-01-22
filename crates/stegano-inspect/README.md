# stegano-inspect

CLI for inspecting JPEG internals. Outputs ASCII tables for easy diffing.

## Usage

```bash
# Show quantization tables
cargo run -p stegano-inspect -- quantization path/to/image.jpg

# Multiple files
cargo run -p stegano-inspect -- quantization img1.jpg img2.jpg
```

## Test Fixtures

Checkerboard pattern JPEGs at various quality levels (0-100, step 10).

```bash
# Regenerate 8x8 fixtures
cargo test -p stegano-inspect --release -- --ignored generate_8x8_fixtures

# Regenerate 512x512 fixtures
cargo test -p stegano-inspect --release -- --ignored generate_512x512_fixtures
```
