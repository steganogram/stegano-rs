pub use argon2::Error as Argon2Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SeasmokeError {
    #[error("Key derivation error")]
    KeyDerivationError(Argon2Error),

    #[error("Key derivation parameter error")]
    KeyDerivationParamEarror(Argon2Error),
}
