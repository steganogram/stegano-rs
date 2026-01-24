//! JPEG parsing for F5 steganography.
//!
//! Extracts the components needed for coefficient-level manipulation:
//! - Quantization tables (DQT)
//! - Huffman tables (DHT)
//! - Frame info (SOF)
//! - Scan data (after SOS)
//!
//! Adapted from [jpeg-decoder](https://github.com/image-rs/jpeg-decoder).

use super::marker::Marker;
use crate::error::{F5Error, Result};
use std::io::{Read, Seek};

/// Zigzag order to natural (row-major) order mapping.
/// JPEG stores quantization/coefficient values in zigzag order.
pub const ZIGZAG_TO_NATURAL: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

/// Natural (row-major) order to zigzag order mapping.
pub const NATURAL_TO_ZIGZAG: [usize; 64] = [
    0, 1, 5, 6, 14, 15, 27, 28, 2, 4, 7, 13, 16, 26, 29, 42, 3, 8, 12, 17, 25, 30, 41, 43, 9, 11,
    18, 24, 31, 40, 44, 53, 10, 19, 23, 32, 39, 45, 52, 54, 20, 22, 33, 38, 46, 51, 55, 60, 21, 34,
    37, 47, 50, 56, 59, 61, 35, 36, 48, 49, 57, 58, 62, 63,
];

/// A JPEG quantization table (8x8 = 64 values).
#[derive(Debug, Clone)]
pub struct QuantizationTable {
    /// Table ID (0-3).
    pub id: u8,
    /// Precision: 0 = 8-bit, 1 = 16-bit.
    pub precision: u8,
    /// Table values in zigzag order (as stored in JPEG).
    pub values: [u16; 64],
}

impl QuantizationTable {
    /// Get value at zigzag index.
    #[inline]
    pub fn get_zigzag(&self, index: usize) -> u16 {
        self.values[index]
    }

    /// Get value at natural (row, col) position.
    #[inline]
    pub fn get_natural(&self, row: usize, col: usize) -> u16 {
        self.values[NATURAL_TO_ZIGZAG[row * 8 + col]]
    }

    /// Format table as ASCII art for display.
    ///
    /// Values are displayed in natural (row-major) order.
    pub fn to_ascii_table(&self) -> String {
        let mut out = String::new();

        // Header row
        out.push_str("|    |");
        for x in 0..8 {
            out.push_str(&format!("   x{} |", x));
        }
        out.push('\n');

        // Separator
        out.push_str("|----|");
        for _ in 0..8 {
            out.push_str("------|");
        }
        out.push('\n');

        // Data rows (convert from zigzag to natural order for display)
        for y in 0..8 {
            out.push_str(&format!("| y{} ", y));
            for x in 0..8 {
                let natural_idx = y * 8 + x;
                let zigzag_idx = NATURAL_TO_ZIGZAG[natural_idx];
                out.push_str(&format!("| {:4} ", self.values[zigzag_idx]));
            }
            out.push_str("|\n");
        }

        out
    }
}

/// Huffman table for encoding/decoding.
#[derive(Debug, Clone)]
pub struct HuffmanTable {
    /// Table class: 0 = DC, 1 = AC.
    pub class: u8,
    /// Table ID (0-3).
    pub id: u8,
    /// Number of codes of each length (1-16 bits).
    pub code_lengths: [u8; 16],
    /// Symbol values (up to 256).
    pub values: Vec<u8>,
}

/// JPEG component information.
#[derive(Debug, Clone)]
pub struct Component {
    /// Component ID.
    pub id: u8,
    /// Horizontal sampling factor.
    pub h_sampling: u8,
    /// Vertical sampling factor.
    pub v_sampling: u8,
    /// Quantization table ID to use.
    pub quant_table_id: u8,
    /// DC Huffman table ID (set during SOS parsing).
    pub dc_table_id: u8,
    /// AC Huffman table ID (set during SOS parsing).
    pub ac_table_id: u8,
}

