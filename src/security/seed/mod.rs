//! Secure seed phrase (mnemonic) management
//!
//! This module handles BIP39 seed phrase generation, validation, import/export,
//! and secure storage with proper zeroization of sensitive data.
//!
//! ## Module Structure
//! - `types` - Core data structures (SecureSeed, SeedStrength, etc.)
//! - `encryption` - AES-256-GCM encryption with Argon2/PBKDF2 key derivation
//! - `derivation` - BIP32/BIP39 HD wallet derivation
//! - `validation` - Seed phrase validation and word suggestions
//! - `zeroization` - Secure memory handling utilities
//! - `utils` - BIP39 wordlist utilities

// Submodules
pub mod derivation;
pub mod encryption;
pub mod types;
pub mod utils;
pub mod validation;
pub mod zeroization;

// Re-exports from types module
pub use types::{
    DerivationPathConfig, DerivedAccount, ExportFormat, ExportMetadata, ExportOptions, ExportResult,
    MultiAccountDerivation, SecureSeed, SecurityUseCase, SeedAnalysis, SeedBackup, SeedImportConfig,
    SeedImportValidation, SeedStrength, WordSuggestion,
};

// Re-exports from encryption module
pub use encryption::{EncryptedSeedData, EncryptedSeedDataV2, EncryptionAlgorithm, KeyDerivationAlgorithm};

// Core imports
use super::{KeyReference, KeychainInterface, SecureAccount};
use crate::error::{Result, SecurityError};
use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use bip32::{secp256k1::SecretKey, ExtendedPrivateKey};
use bip39::Mnemonic;
use secrecy::{ExposeSecret, SecretString};

// ============================================================================
// SecureSeedStorage - Secure storage with keychain integration
// ============================================================================

/// Secure seed phrase storage with AES-256-GCM encryption
pub struct SecureSeedStorage {
    keychain: Box<dyn KeychainInterface>,
    service_name: String,
}

impl SecureSeedStorage {
    /// Create a new secure seed storage instance
    pub fn new(keychain: Box<dyn KeychainInterface>) -> Self {
        Self {
            keychain,
            service_name: "vaughan-wallet-encrypted-seeds".to_string(),
        }
    }

    /// Store encrypted seed phrase in keychain
    pub async fn store_encrypted_seed_phrase(
        &self,
        wallet_id: &str,
        seed_phrase: &SecretString,
        master_password: &SecretString,
    ) -> Result<KeyReference> {
        // Validate seed phrase before storing
        let _mnemonic = Mnemonic::parse(seed_phrase.expose_secret()).map_err(|e| SecurityError::InvalidSeedPhrase {
            reason: format!("Invalid BIP39 mnemonic: {e}"),
        })?;

        // Encrypt the seed phrase with enhanced V2 encryption using Argon2 by default
        let encrypted_data = encryption::encrypt_seed_phrase_v2(
            seed_phrase,
            master_password,
            Some(KeyDerivationAlgorithm::Argon2id {
                memory: 65536,  // 64 MiB
                iterations: 3,  // 3 iterations
                parallelism: 4, // 4 threads
            }),
            None, // Use default AES-256-GCM
        )?;

        // Serialize encrypted data
        let serialized_data =
            serde_json::to_string(&encrypted_data).map_err(|e| SecurityError::SerializationError {
                message: format!("Failed to serialize encrypted data: {e}"),
            })?;

        // Create key reference
        let key_ref = KeyReference {
            id: uuid::Uuid::new_v4().to_string(),
            service: self.service_name.clone(),
            account: format!("encrypted-seed-v2-{wallet_id}"),
        };

        // Store in keychain
        self.keychain
            .store(&key_ref, SecretString::new(serialized_data))
            .map_err(|e| SecurityError::KeychainError {
                message: format!("Failed to store encrypted seed phrase: {e}"),
            })?;

        tracing::info!("Encrypted seed phrase stored securely for wallet: {}", wallet_id);

        Ok(key_ref)
    }

