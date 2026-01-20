//! Property-based tests for account type detection
//!
//! **Feature: password-workflow, Property 14: Account Type Detection**
//! **Validates: Requirements 5.1**

use proptest::prelude::*;
use vaughan::gui::services::account_service::{get_account_type, is_seed_based_account};
use vaughan::gui::wallet_types::AccountType;
use vaughan::security::{KeyReference, SecureAccount};

/// Generate arbitrary KeyReference for testing
fn arb_key_reference() -> impl Strategy<Value = KeyReference> {
    (
        "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}",
        prop_oneof![
            Just("vaughan-wallet-encrypted-seeds".to_string()),
            Just("vaughan-wallet".to_string()),
        ],
        "[a-z0-9]{40}",
    )
        .prop_map(|(id, service, account)| KeyReference { id, service, account })
}

/// Generate arbitrary SecureAccount for testing
fn arb_secure_account() -> impl Strategy<Value = SecureAccount> {
    (
        "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}",
        "[A-Za-z ]{5,20}",
        arb_key_reference(),
        any::<bool>(),
        prop_oneof![Just(Some("m/44'/60'/0'/0/0".to_string())), Just(None),],
    )
        .prop_map(|(id, name, key_reference, is_hardware, derivation_path)| {
            // Generate a random address (20 bytes)
            let address_bytes: [u8; 20] = rand::random();
            let address = alloy::primitives::Address::from(address_bytes);

            SecureAccount {
                id,
                name,
                address,
                key_reference,
                created_at: chrono::Utc::now(),
                is_hardware,
                derivation_path,
            }
        })
}

proptest! {
    /// Property 14: Account Type Detection
    ///
    /// For any set of accounts, the wallet should correctly identify each account
    /// as either seed-based or private-key based according to its KeyReference service.
    ///
    /// This property verifies that:
    /// 1. Accounts with service "vaughan-wallet-encrypted-seeds" are identified as SeedBased
    /// 2. Accounts with service "vaughan-wallet" are identified as PrivateKey
    /// 3. The is_seed_based_account helper returns consistent results
    #[test]
    fn prop_account_type_detection_is_correct(account in arb_secure_account()) {
        let detected_type = get_account_type(&account);
        let is_seed = is_seed_based_account(&account);

        // Verify that the account type matches the service name
        if account.key_reference.service == "vaughan-wallet-encrypted-seeds" {
            prop_assert_eq!(detected_type, AccountType::SeedBased);
            prop_assert!(is_seed);
        } else if account.key_reference.service == "vaughan-wallet" {
            prop_assert_eq!(detected_type, AccountType::PrivateKey);
            prop_assert!(!is_seed);
        }

        // Verify consistency between get_account_type and is_seed_based_account
        prop_assert_eq!(
            is_seed,
            detected_type == AccountType::SeedBased,
            "is_seed_based_account should be consistent with get_account_type"
        );
    }

    /// Property: Account type detection is deterministic
    ///
    /// For any account, calling get_account_type multiple times should
    /// always return the same result.
    #[test]
    fn prop_account_type_detection_is_deterministic(account in arb_secure_account()) {
        let type1 = get_account_type(&account);
        let type2 = get_account_type(&account);
        let type3 = get_account_type(&account);

        prop_assert_eq!(type1, type2);
        prop_assert_eq!(type2, type3);
    }

    /// Property: Account type is based solely on service name
    ///
    /// For any two accounts with the same service name but different other fields,
    /// they should have the same account type.
    #[test]
    fn prop_account_type_depends_only_on_service(
        account1 in arb_secure_account(),
        account2 in arb_secure_account()
    ) {
        // If both accounts have the same service, they should have the same type
        if account1.key_reference.service == account2.key_reference.service {
            prop_assert_eq!(
                get_account_type(&account1),
                get_account_type(&account2),
                "Accounts with same service should have same type"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_seed_based_account_detection() {
        let account = SecureAccount {
            id: "test-id".to_string(),
            name: "Test Seed Account".to_string(),
            address: alloy::primitives::Address::ZERO,
            key_reference: KeyReference {
                id: "key-id".to_string(),
                service: "vaughan-wallet-encrypted-seeds".to_string(),
                account: "test-account".to_string(),
            },
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        };

        assert_eq!(get_account_type(&account), AccountType::SeedBased);
        assert!(is_seed_based_account(&account));
    }

    #[test]
    fn test_private_key_account_detection() {
        let account = SecureAccount {
            id: "test-id".to_string(),
            name: "Test Private Key Account".to_string(),
            address: alloy::primitives::Address::ZERO,
            key_reference: KeyReference {
                id: "key-id".to_string(),
                service: "vaughan-wallet".to_string(),
                account: "test-account".to_string(),
            },
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: None,
        };

        assert_eq!(get_account_type(&account), AccountType::PrivateKey);
        assert!(!is_seed_based_account(&account));
    }
}
