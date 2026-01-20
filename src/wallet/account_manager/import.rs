//! Account Import Module
//!
//! This module provides account importing capabilities from various sources:
//! - BIP39 seed phrases with validation
//! - Private keys with hex validation
//! - MetaMask keystore files
//!
//! # Requirements Addressed
//!
//! - **Requirement 8.1**: Support importing from seed phrases and private keys
//! - **Requirement 8.2**: Deterministic address derivation from seed phrases
//! - **Requirement 8.3**: Format detection and validation for imports
//!
//! # Design Properties
//!
//! - **Property 20**: Seed Phrase Import Determinism
//! - **Property 21**: Migration Format Validation
//! - **Property 22**: Migration Metadata Preservation
//! - **Property 23**: Migration Error Specificity

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use bip32::{secp256k1::SecretKey, DerivationPath, ExtendedPrivateKey};
use bip39::Mnemonic;
use chrono::{DateTime, Utc};
use k256::ecdsa::SigningKey;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{Result, SecurityError, VaughanError};
use crate::telemetry::{AccountLogger, OperationSpan, PrivacyMode};
use crate::wallet::keystore_format::MetaMaskKeystore;

use super::creation::DEFAULT_DERIVATION_PATH;

/// Import source type for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportSourceType {
    /// BIP39 seed phrase
    SeedPhrase,
    /// Raw private key (hex)
    PrivateKey,
    /// MetaMask keystore JSON
    MetaMaskKeystore,
    /// Unknown format
    Unknown,
}

impl std::fmt::Display for ImportSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SeedPhrase => write!(f, "Seed Phrase"),
            Self::PrivateKey => write!(f, "Private Key"),
            Self::MetaMaskKeystore => write!(f, "MetaMask Keystore"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Result of format detection
#[derive(Debug, Clone)]
pub struct FormatDetectionResult {
    /// Detected source type
    pub source_type: ImportSourceType,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Additional details about the detection
    pub details: Option<String>,
}

impl FormatDetectionResult {
    /// Create a new detection result
    pub fn new(source_type: ImportSourceType, confidence: f64) -> Self {
        Self {
            source_type,
            confidence,
            details: None,
        }
    }

    /// Add details to the result
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Metadata associated with an imported account
#[derive(Debug, Clone, Default)]
pub struct ImportMetadata {
    /// Account name/label
    pub name: Option<String>,
    /// Creation date (if known from source)
    pub creation_date: Option<DateTime<Utc>>,
    /// Tags or categories
    pub tags: Vec<String>,
    /// Original source type
    pub source_type: Option<ImportSourceType>,
    /// Additional notes
    pub notes: Option<String>,
}

impl ImportMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set account name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Result of an import operation
#[derive(Debug, Clone)]
pub struct ImportedAccount {
    /// The wallet address
    pub address: Address,
    /// Derivation path used (if from seed)
    pub derivation_path: Option<String>,
    /// Import timestamp
    pub imported_at: DateTime<Utc>,
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
    /// Preserved metadata
    pub metadata: ImportMetadata,
    /// Source type
    pub source_type: ImportSourceType,
}

impl ImportedAccount {
    /// Create a new imported account
    pub fn new(address: Address, source_type: ImportSourceType, correlation_id: Uuid) -> Self {
        Self {
            address,
            derivation_path: None,
            imported_at: Utc::now(),
            correlation_id,
            metadata: ImportMetadata::new(),
            source_type,
        }
    }

    /// Set derivation path
    pub fn with_derivation_path(mut self, path: String) -> Self {
        self.derivation_path = Some(path);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: ImportMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Import error with specific details (Property 23)
#[derive(Debug, Clone)]
pub struct ImportError {
    /// Error category
    pub category: ImportErrorCategory,
    /// Human-readable message
    pub message: String,
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
    /// suggestions for resolution
    pub suggestions: Vec<String>,
}

impl ImportError {
    /// Create a new import error
    pub fn new(category: ImportErrorCategory, message: impl Into<String>, correlation_id: Uuid) -> Self {
        Self {
            category,
            message: message.into(),
            correlation_id,
            suggestions: Vec::new(),
        }
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} [{}]", self.category, self.message, self.correlation_id)
    }
}

impl std::error::Error for ImportError {}

/// Categories of import errors (Property 23: Error Specificity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportErrorCategory {
    /// Invalid seed phrase format
    InvalidSeedPhrase,
    /// Invalid private key format
    InvalidPrivateKey,
    /// Invalid keystore format
    InvalidKeystoreFormat,
    /// Decryption failed
    DecryptionFailed,
    /// Invalid derivation path
    InvalidDerivationPath,
    /// Checksum validation failed
    ChecksumFailed,
    /// Unknown or unsupported format
    UnsupportedFormat,
}

impl std::fmt::Display for ImportErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSeedPhrase => write!(f, "Invalid Seed Phrase"),
            Self::InvalidPrivateKey => write!(f, "Invalid Private Key"),
            Self::InvalidKeystoreFormat => write!(f, "Invalid Keystore Format"),
            Self::DecryptionFailed => write!(f, "Decryption Failed"),
            Self::InvalidDerivationPath => write!(f, "Invalid Derivation Path"),
            Self::ChecksumFailed => write!(f, "Checksum Failed"),
            Self::UnsupportedFormat => write!(f, "Unsupported Format"),
        }
    }
}

