//! Property-based tests for TransactionFormService
//!
//! These tests verify that transaction validation behaves correctly
//! for all possible inputs following Alloy and MetaMask security standards.

use proptest::prelude::*;
use vaughan::gui::services::{TransactionFormService, TransactionFormServiceTrait, TransactionValidationError};
use alloy::primitives::U256;

/// Generate valid Ethereum addresses (40 hex characters with 0x prefix)
fn ethereum_address() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&[
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'a', 'b', 'c', 'd', 'e', 'f',
    ]), 40..=40)
    .prop_map(|chars| format!("0x{}", chars.into_iter().collect::<String>()))
}

/// Generate positive amounts as strings
fn positive_amount() -> impl Strategy<Value = String> {
    (0.000001f64..1000000.0f64)
        .prop_map(|f| format!("{}", f))
}

/// Generate gas limits in reasonable range
fn reasonable_gas_limit() -> impl Strategy<Value = String> {
    (21000u64..30_000_000u64)
        .prop_map(|g| g.to_string())
}

/// Generate gas prices in Gwei
fn reasonable_gas_price() -> impl Strategy<Value = String> {
    (1.0f64..10000.0f64)
        .prop_map(|g| format!("{}", g))
}

proptest! {
    /// Property: Valid Ethereum addresses should always parse successfully
    #[test]
    fn prop_valid_addresses_parse(address in ethereum_address()) {
        let service = TransactionFormService::new();
        let result = service.validate_recipient(&address);
        
        // Should either succeed or fail with zero address error
        if address == "0x0000000000000000000000000000000000000000" {
            prop_assert!(matches!(result, Err(TransactionValidationError::RecipientIsZeroAddress)));
        } else {
            prop_assert!(result.is_ok(), "Valid address should parse: {}", address);
        }
    }

    /// Property: Positive amounts within balance should always validate
    #[test]
    fn prop_positive_amounts_within_balance(amount_str in positive_amount()) {
        let service = TransactionFormService::new();
        // Set balance to 1 million ETH (more than any test amount)
        let balance = U256::from(1_000_000_000_000_000_000_000_000u128);
        
        let result = service.validate_amount(&amount_str, balance, 18);
        prop_assert!(result.is_ok(), "Positive amount within balance should validate: {}", amount_str);
    }

    /// Property: Amounts exceeding balance should always fail
    #[test]
    fn prop_amounts_exceeding_balance_fail(amount_str in positive_amount()) {
        let service = TransactionFormService::new();
        // Parse the amount first
        let amount_f64: f64 = amount_str.parse().unwrap();
        
        // Set balance to half the amount
        let balance_f64 = amount_f64 / 2.0;
        let balance_wei = (balance_f64 * 1e18) as u128;
        let balance = U256::from(balance_wei);
        
        let result = service.validate_amount(&amount_str, balance, 18);
        prop_assert!(
            matches!(result, Err(TransactionValidationError::InsufficientBalance)),
            "Amount {} exceeding balance should fail", amount_str
        );
    }

    /// Property: Zero and negative amounts should always fail
    #[test]
    fn prop_zero_negative_amounts_fail(amount in prop::num::f64::ANY) {
        let service = TransactionFormService::new();
        let balance = U256::from(1_000_000_000_000_000_000u128);
        
        if amount <= 0.0 {
            let result = service.validate_amount(&amount.to_string(), balance, 18);
            prop_assert!(result.is_err(), "Zero or negative amount should fail: {}", amount);
        }
    }

    /// Property: Valid gas limits should always parse
    #[test]
    fn prop_valid_gas_limits_parse(gas_limit in reasonable_gas_limit()) {
        let service = TransactionFormService::new();
        let result = service.validate_gas_limit(&gas_limit);
        prop_assert!(result.is_ok(), "Valid gas limit should parse: {}", gas_limit);
    }

    /// Property: Gas limits below 21000 should always fail
    #[test]
    fn prop_low_gas_limits_fail(gas_limit in 1u64..21000u64) {
        let service = TransactionFormService::new();
        let result = service.validate_gas_limit(&gas_limit.to_string());
        prop_assert!(
            matches!(result, Err(TransactionValidationError::InvalidGasLimit(_))),
            "Gas limit below 21000 should fail: {}", gas_limit
        );
    }

    /// Property: Gas limits above 30M should always fail
    #[test]
    fn prop_high_gas_limits_fail(gas_limit in 30_000_001u64..100_000_000u64) {
        let service = TransactionFormService::new();
        let result = service.validate_gas_limit(&gas_limit.to_string());
        prop_assert!(
            matches!(result, Err(TransactionValidationError::InvalidGasLimit(_))),
            "Gas limit above 30M should fail: {}", gas_limit
        );
    }

    /// Property: Valid gas prices should always parse
    #[test]
    fn prop_valid_gas_prices_parse(gas_price in reasonable_gas_price()) {
        let service = TransactionFormService::new();
        let result = service.validate_gas_price(&gas_price);
        prop_assert!(result.is_ok(), "Valid gas price should parse: {}", gas_price);
    }

    /// Property: Negative gas prices should always fail
    #[test]
    fn prop_negative_gas_prices_fail(gas_price in -10000.0f64..-0.001f64) {
        let service = TransactionFormService::new();
        let result = service.validate_gas_price(&gas_price.to_string());
        prop_assert!(result.is_err(), "Negative gas price should fail: {}", gas_price);
    }

    /// Property: Sufficient balance check should pass when balance > amount + gas
    #[test]
    fn prop_sufficient_balance_check(
        amount_eth in 0.1f64..10.0f64,
        gas_limit in 21000u64..100_000u64,
        gas_price_gwei in 1.0f64..1000.0f64,
    ) {
        let service = TransactionFormService::new();
        
        let amount = U256::from((amount_eth * 1e18) as u128);
        let gas_price = U256::from((gas_price_gwei * 1e9) as u128);
        
        // Calculate required balance
        let gas_cost = U256::from(gas_limit) * gas_price;
        let required = amount + gas_cost;
        
        // Set balance to 2x required (definitely sufficient)
        let balance = required * U256::from(2);
        
        let result = service.check_sufficient_balance(amount, gas_limit, gas_price, balance);
        prop_assert!(result.is_ok(), "Sufficient balance check should pass");
    }

    /// Property: Insufficient balance check should fail when balance < amount + gas
    #[test]
    fn prop_insufficient_balance_check(
        amount_eth in 1.0f64..10.0f64,
        gas_limit in 21000u64..100_000u64,
        gas_price_gwei in 1.0f64..1000.0f64,
    ) {
        let service = TransactionFormService::new();
        
        let amount = U256::from((amount_eth * 1e18) as u128);
        let gas_price = U256::from((gas_price_gwei * 1e9) as u128);
        
        // Calculate required balance
        let gas_cost = U256::from(gas_limit) * gas_price;
        let required = amount + gas_cost;
        
        // Set balance to half required (definitely insufficient)
        let balance = required / U256::from(2);
        
        let result = service.check_sufficient_balance(amount, gas_limit, gas_price, balance);
        prop_assert!(
            matches!(result, Err(TransactionValidationError::InsufficientBalance)),
            "Insufficient balance check should fail"
        );
    }

    /// Property: Address validation should accept both cases
    #[test]
    fn prop_address_case_insensitive(address in ethereum_address()) {
        let service = TransactionFormService::new();
        
        // Skip zero address as it's always rejected
        if address == "0x0000000000000000000000000000000000000000" {
            return Ok(());
        }
        
        let lowercase = address.to_lowercase();
        let uppercase = address.to_uppercase();
        
        let result_lower = service.validate_recipient(&lowercase);
        let result_upper = service.validate_recipient(&uppercase);
        
        // Both should succeed (Alloy accepts both cases)
        prop_assert!(result_lower.is_ok(), "Lowercase should parse: {}", lowercase);
        prop_assert!(result_upper.is_ok(), "Uppercase should parse: {}", uppercase);
    }

    /// Property: Whitespace should be trimmed from addresses
    #[test]
    fn prop_address_whitespace_trimmed(
        address in ethereum_address(),
        leading_spaces in 0usize..5,
        trailing_spaces in 0usize..5,
    ) {
        let service = TransactionFormService::new();
        
        let padded = format!(
            "{}{}{}",
            " ".repeat(leading_spaces),
            address,
            " ".repeat(trailing_spaces)
        );
        
        let result_padded = service.validate_recipient(&padded);
        let result_clean = service.validate_recipient(&address);
        
        // Should have same result
        prop_assert_eq!(result_padded.is_ok(), result_clean.is_ok());
    }

    /// Property: Amount parsing should handle various decimal formats
    #[test]
    fn prop_amount_decimal_formats(
        whole in 0u64..1000u64,
        decimal in 0u64..999999999999999999u64,
    ) {
        let service = TransactionFormService::new();
        let balance = U256::from(1_000_000_000_000_000_000_000_000u128); // 1M ETH
        
        let amount_str = format!("{}.{}", whole, decimal);
        let result = service.validate_amount(&amount_str, balance, 18);
        
        // Should either parse successfully or fail with clear error
        if result.is_err() {
            prop_assert!(
                matches!(result.unwrap_err(), 
                    TransactionValidationError::InvalidAmount(_) |
                    TransactionValidationError::InsufficientBalance |
                    TransactionValidationError::AmountTooSmall
                )
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_zero_address_always_rejected() {
        let service = TransactionFormService::new();
        let zero_addresses = vec![
            "0x0000000000000000000000000000000000000000",
            "0X0000000000000000000000000000000000000000",
        ];
        
        for addr in &zero_addresses {
            let result = service.validate_recipient(addr);
            assert!(matches!(result, Err(TransactionValidationError::RecipientIsZeroAddress)),
                "Zero address should be rejected: {}", addr);
        }
    }

    #[test]
    fn test_amount_precision_18_decimals() {
        let service = TransactionFormService::new();
        let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        
        // Test maximum precision (18 decimals)
        let amount = "0.123456789012345678";
        let result = service.validate_amount(amount, balance, 18);
        assert!(result.is_ok());
        
        // Verify the parsed value (allow for floating point rounding)
        let parsed = result.unwrap();
        let expected = U256::from(123456789012345678u128);
        // Allow small rounding error due to f64 precision
        let diff = if parsed > expected {
            parsed - expected
        } else {
            expected - parsed
        };
        assert!(diff < U256::from(100), "Precision error too large: {} vs {}", parsed, expected);
    }

    #[test]
    fn test_gas_cost_calculation_no_overflow() {
        let service = TransactionFormService::new();
        
        // Test with maximum reasonable values
        let amount = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        let gas_limit = 30_000_000;
        let gas_price = U256::from(10_000_000_000_000u128); // 10000 Gwei
        let balance = U256::from(1_000_000_000_000_000_000_000u128); // 1000 ETH
        
        let result = service.check_sufficient_balance(amount, gas_limit, gas_price, balance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case_minimum_gas_limit() {
        let service = TransactionFormService::new();
        
        // Exactly 21000 should pass
        let result = service.validate_gas_limit("21000");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 21000);
        
        // 20999 should fail
        let result = service.validate_gas_limit("20999");
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_maximum_gas_limit() {
        let service = TransactionFormService::new();
        
        // Exactly 30M should pass
        let result = service.validate_gas_limit("30000000");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30_000_000);
        
        // 30M + 1 should fail
        let result = service.validate_gas_limit("30000001");
        assert!(result.is_err());
    }
}
