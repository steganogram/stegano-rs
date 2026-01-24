//! Huffman coding for JPEG entropy coding.
//!
//! Implements Huffman encoding and decoding for JPEG scan data.
//!
//! Adapted from:
//! - [jpeg-decoder](https://github.com/image-rs/jpeg-decoder) - decoding
//! - [jpeg-encoder](https://github.com/vstroebel/jpeg-encoder) - encoding

use super::parser::HuffmanTable;
use crate::error::{F5Error, Result};

/// Lookup table size (8-bit fast path).
const LUT_BITS: usize = 8;
const LUT_SIZE: usize = 1 << LUT_BITS;

/// Compiled Huffman table for fast decoding.
///
/// Uses a two-level approach:
/// 1. Fast LUT for codes ≤ 8 bits
/// 2. Linear search for longer codes
#[derive(Debug, Clone)]
pub struct HuffmanLookup {
    /// Fast lookup table: (symbol, code_length) for codes ≤ 8 bits.
    /// Entry is (0, 0) if code is longer than 8 bits.
    lut: [(u8, u8); LUT_SIZE],

    /// Huffman codes in order.
    codes: Vec<u16>,

    /// Code lengths in order.
    code_sizes: Vec<u8>,

    /// Symbol values in code-length order.
    values: Vec<u8>,
}

impl HuffmanLookup {
    /// Build lookup tables from a parsed Huffman table.
    pub fn from_table(table: &HuffmanTable) -> Result<Self> {
        // Generate Huffman codes from code lengths (JPEG spec Figure C.1)
        let (code_sizes, codes) = derive_huffman_codes(&table.code_lengths)?;

        let mut lookup = HuffmanLookup {
            lut: [(0, 0); LUT_SIZE],
            codes,
            code_sizes,
            values: table.values.clone(),
        };

        // Build fast LUT for codes ≤ 8 bits
        for (idx, (&code, &len)) in lookup.codes.iter().zip(lookup.code_sizes.iter()).enumerate() {
            if len as usize <= LUT_BITS {
                let symbol = lookup.values[idx];
                // Fill all LUT entries that match this code
                let shift = LUT_BITS - len as usize;
                let base = (code as usize) << shift;
                let fill_count = 1 << shift;
                for k in 0..fill_count {
                    lookup.lut[base + k] = (symbol, len);
                }
            }
        }

        Ok(lookup)
    }

    /// Get the symbol values slice.
    #[inline]
    pub fn values(&self) -> &[u8] {
        &self.values
    }
}

/// Compiled Huffman table for fast encoding.
///
/// Maps symbols to (code, length) pairs for O(1) encoding lookup.
#[derive(Debug, Clone)]
pub struct HuffmanEncoder {
    /// Encode lookup: symbol → (code, code_length).
    /// None if symbol is not in table.
    encode_map: [Option<(u16, u8)>; 256],
}

impl HuffmanEncoder {
    /// Build encoder lookup from a parsed Huffman table.
    pub fn from_table(table: &HuffmanTable) -> Result<Self> {
        let (code_sizes, codes) = derive_huffman_codes(&table.code_lengths)?;

        let mut encode_map = [None; 256];
        for (idx, (&code, &len)) in codes.iter().zip(code_sizes.iter()).enumerate() {
            let symbol = table.values[idx];
            encode_map[symbol as usize] = Some((code, len));
        }

        Ok(HuffmanEncoder { encode_map })
    }

    /// Get code and length for a symbol.
    #[inline]
    pub fn encode(&self, symbol: u8) -> Option<(u16, u8)> {
        self.encode_map[symbol as usize]
    }
}

/// Derive Huffman codes from code length counts.
///
/// Implements JPEG specification Figure C.1 and C.2.
fn derive_huffman_codes(code_lengths: &[u8; 16]) -> Result<(Vec<u8>, Vec<u16>)> {
    // Count total codes
    let total: usize = code_lengths.iter().map(|&n| n as usize).sum();
    if total > 256 {
        return Err(F5Error::InvalidCoefficients {
            reason: "Huffman table has more than 256 symbols".to_string(),
        });
    }

    // Build HUFFSIZE: list of code lengths
    let mut huffsize = Vec::with_capacity(total);
    for (len, &count) in code_lengths.iter().enumerate() {
        for _ in 0..count {
            huffsize.push((len + 1) as u8);
        }
    }

    // Build HUFFCODE: Huffman codes for each symbol
    let mut huffcode = Vec::with_capacity(total);
    let mut code: u32 = 0;
    let mut si = huffsize.first().copied().unwrap_or(0);

    for &size in &huffsize {
        while si < size {
            code <<= 1;
            si += 1;
        }
        if code >= (1u32 << size) {
            return Err(F5Error::InvalidCoefficients {
                reason: "invalid Huffman code (overflow)".to_string(),
            });
        }
        huffcode.push(code as u16);
        code += 1;
    }

    Ok((huffsize, huffcode))
}

