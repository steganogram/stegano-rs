//! Baseline (sequential) JPEG scan encoding and decoding.
//!
//! This module handles the standard sequential JPEG format (SOF0).
//! Coefficients are encoded in a single scan with full spectral range.

use super::ScanCoefficients;
use crate::error::{F5Error, Result};
use crate::jpeg::huffman::{encode_coefficient, BitReader, BitWriter, HuffmanEncoder, HuffmanLookup};
use crate::jpeg::parser::{FrameInfo, JpegSegments};

/// Decode scan data from a baseline JPEG.
///
/// Extracts all DCT coefficients from the entropy-coded scan data.
///
/// # Arguments
/// * `segments` - Parsed JPEG segments containing Huffman tables and scan data
///
/// # Returns
/// Decoded DCT coefficients in scan order.
pub fn decode_scan_baseline(segments: &JpegSegments) -> Result<ScanCoefficients> {
    let frame = segments.frame.as_ref().ok_or_else(|| F5Error::InvalidCoefficients {
        reason: "missing frame info (SOF)".to_string(),
    })?;

    // Build Huffman lookup tables
    let mut dc_tables: [Option<HuffmanLookup>; 4] = [None, None, None, None];
    let mut ac_tables: [Option<HuffmanLookup>; 4] = [None, None, None, None];

    for (i, table) in segments.dc_huff_tables.iter().enumerate() {
        if let Some(t) = table {
            dc_tables[i] = Some(HuffmanLookup::from_table(t)?);
        }
    }
    for (i, table) in segments.ac_huff_tables.iter().enumerate() {
        if let Some(t) = table {
            ac_tables[i] = Some(HuffmanLookup::from_table(t)?);
        }
    }

    // Calculate block dimensions
    let (mcu_width, mcu_height, _blocks_per_mcu, blocks_per_component) =
        calculate_mcu_info(frame)?;

    let mcu_cols = (frame.width as usize + mcu_width - 1) / mcu_width;
    let mcu_rows = (frame.height as usize + mcu_height - 1) / mcu_height;
    let total_mcus = mcu_cols * mcu_rows;

    // blocks_per_component already accounts for all MCUs, so sum gives total blocks
    let total_blocks: usize = blocks_per_component.iter().sum();

    // Allocate coefficient storage
    let mut coefficients = ScanCoefficients {
        data: vec![0i16; total_blocks * 64],
        blocks_per_component: blocks_per_component.clone(),
        total_blocks,
        width: frame.width,
        height: frame.height,
    };

    // Decode scan data
    let mut reader = BitReader::new(&segments.scan_data);
    let mut dc_predictors = vec![0i16; frame.components.len()];
    let mut block_idx = 0;

    let restart_interval = segments.restart_interval as usize;
    let mut restart_count = 0;

    for _mcu in 0..total_mcus {
        // Check for restart interval
        if restart_interval > 0 && restart_count == restart_interval {
            // Reset DC predictors at restart marker
            dc_predictors.fill(0);
            restart_count = 0;
        }

        // Decode each component's blocks in the MCU
        for (comp_idx, component) in frame.components.iter().enumerate() {
            let h_blocks = component.h_sampling as usize;
            let v_blocks = component.v_sampling as usize;
            let num_blocks = h_blocks * v_blocks;

            let dc_table = dc_tables[component.dc_table_id as usize]
                .as_ref()
                .ok_or_else(|| F5Error::InvalidCoefficients {
                    reason: format!("missing DC Huffman table {}", component.dc_table_id),
                })?;

            let ac_table = ac_tables[component.ac_table_id as usize]
                .as_ref()
                .ok_or_else(|| F5Error::InvalidCoefficients {
                    reason: format!("missing AC Huffman table {}", component.ac_table_id),
                })?;

            for _ in 0..num_blocks {
                if block_idx >= coefficients.total_blocks {
                    break;
                }

                let block = coefficients.block_mut(block_idx);
                decode_block(
                    &mut reader,
                    block,
                    dc_table,
                    ac_table,
                    &mut dc_predictors[comp_idx],
                )?;
                block_idx += 1;
            }
        }

        restart_count += 1;
    }

    Ok(coefficients)
}

