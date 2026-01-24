//! Unified Account Manager Interface
//!
//! This module provides a unified trait interface for all account operations,
//! following the design specification for enhanced account management.
//!
//! ## Design Principles
//!
//! - **Alloy-First**: Uses Alloy primitives for all blockchain operations
//! - **Unified Interface**: Single trait for all account operations (Requirement 1.1)
//! - **Async Patterns**: All operations are async for proper concurrency (Requirement 1.4)
//! - **Separation of Concerns**: UI concerns are completely separated from business logic
//!
//! ## Usage Examples
//!
//! ### Creating a New Wallet
//!
//! ```rust,ignore
//! use vaughan::wallet::account_manager::{AccountManager, AccountConfig, SeedStrength};
//! use secrecy::SecretString;
//!
//! async fn create_example(manager: &mut AccountManager) -> Result<()> {
//!     let config = AccountConfig::seed_based("Main Wallet")
//!         .with_seed_strength(SeedStrength::Words24);
//!     
//!     let password = SecretString::new("secure-password".to_string());
//!     let account = manager.create_account(config, &password).await?;
//!     println!("Created account: {}", account.address);
//!     Ok(())
//! }
//! ```
//!
//! ### Importing a Private Key
//!
//! ```rust,ignore
//! use vaughan::wallet::account_manager::{AccountManager, ImportSource};
//! use secrecy::SecretString;
//!
//! async fn import_example(manager: &mut AccountManager) -> Result<()> {
//!     let source = ImportSource::PrivateKey {
//!         key: SecretString::new("0x123...".to_string()),
//!         name: "Imported Account".to_string(),
//!         password: SecretString::new("encryption-password".to_string()),
//!     };
//!     
//!     let account = manager.import_account(source).await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Integrating with Alloy
//!
//! ```rust,ignore
//! use vaughan::wallet::account_manager::signer_integration::VaughanSigner;
//! use alloy::providers::ProviderBuilder;
//!
//! async fn provider_example(manager: &mut AccountManager, account_address: Address) -> Result<()> {
//!     // 1. Unlock wallet first
//!     manager.unlock(&SecretString::new("password".to_string())).await?;
//!     
//!     // 2. Get signer (assuming you have access to the internal signer components)
//!     // Note: In practice, high-level services wrap this.
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Requirements Addressed
//!
//! - **Requirement 1.1**: THE Account_Manager SHALL provide a unified trait interface
//! - **Requirement 1.2**: WHEN any account operation is requested, THE Account_Manager
//!   SHALL handle it through the unified interface
//! - **Requirement 1.5**: THE Account_Manager SHALL use Alloy primitives for all
//!   blockchain-related operations

pub mod creation;
pub mod import;
pub mod export;
pub mod metadata;
pub mod signer_integration;
pub mod discovery;
pub mod eip712;

pub use creation::{AccountCreator, AccountCreationConfig, CreatedAccount, KeyValidation, SeedValidation};
pub use import::{AccountImporter, FormatDetectionResult, ImportedAccount, ImportMetadata, ImportSourceType, ImportValidationResult};
pub use export::*;

use alloy::primitives::Address;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AccountError;
use crate::security::{SecureAccount, SecureExport};

/// Authentication token for sensitive operations
///
/// Used to authenticate export operations and other sensitive actions.
/// Tokens are time-limited and should be generated fresh for each sensitive operation.
#[derive(Debug, Clone)]
pub struct AuthToken {
    /// Unique token identifier
    pub id: Uuid,
    /// Token creation timestamp
    pub created_at: DateTime<Utc>,
    /// Token expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Operation this token authorizes
    pub operation: AuthorizedOperation,
}

impl AuthToken {
    /// Create a new auth token with default expiration (5 minutes)
    pub fn new(operation: AuthorizedOperation) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
            operation,
        }
    }

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token is valid for the given operation
    pub fn is_valid_for(&self, operation: &AuthorizedOperation) -> bool {
        !self.is_expired() && &self.operation == operation
    }
}

/// Operations that require authentication tokens
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizedOperation {
    /// Export seed phrase
    ExportSeed,
    /// Export private key
    ExportPrivateKey,
    /// Remove account
    RemoveAccount,
}

/// Configuration for creating a new account
///
/// Specifies all parameters needed to create a new account,
/// including account type, seed strength, and derivation path.
#[derive(Debug, Clone)]
pub struct AccountConfig {
    /// Human-readable account name
    pub name: String,
    /// Type of account to create
    pub account_type: AccountType,
    /// Seed phrase strength (only for SeedBased accounts)
    pub seed_strength: Option<SeedStrength>,
    /// Custom derivation path (optional, uses default if not specified)
    pub derivation_path: Option<String>,
    /// Derivation standard to use (optional, overrides derivation_path if set)
    pub derivation_standard: Option<crate::wallet::hardware::DerivationStandard>,
}

impl AccountConfig {
    /// Create a new seed-based account configuration
    pub fn seed_based(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            account_type: AccountType::SeedBased,
            seed_strength: Some(SeedStrength::Words12),
            derivation_path: None,
            derivation_standard: None,
        }
    }

    /// Create a new private key account configuration
    pub fn private_key(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            account_type: AccountType::PrivateKey,
            seed_strength: None,
            derivation_path: None,
            derivation_standard: None,
        }
    }

    /// Create a new hardware wallet account configuration
    pub fn hardware(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            account_type: AccountType::Hardware,
            seed_strength: None,
            derivation_path: None,
            derivation_standard: Some(crate::wallet::hardware::DerivationStandard::Bip44),
        }
    }

    /// Set the seed strength (for seed-based accounts)
    pub fn with_seed_strength(mut self, strength: SeedStrength) -> Self {
        self.seed_strength = Some(strength);
        self
    }

    /// Set a custom derivation path
    pub fn with_derivation_path(mut self, path: impl Into<String>) -> Self {
        self.derivation_path = Some(path.into());
        self
    }

    /// Set the derivation standard
    pub fn with_standard(mut self, standard: crate::wallet::hardware::DerivationStandard) -> Self {
        self.derivation_standard = Some(standard);
        self
    }
}

