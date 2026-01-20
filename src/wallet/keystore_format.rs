//! MetaMask-compatible keystore format
//!
//! This module defines the keystore format used by Vaughan wallet, which is
//! compatible with MetaMask v3 standard. This allows interoperability
//! with other Ethereum wallets that use the same format.
//!
//! ## Keystore Format
//!
//! The keystore is a JSON file containing encrypted wallet data with the
//! following structure (MetaMask v3 standard):
//!
//! ```json
//! {
//!   "version": 3,
//!   "id": "uuid-v4",
//!   "address": "0x...",
//!   "crypto": {
//!     "cipher": "aes-256-ctr",
//!     "ciphertext": "hex-encoded",
//!     "cipherparams": {
//!       "iv": "hex-encoded"
//!     },
//!     "kdf": "pbkdf2",
//!     "kdfparams": {
//!       "dklen": 32,
//!       "prf": "hmac-sha256",
//!       "salt": "hex-encoded"
//!     },
//!     "mac": "hex-encoded"
//!   },
//!   "timestamp": 1234567890
//! }
//! ```
//!
//! ## Security
//!
//! This keystore format uses:
//! - **PBKDF2** with 262,144 iterations for key derivation (same as MetaMask)
//! - **AES-256-CTR** for encryption (same as MetaMask)
//! - **SHA256** for MAC verification
//!
//! # Inspiration
//!
//! This keystore format is based on MetaMask v3 standard:
//! https://github.com/ethereum/wiki/wiki/Web3-Secret-Storage-Definition
//!
//! While Alloy provides cryptographic primitives, the keystore format structure
//! follows the MetaMask/Web3 Secret Storage Definition for compatibility.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

/// MetaMask v3 keystore format
///
/// This struct represents the complete keystore file format that is
/// compatible with MetaMask and other Ethereum wallets.
///
/// # Fields
///
/// * `version` - Keystore version (must be 3 for MetaMask compatibility)
/// * `id` - Unique identifier (UUID v4)
/// * `address` - Ethereum wallet address
/// * `crypto` - Encrypted data section containing ciphertext and metadata
/// * `timestamp` - Optional Unix timestamp of creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskKeystore {
    /// Keystore version
    ///
    /// Must be 3 for MetaMask compatibility. Other versions may exist
    /// but v3 is the current standard.
    #[serde(rename = "version")]
    pub version: u8,

    /// Unique wallet identifier
    ///
    /// UUID v4 format for identifying this specific keystore file.
    /// This is useful for multiple wallets or backup identification.
    pub id: String,

    /// Wallet address
    ///
    /// The Ethereum address derived from the private key.
    /// Stored here for convenience and verification.
    #[serde(rename = "address")]
    pub address: Address,

    /// Encrypted data section
    ///
    /// Contains the ciphertext, encryption parameters, and MAC for
    /// verifying decryption integrity.
    #[serde(rename = "crypto")]
    pub crypto: CryptoSection,

    /// Timestamp of creation
    ///
    /// Optional Unix timestamp (seconds since epoch) indicating when
    /// this keystore was created. Not required by MetaMask format
    /// but useful for wallet management.
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

/// Encrypted data section
///
/// Contains all information needed to decrypt the private key:
/// ciphertext, cipher algorithm, and all cryptographic parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSection {
    /// Cipher algorithm
    ///
    /// The encryption algorithm used. MetaMask uses "aes-256-ctr".
    /// Other valid options include "aes-128-ctr".
    #[serde(rename = "cipher")]
    pub cipher: String,

    /// Encrypted ciphertext
    ///
    /// The private key data encrypted with the cipher algorithm.
    /// Encoded as hexadecimal string.
    #[serde(rename = "ciphertext")]
    pub ciphertext: String,

    /// Cipher parameters
    ///
    /// Parameters specific to the cipher algorithm, such as
    /// initialization vector (IV).
    #[serde(rename = "cipherparams")]
    pub cipherparams: CipherParams,

    /// Key derivation function
    ///
    /// The algorithm used to derive the encryption key from the password.
    /// MetaMask uses "pbkdf2".
    #[serde(rename = "kdf")]
    pub kdf: String,

    /// Key derivation function parameters
    ///
    /// Parameters specific to the KDF algorithm, including salt,
    /// iteration count, and other settings.
    #[serde(rename = "kdfparams")]
    pub kdfparams: KdfParams,

    /// Message Authentication Code
    ///
    /// SHA256 hash of (derived_key + ciphertext). Used to verify
    /// that the password is correct and data hasn't been tampered with.
    #[serde(rename = "mac")]
    pub mac: String,
}

