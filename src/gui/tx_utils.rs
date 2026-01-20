use alloy::primitives::{Address, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};

/// Build a TransactionRequest for either Legacy or EIP-1559 given fee inputs in Gwei.
///
/// Parameters:
/// - tx_type: "Legacy" or "EIP-1559" (case-insensitive)
/// - gas_price_gwei: used only for Legacy (Some), ignored for EIP-1559
/// - max_fee_gwei: used only for EIP-1559 (Some), ignored for Legacy
/// - max_priority_fee_gwei: used only for EIP-1559 (Some), ignored for Legacy
#[allow(clippy::too_many_arguments)]
pub fn build_tx_request(
    from: Address,
    to: Address,
    chain_id: u64,
    gas_limit: u64,
    tx_type: &str,
    gas_price_gwei: Option<f64>,
    max_fee_gwei: Option<f64>,
    max_priority_fee_gwei: Option<f64>,
    nonce: Option<u64>,
    value: U256,
    input_bytes: Option<Vec<u8>>,
) -> TransactionRequest {
    let mut tx = TransactionRequest {
        from: Some(from),
        to: Some(to.into()),
        gas: Some(gas_limit),
        chain_id: Some(chain_id),
        value: Some(value),
        input: TransactionInput::from(input_bytes.unwrap_or_default()),
        nonce,
        ..Default::default()
    };

    if tx_type.eq_ignore_ascii_case("EIP-1559") {
        if let Some(max_fee) = max_fee_gwei {
            tx.max_fee_per_gas = Some((max_fee * 1e9) as u128);
        }
        if let Some(max_prio) = max_priority_fee_gwei {
            tx.max_priority_fee_per_gas = Some((max_prio * 1e9) as u128);
        }
    } else if let Some(gp) = gas_price_gwei {
        tx.gas_price = Some((gp * 1e9) as u128);
    }

    tx
}
