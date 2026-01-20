//! Wallet Configuration Management
//!
//! This module handles wallet-level configuration and metadata storage,
//! separate from individual account management. The wallet config is encrypted
//! with a master password and contains account metadata and wallet settings.

use crate::error::{Result, SecurityError};
use crate::security::KeyReference;
use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use alloy::primitives::Address;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current wallet configuration format version
pub const WALLET_CONFIG_VERSION: u32 = 1;

/// Wallet configuration containing encrypted account metadata and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Configuration format version for future compatibility
    pub version: u32,

    /// Wallet unique identifier
    pub wallet_id: String,

    /// Wallet display name (user-friendly)
    pub wallet_name: String,

    /// When this wallet was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last time wallet was unlocked
    pub last_unlocked_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Encrypted account metadata list
    pub encrypted_account_metadata: EncryptedData,

    /// Wallet-level settings (also encrypted)
    pub encrypted_settings: EncryptedData,

    /// Encryption metadata for master password validation
    pub encryption_info: EncryptionInfo,
}

/// Account metadata stored in wallet config (before encryption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccountMetadata {
    /// Account unique identifier
    pub id: String,

    /// Account display name
    pub name: String,

    /// Account address
    pub address: Address,

    /// Reference to encrypted account data (seed/private key)
    pub key_reference: KeyReference,

    /// When account was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Whether this is a hardware wallet account
    pub is_hardware: bool,

    /// BIP44 derivation path if applicable
    pub derivation_path: Option<String>,

    /// Account-specific settings
    pub settings: AccountSettings,
}

/// Account-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSettings {
    /// Whether this account requires password for transactions
    pub requires_password_for_transactions: bool,

    /// Session timeout for this account (minutes)
    pub session_timeout_minutes: u32,

    /// Whether to show this account in main list
    pub visible: bool,

    /// Account color/icon for UI
    pub ui_color: Option<String>,
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self {
            requires_password_for_transactions: true,
            session_timeout_minutes: 15,
            visible: true,
            ui_color: None,
        }
    }
}

/// Wallet-level settings stored in config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSettings {
    /// Default session timeout for new accounts (minutes)
    pub default_session_timeout_minutes: u32,

    /// Whether to auto-lock wallet on system idle
    pub auto_lock_on_idle: bool,

    /// Idle timeout for auto-lock (minutes)
    pub auto_lock_idle_minutes: u32,

    /// Default network for new accounts
    pub default_network_id: Option<u64>,

    /// Whether to require master password for wallet operations
    pub require_master_password: bool,

    /// Security level setting
    pub security_level: SecurityLevel,
}

impl Default for WalletSettings {
    fn default() -> Self {
        Self {
            default_session_timeout_minutes: 15,
            auto_lock_on_idle: true,
            auto_lock_idle_minutes: 30,
            default_network_id: Some(943), // PulseChain Testnet v4
            require_master_password: true,
            security_level: SecurityLevel::Standard,
        }
    }
}

/// Security level settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security (single password confirmation)
    Basic,
    /// Standard security (password + session timeouts)
    Standard,
    /// High security (shorter timeouts, additional confirmations)
    High,
    /// Maximum security (very short timeouts, confirm all operations)
    Maximum,
}

/// Encrypted data container with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// AES-GCM encrypted data
    pub ciphertext: Vec<u8>,

    /// AES-GCM nonce (96 bits)
    pub nonce: [u8; 12],

    /// Salt used for key derivation
    pub salt: [u8; 32],

    /// HMAC for integrity verification
    pub hmac: [u8; 32],

    /// Encryption algorithm version
    pub algorithm_version: u32,
}

/// Encryption metadata for master password validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    /// Argon2 parameters used for key derivation
    pub argon2_params: Argon2Params,

    /// Salt for master password verification
    pub master_password_salt: [u8; 32],

    /// Verification hash of master password (using different salt)
    pub master_password_verification_hash: [u8; 32],

    /// When encryption was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Argon2 parameters for key derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Params {
    /// Memory cost (KB)
    pub memory_cost: u32,

    /// Time cost (iterations)
    pub time_cost: u32,

    /// Parallelism factor
    pub parallelism: u32,

    /// Output length
    pub output_length: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MB
            time_cost: 3,       // 3 iterations
            parallelism: 4,     // 4 threads
            output_length: 32,  // 32 bytes (256 bits)
        }
    }
}

