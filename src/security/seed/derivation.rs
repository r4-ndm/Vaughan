//! BIP32/BIP39 seed derivation and HD wallet creation
//!
//! This module provides hierarchical deterministic (HD) wallet derivation
//! following BIP-32, BIP-39, and BIP-44 standards for Ethereum.
//!
//! Key features:
//! - Mnemonic to seed conversion
//! - HD key derivation following standard paths
//! - Multi-account derivation
//! - Alloy-compatible wallet generation

use crate::error::{Result, SecurityError};
use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use bip32::{secp256k1::SecretKey, ExtendedPrivateKey};
use bip39::Mnemonic;
use k256::ecdsa::SigningKey;
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;

use super::types::{
    DerivationPathConfig, DerivedAccount, MultiAccountDerivation, SecureSeed, SeedAnalysis, SeedStrength,
};

// ============================================================================
// Seed Generation
// ============================================================================

/// Generate seed from mnemonic phrase
pub fn phrase_to_seed(phrase: &SecretString, passphrase: Option<&SecretString>) -> Result<SecureSeed> {
    let phrase_str = phrase.expose_secret();

    // Parse mnemonic
    let mnemonic = Mnemonic::parse(phrase_str).map_err(|e| SecurityError::InvalidSeedPhrase {
        reason: format!("Invalid mnemonic: {e}"),
    })?;

    // Generate seed with optional passphrase
    let passphrase_str = passphrase.map(|p| p.expose_secret().as_str()).unwrap_or("");

    let seed = mnemonic.to_seed(passphrase_str);

    // Convert to our secure seed format
    let mut seed_bytes = [0u8; 64];
    seed_bytes.copy_from_slice(&seed);

    Ok(SecureSeed::from_bytes(seed_bytes))
}

// ============================================================================
// Wallet Derivation
// ============================================================================

/// Derive Ethereum wallet from seed phrase
pub fn derive_wallet_from_seed(
    phrase: &SecretString,
    passphrase: Option<&SecretString>,
    derivation_path: Option<&str>,
) -> Result<PrivateKeySigner> {
    use bip32::DerivationPath;
    use std::str::FromStr;

    // Convert to BIP-39 seed
    let phrase_str = phrase.expose_secret();
    let mnemonic =
        Mnemonic::parse(phrase_str).map_err(|e| SecurityError::InvalidSeedPhrase { reason: e.to_string() })?;
    let bip39_pass = passphrase.map(|p| p.expose_secret().as_str()).unwrap_or("");
    let seed = mnemonic.to_seed(bip39_pass);

    // Parse derivation path (default to standard Ethereum path)
    let path_str = derivation_path.unwrap_or("m/44'/60'/0'/0/0");
    let path = DerivationPath::from_str(path_str).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Invalid derivation path '{path_str}': {e}"),
    })?;

    // Master extended private key
    let mut xprv = ExtendedPrivateKey::<SecretKey>::new(seed).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Failed to create master key: {e}"),
    })?;

    // Derive along the path
    for child in path.into_iter() {
        xprv = xprv
            .derive_child(child)
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Failed to derive child key: {e}"),
            })?;
    }

    // Build signing key from derived secret key bytes
    let secret_bytes = xprv.private_key().to_bytes();
    let signing_key = SigningKey::from_bytes(&secret_bytes).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Failed to build signing key: {e}"),
    })?;
    let wallet = PrivateKeySigner::from(signing_key);
    tracing::info!("Derived wallet from seed (HD), address: {:?}", wallet.address());
    Ok(wallet)
}

/// Enhanced BIP-32 compliant HD wallet derivation
pub fn derive_hd_wallet_from_seed(
    phrase: &SecretString,
    passphrase: Option<&SecretString>,
    derivation_path: &str,
) -> Result<ExtendedPrivateKey<SecretKey>> {
    use bip32::DerivationPath;
    use std::str::FromStr;

    // Convert mnemonic to seed
    let phrase_str = phrase.expose_secret();
    let mnemonic =
        Mnemonic::parse(phrase_str).map_err(|e| SecurityError::InvalidSeedPhrase { reason: e.to_string() })?;
    let bip39_pass = passphrase.map(|p| p.expose_secret().as_str()).unwrap_or("");
    let seed = mnemonic.to_seed(bip39_pass);

    // Master extended private key
    let mut xprv = ExtendedPrivateKey::<SecretKey>::new(seed).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Failed to create master key: {e}"),
    })?;

    // Parse and derive along path
    let path = DerivationPath::from_str(derivation_path).map_err(|e| SecurityError::KeyDerivationError {
        message: format!("Invalid derivation path '{derivation_path}': {e}"),
    })?;
    for child in path.into_iter() {
        xprv = xprv
            .derive_child(child)
            .map_err(|e| SecurityError::KeyDerivationError {
                message: format!("Failed to derive child key: {e}"),
            })?;
    }

    tracing::info!("Derived HD key for path: {}", derivation_path);
    Ok(xprv)
}

