// Import Format Converters
//
// This module handles conversion between different import formats and
// creation of Account/Signer pairs from validated import data:
// - Seed phrase → HD wallet account
// - Private key → single account
// - Keystore → decrypted account
//
// Attribution: Conversion logic uses Alloy libraries for all cryptographic operations
// HD wallet derivation follows BIP32/BIP44 standards

use alloy::signers::local::PrivateKeySigner;
use secrecy::{ExposeSecret, SecretString};
use std::str::FromStr;

use crate::error::WalletError;
use crate::wallet::account_manager::types::{Account, ImportMetadata, ImportSourceType};

/// Convert a validated seed phrase into an Account and Signer
///
/// Uses BIP39 for mnemonic handling and BIP32/BIP44 for derivation
/// Default path: m/44'/60'/0'/0/0 (Ethereum standard)
///
/// # Arguments
/// * `phrase` - Validated BIP39 seed phrase
/// * `password` - Optional BIP39 passphrase (not wallet password)
/// * `derivation_path` - Optional custom derivation path
/// * `metadata` - Import metadata (name, tags, etc.)
pub fn seed_phrase_to_account(
    phrase: &SecretString,
    password: Option<&str>,
    derivation_path: Option<&str>,
    metadata: ImportMetadata,
) -> Result<(Account, PrivateKeySigner), WalletError> {
    use alloy::signers::local::MnemonicBuilder;
    use bip39::Mnemonic;

    // Parse and validate mnemonic
    let mnemonic = Mnemonic::from_str(phrase.expose_secret())
        .map_err(|e| WalletError::WalletError { 
            message: format!("Invalid mnemonic: {}", e) 
        })?;

    // Build signer using Alloy's MnemonicBuilder
    let mut builder = MnemonicBuilder::<alloy::signers::local::coins_bip39::English>::default()
        .phrase(mnemonic.to_string());

    // Add BIP39 passphrase if provided
    if let Some(pwd) = password {
        builder = builder.password(pwd);
    }

    // Set derivation path (default: m/44'/60'/0'/0/0)
    let path = derivation_path.unwrap_or("m/44'/60'/0'/0/0");
    builder = builder.derivation_path(path)
        .map_err(|e| WalletError::WalletError { 
            message: format!("Invalid path: {}", e) 
        })?;

    // Build the signer
    let signer = builder.build()
        .map_err(|e| WalletError::WalletError { 
            message: format!("Failed to derive key: {}", e) 
        })?;

    // Create account from signer
    let address = signer.address();
    let account = Account::new_imported(
        address,
        ImportSourceType::SeedPhrase,
        metadata,
    );

    Ok((account, signer))
}

/// Convert a validated private key into an Account and Signer
///
/// Uses Alloy's PrivateKeySigner for secure key handling
///
/// # Arguments
/// * `key` - Validated private key (hex format)
/// * `metadata` - Import metadata (name, tags, etc.)
pub fn private_key_to_account(
    key: &SecretString,
    metadata: ImportMetadata,
) -> Result<(Account, PrivateKeySigner), WalletError> {
    use super::parsers::parse_private_key;

    // Parse the private key
    let signer = parse_private_key(key)?;

    // Create account from signer
    let address = signer.address();
    let account = Account::new_imported(
        address,
        ImportSourceType::PrivateKey,
        metadata,
    );

    Ok((account, signer))
}

/// Convert a keystore JSON into an Account and Signer
///
/// Decrypts EIP-2335 keystore format using provided password
/// Uses Alloy's keystore utilities for decryption
///
/// # Arguments
/// * `keystore_json` - EIP-2335 keystore JSON string
/// * `password` - Keystore decryption password
/// * `metadata` - Import metadata (name, tags, etc.)
pub fn keystore_to_account(
    keystore_json: &str,
    password: &SecretString,
    metadata: ImportMetadata,
) -> Result<(Account, PrivateKeySigner), WalletError> {
    // Parse keystore JSON
    let _keystore: eth_keystore::EthKeystore = serde_json::from_str(keystore_json)
        .map_err(|e| WalletError::WalletError { 
            message: format!("Invalid keystore JSON: {}", e) 
        })?;

    // Decrypt keystore to get private key
    let private_key_bytes = eth_keystore::decrypt_key(keystore_json, password.expose_secret())
        .map_err(|e| WalletError::WalletError { 
            message: format!("Decryption failed: {}", e) 
        })?;

    // Convert bytes to hex string for PrivateKeySigner
    let key_hex = hex::encode(private_key_bytes);
    let signer = PrivateKeySigner::from_str(&key_hex)
        .map_err(|_| WalletError::InvalidPrivateKey)?;

    // Create account from signer
    let address = signer.address();
    let account = Account::new_imported(
        address,
        ImportSourceType::Keystore,
        metadata,
    );

    Ok((account, signer))
}

