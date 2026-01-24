//! JPEG file writer for reassembling modified scan data.
//!
//! Takes parsed JPEG segments and new scan data to produce a complete JPEG file.
//!
//! Adapted from [jpeg-encoder](https://github.com/vstroebel/jpeg-encoder).

use super::marker::Marker;
use super::parser::JpegSegments;

/// Write a complete JPEG file from parsed segments and new scan data.
///
/// Preserves all original segments (APP markers, comments, quantization tables,
/// Huffman tables, etc.) and replaces only the scan data.
///
/// # Arguments
/// * `segments` - Parsed JPEG segments from the original file
/// * `new_scan_data` - New entropy-coded scan data (with byte stuffing)
///
/// # Returns
/// Complete JPEG file as a byte vector.
pub fn write_jpeg(segments: &JpegSegments, new_scan_data: &[u8]) -> Vec<u8> {
    // Estimate output size: original size is a good estimate
    let estimated_size = segments
        .segments
        .iter()
        .map(|s| s.data.len() + 4)
        .sum::<usize>()
        + new_scan_data.len()
        + 100;

    let mut output = Vec::with_capacity(estimated_size);

    // Write SOI marker
    output.push(0xFF);
    output.push(Marker::SOI.to_u8());

    // Write all segments except SOS (which we handle separately)
    for segment in &segments.segments {
        if segment.marker == Marker::SOS {
            // We'll write SOS with new scan data at the end
            continue;
        }

        write_marker(&mut output, segment.marker);

        if segment.marker.has_length() {
            // Write length (includes the 2-byte length field)
            let length = (segment.data.len() + 2) as u16;
            output.push((length >> 8) as u8);
            output.push(length as u8);
        }

        // Write segment data
        output.extend_from_slice(&segment.data);
    }

    // Write SOS marker and header
    write_sos_header(&mut output, segments);

    // Write new scan data
    output.extend_from_slice(new_scan_data);

    // Write EOI marker
    output.push(0xFF);
    output.push(Marker::EOI.to_u8());

    output
}

/// Write a marker to the output.
fn write_marker(output: &mut Vec<u8>, marker: Marker) {
    output.push(0xFF);
    output.push(marker.to_u8());
}

/// Write the SOS (Start of Scan) header.
///
/// Reconstructs the SOS header from the frame info and component data.
fn write_sos_header(output: &mut Vec<u8>, segments: &JpegSegments) {
    write_marker(output, Marker::SOS);

    let frame = match &segments.frame {
        Some(f) => f,
        None => return,
    };

    // SOS header length: 2 (length) + 1 (num_components) + 2*n (component specs) + 3 (spectral selection, etc.)
    let num_components = frame.components.len() as u8;
    let length = 6 + 2 * num_components as u16;

    output.push((length >> 8) as u8);
    output.push(length as u8);

    // Number of components in scan
    output.push(num_components);

    // Component specifications
    for component in &frame.components {
        output.push(component.id);
        // DC table (high nibble) and AC table (low nibble)
        output.push((component.dc_table_id << 4) | component.ac_table_id);
    }

    // Spectral selection start (Ss) - 0 for baseline
    output.push(0);
    // Spectral selection end (Se) - 63 for baseline
    output.push(63);
    // Successive approximation (Ah, Al) - 0 for baseline
    output.push(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpeg::{decode_scan, encode_scan, parse_jpeg};

    #[test]
    fn test_write_jpeg_roundtrip() {
        // Parse original JPEG
        let original_jpeg = include_bytes!("../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(original_jpeg).expect("Failed to parse JPEG");

        // Decode coefficients
        let coefficients = decode_scan(&segments).expect("Failed to decode scan");

        // Re-encode coefficients
        let new_scan_data = encode_scan(&coefficients, &segments).expect("Failed to encode scan");

        // Write new JPEG
        let output_jpeg = write_jpeg(&segments, &new_scan_data);

        // Verify it's a valid JPEG
        assert!(output_jpeg.starts_with(&[0xFF, 0xD8]), "Should start with SOI");
        assert!(output_jpeg.ends_with(&[0xFF, 0xD9]), "Should end with EOI");

        // Parse the output JPEG
        let segments2 = parse_jpeg(&output_jpeg).expect("Failed to parse output JPEG");

        // Decode and verify coefficients match
        let coefficients2 = decode_scan(&segments2).expect("Failed to decode output scan");

        assert_eq!(coefficients.total_blocks, coefficients2.total_blocks);
        assert_eq!(coefficients.data, coefficients2.data);

        println!("Original JPEG size: {} bytes", original_jpeg.len());
        println!("Output JPEG size: {} bytes", output_jpeg.len());
        println!(
            "Size difference: {} bytes ({:.1}%)",
            output_jpeg.len() as i64 - original_jpeg.len() as i64,
            100.0 * (output_jpeg.len() as f64 - original_jpeg.len() as f64)
                / original_jpeg.len() as f64
        );
    }

    #[test]
    fn test_write_jpeg_structure() {
        // Parse original JPEG
        let original_jpeg = include_bytes!("../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(original_jpeg).expect("Failed to parse JPEG");

        // Decode and re-encode
        let coefficients = decode_scan(&segments).expect("Failed to decode scan");
        let new_scan_data = encode_scan(&coefficients, &segments).expect("Failed to encode scan");
        let output_jpeg = write_jpeg(&segments, &new_scan_data);

        // Count markers in output
        let mut marker_count = 0;
        let mut i = 0;
        while i < output_jpeg.len() - 1 {
            if output_jpeg[i] == 0xFF && output_jpeg[i + 1] != 0x00 && output_jpeg[i + 1] != 0xFF {
                marker_count += 1;
            }
            i += 1;
        }

        println!("Output JPEG has {} markers", marker_count);
        assert!(marker_count >= 4, "Should have at least SOI, DQT, SOF, SOS, EOI markers");
    }
}