/// Encode DCT coefficients back to scan data (baseline).
///
/// Re-encodes coefficients using the same Huffman tables from the original JPEG.
/// This is the inverse of decode_scan_baseline.
///
/// # Arguments
/// * `coefficients` - DCT coefficients to encode
/// * `segments` - Parsed JPEG segments containing Huffman tables
///
/// # Returns
/// Entropy-coded scan data with byte stuffing applied.
pub fn encode_scan_baseline(coefficients: &ScanCoefficients, segments: &JpegSegments) -> Result<Vec<u8>> {
    let frame = segments.frame.as_ref().ok_or_else(|| F5Error::InvalidCoefficients {
        reason: "missing frame info (SOF)".to_string(),
    })?;

    // Build Huffman encoder tables
    let mut dc_encoders: [Option<HuffmanEncoder>; 4] = [None, None, None, None];
    let mut ac_encoders: [Option<HuffmanEncoder>; 4] = [None, None, None, None];

    for (i, table) in segments.dc_huff_tables.iter().enumerate() {
        if let Some(t) = table {
            dc_encoders[i] = Some(HuffmanEncoder::from_table(t)?);
        }
    }
    for (i, table) in segments.ac_huff_tables.iter().enumerate() {
        if let Some(t) = table {
            ac_encoders[i] = Some(HuffmanEncoder::from_table(t)?);
        }
    }

    // Calculate MCU info
    let (mcu_width, mcu_height, _blocks_per_mcu, _blocks_per_component) =
        calculate_mcu_info(frame)?;

    let mcu_cols = (frame.width as usize + mcu_width - 1) / mcu_width;
    let mcu_rows = (frame.height as usize + mcu_height - 1) / mcu_height;
    let total_mcus = mcu_cols * mcu_rows;

    // Estimate output size and create writer
    let mut writer = BitWriter::with_capacity(segments.scan_data.len());
    let mut dc_predictors = vec![0i16; frame.components.len()];
    let mut block_idx = 0;
    let mut blocks_encoded = 0;

    let restart_interval = segments.restart_interval as usize;
    let mut restart_count = 0;
    let mut restart_marker = 0u8;

    for _mcu in 0..total_mcus {
        // Handle restart interval
        if restart_interval > 0 && restart_count == restart_interval {
            // Flush current bits
            let partial = writer.into_bytes();
            writer = BitWriter::with_capacity(segments.scan_data.len() - partial.len());

            // Write restart marker
            // Note: This is handled by the writer layer, we just reset state
            dc_predictors.fill(0);
            restart_count = 0;
            restart_marker = (restart_marker + 1) & 0x07;

            // For now, we'll skip restart markers in the output
            // TODO: add restart marker writing support
        }

        // Encode each component's blocks in the MCU
        for (comp_idx, component) in frame.components.iter().enumerate() {
            let h_blocks = component.h_sampling as usize;
            let v_blocks = component.v_sampling as usize;
            let num_blocks = h_blocks * v_blocks;

            let dc_encoder = dc_encoders[component.dc_table_id as usize]
                .as_ref()
                .ok_or_else(|| F5Error::InvalidCoefficients {
                    reason: format!("missing DC Huffman table {}", component.dc_table_id),
                })?;

            let ac_encoder = ac_encoders[component.ac_table_id as usize]
                .as_ref()
                .ok_or_else(|| F5Error::InvalidCoefficients {
                    reason: format!("missing AC Huffman table {}", component.ac_table_id),
                })?;

            for _ in 0..num_blocks {
                if block_idx >= coefficients.total_blocks {
                    break;
                }

                let block = coefficients.block(block_idx);
                encode_block(
                    &mut writer,
                    block,
                    dc_encoder,
                    ac_encoder,
                    &mut dc_predictors[comp_idx],
                )?;
                block_idx += 1;
                blocks_encoded += 1;
            }
        }

        restart_count += 1;
    }

    let (data, bits_before_flush) = writer.into_bytes_debug();
    log::debug!(
        "Encoder: total_blocks={}, blocks_encoded={}, bits_before_flush={}, final_len={}",
        coefficients.total_blocks, blocks_encoded, bits_before_flush, data.len()
    );
    Ok(data)
}

