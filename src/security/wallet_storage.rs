//! Wallet Configuration Storage
//!
//! This module provides persistent storage for wallet configuration
//! using the OS keychain, separate from individual account storage.

use crate::error::{Result, SecurityError};
use crate::security::{KeyReference, KeychainInterface, WalletConfig};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Wallet configuration storage manager
#[derive(Debug)]
pub struct WalletConfigStorage {
    keychain: Box<dyn KeychainInterface>,
    config_path: PathBuf,
}

impl Clone for WalletConfigStorage {
    fn clone(&self) -> Self {
        Self {
            keychain: self.keychain.clone_box(),
            config_path: self.config_path.clone(),
        }
    }
}

/// Wallet metadata stored in local config directory (non-sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetadata {
    /// Wallet unique identifier
    pub wallet_id: String,

    /// Wallet display name
    pub wallet_name: String,

    /// When wallet was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last time wallet config was updated
    pub last_updated_at: chrono::DateTime<chrono::Utc>,

    /// Configuration format version
    pub config_version: u32,

    /// Reference to encrypted wallet config in keychain
    pub config_key_reference: KeyReference,

    /// Whether this wallet has been migrated to new format
    pub is_migrated: bool,
}

impl WalletConfigStorage {
    /// Create a new wallet config storage instance with default keychain
    pub fn new() -> Result<Self> {
        let keychain = crate::security::create_keychain_interface()?;
        Self::new_with_keychain(keychain)
    }

    /// Create a new wallet config storage instance with custom keychain
    pub fn new_with_keychain(keychain: Box<dyn KeychainInterface>) -> Result<Self> {
        let config_path = Self::get_wallet_metadata_path()?;

        Ok(Self { keychain, config_path })
    }

