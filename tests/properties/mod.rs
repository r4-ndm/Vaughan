//! Property-Based Testing Infrastructure
//!
//! This module provides shared utilities and generators for property-based testing
//! of the Vaughan wallet. All property tests follow industry standards:
//! - Memory safety properties: 10,000 iterations
//! - Cryptographic properties: 1,000 iterations
//! - Functional properties: 500 iterations
//!
//! ## Architecture
//! - Uses proptest for property-based testing
//! - Alloy-first approach for all blockchain operations
//! - MetaMask-compatible patterns where Alloy insufficient
//!
//! ## Test Organization
//! - `security.rs` - Memory safety and security properties
//! - `crypto.rs` - Cryptographic correctness properties
//! - `interface.rs` - API consistency properties
//! - `error.rs` - Error handling properties

use proptest::prelude::*;
use secrecy::SecretString;

/// Proptest configuration for memory safety tests (10,000 iterations)
pub fn memory_safety_config() -> ProptestConfig {
    ProptestConfig {
        cases: 10_000,
        max_shrink_iters: 10_000,
        timeout: 60_000, // 60 seconds
        ..ProptestConfig::default()
    }
}

/// Proptest configuration for cryptographic tests (1,000 iterations)
pub fn crypto_config() -> ProptestConfig {
    ProptestConfig {
        cases: 1_000,
        max_shrink_iters: 1_000,
        timeout: 60_000,
        ..ProptestConfig::default()
    }
}

/// Proptest configuration for functional tests (500 iterations)
pub fn functional_config() -> ProptestConfig {
    ProptestConfig {
        cases: 500,
        max_shrink_iters: 500,
        timeout: 30_000, // 30 seconds
        ..ProptestConfig::default()
    }
}

/// Proptest configuration for interface consistency tests (1,000 iterations)
pub fn interface_config() -> ProptestConfig {
    ProptestConfig {
        cases: 1_000,
        max_shrink_iters: 1_000,
        timeout: 60_000,
        ..ProptestConfig::default()
    }
}

/// Generator for valid BIP-39 mnemonics (12, 15, 18, 21, or 24 words)
pub fn arb_mnemonic() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(12usize),
        Just(15usize),
        Just(18usize),
        Just(21usize),
        Just(24usize),
    ]
    .prop_flat_map(|word_count| {
        // Generate random entropy for the mnemonic
        let entropy_bits = word_count * 11 - (word_count / 3);
        let entropy_bytes = entropy_bits / 8;
        
        prop::collection::vec(any::<u8>(), entropy_bytes..=entropy_bytes)
            .prop_map(move |entropy| {
                use bip39::{Mnemonic, Language};
                // Generate mnemonic from entropy
                Mnemonic::from_entropy_in(Language::English, &entropy)
                    .map(|m| m.to_string())
                    .unwrap_or_else(|_| {
                        // Fallback: create a simple valid mnemonic
                        // Use a known valid 12-word mnemonic as fallback
                        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
                    })
            })
    })
}

/// Generator for valid BIP-44 derivation paths
pub fn arb_derivation_path() -> impl Strategy<Value = String> {
    (0u32..10, 0u32..10, 0u32..10, 0u32..10, 0u32..100)
        .prop_map(|(purpose, coin, account, change, index)| {
            format!("m/{}'/{}'/{}'/{}/{}", purpose, coin, account, change, index)
        })
}

/// Generator for valid Ethereum addresses (checksummed)
pub fn arb_eth_address() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<u8>(), 20)
        .prop_map(|bytes| {
            use alloy::primitives::Address;
            let addr = Address::from_slice(&bytes);
            format!("{:?}", addr) // Checksummed format
        })
}

/// Generator for valid passwords (8-128 characters)
pub fn arb_password() -> impl Strategy<Value = SecretString> {
    "[a-zA-Z0-9!@#$%^&*()_+\\-=\\[\\]{}|;:,.<>?]{8,128}"
        .prop_map(|s| SecretString::new(s))
}

/// Generator for account nicknames (1-50 characters, alphanumeric + spaces)
pub fn arb_nickname() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}".prop_map(|s| s.trim().to_string())
}

/// Generator for secret data (for testing zeroization)
pub fn arb_secret_data() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 16..=64)
}

/// Generator for Shamir Secret Sharing configurations (threshold, total_shares)
pub fn arb_shamir_config() -> impl Strategy<Value = (u8, u8)> {
    (2u8..=5, 3u8..=10).prop_filter("threshold <= total_shares", |(threshold, total)| {
        threshold <= total
    })
}

/// Generator for concurrent operation counts (for testing thread safety)
pub fn arb_concurrent_ops() -> impl Strategy<Value = usize> {
    2usize..=20
}

/// Generator for timeout durations (in milliseconds)
pub fn arb_timeout_ms() -> impl Strategy<Value = u64> {
    10u64..=5000
}

/// Helper: Check if memory contains sensitive data
/// 
/// This is a best-effort check for testing purposes.
/// In production, zeroization is handled by the `secrecy` crate.
pub fn contains_sensitive_data(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return false;
    }
    
    haystack.windows(needle.len())
        .any(|window| window == needle)
}

/// Helper: Generate test wallet with random seed
#[cfg(test)]
pub fn create_test_wallet() -> (SecretString, String) {
    use bip39::Mnemonic;
    
    // Use a fixed valid mnemonic for testing
    let mnemonic_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse(mnemonic_str).unwrap();
    let seed = SecretString::new(mnemonic.to_string());
    
    (seed, mnemonic_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_safety_config() {
        let config = memory_safety_config();
        assert_eq!(config.cases, 10_000);
        assert_eq!(config.timeout, 60_000);
    }

    #[test]
    fn test_crypto_config() {
        let config = crypto_config();
        assert_eq!(config.cases, 1_000);
    }

    #[test]
    fn test_functional_config() {
        let config = functional_config();
        assert_eq!(config.cases, 500);
    }

    #[test]
    fn test_generators_compile() {
        // Verify generators compile and produce values
        proptest!(|(
            _mnemonic in arb_mnemonic(),
            _path in arb_derivation_path(),
            _addr in arb_eth_address(),
            _pass in arb_password(),
            _nick in arb_nickname(),
            _secret in arb_secret_data(),
            _shamir in arb_shamir_config(),
            _ops in arb_concurrent_ops(),
            _timeout in arb_timeout_ms(),
        )| {
            // Just verify generators work
        });
    }
}