/// Type of account
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Account derived from BIP39 seed phrase
    SeedBased,
    /// Account from a raw private key
    PrivateKey,
    /// Hardware wallet account (Ledger/Trezor)
    Hardware,
}

/// Seed phrase strength (word count)
///
/// Higher word counts provide more entropy for key derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedStrength {
    /// 12 words (128 bits of entropy)
    Words12,
    /// 15 words (160 bits of entropy)
    Words15,
    /// 18 words (192 bits of entropy)
    Words18,
    /// 21 words (224 bits of entropy)
    Words21,
    /// 24 words (256 bits of entropy)
    Words24,
}

impl SeedStrength {
    /// Get the number of words for this strength
    pub fn word_count(&self) -> usize {
        match self {
            Self::Words12 => 12,
            Self::Words15 => 15,
            Self::Words18 => 18,
            Self::Words21 => 21,
            Self::Words24 => 24,
        }
    }

    /// Get the entropy bits for this strength
    pub fn entropy_bits(&self) -> usize {
        match self {
            Self::Words12 => 128,
            Self::Words15 => 160,
            Self::Words18 => 192,
            Self::Words21 => 224,
            Self::Words24 => 256,
        }
    }
}

/// Source for importing an account
///
/// Supports importing from various sources including seed phrases,
/// private keys, and MetaMask keystores.
#[derive(Debug, Clone)]
pub enum ImportSource {
    /// Import from BIP39 seed phrase
    SeedPhrase {
        /// The mnemonic words
        mnemonic: SecretString,
        /// Account name
        name: String,
        /// Custom derivation path (optional)
        derivation_path: Option<String>,
        /// Password to encrypt the imported account
        password: SecretString,
    },
    /// Import from raw private key
    PrivateKey {
        /// The private key (hex format, with or without 0x prefix)
        key: SecretString,
        /// Account name
        name: String,
        /// Password to encrypt the imported account
        password: SecretString,
    },
    /// Import from MetaMask keystore format
    MetaMaskKeystore {
        /// Path to the keystore file or JSON content
        keystore_json: String,
        /// Password to decrypt the MetaMask keystore
        keystore_password: SecretString,
        /// Account name
        name: String,
        /// New password to encrypt the imported account
        new_password: SecretString,
    },
}

/// Unified Account Manager Trait
///
/// This trait defines the interface for all account management operations.
/// All implementations must be `Send + Sync` for safe concurrent access.
///
/// ## Requirements
///
/// - **1.1**: Unified trait interface for all account operations
/// - **1.2**: All operations go through this interface
/// - **1.4**: Safe concurrent operation handling with async patterns
/// - **1.5**: Uses Alloy primitives for blockchain operations
#[async_trait]
pub trait AccountManagerTrait: Send + Sync {
    // ========== Lifecycle Operations ==========

    /// Create a new account
    ///
    /// Generates a new account based on the provided configuration.
    /// For seed-based accounts, generates a new random mnemonic.
    ///
    /// # Arguments
    /// * `config` - Account configuration specifying type, name, etc.
    /// * `password` - Password to encrypt the account
    ///
    /// # Returns
    /// * `SecureAccount` - The newly created account
    async fn create_account(
        &mut self,
        config: AccountConfig,
        password: &SecretString,
    ) -> Result<SecureAccount, AccountError>;

    /// Import an account from an external source
    ///
    /// Imports an account from seed phrase, private key, or MetaMask keystore.
    ///
    /// # Arguments
    /// * `source` - The import source (seed, private key, or keystore)
    ///
    /// # Returns
    /// * `SecureAccount` - The imported account
    async fn import_account(
        &mut self,
        source: ImportSource,
    ) -> Result<SecureAccount, AccountError>;

    /// Remove an account
    ///
    /// Removes an account from the wallet. Requires authentication token.
    ///
    /// # Arguments
    /// * `address` - Address of the account to remove
    /// * `auth_token` - Authentication token for this sensitive operation
    ///
    /// # Returns
    /// * `()` on success
    async fn remove_account(
        &mut self,
        address: Address,
        auth_token: AuthToken,
    ) -> Result<(), AccountError>;

    // ========== Query Operations ==========

    /// List all accounts
    ///
    /// Returns all accounts managed by this wallet.
    ///
    /// # Returns
    /// * `Vec<SecureAccount>` - List of all accounts
    async fn list_accounts(&self) -> Result<Vec<SecureAccount>, AccountError>;

    /// Get a specific account by address
    ///
    /// # Arguments
    /// * `address` - The account address to look up
    ///
    /// # Returns
    /// * `Option<SecureAccount>` - The account if found
    async fn get_account(
        &self,
        address: Address,
    ) -> Result<Option<SecureAccount>, AccountError>;

    /// Get the currently active account
    ///
    /// # Returns
    /// * `Option<SecureAccount>` - The current account if one is selected
    async fn get_current_account(&self) -> Result<Option<SecureAccount>, AccountError>;

    // ========== State Operations ==========

    /// Set the current active account
    ///
    /// # Arguments
    /// * `address` - Address of the account to make current
    async fn set_current_account(&mut self, address: Address) -> Result<(), AccountError>;

    /// Lock the wallet
    ///
    /// Clears all sensitive data from memory. Operations requiring
    /// authentication will fail until the wallet is unlocked.
    async fn lock(&mut self) -> Result<(), AccountError>;

    /// Unlock the wallet
    ///
    /// Unlocks the wallet with the provided password, restoring
    /// access to all account operations.
    ///
    /// # Arguments
    /// * `password` - The wallet password
    async fn unlock(&mut self, password: &SecretString) -> Result<(), AccountError>;

    /// Check if the wallet is currently locked
    fn is_locked(&self) -> bool;

    // ========== Export Operations ==========

    /// Export the seed phrase for an account
    ///
    /// Requires password and authentication token for security.
    ///
    /// # Arguments
    /// * `address` - Address of the account to export
    /// * `password` - Password to verify authorization
    /// * `auth_token` - Authentication token for this sensitive operation
    ///
    /// # Returns
    /// * `SecureExport` - Encrypted export data
    async fn export_seed(
        &self,
        address: Address,
        password: &SecretString,
        auth_token: AuthToken,
    ) -> Result<SecureExport, AccountError>;