    /// Set custom configuration path (useful for testing)
    #[cfg(test)]
    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = path;
    }

    /// Create and store a new wallet configuration
    pub async fn create_wallet_config(
        &self,
        wallet_name: String,
        master_password: &SecretString,
    ) -> Result<WalletConfig> {
        tracing::info!("Creating new wallet configuration: {}", wallet_name);

        // Create new wallet configuration
        let wallet_config = WalletConfig::new(wallet_name.clone(), master_password)?;

        // Create key reference for storing in keychain
        let config_key_ref = KeyReference {
            id: format!("wallet-config-{}", wallet_config.wallet_id),
            service: "vaughan-wallet-configs".to_string(),
            account: wallet_config.wallet_id.clone(),
        };

        // Serialize and store wallet config in keychain
        let config_json = serde_json::to_string(&wallet_config).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize wallet config: {e}"),
        })?;

        let config_secret = SecretString::new(config_json);
        self.keychain.store(&config_key_ref, config_secret)?;

        // Create and store metadata
        let metadata = WalletMetadata {
            wallet_id: wallet_config.wallet_id.clone(),
            wallet_name: wallet_name.clone(),
            created_at: wallet_config.created_at,
            last_updated_at: wallet_config.created_at,
            config_version: wallet_config.version,
            config_key_reference: config_key_ref.clone(),
            is_migrated: true,
        };

        self.save_wallet_metadata(&metadata)?;

        tracing::info!("✅ Wallet configuration created successfully: {}", wallet_name);
        Ok(wallet_config)
    }

    /// Load wallet configuration from storage
    pub async fn load_wallet_config(&self, master_password: &SecretString) -> Result<Option<WalletConfig>> {
        // First, check if we have wallet metadata
        let metadata = match self.load_wallet_metadata()? {
            Some(meta) => meta,
            None => {
                tracing::debug!("No wallet metadata found");
                return Ok(None);
            }
        };

        tracing::info!("Loading wallet configuration: {}", metadata.wallet_name);

        // Load encrypted config from keychain
        let config_secret = match self.keychain.retrieve(&metadata.config_key_reference) {
            Ok(secret) => secret,
            Err(_) => {
                tracing::warn!("Wallet config not found in keychain for wallet: {}", metadata.wallet_id);
                return Ok(None);
            }
        };

        // Deserialize wallet config
        use secrecy::ExposeSecret;

        let wallet_config: WalletConfig =
            serde_json::from_str(config_secret.expose_secret()).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to deserialize wallet config: {e}"),
            })?;

        // Verify master password
        if !wallet_config.verify_master_password(master_password)? {
            return Err(SecurityError::DecryptionError {
                message: "Invalid master password".to_string(),
            }
            .into());
        }

        tracing::info!("✅ Wallet configuration loaded successfully: {}", metadata.wallet_name);
        Ok(Some(wallet_config))
    }

    /// Update wallet configuration in storage
    pub async fn save_wallet_config(&self, wallet_config: &WalletConfig, master_password: &SecretString) -> Result<()> {
        // Verify master password before saving
        if !wallet_config.verify_master_password(master_password)? {
            return Err(SecurityError::DecryptionError {
                message: "Invalid master password for wallet config update".to_string(),
            }
            .into());
        }

        tracing::info!("Updating wallet configuration: {}", wallet_config.wallet_name);

        // Load existing metadata to get key reference
        let mut metadata = self
            .load_wallet_metadata()?
            .ok_or_else(|| SecurityError::KeystoreError {
                message: "No wallet metadata found for config update".to_string(),
            })?;

        // Update metadata
        metadata.last_updated_at = chrono::Utc::now();
        metadata.config_version = wallet_config.version;
        metadata.wallet_name = wallet_config.wallet_name.clone();

        // Serialize and update wallet config in keychain
        let config_json = serde_json::to_string(wallet_config).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize wallet config: {e}"),
        })?;

        let config_secret = SecretString::new(config_json);
        self.keychain.store(&metadata.config_key_reference, config_secret)?;

        // Save updated metadata
        self.save_wallet_metadata(&metadata)?;

        tracing::info!("✅ Wallet configuration updated successfully");
        Ok(())
    }

    /// Check if a wallet configuration exists
    pub fn wallet_exists(&self) -> Result<bool> {
        Ok(self.config_path.exists() && self.config_path.is_file())
    }

    /// Get wallet metadata without loading the full config
    pub fn get_wallet_info(&self) -> Result<Option<WalletMetadata>> {
        self.load_wallet_metadata()
    }

    /// Delete wallet configuration completely
    pub async fn delete_wallet_config(&self, master_password: &SecretString) -> Result<()> {
        // Load and verify access to current config
        let wallet_config = match self.load_wallet_config(master_password).await? {
            Some(config) => config,
            None => {
                return Err(SecurityError::KeystoreError {
                    message: "No wallet configuration found to delete".to_string(),
                }
                .into());
            }
        };

        tracing::warn!("🗑️  Deleting wallet configuration: {}", wallet_config.wallet_name);

        // Load metadata to get key reference
        let metadata = self
            .load_wallet_metadata()?
            .ok_or_else(|| SecurityError::KeystoreError {
                message: "No wallet metadata found for deletion".to_string(),
            })?;

        // Delete from keychain
        self.keychain.delete(&metadata.config_key_reference)?;

        // Delete metadata file
        if self.config_path.exists() {
            std::fs::remove_file(&self.config_path).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to delete wallet metadata file: {e}"),
            })?;
        }

        tracing::warn!("✅ Wallet configuration deleted successfully");
        Ok(())
    }

    /// Reset wallet to factory defaults (removes all data)
    pub async fn factory_reset(&self) -> Result<()> {
        tracing::warn!("🚨 Performing factory reset - all wallet data will be lost!");

        // Try to load metadata
        if let Ok(Some(metadata)) = self.load_wallet_metadata() {
            // Delete keychain entry if it exists
            let _ = self.keychain.delete(&metadata.config_key_reference);
        }

        // Delete metadata file
        if self.config_path.exists() {
            std::fs::remove_file(&self.config_path).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to delete wallet metadata during factory reset: {e}"),
            })?;
        }

        // Also try to delete parent directory if it's empty
        if let Some(parent) = self.config_path.parent() {
            let _ = std::fs::remove_dir(parent); // Ignore errors - directory might not be empty
        }

        tracing::warn!("✅ Factory reset completed");
        Ok(())
    }

    /// Migrate from old account-based storage to new wallet-based storage
    pub async fn migrate_from_legacy(
        &self,
        wallet_name: String,
        master_password: &SecretString,
    ) -> Result<WalletConfig> {
        tracing::info!("🔄 Migrating from legacy storage to wallet-based configuration");

        // Create new wallet config
        let wallet_config = self.create_wallet_config(wallet_name, master_password).await?;

        tracing::info!("✅ Migration completed successfully");
        Ok(wallet_config)
    }

    /// Export wallet configuration for backup (encrypted)
    pub async fn export_wallet_config(
        &self,
        master_password: &SecretString,
        export_password: &SecretString,
    ) -> Result<String> {
        let mut wallet_config =
            self.load_wallet_config(master_password)
                .await?
                .ok_or_else(|| SecurityError::KeystoreError {
                    message: "No wallet configuration found for export".to_string(),
                })?;

        // Re-encrypt internal secrets with export_password so they can be decrypted during import
        // (import uses the export password to decrypt both the container and inner fields)
        let accounts = wallet_config.decrypt_account_metadata(master_password)?;
        let settings = wallet_config.decrypt_wallet_settings(master_password)?;

        wallet_config.update_account_metadata(&accounts, export_password)?;
        wallet_config.update_wallet_settings(&settings, export_password)?;

        // Create export data with different encryption
        let export_data = serde_json::to_string(&wallet_config).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize wallet config for export: {e}"),
        })?;

        // Re-encrypt with export password
        let encrypted_export = WalletConfig::encrypt_data(
            export_data.as_bytes(),
            export_password,
            &crate::security::Argon2Params::default(),
        )?;

        let export_json = serde_json::to_string(&encrypted_export).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize export data: {e}"),
        })?;

        tracing::info!("✅ Wallet configuration exported successfully");
        Ok(export_json)
    }

    /// Import wallet configuration from backup
    pub async fn import_wallet_config(
        &self,
        import_data: &str,
        import_password: &SecretString,
        new_master_password: &SecretString,
    ) -> Result<WalletConfig> {
        tracing::info!("📥 Importing wallet configuration from backup");

        // Deserialize encrypted import data
        let encrypted_data: crate::security::EncryptedData =
            serde_json::from_str(import_data).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to deserialize import data: {e}"),
            })?;

        // Decrypt with import password
        let decrypted_data = WalletConfig::decrypt_data(
            &encrypted_data,
            import_password,
            &crate::security::Argon2Params::default(),
        )?;

        // Deserialize wallet config
        let mut wallet_config: WalletConfig =
            serde_json::from_slice(&decrypted_data).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to deserialize imported wallet config: {e}"),
            })?;

        // Re-encrypt with new master password
        let current_accounts = wallet_config.decrypt_account_metadata(import_password)?;
        let current_settings = wallet_config.decrypt_wallet_settings(import_password)?;

        // Update encryption with new master password
        wallet_config.update_account_metadata(&current_accounts, new_master_password)?;
        wallet_config.update_wallet_settings(&current_settings, new_master_password)?;

        // Update the encryption info (password verification hash) for the new master password
        wallet_config.update_encryption_info(new_master_password)?;

        // Generate new wallet ID to avoid conflicts
        wallet_config.wallet_id = uuid::Uuid::new_v4().to_string();

        // Create a new key reference for this imported config (don't reuse save_wallet_config)
        let config_key_ref = KeyReference {
            id: wallet_config.wallet_id.clone(),
            service: "vaughan-wallet".to_string(),
            account: format!("wallet-config-{}", wallet_config.wallet_id),
        };

        // Serialize and store wallet config in keychain
        let config_json = serde_json::to_string(&wallet_config).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize imported wallet config: {e}"),
        })?;

        let config_secret = SecretString::new(config_json);
        self.keychain.store(&config_key_ref, config_secret)?;

        // Create and store metadata for the imported wallet
        let metadata = WalletMetadata {
            wallet_id: wallet_config.wallet_id.clone(),
            wallet_name: wallet_config.wallet_name.clone(),
            created_at: wallet_config.created_at,
            last_updated_at: chrono::Utc::now(),
            config_version: wallet_config.version,
            config_key_reference: config_key_ref,
            is_migrated: true,
        };

        self.save_wallet_metadata(&metadata)?;

        tracing::info!("✅ Wallet configuration imported successfully");
        Ok(wallet_config)
    }

    /// Save wallet metadata to local file
    fn save_wallet_metadata(&self, metadata: &WalletMetadata) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to create wallet metadata directory: {e}"),
            })?;
        }

        let metadata_json = serde_json::to_string_pretty(metadata).map_err(|e| SecurityError::SerializationError {
            message: format!("Failed to serialize wallet metadata: {e}"),
        })?;

        std::fs::write(&self.config_path, metadata_json).map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to write wallet metadata file: {e}"),
        })?;

        Ok(())
    }

    /// Load wallet metadata from local file
    fn load_wallet_metadata(&self) -> Result<Option<WalletMetadata>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let metadata_json = std::fs::read_to_string(&self.config_path).map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to read wallet metadata file: {e}"),
        })?;

        let metadata: WalletMetadata =
            serde_json::from_str(&metadata_json).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to deserialize wallet metadata: {e}"),
            })?;

        Ok(Some(metadata))
    }

    /// Get the path for wallet metadata storage
    fn get_wallet_metadata_path() -> Result<PathBuf> {
        let config_dir = if let Ok(home_dir) = std::env::var("HOME") {
            PathBuf::from(home_dir).join(".config").join("vaughan")
        } else if let Ok(appdata_dir) = std::env::var("APPDATA") {
            PathBuf::from(appdata_dir).join("Vaughan")
        } else {
            return Err(SecurityError::KeystoreError {
                message: "Unable to determine config directory".to_string(),
            }
            .into());
        };

        Ok(config_dir.join("wallet_metadata.json"))
    }

    /// Check if wallet configuration exists
    pub fn wallet_config_exists(&self) -> Result<bool> {
        let metadata_path = Self::get_wallet_metadata_path()?;
        Ok(metadata_path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::OSKeychain;

    #[tokio::test]
    async fn test_wallet_config_storage() {
        let _keychain = Box::new(OSKeychain::new("test-wallet-config".to_string()).unwrap());
        let mut storage = WalletConfigStorage::new().unwrap();
        // Use unique path to avoid race conditions
        let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&temp_dir).unwrap();
        storage.set_config_path(temp_dir.join("wallet_metadata.json"));

        let master_password = SecretString::new("test-master-password-123!".to_string());
        let wallet_name = "Test Wallet".to_string();

        // Clean up any existing config
        let _ = storage.factory_reset().await;

        // Create new wallet config
        let wallet_config = storage
            .create_wallet_config(wallet_name.clone(), &master_password)
            .await
            .unwrap();
        assert_eq!(wallet_config.wallet_name, wallet_name);

        // Verify wallet exists
        assert!(storage.wallet_exists().unwrap());

        // Load wallet config
        let loaded_config = storage.load_wallet_config(&master_password).await.unwrap().unwrap();
        assert_eq!(loaded_config.wallet_id, wallet_config.wallet_id);
        assert_eq!(loaded_config.wallet_name, wallet_name);

        // Test wrong password
        let wrong_password = SecretString::new("wrong-password".to_string());
        assert!(storage.load_wallet_config(&wrong_password).await.is_err());

        // Clean up
        let _ = storage.delete_wallet_config(&master_password).await;
    }

    #[tokio::test]
    async fn test_wallet_export_import() {
        let _keychain = Box::new(OSKeychain::new("test-wallet-export".to_string()).unwrap());
        let mut storage = WalletConfigStorage::new().unwrap();
        // Use unique path to avoid race conditions
        let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&temp_dir).unwrap();
        storage.set_config_path(temp_dir.join("wallet_metadata.json"));

        let master_password = SecretString::new("master-password-123!".to_string());
        let export_password = SecretString::new("export-password-456!".to_string());
        let wallet_name = "Export Test Wallet".to_string();

        // Clean up
        let _ = storage.factory_reset().await;

        // Create wallet
        let _wallet_config = storage
            .create_wallet_config(wallet_name.clone(), &master_password)
            .await
            .unwrap();

        // Export wallet
        let export_data = storage
            .export_wallet_config(&master_password, &export_password)
            .await
            .unwrap();
        assert!(!export_data.is_empty());

        // Delete original
        storage.delete_wallet_config(&master_password).await.unwrap();

        // Import wallet with new master password
        let new_master_password = SecretString::new("new-master-password-789!".to_string());
        let imported_config = storage
            .import_wallet_config(&export_data, &export_password, &new_master_password)
            .await
            .unwrap();

        assert_eq!(imported_config.wallet_name, wallet_name);

        // Verify new master password works
        let loaded_config = storage.load_wallet_config(&new_master_password).await.unwrap().unwrap();
        assert_eq!(loaded_config.wallet_name, wallet_name);

        // Clean up
        let _ = storage.delete_wallet_config(&new_master_password).await;
    }
}
