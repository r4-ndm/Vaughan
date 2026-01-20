//! Account Creation Module
//!
//! This module provides account creation capabilities using Alloy's signer API.
//! Implements unified account creation interface for the enhanced account management system.
//!
//! # Requirements Addressed
//!
//! - **Requirement 1.1**: Unified interface for account creation operations
//!
//! # Design Principles
//!
//! - **Alloy-First**: Uses `PrivateKeySigner` for all signer operations
//! - **Correlation Tracking**: All operations include correlation IDs
//! - **Validation**: Comprehensive input validation before creation
//! - **Security**: Zeroize sensitive data after use

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use bip32::{secp256k1::SecretKey, DerivationPath, ExtendedPrivateKey};
use bip39::Mnemonic;
use chrono::{DateTime, Utc};
use k256::ecdsa::SigningKey;
use secrecy::{ExposeSecret, SecretString};
use std::str::FromStr;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::error::{Result, SecurityError};
use crate::telemetry::{AccountLogger, OperationSpan, PrivacyMode};

/// Default Ethereum derivation path (BIP-44)
pub const DEFAULT_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";

/// Configuration for account creation
#[derive(Debug, Clone)]
pub struct AccountCreationConfig {
    /// Derivation path for HD wallet derivation
    pub derivation_path: String,
    /// Account name/label
    pub name: Option<String>,
    /// Whether to generate additional derived accounts
    pub derive_additional: u32,
}

impl Default for AccountCreationConfig {
    fn default() -> Self {
        Self {
            derivation_path: DEFAULT_DERIVATION_PATH.to_string(),
            name: None,
            derive_additional: 0,
        }
    }
}

impl AccountCreationConfig {
    /// Create config with custom derivation path
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            derivation_path: path.into(),
            ..Default::default()
        }
    }

    /// Set account name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Result of account creation operation
#[derive(Debug, Clone)]
pub struct CreatedAccount {
    /// The wallet address
    pub address: Address,
    /// Derivation path used (if from seed)
    pub derivation_path: Option<String>,
    /// Account name/label
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
    /// Account index (for HD derived accounts)
    pub account_index: Option<u32>,
}

impl CreatedAccount {
    /// Create a new CreatedAccount
    pub fn new(address: Address, correlation_id: Uuid) -> Self {
        Self {
            address,
            derivation_path: None,
            name: None,
            created_at: Utc::now(),
            correlation_id,
            account_index: None,
        }
    }

    /// Set derivation path
    pub fn with_derivation_path(mut self, path: String) -> Self {
        self.derivation_path = Some(path);
        self
    }

    /// Set account name
    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    /// Set account index
    pub fn with_index(mut self, index: u32) -> Self {
        self.account_index = Some(index);
        self
    }
}

/// Account creator with Alloy-based implementation
///
/// Provides unified account creation from various sources:
/// - Seed phrases (BIP-39 mnemonic)
/// - Private keys (raw hex or bytes)
/// - Random generation
#[derive(Debug)]
pub struct AccountCreator {
    /// Logger for structured logging with correlation tracking
    logger: AccountLogger,
}

impl AccountCreator {
    /// Create a new AccountCreator with default settings
    pub fn new() -> Self {
        Self {
            logger: AccountLogger::new(PrivacyMode::Enabled),
        }
    }

    /// Create with custom logger settings
    pub fn with_logger(logger: AccountLogger) -> Self {
        Self { logger }
    }