/// Cipher parameters
///
/// Parameters for the cipher algorithm. Currently only supports
/// AES-CTR which requires an initialization vector (IV).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherParams {
    /// Initialization vector
    ///
    /// Random bytes used to initialize the cipher. Must be unique
    /// for each encryption. Encoded as hexadecimal string.
    #[serde(rename = "iv")]
    pub iv: String,
}

/// Key derivation function parameters (PBKDF2)
///
/// Parameters for PBKDF2 key derivation. These are standardized
/// by MetaMask for security and compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Salt for key derivation
    ///
    /// Random bytes added to password before hashing to prevent
    /// rainbow table attacks. Encoded as hexadecimal string.
    #[serde(rename = "salt")]
    pub salt: String,

    /// Derived key length
    ///
    /// Length of the derived key in bytes. MetaMask uses 32 bytes
    /// (256 bits) which is the standard for AES-256.
    #[serde(rename = "dklen")]
    pub dklen: u32,

    /// Pseudorandom function
    ///
    /// The hash function used by PBKDF2. MetaMask uses "hmac-sha256".
    #[serde(rename = "prf")]
    pub prf: String,

    /// Iteration count (PBKDF2)
    ///
    /// Number of PBKDF2 iterations. MetaMask uses 262144 iterations.
    /// Higher values increase security but slow down unlock.
    #[serde(rename = "c")]
    pub c: u32,
}

impl MetaMaskKeystore {
    /// Create a new empty keystore template
    ///
    /// This is useful for creating a keystore structure that will be
    /// filled in with actual encrypted data.
    pub fn new() -> Self {
        Self {
            version: 3,
            id: uuid::Uuid::new_v4().to_string(),
            address: Address::ZERO,
            crypto: CryptoSection {
                cipher: "aes-256-ctr".to_string(),
                ciphertext: String::new(),
                cipherparams: CipherParams { iv: String::new() },
                kdf: "pbkdf2".to_string(),
                kdfparams: KdfParams {
                    salt: String::new(),
                    dklen: 32,
                    prf: "hmac-sha256".to_string(),
                    c: 262144, // MetaMask standard iteration count
                },
                mac: String::new(),
            },
            timestamp: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        }
    }
}

impl Default for MetaMaskKeystore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_structure() {
        // Test that keystore structure is correct and serializable
        let keystore = MetaMaskKeystore::new();

        assert_eq!(keystore.version, 3);
        assert!(!keystore.id.is_empty());
        assert_eq!(keystore.crypto.cipher, "aes-256-ctr");
        assert_eq!(keystore.crypto.kdf, "pbkdf2");
        assert_eq!(keystore.crypto.kdfparams.dklen, 32);
        assert_eq!(keystore.crypto.kdfparams.prf, "hmac-sha256");
    }

    #[test]
    fn test_keystore_serialization() {
        // Test JSON serialization/deserialization
        let keystore = MetaMaskKeystore::new();

        // Serialize to JSON
        let json = serde_json::to_string(&keystore).unwrap();
        assert!(json.contains(r#""version":3"#));
        assert!(json.contains(r#""cipher":"aes-256-ctr""#));
        assert!(json.contains(r#""kdf":"pbkdf2""#));

        // Deserialize from JSON
        let deserialized: MetaMaskKeystore = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, keystore.version);
        assert_eq!(deserialized.id, keystore.id);
    }

    #[test]
    fn test_crypto_section_structure() {
        // Test that crypto section has correct structure
        let crypto = CryptoSection {
            cipher: "aes-256-ctr".to_string(),
            ciphertext: "test".to_string(),
            cipherparams: CipherParams {
                iv: "testiv".to_string(),
            },
            kdf: "pbkdf2".to_string(),
            kdfparams: KdfParams {
                salt: "testsalt".to_string(),
                dklen: 32,
                prf: "hmac-sha256".to_string(),
                c: 262144,
            },
            mac: "testmac".to_string(),
        };

        assert_eq!(crypto.cipher, "aes-256-ctr");
        assert_eq!(crypto.kdf, "pbkdf2");
        assert_eq!(crypto.kdfparams.dklen, 32);
    }
}
