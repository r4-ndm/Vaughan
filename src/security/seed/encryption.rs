//! Seed phrase encryption and decryption
//!
//! This module handles secure encryption and decryption of seed phrases
//! using AES-256-GCM with Argon2id or PBKDF2 key derivation.
//!
//! Security Design (inspired by MetaMask vault encryption):
//! - AES-256-GCM for authenticated encryption
//! - Argon2id for memory-hard key derivation (default)
//! - PBKDF2-SHA256 for compatibility with legacy data
//! - Random salt and nonce generation using getrandom

use crate::error::{Result, SecurityError};
use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use pbkdf2::pbkdf2_hmac;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

// ============================================================================
// Key Derivation Algorithm Types
// ============================================================================

/// Key derivation algorithm specification for enhanced encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivationAlgorithm {
    /// PBKDF2-SHA256 with specified iterations
    Pbkdf2Sha256 { iterations: u32 },
    /// Argon2id with memory, iterations, and parallelism parameters
    Argon2id {
        memory: u32,      // Memory usage in KiB
        iterations: u32,  // Number of iterations
        parallelism: u32, // Degree of parallelism
    },
}

impl Default for KeyDerivationAlgorithm {
    fn default() -> Self {
        Self::Pbkdf2Sha256 { iterations: 500_000 }
    }
}

/// Encryption algorithm specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM authenticated encryption
    #[default]
    Aes256Gcm,
    /// ChaCha20-Poly1305 authenticated encryption (future)
    ChaCha20Poly1305,
}

// ============================================================================
// Encrypted Data Structures
// ============================================================================

/// Encrypted seed phrase data structure (legacy format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSeedData {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; 32],
    pub nonce: [u8; 12],
    pub version: u32,
}

/// Enhanced encrypted seed phrase data structure with versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSeedDataV2 {
    /// Encrypted seed phrase data
    pub ciphertext: Vec<u8>,
    /// Salt used for key derivation
    pub salt: [u8; 32],
    /// Nonce/IV for encryption
    pub nonce: [u8; 12],
    /// Format version for future compatibility
    pub version: u32,
    /// Key derivation algorithm and parameters
    pub kdf_algorithm: KeyDerivationAlgorithm,
    /// Encryption algorithm used
    pub encryption_algorithm: EncryptionAlgorithm,
    /// Timestamp when encryption was performed
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Integrity hash of the encrypted data
    pub integrity_hash: [u8; 32],
    /// Additional authenticated data (optional)
    pub aad: Option<Vec<u8>>,
}

impl Default for EncryptedSeedDataV2 {
    fn default() -> Self {
        Self {
            ciphertext: Vec::new(),
            salt: [0u8; 32],
            nonce: [0u8; 12],
            version: 2,
            kdf_algorithm: KeyDerivationAlgorithm::default(),
            encryption_algorithm: EncryptionAlgorithm::default(),
            created_at: chrono::Utc::now(),
            integrity_hash: [0u8; 32],
            aad: None,
        }
    }
}

// ============================================================================
// Encryption Helper Functions
// ============================================================================

/// Derive encryption key from master password and salt (legacy method)
pub fn derive_encryption_key(master_password: &SecretString, salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        master_password.expose_secret().as_bytes(),
        salt,
        100_000, // 100k iterations for security
        &mut key,
    );

    Ok(key)
}

/// Enhanced key derivation using specified algorithm
pub fn derive_key_enhanced(
    master_password: &SecretString,
    salt: &[u8],
    algorithm: &KeyDerivationAlgorithm,
) -> Result<[u8; 32]> {
    match algorithm {
        KeyDerivationAlgorithm::Pbkdf2Sha256 { iterations } => {
            let mut key = [0u8; 32];
            pbkdf2_hmac::<Sha256>(master_password.expose_secret().as_bytes(), salt, *iterations, &mut key);
            Ok(key)
        }
        KeyDerivationAlgorithm::Argon2id {
            memory,
            iterations,
            parallelism,
        } => derive_key_argon2id(master_password, salt, *memory, *iterations, *parallelism),
    }
}

/// Derive key using Argon2id (memory-hard function)
pub fn derive_key_argon2id(
    master_password: &SecretString,
    salt: &[u8],
    memory: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; 32]> {
    // Create Argon2 configuration
    let params =
        Params::new(memory, iterations, parallelism, Some(32)).map_err(|e| SecurityError::KeyDerivationError {
            message: format!("Invalid Argon2 parameters: {e}"),
        })?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // Create salt string for Argon2
    let salt_string = SaltString::encode_b64(salt).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Failed to encode salt: {e}"),
    })?;

    // Hash the password
    let password_hash = argon2
        .hash_password(master_password.expose_secret().as_bytes(), &salt_string)
        .map_err(|e| SecurityError::KeyDerivationError {
            message: format!("Argon2 key derivation failed: {e}"),
        })?;

    // Extract the hash bytes
    let hash_bytes = password_hash.hash.ok_or_else(|| SecurityError::KeyDerivationError {
        message: "No hash in Argon2 result".to_string(),
    })?;

    // Convert to fixed-size array
    let mut key = [0u8; 32];
    let hash_slice = hash_bytes.as_bytes();
    key.copy_from_slice(&hash_slice[..32.min(hash_slice.len())]);

    Ok(key)
}