    /// Create account from BIP-39 seed phrase
    ///
    /// Uses Alloy's PrivateKeySigner with BIP-32 HD derivation.
    /// 
    /// # Arguments
    /// * `phrase` - BIP-39 mnemonic phrase (12, 15, 18, 21, or 24 words)
    /// * `passphrase` - Optional BIP-39 passphrase
    /// * `config` - Account creation configuration
    ///
    /// # Returns
    /// Created account details and the signer (kept separate for security)
    pub fn create_from_seed(
        &self,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
        config: &AccountCreationConfig,
    ) -> Result<(CreatedAccount, PrivateKeySigner)> {
        let span = OperationSpan::new("create_from_seed");
        self.logger.log_operation_start(&span, "Creating account from seed phrase");

        // Validate seed phrase
        let phrase_str = phrase.expose_secret();
        let mnemonic = match Mnemonic::parse(phrase_str) {
            Ok(m) => m,
            Err(e) => {
                self.logger.log_operation_error(&span, &format!("Invalid mnemonic: {}", e));
                return Err(SecurityError::InvalidSeedPhrase {
                    reason: format!("Invalid mnemonic: {e}"),
                }.into());
            }
        };

        // Validate derivation path
        let path = match DerivationPath::from_str(&config.derivation_path) {
            Ok(p) => p,
            Err(e) => {
                self.logger.log_operation_error(&span, &format!("Invalid derivation path: {}", e));
                return Err(SecurityError::KeyDerivationError {
                    message: format!("Invalid derivation path '{}': {e}", config.derivation_path),
                }.into());
            }
        };

        // Generate seed from mnemonic
        let bip39_pass = passphrase.map(|p| p.expose_secret().as_str()).unwrap_or("");
        let seed = mnemonic.to_seed(bip39_pass);

        // Create master extended private key
        let mut xprv = ExtendedPrivateKey::<SecretKey>::new(seed)
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Failed to create master key: {e}"),
            })?;

        // Derive along the path
        for child in path.into_iter() {
            xprv = xprv.derive_child(child).map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Failed to derive child key: {e}"),
            })?;
        }

        // Build signing key from derived secret
        let secret_bytes = xprv.private_key().to_bytes();
        let signing_key = SigningKey::from_bytes(&secret_bytes)
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Failed to build signing key: {e}"),
            })?;

        // Create Alloy PrivateKeySigner
        let signer = PrivateKeySigner::from(signing_key);
        let address = signer.address();

        let account = CreatedAccount::new(address, span.correlation_id)
            .with_derivation_path(config.derivation_path.clone())
            .with_name(config.name.clone())
            .with_index(0);

        self.logger.log_account_event(
            &span,
            "account_created",
            &address.to_string(),
            "Account created from seed phrase",
        );
        self.logger.log_operation_complete(&span, "Account created successfully");

        Ok((account, signer))
    }

    /// Create account from raw private key
    ///
    /// Uses Alloy's PrivateKeySigner directly from private key bytes.
    ///
    /// # Arguments
    /// * `private_key` - Private key as hex string (with or without 0x prefix)
    /// * `config` - Account creation configuration
    ///
    /// # Returns
    /// Created account details and the signer
    pub fn create_from_private_key(
        &self,
        private_key: &SecretString,
        config: &AccountCreationConfig,
    ) -> Result<(CreatedAccount, PrivateKeySigner)> {
        let span = OperationSpan::new("create_from_private_key");
        self.logger.log_operation_start(&span, "Creating account from private key");

        let key_str = private_key.expose_secret();

        // Parse private key (handle 0x prefix)
        let signer: PrivateKeySigner = key_str.parse().map_err(|_| {
            self.logger.log_operation_error(&span, "Invalid private key format");
            SecurityError::KeyDerivationError {
                message: "Failed to parse private key. Expected 32-byte hex string.".to_string(),
            }
        })?;

        let address = signer.address();

        let account = CreatedAccount::new(address, span.correlation_id)
            .with_name(config.name.clone());

        self.logger.log_account_event(
            &span,
            "account_created",
            &address.to_string(),
            "Account created from private key",
        );
        self.logger.log_operation_complete(&span, "Account created successfully");

        Ok((account, signer))
    }

    /// Create account from private key bytes
    ///
    /// # Arguments
    /// * `key_bytes` - 32-byte private key
    /// * `config` - Account creation configuration
    pub fn create_from_bytes(
        &self,
        key_bytes: &[u8; 32],
        config: &AccountCreationConfig,
    ) -> Result<(CreatedAccount, PrivateKeySigner)> {
        let span = OperationSpan::new("create_from_bytes");
        self.logger.log_operation_start(&span, "Creating account from key bytes");

        // Use Alloy's from_slice for byte array
        let signer = PrivateKeySigner::from_slice(key_bytes).map_err(|e| {
            self.logger.log_operation_error(&span, &format!("Invalid key bytes: {}", e));
            SecurityError::KeyDerivationError {
                message: format!("Failed to create signer from bytes: {e}"),
            }
        })?;

        let address = signer.address();

        let account = CreatedAccount::new(address, span.correlation_id)
            .with_name(config.name.clone());

        self.logger.log_account_event(
            &span,
            "account_created",
            &address.to_string(),
            "Account created from key bytes",
        );
        self.logger.log_operation_complete(&span, "Account created successfully");

        Ok((account, signer))
    }

    /// Create multiple accounts from a single seed phrase
    ///
    /// Derives multiple accounts using incremental account indices.
    ///
    /// # Arguments
    /// * `phrase` - BIP-39 mnemonic phrase
    /// * `passphrase` - Optional BIP-39 passphrase
    /// * `base_path` - Base derivation path (e.g., "m/44'/60'/0'/0")
    /// * `count` - Number of accounts to derive
    pub fn create_multiple_from_seed(
        &self,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
        base_path: &str,
        count: u32,
    ) -> Result<Vec<(CreatedAccount, PrivateKeySigner)>> {
        let span = OperationSpan::new("create_multiple_from_seed");
        self.logger.log_operation_start(&span, &format!("Creating {} accounts from seed", count));

        // Validate seed phrase first
        let phrase_str = phrase.expose_secret();
        let mnemonic = Mnemonic::parse(phrase_str).map_err(|e| {
            SecurityError::InvalidSeedPhrase {
                reason: format!("Invalid mnemonic: {e}"),
            }
        })?;

        let bip39_pass = passphrase.map(|p| p.expose_secret().as_str()).unwrap_or("");
        let seed = mnemonic.to_seed(bip39_pass);

        let mut accounts = Vec::with_capacity(count as usize);

        for index in 0..count {
            let account_path = format!("{}/{}", base_path, index);
            
            // Parse and validate path
            let path = DerivationPath::from_str(&account_path).map_err(|e| {
                SecurityError::KeyDerivationError {
                    message: format!("Invalid path '{}': {e}", account_path),
                }
            })?;

            // Derive key
            let mut xprv = ExtendedPrivateKey::<SecretKey>::new(seed.clone())
                .map_err(|e| SecurityError::KeyDerivationError {
                    message: format!("Failed to create master key: {e}"),
                })?;

            for child in path.into_iter() {
                xprv = xprv.derive_child(child).map_err(|e| SecurityError::KeyDerivationError {
                    message: format!("Failed to derive child key: {e}"),
                })?;
            }

            let secret_bytes = xprv.private_key().to_bytes();
            let signing_key = SigningKey::from_bytes(&secret_bytes)
                .map_err(|e| SecurityError::KeyDerivationError {
                    message: format!("Failed to build signing key: {e}"),
                })?;

            let signer = PrivateKeySigner::from(signing_key);
            let address = signer.address();

            let account = CreatedAccount::new(address, span.correlation_id)
                .with_derivation_path(account_path)
                .with_index(index);

            accounts.push((account, signer));
        }

        self.logger.log_operation_complete(&span, &format!("Created {} accounts", count));

        Ok(accounts)
    }

    /// Validate a seed phrase without creating an account
    pub fn validate_seed_phrase(&self, phrase: &SecretString) -> Result<SeedValidation> {
        let phrase_str = phrase.expose_secret();
        let words: Vec<&str> = phrase_str.split_whitespace().collect();
        let word_count = words.len();

        // Check word count
        if ![12, 15, 18, 21, 24].contains(&word_count) {
            return Ok(SeedValidation {
                is_valid: false,
                word_count,
                error: Some(format!("Invalid word count: {}. Expected 12, 15, 18, 21, or 24.", word_count)),
            });
        }

        // Try to parse as BIP-39 mnemonic
        match Mnemonic::parse(phrase_str) {
            Ok(_) => Ok(SeedValidation {
                is_valid: true,
                word_count,
                error: None,
            }),
            Err(e) => Ok(SeedValidation {
                is_valid: false,
                word_count,
                error: Some(format!("Invalid mnemonic: {}", e)),
            }),
        }
    }

    /// Validate a private key without creating an account
    pub fn validate_private_key(&self, key: &SecretString) -> Result<KeyValidation> {
        let key_str = key.expose_secret();

        // Check format
        let hex_str = key_str.strip_prefix("0x").unwrap_or(key_str);

        if hex_str.len() != 64 {
            return Ok(KeyValidation {
                is_valid: false,
                error: Some(format!("Invalid length: expected 64 hex chars, got {}", hex_str.len())),
            });
        }

        // Try to parse
        match key_str.parse::<PrivateKeySigner>() {
            Ok(_) => Ok(KeyValidation {
                is_valid: true,
                error: None,
            }),
            Err(_) => Ok(KeyValidation {
                is_valid: false,
                error: Some("Failed to parse as valid private key".to_string()),
            }),
        }
    }
}

