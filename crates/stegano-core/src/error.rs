use std::string::FromUtf8Error;
use thiserror::Error;
use zip::result::ZipError;

pub use stegano_seasmoke::SeasmokeError;

#[derive(Error, Debug)]
pub enum SteganoError {
    /// Represents an unsupported carrier media. For example, a Movie file is not supported
    #[error("Media format is not supported")]
    UnsupportedMedia,

    /// Represents an invalid carrier audio media. For example, a broken WAV file
    #[error("Audio media is invalid")]
    InvalidAudioMedia,

    /// Represents an invalid carrier image media. For example, a broken PNG file
    #[error("Image media is invalid")]
    InvalidImageMedia,

    /// Represents an unsupported message format version, for example foreign formats or just data crap
    #[error("Unsupported message format version: {0}")]
    UnsupportedMessageFormat(u8),

    /// Represents the error of invalid UTF-8 text data found inside of a text only message
    #[error("Invalid text data found inside a message")]
    InvalidTextData(#[from] FromUtf8Error),

    /// Represents an unveil of no secret data. For example when a media did not contain any secrets
    #[error("No secret data found")]
    NoSecretData,

    /// Represents an error caused by an invalid filename, for example not unsupported charset or empty filename
    #[error("A file with an invalid file name was provided")]
    InvalidFileName,

    /// Represents an error when interacting with the document message payload
    #[error("Error during the payload processing for documents")]
    PayloadProcessingError(#[from] ZipError),

    /// Represents a failure to read from input.
    #[error("Read error")]
    ReadError { source: std::io::Error },

    /// Represents a failure to write target file.
    #[error("Write error")]
    WriteError { source: std::io::Error },

    /// Represents a failure when encoding an audio file.
    #[error("Audio encoding error")]
    AudioEncodingError,

    /// Represents a failure when encoding an image file.
    #[error("Image encoding error")]
    ImageEncodingError,

    /// Represents a failure when creating an audio file.
    #[error("Audio creation error")]
    AudioCreationError,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Represents an error when encrypting the data
    #[error("Encryption error")]
    EncryptionError(SeasmokeError),

    /// Represents an error when decrypting the data
    #[error("Decryption error")]
    DecryptionError(SeasmokeError),

    #[error("No carrier media set")]
    CarrierNotSet,

    #[error("No target file set")]
    TargetNotSet,

    #[error(
"Capacity Error: The provided input image with the dimensions {0}x{1} is too small to accept all provided data.
                The image dimensions required are at least {2}x{3}"
    )]
    ImageCapacityError(usize, usize, usize, usize),

    #[error("API Error: Missing message")]
    MissingMessage,

    #[error("API Error: Missing files")]
    MissingFiles,
}