/// Calculate integrity hash for encrypted data
pub fn calculate_integrity_hash(ciphertext: &[u8], salt: &[u8], nonce: &[u8]) -> [u8; 32] {
    use sha2::Digest;

    let mut hasher = Sha256::new();
    hasher.update(ciphertext);
    hasher.update(salt);
    hasher.update(nonce);
    hasher.update(b"vaughan-seed-v2"); // Version tag

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

/// Generate a random salt for key derivation
pub fn generate_salt() -> Result<[u8; 32]> {
    let mut salt = [0u8; 32];
    getrandom::getrandom(&mut salt).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Failed to generate salt: {e}"),
    })?;
    Ok(salt)
}

/// Generate a random nonce for AES-GCM
pub fn generate_nonce() -> Result<[u8; 12]> {
    let mut nonce = [0u8; 12];
    getrandom::getrandom(&mut nonce).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Failed to generate nonce: {e}"),
    })?;
    Ok(nonce)
}

/// Decrypt seed phrase with AES-256-GCM (legacy format)
pub fn decrypt_seed_phrase(encrypted_data: &EncryptedSeedData, master_password: &SecretString) -> Result<SecretString> {
    // Derive encryption key using stored salt
    let key_bytes = derive_encryption_key(master_password, &encrypted_data.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    // Create cipher
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_data.nonce);

    // Decrypt the seed phrase
    let plaintext =
        cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| SecurityError::DecryptionError {
                message: format!("Failed to decrypt seed phrase: {e}"),
            })?;

    // Convert to SecretString
    let seed_phrase = String::from_utf8(plaintext).map_err(|e| SecurityError::DecryptionError {
        message: format!("Invalid UTF-8 in decrypted seed phrase: {e}"),
    })?;

    Ok(SecretString::new(seed_phrase))
}

/// Enhanced encryption with versioning and algorithm selection
pub fn encrypt_seed_phrase_v2(
    seed_phrase: &SecretString,
    master_password: &SecretString,
    kdf_algorithm: Option<KeyDerivationAlgorithm>,
    encryption_algorithm: Option<EncryptionAlgorithm>,
) -> Result<EncryptedSeedDataV2> {
    // Use default algorithms if none specified
    let kdf_alg = kdf_algorithm.unwrap_or_default();
    let enc_alg = encryption_algorithm.unwrap_or_default();

    // Generate salt and nonce
    let salt = generate_salt()?;
    let nonce_bytes = generate_nonce()?;

    // Derive encryption key using specified algorithm
    let key_bytes = derive_key_enhanced(master_password, &salt, &kdf_alg)?;

    // Encrypt based on algorithm
    let ciphertext = match enc_alg {
        EncryptionAlgorithm::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&nonce_bytes);

            cipher
                .encrypt(nonce, seed_phrase.expose_secret().as_bytes())
                .map_err(|e| SecurityError::EncryptionError {
                    message: format!("AES-256-GCM encryption failed: {e}"),
                })?
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            // TODO: Implement ChaCha20-Poly1305 encryption
            return Err(SecurityError::EncryptionError {
                message: "ChaCha20-Poly1305 not yet implemented".to_string(),
            }
            .into());
        }
    };

    // Calculate integrity hash
    let integrity_hash = calculate_integrity_hash(&ciphertext, &salt, &nonce_bytes);

    Ok(EncryptedSeedDataV2 {
        ciphertext,
        salt,
        nonce: nonce_bytes,
        version: 2,
        kdf_algorithm: kdf_alg,
        encryption_algorithm: enc_alg,
        created_at: chrono::Utc::now(),
        integrity_hash,
        aad: None,
    })
}

/// Enhanced decryption with version support
pub fn decrypt_seed_phrase_v2(
    encrypted_data: &EncryptedSeedDataV2,
    master_password: &SecretString,
) -> Result<SecretString> {
    // Verify integrity hash
    let calculated_hash =
        calculate_integrity_hash(&encrypted_data.ciphertext, &encrypted_data.salt, &encrypted_data.nonce);

    if calculated_hash != encrypted_data.integrity_hash {
        return Err(SecurityError::DecryptionError {
            message: "Integrity verification failed - data may be corrupted".to_string(),
        }
        .into());
    }

    // Derive decryption key using the stored algorithm
    let key_bytes = derive_key_enhanced(master_password, &encrypted_data.salt, &encrypted_data.kdf_algorithm)?;

    // Decrypt based on algorithm
    let plaintext = match &encrypted_data.encryption_algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&encrypted_data.nonce);

            cipher
                .decrypt(nonce, encrypted_data.ciphertext.as_ref())
                .map_err(|e| SecurityError::DecryptionError {
                    message: format!("AES-256-GCM decryption failed: {e}"),
                })?
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            // TODO: Implement ChaCha20-Poly1305 decryption
            return Err(SecurityError::DecryptionError {
                message: "ChaCha20-Poly1305 not yet implemented".to_string(),
            }
            .into());
        }
    };

    // Convert to SecretString
    let seed_phrase = String::from_utf8(plaintext).map_err(|e| SecurityError::DecryptionError {
        message: format!("Invalid UTF-8 in decrypted seed phrase: {e}"),
    })?;

    Ok(SecretString::new(seed_phrase))
}