impl Default for AccountCreator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of seed phrase validation
#[derive(Debug, Clone)]
pub struct SeedValidation {
    /// Whether the seed is valid
    pub is_valid: bool,
    /// Number of words in the phrase
    pub word_count: usize,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Result of private key validation
#[derive(Debug, Clone)]
pub struct KeyValidation {
    /// Whether the key is valid
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test mnemonic (DO NOT use in production - this is a well-known test phrase)
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn test_create_from_seed() {
        let creator = AccountCreator::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let config = AccountCreationConfig::default();

        let result = creator.create_from_seed(&phrase, None, &config);
        assert!(result.is_ok());

        let (account, signer) = result.unwrap();
        assert!(!account.address.is_zero());
        assert_eq!(account.derivation_path, Some(DEFAULT_DERIVATION_PATH.to_string()));
        assert_eq!(signer.address(), account.address);
    }

    #[test]
    fn test_create_from_seed_custom_path() {
        let creator = AccountCreator::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let config = AccountCreationConfig::with_path("m/44'/60'/0'/0/1");

        let result = creator.create_from_seed(&phrase, None, &config);
        assert!(result.is_ok());

        let (account, _) = result.unwrap();
        assert_eq!(account.derivation_path, Some("m/44'/60'/0'/0/1".to_string()));
    }

