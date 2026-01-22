//! Error types for F5 steganography operations.

use thiserror::Error;

/// Result type alias for F5 operations.
pub type Result<T> = std::result::Result<T, F5Error>;

/// Errors that can occur during F5 encoding/decoding.
#[derive(Debug, Error)]
pub enum F5Error {
    /// Message is too large for the available capacity.
    #[error(
        "capacity exceeded: message requires {required} bytes but only {available} bytes available"
    )]
    CapacityExceeded {
        /// Bytes required to embed the message.
        required: usize,
        /// Bytes available in the carrier.
        available: usize,
    },

    /// No valid F5 data found in the coefficients.
    #[error("no valid F5 data found: {reason}")]
    NoDataFound {
        /// Reason why no data was found.
        reason: String,
    },

    /// Invalid encoding parameter.
    #[error("invalid parameter: {param} = {value}, {reason}")]
    InvalidParameter {
        /// Parameter name.
        param: &'static str,
        /// Invalid value.
        value: String,
        /// Reason why it's invalid.
        reason: String,
    },

    /// Coefficient array has invalid structure.
    #[error("invalid coefficient structure: {reason}")]
    InvalidCoefficients {
        /// Reason why coefficients are invalid.
        reason: String,
    },

    /// I/O error during bit operations.
    #[error("bit I/O error: {0}")]
    BitIo(#[from] std::io::Error),
}