    /// Retrieve and decrypt seed phrase from keychain
    pub async fn retrieve_encrypted_seed_phrase(
        &self,
        key_ref: &KeyReference,
        master_password: &SecretString,
    ) -> Result<SecretString> {
        // Retrieve encrypted data from keychain
        let serialized_data = self
            .keychain
            .retrieve(key_ref)
            .map_err(|e| SecurityError::KeychainError {
                message: format!("Failed to retrieve encrypted seed phrase: {e}"),
            })?;

        // Try to deserialize as V2 format first
        let seed_phrase =
            match serde_json::from_str::<EncryptedSeedDataV2>(serialized_data.expose_secret()) {
                Ok(encrypted_data_v2) => {
                    // Decrypt using V2 method
                    encryption::decrypt_seed_phrase_v2(&encrypted_data_v2, master_password)?
                }
                Err(_) => {
                    // Try V1 format
                    let encrypted_data: EncryptedSeedData = serde_json::from_str(serialized_data.expose_secret())
                        .map_err(|e| SecurityError::DeserializationError {
                            message: format!("Failed to deserialize encrypted data: {e}"),
                        })?;

                    // Decrypt using V1 method
                    encryption::decrypt_seed_phrase(&encrypted_data, master_password)?
                }
            };

        // Validate decrypted seed phrase
        let _mnemonic = Mnemonic::parse(seed_phrase.expose_secret()).map_err(|e| SecurityError::InvalidSeedPhrase {
            reason: format!("Decrypted seed phrase is invalid: {e}"),
        })?;

        tracing::info!("Encrypted seed phrase retrieved and decrypted successfully");

        Ok(seed_phrase)
    }

    /// Securely delete encrypted seed phrase from keychain
    pub async fn delete_encrypted_seed_phrase(&self, key_ref: &KeyReference) -> Result<()> {
        self.keychain
            .delete(key_ref)
            .map_err(|e| SecurityError::KeychainError {
                message: format!("Failed to delete encrypted seed phrase: {e}"),
            })?;

        tracing::info!("Encrypted seed phrase deleted from keychain");

        Ok(())
    }

    /// Create backup of encrypted seed phrase
    pub async fn create_backup(&self, key_ref: &KeyReference, backup_password: &SecretString) -> Result<SeedBackup> {
        // Retrieve the encrypted data
        let serialized_data = self
            .keychain
            .retrieve(key_ref)
            .map_err(|e| SecurityError::KeychainError {
                message: format!("Failed to retrieve seed phrase for backup: {e}"),
            })?;

        // Generate backup encryption parameters
        let backup_salt = encryption::generate_salt()?;
        let backup_nonce = encryption::generate_nonce()?;

        // Use Argon2 for backup encryption
        let backup_key_bytes = encryption::derive_key_enhanced(
            backup_password,
            &backup_salt,
            &KeyDerivationAlgorithm::Argon2id {
                memory: 65536,  // 64 MiB
                iterations: 3,  // 3 iterations
                parallelism: 4, // 4 threads
            },
        )?;

        let backup_key = Key::<Aes256Gcm>::from_slice(&backup_key_bytes);
        let cipher = Aes256Gcm::new(backup_key);
        let nonce = Nonce::from_slice(&backup_nonce);

        let backup_ciphertext = cipher
            .encrypt(nonce, serialized_data.expose_secret().as_bytes())
            .map_err(|e| SecurityError::EncryptionError {
                message: format!("Failed to encrypt backup: {e}"),
            })?;

        Ok(SeedBackup {
            encrypted_data: backup_ciphertext,
            salt: backup_salt,
            nonce: backup_nonce,
            created_at: chrono::Utc::now(),
            version: 2,
        })
    }

