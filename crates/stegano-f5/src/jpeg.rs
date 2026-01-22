//! JPEG utilities for F5 steganography.
//!
//! This module provides low-level JPEG parsing utilities needed for F5 operations.
//! It handles marker parsing without requiring full image decoding.

use std::io::{Read, Seek, SeekFrom};

use crate::error::{F5Error, Result};

/// JPEG marker constants.
pub mod markers {
    pub const SOI: u8 = 0xD8; // Start of Image
    pub const EOI: u8 = 0xD9; // End of Image
    pub const SOS: u8 = 0xDA; // Start of Scan
    pub const DQT: u8 = 0xDB; // Define Quantization Table
    pub const DHT: u8 = 0xC4; // Define Huffman Table
    pub const SOF0: u8 = 0xC0; // Start of Frame (Baseline)
    pub const SOF2: u8 = 0xC2; // Start of Frame (Progressive)
}

/// A JPEG quantization table (8x8 = 64 values).
#[derive(Debug, Clone)]
pub struct QuantizationTable {
    /// Table ID (0-3).
    pub id: u8,
    /// Precision: 0 = 8-bit, 1 = 16-bit.
    pub precision: u8,
    /// Table values in natural (row-major) order.
    pub values: [u16; 64],
}

impl QuantizationTable {
    /// Get value at (row, col) position.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> u16 {
        self.values[row * 8 + col]
    }

    /// Format table as ASCII art for display.
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

        // Data rows
        for y in 0..8 {
            out.push_str(&format!("| y{} ", y));
            for x in 0..8 {
                out.push_str(&format!("| {:4} ", self.values[y * 8 + x]));
            }
            out.push_str("|\n");
        }

        out
    }
}

/// Parse quantization tables from a JPEG file.
///
/// Reads the file and extracts all DQT (Define Quantization Table) markers.
///
/// # Arguments
/// * `reader` - A reader positioned at the start of a JPEG file
///
/// # Returns
/// Vector of quantization tables found in the file.
pub fn parse_quantization_tables<R: Read + Seek>(reader: &mut R) -> Result<Vec<QuantizationTable>> {
    let mut tables = Vec::new();

    // Check JPEG magic bytes (SOI marker)
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xFF, markers::SOI] {
        return Err(F5Error::InvalidCoefficients {
            reason: "not a JPEG file (missing SOI marker)".to_string(),
        });
    }

    // Parse markers until we hit SOS or EOI
    loop {
        // Find next marker (0xFF followed by non-zero byte)
        let mut marker = [0u8; 2];
        if reader.read_exact(&mut marker).is_err() {
            break;
        }

        if marker[0] != 0xFF {
            continue;
        }

        match marker[1] {
            markers::DQT => {
                // Read segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes) as usize - 2;

                // Read segment data
                let mut data = vec![0u8; length];
                reader.read_exact(&mut data)?;

                // Parse tables from segment
                let mut pos = 0;
                while pos < data.len() {
                    let pq_tq = data[pos];
                    let precision = (pq_tq >> 4) & 0x0F;
                    let id = pq_tq & 0x0F;
                    pos += 1;

                    let mut values = [0u16; 64];
                    if precision == 0 {
                        // 8-bit precision
                        for i in 0..64 {
                            if pos < data.len() {
                                values[ZIGZAG_TO_NATURAL[i]] = data[pos] as u16;
                                pos += 1;
                            }
                        }
                    } else {
                        // 16-bit precision
                        for i in 0..64 {
                            if pos + 1 < data.len() {
                                values[ZIGZAG_TO_NATURAL[i]] =
                                    u16::from_be_bytes([data[pos], data[pos + 1]]);
                                pos += 2;
                            }
                        }
                    }

                    tables.push(QuantizationTable {
                        id,
                        precision,
                        values,
                    });
                }
            }
            markers::EOI | markers::SOS => break,
            0x00 => continue, // Stuffed byte
            _ => {
                // Skip other markers
                let mut length_bytes = [0u8; 2];
                if reader.read_exact(&mut length_bytes).is_ok() {
                    let length = u16::from_be_bytes(length_bytes) as i64 - 2;
                    if length > 0 {
                        reader.seek(SeekFrom::Current(length)).ok();
                    }
                }
            }
        }
    }

    Ok(tables)
}

/// JPEG zigzag order to natural (row-major) order mapping.
///
/// JPEG stores quantization table values in zigzag order.
/// This table maps zigzag index to natural index.
const ZIGZAG_TO_NATURAL: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_quantization_table_get() {
        let mut values = [0u16; 64];
        values[0] = 16; // (0,0)
        values[7] = 24; // (0,7)
        values[63] = 99; // (7,7)

        let table = QuantizationTable {
            id: 0,
            precision: 0,
            values,
        };

        assert_eq!(table.get(0, 0), 16);
        assert_eq!(table.get(0, 7), 24);
        assert_eq!(table.get(7, 7), 99);
    }

    #[test]
    fn test_parse_not_jpeg() {
        let data = b"not a jpeg file";
        let mut cursor = Cursor::new(data);

        let result = parse_quantization_tables(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_quantization_table_ascii() {
        let values = [1u16; 64];
        let table = QuantizationTable {
            id: 0,
            precision: 0,
            values,
        };

        let ascii = table.to_ascii_table();
        assert!(ascii.contains("x0"));
        assert!(ascii.contains("y0"));
        assert!(ascii.contains("1"));
    }
}
