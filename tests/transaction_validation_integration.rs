//! Integration tests for transaction validation service
//!
//! These tests verify that the TransactionFormService integrates correctly
//! with the application state and provides consistent validation.

use vaughan::gui::services::{TransactionFormService, TransactionFormServiceTrait};
use alloy::primitives::U256;

#[test]
fn test_service_validates_valid_transaction() {
    let service = TransactionFormService::new();
    
    // Valid recipient
    let recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
    assert!(service.validate_recipient(recipient).is_ok());
    
    // Valid amount within balance
    let amount = "0.5";
    let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
    assert!(service.validate_amount(amount, balance, 18).is_ok());
    
    // Valid gas limit
    let gas_limit = "21000";
    assert!(service.validate_gas_limit(gas_limit).is_ok());
    
    // Valid gas price
    let gas_price = "50"; // 50 Gwei
    assert!(service.validate_gas_price(gas_price).is_ok());
}

#[test]
fn test_service_rejects_invalid_recipient() {
    let service = TransactionFormService::new();
    
    // Invalid address format
    assert!(service.validate_recipient("not-an-address").is_err());
    
    // Zero address
    assert!(service.validate_recipient("0x0000000000000000000000000000000000000000").is_err());
    
    // Empty address
    assert!(service.validate_recipient("").is_err());
}

#[test]
fn test_service_rejects_insufficient_balance() {
    let service = TransactionFormService::new();
    
    let amount = "2.0"; // 2 ETH
    let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
    
    let result = service.validate_amount(amount, balance, 18);
    assert!(result.is_err());
}

#[test]
fn test_service_rejects_invalid_gas_limits() {
    let service = TransactionFormService::new();
    
    // Too low
    assert!(service.validate_gas_limit("20000").is_err());
    
    // Too high
    assert!(service.validate_gas_limit("40000000").is_err());
    
    // Invalid format
    assert!(service.validate_gas_limit("not-a-number").is_err());
}

#[test]
fn test_service_handles_edge_cases() {
    let service = TransactionFormService::new();
    
    // Minimum valid gas limit
    assert!(service.validate_gas_limit("21000").is_ok());
    
    // Maximum valid gas limit
    assert!(service.validate_gas_limit("30000000").is_ok());
    
    // Very small amount
    let amount = "0.000000000000000001"; // 1 wei
    let balance = U256::from(1_000_000_000_000_000_000u128);
    assert!(service.validate_amount(amount, balance, 18).is_ok());
}

#[test]
fn test_balance_check_with_gas_costs() {
    let service = TransactionFormService::new();
    
    // Scenario: Sending 0.5 ETH with gas costs
    let amount = U256::from(500_000_000_000_000_000u128); // 0.5 ETH
    let gas_limit = 21000;
    let gas_price = U256::from(50_000_000_000u128); // 50 Gwei
    
    // Gas cost = 21000 * 50 Gwei = 0.00105 ETH
    // Total needed = 0.5 + 0.00105 = 0.50105 ETH
    
    // Balance exactly enough
    let balance = U256::from(501_050_000_000_000_000u128); // 0.50105 ETH
    assert!(service.check_sufficient_balance(amount, gas_limit, gas_price, balance).is_ok());
    
    // Balance not enough
    let insufficient_balance = U256::from(500_000_000_000_000_000u128); // 0.5 ETH (no gas)
    assert!(service.check_sufficient_balance(amount, gas_limit, gas_price, insufficient_balance).is_err());
}

#[test]
fn test_service_consistency_across_calls() {
    let service = TransactionFormService::new();
    
    let recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
    
    // Multiple calls should give same result
    let result1 = service.validate_recipient(recipient);
    let result2 = service.validate_recipient(recipient);
    
    assert_eq!(result1.is_ok(), result2.is_ok());
    if let (Ok(addr1), Ok(addr2)) = (result1, result2) {
        assert_eq!(addr1, addr2);
    }
}

#[test]
fn test_realistic_transaction_scenario() {
    let service = TransactionFormService::new();
    
    // Realistic scenario: Send 0.1 ETH with standard gas
    let recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
    let amount = "0.1";
    let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
    let gas_limit = "21000";
    let gas_price = "50"; // 50 Gwei
    
    // All validations should pass
    assert!(service.validate_recipient(recipient).is_ok());
    assert!(service.validate_amount(amount, balance, 18).is_ok());
    assert!(service.validate_gas_limit(gas_limit).is_ok());
    assert!(service.validate_gas_price(gas_price).is_ok());
    
    // Balance check should pass
    let amount_wei = U256::from(100_000_000_000_000_000u128); // 0.1 ETH
    let gas_limit_u64 = 21000;
    let gas_price_wei = U256::from(50_000_000_000u128); // 50 Gwei
    
    assert!(service.check_sufficient_balance(amount_wei, gas_limit_u64, gas_price_wei, balance).is_ok());
}