    /// Export seed phrase with security verification
    pub async fn export_seed_phrase(
        &self,
        key_ref: &KeyReference,
        master_password: &SecretString,
        options: ExportOptions,
        export_password: Option<&SecretString>,
    ) -> Result<ExportResult> {
        // Security logging - note export attempt
        tracing::warn!("🚨 SEED PHRASE EXPORT ATTEMPT - Key ID: {}", key_ref.id);

        // Retrieve and decrypt the seed phrase
        let seed_phrase = self.retrieve_encrypted_seed_phrase(key_ref, master_password).await?;

        // Standard security warnings
        let security_warnings = vec![
            "⚠️  CRITICAL: Your seed phrase grants full access to your wallet.".to_string(),
            "⚠️  Never share your seed phrase with anyone.".to_string(),
            "⚠️  Store it securely offline in multiple safe locations.".to_string(),
            "⚠️  Anyone with this seed phrase can steal all your funds.".to_string(),
        ];

        // Create metadata if requested
        let metadata = if options.include_metadata {
            Some(ExportMetadata {
                wallet_name: options
                    .custom_export_name
                    .clone()
                    .unwrap_or_else(|| "Vaughan Wallet".to_string()),
                seed_strength: SeedStrength::from_word_count(seed_phrase.expose_secret().split_whitespace().count())?,
                creation_timestamp: Some(chrono::Utc::now()),
                derivation_paths: if options.include_derivation_paths {
                    derivation::get_standard_derivation_paths()
                } else {
                    vec![]
                },
                export_version: "2.0".to_string(),
            })
        } else {
            None
        };

        // Format the export data
        let formatted_data = match options.format {
            ExportFormat::PlainText => SecretString::new(seed_phrase.expose_secret().to_string()),
            ExportFormat::QrCode => {
                let qr_data = format!("VAUGHAN_SEED:{}", seed_phrase.expose_secret());
                SecretString::new(qr_data)
            }
            ExportFormat::Json => {
                let export_json = serde_json::json!({
                    "seed_phrase": seed_phrase.expose_secret(),
                    "metadata": metadata,
                    "export_timestamp": chrono::Utc::now(),
                    "version": "2.0"
                });
                SecretString::new(export_json.to_string())
            }
            ExportFormat::EncryptedJson => {
                let export_password = export_password.ok_or_else(|| SecurityError::KeystoreError {
                    message: "Export password required for encrypted format".to_string(),
                })?;

                let export_json = serde_json::json!({
                    "seed_phrase": seed_phrase.expose_secret(),
                    "metadata": metadata,
                    "export_timestamp": chrono::Utc::now(),
                    "version": "2.0"
                });

                // Encrypt with export password
                let salt = encryption::generate_salt()?;
                let nonce = encryption::generate_nonce()?;
                let key_bytes = encryption::derive_key_enhanced(
                    export_password,
                    &salt,
                    &KeyDerivationAlgorithm::Argon2id {
                        memory: 65536,
                        iterations: 3,
                        parallelism: 4,
                    },
                )?;

                let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
                let cipher = Aes256Gcm::new(key);
                let nonce_slice = Nonce::from_slice(&nonce);

                let ciphertext = cipher
                    .encrypt(nonce_slice, export_json.to_string().as_bytes())
                    .map_err(|e| SecurityError::EncryptionError {
                        message: format!("Failed to encrypt export data: {e}"),
                    })?;

                use base64::{engine::general_purpose, Engine as _};

                let encrypted_export = serde_json::json!({
                    "encrypted_data": general_purpose::STANDARD.encode(&ciphertext),
                    "salt": general_purpose::STANDARD.encode(salt),
                    "nonce": general_purpose::STANDARD.encode(nonce),
                    "encryption_method": "AES-256-GCM",
                    "kdf": "Argon2id",
                    "version": "2.0"
                });

                SecretString::new(encrypted_export.to_string())
            }
        };

        tracing::warn!(
            "🚨 SEED PHRASE EXPORTED - Format: {:?}, Key ID: {}",
            options.format,
            key_ref.id
        );

        Ok(ExportResult {
            format: options.format,
            data: formatted_data,
            metadata,
            export_timestamp: chrono::Utc::now(),
            security_warnings,
        })
    }
}

// ============================================================================
// SeedManager - Main seed management interface
// ============================================================================

/// Seed phrase manager with secure storage
pub struct SeedManager {
    keychain: Box<dyn KeychainInterface>,
    #[allow(dead_code)] // Reserved for future keychain operations service identifier
    service_name: String,
}

impl SeedManager {
    /// Create a new seed manager
    pub fn new(keychain: Box<dyn KeychainInterface>) -> Self {
        Self {
            keychain,
            service_name: "vaughan-wallet-seeds".to_string(),
        }
    }

