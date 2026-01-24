use crate::wallet::transaction::fees::{FeePriority, FeeEstimator};
use alloy::primitives::U256;

#[test]
fn test_priority_custom_values() {
    let max_fee = U256::from(100);
    let max_prio = U256::from(10);
    let custom = FeePriority::Custom {
        max_fee_per_gas: max_fee,
        max_priority_fee_per_gas: max_prio,
    };
    
    match custom {
        FeePriority::Custom { max_fee_per_gas, max_priority_fee_per_gas } => {
            assert_eq!(max_fee_per_gas, max_fee);
            assert_eq!(max_priority_fee_per_gas, max_prio);
        },
        _ => panic!("Expected Custom variant"),
    }
}

#[test]
fn test_fee_calculation_standard() {
    let base_fee = U256::from(100_000_000_000u64); // 100 Gwei
    let priority_fee = U256::from(2_000_000_000u64); // 2 Gwei
    
    let (max_fee, max_prio, seconds) = FeeEstimator::calculate_fee_parameters(
        base_fee, 
        priority_fee, 
        FeePriority::Standard
    );
    
    // Standard should be 1.0x
    assert_eq!(max_prio, priority_fee);
    // Max fee = base + prio
    // U256 implements Add
    assert_eq!(max_fee, base_fee + priority_fee);
    assert_eq!(seconds, 12);
}

#[test]
fn test_fee_calculation_fast() {
    let base_fee = U256::from(100);
    let priority_fee = U256::from(10);
    
    let (max_fee, max_prio, _) = FeeEstimator::calculate_fee_parameters(
        base_fee, 
        priority_fee, 
        FeePriority::Fast
    );
    
    // Fast priority: 1.2x = 12
    assert_eq!(max_prio, U256::from(12));
    
    // Fast base buffer: 1.1x = 110
    // Max fee = 110 + 12 = 122
    assert_eq!(max_fee, U256::from(122));
}

#[test]
fn test_fee_calculation_slow() {
    let base_fee = U256::from(100);
    let priority_fee = U256::from(10);
    
    let (max_fee, max_prio, _) = FeeEstimator::calculate_fee_parameters(
        base_fee, 
        priority_fee, 
        FeePriority::Slow
    );
    
    // Slow priority: 0.8x = 8
    assert_eq!(max_prio, U256::from(8));
    
    // Slow base: 1.0x = 100
    // Max fee = 100 + 8 = 108
    assert_eq!(max_fee, U256::from(108));
}

#[test]
fn test_fee_calculation_custom() {
    let custom_max = U256::from(500);
    let custom_prio = U256::from(50);
    
    let (max_fee, max_prio, _) = FeeEstimator::calculate_fee_parameters(
        U256::from(0), 
        U256::from(0), 
        FeePriority::Custom { max_fee_per_gas: custom_max, max_priority_fee_per_gas: custom_prio }
    );
    
    assert_eq!(max_fee, custom_max);
    assert_eq!(max_prio, custom_prio);
}