// ============================================================================
// Multi-Account Derivation
// ============================================================================

/// Derive multiple HD wallet accounts from seed phrase
pub fn derive_multiple_hd_accounts(
    phrase: &SecretString,
    passphrase: Option<&SecretString>,
    base_path: &str,
    account_count: u32,
) -> Result<Vec<(u32, ExtendedPrivateKey<SecretKey>, Address)>> {
    let mut accounts = Vec::new();

    for account_index in 0..account_count {
        // Create account-specific derivation path
        let account_path = format!("{base_path}/{account_index}");

        // Derive the extended private key
        let extended_key = derive_hd_wallet_from_seed(phrase, passphrase, &account_path)?;

        // Compute address for the derived key
        // Re-derive wallet at this path to obtain the address
        let wallet = derive_wallet_from_seed(phrase, passphrase, Some(&account_path))?;
        let address = wallet.address();

        accounts.push((account_index, extended_key, address));
    }

    tracing::info!("Derived {} HD wallet accounts from path: {}", account_count, base_path);

    Ok(accounts)
}

/// Derive multiple accounts from a single seed phrase using derivation config
pub fn derive_multiple_accounts_from_seed(
    phrase: &SecretString,
    derivation_config: &DerivationPathConfig,
    account_count: u32,
    passphrase: Option<&SecretString>,
) -> Result<MultiAccountDerivation> {
    // Validate the seed phrase first
    super::validation::validate_seed_phrase(phrase)?;

    let mut accounts = Vec::new();

    for index in 0..account_count {
        // Create derivation path for this account
        let account_path = if derivation_config.path.ends_with("/0") {
            // Replace the last /0 with the account index
            derivation_config.path.replace("/0", &format!("/{index}"))
        } else {
            // Append the account index
            format!("{}/{}", derivation_config.path, index)
        };

        // Derive wallet for this path
        let wallet = derive_wallet_from_seed(phrase, passphrase, Some(&account_path))?;

        accounts.push(DerivedAccount {
            index,
            address: wallet.address(),
            derivation_path: account_path,
            public_key: None, // Could be added if needed
        });
    }

    tracing::info!(
        "Derived {} accounts from seed phrase using path: {}",
        account_count,
        derivation_config.path
    );

    Ok(MultiAccountDerivation {
        accounts,
        derivation_path: derivation_config.path.clone(),
        total_derived: account_count as usize,
    })
}

// ============================================================================
// Seed Analysis
// ============================================================================

/// Analyze imported seed phrase strength and provide recommendations
pub fn analyze_imported_seed_phrase(phrase: &SecretString) -> Result<SeedAnalysis> {
    let phrase_str = phrase.expose_secret();
    let words: Vec<&str> = phrase_str.split_whitespace().collect();
    let word_count = words.len();

    let strength = match word_count {
        12 => SeedStrength::Words12,
        15 => SeedStrength::Words15,
        18 => SeedStrength::Words18,
        21 => SeedStrength::Words21,
        24 => SeedStrength::Words24,
        _ => {
            return Err(SecurityError::InvalidSeedPhrase {
                reason: format!("Invalid word count: {word_count}"),
            }
            .into());
        }
    };

    let mut warnings = Vec::new();

    // Check for security issues
    let mut word_counts = HashMap::new();
    for word in &words {
        *word_counts.entry(*word).or_insert(0) += 1;
    }

    for (word, count) in word_counts {
        if count > 1 {
            warnings.push(format!("Repeated word '{word}' reduces entropy"));
        }
    }

    // Validate BIP39 checksum
    let is_valid = Mnemonic::parse(phrase_str).is_ok();

    Ok(SeedAnalysis {
        strength,
        entropy_bits: strength.entropy_bits(),
        security_level: strength.security_level().to_string(),
        recommendation: strength.security_recommendation().to_string(),
        brute_force_estimate: strength.brute_force_time_estimate().to_string(),
        warnings,
        is_valid,
    })
}

/// Get standard Ethereum derivation paths
pub fn get_standard_derivation_paths() -> Vec<DerivationPathConfig> {
    vec![
        DerivationPathConfig::ethereum_standard(),     // m/44'/60'/0'/0/0
        DerivationPathConfig::ethereum_account_base(), // m/44'/60'/0'/0
        DerivationPathConfig::ledger_live(),           // m/44'/60'/0'/0
        DerivationPathConfig::legacy(),                // m/44'/60'/0'
    ]
}
