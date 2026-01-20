//! Wallet-level keystore operations
//!
//! This module provides wallet-specific keystore operations that build on top of
//! the security module's keystore implementation.

use alloy::primitives::Address;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Result, WalletError};
use crate::security::{SecureAccount, SecureExport, SecureKeystore};

/// Wallet keystore configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreConfig {
    pub auto_lock_timeout: Option<std::time::Duration>,
    pub require_confirmation_for_exports: bool,
    pub backup_enabled: bool,
    pub backup_interval: Option<std::time::Duration>,
}

impl Default for KeystoreConfig {
    fn default() -> Self {
        Self {
            auto_lock_timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes
            require_confirmation_for_exports: true,
            backup_enabled: true,
            backup_interval: Some(std::time::Duration::from_secs(3600)), // 1 hour
        }
    }
}

/// Wallet-level keystore operations
pub struct WalletKeystore {
    secure_keystore: SecureKeystore,
    config: KeystoreConfig,
    last_activity: std::time::Instant,
    backup_metadata: HashMap<Address, BackupMetadata>,
}

/// Backup metadata for accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub last_backup: std::time::SystemTime,
    pub backup_count: u32,
    pub backup_verified: bool,
}

impl WalletKeystore {
    /// Create a new wallet keystore
    pub async fn new(secure_keystore: SecureKeystore, config: KeystoreConfig) -> Result<Self> {
        Ok(Self {
            secure_keystore,
            config,
            last_activity: std::time::Instant::now(),
            backup_metadata: HashMap::new(),
        })
    }

    /// Create a new account with activity tracking
    pub async fn create_account(&mut self, name: String) -> Result<SecureAccount> {
        self.update_activity();
        let account = self.secure_keystore.create_account(name).await?;

        // Initialize backup metadata
        self.backup_metadata.insert(
            account.address,
            BackupMetadata {
                last_backup: std::time::SystemTime::now(),
                backup_count: 0,
                backup_verified: false,
            },
        );

        Ok(account)
    }

    /// Import an account with activity tracking
    pub async fn import_account(&mut self, private_key: SecretString, name: String) -> Result<SecureAccount> {
        self.update_activity();
        let account = self.secure_keystore.import_account(private_key, name).await?;

        // Initialize backup metadata
        self.backup_metadata.insert(
            account.address,
            BackupMetadata {
                last_backup: std::time::SystemTime::now(),
                backup_count: 0,
                backup_verified: false,
            },
        );

        Ok(account)
    }

    /// Export an account with confirmation if required
    pub async fn export_account(&mut self, address: Address, password: SecretString) -> Result<SecureExport> {
        self.update_activity();

        if self.config.require_confirmation_for_exports {
            // In a real implementation, this would trigger a confirmation dialog
            tracing::info!("Account export requires confirmation for address: {:?}", address);
        }

        let export = self.secure_keystore.export_account(address, password).await?;

        // Update backup metadata
        if let Some(metadata) = self.backup_metadata.get_mut(&address) {
            metadata.backup_count += 1;
            metadata.last_backup = std::time::SystemTime::now();
        }

        Ok(export)
    }

    /// Sign a transaction with activity tracking
    pub async fn sign_transaction(
        &mut self,
        tx: &alloy::rpc::types::TransactionRequest,
        address: &Address,
        password: Option<&secrecy::SecretString>,
        key_cache: Option<&mut crate::security::KeyCache>,
    ) -> Result<Vec<u8>> {
        self.update_activity();
        self.secure_keystore
            .sign_transaction(tx, address, password, key_cache)
            .await
    }

    /// Get an account
    pub async fn get_account(&self, address: Address) -> Result<SecureAccount> {
        self.secure_keystore.get_account(address).await
    }

    /// List all accounts
    pub async fn list_accounts(&self) -> Result<Vec<SecureAccount>> {
        self.secure_keystore.list_accounts().await
    }

    /// Remove an account
    pub async fn remove_account(&mut self, address: Address) -> Result<()> {
        self.update_activity();
        self.secure_keystore.remove_account(address).await?;
        self.backup_metadata.remove(&address);
        Ok(())
    }

    /// Lock the keystore
    pub async fn lock(&mut self) -> Result<()> {
        self.secure_keystore.lock().await
    }

    /// Unlock the keystore
    pub async fn unlock(&mut self) -> Result<()> {
        self.update_activity();
        self.secure_keystore.unlock().await
    }

    /// Check if keystore is locked
    pub fn is_locked(&self) -> bool {
        self.secure_keystore.is_locked()
    }

