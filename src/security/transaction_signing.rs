//! Transaction Signing Helpers
//!
//! Helper functions for decrypting seeds and deriving keys for transaction signing.

use crate::error::Result;
use crate::security::{KeyReference, SecureMemory, SecureSeedStorage};
use alloy::signers::local::PrivateKeySigner;
use secrecy::SecretString;

/// Decrypt a seed phrase with a password
///
/// This is a convenience wrapper around `SecureSeedStorage::retrieve_encrypted_seed_phrase`
pub async fn decrypt_seed_with_password(
    seed_storage: &SecureSeedStorage,
    key_ref: &KeyReference,
    password: &SecretString,
) -> Result<SecretString> {
    seed_storage.retrieve_encrypted_seed_phrase(key_ref, password).await
}

/// Derive a private key from a seed phrase
///
/// Returns the private key bytes in SecureMemory (auto-zeroized on drop)
pub fn derive_key_from_seed(
    keychain: Box<dyn crate::security::KeychainInterface>,
    seed_phrase: &SecretString,
    derivation_path: Option<&str>,
) -> Result<SecureMemory> {
    use crate::security::seed::SeedManager;

    // Create seed manager
    let seed_manager = SeedManager::new(keychain);

    // Derive wallet from seed
    let wallet = seed_manager.derive_wallet_from_seed(seed_phrase, None, derivation_path)?;

    // Extract private key bytes
    let private_key_bytes = wallet.to_bytes();

    // Store in secure memory
    let mut secure_key = SecureMemory::new(32)?; // Private keys are always 32 bytes
    secure_key.as_mut_slice().copy_from_slice(private_key_bytes.as_slice());

    Ok(secure_key)
}

/// Derive a wallet (PrivateKeySigner) from a seed phrase
///
/// This is useful when you need the full wallet object, not just the key bytes
pub fn derive_wallet_from_seed(
    keychain: Box<dyn crate::security::KeychainInterface>,
    seed_phrase: &SecretString,
    derivation_path: Option<&str>,
) -> Result<PrivateKeySigner> {
    use crate::security::seed::SeedManager;

    let seed_manager = SeedManager::new(keychain);
    seed_manager.derive_wallet_from_seed(seed_phrase, None, derivation_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_from_seed() {
        // Test seed phrase (12 words)
        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );

        // Create keychain
        let keychain = Box::new(crate::security::keychain::OSKeychain::new("test".to_string()).unwrap());

        // Derive key
        let mut secure_key = derive_key_from_seed(keychain, &seed_phrase, None).unwrap();

        // Should have 32 bytes
        assert_eq!(secure_key.as_mut_slice().len(), 32);
    }

    #[test]
    fn test_derive_wallet_from_seed() {
        // Test seed phrase
        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );

        // Create keychain
        let keychain = Box::new(crate::security::keychain::OSKeychain::new("test".to_string()).unwrap());

        // Derive wallet
        let wallet = derive_wallet_from_seed(keychain, &seed_phrase, None).unwrap();

        // Should have a valid address
        assert_ne!(wallet.address(), alloy::primitives::Address::ZERO);
    }

    #[test]
    fn test_derive_with_custom_path() {
        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );

        // Create keychains
        let keychain1 = Box::new(crate::security::keychain::OSKeychain::new("test".to_string()).unwrap());
        let keychain2 = Box::new(crate::security::keychain::OSKeychain::new("test".to_string()).unwrap());

        // Derive with standard Ethereum path
        let wallet1 = derive_wallet_from_seed(keychain1, &seed_phrase, Some("m/44'/60'/0'/0/0")).unwrap();

        // Derive with different index
        let wallet2 = derive_wallet_from_seed(keychain2, &seed_phrase, Some("m/44'/60'/0'/0/1")).unwrap();

        // Should have different addresses
        assert_ne!(wallet1.address(), wallet2.address());
    }
}
