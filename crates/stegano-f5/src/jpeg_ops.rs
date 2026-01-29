//! JPEG orchestration for F5 steganography.
//!
//! This module provides high-level functions that coordinate the forked
//! encoder and decoder crates to embed/extract F5 data in JPEG images.

use crate::error::{F5Error, Result};
use crate::{F5Decoder, F5Encoder};
use stegano_f5_jpeg_encoder::{EncodingError, ZIGZAG};

/// Convert a block from natural (row-major) order to zigzag order.
#[inline]
fn block_natural_to_zigzag(natural: &[i16]) -> [i16; 64] {
    let mut zigzag = [0i16; 64];
    for zz_pos in 0..64 {
        zigzag[zz_pos] = natural[ZIGZAG[zz_pos] as usize];
    }
    zigzag
}

/// Embed data into a JPEG by re-encoding from an existing JPEG.
///
/// Decodes the JPEG to pixels, then re-encodes with F5 embedding via
/// the encoder's coefficient hook.
///
/// Note: This re-encodes the image, which may change compression characteristics.
/// The embedded message is preserved in the output JPEG's DCT coefficients.
pub fn embed_in_jpeg(jpeg_data: &[u8], message: &[u8], seed: Option<&[u8]>) -> Result<Vec<u8>> {
    // Decode JPEG to pixels
    let mut decoder = stegano_f5_jpeg_decoder::Decoder::new(jpeg_data);
    let pixels = decoder
        .decode()
        .map_err(|e| F5Error::JpegDecodeFailed(Box::new(e)))?;
    let info = decoder.info().ok_or_else(|| {
        F5Error::JpegDecodeFailed(Box::new(std::io::Error::other("no image info available")))
    })?;

    let color_type = match info.pixel_format {
        stegano_f5_jpeg_decoder::PixelFormat::L8 | stegano_f5_jpeg_decoder::PixelFormat::L16 => {
            stegano_f5_jpeg_encoder::ColorType::Luma
        }
        stegano_f5_jpeg_decoder::PixelFormat::RGB24 => stegano_f5_jpeg_encoder::ColorType::Rgb,
        stegano_f5_jpeg_decoder::PixelFormat::CMYK32 => stegano_f5_jpeg_encoder::ColorType::Cmyk,
    };

    embed_in_jpeg_from_image(
        &pixels,
        info.width,
        info.height,
        90,
        color_type,
        message,
        seed,
    )
}

/// Embed data into a JPEG by encoding from raw pixel data.
///
/// Uses the encoder's coefficient hook to intercept quantized DCT coefficients
/// and embed F5 data before Huffman encoding.
pub fn embed_in_jpeg_from_image(
    image_data: &[u8],
    width: u16,
    height: u16,
    quality: u8,
    color_type: stegano_f5_jpeg_encoder::ColorType,
    message: &[u8],
    seed: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut encoder = stegano_f5_jpeg_encoder::Encoder::new(&mut output, quality);

    // Capture message and seed for the hook closure
    let message = message.to_vec();
    let seed = seed.map(|s| s.to_vec());

    encoder.set_coefficient_hook(
        move |blocks: &mut [Vec<[i16; 64]>; 4]| -> std::result::Result<(), EncodingError> {
            // Flatten all component blocks into a single coefficient slice for F5.
            // Encoder blocks are already in zigzag order (from quantize_block).
            let mut flat: Vec<i16> = blocks
                .iter()
                .flat_map(|component| component.iter().flat_map(|block| block.iter()))
                .copied()
                .collect();

            let f5 = F5Encoder::new();
            f5.embed(&mut flat, &message, seed.as_deref())
                .map_err(|e| EncodingError::HookError(e.to_string()))?;

            // Write back modified coefficients
            let mut offset = 0;
            for component in blocks.iter_mut() {
                for block in component.iter_mut() {
                    block.copy_from_slice(&flat[offset..offset + 64]);
                    offset += 64;
                }
            }
            Ok(())
        },
    );

    encoder
        .encode(image_data, width, height, color_type)
        .map_err(|e| match e {
            // HookError contains an F5 error message - propagate it directly
            EncodingError::HookError(msg) => F5Error::EmbeddingFailed(msg),
            // Other encoder errors
            other => F5Error::JpegEncodeFailed(Box::new(other)),
        })?;

    Ok(output)
}