    /// Export the private key for an account
    ///
    /// Requires password and authentication token for security.
    ///
    /// # Arguments
    /// * `address` - Address of the account to export
    /// * `password` - Password to verify authorization
    /// * `auth_token` - Authentication token for this sensitive operation
    ///
    /// # Returns
    /// * `SecureExport` - Encrypted export data
    async fn export_private_key(
        &self,
        address: Address,
        password: &SecretString,
        auth_token: AuthToken,
    ) -> Result<SecureExport, AccountError>;
}

/// Result type alias for AccountManager operations
pub type AccountManagerResult<T> = Result<T, AccountError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_expiration() {
        let token = AuthToken::new(AuthorizedOperation::ExportSeed);
        assert!(!token.is_expired());
        assert!(token.is_valid_for(&AuthorizedOperation::ExportSeed));
        assert!(!token.is_valid_for(&AuthorizedOperation::ExportPrivateKey));
    }

    #[test]
    fn test_account_config_builders() {
        let config = AccountConfig::seed_based("My Wallet")
            .with_seed_strength(SeedStrength::Words24)
            .with_derivation_path("m/44'/60'/0'/0/0");

        assert_eq!(config.name, "My Wallet");
        assert_eq!(config.account_type, AccountType::SeedBased);
        assert_eq!(config.seed_strength, Some(SeedStrength::Words24));
        assert_eq!(config.derivation_path, Some("m/44'/60'/0'/0/0".to_string()));
    }

    #[test]
    fn test_seed_strength_values() {
        assert_eq!(SeedStrength::Words12.word_count(), 12);
        assert_eq!(SeedStrength::Words12.entropy_bits(), 128);
        assert_eq!(SeedStrength::Words24.word_count(), 24);
        assert_eq!(SeedStrength::Words24.entropy_bits(), 256);
    }

    #[test]
    fn test_account_type_serialization() {
        let account_type = AccountType::SeedBased;
        let json = serde_json::to_string(&account_type).expect("Serialization failed");
        let deserialized: AccountType = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(account_type, deserialized);
    }
}

/// Unit tests for AccountManager interface
///
/// These tests validate Requirements 1.2 and 1.4:
/// - 1.2: All operations go through unified interface
/// - 1.4: Concurrent operation safety with async patterns
#[cfg(test)]
mod interface_tests {
    use super::*;
    use crate::security::{EncryptionType, KeyReference};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock implementation of AccountManagerTrait for testing
    ///
    /// This mock tracks all operations and simulates realistic async behavior.
    struct MockAccountManager {
        accounts: Arc<RwLock<HashMap<Address, SecureAccount>>>,
        current_account: Arc<RwLock<Option<Address>>>,
        locked: Arc<RwLock<bool>>,
        operation_count: Arc<RwLock<u64>>,
    }

    impl MockAccountManager {
        fn new() -> Self {
            Self {
                accounts: Arc::new(RwLock::new(HashMap::new())),
                current_account: Arc::new(RwLock::new(None)),
                locked: Arc::new(RwLock::new(false)),
                operation_count: Arc::new(RwLock::new(0)),
            }
        }

        /// Get the total operation count (for testing concurrent operations)
        async fn get_operation_count(&self) -> u64 {
            *self.operation_count.read().await
        }

        /// Create a test address from a u64 seed
        fn test_address(seed: u64) -> Address {
            let mut bytes = [0u8; 20];
            bytes[12..20].copy_from_slice(&seed.to_be_bytes());
            Address::from(bytes)
        }

        /// Create a test SecureAccount
        fn test_account(address: Address, name: &str) -> SecureAccount {
            SecureAccount {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                address,
                key_reference: KeyReference {
                    id: Uuid::new_v4().to_string(),
                    service: "test".to_string(),
                    account: "test".to_string(),
                },
                created_at: Utc::now(),
                is_hardware: false,
                derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
                tags: Vec::new(),
                last_used: None,
                transaction_count: 0,
            }
        }
    }

    #[async_trait]
    impl AccountManagerTrait for MockAccountManager {
        async fn create_account(
            &mut self,
            config: AccountConfig,
            _password: &SecretString,
        ) -> Result<SecureAccount, AccountError> {
            // Increment operation count
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;

            let address = Self::test_address(Uuid::new_v4().as_u128() as u64);
            let account = Self::test_account(address, &config.name);

            let mut accounts = self.accounts.write().await;
            accounts.insert(address, account.clone());

            Ok(account)
        }

        async fn import_account(
            &mut self,
            source: ImportSource,
        ) -> Result<SecureAccount, AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;

            let name = match &source {
                ImportSource::SeedPhrase { name, .. } => name.clone(),
                ImportSource::PrivateKey { name, .. } => name.clone(),
                ImportSource::MetaMaskKeystore { name, .. } => name.clone(),
            };

            let address = Self::test_address(Uuid::new_v4().as_u128() as u64);
            let account = Self::test_account(address, &name);

            let mut accounts = self.accounts.write().await;
            accounts.insert(address, account.clone());

            Ok(account)
        }

        async fn remove_account(
            &mut self,
            address: Address,
            auth_token: AuthToken,
        ) -> Result<(), AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            if !auth_token.is_valid_for(&AuthorizedOperation::RemoveAccount) {
                return Err(AccountError::invalid_credentials());
            }

            let mut accounts = self.accounts.write().await;
            if accounts.remove(&address).is_none() {
                return Err(AccountError::account_not_found(format!("{}", address)));
            }

