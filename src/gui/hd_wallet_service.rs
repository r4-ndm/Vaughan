//! HD Wallet Service - Industry Standard BIP39/BIP44 Implementation
//!
//! Provides secure hierarchical deterministic wallet functionality using Alloy.
//! Follows industry standards for mnemonic-based account derivation.

use crate::security::seed::SecureSeedStorage;
use crate::security::{KeyReference, KeychainInterface};
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use secrecy::{ExposeSecret, SecretString};
use std::str::FromStr;

/// HD Wallet service for secure transaction signing
pub struct HDWalletService;

impl HDWalletService {
    /// Create an Ethereum wallet from encrypted seed data
    ///
    /// This is the industry-standard approach:
    /// 1. Decrypt stored BIP39 mnemonic with user password
    /// 2. Derive HD wallet using BIP44 derivation path
    /// 3. Return wallet ready for transaction signing
    pub async fn create_wallet_from_encrypted_seed(
        keychain: &dyn KeychainInterface,
        key_reference: &KeyReference,
        derivation_path: &str,
        master_password: &SecretString,
    ) -> Result<EthereumWallet> {
        tracing::info!(
            "🔐 Creating HD wallet from encrypted seed for account {}",
            key_reference.account
        );

        // Step 1: Retrieve encrypted seed data from keychain
        let _encrypted_seed_json = keychain
            .retrieve(key_reference)
            .context("Failed to retrieve encrypted seed from keychain")?;

        // Step 2: Decrypt the seed data using existing SecureSeedStorage
        let seed_storage = SecureSeedStorage::new(Box::new(crate::security::keychain::OSKeychain::new(
            "vaughan-wallet-encrypted-seeds".to_string(),
        )?));
        let decrypted_mnemonic = seed_storage
            .retrieve_encrypted_seed_phrase(key_reference, master_password)
            .await
            .context("Failed to retrieve and decrypt seed phrase")?;

        // Step 3: Create HD wallet using industry standard BIP39/BIP44
        let wallet = Self::create_hd_wallet(&decrypted_mnemonic, derivation_path)
            .context("Failed to create HD wallet from mnemonic")?;

        tracing::info!(
            "✅ HD wallet created successfully with derivation path: {}",
            derivation_path
        );
        Ok(wallet)
    }

    /// Create HD wallet using Alloy's standard BIP39/BIP44 implementation
    fn create_hd_wallet(mnemonic_phrase: &SecretString, derivation_path: &str) -> Result<EthereumWallet> {
        tracing::debug!("🌱 Creating HD wallet with derivation path: {}", derivation_path);

        // Parse BIP39 mnemonic
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase.expose_secret())
            .context("Invalid BIP39 mnemonic phrase")?;

        // Derive seed from mnemonic
        let seed = mnemonic.to_seed("");

        // Create master key using BIP32 with the correct type
        let mut xprv = bip32::ExtendedPrivateKey::<bip32::secp256k1::SecretKey>::new(seed)
            .map_err(|e| anyhow::anyhow!("Failed to create master key from seed: {e}"))?;

        // Parse derivation path (e.g., "m/44'/60'/0'/0/0")
        let derivation_path =
            bip32::DerivationPath::from_str(derivation_path).context("Invalid BIP44 derivation path")?;

        // Derive along the path step by step (following existing pattern)
        for child in derivation_path.into_iter() {
            xprv = xprv
                .derive_child(child)
                .map_err(|e| anyhow::anyhow!("Failed to derive child key: {e}"))?;
        }

        let account_key = xprv;

        // Create Alloy signer from derived private key
        let private_key_bytes = account_key.private_key().to_bytes();
        let signer =
            PrivateKeySigner::from_slice(&private_key_bytes).context("Failed to create signer from derived key")?;

        // Create Ethereum wallet
        let wallet = EthereumWallet::from(signer);

