// Import Format Parsers
//
// This module handles detection and parsing of different import formats:
// - BIP39 seed phrases (12/15/18/21/24 words)
// - Private keys (hex format with/without 0x prefix)
// - Keystore files (EIP-2335 format)
//
// Attribution: Format detection patterns inspired by MetaMask's import validation
// Alloy libraries used for cryptographic operations

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use secrecy::{ExposeSecret, SecretString};
use std::str::FromStr;

use crate::error::WalletError;
use crate::wallet::account_manager::types::ImportSourceType;

/// Parse result containing detected format and validation info
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub source_type: ImportSourceType,
    pub is_valid: bool,
    #[allow(dead_code)] // Used in tests
    pub word_count: Option<usize>,
    pub error: Option<String>,
}

impl ParseResult {
    pub fn valid(source_type: ImportSourceType) -> Self {
        Self {
            source_type,
            is_valid: true,
            word_count: None,
            error: None,
        }
    }

    pub fn valid_with_words(source_type: ImportSourceType, word_count: usize) -> Self {
        Self {
            source_type,
            is_valid: true,
            word_count: Some(word_count),
            error: None,
        }
    }

    pub fn invalid(error: String) -> Self {
        Self {
            source_type: ImportSourceType::Unknown,
            is_valid: false,
            word_count: None,
            error: Some(error),
        }
    }
}

/// Detect the import format from raw input data
///
/// Supports:
/// - BIP39 seed phrases (12/15/18/21/24 words)
/// - Private keys (64 hex chars with optional 0x prefix)
/// - Keystore JSON (EIP-2335 format detection)
pub fn detect_import_format(data: &str) -> ParseResult {
    let trimmed = data.trim();

    // Check for keystore JSON format
    if trimmed.starts_with('{') && trimmed.contains("\"crypto\"") {
        return ParseResult::valid(ImportSourceType::Keystore);
    }

    // Check for private key format (hex string)
    if is_private_key_format(trimmed) {
        return ParseResult::valid(ImportSourceType::PrivateKey);
    }

    // Check for seed phrase format
    if let Some(word_count) = is_seed_phrase_format(trimmed) {
        return ParseResult::valid_with_words(ImportSourceType::SeedPhrase, word_count);
    }

    ParseResult::invalid("Unknown import format".to_string())
}

/// Check if data looks like a private key (hex format)
fn is_private_key_format(data: &str) -> bool {
    let cleaned = data.strip_prefix("0x").unwrap_or(data);
    
    // Private key should be 64 hex characters
    cleaned.len() == 64 && cleaned.chars().all(|c| c.is_ascii_hexdigit())
}

/// Check if data looks like a BIP39 seed phrase
///
/// Returns word count if valid format, None otherwise
fn is_seed_phrase_format(data: &str) -> Option<usize> {
    let words: Vec<&str> = data.split_whitespace().collect();
    let word_count = words.len();

    // BIP39 supports 12, 15, 18, 21, or 24 word phrases
    match word_count {
        12 | 15 | 18 | 21 | 24 => {
            // Basic validation: all words should be lowercase alphabetic
            if words.iter().all(|w| w.chars().all(|c| c.is_ascii_lowercase())) {
                Some(word_count)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse a private key string into a PrivateKeySigner
///
/// Accepts hex strings with or without 0x prefix
/// Uses Alloy's PrivateKeySigner for secure key handling
pub fn parse_private_key(key_data: &SecretString) -> Result<PrivateKeySigner, WalletError> {
    let key_str = key_data.expose_secret();
    let cleaned = key_str.trim().strip_prefix("0x").unwrap_or(key_str.trim());

    PrivateKeySigner::from_str(cleaned)
        .map_err(|_| WalletError::InvalidPrivateKey)
}

/// Parse and validate a BIP39 seed phrase
///
/// Returns the phrase if valid, error otherwise
/// Uses bip39 crate for standard-compliant validation
pub fn parse_seed_phrase(phrase_data: &SecretString) -> Result<String, WalletError> {
    use bip39::Mnemonic;

    let phrase_str = phrase_data.expose_secret().trim();
    
    // Validate using bip39 crate
    Mnemonic::from_str(phrase_str)
        .map(|m| m.to_string())
        .map_err(|e| WalletError::WalletError { 
            message: format!("Invalid BIP39 phrase: {}", e) 
        })
}

/// Extract address from a signer
///
/// Helper function to get the address from an Alloy signer
#[allow(dead_code)] // Utility function for future use
pub fn extract_address(signer: &PrivateKeySigner) -> Address {
    signer.address()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn test_detect_seed_phrase() {
        let result = detect_import_format(TEST_MNEMONIC);
        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::SeedPhrase);
        assert_eq!(result.word_count, Some(12));
    }

    #[test]
    fn test_detect_private_key() {
        let result = detect_import_format(TEST_PRIVATE_KEY);
        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_detect_private_key_without_prefix() {
        let key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let result = detect_import_format(key);
        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_detect_keystore_json() {
        let json = r#"{"crypto": {"cipher": "aes-128-ctr"}}"#;
        let result = detect_import_format(json);
        assert!(result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::Keystore);
    }

    #[test]
    fn test_detect_unknown_format() {
        let result = detect_import_format("random invalid data");
        assert!(!result.is_valid);
        assert_eq!(result.source_type, ImportSourceType::Unknown);
    }

    #[test]
    fn test_parse_valid_private_key() {
        let key = SecretString::from(TEST_PRIVATE_KEY.to_string());
        let result = parse_private_key(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_private_key() {
        let key = SecretString::from("not a valid key".to_string());
        let result = parse_private_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid_seed_phrase() {
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let result = parse_seed_phrase(&phrase);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_seed_phrase() {
        let phrase = SecretString::from("invalid seed phrase words".to_string());
        let result = parse_seed_phrase(&phrase);
        assert!(result.is_err());
    }
}