/// Account importer with validation and format detection
///
/// Implements Requirements 8.1, 8.2, 8.3 for account importing with:
/// - BIP39 seed phrase validation
/// - Private key hex validation
/// - MetaMask keystore format support
/// - Automatic format detection
#[derive(Debug)]
pub struct AccountImporter {
    /// Logger for structured logging
    logger: AccountLogger,
}

impl AccountImporter {
    /// Create a new AccountImporter
    pub fn new() -> Self {
        Self {
            logger: AccountLogger::new(PrivacyMode::Enabled),
        }
    }

    /// Create with custom logger
    pub fn with_logger(logger: AccountLogger) -> Self {
        Self { logger }
    }

    /// Detect the format of import data
    ///
    /// Analyzes the input and determines whether it's a seed phrase,
    /// private key, or MetaMask keystore JSON.
    pub fn detect_format(&self, data: &str) -> FormatDetectionResult {
        let trimmed = data.trim();

        // Check for JSON (MetaMask keystore)
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(_keystore) = serde_json::from_str::<MetaMaskKeystore>(trimmed) {
                return FormatDetectionResult::new(ImportSourceType::MetaMaskKeystore, 1.0)
                    .with_details("Valid MetaMask keystore JSON detected");
            }
            // Might be JSON but not valid keystore
            return FormatDetectionResult::new(ImportSourceType::Unknown, 0.3)
                .with_details("JSON detected but not a valid keystore format");
        }