    /// Get recommended seed strengths for a use case
    pub fn get_recommended_strengths(use_case: SecurityUseCase) -> Vec<SeedStrength> {
        match use_case {
            SecurityUseCase::Personal => vec![SeedStrength::Words12, SeedStrength::Words24],
            SecurityUseCase::Business => vec![SeedStrength::Words18, SeedStrength::Words21, SeedStrength::Words24],
            SecurityUseCase::Institutional => vec![SeedStrength::Words21, SeedStrength::Words24],
            SecurityUseCase::LongTermStorage => vec![SeedStrength::Words24],
        }
    }

    /// Get default recommended strength for a use case
    pub fn get_default_strength(use_case: SecurityUseCase) -> SeedStrength {
        match use_case {
            SecurityUseCase::Personal => SeedStrength::Words12,
            SecurityUseCase::Business => SeedStrength::Words18,
            SecurityUseCase::Institutional => SeedStrength::Words24,
            SecurityUseCase::LongTermStorage => SeedStrength::Words24,
        }
    }

    /// Generate a new cryptographically secure seed phrase
    pub fn generate_seed_phrase(&self, strength: SeedStrength) -> Result<SecretString> {
        let entropy_bits = strength.entropy_bits();
        let entropy_bytes = entropy_bits / 8;

        let mut entropy = vec![0u8; entropy_bytes];

        // Use getrandom for cryptographically secure entropy
        getrandom::getrandom(&mut entropy).map_err(|e| SecurityError::KeyDerivationError {
            message: format!("Failed to generate secure entropy: {e}"),
        })?;

        // Generate mnemonic from entropy
        let mnemonic = Mnemonic::from_entropy(&entropy).map_err(|e| SecurityError::KeyDerivationError {
            message: format!("Failed to generate mnemonic: {e}"),
        })?;

        let phrase = SecretString::new(mnemonic.to_string());

        tracing::info!(
            "Generated new {}-word seed phrase with {}-bit entropy",
            strength.word_count(),
            entropy_bits
        );

        Ok(phrase)
    }

    /// Validate a seed phrase
    pub fn validate_seed_phrase(&self, phrase: &SecretString) -> Result<()> {
        validation::validate_seed_phrase(phrase)
    }

    /// Comprehensive seed phrase validation with detailed error reporting
    /// This is a wrapper method for backward compatibility
    pub fn validate_seed_phrase_comprehensive(
        &self,
        phrase: &str,
        config: &SeedImportConfig,
    ) -> Result<SeedImportValidation> {
        validation::validate_seed_phrase_comprehensive(phrase, config)
    }

    /// Import seed phrase and validate it (legacy method)
    pub fn import_seed_phrase(&self, phrase: SecretString) -> Result<SecretString> {
        self.validate_seed_phrase(&phrase)?;
        Ok(phrase)
    }

    /// Comprehensive seed phrase import with advanced validation and correction
    pub fn import_seed_phrase_comprehensive(
        &self,
        phrase: &str,
        config: &SeedImportConfig,
    ) -> Result<(SecretString, SeedImportValidation)> {
        // Preprocess the phrase
        let cleaned_phrase = validation::preprocess_seed_phrase(phrase);

        // Validate comprehensively
        let validation_result = validation::validate_seed_phrase_comprehensive(&cleaned_phrase, config)?;

        if !validation_result.is_valid {
            // Try to apply corrections if enabled
            if config.enable_fuzzy_matching && !validation_result.suggestions.is_empty() {
                let corrected = validation::apply_corrections(&cleaned_phrase, &validation_result, config)?;
                let revalidation = validation::validate_seed_phrase_comprehensive(&corrected, config)?;

                if revalidation.is_valid {
                    return Ok((SecretString::new(corrected), revalidation));
                }
            }

            return Err(SecurityError::InvalidSeedPhrase {
                reason: validation_result.errors.join("; "),
            }
            .into());
        }

        Ok((SecretString::new(cleaned_phrase), validation_result))
    }

    /// Generate seed from mnemonic phrase
    pub fn phrase_to_seed(&self, phrase: &SecretString, passphrase: Option<&SecretString>) -> Result<SecureSeed> {
        derivation::phrase_to_seed(phrase, passphrase)
    }

    /// Derive Ethereum wallet from seed phrase
    pub fn derive_wallet_from_seed(
        &self,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
        derivation_path: Option<&str>,
    ) -> Result<alloy::signers::local::PrivateKeySigner> {
        derivation::derive_wallet_from_seed(phrase, passphrase, derivation_path)
    }