/// Frame information from SOF marker.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// SOF type (0 = baseline, 2 = progressive, etc.).
    pub sof_type: u8,
    /// Sample precision (usually 8 bits).
    pub precision: u8,
    /// Image height in pixels.
    pub height: u16,
    /// Image width in pixels.
    pub width: u16,
    /// Components (Y, Cb, Cr for color JPEG).
    pub components: Vec<Component>,
}

impl FrameInfo {
    /// Check if this is a baseline DCT image (SOF0).
    pub fn is_baseline(&self) -> bool {
        self.sof_type == 0
    }

    /// Check if this is a progressive DCT image (SOF2).
    pub fn is_progressive(&self) -> bool {
        self.sof_type == 2
    }
}

/// Raw segment data with its marker.
#[derive(Debug, Clone)]
pub struct Segment {
    /// The marker type.
    pub marker: Marker,
    /// Raw segment data (excluding marker and length bytes).
    pub data: Vec<u8>,
}

/// Parsed JPEG structure containing all segments needed for transcoding.
#[derive(Debug, Clone)]
pub struct JpegSegments {
    /// All segments in order (for reconstruction).
    pub segments: Vec<Segment>,
    /// Parsed quantization tables (indexed by ID).
    pub quant_tables: [Option<QuantizationTable>; 4],
    /// Parsed DC Huffman tables (indexed by ID).
    pub dc_huff_tables: [Option<HuffmanTable>; 4],
    /// Parsed AC Huffman tables (indexed by ID).
    pub ac_huff_tables: [Option<HuffmanTable>; 4],
    /// Frame info from SOF marker.
    pub frame: Option<FrameInfo>,
    /// Restart interval (0 if not set).
    pub restart_interval: u16,
    /// Raw scan data (entropy-coded, after SOS header).
    pub scan_data: Vec<u8>,
    /// SOS header data (needed for reconstruction).
    pub sos_header: Vec<u8>,
}

impl Default for JpegSegments {
    fn default() -> Self {
        JpegSegments {
            segments: Vec::new(),
            quant_tables: [None, None, None, None],
            dc_huff_tables: [None, None, None, None],
            ac_huff_tables: [None, None, None, None],
            frame: None,
            restart_interval: 0,
            scan_data: Vec::new(),
            sos_header: Vec::new(),
        }
    }
}

/// Parse a JPEG file into its constituent segments.
///
/// # Arguments
/// * `data` - Complete JPEG file data
///
/// # Returns
/// Parsed segments and tables needed for F5 transcoding.
pub fn parse_jpeg(data: &[u8]) -> Result<JpegSegments> {
    let mut cursor = std::io::Cursor::new(data);
    parse_jpeg_reader(&mut cursor)
}

