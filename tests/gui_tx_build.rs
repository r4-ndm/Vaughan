use alloy::primitives::{Address, U256};
use vaughan::gui::tx_utils::build_tx_request;

#[test]
fn test_build_legacy_with_nonce_override() {
    let from = Address::from([0xAA; 20]);
    let to = Address::from([0xBB; 20]);
    let chain_id = 1u64;
    let gas_limit = 21_000u64;
    let nonce = Some(42u64);
    let value = U256::from(12345u64);

    let tx = build_tx_request(
        from,
        to,
        chain_id,
        gas_limit,
        "Legacy",
        Some(20.0), // 20 gwei
        None,
        None,
        nonce,
        value,
        None,
    );

    assert_eq!(tx.chain_id, Some(chain_id));
    assert_eq!(tx.nonce, nonce);
    assert_eq!(tx.gas, Some(gas_limit));
    assert_eq!(tx.gas_price, Some(20_000_000_000u128));
    assert!(tx.max_fee_per_gas.is_none());
    assert!(tx.max_priority_fee_per_gas.is_none());
}

#[test]
fn test_build_eip1559_fees() {
    let from = Address::from([0x01; 20]);
    let to = Address::from([0x02; 20]);
    let chain_id = 1u64;
    let gas_limit = 50_000u64;
    let nonce = Some(7u64);
    let value = U256::from(0u64);

    let tx = build_tx_request(
        from,
        to,
        chain_id,
        gas_limit,
        "EIP-1559",
        None,
        Some(30.0), // max fee (gwei)
        Some(2.0),  // max priority fee (gwei)
        nonce,
        value,
        None,
    );

    assert_eq!(tx.chain_id, Some(chain_id));
    assert_eq!(tx.nonce, nonce);
    assert_eq!(tx.gas, Some(gas_limit));
    assert!(tx.gas_price.is_none());
    assert_eq!(tx.max_fee_per_gas, Some(30_000_000_000u128));
    assert_eq!(tx.max_priority_fee_per_gas, Some(2_000_000_000u128));
}