        tracing::info!(
            "✅ HD wallet created - Address: 0x{:x}",
            wallet.default_signer().address()
        );
        Ok(wallet)
    }

    /// Extract private key from encrypted seed data for direct transaction signing
    ///
    /// This method provides the private key directly for transaction signing,
    /// avoiding the need to extract it from the wallet afterwards
    pub async fn extract_private_key_from_encrypted_seed(
        keychain: &dyn KeychainInterface,
        key_reference: &KeyReference,
        derivation_path: &str,
        master_password: &SecretString,
    ) -> Result<String> {
        tracing::info!(
            "🔐 Extracting private key from encrypted seed for account {}",
            key_reference.account
        );

        // Step 1: Retrieve encrypted seed data from keychain
        let _encrypted_seed_json = keychain
            .retrieve(key_reference)
            .context("Failed to retrieve encrypted seed from keychain")?;

        // Step 2: Decrypt the seed data using existing SecureSeedStorage
        let seed_storage = SecureSeedStorage::new(Box::new(crate::security::keychain::OSKeychain::new(
            "vaughan-wallet-encrypted-seeds".to_string(),
        )?));
        let decrypted_mnemonic = seed_storage
            .retrieve_encrypted_seed_phrase(key_reference, master_password)
            .await
            .context("Failed to retrieve and decrypt seed phrase")?;

        // Step 3: Derive private key using industry standard BIP39/BIP44
        let private_key_hex = Self::derive_private_key(&decrypted_mnemonic, derivation_path)
            .context("Failed to derive private key from mnemonic")?;

        tracing::info!(
            "✅ Private key extracted successfully with derivation path: {}",
            derivation_path
        );
        Ok(private_key_hex)
    }

    /// Derive private key from mnemonic using BIP39/BIP44
    fn derive_private_key(mnemonic_phrase: &SecretString, derivation_path: &str) -> Result<String> {
        tracing::debug!("🌱 Deriving private key with derivation path: {}", derivation_path);

        // Parse BIP39 mnemonic
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase.expose_secret())
            .context("Invalid BIP39 mnemonic phrase")?;

        // Derive seed from mnemonic
        let seed = mnemonic.to_seed("");

        // Create master key using BIP32 with the correct type
        let mut xprv = bip32::ExtendedPrivateKey::<bip32::secp256k1::SecretKey>::new(seed)
            .map_err(|e| anyhow::anyhow!("Failed to create master key from seed: {e}"))?;

        // Parse derivation path (e.g., "m/44'/60'/0'/0/0")
        let derivation_path =
            bip32::DerivationPath::from_str(derivation_path).context("Invalid BIP44 derivation path")?;

        // Derive along the path step by step (following existing pattern)
        for child in derivation_path.into_iter() {
            xprv = xprv
                .derive_child(child)
                .map_err(|e| anyhow::anyhow!("Failed to derive child key: {e}"))?;
        }

        let account_key = xprv;

        // Return private key as hex string
        let private_key_bytes = account_key.private_key().to_bytes();
        Ok(hex::encode(private_key_bytes))
    }

    /// Simple helper to derive a signer for testing/debugging
    pub fn create_signer_from_mnemonic(mnemonic_phrase: &str, derivation_path: &str) -> Result<PrivateKeySigner> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase).context("Invalid mnemonic")?;

        let seed = mnemonic.to_seed("");
        let mut xprv = bip32::ExtendedPrivateKey::<bip32::secp256k1::SecretKey>::new(seed)
            .map_err(|e| anyhow::anyhow!("Failed to create master key: {e}"))?;

        let derivation_path = bip32::DerivationPath::from_str(derivation_path).context("Invalid derivation path")?;

        // Derive along the path
        for child in derivation_path.into_iter() {
            xprv = xprv
                .derive_child(child)
                .map_err(|e| anyhow::anyhow!("Failed to derive child key: {e}"))?;
        }

        let account_key = xprv;

        let private_key_bytes = account_key.private_key().to_bytes();
        let signer = PrivateKeySigner::from_slice(&private_key_bytes).context("Failed to create signer")?;

        Ok(signer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hd_wallet_derivation() {
        // Test with a known mnemonic
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let derivation_path = "m/44'/60'/0'/0/0";

        let signer =
            HDWalletService::create_signer_from_mnemonic(mnemonic, derivation_path).expect("Should create signer");

        // Verify we get a valid Ethereum address
        let address = signer.address();
        println!("Derived address: 0x{:x}", address);

        // This should derive to a known test address
        assert_eq!(format!("0x{:x}", address), "0x9858effd232b4033e47d90003d41ec34ecaeda94");
    }
}