/// Parse a JPEG from a reader.
pub fn parse_jpeg_reader<R: Read + Seek>(reader: &mut R) -> Result<JpegSegments> {
    let mut segments = JpegSegments::default();

    // Check SOI marker
    let mut marker_bytes = [0u8; 2];
    reader.read_exact(&mut marker_bytes)?;
    if marker_bytes != [0xFF, 0xD8] {
        return Err(F5Error::InvalidCoefficients {
            reason: "not a JPEG file (missing SOI marker)".to_string(),
        });
    }

    // Parse segments until we hit SOS or EOI
    loop {
        let marker = read_marker(reader)?;

        match marker {
            Marker::EOI => break,

            Marker::SOS => {
                // Read SOS header
                let length = read_length(reader)?;
                let mut header = vec![0u8; length];
                reader.read_exact(&mut header)?;

                // Parse SOS header to get component table assignments
                parse_sos_header(&header, &mut segments)?;
                segments.sos_header = header;

                // Read scan data until EOI
                segments.scan_data = read_scan_data(reader)?;
                break;
            }

            Marker::DQT => {
                let length = read_length(reader)?;
                let mut data = vec![0u8; length];
                reader.read_exact(&mut data)?;

                // Parse and store quantization tables
                parse_dqt(&data, &mut segments)?;

                segments.segments.push(Segment {
                    marker,
                    data,
                });
            }

            Marker::DHT => {
                let length = read_length(reader)?;
                let mut data = vec![0u8; length];
                reader.read_exact(&mut data)?;

                // Parse and store Huffman tables
                parse_dht(&data, &mut segments)?;

                segments.segments.push(Segment {
                    marker,
                    data,
                });
            }

            Marker::SOF(n) => {
                let length = read_length(reader)?;
                let mut data = vec![0u8; length];
                reader.read_exact(&mut data)?;

                // Parse frame info
                segments.frame = Some(parse_sof(n, &data)?);

                segments.segments.push(Segment {
                    marker,
                    data,
                });
            }

            Marker::DRI => {
                let length = read_length(reader)?;
                let mut data = vec![0u8; length];
                reader.read_exact(&mut data)?;

                if data.len() >= 2 {
                    segments.restart_interval = u16::from_be_bytes([data[0], data[1]]);
                }

                segments.segments.push(Segment {
                    marker,
                    data,
                });
            }

            _ if marker.has_length() => {
                // Store other segments with length (APP, COM, etc.)
                let length = read_length(reader)?;
                let mut data = vec![0u8; length];
                reader.read_exact(&mut data)?;

                segments.segments.push(Segment {
                    marker,
                    data,
                });
            }

            _ => {
                // Markers without length (RST, etc.) - shouldn't appear before SOS
            }
        }
    }

    Ok(segments)
}

/// Parse quantization tables from a JPEG file.
///
/// This is a convenience function that extracts only the quantization tables
/// without parsing the full JPEG structure. Useful for inspection tools.
///
/// # Arguments
/// * `reader` - A reader positioned at the start of a JPEG file
///
/// # Returns
/// Vector of quantization tables found in the file.
pub fn parse_quantization_tables<R: Read + Seek>(reader: &mut R) -> Result<Vec<QuantizationTable>> {
    let segments = parse_jpeg_reader(reader)?;
    Ok(segments
        .quant_tables
        .into_iter()
        .flatten()
        .collect())
}

/// Read the next marker from the stream.
fn read_marker<R: Read>(reader: &mut R) -> Result<Marker> {
    let mut buf = [0u8; 1];

    // Find 0xFF
    loop {
        reader.read_exact(&mut buf)?;
        if buf[0] == 0xFF {
            break;
        }
    }

    // Skip fill bytes (0xFF), read marker byte
    loop {
        reader.read_exact(&mut buf)?;
        if buf[0] != 0xFF {
            break;
        }
    }

    Marker::from_u8(buf[0]).ok_or_else(|| F5Error::InvalidCoefficients {
        reason: format!("invalid marker byte: 0x{:02X}", buf[0]),
    })
}

/// Read segment length (2 bytes, big-endian, includes the 2 length bytes).
fn read_length<R: Read>(reader: &mut R) -> Result<usize> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    let length = u16::from_be_bytes(buf) as usize;
    if length < 2 {
        return Err(F5Error::InvalidCoefficients {
            reason: "segment length too small".to_string(),
        });
    }
    Ok(length - 2) // Subtract length field size
}

/// Read scan data until EOI or another marker.
///
/// Read entropy-coded scan data.
/// Preserves byte stuffing (0xFF 0x00) - BitReader handles de-stuffing.
/// Stops at any marker except RST (restart markers are included in scan data).
fn read_scan_data<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut buf = [0u8; 1];

    loop {
        if reader.read_exact(&mut buf).is_err() {
            break;
        }

        if buf[0] == 0xFF {
            data.push(0xFF);

            if reader.read_exact(&mut buf).is_err() {
                break;
            }

            match buf[0] {
                0x00 => {
                    // Stuffed byte: keep both bytes, BitReader handles de-stuffing
                    data.push(0x00);
                }
                0xD0..=0xD7 => {
                    // RST marker - include in scan data
                    data.push(buf[0]);
                }
                0xD9 => {
                    // EOI marker - end of scan (and image)
                    // Remove the 0xFF we pushed
                    data.pop();
                    break;
                }
                0xFF => {
                    // Fill byte - 0xFF is already pushed, continue
                    continue;
                }
                _ => {
                    // Any other marker ends the scan data
                    // Remove the 0xFF we pushed
                    data.pop();
                    break;
                }
            }
        } else {
            data.push(buf[0]);
        }
    }

    Ok(data)
}

