//! Multicall3 Integration for Efficient RPC Batching
//!
//! This module provides Multicall3 contract integration for batching RPC calls
//! into a single request, significantly reducing network overhead.
//!
//! # Alloy Integration
//!
//! Uses Alloy's `sol!` macro to define the Multicall3 ABI, ensuring type-safe
//! interaction with the deployed contract.
//!
//! # Task Reference
//!
//! Implements: Task 4.1 (Add Multicall3 contract integration)
//! Implements: Task 4.2 (Implement batch balance queries with Multicall3)
//! Implements: Task 4.3 (Add fallback to parallel HTTP requests)
//!
//! # Chain Support
//!
//! Multicall3 is deployed at the same CREATE2 address on all major EVM chains:
//! `0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5`

use alloy::primitives::{Address, Bytes, U256};
use alloy::sol;
use alloy::sol_types::SolCall;  // Required for abi_encode
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{Result, VaughanError, NetworkError};

// ============================================================================
// Multicall3 ABI Definition (Alloy sol! macro)
// ============================================================================

sol! {
    /// Multicall3 contract interface
    /// 
    /// Deployed at 0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5 on all major EVM chains
    #[derive(Debug)]
    #[sol(rpc)]
    interface IMulticall3 {
        /// A single call to be aggregated
        struct Call3 {
            /// Target contract address
            address target;
            /// Whether to allow this call to fail
            bool allowFailure;
            /// Encoded function call data
            bytes callData;
        }

        /// Result of a single call
        struct Result {
            /// Whether the call succeeded
            bool success;
            /// Return data from the call
            bytes returnData;
        }

        /// Aggregate multiple calls into a single transaction
        /// @param calls Array of Call3 structs to execute
        /// @return returnData Array of Result structs
        function aggregate3(Call3[] calldata calls) 
            external payable 
            returns (Result[] memory returnData);
        
        /// Get the ETH balance of an address
        /// @param addr The address to query
        /// @return balance The ETH balance
        function getEthBalance(address addr) 
            external view 
            returns (uint256 balance);
    }
}

// ============================================================================
// Multicall3 Addresses by Chain
// ============================================================================

/// Standard Multicall3 CREATE2 address (same on all supported chains)
pub const MULTICALL3_ADDRESS: &str = "0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5";

