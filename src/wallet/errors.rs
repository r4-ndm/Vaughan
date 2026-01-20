//! Wallet manager error types
//!
//! This module provides comprehensive error handling for wallet operations,
//! following Rust best practices and ensuring user-friendly error messages.

use thiserror::Error;

/// Wallet manager error type
///
/// This enum covers all possible error cases for wallet operations,
/// providing clear, actionable error messages to users.
#[derive(Error, Debug)]
pub enum WalletManagerError {
    /// Keystore is locked
    ///
    /// Attempted to perform an operation that requires the wallet to be unlocked,
    /// but the keystore is currently locked.
    #[error("Keystore is locked. Please unlock the wallet first.")]
    KeystoreLocked,

    /// Invalid password
    ///
    /// The provided password was incorrect. This error is intentionally vague
    /// to prevent information leakage about which part of the authentication failed.
    #[error("Invalid password. Please try again.")]
    InvalidPassword,

    /// Account not found
    ///
    /// The requested account address does not exist in the wallet.
    #[error("Account not found: {address}")]
    AccountNotFound {
        /// The account address that was not found
        address: String,
    },

    /// Encryption failed
    ///
    /// An error occurred while encrypting data. This could be due to
    /// system issues, invalid parameters, or underlying crypto library errors.
    #[error("Encryption failed: {reason}")]
    EncryptionFailed {
        /// Detailed reason for the encryption failure
        reason: String,
    },

    /// Decryption failed
    ///
    /// An error occurred while decrypting data. This could be due to
    /// invalid ciphertext, wrong password, or underlying crypto library errors.
    #[error("Decryption failed: {reason}")]
    DecryptionFailed {
        /// Detailed reason for the decryption failure
        reason: String,
    },

    /// Invalid seed phrase
    ///
    /// The provided seed phrase (mnemonic) is invalid. This could be due to:
    /// - Wrong word count (not 12, 15, 18, 21, or 24)
    /// - Invalid word from BIP39 wordlist
    /// - Invalid checksum
    #[error("Invalid seed phrase: {reason}")]
    InvalidSeedPhrase {
        /// Detailed reason why the seed phrase is invalid
        reason: String,
    },

    /// IO error
    ///
    /// Wrapper for standard IO errors that occur during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    ///
    /// Wrapper for JSON serialization/deserialization errors that occur
    /// during keystore operations.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Keychain error
    ///
    /// Errors from the OS keychain integration (e.g., Keychain on macOS,
    /// Windows Credential Manager, or libsecret on Linux).
    #[error("Keychain error: {0}")]
    Keychain(String),

    /// Device not found
    ///
    /// Requested hardware wallet device not found or not connected.
    #[error("Hardware wallet device not found")]
    DeviceNotFound,

    /// Transaction signing failed
    ///
    /// An error occurred while signing a transaction.
    #[error("Transaction signing failed: {reason}")]
    SigningFailed {
        /// Detailed reason for the signing failure
        reason: String,
    },

    /// Network error
    ///
    /// An error occurred while communicating with the blockchain network.
    #[error("Network error: {reason}")]
    NetworkError {
        /// Detailed reason for the network failure
        reason: String,
    },
}

/// Type alias for Result with WalletManagerError
///
/// This provides a convenient shorthand for functions that return
/// WalletManagerError as their error type.
///
/// # Example
///
/// ```rust,ignore
/// use crate::wallet::errors::{WalletResult, WalletManagerError};
///
/// fn create_wallet() -> WalletResult<Address> {
///     // ... implementation
///     Ok(address)
/// }
/// ```
pub type WalletResult<T> = Result<T, WalletManagerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // Test that all error variants implement Display correctly
        let err = WalletManagerError::KeystoreLocked;
        assert_eq!(err.to_string(), "Keystore is locked. Please unlock the wallet first.");

        let err = WalletManagerError::InvalidPassword;
        assert_eq!(err.to_string(), "Invalid password. Please try again.");

        let err = WalletManagerError::AccountNotFound {
            address: "0x1234...".to_string(),
        };
        assert!(err.to_string().contains("Account not found"));
    }

    #[test]
    fn test_error_debug() {
        // Test that all error variants implement Debug correctly
        let err = WalletManagerError::EncryptionFailed {
            reason: "Test".to_string(),
        };
        assert!(format!("{:?}", err).contains("EncryptionFailed"));
    }
}