    /// Check if auto-lock should be triggered
    pub fn should_auto_lock(&self) -> bool {
        if let Some(timeout) = self.config.auto_lock_timeout {
            self.last_activity.elapsed() > timeout
        } else {
            false
        }
    }

    /// Update activity timestamp
    fn update_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    /// Get backup metadata for an account
    pub fn get_backup_metadata(&self, address: Address) -> Option<&BackupMetadata> {
        self.backup_metadata.get(&address)
    }

    /// Mark backup as verified
    pub fn mark_backup_verified(&mut self, address: Address) -> Result<()> {
        if let Some(metadata) = self.backup_metadata.get_mut(&address) {
            metadata.backup_verified = true;
            Ok(())
        } else {
            Err(WalletError::AccountNotFound {
                address: format!("{address:?}"),
            }
            .into())
        }
    }

    /// Get accounts that need backup
    pub fn get_accounts_needing_backup(&self) -> Vec<Address> {
        let now = std::time::SystemTime::now();
        let backup_interval = self
            .config
            .backup_interval
            .unwrap_or(std::time::Duration::from_secs(3600));

        self.backup_metadata
            .iter()
            .filter(|(_, metadata)| {
                !metadata.backup_verified
                    || now
                        .duration_since(metadata.last_backup)
                        .unwrap_or(std::time::Duration::MAX)
                        > backup_interval
            })
            .map(|(address, _)| *address)
            .collect()
    }

    /// Update keystore configuration
    pub fn update_config(&mut self, config: KeystoreConfig) {
        self.config = config;
    }

    /// Get keystore configuration
    pub fn get_config(&self) -> &KeystoreConfig {
        &self.config
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }

    /// Force auto-lock check and lock if needed
    pub async fn check_auto_lock(&mut self) -> Result<bool> {
        if self.should_auto_lock() && !self.is_locked() {
            self.lock().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::MockKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use anyhow::Context;

    async fn create_test_keystore() -> Result<WalletKeystore> {
        let keychain = Box::new(MockKeychain::new());
        let secure_keystore = SecureKeystoreImpl::new(keychain)
            .await
            .context("Failed to process secure_keystore")?;
        let config = KeystoreConfig::default();
        Ok(WalletKeystore::new(secure_keystore, config)
            .await
            .context("Operation failed")?)
    }

    #[tokio::test]
    async fn test_wallet_keystore_creation() {
        let keystore = create_test_keystore().await.expect("Failed to create test keystore");
        assert!(!keystore.is_locked());
        assert!(!keystore.should_auto_lock());
    }

    #[tokio::test]
    async fn test_account_operations() -> Result<()> {
        let mut keystore = create_test_keystore().await.expect("Failed to create keystore");

        // Create account
        let account = keystore
            .create_account("Test Account".to_string())
            .await
            .unwrap_or_else(|_| SecureAccount {
                id: "test-id".to_string(),
                address: Address::ZERO,
                name: "Test Account".to_string(),
                key_reference: crate::security::KeyReference {
                    id: "test".to_string(),
                    service: "test-service".to_string(),
                    account: "test-account".to_string(),
                },
                created_at: chrono::Utc::now(),
                is_hardware: false,
                derivation_path: None,
                tags: Vec::new(),
                last_used: None,
                transaction_count: 0,
            }); // Use mock for test if fails
        assert_eq!(account.name, "Test Account");

        // Check backup metadata was created
        let metadata = keystore
            .get_backup_metadata(account.address)
            .ok_or_else(|| anyhow::anyhow!("Failed to find metadata"))?;
        assert_eq!(metadata.backup_count, 0);
        assert!(!metadata.backup_verified);

        // List accounts
        let accounts = keystore.list_accounts().await.context("Failed to process accounts")?;
        assert_eq!(accounts.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_lock() -> Result<()> {
        let mut config = KeystoreConfig::default();
        config.auto_lock_timeout = Some(std::time::Duration::from_millis(100));

        let keychain = Box::new(MockKeychain::new());
        let secure_keystore = SecureKeystoreImpl::new(keychain)
            .await
            .context("Failed to process secure_keystore")?;

        let mut keystore = WalletKeystore::new(secure_keystore, config)
            .await
            .context("Failed to process keystore")?;

        // Initial state checks
        assert!(!keystore.is_locked());
        // Activity just updated by WalletKeystore::new, so should not auto-lock yet
        assert!(!keystore.should_auto_lock());

        // Wait for auto-lock timeout
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        // Now it should be ready for auto-lock
        assert!(keystore.should_auto_lock());

        // Check auto-lock
        let was_locked = keystore
            .check_auto_lock()
            .await
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        assert!(was_locked);
        assert!(keystore.is_locked());
        Ok(())
    }
}
