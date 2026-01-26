use std::io::{Cursor, Read, Result};

/// A decoder that wraps F5 JPEG extraction to implement the Read trait.
///
/// This allows F5 extraction to use the same `Message::from_raw_data()`
/// API as LSB decoders, eliminating special-case branching in unveil logic.
pub struct F5JpegDecoder {
    inner: Cursor<Vec<u8>>,
}

impl F5JpegDecoder {
    /// Create a new F5 decoder from JPEG source bytes.
    ///
    /// # Arguments
    /// * `jpeg_source` - The raw JPEG bytes containing F5-embedded data
    /// * `seed` - Optional seed for permutation (derived from password)
    pub fn decode(
        jpeg_source: &[u8],
        seed: Option<&[u8]>,
    ) -> std::result::Result<Self, crate::SteganoError> {
        let extracted = stegano_f5::extract_from_jpeg(jpeg_source, seed).map_err(|e| {
            crate::SteganoError::JpegError {
                reason: e.to_string(),
            }
        })?;

        Ok(Self {
            inner: Cursor::new(extracted),
        })
    }
}

impl Read for F5JpegDecoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}