/// Bit reader for entropy-coded data.
///
/// Handles:
/// - Bit-level reading from byte stream
/// - Restart markers (0xFFD0-0xFFD7)
/// - Byte stuffing (0xFF00 → 0xFF)
pub struct BitReader<'a> {
    /// Source data.
    data: &'a [u8],
    /// Current byte position.
    pos: usize,
    /// Bit buffer (holds up to 32 bits).
    bits: u32,
    /// Number of valid bits in buffer.
    num_bits: u8,
}

impl<'a> BitReader<'a> {
    /// Create a new bit reader.
    pub fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            pos: 0,
            bits: 0,
            num_bits: 0,
        }
    }

    /// Check if we've reached the end of data.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.data.len() && self.num_bits == 0
    }

    /// Get current byte position (for debugging).
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Get number of bits currently available in buffer.
    #[inline]
    pub fn num_bits_available(&self) -> u8 {
        self.num_bits
    }

    /// Peek at the next `count` bits without consuming them.
    #[inline]
    pub fn peek_bits(&mut self, count: u8) -> Result<u16> {
        while self.num_bits < count {
            let prev_bits = self.num_bits;
            self.fill_bits()?;
            // Check if we made progress - if not, we're at EOF
            if self.num_bits == prev_bits {
                return Err(F5Error::InvalidCoefficients {
                    reason: format!(
                        "unexpected end of data: need {} bits, have {}",
                        count, self.num_bits
                    ),
                });
            }
        }
        let shift = self.num_bits - count;
        let mask = (1u32 << count) - 1;
        Ok(((self.bits >> shift) & mask) as u16)
    }

    /// Consume `count` bits from the buffer.
    #[inline]
    pub fn consume_bits(&mut self, count: u8) {
        debug_assert!(count <= self.num_bits);
        self.num_bits -= count;
    }

    /// Read `count` bits and return as u16.
    #[inline]
    pub fn read_bits(&mut self, count: u8) -> Result<u16> {
        let value = self.peek_bits(count)?;
        self.consume_bits(count);
        Ok(value)
    }

    /// Fill bit buffer with more bytes.
    fn fill_bits(&mut self) -> Result<()> {
        while self.num_bits <= 24 && self.pos < self.data.len() {
            let byte = self.data[self.pos];
            self.pos += 1;

            if byte == 0xFF {
                // Handle marker or stuffed byte
                if self.pos < self.data.len() {
                    let next = self.data[self.pos];
                    match next {
                        0x00 => {
                            // Stuffed byte: 0xFF00 → 0xFF
                            self.pos += 1;
                            self.bits = (self.bits << 8) | 0xFF;
                            self.num_bits += 8;
                        }
                        0xD0..=0xD7 => {
                            // Restart marker - skip it
                            self.pos += 1;
                            // Reset bit buffer at restart
                            // (caller should handle DC predictor reset)
                        }
                        _ => {
                            // Other marker - treat as end of data
                            self.pos = self.data.len();
                            return Ok(());
                        }
                    }
                }
            } else {
                self.bits = (self.bits << 8) | (byte as u32);
                self.num_bits += 8;
            }
        }
        Ok(())
    }

    /// Decode one Huffman symbol.
    pub fn decode_huffman(&mut self, table: &HuffmanLookup) -> Result<u8> {
        // Try to fill buffer with at least 8 bits for LUT lookup
        let _ = self.fill_bits();

        // Fast path: try 8-bit LUT if we have enough bits
        if self.num_bits >= LUT_BITS as u8 {
            let peek = self.peek_bits(LUT_BITS as u8)?;
            let (symbol, len) = table.lut[peek as usize];

            if len > 0 {
                self.consume_bits(len);
                return Ok(symbol);
            }

            // Slow path: codes longer than 8 bits - linear search
            for (idx, (&code, &code_len)) in
                table.codes.iter().zip(table.code_sizes.iter()).enumerate()
            {
                if code_len as usize > LUT_BITS {
                    let peek_code = self.peek_bits(code_len)?;
                    if peek_code == code {
                        self.consume_bits(code_len);
                        return Ok(table.values[idx]);
                    }
                }
            }
        } else if self.num_bits > 0 {
            // End of stream: we have fewer than 8 bits available.
            // Try to match shorter codes by padding with 1s (JPEG convention).
            // The padded 1s should not match any valid code prefix since
            // JPEG uses all-1s padding specifically to avoid accidental matches.
            let available = self.num_bits;
            let peek = self.peek_bits(available)?;
            // Pad with 1s to make 8 bits for LUT lookup
            let padded = ((peek as usize) << (LUT_BITS - available as usize))
                | ((1usize << (LUT_BITS - available as usize)) - 1);
            let (symbol, len) = table.lut[padded];

            if len > 0 && len <= available {
                self.consume_bits(len);
                return Ok(symbol);
            }

            // Try linear search for codes that fit in available bits
            for (idx, (&code, &code_len)) in
                table.codes.iter().zip(table.code_sizes.iter()).enumerate()
            {
                if code_len <= available {
                    let peek_code = self.peek_bits(code_len)?;
                    if peek_code == code {
                        self.consume_bits(code_len);
                        return Ok(table.values[idx]);
                    }
                }
            }
        }

        Err(F5Error::InvalidCoefficients {
            reason: format!(
                "invalid Huffman code (bits available: {})",
                self.num_bits
            ),
        })
    }

    /// Read and sign-extend a value.
    ///
    /// JPEG uses a sign-magnitude representation where the first bit
    /// indicates sign (0 = negative, 1 = positive).
    pub fn receive_extend(&mut self, size: u8) -> Result<i16> {
        if size == 0 {
            return Ok(0);
        }

        let value = self.read_bits(size)? as i16;

        // Sign extension (JPEG spec Figure F.12)
        let vt = 1 << (size - 1);
        if value < vt {
            // Negative: extend sign
            Ok(value + (-1 << size) + 1)
        } else {
            Ok(value)
        }
    }
}

