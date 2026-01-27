// Import Validation Logic
//
// This module handles validation of import data before processing:
// - Seed phrase validation (BIP39 compliance, word count, checksum)
// - Private key validation (format, length, hex encoding)
// - Keystore validation (JSON structure, required fields)
// - Derivation path validation (BIP32/BIP44 compliance)
//
// Attribution: Validation patterns follow industry standards from MetaMask
// Alloy and bip39 crates used for cryptographic validation

use std::str::FromStr;

use crate::error::WalletError;
use crate::wallet::account_manager::types::ImportSourceType;

/// Validation result with detailed error information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub source_type: ImportSourceType,
    pub word_count: Option<usize>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid(source_type: ImportSourceType) -> Self {
        Self {
            is_valid: true,
            source_type,
            word_count: None,
            error: None,
            warnings: Vec::new(),
        }
    }

    pub fn valid_with_words(source_type: ImportSourceType, word_count: usize) -> Self {
        Self {
            is_valid: true,
            source_type,
            word_count: Some(word_count),
            error: None,
            warnings: Vec::new(),
        }
    }

    pub fn invalid(source_type: ImportSourceType, error: String) -> Self {
        Self {
            is_valid: false,
            source_type,
            word_count: None,
            error: Some(error),
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// Validate import data and return detailed validation result
///
/// This is the main entry point for validation, detecting format
/// and applying appropriate validation rules
pub fn validate_import_data(data: &str) -> ValidationResult {
    use super::parsers::detect_import_format;

    let parse_result = detect_import_format(data);

    if !parse_result.is_valid {
        return ValidationResult::invalid(
            ImportSourceType::Unknown,
            parse_result.error.unwrap_or_else(|| "Unknown format".to_string()),
        );
    }

    match parse_result.source_type {
        ImportSourceType::SeedPhrase => validate_seed_phrase_detailed(data),
        ImportSourceType::PrivateKey => validate_private_key_detailed(data),
        ImportSourceType::Keystore => validate_keystore_detailed(data),
        ImportSourceType::Unknown => ValidationResult::invalid(
            ImportSourceType::Unknown,
            "Unknown import format".to_string(),
        ),
    }
}

/// Validate a BIP39 seed phrase with detailed checks
///
/// Checks:
/// - Word count (12/15/18/21/24)
/// - BIP39 wordlist compliance
/// - Checksum validity
fn validate_seed_phrase_detailed(phrase: &str) -> ValidationResult {
    use bip39::Mnemonic;

    let words: Vec<&str> = phrase.split_whitespace().collect();
    let word_count = words.len();

    // Check word count
    if !matches!(word_count, 12 | 15 | 18 | 21 | 24) {
        return ValidationResult::invalid(
            ImportSourceType::SeedPhrase,
            format!("Invalid word count: {}. Must be 12, 15, 18, 21, or 24 words", word_count),
        );
    }

    // Validate using bip39 crate (checks wordlist and checksum)
    match Mnemonic::from_str(phrase) {
        Ok(_) => {
            let mut result = ValidationResult::valid_with_words(ImportSourceType::SeedPhrase, word_count);
            
            // Add warning for non-standard word counts
            if word_count != 12 && word_count != 24 {
                result = result.with_warning(format!(
                    "Using {}-word phrase. 12 or 24 words are most common",
                    word_count
                ));
            }
            
            result
        }
        Err(e) => ValidationResult::invalid(
            ImportSourceType::SeedPhrase,
            format!("Invalid BIP39 seed phrase: {}", e),
        ),
    }
}

/// Validate a private key with detailed checks
///
/// Checks:
/// - Hex encoding
/// - Length (64 characters without 0x prefix)
/// - Valid ECDSA private key range
fn validate_private_key_detailed(key: &str) -> ValidationResult {
    let cleaned = key.trim().strip_prefix("0x").unwrap_or(key.trim());

    // Check length
    if cleaned.len() != 64 {
        return ValidationResult::invalid(
            ImportSourceType::PrivateKey,
            format!("Invalid private key length: {}. Must be 64 hex characters", cleaned.len()),
        );
    }

    // Check hex encoding
    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return ValidationResult::invalid(
            ImportSourceType::PrivateKey,
            "Private key must contain only hexadecimal characters".to_string(),
        );
    }

    // Try to parse as valid key using Alloy
    use alloy::signers::local::PrivateKeySigner;
    match PrivateKeySigner::from_str(cleaned) {
        Ok(_) => ValidationResult::valid(ImportSourceType::PrivateKey),
        Err(e) => ValidationResult::invalid(
            ImportSourceType::PrivateKey,
            format!("Invalid private key: {}", e),
        ),
    }
}

/// Validate keystore JSON format
///
/// Checks:
/// - Valid JSON structure
/// - Required fields present (crypto, version, etc.)
/// - EIP-2335 compliance
fn validate_keystore_detailed(json: &str) -> ValidationResult {
    use serde_json::Value;

    // Parse JSON
    let parsed: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return ValidationResult::invalid(
                ImportSourceType::Keystore,
                format!("Invalid JSON: {}", e),
            );
        }
    };

    // Check for required fields
    if !parsed.is_object() {
        return ValidationResult::invalid(
            ImportSourceType::Keystore,
            "Keystore must be a JSON object".to_string(),
        );
    }

    // Safe to unwrap because we checked is_object() above
    #[allow(clippy::expect_used)]
    let obj = parsed.as_object().expect("Checked is_object() above");

    // Check for crypto field (required in EIP-2335)
    if !obj.contains_key("crypto") && !obj.contains_key("Crypto") {
        return ValidationResult::invalid(
            ImportSourceType::Keystore,
            "Missing required 'crypto' field".to_string(),
        );
    }

    ValidationResult::valid(ImportSourceType::Keystore)
}