/// Parse DQT (Define Quantization Table) segment.
fn parse_dqt(data: &[u8], segments: &mut JpegSegments) -> Result<()> {
    let mut pos = 0;

    while pos < data.len() {
        let pq_tq = data[pos];
        let precision = (pq_tq >> 4) & 0x0F;
        let id = pq_tq & 0x0F;
        pos += 1;

        if id > 3 {
            return Err(F5Error::InvalidCoefficients {
                reason: format!("invalid quantization table ID: {}", id),
            });
        }

        let mut values = [0u16; 64];
        if precision == 0 {
            // 8-bit precision
            for i in 0..64 {
                if pos >= data.len() {
                    return Err(F5Error::InvalidCoefficients {
                        reason: "DQT segment too short".to_string(),
                    });
                }
                values[i] = data[pos] as u16;
                pos += 1;
            }
        } else {
            // 16-bit precision
            for i in 0..64 {
                if pos + 1 >= data.len() {
                    return Err(F5Error::InvalidCoefficients {
                        reason: "DQT segment too short".to_string(),
                    });
                }
                values[i] = u16::from_be_bytes([data[pos], data[pos + 1]]);
                pos += 2;
            }
        }

        segments.quant_tables[id as usize] = Some(QuantizationTable {
            id,
            precision,
            values,
        });
    }

    Ok(())
}

/// Parse DHT (Define Huffman Table) segment.
fn parse_dht(data: &[u8], segments: &mut JpegSegments) -> Result<()> {
    let mut pos = 0;

    while pos < data.len() {
        let tc_th = data[pos];
        let class = (tc_th >> 4) & 0x0F; // 0 = DC, 1 = AC
        let id = tc_th & 0x0F;
        pos += 1;

        if class > 1 || id > 3 {
            return Err(F5Error::InvalidCoefficients {
                reason: format!("invalid Huffman table: class={}, id={}", class, id),
            });
        }

        // Read code lengths
        let mut code_lengths = [0u8; 16];
        if pos + 16 > data.len() {
            return Err(F5Error::InvalidCoefficients {
                reason: "DHT segment too short for code lengths".to_string(),
            });
        }
        code_lengths.copy_from_slice(&data[pos..pos + 16]);
        pos += 16;

        // Calculate total number of codes
        let total_codes: usize = code_lengths.iter().map(|&n| n as usize).sum();

        // Read symbol values
        if pos + total_codes > data.len() {
            return Err(F5Error::InvalidCoefficients {
                reason: "DHT segment too short for symbol values".to_string(),
            });
        }
        let values = data[pos..pos + total_codes].to_vec();
        pos += total_codes;

        let table = HuffmanTable {
            class,
            id,
            code_lengths,
            values,
        };

        if class == 0 {
            segments.dc_huff_tables[id as usize] = Some(table);
        } else {
            segments.ac_huff_tables[id as usize] = Some(table);
        }
    }

    Ok(())
}

/// Parse SOF (Start of Frame) segment.
fn parse_sof(sof_type: u8, data: &[u8]) -> Result<FrameInfo> {
    if data.len() < 6 {
        return Err(F5Error::InvalidCoefficients {
            reason: "SOF segment too short".to_string(),
        });
    }

    let precision = data[0];
    let height = u16::from_be_bytes([data[1], data[2]]);
    let width = u16::from_be_bytes([data[3], data[4]]);
    let num_components = data[5] as usize;

    if data.len() < 6 + num_components * 3 {
        return Err(F5Error::InvalidCoefficients {
            reason: "SOF segment too short for components".to_string(),
        });
    }

    let mut components = Vec::with_capacity(num_components);
    for i in 0..num_components {
        let offset = 6 + i * 3;
        let id = data[offset];
        let sampling = data[offset + 1];
        let quant_table_id = data[offset + 2];

        components.push(Component {
            id,
            h_sampling: (sampling >> 4) & 0x0F,
            v_sampling: sampling & 0x0F,
            quant_table_id,
            dc_table_id: 0,
            ac_table_id: 0,
        });
    }

    Ok(FrameInfo {
        sof_type,
        precision,
        height,
        width,
        components,
    })
}