/// Extract data from a JPEG image using F5 steganography.
///
/// Uses the decoder fork to get raw quantized coefficients, converts them
/// to zigzag order (matching the encoder's format), then extracts the F5 message.
pub fn extract_from_jpeg(jpeg_data: &[u8], seed: Option<&[u8]>) -> Result<Vec<u8>> {
    let mut decoder = stegano_f5_jpeg_decoder::Decoder::new(jpeg_data);
    let raw = decoder
        .decode_raw_coefficients()
        .map_err(|e| F5Error::JpegDecodeFailed(Box::new(e)))?;

    // Convert from natural order (decoder output) to zigzag order (F5 standard).
    // Process each 64-coefficient block through the zigzag mapping.
    let flat: Vec<i16> = raw
        .components
        .iter()
        .flat_map(|component| component.chunks_exact(64).flat_map(block_natural_to_zigzag))
        .collect();

    let f5 = F5Decoder::new();
    f5.extract(&flat, seed)
}

/// Calculate the embedding capacity of a JPEG image.
///
/// Returns the maximum number of bytes that can be embedded using F5.
pub fn jpeg_capacity(jpeg_data: &[u8]) -> Result<usize> {
    let mut decoder = stegano_f5_jpeg_decoder::Decoder::new(jpeg_data);
    let raw = decoder
        .decode_raw_coefficients()
        .map_err(|e| F5Error::JpegDecodeFailed(Box::new(e)))?;

    // Count usable coefficients: non-zero AC coefficients across all components.
    // DC coefficient is at index 0 of each 64-value block (in both natural and zigzag order).
    let mut usable_count = 0;
    for component in &raw.components {
        for block in component.chunks_exact(64) {
            // Skip DC (index 0), count non-zero AC coefficients
            for &coeff in &block[1..] {
                if coeff != 0 {
                    usable_count += 1;
                }
            }
        }
    }

    // Conservative capacity estimate: 1 bit per usable coefficient
    Ok(usable_count / 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VESSEL: &[u8] = include_bytes!("../resources/test_512x512_255_90.jpg");

    #[test]
    fn test_embed_extract_roundtrip_from_image() {
        // Create a 128x128 image with varied pixel data (pseudo-random pattern)
        // A uniform image would produce zero AC coefficients, so we need texture.
        let width = 128u16;
        let height = 128u16;
        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 3];
        let mut rng = fastrand::Rng::with_seed(42);
        for pixel in pixels.iter_mut() {
            *pixel = rng.u8(..);
        }
        let message = b"Hello, F5 steganography!";
        let seed = b"test_seed_123";

        // Embed
        let stego = embed_in_jpeg_from_image(
            &pixels,
            width,
            height,
            90,
            stegano_f5_jpeg_encoder::ColorType::Rgb,
            message,
            Some(seed),
        )
        .expect("embed should succeed");

        // Verify it's a valid JPEG
        assert_eq!(&stego[0..2], &[0xFF, 0xD8]); // SOI marker

        // Extract
        let extracted = extract_from_jpeg(&stego, Some(seed)).expect("extract should succeed");

        assert!(
            extracted.starts_with(message),
            "Extracted message should match. Got: {:?}",
            String::from_utf8_lossy(&extracted[..message.len().min(extracted.len())])
        );
    }

    #[test]
    fn test_embed_extract_roundtrip_from_jpeg() {
        let cover = VESSEL;
        let message = b"Hello World";
        let seed = b"test_seed";

        // Embed via transcode
        let stego =
            embed_in_jpeg(cover, message, Some(seed)).expect("embed_in_jpeg should succeed");

        // Extract
        let extracted = extract_from_jpeg(&stego, Some(seed)).expect("extract should succeed");

        assert!(
            extracted.starts_with(message),
            "Extracted message should match. Got: {:?}",
            String::from_utf8_lossy(&extracted[..message.len().min(extracted.len())])
        );
    }

    #[test]
    fn test_jpeg_capacity() {
        let jpeg = VESSEL;
        let capacity = jpeg_capacity(jpeg).expect("capacity should succeed");

        println!("JPEG capacity: {} bytes", capacity);
        assert!(
            capacity > 1000,
            "Large image should have significant capacity"
        );
    }

    #[test]
    fn test_extract_wrong_seed() {
        let mut pixels = vec![0u8; 128 * 128 * 3];
        let mut rng = fastrand::Rng::with_seed(99);
        for pixel in pixels.iter_mut() {
            *pixel = rng.u8(..);
        }
        let message = b"Secret";
        let seed = b"correct_seed";
        let wrong_seed = b"wrong_seed";

        let stego = embed_in_jpeg_from_image(
            &pixels,
            128,
            128,
            90,
            stegano_f5_jpeg_encoder::ColorType::Rgb,
            message,
            Some(seed),
        )
        .unwrap();

        let result = extract_from_jpeg(&stego, Some(wrong_seed));
        match result {
            Err(_) => { /* Expected */ }
            Ok(extracted) => assert_ne!(&extracted[..message.len().min(extracted.len())], message),
        }
    }

    #[test]
    fn test_zigzag_roundtrip() {
        // Verify that natural→zigzag conversion is correct
        let mut natural = [0i16; 64];
        for (i, item) in natural.iter_mut().enumerate() {
            *item = i as i16;
        }

        let zigzag = block_natural_to_zigzag(&natural);

        // DC should stay at position 0
        assert_eq!(zigzag[0], 0);
        // Position 1 in zigzag should be natural position 1
        assert_eq!(zigzag[1], 1);
        // Position 2 in zigzag should be natural position 8 (first in second row)
        assert_eq!(zigzag[2], 8);
    }

    #[test]
    #[should_panic(expected = "JPEG decoding failed")]
    fn extract_from_corrupt_jpeg_panics_with_decode_error() {
        let corrupt_data = b"not a valid jpeg at all";
        extract_from_jpeg(corrupt_data, None).unwrap();
    }

    #[test]
    fn extract_from_corrupt_jpeg_preserves_error_chain() {
        let corrupt_data = b"not a valid jpeg at all";
        let err = extract_from_jpeg(corrupt_data, None).unwrap_err();

        assert_eq!(err.to_string(), "JPEG decoding failed");

        // Verify the source error from the decoder is preserved
        let source = std::error::Error::source(&err).expect("should have source error");
        assert_eq!(
            source.to_string(),
            "invalid JPEG format: first two bytes are not an SOI marker"
        );
    }

    #[test]
    #[should_panic(expected = "JPEG decoding failed")]
    fn embed_in_corrupt_jpeg_panics_with_decode_error() {
        let corrupt_data = b"this is not jpeg data";
        embed_in_jpeg(corrupt_data, b"message", None).unwrap();
    }

    #[test]
    fn embed_in_corrupt_jpeg_preserves_error_chain() {
        let corrupt_data = b"this is not jpeg data";
        let err = embed_in_jpeg(corrupt_data, b"message", None).unwrap_err();

        assert_eq!(err.to_string(), "JPEG decoding failed");

        let source = std::error::Error::source(&err).expect("should have source error");
        assert_eq!(
            source.to_string(),
            "invalid JPEG format: first two bytes are not an SOI marker"
        );
    }

    #[test]
    #[should_panic(
        expected = "capacity exceeded"
    )]
    fn test_embed_capacity_exceeded_returns_error() {
        use crate::F5Encoder;

        let cover = VESSEL;
        let raw_capacity = jpeg_capacity(cover).unwrap();

        // Account for F5 header overhead when calculating usable capacity
        let usable_capacity = raw_capacity.saturating_sub(F5Encoder::HEADER_BYTES);

        // Message that fits within usable capacity should succeed
        let right_sized_message = vec![0xAB_u8; usable_capacity];
        assert!(embed_in_jpeg(cover, &right_sized_message, Some(b"seed")).is_ok(),
            "Message of {} bytes should fit in capacity of {} bytes",
            usable_capacity, raw_capacity);

        // Message that exceeds capacity should fail
        let oversized_message = vec![0xAB_u8; raw_capacity + 10000];
        embed_in_jpeg(cover, &oversized_message, Some(b"seed")).unwrap();
    }
}