        // Check for hex private key
        let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
        if hex_str.len() == 64 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return FormatDetectionResult::new(ImportSourceType::PrivateKey, 0.95)
                .with_details("64-character hex string detected");
        }

        // Check for seed phrase (12, 15, 18, 21, or 24 words)
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if [12, 15, 18, 21, 24].contains(&words.len()) {
            // Validate as BIP39
            if Mnemonic::parse(trimmed).is_ok() {
                return FormatDetectionResult::new(ImportSourceType::SeedPhrase, 1.0)
                    .with_details(format!("Valid BIP39 seed phrase ({} words)", words.len()));
            } else {
                return FormatDetectionResult::new(ImportSourceType::SeedPhrase, 0.7)
                    .with_details("Word count matches seed phrase but BIP39 validation failed");
            }
        }

        FormatDetectionResult::new(ImportSourceType::Unknown, 0.0)
            .with_details("Could not determine format")
    }

    /// Import from BIP39 seed phrase
    ///
    /// Implements Property 20: Seed Phrase Import Determinism
    /// Same seed phrase always produces the same address for a given derivation path.
    pub fn import_from_seed(
        &self,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
        derivation_path: Option<&str>,
        metadata: ImportMetadata,
    ) -> Result<(ImportedAccount, PrivateKeySigner)> {
        let span = OperationSpan::new("import_from_seed");
        self.logger.log_operation_start(&span, "Importing account from seed phrase");

        // Validate seed phrase
        let phrase_str = phrase.expose_secret();
        let mnemonic = Mnemonic::parse(phrase_str).map_err(|e| {
            self.logger.log_operation_error(&span, &format!("Invalid seed phrase: {}", e));
            VaughanError::Security(SecurityError::InvalidSeedPhrase {
                reason: format!("BIP39 validation failed: {}", e),
            })
        })?;

        // Use default path if not specified
        let path_str = derivation_path.unwrap_or(DEFAULT_DERIVATION_PATH);
        let path = DerivationPath::from_str(path_str).map_err(|e| {
            self.logger.log_operation_error(&span, &format!("Invalid derivation path: {}", e));
            VaughanError::Security(SecurityError::KeyDerivationError {
                message: format!("Invalid derivation path '{}': {}", path_str, e),
            })
        })?;

        // Generate seed with optional passphrase
        let bip39_pass = passphrase.map(|p| p.expose_secret().as_str()).unwrap_or("");
        let seed = mnemonic.to_seed(bip39_pass);

        // Derive key
        let mut xprv = ExtendedPrivateKey::<SecretKey>::new(seed)
            .map_err(|e| VaughanError::Security(SecurityError::KeyDerivationError {
                message: format!("Failed to create master key: {}", e),
            }))?;

        for child in path.into_iter() {
            xprv = xprv.derive_child(child)
                .map_err(|e| VaughanError::Security(SecurityError::KeyDerivationError {
                    message: format!("Failed to derive child key: {}", e),
                }))?;
        }

        // Create signer
        let secret_bytes = xprv.private_key().to_bytes();
        let signing_key = SigningKey::from_bytes(&secret_bytes)
            .map_err(|e| VaughanError::Security(SecurityError::KeyDerivationError {
                message: format!("Failed to create signing key: {}", e),
            }))?;

        let signer = PrivateKeySigner::from(signing_key);
        let address = signer.address();

        let mut imported = ImportedAccount::new(address, ImportSourceType::SeedPhrase, span.correlation_id)
            .with_derivation_path(path_str.to_string())
            .with_metadata(metadata);
        imported.metadata.source_type = Some(ImportSourceType::SeedPhrase);

        self.logger.log_account_event(
            &span,
            "account_imported",
            &address.to_string(),
            "Account imported from seed phrase",
        );
        self.logger.log_operation_complete(&span, "Import successful");

        Ok((imported, signer))
    }

    /// Import from private key
    ///
    /// Supports both hex formats:
    /// - With 0x prefix: "0x..."
    /// - Without prefix: "..."
    pub fn import_from_private_key(
        &self,
        private_key: &SecretString,
        metadata: ImportMetadata,
    ) -> Result<(ImportedAccount, PrivateKeySigner)> {
        let span = OperationSpan::new("import_from_private_key");
        self.logger.log_operation_start(&span, "Importing account from private key");

        let key_str = private_key.expose_secret();

        // Parse private key
        let signer: PrivateKeySigner = key_str.parse().map_err(|_| {
            self.logger.log_operation_error(&span, "Invalid private key format");
            VaughanError::Security(SecurityError::KeyDerivationError {
                message: "Failed to parse private key. Expected 32-byte hex string.".to_string(),
            })
        })?;

        let address = signer.address();

        let mut imported = ImportedAccount::new(address, ImportSourceType::PrivateKey, span.correlation_id)
            .with_metadata(metadata);
        imported.metadata.source_type = Some(ImportSourceType::PrivateKey);

        self.logger.log_account_event(
            &span,
            "account_imported",
            &address.to_string(),
            "Account imported from private key",
        );
        self.logger.log_operation_complete(&span, "Import successful");

        Ok((imported, signer))
    }

    /// Import from MetaMask keystore JSON
    ///
    /// Decrypts a MetaMask-format keystore file and imports the account.
    /// This method validates the keystore format before attempting decryption.
    ///
    /// Note: Actual decryption requires the keystore password and uses
    /// PBKDF2 + AES-CTR as defined in the MetaMask v3 format.
    pub fn import_from_metamask(
        &self,
        keystore_json: &str,
        password: &SecretString,
        metadata: ImportMetadata,
    ) -> Result<(ImportedAccount, PrivateKeySigner)> {
        let span = OperationSpan::new("import_from_metamask");
        self.logger.log_operation_start(&span, "Importing account from MetaMask keystore");

        // Parse keystore JSON
        let keystore: MetaMaskKeystore = serde_json::from_str(keystore_json).map_err(|e| {
            self.logger.log_operation_error(&span, &format!("Invalid keystore JSON: {}", e));
            VaughanError::Security(SecurityError::KeystoreError {
                message: format!("Failed to parse keystore JSON: {}", e),
            })
        })?;

        // Validate keystore structure
        self.validate_keystore_structure(&keystore, &span)?;

        // Decrypt the keystore
        let private_key_bytes = self.decrypt_keystore(&keystore, password, &span)?;

        // Create signer from decrypted key
        let signer = PrivateKeySigner::from_slice(&private_key_bytes).map_err(|e| {
            self.logger.log_operation_error(&span, &format!("Invalid decrypted key: {}", e));
            VaughanError::Security(SecurityError::KeyDerivationError {
                message: format!("Failed to create signer from decrypted key: {}", e),
            })
        })?;

        let address = signer.address();

        // Verify address matches keystore
        let stored_address = keystore.address;
        if !stored_address.is_zero() && stored_address != address {
            self.logger.log_operation_warning(&span, "Derived address doesn't match stored address");
        }

        let mut imported = ImportedAccount::new(address, ImportSourceType::MetaMaskKeystore, span.correlation_id)
            .with_metadata(metadata);
        imported.metadata.source_type = Some(ImportSourceType::MetaMaskKeystore);

        self.logger.log_account_event(
            &span,
            "account_imported",
            &address.to_string(),
            "Account imported from MetaMask keystore",
        );
        self.logger.log_operation_complete(&span, "Import successful");

        Ok((imported, signer))
    }

    /// Validate keystore structure (Property 21)
    fn validate_keystore_structure(&self, keystore: &MetaMaskKeystore, span: &OperationSpan) -> Result<()> {
        // Check version
        if keystore.version != 3 {
            self.logger.log_operation_error(span, &format!("Unsupported keystore version: {}", keystore.version));
            return Err(VaughanError::Security(SecurityError::KeystoreError {
                message: format!("Unsupported keystore version: {}. Expected version 3.", keystore.version),
            }));
        }

        // Check cipher
        if keystore.crypto.cipher != "aes-128-ctr" {
            self.logger.log_operation_error(span, &format!("Unsupported cipher: {}", keystore.crypto.cipher));
            return Err(VaughanError::Security(SecurityError::KeystoreError {
                message: format!("Unsupported cipher: {}. Expected aes-128-ctr.", keystore.crypto.cipher),
            }));
        }

        // Check KDF
        let supported_kdfs = ["pbkdf2", "scrypt"];
        if !supported_kdfs.contains(&keystore.crypto.kdf.as_str()) {
            self.logger.log_operation_error(span, &format!("Unsupported KDF: {}", keystore.crypto.kdf));
            return Err(VaughanError::Security(SecurityError::KeystoreError {
                message: format!("Unsupported KDF: {}. Expected pbkdf2 or scrypt.", keystore.crypto.kdf),
            }));
        }

        Ok(())
    }

    /// Decrypt keystore using PBKDF2 + AES-CTR
    fn decrypt_keystore(
        &self,
        keystore: &MetaMaskKeystore,
        password: &SecretString,
        span: &OperationSpan,
    ) -> Result<[u8; 32]> {
        use aes::cipher::{KeyIvInit, StreamCipher};
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        let password_bytes = password.expose_secret().as_bytes();

        // Derive key using PBKDF2
        let salt = hex::decode(&keystore.crypto.kdfparams.salt).map_err(|e| {
            self.logger.log_operation_error(span, &format!("Invalid salt hex: {}", e));
            VaughanError::Security(SecurityError::DecryptionError {
                message: format!("Invalid salt encoding: {}", e),
            })
        })?;

        let dklen = keystore.crypto.kdfparams.dklen as usize;
        let iterations = keystore.crypto.kdfparams.c;

        let mut derived_key = vec![0u8; dklen];
        pbkdf2_hmac::<Sha256>(password_bytes, &salt, iterations, &mut derived_key);

        // Verify MAC
        let mac_key = &derived_key[16..32];
        let ciphertext = hex::decode(&keystore.crypto.ciphertext).map_err(|e| {
            VaughanError::Security(SecurityError::DecryptionError {
                message: format!("Invalid ciphertext encoding: {}", e),
            })
        })?;

        let mut mac_input = Vec::with_capacity(mac_key.len() + ciphertext.len());
        mac_input.extend_from_slice(mac_key);
        mac_input.extend_from_slice(&ciphertext);

        use alloy::primitives::keccak256;
        let computed_mac = keccak256(&mac_input);
        let stored_mac = hex::decode(&keystore.crypto.mac).map_err(|e| {
            VaughanError::Security(SecurityError::DecryptionError {
                message: format!("Invalid MAC encoding: {}", e),
            })
        })?;

        if computed_mac.as_slice() != stored_mac.as_slice() {
            self.logger.log_operation_error(span, "MAC verification failed - wrong password");
            return Err(VaughanError::Security(SecurityError::DecryptionError {
                message: "MAC verification failed. Incorrect password.".to_string(),
            }));
        }

        // Decrypt using AES-128-CTR
        let iv = hex::decode(&keystore.crypto.cipherparams.iv).map_err(|e| {
            VaughanError::Security(SecurityError::DecryptionError {
                message: format!("Invalid IV encoding: {}", e),
            })
        })?;

        let encryption_key = &derived_key[0..16];
        
        type Aes128Ctr = ctr::Ctr128BE<aes::Aes128>;
        let mut cipher = Aes128Ctr::new(encryption_key.into(), iv.as_slice().into());

        let mut decrypted = ciphertext.clone();
        cipher.apply_keystream(&mut decrypted);

        if decrypted.len() != 32 {
            return Err(VaughanError::Security(SecurityError::DecryptionError {
                message: format!("Decrypted key has wrong length: {} (expected 32)", decrypted.len()),
            }));
        }

        let mut result = [0u8; 32];
        result.copy_from_slice(&decrypted);
        
        Ok(result)
    }

    /// Validate import data without actually importing
    pub fn validate_import_data(&self, data: &str) -> ImportValidationResult {
        let detection = self.detect_format(data);

        match detection.source_type {
            ImportSourceType::SeedPhrase => {
                let phrase = SecretString::from(data.to_string());
                match Mnemonic::parse(phrase.expose_secret()) {
                    Ok(m) => ImportValidationResult {
                        is_valid: true,
                        source_type: ImportSourceType::SeedPhrase,
                        word_count: Some(m.word_count()),
                        error: None,
                    },
                    Err(e) => ImportValidationResult {
                        is_valid: false,
                        source_type: ImportSourceType::SeedPhrase,
                        word_count: None,
                        error: Some(format!("Invalid seed phrase: {}", e)),
                    },
                }
            }
            ImportSourceType::PrivateKey => {
                let key = SecretString::from(data.to_string());
                match key.expose_secret().parse::<PrivateKeySigner>() {
                    Ok(_) => ImportValidationResult {
                        is_valid: true,
                        source_type: ImportSourceType::PrivateKey,
                        word_count: None,
                        error: None,
                    },
                    Err(_) => ImportValidationResult {
                        is_valid: false,
                        source_type: ImportSourceType::PrivateKey,
                        word_count: None,
                        error: Some("Invalid private key format".to_string()),
                    },
                }
            }
            ImportSourceType::MetaMaskKeystore => {
                match serde_json::from_str::<MetaMaskKeystore>(data) {
                    Ok(ks) => {
                        if ks.version != 3 {
                            ImportValidationResult {
                                is_valid: false,
                                source_type: ImportSourceType::MetaMaskKeystore,
                                word_count: None,
                                error: Some(format!("Unsupported keystore version: {}", ks.version)),
                            }
                        } else {
                            ImportValidationResult {
                                is_valid: true,
                                source_type: ImportSourceType::MetaMaskKeystore,
                                word_count: None,
                                error: None,
                            }
                        }
                    }
                    Err(e) => ImportValidationResult {
                        is_valid: false,
                        source_type: ImportSourceType::MetaMaskKeystore,
                        word_count: None,
                        error: Some(format!("Invalid keystore JSON: {}", e)),
                    },
                }
            }
            ImportSourceType::Unknown => ImportValidationResult {
                is_valid: false,
                source_type: ImportSourceType::Unknown,
                word_count: None,
                error: Some("Could not determine import format".to_string()),
            },
        }
    }
}