    #[test]
    fn test_same_seed_same_address() {
        let creator = AccountCreator::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let config = AccountCreationConfig::default();

        let (account1, _) = creator.create_from_seed(&phrase, None, &config).unwrap();
        let (account2, _) = creator.create_from_seed(&phrase, None, &config).unwrap();

        assert_eq!(account1.address, account2.address);
    }

    #[test]
    fn test_create_from_private_key() {
        let creator = AccountCreator::new();
        // Well-known test private key (DO NOT use in production)
        let key = SecretString::from("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string());
        let config = AccountCreationConfig::default();

        let result = creator.create_from_private_key(&key, &config);
        assert!(result.is_ok());

        let (account, signer) = result.unwrap();
        assert!(!account.address.is_zero());
        assert_eq!(signer.address(), account.address);
    }

    #[test]
    fn test_create_from_private_key_no_prefix() {
        let creator = AccountCreator::new();
        let key = SecretString::from("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string());
        let config = AccountCreationConfig::default();

        let result = creator.create_from_private_key(&key, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_seed_phrase() {
        let creator = AccountCreator::new();
        let phrase = SecretString::from("invalid seed phrase".to_string());
        let config = AccountCreationConfig::default();

        let result = creator.create_from_seed(&phrase, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_private_key() {
        let creator = AccountCreator::new();
        let key = SecretString::from("not a valid key".to_string());
        let config = AccountCreationConfig::default();

        let result = creator.create_from_private_key(&key, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_multiple_from_seed() {
        let creator = AccountCreator::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());

        let result = creator.create_multiple_from_seed(
            &phrase,
            None,
            "m/44'/60'/0'/0",
            3,
        );
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 3);

        // All addresses should be different
        let addresses: Vec<_> = accounts.iter().map(|(a, _)| a.address).collect();
        let mut unique = addresses.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_validate_seed_phrase() {
        let creator = AccountCreator::new();

        // Valid phrase
        let valid = SecretString::from(TEST_MNEMONIC.to_string());
        let result = creator.validate_seed_phrase(&valid).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.word_count, 12);

        // Invalid phrase
        let invalid = SecretString::from("invalid phrase here".to_string());
        let result = creator.validate_seed_phrase(&invalid).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_private_key() {
        let creator = AccountCreator::new();

        // Valid key
        let valid = SecretString::from("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string());
        let result = creator.validate_private_key(&valid).unwrap();
        assert!(result.is_valid);

        // Invalid key
        let invalid = SecretString::from("invalid".to_string());
        let result = creator.validate_private_key(&invalid).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_create_from_bytes() {
        let creator = AccountCreator::new();
        let mut key_bytes = [0u8; 32];
        // Use a valid test key
        key_bytes[31] = 1; // Minimal valid key
        let config = AccountCreationConfig::default();

        let result = creator.create_from_bytes(&key_bytes, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_account_with_name() {
        let creator = AccountCreator::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let config = AccountCreationConfig::default().with_name("My Test Account");

        let (account, _) = creator.create_from_seed(&phrase, None, &config).unwrap();
        assert_eq!(account.name, Some("My Test Account".to_string()));
    }
}