    /// Enhanced BIP-32 compliant HD wallet derivation
    pub fn derive_hd_wallet_from_seed(
        &self,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
        derivation_path: &str,
    ) -> Result<ExtendedPrivateKey<SecretKey>> {
        derivation::derive_hd_wallet_from_seed(phrase, passphrase, derivation_path)
    }

    /// Derive multiple HD wallet accounts from seed phrase
    pub fn derive_multiple_hd_accounts(
        &self,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
        base_path: &str,
        account_count: u32,
    ) -> Result<Vec<(u32, ExtendedPrivateKey<SecretKey>, alloy::primitives::Address)>> {
        derivation::derive_multiple_hd_accounts(phrase, passphrase, base_path, account_count)
    }

    /// Get standard Ethereum derivation paths
    pub fn get_standard_derivation_paths() -> Vec<DerivationPathConfig> {
        derivation::get_standard_derivation_paths()
    }

    /// Analyze seed phrase and provide comprehensive security information
    pub fn analyze_seed_phrase(&self, phrase: &SecretString) -> Result<SeedAnalysis> {
        derivation::analyze_imported_seed_phrase(phrase)
    }

    /// Get seed phrase strength from word count
    pub fn detect_seed_strength(phrase: &SecretString) -> Option<SeedStrength> {
        let word_count = phrase.expose_secret().split_whitespace().count();
        SeedStrength::from_word_count(word_count).ok()
    }

    /// Securely store seed phrase with encryption using master password
    pub async fn store_seed_phrase_encrypted(
        &self,
        wallet_id: &str,
        phrase: &SecretString,
        master_password: &SecretString,
    ) -> Result<KeyReference> {
        self.validate_seed_phrase(phrase)?;
        let secure_storage = SecureSeedStorage::new(self.keychain.clone_box());
        secure_storage
            .store_encrypted_seed_phrase(wallet_id, phrase, master_password)
            .await
    }

    /// Retrieve seed phrase from OS keychain  
    pub fn retrieve_seed_phrase(&self, key_ref: &KeyReference) -> Result<SecretString> {
        self.keychain.retrieve(key_ref).map_err(|e| {
            SecurityError::KeychainError {
                message: format!("Failed to retrieve seed phrase: {e}"),
            }
            .into()
        })
    }

    /// Delete seed phrase from keychain
    pub fn delete_seed_phrase(&self, key_ref: &KeyReference) -> Result<()> {
        self.keychain.delete(key_ref).map_err(|e| {
            SecurityError::KeychainError {
                message: format!("Failed to delete seed phrase: {e}"),
            }
            .into()
        })
    }

