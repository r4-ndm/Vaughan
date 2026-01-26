//! Account Manager Type Definitions
//!
//! This module contains all type definitions used by the Account Manager,
//! including configuration types, enums, and authentication tokens.
//!
//! ## Design Principles
//! - **Alloy-First**: Uses Alloy primitives for blockchain operations
//! - **Type Safety**: Strong typing for all account operations
//! - **Security**: Authentication tokens for sensitive operations

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Import source type for tracking account origin
///
/// Used to identify how an account was imported into the wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportSourceType {
    /// Imported from BIP39 seed phrase
    SeedPhrase,
    /// Imported from raw private key
    PrivateKey,
    /// Imported from EIP-2335 keystore file
    Keystore,
    /// Unknown or unspecified import source
    Unknown,
}

/// Metadata for imported accounts
///
/// Contains additional information about imported accounts
/// such as name, tags, and import timestamp.
#[derive(Debug, Clone)]
pub struct ImportMetadata {
    /// Human-readable account name
    pub name: Option<String>,
    /// Tags for organizing accounts
    pub tags: Vec<String>,
    /// Import timestamp
    pub imported_at: DateTime<Utc>,
}

impl ImportMetadata {
    /// Create new import metadata with default values
    pub fn new() -> Self {
        Self {
            name: None,
            tags: Vec::new(),
            imported_at: Utc::now(),
        }
    }

    /// Set the account name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

impl Default for ImportMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified Account structure for import operations
///
/// This is a temporary structure used during import.
/// The full Account structure is defined in wallet::account.
#[derive(Debug, Clone)]
pub struct Account {
    /// Account address
    pub address: alloy::primitives::Address,
    /// Import source type
    pub source_type: ImportSourceType,
    /// Account metadata
    pub metadata: ImportMetadata,
}

impl Account {
    /// Create a new imported account
    pub fn new_imported(
        address: alloy::primitives::Address,
        source_type: ImportSourceType,
        metadata: ImportMetadata,
    ) -> Self {
        Self {
            address,
            source_type,
            metadata,
        }
    }
}
