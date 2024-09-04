pub use argon2::Error as Argon2Error;
pub use chacha20poly1305::Error as Chacha20Poly1305Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SeasmokeError {
    #[error("Key derivation error")]
    KeyDerivationError(Argon2Error),

    #[error("Key derivation parameter error")]
    KeyDerivationParamEarror(Argon2Error),

    #[error("Decryption error")]
    DecryptionError(Chacha20Poly1305Error),

    #[error("Encryption error")]
    EncryptionError(Chacha20Poly1305Error),
}