impl WalletConfig {
    /// Create a new wallet configuration with master password
    pub fn new(wallet_name: String, master_password: &SecretString) -> Result<Self> {
        let wallet_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // Generate encryption parameters
        let argon2_params = Argon2Params::default();
        let master_password_salt = Self::generate_salt();

        // Create master password verification hash
        let master_password_verification_hash =
            Self::create_password_verification_hash(master_password, &master_password_salt, &argon2_params)?;

        let encryption_info = EncryptionInfo {
            argon2_params,
            master_password_salt,
            master_password_verification_hash,
            last_updated: now,
        };

        // Create default settings
        let default_settings = WalletSettings::default();
        let encrypted_settings = Self::encrypt_data(
            &serde_json::to_vec(&default_settings).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to serialize wallet settings: {e}"),
            })?,
            master_password,
            &encryption_info.argon2_params,
        )?;

        // Create empty account metadata
        let empty_accounts: Vec<WalletAccountMetadata> = Vec::new();
        let encrypted_account_metadata = Self::encrypt_data(
            &serde_json::to_vec(&empty_accounts).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to serialize account metadata: {e}"),
            })?,
            master_password,
            &encryption_info.argon2_params,
        )?;

        Ok(WalletConfig {
            version: WALLET_CONFIG_VERSION,
            wallet_id,
            wallet_name,
            created_at: now,
            last_unlocked_at: None,
            encrypted_account_metadata,
            encrypted_settings,
            encryption_info,
        })
    }

    /// Verify master password against stored verification hash
    pub fn verify_master_password(&self, password: &SecretString) -> Result<bool> {
        let computed_hash = Self::create_password_verification_hash(
            password,
            &self.encryption_info.master_password_salt,
            &self.encryption_info.argon2_params,
        )?;

        Ok(computed_hash == self.encryption_info.master_password_verification_hash)
    }

    /// Decrypt and return account metadata
    pub fn decrypt_account_metadata(&self, master_password: &SecretString) -> Result<Vec<WalletAccountMetadata>> {
        let decrypted_data = Self::decrypt_data(
            &self.encrypted_account_metadata,
            master_password,
            &self.encryption_info.argon2_params,
        )?;

        let accounts: Vec<WalletAccountMetadata> =
            serde_json::from_slice(&decrypted_data).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to deserialize account metadata: {e}"),
            })?;

        Ok(accounts)
    }

    /// Decrypt and return wallet settings
    pub fn decrypt_wallet_settings(&self, master_password: &SecretString) -> Result<WalletSettings> {
        let decrypted_data = Self::decrypt_data(
            &self.encrypted_settings,
            master_password,
            &self.encryption_info.argon2_params,
        )?;

        let settings: WalletSettings =
            serde_json::from_slice(&decrypted_data).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to deserialize wallet settings: {e}"),
            })?;

        Ok(settings)
    }

    /// Update account metadata (requires master password)
    pub fn update_account_metadata(
        &mut self,
        accounts: &[WalletAccountMetadata],
        master_password: &SecretString,
    ) -> Result<()> {
        let serialized = serde_json::to_vec(accounts).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize account metadata: {e}"),
        })?;

        self.encrypted_account_metadata =
            Self::encrypt_data(&serialized, master_password, &self.encryption_info.argon2_params)?;

        Ok(())
    }

    /// Update wallet settings (requires master password)
    pub fn update_wallet_settings(&mut self, settings: &WalletSettings, master_password: &SecretString) -> Result<()> {
        let serialized = serde_json::to_vec(settings).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize wallet settings: {e}"),
        })?;

        self.encrypted_settings =
            Self::encrypt_data(&serialized, master_password, &self.encryption_info.argon2_params)?;

        Ok(())
    }

    /// Update encryption info for a new master password
    /// This regenerates the password verification hash for the new password
    pub fn update_encryption_info(&mut self, new_password: &SecretString) -> Result<()> {
        let new_salt = Self::generate_salt();
        let new_verification_hash =
            Self::create_password_verification_hash(new_password, &new_salt, &self.encryption_info.argon2_params)?;

        self.encryption_info.master_password_salt = new_salt;
        self.encryption_info.master_password_verification_hash = new_verification_hash;
        self.encryption_info.last_updated = chrono::Utc::now();

        Ok(())
    }

    /// Generate a random salt
    fn generate_salt() -> [u8; 32] {
        use rand::RngCore;
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }

    /// Create password verification hash using Argon2
    fn create_password_verification_hash(
        password: &SecretString,
        salt: &[u8; 32],
        params: &Argon2Params,
    ) -> Result<[u8; 32]> {
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(
                params.memory_cost,
                params.time_cost,
                params.parallelism,
                Some(params.output_length as usize),
            )
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Invalid Argon2 parameters: {e}"),
            })?,
        );

        let salt_string = SaltString::encode_b64(salt).map_err(|e| SecurityError::KeyDerivationError {
            message: format!("Failed to encode salt: {e}"),
        })?;

        let password_hash = argon2
            .hash_password(password.expose_secret().as_bytes(), &salt_string)
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Argon2 password hashing failed: {e}"),
            })?;

        let hash_bytes = password_hash.hash.ok_or_else(|| SecurityError::KeyDerivationError {
            message: "No hash in Argon2 result".to_string(),
        })?;

        let mut result = [0u8; 32];
        let hash_slice = hash_bytes.as_bytes();
        result.copy_from_slice(&hash_slice[..32.min(hash_slice.len())]);

        Ok(result)
    }

    /// Encrypt data using AES-256-GCM with password-derived key
    pub fn encrypt_data(data: &[u8], password: &SecretString, argon2_params: &Argon2Params) -> Result<EncryptedData> {
        use rand::RngCore;
        use sha2::{Digest, Sha256};

        // Generate random salt and nonce
        let mut salt = [0u8; 32];
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce);

        // Derive encryption key using Argon2
        let encryption_key = Self::derive_encryption_key(password, &salt, argon2_params)?;

        // Encrypt data using AES-256-GCM
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&encryption_key));
        let nonce_obj = Nonce::from_slice(&nonce);

        let ciphertext = cipher
            .encrypt(nonce_obj, data)
            .map_err(|e| SecurityError::EncryptionError {
                message: format!("AES-GCM encryption failed: {e}"),
            })?;

        // Calculate HMAC for integrity
        let mut hasher = Sha256::new();
        hasher.update(&ciphertext);
        hasher.update(salt);
        hasher.update(nonce);
        hasher.update(b"vaughan-wallet-v1");
        let hmac = hasher.finalize();

        let mut hmac_array = [0u8; 32];
        hmac_array.copy_from_slice(&hmac);

        Ok(EncryptedData {
            ciphertext,
            nonce,
            salt,
            hmac: hmac_array,
            algorithm_version: 1,
        })
    }

    /// Decrypt data using AES-256-GCM with password-derived key
    pub fn decrypt_data(
        encrypted: &EncryptedData,
        password: &SecretString,
        argon2_params: &Argon2Params,
    ) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};

        // Verify HMAC integrity
        let mut hasher = Sha256::new();
        hasher.update(&encrypted.ciphertext);
        hasher.update(encrypted.salt);
        hasher.update(encrypted.nonce);
        hasher.update(b"vaughan-wallet-v1");
        let computed_hmac = hasher.finalize();

        if computed_hmac.as_slice() != encrypted.hmac {
            return Err(SecurityError::DecryptionError {
                message: "HMAC verification failed - data may be corrupted".to_string(),
            }
            .into());
        }

        // Derive decryption key using Argon2
        let decryption_key = Self::derive_encryption_key(password, &encrypted.salt, argon2_params)?;

        // Decrypt data using AES-256-GCM
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&decryption_key));
        let nonce_obj = Nonce::from_slice(&encrypted.nonce);

        let plaintext =
            cipher
                .decrypt(nonce_obj, encrypted.ciphertext.as_ref())
                .map_err(|e| SecurityError::DecryptionError {
                    message: format!("AES-GCM decryption failed: {e}"),
                })?;

        Ok(plaintext)
    }

    /// Derive encryption key from password using Argon2
    fn derive_encryption_key(password: &SecretString, salt: &[u8; 32], params: &Argon2Params) -> Result<[u8; 32]> {
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(
                params.memory_cost,
                params.time_cost,
                params.parallelism,
                Some(params.output_length as usize),
            )
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Invalid Argon2 parameters: {e}"),
            })?,
        );

        let salt_string = SaltString::encode_b64(salt).map_err(|e| SecurityError::KeyDerivationError {
            message: format!("Failed to encode salt: {e}"),
        })?;

        let password_hash = argon2
            .hash_password(password.expose_secret().as_bytes(), &salt_string)
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Argon2 key derivation failed: {e}"),
            })?;

        let hash_bytes = password_hash.hash.ok_or_else(|| SecurityError::KeyDerivationError {
            message: "No hash in Argon2 result".to_string(),
        })?;

        let mut key = [0u8; 32];
        let hash_slice = hash_bytes.as_bytes();
        key.copy_from_slice(&hash_slice[..32.min(hash_slice.len())]);

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_config_creation() {
        let password = SecretString::new("test-master-password-123!".to_string());
        let config = WalletConfig::new("Test Wallet".to_string(), &password).unwrap();

        assert_eq!(config.version, WALLET_CONFIG_VERSION);
        assert_eq!(config.wallet_name, "Test Wallet");
        assert!(config.verify_master_password(&password).unwrap());

        // Wrong password should fail
        let wrong_password = SecretString::new("wrong-password".to_string());
        assert!(!config.verify_master_password(&wrong_password).unwrap());
    }

    #[test]
    fn test_account_metadata_encryption() {
        let password = SecretString::new("test-master-password-123!".to_string());
        let mut config = WalletConfig::new("Test Wallet".to_string(), &password).unwrap();

        // Create test account metadata
        let test_account = WalletAccountMetadata {
            id: "test-account".to_string(),
            name: "Test Account".to_string(),
            address: Address::ZERO,
            key_reference: KeyReference {
                id: "key-ref-123".to_string(),
                service: "test-service".to_string(),
                account: "test-account".to_string(),
            },
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
            settings: AccountSettings::default(),
        };

        let accounts = vec![test_account.clone()];

        // Update account metadata
        config.update_account_metadata(&accounts, &password).unwrap();

        // Decrypt and verify
        let decrypted_accounts = config.decrypt_account_metadata(&password).unwrap();
        assert_eq!(decrypted_accounts.len(), 1);
        assert_eq!(decrypted_accounts[0].id, test_account.id);
        assert_eq!(decrypted_accounts[0].name, test_account.name);
    }

    #[test]
    fn test_wallet_settings_encryption() {
        let password = SecretString::new("test-master-password-123!".to_string());
        let mut config = WalletConfig::new("Test Wallet".to_string(), &password).unwrap();

        // Create custom settings
        let custom_settings = WalletSettings {
            default_session_timeout_minutes: 30,
            auto_lock_on_idle: false,
            auto_lock_idle_minutes: 60,
            default_network_id: Some(1),
            require_master_password: false,
            security_level: SecurityLevel::High,
        };

        // Update settings
        config.update_wallet_settings(&custom_settings, &password).unwrap();

        // Decrypt and verify
        let decrypted_settings = config.decrypt_wallet_settings(&password).unwrap();
        assert_eq!(decrypted_settings.default_session_timeout_minutes, 30);
        assert!(!decrypted_settings.auto_lock_on_idle);
        assert_eq!(decrypted_settings.auto_lock_idle_minutes, 60);
    }
}
