//! Property-based tests for AccountDisplayService
//!
//! These tests verify that address formatting behaves correctly
//! for all possible valid Ethereum addresses.

use proptest::prelude::*;
use vaughan::gui::services::{AccountDisplayService, AccountDisplayServiceTrait};

/// Generate valid Ethereum addresses (40 hex characters)
fn ethereum_address() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&[
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'a', 'b', 'c', 'd', 'e', 'f',
        'A', 'B', 'C', 'D', 'E', 'F',
    ]), 40..=40)
    .prop_map(|chars| format!("0x{}", chars.into_iter().collect::<String>()))
}

proptest! {
    /// Property: Short address should always contain "..."
    #[test]
    fn test_short_address_contains_ellipsis(address in ethereum_address()) {
        let service = AccountDisplayService::new();
        let short = service.format_address_short(&address);
        
        prop_assert!(
            short.contains("..."),
            "Short address '{}' should contain '...' for address '{}'",
            short,
            address
        );
    }

    /// Property: Short address should always be shorter than or equal to original
    #[test]
    fn test_short_address_length(address in ethereum_address()) {
        let service = AccountDisplayService::new();
        let short = service.format_address_short(&address);
        
        prop_assert!(
            short.len() <= address.len(),
            "Short address '{}' (len={}) should be <= original '{}' (len={})",
            short,
            short.len(),
            address,
            address.len()
        );
    }

    /// Property: Short address should preserve prefix
    #[test]
    fn test_short_address_preserves_prefix(address in ethereum_address()) {
        let service = AccountDisplayService::new();
        let short = service.format_address_short(&address);
        
        prop_assert!(
            short.starts_with("0x"),
            "Short address '{}' should start with '0x' for address '{}'",
            short,
            address
        );
    }

    /// Property: Short address should preserve suffix
    #[test]
    fn test_short_address_preserves_suffix(address in ethereum_address()) {
        let service = AccountDisplayService::new();
        let short = service.format_address_short(&address);
        
        // Extract the suffix after "..."
        if let Some(ellipsis_pos) = short.find("...") {
            let suffix = &short[ellipsis_pos + 3..];
            let original_suffix = &address[address.len() - 4..];
            
            prop_assert_eq!(
                suffix,
                original_suffix,
                "Short address suffix '{}' should match original suffix '{}' for address '{}'",
                suffix,
                original_suffix,
                address
            );
        }
    }

    /// Property: Formatting should be idempotent (formatting twice gives same result)
    #[test]
    fn test_formatting_idempotent(address in ethereum_address()) {
        let service = AccountDisplayService::new();
        let short1 = service.format_address_short(&address);
        let short2 = service.format_address_short(&short1);
        
        // Second formatting should return the same result (already short)
        prop_assert_eq!(
            &short1,
            &short2,
            "Formatting should be idempotent: '{}' != '{}'",
            short1,
            short2
        );
    }

    /// Property: Empty or very short addresses should be returned as-is
    #[test]
    fn test_short_addresses_unchanged(
        address in prop::string::string_regex("[0-9a-fA-F]{0,10}").unwrap()
    ) {
        let service = AccountDisplayService::new();
        let short = service.format_address_short(&address);
        
        prop_assert_eq!(
            &short,
            &address,
            "Short addresses should be unchanged: '{}' != '{}'",
            short,
            address
        );
    }

    /// Property: Account type labels should be consistent
    #[test]
    fn test_account_type_labels_consistent(
        account_type in prop::sample::select(&["keystore", "KEYSTORE", "Keystore"])
    ) {
        let service = AccountDisplayService::new();
        let label = service.get_account_type_label(&account_type);
        
        prop_assert_eq!(
            &label,
            "Keystore",
            "All keystore variants should map to 'Keystore', got '{}'",
            label
        );
    }

    /// Property: HD wallet type labels should be consistent
    #[test]
    fn test_hd_wallet_labels_consistent(
        account_type in prop::sample::select(&["seed", "hd", "hdwallet", "HD", "SEED"])
    ) {
        let service = AccountDisplayService::new();
        let label = service.get_account_type_label(&account_type);
        
        prop_assert_eq!(
            &label,
            "HD Wallet",
            "All HD wallet variants should map to 'HD Wallet', got '{}'",
            label
        );
    }

    /// Property: Hardware wallet type labels should be consistent
    #[test]
    fn test_hardware_wallet_labels_consistent(
        account_type in prop::sample::select(&["hardware", "ledger", "trezor", "HARDWARE", "LEDGER"])
    ) {
        let service = AccountDisplayService::new();
        let label = service.get_account_type_label(&account_type);
        
        prop_assert_eq!(
            &label,
            "Hardware",
            "All hardware wallet variants should map to 'Hardware', got '{}'",
            label
        );
    }

    /// Property: create_display_info should preserve all input data
    #[test]
    fn test_create_display_info_preserves_data(
        id in "[a-z0-9-]{1,20}",
        name in "[A-Za-z ]{1,30}",
        address in ethereum_address(),
        account_type in prop::sample::select(&["keystore", "seed", "hardware"])
    ) {
        let service = AccountDisplayService::new();
        let info = service.create_display_info(&id, &name, &address, &account_type);
        
        prop_assert_eq!(info.id, id, "ID should be preserved");
        prop_assert_eq!(info.name, name, "Name should be preserved");
        prop_assert_eq!(info.address, address, "Address should be preserved");
        prop_assert!(info.short_address.contains("..."), "Short address should be formatted");
        prop_assert!(!info.account_type.is_empty(), "Account type should be set");
    }

    /// Property: Whitespace should be trimmed from addresses
    #[test]
    fn test_whitespace_trimming(
        address in ethereum_address(),
        leading_spaces in 0usize..5,
        trailing_spaces in 0usize..5
    ) {
        let service = AccountDisplayService::new();
        let padded_address = format!(
            "{}{}{}",
            " ".repeat(leading_spaces),
            address,
            " ".repeat(trailing_spaces)
        );
        
        let short_padded = service.format_address_short(&padded_address);
        let short_clean = service.format_address_short(&address);
        
        prop_assert_eq!(
            &short_padded,
            &short_clean,
            "Whitespace should be trimmed: '{}' != '{}'",
            short_padded,
            short_clean
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_standard_ethereum_address() {
        let service = AccountDisplayService::new();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let short = service.format_address_short(address);
        
        assert_eq!(short, "0x1234...5678");
        assert!(short.contains("..."));
        assert!(short.len() < address.len());
    }

    #[test]
    fn test_empty_address() {
        let service = AccountDisplayService::new();
        let short = service.format_address_short("");
        assert_eq!(short, "");
    }

    #[test]
    fn test_very_short_address() {
        let service = AccountDisplayService::new();
        let address = "0x123";
        let short = service.format_address_short(address);
        assert_eq!(short, address);
    }

    #[test]
    fn test_address_without_0x_prefix() {
        let service = AccountDisplayService::new();
        let address = "1234567890abcdef1234567890abcdef12345678";
        let short = service.format_address_short(address);
        
        assert_eq!(short, "123456...5678");
        assert!(short.contains("..."));
    }

    #[test]
    fn test_account_type_unknown() {
        let service = AccountDisplayService::new();
        let label = service.get_account_type_label("unknown_type");
        assert_eq!(label, "unknown_type");
    }

    #[test]
    fn test_display_info_completeness() {
        let service = AccountDisplayService::new();
        let info = service.create_display_info(
            "test-id",
            "Test Account",
            "0x1234567890abcdef1234567890abcdef12345678",
            "keystore",
        );
        
        assert_eq!(info.id, "test-id");
        assert_eq!(info.name, "Test Account");
        assert_eq!(info.address, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(info.short_address, "0x1234...5678");
        assert_eq!(info.account_type, "Keystore");
    }
}
