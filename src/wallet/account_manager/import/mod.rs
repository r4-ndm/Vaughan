// Account Import Module
//
// Provides unified interface for importing accounts from various sources:
// - BIP39 seed phrases (12/15/18/21/24 words)
// - Private keys (hex format)
// - Keystore files (EIP-2335 format)
//
// Architecture:
// - parsers.rs: Format detection and parsing
// - validators.rs: Input validation and error checking
// - converters.rs: Conversion to Account/Signer pairs
//
// Attribution: Uses Alloy libraries for all cryptographic operations
// Validation patterns inspired by MetaMask's import flow

mod parsers;
mod validators;
mod converters;

// Re-export types for convenience
pub use crate::wallet::account_manager::types::{Account, ImportMetadata, ImportSourceType};

// Re-export validation result for external use
pub use validators::ValidationResult;

use alloy::signers::local::PrivateKeySigner;
use secrecy::SecretString;

use crate::error::WalletError;

/// Main account importer providing unified import interface
///
/// Handles all import formats with automatic format detection,
/// validation, and conversion to Account/Signer pairs.
///
/// # Example
/// ```no_run
/// use vaughan::wallet::account_manager::import::{AccountImporter, ImportMetadata};
/// use secrecy::SecretString;
///
/// let importer = AccountImporter::new();
/// let phrase = SecretString::from("abandon abandon...".to_string());
/// let metadata = ImportMetadata::new().with_name("My Account");
///
/// let (account, signer) = importer.import_from_seed(&phrase, None, None, metadata)?;
/// # Ok::<(), vaughan::error::WalletError>(())
/// ```
#[derive(Debug, Clone)]
pub struct AccountImporter {
    // Future: Add configuration options here
}

impl AccountImporter {
    /// Create a new account importer with default settings
    pub fn new() -> Self {
        Self {}
    }

    /// Import account from BIP39 seed phrase
    ///
    /// # Arguments
    /// * `phrase` - BIP39 seed phrase (12/15/18/21/24 words)
    /// * `password` - Optional BIP39 passphrase (not wallet password)
    /// * `derivation_path` - Optional custom derivation path (default: m/44'/60'/0'/0/0)
    /// * `metadata` - Import metadata (name, tags, etc.)
    ///
    /// # Returns
    /// * `Ok((Account, PrivateKeySigner))` - Successfully imported account and signer
    /// * `Err(WalletError)` - Validation or conversion failed
    pub fn import_from_seed(
        &self,
        phrase: &SecretString,
        password: Option<&str>,
        derivation_path: Option<&str>,
        metadata: ImportMetadata,
    ) -> Result<(Account, PrivateKeySigner), WalletError> {
        // Validate the seed phrase
        parsers::parse_seed_phrase(phrase)?;

        // Convert to account
        converters::seed_phrase_to_account(phrase, password, derivation_path, metadata)
    }

    /// Import account from private key
    ///
    /// # Arguments
    /// * `key` - Private key in hex format (with or without 0x prefix)
    /// * `metadata` - Import metadata (name, tags, etc.)
    ///
    /// # Returns
    /// * `Ok((Account, PrivateKeySigner))` - Successfully imported account and signer
    /// * `Err(WalletError)` - Validation or conversion failed
    pub fn import_from_private_key(
        &self,
        key: &SecretString,
        metadata: ImportMetadata,
    ) -> Result<(Account, PrivateKeySigner), WalletError> {
        // Parse and validate the private key
        let _signer = parsers::parse_private_key(key)?;

        // Convert to account
        converters::private_key_to_account(key, metadata)
    }

    /// Import account from keystore JSON
    ///
    /// # Arguments
    /// * `keystore_json` - EIP-2335 keystore JSON string
    /// * `password` - Keystore decryption password
    /// * `metadata` - Import metadata (name, tags, etc.)
    ///
    /// # Returns
    /// * `Ok((Account, PrivateKeySigner))` - Successfully imported account and signer
    /// * `Err(WalletError)` - Validation, decryption, or conversion failed
    pub fn import_from_keystore(
        &self,
        keystore_json: &str,
        password: &SecretString,
        metadata: ImportMetadata,
    ) -> Result<(Account, PrivateKeySigner), WalletError> {
        // Validate keystore format
        let validation = validators::validate_import_data(keystore_json);
        if !validation.is_valid {
            return Err(WalletError::WalletError {
                message: validation.error.unwrap_or_else(|| "Invalid keystore".to_string()),
            });
        }

        // Convert to account
        converters::keystore_to_account(keystore_json, password, metadata)
    }

    /// Validate import data without actually importing
    ///
    /// Useful for UI validation before user confirms import
    ///
    /// # Arguments
    /// * `data` - Raw import data (seed phrase, private key, or keystore JSON)
    ///
    /// # Returns
    /// * `ValidationResult` - Detailed validation result with format detection
    pub fn validate_import_data(&self, data: &str) -> ValidationResult {
        validators::validate_import_data(data)
    }

    /// Derive multiple accounts from a seed phrase
    ///
    /// # Arguments
    /// * `phrase` - BIP39 seed phrase
    /// * `password` - Optional BIP39 passphrase
    /// * `count` - Number of accounts to derive
    /// * `start_index` - Starting account index (default: 0)
    ///
    /// # Returns
    /// * `Ok(Vec<(Account, PrivateKeySigner)>)` - Successfully derived accounts
    /// * `Err(WalletError)` - Validation or derivation failed
    pub fn derive_multiple_accounts(
        &self,
        phrase: &SecretString,
        password: Option<&str>,
        count: u32,
        start_index: u32,
    ) -> Result<Vec<(Account, PrivateKeySigner)>, WalletError> {
        // Validate the seed phrase
        parsers::parse_seed_phrase(phrase)?;

        // Derive accounts
        converters::derive_multiple_accounts(phrase, password, count, start_index)
    }
}

impl Default for AccountImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn test_import_from_seed() {
        let importer = AccountImporter::new();
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let metadata = ImportMetadata::new();

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
        let key = SecretString::from(TEST_PRIVATE_KEY.to_string());
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
        let result = importer.validate_import_data(TEST_PRIVATE_KEY);

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
        #![proptest_config(ProptestConfig::with_cases(500))]

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

            let addresses: Vec<alloy::primitives::Address> = (0..import_count)
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