            Ok(())
        }

        async fn list_accounts(&self) -> Result<Vec<SecureAccount>, AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;

            let accounts = self.accounts.read().await;
            Ok(accounts.values().cloned().collect())
        }

        async fn get_account(
            &self,
            address: Address,
        ) -> Result<Option<SecureAccount>, AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            let accounts = self.accounts.read().await;
            Ok(accounts.get(&address).cloned())
        }

        async fn get_current_account(&self) -> Result<Option<SecureAccount>, AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            let current = self.current_account.read().await;
            if let Some(address) = *current {
                let accounts = self.accounts.read().await;
                Ok(accounts.get(&address).cloned())
            } else {
                Ok(None)
            }
        }

        async fn set_current_account(&mut self, address: Address) -> Result<(), AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            let accounts = self.accounts.read().await;
            if !accounts.contains_key(&address) {
                return Err(AccountError::account_not_found(format!("{}", address)));
            }
            drop(accounts);

            let mut current = self.current_account.write().await;
            *current = Some(address);

            Ok(())
        }

        async fn lock(&mut self) -> Result<(), AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            let mut locked = self.locked.write().await;
            *locked = true;

            // Clear current account on lock
            let mut current = self.current_account.write().await;
            *current = None;

            Ok(())
        }

        async fn unlock(&mut self, _password: &SecretString) -> Result<(), AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            let mut locked = self.locked.write().await;
            *locked = false;

            Ok(())
        }

        fn is_locked(&self) -> bool {
            // Use try_read to avoid blocking in sync context
            self.locked.try_read().map(|l| *l).unwrap_or(false)
        }

        async fn export_seed(
            &self,
            address: Address,
            _password: &SecretString,
            auth_token: AuthToken,
        ) -> Result<SecureExport, AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            if !auth_token.is_valid_for(&AuthorizedOperation::ExportSeed) {
                return Err(AccountError::invalid_credentials());
            }

            let accounts = self.accounts.read().await;
            if !accounts.contains_key(&address) {
                return Err(AccountError::account_not_found(format!("{}", address)));
            }

            Ok(SecureExport {
                encrypted_data: vec![0u8; 32], // Mock encrypted data
                encryption_type: EncryptionType::Aes256Gcm,
                timestamp: Utc::now().timestamp() as u64,
            })
        }

        async fn export_private_key(
            &self,
            address: Address,
            _password: &SecretString,
            auth_token: AuthToken,
        ) -> Result<SecureExport, AccountError> {
            {
                let mut count = self.operation_count.write().await;
                *count += 1;
            }

            if !auth_token.is_valid_for(&AuthorizedOperation::ExportPrivateKey) {
                return Err(AccountError::invalid_credentials());
            }

            let accounts = self.accounts.read().await;
            if !accounts.contains_key(&address) {
                return Err(AccountError::account_not_found(format!("{}", address)));
            }

            Ok(SecureExport {
                encrypted_data: vec![0u8; 32],
                encryption_type: EncryptionType::Aes256Gcm,
                timestamp: Utc::now().timestamp() as u64,
            })
        }
    }

    // ========== Unit Tests for Unified Interface (Requirement 1.2) ==========

    #[tokio::test]
    async fn test_create_account_through_interface() {
        let mut manager = MockAccountManager::new();
        let config = AccountConfig::seed_based("Test Account");
        let password = SecretString::from("test_password".to_string());

        let account = manager.create_account(config, &password).await;
        assert!(account.is_ok());

        let account = account.unwrap();
        assert_eq!(account.name, "Test Account");
        assert!(!account.is_hardware);
    }

    #[tokio::test]
    async fn test_list_accounts_through_interface() {
        let mut manager = MockAccountManager::new();
        let password = SecretString::from("test".to_string());

        // Create multiple accounts
        for i in 0..3 {
            let config = AccountConfig::seed_based(format!("Account {}", i));
            manager.create_account(config, &password).await.unwrap();
        }

        let accounts = manager.list_accounts().await.unwrap();
        assert_eq!(accounts.len(), 3);
    }

    #[tokio::test]
    async fn test_get_account_through_interface() {
        let mut manager = MockAccountManager::new();
        let config = AccountConfig::seed_based("Find Me");
        let password = SecretString::from("test".to_string());

        let created = manager.create_account(config, &password).await.unwrap();
        let found = manager.get_account(created.address).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Find Me");
    }

    #[tokio::test]
    async fn test_set_and_get_current_account() {
        let mut manager = MockAccountManager::new();
        let config = AccountConfig::seed_based("Current");
        let password = SecretString::from("test".to_string());

        let account = manager.create_account(config, &password).await.unwrap();

        // Initially no current account
        let current = manager.get_current_account().await.unwrap();
        assert!(current.is_none());

        // Set current account
        manager.set_current_account(account.address).await.unwrap();

        // Now should have current account
        let current = manager.get_current_account().await.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().address, account.address);
    }

    #[tokio::test]
    async fn test_lock_unlock_through_interface() {
        let mut manager = MockAccountManager::new();
        let password = SecretString::from("test".to_string());

        assert!(!manager.is_locked());

        manager.lock().await.unwrap();
        assert!(manager.is_locked());

        manager.unlock(&password).await.unwrap();
        assert!(!manager.is_locked());
    }

    #[tokio::test]
    async fn test_export_with_valid_token() {
        let mut manager = MockAccountManager::new();
        let config = AccountConfig::seed_based("Export Test");
        let password = SecretString::from("test".to_string());

        let account = manager.create_account(config, &password).await.unwrap();
        let token = AuthToken::new(AuthorizedOperation::ExportSeed);

        let export = manager.export_seed(account.address, &password, token).await;
        assert!(export.is_ok());
    }

    #[tokio::test]
    async fn test_export_with_invalid_token() {
        let mut manager = MockAccountManager::new();
        let config = AccountConfig::seed_based("Export Test");
        let password = SecretString::from("test".to_string());

        let account = manager.create_account(config, &password).await.unwrap();
        // Use wrong token type
        let token = AuthToken::new(AuthorizedOperation::ExportPrivateKey);

        let export = manager.export_seed(account.address, &password, token).await;
        assert!(export.is_err());
    }

    #[tokio::test]
    async fn test_remove_account_with_auth() {
        let mut manager = MockAccountManager::new();
        let config = AccountConfig::seed_based("Remove Me");
        let password = SecretString::from("test".to_string());

        let account = manager.create_account(config, &password).await.unwrap();
        let token = AuthToken::new(AuthorizedOperation::RemoveAccount);

        // Verify account exists
        let found = manager.get_account(account.address).await.unwrap();
        assert!(found.is_some());

        // Remove account
        manager.remove_account(account.address, token).await.unwrap();

        // Verify account is gone
        let found = manager.get_account(account.address).await.unwrap();
        assert!(found.is_none());
    }

    // ========== Concurrent Operation Safety Tests (Requirement 1.4) ==========

    #[tokio::test]
    async fn test_concurrent_account_creation() {
        let manager = Arc::new(RwLock::new(MockAccountManager::new()));
        let password = SecretString::from("test".to_string());
        let mut handles = Vec::new();

        // Spawn 10 concurrent account creation tasks
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let pwd = password.clone();
            let handle = tokio::spawn(async move {
                let config = AccountConfig::seed_based(format!("Concurrent {}", i));
                let mut mgr = manager_clone.write().await;
                mgr.create_account(config, &pwd).await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Verify all accounts were created
        let mgr = manager.read().await;
        let accounts = mgr.list_accounts().await.unwrap();
        assert_eq!(accounts.len(), 10);
    }

    #[tokio::test]
    async fn test_concurrent_read_operations() {
        let manager = Arc::new(RwLock::new(MockAccountManager::new()));
        let password = SecretString::from("test".to_string());

        // Create some accounts first
        {
            let mut mgr = manager.write().await;
            for i in 0..5 {
                let config = AccountConfig::seed_based(format!("Account {}", i));
                mgr.create_account(config, &password).await.unwrap();
            }
        }

        let mut handles = Vec::new();

        // Spawn 20 concurrent read operations
        for _ in 0..20 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let mgr = manager_clone.read().await;
                mgr.list_accounts().await
            });
            handles.push(handle);
        }

        // All reads should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 5);
        }
    }

    #[tokio::test]
    async fn test_concurrent_mixed_operations() {
        let manager = Arc::new(RwLock::new(MockAccountManager::new()));
        let password = SecretString::from("test".to_string());
        let mut handles = Vec::new();

        // Mix of read and write operations
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let pwd = password.clone();

            if i % 2 == 0 {
                // Write operation
                let handle = tokio::spawn(async move {
                    let config = AccountConfig::seed_based(format!("Mixed {}", i));
                    let mut mgr = manager_clone.write().await;
                    mgr.create_account(config, &pwd).await.map(|_| ())
                });
                handles.push(handle);
            } else {
                // Read operation
                let handle = tokio::spawn(async move {
                    let mgr = manager_clone.read().await;
                    mgr.list_accounts().await.map(|_| ())
                });
                handles.push(handle);
            }
        }

        // All operations should complete without panic or data race
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_operation_count_tracking() {
        let mut manager = MockAccountManager::new();
        let password = SecretString::from("test".to_string());

        // Perform various operations
        let config = AccountConfig::seed_based("Count Test");
        let account = manager.create_account(config, &password).await.unwrap();
        manager.list_accounts().await.unwrap();
        manager.get_account(account.address).await.unwrap();
        manager.set_current_account(account.address).await.unwrap();
        manager.get_current_account().await.unwrap();
        manager.lock().await.unwrap();
        manager.unlock(&password).await.unwrap();

        // Verify operation count
        let count = manager.get_operation_count().await;
        assert_eq!(count, 7);
    }

    #[tokio::test]
    async fn test_import_sources() {
        let mut manager = MockAccountManager::new();

        // Test SeedPhrase import
        let seed_import = ImportSource::SeedPhrase {
            mnemonic: SecretString::from("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()),
            name: "Seed Import".to_string(),
            derivation_path: None,
            password: SecretString::from("test".to_string()),
        };
        let result = manager.import_account(seed_import).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Seed Import");

        // Test PrivateKey import
        let key_import = ImportSource::PrivateKey {
            key: SecretString::from("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()),
            name: "Key Import".to_string(),
            password: SecretString::from("test".to_string()),
        };
        let result = manager.import_account(key_import).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Key Import");

        // Verify both accounts exist
        let accounts = manager.list_accounts().await.unwrap();
        assert_eq!(accounts.len(), 2);
    }
}

/// Property-based tests for concurrent operation safety
///
/// These tests validate **Property 2: Concurrent Operation Safety** from design.md
/// and **Requirement 1.4** from requirements.md:
///
/// *For any* set of concurrent account operations, the system should handle them
/// safely without data races, corruption, or inconsistent state.
///
/// Uses proptest with minimum 100 iterations as specified in design.md.
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::security::{EncryptionType, KeyReference};
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Represents a type of operation that can be performed on the AccountManager
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum OperationType {
        CreateAccount,
        ListAccounts,
        GetAccount,
        SetCurrentAccount,
        GetCurrentAccount,
        Lock,
        Unlock,
    }

    impl OperationType {
        fn is_read(&self) -> bool {
            matches!(
                self,
                OperationType::ListAccounts
                    | OperationType::GetAccount
                    | OperationType::GetCurrentAccount
            )
        }

        fn is_write(&self) -> bool {
            !self.is_read()
        }
    }

    /// Strategy for generating random operation types
    fn operation_type_strategy() -> impl Strategy<Value = OperationType> {
        prop_oneof![
            Just(OperationType::CreateAccount),
            Just(OperationType::ListAccounts),
            Just(OperationType::GetAccount),
            Just(OperationType::SetCurrentAccount),
            Just(OperationType::GetCurrentAccount),
            Just(OperationType::Lock),
            Just(OperationType::Unlock),
        ]
    }

    /// Strategy for generating a sequence of operations
    fn operations_strategy() -> impl Strategy<Value = Vec<OperationType>> {
        prop::collection::vec(operation_type_strategy(), 5..20)
    }

    /// Mock AccountManager for property testing
    /// Simpler version focused on detecting concurrency issues
    struct PropertyTestManager {
        accounts: Arc<RwLock<HashMap<Address, SecureAccount>>>,
        current_account: Arc<RwLock<Option<Address>>>,
        locked: Arc<RwLock<bool>>,
        created_addresses: Arc<RwLock<Vec<Address>>>,
    }

    impl PropertyTestManager {
        fn new() -> Self {
            Self {
                accounts: Arc::new(RwLock::new(HashMap::new())),
                current_account: Arc::new(RwLock::new(None)),
                locked: Arc::new(RwLock::new(false)),
                created_addresses: Arc::new(RwLock::new(Vec::new())),
            }
        }

        fn test_address(seed: u64) -> Address {
            let mut bytes = [0u8; 20];
            bytes[12..20].copy_from_slice(&seed.to_be_bytes());
            Address::from(bytes)
        }

        fn test_account(address: Address, name: &str) -> SecureAccount {
            SecureAccount {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                address,
                key_reference: KeyReference {
                    id: Uuid::new_v4().to_string(),
                    service: "test".to_string(),
                    account: "test".to_string(),
                },
                created_at: Utc::now(),
                is_hardware: false,
                derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
                tags: Vec::new(),
                last_used: None,
                transaction_count: 0,
            }
        }

        async fn create_account(&self, index: usize) -> Result<Address, AccountError> {
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;

            let address = Self::test_address(index as u64);
            let account = Self::test_account(address, &format!("Account {}", index));

            let mut accounts = self.accounts.write().await;
            accounts.insert(address, account);

            let mut created = self.created_addresses.write().await;
            created.push(address);

            Ok(address)
        }

        async fn list_accounts(&self) -> Result<Vec<SecureAccount>, AccountError> {
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            let accounts = self.accounts.read().await;
            Ok(accounts.values().cloned().collect())
        }

        async fn get_account(&self, address: Address) -> Result<Option<SecureAccount>, AccountError> {
            let accounts = self.accounts.read().await;
            Ok(accounts.get(&address).cloned())
        }

        async fn set_current_account(&self, address: Address) -> Result<(), AccountError> {
            let accounts = self.accounts.read().await;
            if accounts.contains_key(&address) {
                drop(accounts);
                let mut current = self.current_account.write().await;
                *current = Some(address);
                Ok(())
            } else {
                Err(AccountError::account_not_found(format!("{}", address)))
            }
        }

        async fn get_current_account(&self) -> Result<Option<SecureAccount>, AccountError> {
            let current = self.current_account.read().await;
            if let Some(addr) = *current {
                let accounts = self.accounts.read().await;
                Ok(accounts.get(&addr).cloned())
            } else {
                Ok(None)
            }
        }

        async fn lock(&self) -> Result<(), AccountError> {
            let mut locked = self.locked.write().await;
            *locked = true;
            let mut current = self.current_account.write().await;
            *current = None;
            Ok(())
        }

        async fn unlock(&self) -> Result<(), AccountError> {
            let mut locked = self.locked.write().await;
            *locked = false;
            Ok(())
        }

        async fn get_created_addresses(&self) -> Vec<Address> {
            self.created_addresses.read().await.clone()
        }

        async fn account_count(&self) -> usize {
            self.accounts.read().await.len()
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 2: Concurrent Operation Safety - No Data Races
        ///
        /// *For any* set of concurrent operations with random ordering,
        /// the system should complete all operations without data races or panics.
        ///
        /// Validates: Requirement 1.4
        #[test]
        fn prop_concurrent_operations_no_data_races(
            operations in operations_strategy()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let all_ok = rt.block_on(async {
                let manager = Arc::new(PropertyTestManager::new());
                let mut handles = Vec::new();

                // First create some accounts so operations have something to work with
                for i in 0..3 {
                    let mgr = Arc::clone(&manager);
                    mgr.create_account(i).await.unwrap();
                }

                // Execute operations concurrently
                for (i, op) in operations.iter().enumerate() {
                    let mgr = Arc::clone(&manager);
                    let op = *op;

                    let handle = tokio::spawn(async move {
                        match op {
                            OperationType::CreateAccount => {
                                let _ = mgr.create_account(100 + i).await;
                            }
                            OperationType::ListAccounts => {
                                let _ = mgr.list_accounts().await;
                            }
                            OperationType::GetAccount => {
                                let addrs = mgr.get_created_addresses().await;
                                if !addrs.is_empty() {
                                    let _ = mgr.get_account(addrs[0]).await;
                                }
                            }
                            OperationType::SetCurrentAccount => {
                                let addrs = mgr.get_created_addresses().await;
                                if !addrs.is_empty() {
                                    let _ = mgr.set_current_account(addrs[0]).await;
                                }
                            }
                            OperationType::GetCurrentAccount => {
                                let _ = mgr.get_current_account().await;
                            }
                            OperationType::Lock => {
                                let _ = mgr.lock().await;
                            }
                            OperationType::Unlock => {
                                let _ = mgr.unlock().await;
                            }
                        }
                    });
                    handles.push(handle);
                }

                // All operations should complete without panic
                let mut success = true;
                for handle in handles {
                    if handle.await.is_err() {
                        success = false;
                    }
                }
                success
            });
            prop_assert!(all_ok, "At least one concurrent operation panicked");
        }

        /// Property 2: Concurrent Operation Safety - State Consistency
        ///
        /// *For any* set of concurrent account creations, the final account count
        /// should equal the number of successful creations.
        ///
        /// Validates: Requirement 1.4
        #[test]
        fn prop_concurrent_creations_maintain_consistency(
            num_creations in 5u32..15
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (final_count, success_count) = rt.block_on(async {
                let manager = Arc::new(PropertyTestManager::new());
                let mut handles = Vec::new();

                // Spawn concurrent account creations
                for i in 0..num_creations {
                    let mgr = Arc::clone(&manager);
                    let handle = tokio::spawn(async move {
                        mgr.create_account(i as usize).await
                    });
                    handles.push(handle);
                }

                // Count successful creations
                let mut success = 0u32;
                for handle in handles {
                    if let Ok(Ok(_)) = handle.await {
                        success += 1;
                    }
                }

                // Get final account count
                let count = manager.account_count().await;
                (count as u32, success)
            });
            prop_assert_eq!(
                final_count,
                success_count,
                "Account count {} != success count {}",
                final_count,
                success_count
            );
        }

        /// Property 2: Concurrent Operation Safety - Read Operations Non-Blocking
        ///
        /// *For any* number of concurrent read operations, they should all
        /// complete successfully and return consistent data.
        ///
        /// Validates: Requirement 1.4
        #[test]
        fn prop_concurrent_reads_succeed(
            num_reads in 10u32..30
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (all_ok, all_correct_len) = rt.block_on(async {
                let manager = Arc::new(PropertyTestManager::new());

                // Create initial accounts
                for i in 0..5 {
                    manager.create_account(i).await.unwrap();
                }

                let mut handles = Vec::new();

                // Spawn concurrent read operations
                for _ in 0..num_reads {
                    let mgr = Arc::clone(&manager);
                    let handle = tokio::spawn(async move {
                        mgr.list_accounts().await
                    });
                    handles.push(handle);
                }

                // All reads should succeed and return same count
                let mut all_success = true;
                let mut all_len_5 = true;
                for handle in handles {
                    match handle.await {
                        Ok(Ok(accounts)) => {
                            if accounts.len() != 5 {
                                all_len_5 = false;
                            }
                        }
                        _ => {
                            all_success = false;
                        }
                    }
                }
                (all_success, all_len_5)
            });
            prop_assert!(all_ok, "Some read operations failed");
            prop_assert!(all_correct_len, "Some reads returned wrong account count");
        }

        /// Property 2: Concurrent Operation Safety - Lock State Atomicity
        ///
        /// *For any* sequence of lock/unlock operations, the final locked state
        /// should reflect the last operation.
        ///
        /// Validates: Requirement 1.4
        #[test]
        fn prop_lock_unlock_atomic(
            final_locked in prop::bool::ANY
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let is_locked = rt.block_on(async {
                let manager = Arc::new(PropertyTestManager::new());

                // Perform lock or unlock as final op
                if final_locked {
                    manager.lock().await.unwrap();
                } else {
                    manager.unlock().await.unwrap();
                }

                // Check state is consistent
                let locked_state = *manager.locked.read().await;
                locked_state
            });
            prop_assert_eq!(is_locked, final_locked);
        }
    }
}

/// Property-based tests for lock memory clearing and unlock restoration
///
/// These tests validate:
/// - **Property 3: Lock Memory Clearing** (Requirements 2.3)
/// - **Property 4: Unlock Restoration** (Requirements 2.4)
///
/// Property 3: *For any* wallet state, when a lock operation is performed,
/// all sensitive data should be cleared and subsequent operations requiring
/// authentication should fail.
///
/// Property 4: *For any* locked wallet, when unlocked with correct credentials,
/// all account operations should become available again.
#[cfg(test)]
mod lock_property_tests {
    use super::*;
    use crate::security::KeyReference;
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// A wallet-like structure that mirrors production locking behavior
    /// Used for property testing the lock/unlock semantics
    struct LockableWallet {
        accounts: Arc<RwLock<HashMap<Address, SecureAccount>>>,
        current_account: Arc<RwLock<Option<SecureAccount>>>,
        locked: Arc<RwLock<bool>>,
    }

    impl LockableWallet {
        fn new() -> Self {
            Self {
                accounts: Arc::new(RwLock::new(HashMap::new())),
                current_account: Arc::new(RwLock::new(None)),
                locked: Arc::new(RwLock::new(false)),
            }
        }

        fn test_address(seed: u64) -> Address {
            let mut bytes = [0u8; 20];
            bytes[12..20].copy_from_slice(&seed.to_be_bytes());
            Address::from(bytes)
        }

        fn test_account(address: Address, name: &str) -> SecureAccount {
            SecureAccount {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                address,
                key_reference: KeyReference {
                    id: Uuid::new_v4().to_string(),
                    service: "test".to_string(),
                    account: "test".to_string(),
                },
                created_at: Utc::now(),
                is_hardware: false,
                derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
                tags: Vec::new(),
                last_used: None,
                transaction_count: 0,
            }
        }

        /// Add an account to the wallet
        async fn add_account(&self, seed: u64) -> SecureAccount {
            let address = Self::test_address(seed);
            let account = Self::test_account(address, &format!("Account {}", seed));

            let mut accounts = self.accounts.write().await;
            accounts.insert(address, account.clone());

            account
        }

        /// Set the current account (simulates unlocking with an account)
        async fn set_current_account(&self, account: SecureAccount) {
            let mut current = self.current_account.write().await;
            *current = Some(account);

            let mut locked = self.locked.write().await;
            *locked = false;
        }

        /// Lock the wallet (production behavior)
        /// Clears current_account and sets locked = true
        async fn lock(&self) {
            // Clear current account (sensitive data reference)
            {
                let mut current = self.current_account.write().await;
                *current = None;
            }

            // Set locked state
            {
                let mut locked = self.locked.write().await;
                *locked = true;
            }
        }

        /// Unlock the wallet with an address
        async fn unlock(&self, address: Address) -> Result<(), AccountError> {
            let accounts = self.accounts.read().await;
            let account = accounts
                .get(&address)
                .cloned()
                .ok_or_else(|| AccountError::account_not_found(format!("{}", address)))?;

            {
                let mut current = self.current_account.write().await;
                *current = Some(account);
            }

            {
                let mut locked = self.locked.write().await;
                *locked = false;
            }

            Ok(())
        }

        /// Check if wallet is locked
        async fn is_locked(&self) -> bool {
            *self.locked.read().await
        }

        /// Get current account (only works when unlocked)
        async fn get_current_account(&self) -> Result<SecureAccount, AccountError> {
            let locked = self.locked.read().await;
            if *locked {
                return Err(AccountError::account_locked());
            }
            drop(locked);

            let current = self.current_account.read().await;
            current
                .clone()
                .ok_or_else(|| AccountError::account_not_found("No current account".to_string()))
        }

        /// Operation that requires authentication (simulates signing, export, etc.)
        async fn authenticated_operation(&self) -> Result<String, AccountError> {
            let locked = self.locked.read().await;
            if *locked {
                return Err(AccountError::account_locked());
            }
            drop(locked);

            let current = self.current_account.read().await;
            if current.is_none() {
                return Err(AccountError::account_not_found("No current account".to_string()));
            }

            Ok("Operation successful".to_string())
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 3: Lock Memory Clearing - Current Account Cleared
        ///
        /// *For any* wallet with an active current account, when locked,
        /// the current account should be None.
        ///
        /// Validates: Requirement 2.3
        #[test]
        fn prop_lock_clears_current_account(
            account_seed in 1u64..1000
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let current_after_lock = rt.block_on(async {
                let wallet = LockableWallet::new();

                // Create and set an account as current
                let account = wallet.add_account(account_seed).await;
                wallet.set_current_account(account).await;

                // Verify account is set
                let before_lock = wallet.get_current_account().await;
                assert!(before_lock.is_ok(), "Should have current account before lock");

                // Lock the wallet
                wallet.lock().await;

                // Try to get current account after lock
                wallet.get_current_account().await
            });

            // After lock, getting current account should fail
            prop_assert!(
                current_after_lock.is_err(),
                "Current account should be cleared after lock"
            );
        }

        /// Property 3: Lock Memory Clearing - Authenticated Operations Fail
        ///
        /// *For any* wallet state, when locked, operations requiring
        /// authentication should fail with an appropriate error.
        ///
        /// Validates: Requirement 2.3
        #[test]
        fn prop_lock_prevents_authenticated_operations(
            account_seed in 1u64..1000
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (before_result, after_result) = rt.block_on(async {
                let wallet = LockableWallet::new();

                // Setup: create account and set as current
                let account = wallet.add_account(account_seed).await;
                wallet.set_current_account(account).await;

                // Authenticated operation should work before lock
                let before = wallet.authenticated_operation().await;

                // Lock the wallet
                wallet.lock().await;

                // Authenticated operation should fail after lock
                let after = wallet.authenticated_operation().await;

                (before, after)
            });

            prop_assert!(before_result.is_ok(), "Operation should succeed before lock");
            prop_assert!(after_result.is_err(), "Operation should fail after lock");
        }

        /// Property 3: Lock Memory Clearing - Lock State Is Set
        ///
        /// *For any* unlocked wallet, after lock is called,
        /// is_locked() should return true.
        ///
        /// Validates: Requirement 2.3
        #[test]
        fn prop_lock_sets_locked_state(
            account_seed in 1u64..1000
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (before_locked, after_locked) = rt.block_on(async {
                let wallet = LockableWallet::new();

                // Setup
                let account = wallet.add_account(account_seed).await;
                wallet.set_current_account(account).await;

                let before = wallet.is_locked().await;

                // Lock
                wallet.lock().await;

                let after = wallet.is_locked().await;

                (before, after)
            });

            prop_assert!(!before_locked, "Should not be locked before lock()");
            prop_assert!(after_locked, "Should be locked after lock()");
        }

        /// Property 4: Unlock Restoration - Account Access Restored
        ///
        /// *For any* locked wallet, when unlocked with correct credentials,
        /// account operations should become available again.
        ///
        /// Validates: Requirement 2.4
        #[test]
        fn prop_unlock_restores_account_access(
            account_seed in 1u64..1000
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let unlock_result = rt.block_on(async {
                let wallet = LockableWallet::new();

                // Create account
                let account = wallet.add_account(account_seed).await;
                let address = account.address;

                // Set as current, then lock
                wallet.set_current_account(account).await;
                wallet.lock().await;

                // Verify locked
                assert!(wallet.is_locked().await);
                assert!(wallet.get_current_account().await.is_err());

                // Unlock with correct address
                wallet.unlock(address).await.unwrap();

                // Should now have access
                (
                    wallet.is_locked().await,
                    wallet.get_current_account().await,
                    wallet.authenticated_operation().await,
                )
            });

            let (is_locked, current_account, auth_op) = unlock_result;
            prop_assert!(!is_locked, "Should not be locked after unlock");
            prop_assert!(current_account.is_ok(), "Should have current account after unlock");
            prop_assert!(auth_op.is_ok(), "Authenticated operation should work after unlock");
        }

        /// Property 4: Unlock Restoration - Same Account Restored
        ///
        /// *For any* locked wallet, when unlocked, the current account
        /// should be the account used for unlocking.
        ///
        /// Validates: Requirement 2.4
        #[test]
        fn prop_unlock_sets_correct_account(
            account_seed in 1u64..1000
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let addresses_match = rt.block_on(async {
                let wallet = LockableWallet::new();

                // Create account
                let account = wallet.add_account(account_seed).await;
                let original_address = account.address;

                // Set as current, then lock
                wallet.set_current_account(account).await;
                wallet.lock().await;

                // Unlock
                wallet.unlock(original_address).await.unwrap();

                // Get current account and compare
                let current = wallet.get_current_account().await.unwrap();
                current.address == original_address
            });

            prop_assert!(addresses_match, "Current account address should match unlock address");
        }

        /// Property 4: Unlock Restoration - Operations Work After Lock/Unlock Cycle
        ///
        /// *For any* wallet that goes through lock/unlock cycle,
        /// all account operations should work identically.
        ///
        /// Validates: Requirement 2.4
        #[test]
        fn prop_operations_work_after_lock_unlock_cycle(
            cycles in 1u32..5
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let all_cycles_work = rt.block_on(async {
                let wallet = LockableWallet::new();

                // Create account
                let account = wallet.add_account(1).await;
                let address = account.address;
                wallet.set_current_account(account).await;

                let mut all_ok = true;

                for _ in 0..cycles {
                    // Lock
                    wallet.lock().await;
                    if !wallet.is_locked().await {
                        all_ok = false;
                        break;
                    }

                    // Operations should fail while locked
                    if wallet.authenticated_operation().await.is_ok() {
                        all_ok = false;
                        break;
                    }

                    // Unlock
                    wallet.unlock(address).await.unwrap();
                    if wallet.is_locked().await {
                        all_ok = false;
                        break;
                    }

                    // Operations should work after unlock
                    if wallet.authenticated_operation().await.is_err() {
                        all_ok = false;
                        break;
                    }
                }

                all_ok
            });

            prop_assert!(all_cycles_work, "All lock/unlock cycles should work correctly");
        }
    }
}