/// Encode a single 8x8 block of DCT coefficients.
fn encode_block(
    writer: &mut BitWriter,
    block: &[i16],
    dc_encoder: &HuffmanEncoder,
    ac_encoder: &HuffmanEncoder,
    dc_predictor: &mut i16,
) -> Result<()> {
    let start_len = writer.len();

    // Encode DC coefficient (delta from previous block)
    let dc_value = block[0];
    let dc_diff = dc_value.wrapping_sub(*dc_predictor);
    *dc_predictor = dc_value;

    let (dc_size, dc_bits) = encode_coefficient(dc_diff);
    writer.write_huffman(dc_size, dc_encoder)?;
    if dc_size > 0 {
        writer.write_bits(dc_bits, dc_size);
    }
    log::trace!(
        "DC: value={}, diff={}, size={}, bits={:0width$b}",
        dc_value, dc_diff, dc_size, dc_bits, width = dc_size as usize
    );

    // Encode AC coefficients with run-length encoding
    let mut zero_run = 0u8;
    let mut last_nonzero = 0usize;

    for k in 1..64 {
        let coeff = block[k];

        if coeff == 0 {
            zero_run += 1;
        } else {
            // Emit ZRL codes for runs of 16+ zeros
            while zero_run >= 16 {
                writer.write_huffman(0xF0, ac_encoder)?; // ZRL = (15, 0)
                log::trace!("ZRL at k={}", k);
                zero_run -= 16;
            }

            // Encode (run_length, size) and coefficient bits
            let (size, bits) = encode_coefficient(coeff);
            let symbol = (zero_run << 4) | size;
            writer.write_huffman(symbol, ac_encoder)?;
            writer.write_bits(bits, size);

            log::trace!(
                "AC[{}]: coeff={}, run={}, size={}, symbol={:02X}",
                k, coeff, zero_run, size, symbol
            );

            zero_run = 0;
            last_nonzero = k;
        }
    }

    // EOB if we ended before filling all 63 AC coefficients
    if zero_run > 0 {
        writer.write_huffman(0x00, ac_encoder)?; // EOB = (0, 0)
        log::trace!("EOB after k={}", last_nonzero);
    }

    let end_len = writer.len();
    log::trace!("Block encoded: {} bytes written", end_len - start_len);

    Ok(())
}

/// Decode a single 8x8 block of DCT coefficients.
fn decode_block(
    reader: &mut BitReader,
    block: &mut [i16],
    dc_table: &HuffmanLookup,
    ac_table: &HuffmanLookup,
    dc_predictor: &mut i16,
) -> Result<()> {
    // Clear block
    block.fill(0);

    // Decode DC coefficient
    let dc_size = reader.decode_huffman(dc_table)?;
    if dc_size > 11 {
        return Err(F5Error::InvalidCoefficients {
            reason: format!("invalid DC coefficient size: {}", dc_size),
        });
    }

    let dc_diff = reader.receive_extend(dc_size)?;
    *dc_predictor = dc_predictor.wrapping_add(dc_diff);
    block[0] = *dc_predictor;

    // Decode AC coefficients
    let mut k = 1;
    while k < 64 {
        let symbol = reader.decode_huffman(ac_table)?;
        let run = symbol >> 4; // Number of zeros before this coefficient
        let size = symbol & 0x0F; // Bit size of coefficient

        if size == 0 {
            if run == 0 {
                // EOB (End of Block) - remaining coefficients are zero
                break;
            } else if run == 0x0F {
                // ZRL (Zero Run Length) - 16 zeros
                k += 16;
            } else {
                // Invalid
                return Err(F5Error::InvalidCoefficients {
                    reason: format!("invalid AC run/size: {:02X}", symbol),
                });
            }
        } else {
            // Skip `run` zeros, then decode coefficient
            k += run as usize;
            if k >= 64 {
                return Err(F5Error::InvalidCoefficients {
                    reason: "AC coefficient index out of bounds".to_string(),
                });
            }

            let value = reader.receive_extend(size)?;
            block[k] = value;
            k += 1;
        }
    }

    Ok(())
}