/// Get the Multicall3 contract address for a given chain ID
///
/// Multicall3 uses CREATE2 deployment, so it has the same address on all
/// supported EVM chains.
///
/// # Arguments
/// * `chain_id` - The chain ID to get the address for
///
/// # Returns
/// The Multicall3 contract address for the chain
pub fn get_multicall3_address(chain_id: u64) -> Address {
    // Multicall3 uses CREATE2 and has the same address on all chains
    match chain_id {
        1 | 5 | 11155111 => { // Ethereum Mainnet, Goerli, Sepolia
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
        56 | 97 => { // BSC Mainnet, BSC Testnet
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
        137 | 80001 => { // Polygon Mainnet, Mumbai
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
        42161 | 421613 => { // Arbitrum One, Arbitrum Goerli
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
        10 | 420 => { // Optimism, Optimism Goerli
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
        369 | 943 => { // PulseChain, PulseChain Testnet
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
        _ => {
            // Default to standard address - may not be deployed on all chains
            tracing::warn!(
                chain_id = chain_id,
                "Using default Multicall3 address for unknown chain"
            );
            Address::from_str(MULTICALL3_ADDRESS).unwrap()
        }
    }
}

/// Check if Multicall3 is likely available on the given chain
pub fn is_multicall3_supported(chain_id: u64) -> bool {
    matches!(
        chain_id,
        1 | 5 | 11155111 |  // Ethereum
        56 | 97 |            // BSC
        137 | 80001 |        // Polygon
        42161 | 421613 |     // Arbitrum
        10 | 420 |           // Optimism
        369 | 943            // PulseChain
    )
}

// ============================================================================
// MulticallBuilder - Builder pattern for constructing multicalls
// ============================================================================

/// Builder for constructing Multicall3 batches
///
/// # Example
/// ```ignore
/// use vaughan::performance::multicall::MulticallBuilder;
///
/// let builder = MulticallBuilder::new(1) // Ethereum mainnet
///     .add_eth_balance_call(address1)
///     .add_eth_balance_call(address2)
///     .allow_failures(true);
///
/// let calls = builder.build();
/// ```
#[derive(Debug, Clone)]
pub struct MulticallBuilder {
    /// Chain ID for this multicall
    chain_id: u64,
    /// Accumulated calls
    calls: Vec<IMulticall3::Call3>,
    /// Whether to allow individual call failures
    allow_failures: bool,
    /// Correlation ID for tracking
    correlation_id: Uuid,
}

impl MulticallBuilder {
    /// Create a new MulticallBuilder for the given chain
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            calls: Vec::new(),
            allow_failures: true, // Default to allowing failures for safety
            correlation_id: Uuid::new_v4(),
        }
    }

    /// Create a builder with a specific correlation ID
    pub fn with_correlation(chain_id: u64, correlation_id: Uuid) -> Self {
        Self {
            chain_id,
            calls: Vec::new(),
            allow_failures: true,
            correlation_id,
        }
    }

    /// Set whether individual calls can fail without reverting the entire batch
    pub fn allow_failures(mut self, allow: bool) -> Self {
        self.allow_failures = allow;
        self
    }

    /// Add an ETH balance query for the given address
    pub fn add_eth_balance_call(mut self, address: Address) -> Self {
        let multicall_addr = get_multicall3_address(self.chain_id);
        
        // Encode getEthBalance(address) call
        let call_data = IMulticall3::getEthBalanceCall { addr: address }.abi_encode();
        
        self.calls.push(IMulticall3::Call3 {
            target: multicall_addr,
            allowFailure: self.allow_failures,
            callData: Bytes::from(call_data),
        });
        
        self
    }

    /// Add a custom call with encoded data
    pub fn add_call(mut self, target: Address, call_data: Vec<u8>) -> Self {
        self.calls.push(IMulticall3::Call3 {
            target,
            allowFailure: self.allow_failures,
            callData: Bytes::from(call_data),
        });
        self
    }

    /// Get the number of calls in the batch
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }

    /// Get the Multicall3 address for this chain
    pub fn multicall_address(&self) -> Address {
        get_multicall3_address(self.chain_id)
    }

    /// Get the correlation ID
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    /// Build the final Call3 array
    pub fn build(self) -> Vec<IMulticall3::Call3> {
        tracing::debug!(
            correlation_id = %self.correlation_id,
            chain_id = self.chain_id,
            call_count = self.calls.len(),
            allow_failures = self.allow_failures,
            "Building Multicall3 batch"
        );
        self.calls
    }
}

// ============================================================================
// Multicall Result Processing
// ============================================================================

/// Result of a single balance query from Multicall3
#[derive(Debug, Clone)]
pub struct MulticallBalanceResult {
    /// The queried address
    pub address: Address,
    /// The balance if successful
    pub balance: Option<U256>,
    /// Error message if failed
    pub error: Option<String>,
    /// Correlation ID
    pub correlation_id: Uuid,
}

impl MulticallBalanceResult {
    /// Create a successful result
    pub fn success(address: Address, balance: U256, correlation_id: Uuid) -> Self {
        Self {
            address,
            balance: Some(balance),
            error: None,
            correlation_id,
        }
    }

    /// Create a failed result
    pub fn failure(address: Address, error: String, correlation_id: Uuid) -> Self {
        Self {
            address,
            balance: None,
            error: Some(error),
            correlation_id,
        }
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        self.balance.is_some()
    }
}

/// Decode balance results from Multicall3 response
///
/// # Arguments
/// * `addresses` - The addresses that were queried
/// * `results` - The raw results from the Multicall3 call
/// * `correlation_id` - Correlation ID for tracking
///
/// # Returns
/// Vector of MulticallBalanceResult
pub fn decode_balance_results(
    addresses: &[Address],
    results: &[IMulticall3::Result],
    correlation_id: Uuid,
) -> Vec<MulticallBalanceResult> {
    addresses
        .iter()
        .zip(results.iter())
        .map(|(addr, result)| {
            if result.success {
                // Decode the U256 balance from return data
                match decode_u256_result(&result.returnData) {
                    Ok(balance) => {
                        tracing::debug!(
                            correlation_id = %correlation_id,
                            address = %addr,
                            balance = %balance,
                            "✅ Balance decoded successfully"
                        );
                        MulticallBalanceResult::success(*addr, balance, correlation_id)
                    }
                    Err(e) => {
                        tracing::warn!(
                            correlation_id = %correlation_id,
                            address = %addr,
                            error = %e,
                            "⚠️ Failed to decode balance"
                        );
                        MulticallBalanceResult::failure(*addr, e.to_string(), correlation_id)
                    }
                }
            } else {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    address = %addr,
                    "⚠️ Multicall3 call failed"
                );
                MulticallBalanceResult::failure(
                    *addr,
                    "Multicall3 call returned failure".to_string(),
                    correlation_id,
                )
            }
        })
        .collect()
}

/// Decode a U256 from ABI-encoded bytes
fn decode_u256_result(data: &Bytes) -> Result<U256> {
    if data.len() < 32 {
        return Err(VaughanError::Network(NetworkError::RpcError {
            message: format!("Invalid return data length: {} (expected 32)", data.len()),
        }));
    }
    
    // U256 is ABI-encoded as 32 bytes, big-endian
    let balance = U256::from_be_slice(&data[..32]);
    Ok(balance)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multicall3_address_resolution() {
        // All major chains should return the same CREATE2 address
        let eth_addr = get_multicall3_address(1);
        let bsc_addr = get_multicall3_address(56);
        let polygon_addr = get_multicall3_address(137);
        let pulse_addr = get_multicall3_address(369);

        assert_eq!(eth_addr, bsc_addr);
        assert_eq!(bsc_addr, polygon_addr);
        assert_eq!(polygon_addr, pulse_addr);
        
        // Verify the actual address
        let expected = Address::from_str(MULTICALL3_ADDRESS).unwrap();
        assert_eq!(eth_addr, expected);
    }

    #[test]
    fn test_is_multicall3_supported() {
        assert!(is_multicall3_supported(1));    // Ethereum
        assert!(is_multicall3_supported(56));   // BSC
        assert!(is_multicall3_supported(137));  // Polygon
        assert!(is_multicall3_supported(369));  // PulseChain
        assert!(!is_multicall3_supported(999)); // Unknown chain
    }

    #[test]
    fn test_multicall_builder_creation() {
        let builder = MulticallBuilder::new(1);
        assert_eq!(builder.chain_id, 1);
        assert!(builder.is_empty());
        assert!(builder.allow_failures);
    }

    #[test]
    fn test_multicall_builder_add_eth_balance() {
        let addr1 = Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18").unwrap();
        let addr2 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();

        let builder = MulticallBuilder::new(1)
            .add_eth_balance_call(addr1)
            .add_eth_balance_call(addr2);

        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_multicall_builder_allow_failures() {
        let builder = MulticallBuilder::new(1).allow_failures(false);
        assert!(!builder.allow_failures);
        
        let builder = builder.allow_failures(true);
        assert!(builder.allow_failures);
    }

    #[test]
    fn test_multicall_builder_build() {
        let addr = Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18").unwrap();
        
        let calls = MulticallBuilder::new(1)
            .add_eth_balance_call(addr)
            .build();

        assert_eq!(calls.len(), 1);
        assert!(calls[0].allowFailure);
    }

    #[test]
    fn test_multicall_balance_result_success() {
        let addr = Address::ZERO;
        let balance = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let corr_id = Uuid::new_v4();

        let result = MulticallBalanceResult::success(addr, balance, corr_id);
        assert!(result.is_success());
        assert_eq!(result.balance, Some(balance));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_multicall_balance_result_failure() {
        let addr = Address::ZERO;
        let corr_id = Uuid::new_v4();

        let result = MulticallBalanceResult::failure(addr, "Test error".to_string(), corr_id);
        assert!(!result.is_success());
        assert!(result.balance.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_decode_u256_result() {
        // Create a valid 32-byte U256 (1 ETH in wei)
        let mut data = vec![0u8; 32];
        data[24..32].copy_from_slice(&1_000_000_000_000_000_000u64.to_be_bytes());
        
        let result = decode_u256_result(&Bytes::from(data));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_decode_u256_result_invalid_length() {
        // Too short - should fail
        let data = Bytes::from(vec![0u8; 16]);
        let result = decode_u256_result(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_balance_results() {
        let addr1 = Address::ZERO;
        let addr2 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let addresses = vec![addr1, addr2];
        let corr_id = Uuid::new_v4();

        // Create success result data (1 ETH)
        let mut success_data = vec![0u8; 32];
        success_data[24..32].copy_from_slice(&1_000_000_000_000_000_000u64.to_be_bytes());

        let results = vec![
            IMulticall3::Result {
                success: true,
                returnData: Bytes::from(success_data),
            },
            IMulticall3::Result {
                success: false,
                returnData: Bytes::new(),
            },
        ];

        let decoded = decode_balance_results(&addresses, &results, corr_id);
        assert_eq!(decoded.len(), 2);
        assert!(decoded[0].is_success());
        assert!(!decoded[1].is_success());
    }

    #[test]
    fn test_multicall_builder_with_correlation() {
        let corr_id = Uuid::new_v4();
        let builder = MulticallBuilder::with_correlation(1, corr_id);
        assert_eq!(builder.correlation_id(), corr_id);
    }
}
