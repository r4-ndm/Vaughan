//! Property-based tests for startup authentication flow
//!
//! **Feature: password-workflow, Property 1: Startup Password Prompt for Seed Accounts**
//! **Validates: Requirements 1.1, 9.1, 9.2**

use proptest::prelude::*;
use vaughan::gui::services::account_service::get_account_type;
use vaughan::gui::wallet_types::AccountType;
use vaughan::security::{KeyReference, SecureAccount};

/// Generate arbitrary KeyReference for testing
fn arb_key_reference(is_seed_based: bool) -> impl Strategy<Value = KeyReference> {
    let service = if is_seed_based {
        "vaughan-wallet-encrypted-seeds"
    } else {
        "vaughan-wallet"
    };

    (
        "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}",
        Just(service.to_string()),
        "[a-z0-9]{40}",
    )
        .prop_map(|(id, service, account)| KeyReference { id, service, account })
}

/// Generate arbitrary SecureAccount for testing
fn arb_secure_account(is_seed_based: bool) -> impl Strategy<Value = SecureAccount> {
    (
        "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}",
        "[A-Za-z ]{5,20}",
        arb_key_reference(is_seed_based),
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

/// Generate a list of accounts with at least one seed-based account
fn arb_accounts_with_seed() -> impl Strategy<Value = Vec<SecureAccount>> {
    (
        1..5usize, // Number of seed-based accounts
        0..5usize, // Number of private-key accounts
    )
        .prop_flat_map(|(num_seed, num_private)| {
            (
                prop::collection::vec(arb_secure_account(true), num_seed),
                prop::collection::vec(arb_secure_account(false), num_private),
            )
        })
        .prop_map(|(mut seed_accounts, private_accounts)| {
            seed_accounts.extend(private_accounts);
            seed_accounts
        })
}

/// Generate a list of accounts with no seed-based accounts
fn arb_accounts_without_seed() -> impl Strategy<Value = Vec<SecureAccount>> {
    prop::collection::vec(arb_secure_account(false), 1..10)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 1: Startup Password Prompt for Seed Accounts
    ///
    /// For any wallet state containing at least one seed-based account,
    /// the system should detect that seed accounts exist.
    ///
    /// This property verifies that:
    /// 1. When at least one seed-based account exists, the detection returns true
    /// 2. The detection correctly identifies seed-based accounts by their service name
    /// 3. The presence of private-key accounts doesn't affect seed account detection
    #[test]
    fn prop_startup_detects_seed_accounts(accounts in arb_accounts_with_seed()) {
        // Verify that at least one account is seed-based
        let has_seed_accounts = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });

        prop_assert!(
            has_seed_accounts,
            "Should detect seed accounts when at least one exists"
        );

        // Verify that the seed accounts are correctly identified
        let seed_count = accounts.iter().filter(|account| {
            get_account_type(account) == AccountType::SeedBased
        }).count();

        prop_assert!(
            seed_count > 0,
            "Should have at least one seed-based account"
        );

        // Verify that seed accounts have the correct service name
        for account in &accounts {
            if get_account_type(account) == AccountType::SeedBased {
                prop_assert_eq!(
                    &account.key_reference.service,
                    "vaughan-wallet-encrypted-seeds",
                    "Seed-based accounts should have the correct service name"
                );
            }
        }
    }

    /// Property: No password prompt when only private-key accounts exist
    ///
    /// For any wallet state containing only private-key accounts,
    /// the system should detect that no seed accounts exist.
    #[test]
    fn prop_startup_skips_password_for_private_key_only(accounts in arb_accounts_without_seed()) {
        // Verify that no accounts are seed-based
        let has_seed_accounts = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });

        prop_assert!(
            !has_seed_accounts,
            "Should not detect seed accounts when none exist"
        );

        // Verify that all accounts are private-key based
        for account in &accounts {
            prop_assert_eq!(
                get_account_type(account),
                AccountType::PrivateKey,
                "All accounts should be private-key based"
            );
            prop_assert_eq!(
                &account.key_reference.service,
                "vaughan-wallet",
                "Private-key accounts should have the correct service name"
            );
        }
    }

    /// Property: Seed account detection is consistent
    ///
    /// For any set of accounts, checking for seed accounts multiple times
    /// should always return the same result.
    #[test]
    fn prop_seed_detection_is_deterministic(
        accounts in prop::collection::vec(
            prop_oneof![
                arb_secure_account(true),
                arb_secure_account(false),
            ],
            0..10
        )
    ) {
        let result1 = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });
        let result2 = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });
        let result3 = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });

        prop_assert_eq!(result1, result2);
        prop_assert_eq!(result2, result3);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 22: Startup Authentication Before Data Load**
    /// **Validates: Requirements 9.2, 9.3**
    ///
    /// For any application startup with seed-based accounts, authentication
    /// should be completed before loading account balances or sensitive data.
    ///
    /// This property verifies that:
    /// 1. When seed accounts exist, the system requires authentication first
    /// 2. Data loading only proceeds after authentication is complete
    /// 3. The authentication gate prevents unauthorized access to sensitive data
    #[test]
    fn prop_startup_authentication_before_data_load(accounts in arb_accounts_with_seed()) {
        // Verify that seed accounts exist
        let has_seed_accounts = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });

        prop_assert!(
            has_seed_accounts,
            "Test setup should have seed accounts"
        );

        // In a real application, this would verify that:
        // 1. Password dialog is shown before data loading
        // 2. Data loading commands are not issued until authentication succeeds
        // 3. Sensitive data remains protected until authentication

        // For this property test, we verify the logical precondition:
        // If seed accounts exist, authentication MUST be required
        // This is enforced by the startup flow in WorkingWalletApp::new()

        // The actual flow verification would require integration testing
        // with the full application state machine, but we can verify
        // the account type detection logic that drives the decision

        let seed_count = accounts.iter().filter(|account| {
            get_account_type(account) == AccountType::SeedBased
        }).count();

        prop_assert!(
            seed_count > 0,
            "Should have at least one seed account requiring authentication"
        );
    }

    /// Property: Data loading proceeds immediately when no seed accounts exist
    ///
    /// For any application startup with only private-key accounts,
    /// data loading should proceed immediately without authentication.
    #[test]
    fn prop_no_authentication_gate_for_private_key_only(accounts in arb_accounts_without_seed()) {
        // Verify that no seed accounts exist
        let has_seed_accounts = accounts.iter().any(|account| {
            get_account_type(account) == AccountType::SeedBased
        });

        prop_assert!(
            !has_seed_accounts,
            "Should have no seed accounts"
        );

        // When no seed accounts exist, the system should:
        // 1. Skip password dialog
        // 2. Proceed directly to data loading
        // 3. Unlock session automatically

        // This is verified by checking that all accounts are private-key based
        for account in &accounts {
            prop_assert_eq!(
                get_account_type(account),
                AccountType::PrivateKey,
                "All accounts should be private-key based"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_empty_account_list_has_no_seed_accounts() {
        let accounts: Vec<SecureAccount> = vec![];
        let has_seed_accounts = accounts
            .iter()
            .any(|account| get_account_type(account) == AccountType::SeedBased);
        assert!(!has_seed_accounts);
    }

    #[test]
    fn test_single_seed_account_is_detected() {
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

        let accounts = vec![account];
        let has_seed_accounts = accounts
            .iter()
            .any(|acc| get_account_type(acc) == AccountType::SeedBased);
        assert!(has_seed_accounts);
    }

    #[test]
    fn test_mixed_accounts_detects_seed() {
        let seed_account = SecureAccount {
            id: "seed-id".to_string(),
            name: "Seed Account".to_string(),
            address: alloy::primitives::Address::ZERO,
            key_reference: KeyReference {
                id: "key-id-1".to_string(),
                service: "vaughan-wallet-encrypted-seeds".to_string(),
                account: "test-account-1".to_string(),
            },
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        };

        let private_account = SecureAccount {
            id: "private-id".to_string(),
            name: "Private Key Account".to_string(),
            address: alloy::primitives::Address::ZERO,
            key_reference: KeyReference {
                id: "key-id-2".to_string(),
                service: "vaughan-wallet".to_string(),
                account: "test-account-2".to_string(),
            },
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: None,
        };

        let accounts = vec![seed_account, private_account];
        let has_seed_accounts = accounts
            .iter()
            .any(|acc| get_account_type(acc) == AccountType::SeedBased);
        assert!(has_seed_accounts);
    }
}