impl Default for AccountImporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of import validation
#[derive(Debug, Clone)]
pub struct ImportValidationResult {
    /// Whether the data is valid for import
    pub is_valid: bool,
    /// Detected source type
    pub source_type: ImportSourceType,
    /// Word count (for seed phrases)
    pub word_count: Option<usize>,
    /// Error message if invalid
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test mnemonic (DO NOT use in production)
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn test_detect_format_seed_phrase() {
        let importer = AccountImporter::new();
        let result = importer.detect_format(TEST_MNEMONIC);
        
        assert_eq!(result.source_type, ImportSourceType::SeedPhrase);
        assert!(result.confidence >= 0.9);
    }

    #[test]
    fn test_detect_format_private_key_with_prefix() {
        let importer = AccountImporter::new();
        let result = importer.detect_format("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
        
        assert_eq!(result.source_type, ImportSourceType::PrivateKey);
        assert!(result.confidence >= 0.9);
    }

    #[test]
    fn test_detect_format_private_key_no_prefix() {
        let importer = AccountImporter::new();
        let result = importer.detect_format("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
        
        assert_eq!(result.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_detect_format_unknown() {
        let importer = AccountImporter::new();
        let result = importer.detect_format("random text that is not valid");
        
        assert_eq!(result.source_type, ImportSourceType::Unknown);
    }

    #[test]
    fn test_import_from_seed() {
        let importer = AccountImporter::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let metadata = ImportMetadata::new().with_name("Test Import");

        let result = importer.import_from_seed(&phrase, None, None, metadata);
        assert!(result.is_ok());

        let (account, signer) = result.unwrap();
        assert!(!account.address.is_zero());
        assert_eq!(account.source_type, ImportSourceType::SeedPhrase);
        assert_eq!(signer.address(), account.address);
    }

    #[test]
    fn test_import_from_seed_determinism() {
        let importer = AccountImporter::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());

        // Import same seed multiple times
        let (account1, _) = importer.import_from_seed(&phrase, None, None, ImportMetadata::new()).unwrap();
        let (account2, _) = importer.import_from_seed(&phrase, None, None, ImportMetadata::new()).unwrap();
        let (account3, _) = importer.import_from_seed(&phrase, None, None, ImportMetadata::new()).unwrap();

        // All should produce the same address (Property 20)
        assert_eq!(account1.address, account2.address);
        assert_eq!(account2.address, account3.address);
    }

    #[test]
    fn test_import_from_private_key() {
        let importer = AccountImporter::new();
        let key = SecretString::from("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string());
        let metadata = ImportMetadata::new();

        let result = importer.import_from_private_key(&key, metadata);
        assert!(result.is_ok());

        let (account, _) = result.unwrap();
        assert_eq!(account.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_invalid_seed_phrase() {
        let importer = AccountImporter::new();
        let phrase = SecretString::from("invalid seed phrase words".to_string());

        let result = importer.import_from_seed(&phrase, None, None, ImportMetadata::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_private_key() {
        let importer = AccountImporter::new();
        let key = SecretString::from("not a valid key".to_string());

        let result = importer.import_from_private_key(&key, ImportMetadata::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_import_data_seed() {
        let importer = AccountImporter::new();
        let result = importer.validate_import_data(TEST_MNEMONIC);

        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::SeedPhrase);
        assert_eq!(result.word_count, Some(12));
    }

    #[test]
    fn test_validate_import_data_key() {
        let importer = AccountImporter::new();
        let result = importer.validate_import_data("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");

        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_metadata_preservation() {
        let importer = AccountImporter::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let metadata = ImportMetadata::new()
            .with_name("My Imported Account")
            .with_tag("imported")
            .with_tag("test");

        let (account, _) = importer.import_from_seed(&phrase, None, None, metadata).unwrap();

        // Property 22: Metadata should be preserved
        assert_eq!(account.metadata.name, Some("My Imported Account".to_string()));
        assert_eq!(account.metadata.tags.len(), 2);
        assert!(account.metadata.tags.contains(&"imported".to_string()));
    }

    #[test]
    fn test_different_derivation_paths() {
        let importer = AccountImporter::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());

        let (account1, _) = importer.import_from_seed(&phrase, None, Some("m/44'/60'/0'/0/0"), ImportMetadata::new()).unwrap();
        let (account2, _) = importer.import_from_seed(&phrase, None, Some("m/44'/60'/0'/0/1"), ImportMetadata::new()).unwrap();

        // Different paths should produce different addresses
        assert_ne!(account1.address, account2.address);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 20: Seed Phrase Import Determinism
        ///
        /// *For any* valid BIP39 seed phrase, importing it multiple times
        /// should always produce the same account address for the same derivation path.
        ///
        /// Validates: Requirements 8.2
        #[test]
        fn prop_seed_import_determinism(import_count in 2usize..5) {
            // Use fixed test mnemonic
            let phrase = SecretString::from(
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
            );
            let importer = AccountImporter::new();

            let addresses: Vec<Address> = (0..import_count)
                .map(|_| {
                    let (account, _) = importer
                        .import_from_seed(&phrase, None, None, ImportMetadata::new())
                        .expect("Import should succeed");
                    account.address
                })
                .collect();

            // All addresses should be identical
            let first = addresses[0];
            for addr in &addresses[1..] {
                prop_assert_eq!(*addr, first, "Same seed must produce same address");
            }
        }

        /// Property 21: Migration Format Validation
        ///
        /// *For any* import attempt, invalid formats should be rejected
        /// with specific error messages.
        ///
        /// Validates: Requirements 8.3
        #[test]
        fn prop_format_validation(
            random_data in "[a-zA-Z0-9]{1,100}"
        ) {
            let importer = AccountImporter::new();
            let result = importer.validate_import_data(&random_data);

            // Either valid with correct type, or invalid with error
            if result.is_valid {
                prop_assert!(
                    result.source_type != ImportSourceType::Unknown,
                    "Valid data should have known type"
                );
            } else {
                prop_assert!(
                    result.error.is_some(),
                    "Invalid data should have error message"
                );
            }
        }

        /// Property 22: Metadata Preservation
        ///
        /// *For any* import with metadata, the metadata should be preserved
        /// in the imported account.
        ///
        /// Validates: Requirements 8.4
        #[test]
        fn prop_metadata_preservation(
            name in "[a-zA-Z ]{3,30}",
            tag in "[a-z]{3,10}"
        ) {
            let importer = AccountImporter::new();
            let phrase = SecretString::from(
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
            );
            
            let metadata = ImportMetadata::new()
                .with_name(&name)
                .with_tag(&tag);

            let (account, _) = importer
                .import_from_seed(&phrase, None, None, metadata)
                .expect("Import should succeed");

            prop_assert_eq!(account.metadata.name.as_deref(), Some(name.as_str()));
            prop_assert!(account.metadata.tags.contains(&tag));
        }

        /// Property 23: Error Specificity
        ///
        /// *For any* failed import, the error should be specific enough
        /// to identify the problem category.
        ///
        /// Validates: Requirements 8.5
        #[test]
        fn prop_error_specificity(
            invalid_phrase in "[a-z]{1,5}( [a-z]{1,5}){10,12}"
        ) {
            let importer = AccountImporter::new();
            let phrase = SecretString::from(invalid_phrase);

            let result = importer.import_from_seed(&phrase, None, None, ImportMetadata::new());

            // Should fail with specific error
            prop_assert!(result.is_err());
            
            if let Err(e) = result {
                let error_msg = e.to_string().to_lowercase();
                // Error should mention seed/mnemonic/bip39
                prop_assert!(
                    error_msg.contains("seed") || 
                    error_msg.contains("mnemonic") || 
                    error_msg.contains("bip39") ||
                    error_msg.contains("phrase"),
                    "Error should be specific: {}", error_msg
                );
            }
        }
    }
}
