//! # Password Hashing
//! This little lib explores on

use argon2::{Argon2, ParamsBuilder};
use chacha20poly1305::aead::{Aead, AeadCore};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

pub mod error;
pub mod ffi;
pub mod ffi_utils;

pub use crate::error::SeasmokeError;

const NONCE_LEN: usize = 24;
const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32;

pub type Result<T> = std::result::Result<T, SeasmokeError>;
pub type Key = [u8; KEY_LEN];

/// decrypt data with password, it uses argon2id for key derivation and XChaCha20Poly1305 for encryption
pub fn decrypt_data(password: &str, data: &[u8]) -> Result<Vec<u8>> {
    assert!(data.len() >= SALT_LEN + NONCE_LEN, "data is too short");
    let salt = &data[data.len() - SALT_LEN..];
    let nonce = &data[data.len() - SALT_LEN - NONCE_LEN..data.len() - SALT_LEN];
    let key = derive_key(password.as_bytes(), salt)?;

    let decryptor = XChaCha20Poly1305::new(&key.into());
    let decipher_data = decryptor
        .decrypt(nonce.into(), &data[0..data.len() - SALT_LEN - NONCE_LEN])
        .map_err(SeasmokeError::DecryptionError)?;

    // todo write tests for enc and decrypt
    Ok(decipher_data)
}

/// encrypt data with password, it uses argon2id for key derivation and XChaCha20Poly1305 for encryption
pub fn encrypt_data(password: &str, data: &[u8]) -> Result<Vec<u8>> {
    // https://kerkour.com/rust-file-encryption-chacha20poly1305-argon2
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let key = derive_key(password.as_bytes(), &salt)?;

    let mut nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    assert!(nonce.len() == NONCE_LEN);

    let encryptor = XChaCha20Poly1305::new(&key.into());
    let mut cipher_data = encryptor
        .encrypt(&nonce, data)
        .map_err(SeasmokeError::EncryptionError)?;
    cipher_data.extend_from_slice(&nonce);
    cipher_data.extend_from_slice(&salt);

    nonce.zeroize();
    salt.zeroize();

    Ok(cipher_data)
}

fn default_secure_argon<'key>() -> Result<Argon2<'key>> {
    // increased time costs to make it more secure
    let params = ParamsBuilder::default()
        .t_cost(10)
        .output_len(32)
        .build()
        .map_err(SeasmokeError::KeyDerivationParamEarror)?;

    Ok(Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    ))
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    let mut output_key_material = [0u8; 32]; // Can be any desired size
    default_secure_argon()?
        .hash_password_into(password, salt, &mut output_key_material)
        .map_err(SeasmokeError::KeyDerivationError)?;

    Ok(output_key_material)
}

#[cfg(test)]
mod tests {
    use argon2::{password_hash::SaltString, PasswordHash, PasswordVerifier};

    use super::*;

    #[test]
    fn test_kye_derivation() {
        let password = b"hunter42"; // Bad password; don't actually use!
        let salt = rand::random::<[u8; 32]>();

        let mut output_key_material = [0u8; 32]; // Can be any desired size
        Argon2::default()
            .hash_password_into(password, &salt, &mut output_key_material)
            .unwrap();

        assert_ne!(salt, [0u8; 32]);
        assert_ne!(output_key_material, [0u8; 32]);
    }

    #[test]
    fn test_password_hash() {
        // Can be: `$argon2`, `$pbkdf2`, or `$scrypt`
        let hash_string = "$argon2i$v=19$m=65536,t=1,p=1$c29tZXNhbHQAAAAAAAAAAA$+r0d29hqEB0yasKr55ZgICsQGSkl0v0kgwhd+U3wyRo";
        let input_password = "password";

        let password_hash = PasswordHash::new(hash_string).expect("invalid password hash");

        // Trait objects for algorithms to support
        let algs: &[&dyn PasswordVerifier] = &[&Argon2::default()];

        password_hash
            .verify_password(algs, input_password)
            .expect("invalid password");
    }

    #[test]
    fn test_generate_a_password_hash() {
        let password = b"hunter42"; // Bad password; don't actually use!
        let mut rnd = rand::rngs::OsRng;
        let salt = SaltString::generate(&mut rnd);
        let password_hash = PasswordHash::generate(Argon2::default(), password, &salt).unwrap();

        assert_ne!(password_hash.to_string(), "");
    }

    #[test]
    fn test_encryption_round_trip() {
        let password = "resistance is futile";
        // some stupind data
        let data = b"lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        let cipher_data = encrypt_data(password, data).unwrap();
        let decipher_data = decrypt_data(password, &cipher_data).unwrap();

        assert_ne!(data, cipher_data.as_slice());
        assert_eq!(data, decipher_data.as_slice());
    }
}
