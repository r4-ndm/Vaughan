//! Security utilities and key management
//!
//! This module provides secure key handling, OS keychain integration,
//! and security utilities for the wallet.

use alloy::primitives::Address;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::error::Result;
 
/// Service name for private key storage in OS keychain
pub const SERVICE_NAME_PRIVATE_KEYS: &str = "vaughan-wallet";
/// Service name for encrypted seed phrase storage in OS keychain
pub const SERVICE_NAME_ENCRYPTED_SEEDS: &str = "vaughan-wallet-encrypted-seeds";



// pub mod account_migration; // Temporarily disabled due to compilation errors
pub mod hardware;
pub mod hardware_feedback;
pub mod export_auth;
// pub mod hardware_manager; // Removed redundant module

pub mod key_cache;
pub mod keychain;
pub mod keystore;
pub mod memory;
pub mod password_validator;
pub mod seed;
pub mod session;
pub mod rate_limiter;
pub mod transaction_signing;
pub mod validation;
pub mod wallet_config;
pub mod wallet_password_validator;
pub mod wallet_storage;

pub use hardware::*;
pub use hardware_feedback::*;
pub use export_auth::*;
pub use key_cache::*;
pub use keychain::*;
#[allow(ambiguous_glob_reexports)] // encryption module exists in both keystore and seed
pub use keystore::*;
pub use memory::*;
pub use password_validator::*;
#[allow(ambiguous_glob_reexports)] // encryption module exists in both keystore and seed
pub use seed::*;
pub use session::*;
pub use rate_limiter::*;
pub use transaction_signing::*;
pub use validation::*;
pub use wallet_config::*;
pub use wallet_password_validator::*;
pub use wallet_storage::*;

/// Secure account representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecureAccount {
    pub id: String,
    pub name: String,
    pub address: Address,
    pub key_reference: KeyReference,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_hardware: bool,
    pub derivation_path: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub last_used: Option<i64>,
    #[serde(default)]
    pub transaction_count: u64,
}

impl SecureAccount {
    /// Create a SecureAccount from wallet account metadata
    pub fn new_from_metadata(account_meta: &WalletAccountMetadata) -> Result<Self> {
        Ok(SecureAccount {
            id: account_meta.id.clone(),
            name: account_meta.name.clone(),
            address: account_meta.address,
            key_reference: account_meta.key_reference.clone(),
            created_at: account_meta.created_at,
            is_hardware: account_meta.is_hardware,
            derivation_path: account_meta.derivation_path.clone(),
            tags: Vec::new(),
            last_used: None,
            transaction_count: 0,
        })
    }
}

// Display implementation for GUI integration
impl std::fmt::Display for SecureAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Reference to a key in the keystore (not the actual key)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyReference {
    pub id: String,
    pub service: String,
    pub account: String,
}

/// Secure export format for account backup
#[derive(Debug, Clone)]
pub struct SecureExport {
    pub encrypted_data: Vec<u8>,
    pub encryption_type: EncryptionType,
    pub timestamp: u64,
}

/// Encryption type for secure exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Type alias for the secure keystore implementation
pub type SecureKeystore = keystore::SecureKeystoreImpl;

#[cfg(feature = "hardware-wallets")]
pub type HardwareManager = hardware::HardwareWalletManager;

/// Keychain interface trait for OS integration
pub trait KeychainInterface: Send + Sync + std::fmt::Debug {
    fn store(&self, key_ref: &KeyReference, key: SecretString) -> Result<()>;
    fn retrieve(&self, key_ref: &KeyReference) -> Result<SecretString>;
    fn delete(&self, key_ref: &KeyReference) -> Result<()>;

    /// Clone the keychain as a boxed trait object
    fn clone_box(&self) -> Box<dyn KeychainInterface>;
}

/// Create platform-specific keychain interface
pub fn create_keychain_interface() -> Result<Box<dyn KeychainInterface>> {
    keychain::create_keychain_interface()
}