/// Derive multiple accounts from a seed phrase
///
/// Useful for importing multiple accounts from the same seed
/// Uses sequential derivation: m/44'/60'/0'/0/0, m/44'/60'/0'/0/1, etc.
///
/// # Arguments
/// * `phrase` - Validated BIP39 seed phrase
/// * `password` - Optional BIP39 passphrase
/// * `count` - Number of accounts to derive
/// * `start_index` - Starting account index (default: 0)
pub fn derive_multiple_accounts(
    phrase: &SecretString,
    password: Option<&str>,
    count: u32,
    start_index: u32,
) -> Result<Vec<(Account, PrivateKeySigner)>, WalletError> {
    let mut accounts = Vec::with_capacity(count as usize);

    for i in 0..count {
        let index = start_index + i;
        let path = format!("m/44'/60'/0'/0/{}", index);
        
        let metadata = ImportMetadata::new()
            .with_name(&format!("Account {}", index + 1));

        let (account, signer) = seed_phrase_to_account(
            phrase,
            password,
            Some(&path),
            metadata,
        )?;

        accounts.push((account, signer));
    }

    Ok(accounts)
}

/// Convert legacy format to current Account structure
///
/// Handles migration from older wallet formats
/// This is a placeholder for future migration needs
#[allow(dead_code)] // Placeholder for future migration functionality
pub fn legacy_to_account(
    _legacy_data: &str,
    _metadata: ImportMetadata,
) -> Result<(Account, PrivateKeySigner), WalletError> {
    Err(WalletError::WalletError {
        message: "Legacy format conversion not yet implemented".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn test_seed_phrase_to_account() {
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let metadata = ImportMetadata::new();

        let result = seed_phrase_to_account(&phrase, None, None, metadata);
        assert!(result.is_ok());

        let (account, signer) = result.unwrap();
        assert_eq!(account.address, signer.address());
        assert_eq!(account.source_type, ImportSourceType::SeedPhrase);
    }

    #[test]
    fn test_seed_phrase_determinism() {
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());

        let (account1, _) = seed_phrase_to_account(&phrase, None, None, ImportMetadata::new()).unwrap();
        let (account2, _) = seed_phrase_to_account(&phrase, None, None, ImportMetadata::new()).unwrap();

        // Same seed should produce same address (Property 20)
        assert_eq!(account1.address, account2.address);
    }

    #[test]
    fn test_seed_phrase_with_custom_path() {
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());

        let (account1, _) = seed_phrase_to_account(
            &phrase,
            None,
            Some("m/44'/60'/0'/0/0"),
            ImportMetadata::new(),
        ).unwrap();

        let (account2, _) = seed_phrase_to_account(
            &phrase,
            None,
            Some("m/44'/60'/0'/0/1"),
            ImportMetadata::new(),
        ).unwrap();

        // Different paths should produce different addresses
        assert_ne!(account1.address, account2.address);
    }

    #[test]
    fn test_private_key_to_account() {
        let key = SecretString::from(TEST_PRIVATE_KEY.to_string());
        let metadata = ImportMetadata::new();

        let result = private_key_to_account(&key, metadata);
        assert!(result.is_ok());

        let (account, signer) = result.unwrap();
        assert_eq!(account.address, signer.address());
        assert_eq!(account.source_type, ImportSourceType::PrivateKey);
    }

    #[test]
    fn test_derive_multiple_accounts() {
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());

        let result = derive_multiple_accounts(&phrase, None, 3, 0);
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 3);

        // All addresses should be different
        assert_ne!(accounts[0].0.address, accounts[1].0.address);
        assert_ne!(accounts[1].0.address, accounts[2].0.address);
        assert_ne!(accounts[0].0.address, accounts[2].0.address);
    }

    #[test]
    fn test_metadata_preservation() {
        let phrase = SecretString::from(TEST_MNEMONIC.to_string());
        let metadata = ImportMetadata::new()
            .with_name("Test Account")
            .with_tag("imported");

        let (account, _) = seed_phrase_to_account(&phrase, None, None, metadata).unwrap();

        // Property 22: Metadata should be preserved
        assert_eq!(account.metadata.name, Some("Test Account".to_string()));
        assert!(account.metadata.tags.contains(&"imported".to_string()));
    }
}
