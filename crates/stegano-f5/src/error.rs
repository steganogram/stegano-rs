//! Error types for F5 steganography operations.

use std::fmt;
use thiserror::Error;

/// Result type alias for F5 operations.
pub type Result<T> = std::result::Result<T, F5Error>;

/// Errors that can occur during F5 encoding/decoding.
#[derive(Error)]
pub enum F5Error {
    /// Message length exceeds the maximum supported (2^28 - 1 bytes).
    #[error("message length {message_len} exceeds maximum of 268435455 bytes")]
    ExceedsMaxMessageLength { message_len: usize },

    /// Message is too large for the available carrier capacity.
    #[error("capacity exceeded: message requires {required} bytes but only {available} available")]
    CapacityExceeded { required: usize, available: usize },

    /// Invalid w parameter found during decoding (must be 1-9).
    #[error("invalid w parameter in header: {w} (must be 1-9)")]
    InvalidWParameter { w: u8 },

    /// Not enough coefficients for the reported message length (corrupted data or wrong password).
    #[error("insufficient coefficients: need {message_len} bytes but only {coefficient_count} coefficients")]
    InsufficientCoefficientsForLength {
        message_len: usize,
        coefficient_count: usize,
    },

    /// Not enough usable coefficients to decode the F5 header.
    #[error("insufficient coefficients for F5 header")]
    InsufficientCoefficientsForHeader,

    /// Not enough usable coefficients to decode the message.
    #[error("insufficient coefficients for message extraction")]
    InsufficientCoefficientsForMessage,

    /// JPEG decoding failed.
    #[error("JPEG decoding failed")]
    JpegDecodeFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// JPEG encoding failed.
    #[error("JPEG encoding failed")]
    JpegEncodeFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// F5 embedding failed during JPEG encoding.
    #[error("{0}")]
    EmbeddingFailed(String),

    /// I/O error during bit operations.
    #[error("bit I/O error: {0}")]
    BitIo(#[from] std::io::Error),
}

impl fmt::Debug for F5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Display for Debug so unwrap() shows user-friendly messages
        write!(f, "{self}")
    }
}
