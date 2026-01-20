//! Keystore Encryption Operations
//!
//! This module provides AES-256-GCM encryption and decryption for keystore data.
//! Uses secure key derivation with SHA-256 and a salt.

use crate::error::{Result, SecurityError};

/// Encrypt data with password using AES-256-GCM
///
/// The password is hashed with SHA-256 and a salt to derive the encryption key.
/// A random 12-byte nonce is generated and prepended to the ciphertext.
pub fn encrypt_with_password(data: &[u8], password: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
    use sha2::{Digest, Sha256};

    // Derive key from password
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(b"vaughan-wallet-salt"); // Add salt
    let key_bytes = hasher.finalize();

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Generate random nonce
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt data
    let mut ciphertext = cipher.encrypt(nonce, data).map_err(|e| SecurityError::KeystoreError {
        message: format!("Encryption failed: {e}"),
    })?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.append(&mut ciphertext);

    Ok(result)
}

/// Decrypt data with password using AES-256-GCM
///
/// Expects the nonce to be prepended to the ciphertext (12 bytes).
pub fn decrypt_with_password(encrypted_data: &[u8], password: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
    use sha2::{Digest, Sha256};

    if encrypted_data.len() < 12 {
        return Err(SecurityError::KeystoreError {
            message: "Invalid encrypted data".to_string(),
        }
        .into());
    }

    // Extract nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Derive key from password
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(b"vaughan-wallet-salt");
    let key_bytes = hasher.finalize();

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Decrypt data
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| SecurityError::KeystoreError {
            message: format!("Decryption failed: {e}"),
        })?;

    Ok(plaintext)
}