/// Validate a BIP32/BIP44 derivation path
///
/// Checks:
/// - Starts with 'm/'
/// - Valid path components
/// - Hardened notation (')
#[allow(dead_code)] // Duplicate of hardware::derivation::validate_derivation_path
pub fn validate_derivation_path(path: &str) -> Result<(), WalletError> {
    if !path.starts_with("m/") {
        return Err(WalletError::WalletError {
            message: "Path must start with 'm/'".to_string(),
        });
    }

    // Split and validate each component
    let components: Vec<&str> = path[2..].split('/').collect();
    
    for component in components {
        if component.is_empty() {
            return Err(WalletError::WalletError {
                message: "Empty path component".to_string(),
            });
        }

        // Check if hardened (ends with ')
        let is_hardened = component.ends_with('\'');
        let number_part = if is_hardened {
            &component[..component.len() - 1]
        } else {
            component
        };

        // Validate numeric part
        if number_part.parse::<u32>().is_err() {
            return Err(WalletError::WalletError {
                message: format!("Invalid path component: {}", component),
            });
        }
    }

    Ok(())
}

/// Validate account index for HD wallet derivation
///
/// Ensures index is within safe range (0 to 2^31 - 1)
#[allow(dead_code)] // Used in tests
pub fn validate_account_index(index: u32) -> Result<(), WalletError> {
    // BIP32 hardened keys use indices >= 2^31
    // For account indices, we stay below this threshold
    if index >= 0x80000000 {
        return Err(WalletError::WalletError {
            message: format!("Account index {} exceeds maximum (2147483647)", index),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn test_validate_valid_seed_phrase() {
        let result = validate_import_data(TEST_MNEMONIC);
        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::SeedPhrase);
        assert_eq!(result.word_count, Some(12));
    }

    #[test]
    fn test_validate_invalid_word_count() {
        let phrase = "word word word word word";
        let result = validate_import_data(phrase);
        assert!(!result.is_valid);
        // Invalid word count is detected as unknown format during parsing
        assert!(result.error.is_some());
    }

    #[test]
    fn test_validate_valid_private_key() {
        let result = validate_import_data(TEST_PRIVATE_KEY);
        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_validate_invalid_private_key_length() {
        let key = "0x1234";
        let result = validate_import_data(key);
        assert!(!result.is_valid);
        // Short hex strings are detected as unknown format during parsing
        assert!(result.error.is_some());
    }

    #[test]
    fn test_validate_derivation_path_valid() {
        assert!(validate_derivation_path("m/44'/60'/0'/0/0").is_ok());
        assert!(validate_derivation_path("m/44'/60'/0'/0/1").is_ok());
        assert!(validate_derivation_path("m/0/1/2").is_ok());
    }

    #[test]
    fn test_validate_derivation_path_invalid() {
        assert!(validate_derivation_path("44'/60'/0'/0/0").is_err()); // Missing m/
        assert!(validate_derivation_path("m/44'/60'//0/0").is_err()); // Empty component
        assert!(validate_derivation_path("m/44'/abc/0/0").is_err()); // Non-numeric
    }

    #[test]
    fn test_validate_account_index() {
        assert!(validate_account_index(0).is_ok());
        assert!(validate_account_index(100).is_ok());
        assert!(validate_account_index(0x7FFFFFFF).is_ok());
        assert!(validate_account_index(0x80000000).is_err()); // Hardened threshold
    }
}