/// Parse SOS (Start of Scan) header to get table assignments.
fn parse_sos_header(data: &[u8], segments: &mut JpegSegments) -> Result<()> {
    if data.is_empty() {
        return Err(F5Error::InvalidCoefficients {
            reason: "SOS header empty".to_string(),
        });
    }

    let num_components = data[0] as usize;
    if data.len() < 1 + num_components * 2 + 3 {
        return Err(F5Error::InvalidCoefficients {
            reason: "SOS header too short".to_string(),
        });
    }

    // Update component table assignments in frame info
    if let Some(ref mut frame) = segments.frame {
        for i in 0..num_components {
            let offset = 1 + i * 2;
            let component_id = data[offset];
            let table_ids = data[offset + 1];
            let dc_table = (table_ids >> 4) & 0x0F;
            let ac_table = table_ids & 0x0F;

            // Find component by ID and update
            for comp in frame.components.iter_mut() {
                if comp.id == component_id {
                    comp.dc_table_id = dc_table;
                    comp.ac_table_id = ac_table;
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_mapping() {
        // DC coefficient is at position 0 in both orders
        assert_eq!(ZIGZAG_TO_NATURAL[0], 0);
        assert_eq!(NATURAL_TO_ZIGZAG[0], 0);

        // Verify inverse relationship
        for i in 0..64 {
            assert_eq!(NATURAL_TO_ZIGZAG[ZIGZAG_TO_NATURAL[i]], i);
        }
    }

    #[test]
    fn test_parse_minimal_jpeg() {
        // Minimal valid JPEG: SOI + EOI
        let data = [0xFF, 0xD8, 0xFF, 0xD9];
        let result = parse_jpeg(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_not_jpeg() {
        let data = [0x00, 0x00, 0x00, 0x00];
        let result = parse_jpeg(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_baseline_jpeg() {
        // Test with baseline JPEG file (progressive not yet supported)
        let jpeg_data = include_bytes!("../../../../resources/NoSecrets.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Should have frame info
        assert!(segments.frame.is_some());
        let frame = segments.frame.as_ref().unwrap();
        assert!(frame.width > 0);
        assert!(frame.height > 0);
        assert!(frame.is_baseline(), "Expected baseline JPEG, got SOF type {}", frame.sof_type);

        // Should have quantization tables
        assert!(segments.quant_tables.iter().any(|t| t.is_some()));

        // Should have Huffman tables
        assert!(segments.dc_huff_tables.iter().any(|t| t.is_some()));
        assert!(segments.ac_huff_tables.iter().any(|t| t.is_some()));

        // Should have scan data
        assert!(!segments.scan_data.is_empty());

        // Print some debug info
        println!("Image: {}x{}", frame.width, frame.height);
        println!("Components: {}", frame.components.len());
        println!("Scan data size: {} bytes", segments.scan_data.len());
    }

    #[test]
    fn test_parse_progressive_jpeg_partial() {
        // Progressive JPEGs have multiple scans, we only get the first scan's data
        let jpeg_data = include_bytes!("../../../../resources/f5/tryout.jpg");
        let segments = parse_jpeg(jpeg_data).expect("Failed to parse JPEG");

        // Should have frame info
        assert!(segments.frame.is_some());
        let frame = segments.frame.as_ref().unwrap();
        assert!(frame.is_progressive(), "Expected progressive JPEG");

        // Should have quantization tables
        assert!(segments.quant_tables.iter().any(|t| t.is_some()));

        // Note: Progressive JPEGs may have Huffman tables defined later or use defaults
        // We don't require them to be present for this test
    }
}