    /// Create a complete wallet from seed phrase
    pub fn create_wallet_from_seed(
        &self,
        wallet_name: String,
        phrase: &SecretString,
        passphrase: Option<&SecretString>,
    ) -> Result<SecureAccount> {
        self.validate_seed_phrase(phrase)?;

        let wallet = self.derive_wallet_from_seed(phrase, passphrase, None)?;
        let address = wallet.address();

        let key_ref = KeyReference {
            id: uuid::Uuid::new_v4().to_string(),
            service: self.service_name.clone(),
            account: format!("seed-{}", wallet_name),
        };

        Ok(SecureAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: wallet_name,
            address,
            key_reference: key_ref,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        })
    }

    /// Create wallet from seed phrase with encrypted storage
    /// This is the primary method for creating seed-based wallets
    pub async fn create_wallet_from_seed_encrypted(
        &self,
        wallet_name: String,
        phrase: &SecretString,
        master_password: &SecretString,
        passphrase: Option<&SecretString>,
    ) -> Result<SecureAccount> {
        self.validate_seed_phrase(phrase)?;

        // Derive wallet from seed phrase
        let wallet = self.derive_wallet_from_seed(phrase, passphrase, None)?;
        let address = wallet.address();

        // Store the seed phrase with encryption
        let secure_storage = SecureSeedStorage::new(self.keychain.clone_box());
        let key_ref = secure_storage
            .store_encrypted_seed_phrase(&wallet_name, phrase, master_password)
            .await?;

        let account = SecureAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: wallet_name.clone(),
            address,
            key_reference: key_ref,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        };

        tracing::info!("Created wallet from seed phrase: {} ({})", account.name, address);

        Ok(account)
    }

    /// Enhanced wallet creation with BIP-32 HD derivation
    pub async fn create_hd_wallet_from_seed(
        &self,
        wallet_name: String,
        phrase: &SecretString,
        master_password: &SecretString,
        passphrase: Option<&SecretString>,
        derivation_path: Option<&str>,
    ) -> Result<(SecureAccount, ExtendedPrivateKey<SecretKey>)> {
        self.validate_seed_phrase(phrase)?;

        let path = derivation_path.unwrap_or("m/44'/60'/0'/0/0");
        let extended_key = self.derive_hd_wallet_from_seed(phrase, passphrase, path)?;
        let wallet = self.derive_wallet_from_seed(phrase, passphrase, Some(path))?;
        let address = wallet.address();

        let secure_storage = SecureSeedStorage::new(Box::new(crate::security::keychain::OSKeychain::new(
            "vaughan-wallet".to_string(),
        )?));

        let key_ref = secure_storage
            .store_encrypted_seed_phrase(&wallet_name, phrase, master_password)
            .await?;

        let account = SecureAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: wallet_name,
            address,
            key_reference: key_ref,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some(path.to_string()),
        };

        tracing::info!(
            "Created HD wallet from seed phrase: {} ({}) at path: {}",
            account.name,
            address,
            path
        );

        Ok((account, extended_key))
    }

    /// Retrieve seed phrase using enhanced encryption
    pub async fn retrieve_encrypted_seed_phrase(
        &self,
        key_ref: &KeyReference,
        master_password: &SecretString,
    ) -> Result<SecretString> {
        let secure_storage = SecureSeedStorage::new(self.keychain.clone_box());
        secure_storage
            .retrieve_encrypted_seed_phrase(key_ref, master_password)
            .await
    }

    /// Export seed phrase with comprehensive security verification
    pub async fn export_seed_phrase(
        &self,
        key_ref: &KeyReference,
        master_password: &SecretString,
        options: ExportOptions,
        export_password: Option<&SecretString>,
    ) -> Result<ExportResult> {
        let secure_storage = SecureSeedStorage::new(self.keychain.clone_box());
        secure_storage
            .export_seed_phrase(key_ref, master_password, options, export_password)
            .await
    }

    /// Quick export for plain text format
    pub async fn export_plaintext(
        &self,
        key_ref: &KeyReference,
        master_password: &SecretString,
        wallet_name: Option<String>,
    ) -> Result<ExportResult> {
        let options = ExportOptions {
            format: ExportFormat::PlainText,
            include_metadata: false,
            include_derivation_paths: false,
            custom_export_name: wallet_name,
        };
        self.export_seed_phrase(key_ref, master_password, options, None).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::MockKeychain;

    #[test]
    fn test_seed_generation() {
        let keychain = Box::new(MockKeychain::new());
        let seed_manager = SeedManager::new(keychain);

        // Test 12-word generation
        let phrase = seed_manager.generate_seed_phrase(SeedStrength::Words12).unwrap();
        assert_eq!(phrase.expose_secret().split_whitespace().count(), 12);

        // Test 24-word generation
        let phrase = seed_manager.generate_seed_phrase(SeedStrength::Words24).unwrap();
        assert_eq!(phrase.expose_secret().split_whitespace().count(), 24);
    }

    #[test]
    fn test_seed_validation() {
        let keychain = Box::new(MockKeychain::new());
        let seed_manager = SeedManager::new(keychain);

        // Valid phrase
        let valid_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );
        assert!(seed_manager.validate_seed_phrase(&valid_phrase).is_ok());

        // Invalid phrase
        let invalid_phrase = SecretString::new("invalid phrase here".to_string());
        assert!(seed_manager.validate_seed_phrase(&invalid_phrase).is_err());
    }

    #[test]
    fn test_seed_strength_detection() {
        let phrase_12 = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );
        assert_eq!(
            SeedManager::detect_seed_strength(&phrase_12),
            Some(SeedStrength::Words12)
        );
    }
}