/// Bit writer for entropy-coded data.
///
/// Handles:
/// - Bit-level writing to byte stream
/// - Byte stuffing (0xFF → 0xFF 0x00)
/// - Padding to byte boundary
///
/// Adapted from [jpeg-encoder](https://github.com/vstroebel/jpeg-encoder).
pub struct BitWriter {
    /// Output buffer.
    data: Vec<u8>,
    /// Bit accumulator (holds up to 32 bits).
    bits: u32,
    /// Number of valid bits in accumulator.
    num_bits: u8,
}

impl BitWriter {
    /// Create a new bit writer.
    pub fn new() -> Self {
        BitWriter {
            data: Vec::new(),
            bits: 0,
            num_bits: 0,
        }
    }

    /// Create a new bit writer with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        BitWriter {
            data: Vec::with_capacity(capacity),
            bits: 0,
            num_bits: 0,
        }
    }

    /// Write `count` bits from `value`.
    ///
    /// Bits are written from MSB. For example, write_bits(0b101, 3)
    /// writes bits 1, 0, 1 in order.
    #[inline]
    pub fn write_bits(&mut self, value: u16, count: u8) {
        debug_assert!(count <= 16);

        // Add bits to accumulator
        self.bits = (self.bits << count) | (value as u32);
        self.num_bits += count;

        // Flush complete bytes
        while self.num_bits >= 8 {
            self.num_bits -= 8;
            let byte = (self.bits >> self.num_bits) as u8;
            self.write_byte(byte);
        }

        // Clear flushed bits
        self.bits &= (1u32 << self.num_bits) - 1;
    }

    /// Write a Huffman-encoded symbol.
    #[inline]
    pub fn write_huffman(&mut self, symbol: u8, table: &HuffmanEncoder) -> Result<()> {
        let (code, len) = table.encode(symbol).ok_or_else(|| F5Error::InvalidCoefficients {
            reason: format!("symbol {} not in Huffman table", symbol),
        })?;
        self.write_bits(code, len);
        Ok(())
    }

    /// Write a single byte with byte stuffing.
    fn write_byte(&mut self, byte: u8) {
        self.data.push(byte);
        if byte == 0xFF {
            // Byte stuffing: 0xFF → 0xFF 0x00
            self.data.push(0x00);
        }
    }

    /// Pad to byte boundary with 1 bits and flush.
    pub fn flush(&mut self) {
        if self.num_bits > 0 {
            // Pad with 1 bits (JPEG convention)
            let padding = 8 - self.num_bits;
            let value = (self.bits << padding) | ((1u32 << padding) - 1);
            self.write_byte(value as u8);
            self.num_bits = 0;
            self.bits = 0;
        }
    }

    /// Get the written data, consuming the writer.
    pub fn into_bytes(mut self) -> Vec<u8> {
        self.flush();
        self.data
    }

    /// Get the written data with debug info.
    pub fn into_bytes_debug(mut self) -> (Vec<u8>, u8) {
        let bits_before_flush = self.num_bits;
        self.flush();
        (self.data, bits_before_flush)
    }

    /// Get current length of written data.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if no data has been written.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.num_bits == 0
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the bit size (category) and additional bits for a coefficient.
///
/// Returns (size, bits) where:
/// - size: number of bits needed (0-11 for DC, 0-10 for AC)
/// - bits: additional bits to write after Huffman code
///
/// For positive values, bits = value.
/// For negative values, bits = value + 2^size - 1 (the complement representation).
///
/// This is the inverse of receive_extend.
#[inline]
pub fn encode_coefficient(value: i16) -> (u8, u16) {
    if value == 0 {
        return (0, 0);
    }

    let abs_value = value.unsigned_abs();
    let size = 16 - abs_value.leading_zeros() as u8;

    let bits = if value < 0 {
        // Negative: bits = value + 2^size - 1
        // This is equivalent to: (1 << size) - 1 - abs_value
        ((1u16 << size) - 1) - abs_value
    } else {
        abs_value as u16
    };

    (size, bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_decoder_consistency() {
        // Test that encoding and decoding are inverses
        let table = HuffmanTable {
            class: 0,
            id: 0,
            code_lengths: [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
            values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        };

        let encoder = HuffmanEncoder::from_table(&table).unwrap();
        let decoder = HuffmanLookup::from_table(&table).unwrap();

        // For each symbol, encode it and verify the decoder would decode it back
        for &symbol in &table.values {
            let (code, len) = encoder.encode(symbol).unwrap();

            // Simulate what the decoder would see
            // Pad the code to 8 bits for LUT lookup
            if len <= 8 {
                let padded = (code as usize) << (8 - len);
                let (decoded_symbol, decoded_len) = decoder.lut[padded];
                assert_eq!(decoded_symbol, symbol, "Symbol mismatch for {}", symbol);
                assert_eq!(decoded_len, len, "Length mismatch for symbol {}", symbol);
            }
        }
        println!("Encoder/decoder consistency check passed");
    }

    #[test]
    fn test_derive_huffman_codes() {
        // Valid Huffman table: 1 code of length 2, 1 code of length 3
        // This gives codes: 00, 010 (valid because we don't overflow)
        let code_lengths = [0u8, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let (sizes, codes) = derive_huffman_codes(&code_lengths).unwrap();

        assert_eq!(sizes, vec![2, 3]);
        assert_eq!(codes, vec![0b00, 0b010]);
    }

    #[test]
    fn test_derive_huffman_codes_standard_dc() {
        // Standard DC luminance table from JPEG spec
        let code_lengths = [0u8, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
        let (sizes, codes) = derive_huffman_codes(&code_lengths).unwrap();

        // Should have 12 codes total
        assert_eq!(sizes.len(), 12);
        assert_eq!(codes.len(), 12);

        // First code is length 2
        assert_eq!(sizes[0], 2);
    }

    #[test]
    fn test_huffman_lookup_build() {
        // Use standard DC luminance table from JPEG spec
        let table = HuffmanTable {
            class: 0,
            id: 0,
            code_lengths: [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
            values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        };

        let lookup = HuffmanLookup::from_table(&table).unwrap();
        assert_eq!(lookup.values.len(), 12);
    }

    #[test]
    fn test_bit_reader_basic() {
        let data = [0b10110100, 0b11001010];
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_bits(4).unwrap(), 0b1011);
        assert_eq!(reader.read_bits(4).unwrap(), 0b0100);
        assert_eq!(reader.read_bits(8).unwrap(), 0b11001010);
    }

    #[test]
    fn test_bit_reader_stuffed_byte() {
        // 0xFF followed by 0x00 should yield 0xFF
        let data = [0xFF, 0x00, 0x12];
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_bits(8).unwrap(), 0xFF);
        assert_eq!(reader.read_bits(8).unwrap(), 0x12);
    }

    #[test]
    fn test_receive_extend() {
        // Test JPEG sign extension (Figure F.12)
        // Data: bits from MSB = 1, 0, 0, 1, 1, 1, ...
        //       byte 0 = 0b10011100 = 0x9C
        let data = [0b10011100, 0b00000000];
        let mut reader = BitReader::new(&data);

        // Size 1, value 1 (bit=1) → 1 >= 1 → positive → +1
        assert_eq!(reader.receive_extend(1).unwrap(), 1);

        // Size 1, value 0 (bit=0) → 0 < 1 → negative → 0 + (-2) + 1 = -1
        assert_eq!(reader.receive_extend(1).unwrap(), -1);

        // Size 2, value 1 (bits=01) → 1 < 2 → negative → 1 + (-4) + 1 = -2
        assert_eq!(reader.receive_extend(2).unwrap(), -2);

        // Size 2, value 3 (bits=11) → 3 >= 2 → positive → +3
        assert_eq!(reader.receive_extend(2).unwrap(), 3);
    }

    #[test]
    fn test_receive_extend_zero() {
        let data = [0x00];
        let mut reader = BitReader::new(&data);

        // Size 0 should return 0
        assert_eq!(reader.receive_extend(0).unwrap(), 0);
    }

    #[test]
    fn test_encode_coefficient() {
        // Zero
        assert_eq!(encode_coefficient(0), (0, 0));

        // Positive values: bits = value
        assert_eq!(encode_coefficient(1), (1, 1)); // size=1, bits=1
        assert_eq!(encode_coefficient(2), (2, 2)); // size=2, bits=10
        assert_eq!(encode_coefficient(3), (2, 3)); // size=2, bits=11
        assert_eq!(encode_coefficient(7), (3, 7)); // size=3, bits=111

        // Negative values: bits = (2^size - 1) - abs(value)
        // -1: size=1, bits = (2-1) - 1 = 0
        assert_eq!(encode_coefficient(-1), (1, 0));
        // -2: size=2, bits = (4-1) - 2 = 1
        assert_eq!(encode_coefficient(-2), (2, 1));
        // -3: size=2, bits = (4-1) - 3 = 0
        assert_eq!(encode_coefficient(-3), (2, 0));
        // -7: size=3, bits = (8-1) - 7 = 0
        assert_eq!(encode_coefficient(-7), (3, 0));
        // -6: size=3, bits = (8-1) - 6 = 1
        assert_eq!(encode_coefficient(-6), (3, 1));
    }

    #[test]
    fn test_bit_writer_basic() {
        let mut writer = BitWriter::new();

        // Write 4 bits: 1011
        writer.write_bits(0b1011, 4);
        // Write 4 bits: 0100
        writer.write_bits(0b0100, 4);

        let data = writer.into_bytes();
        assert_eq!(data, vec![0b10110100]);
    }

    #[test]
    fn test_bit_writer_byte_stuffing() {
        let mut writer = BitWriter::new();

        // Write 0xFF - should be stuffed to 0xFF 0x00
        writer.write_bits(0xFF, 8);
        writer.write_bits(0x12, 8);

        let data = writer.into_bytes();
        assert_eq!(data, vec![0xFF, 0x00, 0x12]);
    }

    #[test]
    fn test_bit_writer_padding() {
        let mut writer = BitWriter::new();

        // Write 5 bits: 10110
        writer.write_bits(0b10110, 5);

        // Flush should pad with 1s to make 10110111
        let data = writer.into_bytes();
        assert_eq!(data, vec![0b10110111]);
    }

    #[test]
    fn test_huffman_encoder_build() {
        let table = HuffmanTable {
            class: 0,
            id: 0,
            code_lengths: [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
            values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        };

        let encoder = HuffmanEncoder::from_table(&table).unwrap();

        // Symbol 0 should have the first (shortest) code
        let (code, len) = encoder.encode(0).unwrap();
        assert_eq!(len, 2); // First code is 2 bits

        // All 12 symbols should be encodable
        for symbol in 0..12u8 {
            assert!(encoder.encode(symbol).is_some());
        }

        // Symbol 255 should not be encodable
        assert!(encoder.encode(255).is_none());
    }

    #[test]
    fn test_coefficient_roundtrip() {
        // Test that encoding and decoding match
        for value in -255i16..=255 {
            let (size, bits) = encode_coefficient(value);

            if value == 0 {
                assert_eq!(size, 0);
                continue;
            }

            // Simulate decode: receive_extend logic
            let vt = 1i16 << (size - 1);
            let decoded = if (bits as i16) < vt {
                (bits as i16) + ((-1i16) << size) + 1
            } else {
                bits as i16
            };

            assert_eq!(decoded, value, "roundtrip failed for {}", value);
        }
    }
}
