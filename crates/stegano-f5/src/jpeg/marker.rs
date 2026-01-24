//! JPEG marker definitions.
//!
//! Adapted from [jpeg-decoder](https://github.com/image-rs/jpeg-decoder)
//! to maintain compatibility and allow easy diffing against upstream.

/// JPEG marker types (ITU T.81 Table B.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Marker {
    /// Start of Frame (baseline, progressive, etc.)
    /// Parameter indicates the SOF type (0-15).
    SOF(u8),
    /// Reserved for JPEG extensions.
    JPG,
    /// Define Huffman Table.
    DHT,
    /// Define Arithmetic Coding conditioning.
    DAC,
    /// Restart marker (0-7).
    RST(u8),
    /// Start of Image.
    SOI,
    /// End of Image.
    EOI,
    /// Start of Scan.
    SOS,
    /// Define Quantization Table.
    DQT,
    /// Define Number of Lines.
    DNL,
    /// Define Restart Interval.
    DRI,
    /// Define Hierarchical Progression.
    DHP,
    /// Expand Reference Component.
    EXP,
    /// Application segment (0-15).
    APP(u8),
    /// JPEG extension (0-13).
    JPGn(u8),
    /// Comment.
    COM,
    /// Temporary marker for arithmetic coding.
    TEM,
    /// Reserved.
    RES,
}

impl Marker {
    /// Returns true if this marker has a length field following it.
    pub fn has_length(self) -> bool {
        !matches!(self, Marker::RST(..) | Marker::SOI | Marker::EOI | Marker::TEM)
    }

    /// Convert a byte to a Marker, if valid.
    ///
    /// Returns None for 0x00 (stuffed byte) and 0xFF (fill byte).
    pub fn from_u8(n: u8) -> Option<Marker> {
        use Marker::*;
        match n {
            0x00 => None, // Stuffed byte (escaped 0xFF)
            0x01 => Some(TEM),
            0x02..=0xBF => Some(RES),
            0xC0 => Some(SOF(0)),  // Baseline DCT
            0xC1 => Some(SOF(1)),  // Extended sequential DCT
            0xC2 => Some(SOF(2)),  // Progressive DCT
            0xC3 => Some(SOF(3)),  // Lossless (sequential)
            0xC4 => Some(DHT),
            0xC5 => Some(SOF(5)),  // Differential sequential DCT
            0xC6 => Some(SOF(6)),  // Differential progressive DCT
            0xC7 => Some(SOF(7)),  // Differential lossless (sequential)
            0xC8 => Some(JPG),
            0xC9 => Some(SOF(9)),  // Extended sequential DCT, arithmetic
            0xCA => Some(SOF(10)), // Progressive DCT, arithmetic
            0xCB => Some(SOF(11)), // Lossless (sequential), arithmetic
            0xCC => Some(DAC),
            0xCD => Some(SOF(13)), // Differential sequential DCT, arithmetic
            0xCE => Some(SOF(14)), // Differential progressive DCT, arithmetic
            0xCF => Some(SOF(15)), // Differential lossless, arithmetic
            0xD0..=0xD7 => Some(RST(n - 0xD0)),
            0xD8 => Some(SOI),
            0xD9 => Some(EOI),
            0xDA => Some(SOS),
            0xDB => Some(DQT),
            0xDC => Some(DNL),
            0xDD => Some(DRI),
            0xDE => Some(DHP),
            0xDF => Some(EXP),
            0xE0..=0xEF => Some(APP(n - 0xE0)),
            0xF0..=0xFD => Some(JPGn(n - 0xF0)),
            0xFE => Some(COM),
            0xFF => None, // Fill byte
        }
    }

    /// Convert marker back to its byte representation.
    pub fn to_u8(self) -> u8 {
        use Marker::*;
        match self {
            TEM => 0x01,
            RES => 0x02, // Note: RES covers 0x02-0xBF, return first
            SOF(n) => match n {
                0 => 0xC0,
                1 => 0xC1,
                2 => 0xC2,
                3 => 0xC3,
                5 => 0xC5,
                6 => 0xC6,
                7 => 0xC7,
                9 => 0xC9,
                10 => 0xCA,
                11 => 0xCB,
                13 => 0xCD,
                14 => 0xCE,
                15 => 0xCF,
                _ => 0xC0, // Default to baseline
            },
            JPG => 0xC8,
            DHT => 0xC4,
            DAC => 0xCC,
            RST(n) => 0xD0 + n,
            SOI => 0xD8,
            EOI => 0xD9,
            SOS => 0xDA,
            DQT => 0xDB,
            DNL => 0xDC,
            DRI => 0xDD,
            DHP => 0xDE,
            EXP => 0xDF,
            APP(n) => 0xE0 + n,
            JPGn(n) => 0xF0 + n,
            COM => 0xFE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_from_u8() {
        assert_eq!(Marker::from_u8(0xD8), Some(Marker::SOI));
        assert_eq!(Marker::from_u8(0xD9), Some(Marker::EOI));
        assert_eq!(Marker::from_u8(0xC0), Some(Marker::SOF(0)));
        assert_eq!(Marker::from_u8(0xC2), Some(Marker::SOF(2)));
        assert_eq!(Marker::from_u8(0xDA), Some(Marker::SOS));
        assert_eq!(Marker::from_u8(0xDB), Some(Marker::DQT));
        assert_eq!(Marker::from_u8(0xC4), Some(Marker::DHT));
        assert_eq!(Marker::from_u8(0xE0), Some(Marker::APP(0)));
        assert_eq!(Marker::from_u8(0x00), None); // Stuffed byte
        assert_eq!(Marker::from_u8(0xFF), None); // Fill byte
    }

    #[test]
    fn test_marker_to_u8() {
        assert_eq!(Marker::SOI.to_u8(), 0xD8);
        assert_eq!(Marker::EOI.to_u8(), 0xD9);
        assert_eq!(Marker::SOF(0).to_u8(), 0xC0);
        assert_eq!(Marker::SOF(2).to_u8(), 0xC2);
        assert_eq!(Marker::SOS.to_u8(), 0xDA);
        assert_eq!(Marker::DQT.to_u8(), 0xDB);
        assert_eq!(Marker::DHT.to_u8(), 0xC4);
        assert_eq!(Marker::APP(0).to_u8(), 0xE0);
    }

    #[test]
    fn test_has_length() {
        assert!(Marker::SOF(0).has_length());
        assert!(Marker::DQT.has_length());
        assert!(Marker::DHT.has_length());
        assert!(Marker::SOS.has_length());
        assert!(!Marker::SOI.has_length());
        assert!(!Marker::EOI.has_length());
        assert!(!Marker::RST(0).has_length());
    }
}
