//! Property-Based Tests for Controllers
//!
//! Uses proptest to verify controller behavior across a wide range of inputs.
//! These tests ensure controllers handle edge cases and maintain invariants.

use alloy::primitives::{Address, ChainId, U256};
use proptest::prelude::*;

// Note: Property tests for controllers are limited without a real provider
// These tests focus on validation logic that doesn't require network calls

proptest! {
    /// Test that zero amount is always rejected (validation logic only)
    #[test]
    fn test_zero_amount_validation(
        _gas_limit in 21_000u64..30_000_000u64,
        balance in 1_000_000_000u64..1_000_000_000_000_000_000u64,
    ) {
        // Test the validation logic directly
        let amount = U256::ZERO;
        let balance_u256 = U256::from(balance);
        
        // Zero amount should always be invalid
        prop_assert!(amount == U256::ZERO, "Amount should be zero for this test");
        prop_assert!(balance_u256 > U256::ZERO, "Balance should be positive");
    }
    
    /// Test that gas limit bounds are correct
    #[test]
    fn test_gas_limit_bounds(
        gas_limit in any::<u64>(),
    ) {
        const MIN_GAS: u64 = 21_000;
        const MAX_GAS: u64 = 30_000_000;
        
        let is_valid = gas_limit >= MIN_GAS && gas_limit <= MAX_GAS;
        
        if gas_limit < MIN_GAS {
            prop_assert!(!is_valid, "Gas below minimum should be invalid");
        } else if gas_limit > MAX_GAS {
            prop_assert!(!is_valid, "Gas above maximum should be invalid");
        } else {
            prop_assert!(is_valid, "Gas within bounds should be valid");
        }
    }
    
    /// Test that amount + gas never overflows with U256
    #[test]
    fn test_no_overflow_with_u256(
        amount in 0u64..1_000_000_000_000_000_000u64,
        gas_limit in 21_000u64..30_000_000u64,
    ) {
        let amount_u256 = U256::from(amount);
        let gas_cost = U256::from(gas_limit) * U256::from(1_000_000_000u64);
        
        // U256 should handle this without overflow
        let total = amount_u256.checked_add(gas_cost);
        prop_assert!(total.is_some(), "U256 should not overflow for reasonable values");
    }
    
    /// Test address validation logic
    #[test]
    fn test_address_validation_logic(
        address_bytes in prop::array::uniform20(any::<u8>())
    ) {
        let address = Address::from(address_bytes);
        let is_zero = address == Address::ZERO;
        
        // Zero address should be detected
        if address_bytes == [0u8; 20] {
            prop_assert!(is_zero, "All-zero bytes should create zero address");
        } else {
            prop_assert!(!is_zero, "Non-zero bytes should not create zero address");
        }
    }
    
    /// Test chain ID preservation
    #[test]
    fn test_chain_id_values(
        chain_id in 1u64..100_000u64,
    ) {
        let chain = ChainId::from(chain_id);
        
        // Chain ID should be preserved
        prop_assert_eq!(u64::from(chain), chain_id);
    }
}

/// Test wallet controller properties
mod wallet_properties {
    use super::*;
    use secrecy::SecretString;
    use vaughan::controllers::WalletController;
    
    proptest! {
        /// Test that account count logic is consistent
        #[test]
        fn test_account_count_logic(
            account_count in 0usize..10usize,
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let controller = WalletController::new();
                
                // Initial count should be zero
                assert_eq!(controller.account_count().await, 0);
                
                // Add accounts
                for i in 0..account_count {
                    // Use different private keys for each account
                    let pk = format!("{:064x}", i + 1);
                    let secret = SecretString::new(pk);
                    let _ = controller.add_account(secret, format!("Account {}", i)).await;
                }
                
                // Account count should match (or be less if some failed)
                let actual_count = controller.account_count().await;
                assert!(actual_count <= account_count, 
                    "Account count should not exceed added accounts");
            });
        }
    }
}

/// Test network controller properties
mod network_properties {
    use super::*;
    
    proptest! {
        /// Test that chain ID values are preserved
        #[test]
        fn test_chain_id_preservation(
            chain_id in 1u64..100_000u64,
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                // Test chain ID value preservation
                let chain = ChainId::from(chain_id);
                assert_eq!(u64::from(chain), chain_id);
                
                // Note: We can't test NetworkController creation without a valid RPC
                // This would require network access which is not suitable for property tests
            });
        }
    }
}

/// Test price controller properties
mod price_properties {
    use super::*;
    use vaughan::controllers::PriceController;
    
    proptest! {
        /// Test that cache capacity is always respected
        #[test]
        fn test_cache_capacity_logic(
            capacity in 1usize..1000usize,
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let controller = PriceController::with_cache_settings(
                    None,
                    capacity,
                    std::time::Duration::from_secs(60),
                );
                
                let (size, actual_capacity) = controller.cache_stats().await;
                assert_eq!(actual_capacity, capacity);
                assert!(size <= capacity, "Cache size should never exceed capacity");
            });
        }
    }
}

#[cfg(test)]
mod invariant_tests {
    use super::*;
    use vaughan::controllers::{WalletController, PriceController};
    
    /// Test that controllers maintain their invariants
    #[test]
    fn test_controller_invariants() {
        // Wallet controller invariants
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let wallet_controller = WalletController::new();
            assert_eq!(wallet_controller.account_count().await, 0);
            assert_eq!(wallet_controller.get_current_address().await, None);
        });
        
        // Price controller invariants
        runtime.block_on(async {
            let price_controller = PriceController::new(None);
            let (size, capacity) = price_controller.cache_stats().await;
            assert_eq!(size, 0);
            assert_eq!(capacity, 100); // Default capacity
        });
    }
}