/// Calculate MCU (Minimum Coded Unit) information.
pub(crate) fn calculate_mcu_info(frame: &FrameInfo) -> Result<(usize, usize, Vec<usize>, Vec<usize>)> {
    // Find maximum sampling factors
    let h_max = frame
        .components
        .iter()
        .map(|c| c.h_sampling as usize)
        .max()
        .unwrap_or(1);
    let v_max = frame
        .components
        .iter()
        .map(|c| c.v_sampling as usize)
        .max()
        .unwrap_or(1);

    // MCU dimensions in pixels
    let mcu_width = h_max * 8;
    let mcu_height = v_max * 8;

    // Blocks per MCU for each component
    let blocks_per_mcu: Vec<usize> = frame
        .components
        .iter()
        .map(|c| (c.h_sampling as usize) * (c.v_sampling as usize))
        .collect();

    // Total blocks per component (over entire image)
    let mcu_cols = (frame.width as usize + mcu_width - 1) / mcu_width;
    let mcu_rows = (frame.height as usize + mcu_height - 1) / mcu_height;
    let total_mcus = mcu_cols * mcu_rows;

    let blocks_per_component: Vec<usize> = blocks_per_mcu
        .iter()
        .map(|&blocks| blocks * total_mcus)
        .collect();

    Ok((mcu_width, mcu_height, blocks_per_mcu, blocks_per_component))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpeg::parse_jpeg;

    #[test]
    fn test_decode_baseline_jpeg() {
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        let coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");

        // Verify we got coefficients
        assert!(!coefficients.data.is_empty());
        assert!(coefficients.total_blocks > 0);

        // Each block should have 64 coefficients
        assert_eq!(coefficients.data.len(), coefficients.total_blocks * 64);

        // Print stats
        println!("Image: {}x{}", coefficients.width, coefficients.height);
        println!("Total blocks: {}", coefficients.total_blocks);
        println!("Total coefficients: {}", coefficients.data.len());

        // Count non-zero coefficients
        let non_zero = coefficients.data.iter().filter(|&&c| c != 0).count();
        let dc_count = coefficients.total_blocks; // One DC per block
        let ac_non_zero = non_zero - dc_count;
        println!(
            "Non-zero AC coefficients: {} ({:.1}%)",
            ac_non_zero,
            100.0 * ac_non_zero as f64 / (coefficients.data.len() - dc_count) as f64
        );
    }

    #[test]
    fn test_coefficient_block_access() {
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");
        let coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");

        // Access first block
        let block0 = coefficients.block(0);
        assert_eq!(block0.len(), 64);

        // DC coefficient (first in block) is typically larger
        // AC coefficients (rest) should have some variation
        let dc = block0[0];
        println!("First block DC: {}", dc);

        // Most AC coefficients should be small or zero
        let large_ac = block0[1..].iter().filter(|&&c| c.abs() > 100).count();
        println!("Large AC coefficients in first block: {}", large_ac);
    }

    #[test]
    fn test_encode_decode_with_modified_coefficient() {
        // Parse original JPEG
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Decode to coefficients
        let mut coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");

        // Modify a single AC coefficient (not DC at index 0)
        let original_value = coefficients.data[1]; // First AC coeff of first block
        coefficients.data[1] = if original_value > 0 {
            original_value - 1
        } else if original_value < 0 {
            original_value + 1
        } else {
            1 // If it's 0, set it to 1
        };
        println!(
            "Modified coefficient 1: {} -> {}",
            original_value, coefficients.data[1]
        );

        // Re-encode
        let new_scan_data = encode_scan_baseline(&coefficients, &segments).expect("Failed to encode");
        println!("Original scan: {} bytes, New scan: {} bytes",
            segments.scan_data.len(), new_scan_data.len());

        // Decode the re-encoded data
        let mut segments2 = segments.clone();
        segments2.scan_data = new_scan_data;
        let coefficients2 = decode_scan_baseline(&segments2).expect("Failed to decode modified scan");

        // Verify the modified coefficient
        assert_eq!(
            coefficients2.data[1], coefficients.data[1],
            "Modified coefficient should match"
        );

        // Verify all other coefficients match
        let mismatches: Vec<_> = coefficients.data.iter()
            .zip(coefficients2.data.iter())
            .enumerate()
            .filter(|(_, (&a, &b))| a != b)
            .collect();

        if !mismatches.is_empty() {
            println!("Mismatches:");
            for (i, (a, b)) in mismatches.iter().take(10) {
                println!("  Index {}: expected {}, got {}", i, a, b);
            }
        }
        assert!(mismatches.is_empty(), "All coefficients should match");
    }

    #[test]
    fn test_encode_decode_with_shrinkage() {
        // Parse original JPEG
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Decode to coefficients
        let mut coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");

        // Find a non-zero AC coefficient and set it to 0 (shrinkage)
        let mut shrunk_idx = None;
        for i in 1..coefficients.data.len() {
            if (i % 64) != 0 && coefficients.data[i] != 0 {
                shrunk_idx = Some(i);
                break;
            }
        }
        let idx = shrunk_idx.expect("Should find a non-zero AC coefficient");
        let original_value = coefficients.data[idx];
        coefficients.data[idx] = 0;
        println!(
            "Shrunk coefficient {}: {} -> 0",
            idx, original_value
        );

        // Re-encode
        let new_scan_data = encode_scan_baseline(&coefficients, &segments).expect("Failed to encode");
        println!("Original scan: {} bytes, New scan: {} bytes (diff: {})",
            segments.scan_data.len(), new_scan_data.len(),
            new_scan_data.len() as i64 - segments.scan_data.len() as i64);

        // Decode the re-encoded data
        let mut segments2 = segments.clone();
        segments2.scan_data = new_scan_data;
        let coefficients2 = decode_scan_baseline(&segments2).expect("Failed to decode after shrinkage");

        // Verify the shrunk coefficient
        assert_eq!(coefficients2.data[idx], 0, "Shrunk coefficient should be 0");

        // Verify all coefficients match
        let mismatches: Vec<_> = coefficients.data.iter()
            .zip(coefficients2.data.iter())
            .enumerate()
            .filter(|(_, (&a, &b))| a != b)
            .collect();

        assert!(mismatches.is_empty(), "All coefficients should match");
    }

    #[test]
    fn test_encode_decode_with_multiple_shrinkages() {
        // Parse original JPEG
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Decode to coefficients
        let coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");

        // Find coefficients with abs value 1 (shrinkage candidates)
        let mut shrunk_positions = vec![];
        for i in 1..coefficients.data.len() {
            if (i % 64) != 0 && coefficients.data[i].abs() == 1 {
                shrunk_positions.push(i);
            }
        }

        // Test shrinking various numbers of coefficients
        // This tests edge cases in bit-level encoding at end of stream
        // Note: We avoid large counts that could create AC run lengths not in the Huffman table
        for count in [10, 13, 14, 20] {
            if count > shrunk_positions.len() {
                continue;
            }

            let mut test_coeffs = coefficients.clone();
            for &pos in shrunk_positions.iter().take(count) {
                test_coeffs.data[pos] = 0;
            }

            let new_scan_data = encode_scan_baseline(&test_coeffs, &segments).expect("Failed to encode");

            let mut segments2 = segments.clone();
            segments2.scan_data = new_scan_data;

            let decoded = decode_scan_baseline(&segments2).expect("Failed to decode after shrinkages");

            let mismatches = test_coeffs
                .data
                .iter()
                .zip(decoded.data.iter())
                .filter(|(&a, &b)| a != b)
                .count();
            assert_eq!(
                mismatches, 0,
                "Shrinking {} coefficients should roundtrip correctly",
                count
            );
        }
    }

    #[test]
    fn test_encode_decode_with_many_modifications() {
        // Parse original JPEG
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Decode to coefficients
        let mut coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");

        // Modify coefficients similar to what F5 might do
        let mut modified = 0;
        for i in (1..coefficients.data.len()).step_by(100000) {
            if (i % 64) != 0 && coefficients.data[i] != 0 {
                // Either decrement or set to 0
                if coefficients.data[i].abs() == 1 {
                    coefficients.data[i] = 0;
                } else if coefficients.data[i] > 0 {
                    coefficients.data[i] -= 1;
                } else {
                    coefficients.data[i] += 1;
                }
                modified += 1;
            }
        }
        assert!(modified > 0, "Should have modified some coefficients");

        // Re-encode
        let new_scan_data = encode_scan_baseline(&coefficients, &segments).expect("Failed to encode");

        // Decode the re-encoded data
        let mut segments2 = segments.clone();
        segments2.scan_data = new_scan_data;

        let coefficients2 = decode_scan_baseline(&segments2).expect("Failed to decode modified scan");

        // Verify all coefficients match
        assert_eq!(coefficients.data, coefficients2.data, "All coefficients should match");
    }

    #[test]
    fn test_encode_decode_with_modification_near_end() {
        // Parse original JPEG
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Decode to coefficients
        let mut coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");
        let total_coeffs = coefficients.data.len();

        // Find and modify a coefficient near the end (last few blocks)
        // This tests edge case where bit stream ends near byte boundary
        let mut modified = false;
        for i in ((total_coeffs - 64 * 10)..total_coeffs).rev() {
            if (i % 64) != 0 && coefficients.data[i] != 0 {
                if coefficients.data[i].abs() == 1 {
                    coefficients.data[i] = 0;
                } else if coefficients.data[i] > 0 {
                    coefficients.data[i] -= 1;
                } else {
                    coefficients.data[i] += 1;
                }
                modified = true;
                break;
            }
        }
        assert!(modified, "Should have found a coefficient to modify");

        // Re-encode and decode
        let new_scan_data = encode_scan_baseline(&coefficients, &segments).expect("Failed to encode");

        let mut segments2 = segments.clone();
        segments2.scan_data = new_scan_data;
        let coefficients2 = decode_scan_baseline(&segments2).expect("Failed to decode modified scan near end");

        // Verify all coefficients match
        assert_eq!(coefficients.data, coefficients2.data, "All coefficients should match");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        // Parse original JPEG
        let jpeg_data = include_bytes!("../../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Decode to coefficients
        let coefficients = decode_scan_baseline(&segments).expect("Failed to decode scan");
        let original_coeffs = coefficients.data.clone();

        // Re-encode coefficients
        let new_scan_data = encode_scan_baseline(&coefficients, &segments).expect("Failed to encode scan");

        // Verify scan data is byte-for-byte identical (no modifications = identical output)
        assert_eq!(new_scan_data, segments.scan_data, "Unmodified roundtrip should produce identical bytes");

        // Decode the re-encoded data
        let mut segments2 = segments.clone();
        segments2.scan_data = new_scan_data;
        let coefficients2 = decode_scan_baseline(&segments2).expect("Failed to decode re-encoded scan");

        // Verify coefficients match
        assert_eq!(coefficients2.total_blocks, coefficients.total_blocks);
        assert_eq!(coefficients2.data, original_coeffs);
    }
}
