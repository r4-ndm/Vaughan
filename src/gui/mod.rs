//! Modern GUI implementation using Iced 0.12
//!
//! This module provides the native cross-platform interface using Iced with
//! reactive architecture for the wallet application.

// State management modules
pub mod services;
pub mod state;

// Application components
pub mod command_helpers;
pub mod coordinators;
pub mod launcher;
pub mod safe_calculations;

use crate::network::{NetworkConfig, NetworkId};
use crate::security::keychain::OSKeychain;
use crate::security::keystore::SecureKeystoreImpl;

/// Account information for selection and management
#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub balance: String,
    pub is_connected: bool,
}

/// Transaction record for history display
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub hash: String,
    pub address: String,
    pub amount: String,
    pub direction: TransactionDirection,
    pub network: NetworkId,
    pub timestamp: u64,
    pub status: TransactionStatus,
}

/// Transaction direction enum
#[derive(Debug, Clone)]
pub enum TransactionDirection {
    Sent,
    Received,
}

/// Simple token balance for core wallet functionality
#[derive(Debug, Clone)]
pub struct SimpleTokenBalance {
    /// Token symbol (e.g., "ETH", "USDC")
    pub symbol: String,
    /// Token name (e.g., "Ethereum", "USD Coin")
    pub name: String,
    /// Contract address (None for native tokens)
    pub contract_address: Option<alloy::primitives::Address>,
    /// Balance in human-readable format
    pub balance: String,
    /// Token decimals
    pub decimals: u8,
}

/// Transaction form state
#[derive(Debug, Clone, Default)]
pub struct TransactionFormState {
    pub to_address: String,
    pub amount: String,
    pub gas_limit: String,
    pub gas_price: String,
    pub data: String,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    /// Selected token symbol for the transaction ("NATIVE" = native token)
    pub selected_token_symbol: String,
    /// Available tokens in wallet
    pub available_tokens: Vec<SimpleTokenBalance>,
    /// Custom token contract address (for manual entry)
    pub custom_token_address: String,
    /// Whether to show custom token input
    pub show_custom_token_input: bool,
    /// Advanced options
    pub tx_type: String, // "Legacy" or "EIP-1559"
    pub max_fee_gwei: String,          // for EIP-1559
    pub max_priority_fee_gwei: String, // for EIP-1559
    pub nonce_override: String,        // optional
    pub simulate_before_send: bool,    // eth_call simulation
    pub dry_run: bool,                 // sign only
    pub force_broadcast: bool,         // bypass chain id mismatch
}

/// Async result types for core wallet operations
#[derive(Debug, Clone)]
pub enum AsyncResult {
    BalanceFetched(String),
    TransactionSent(String),
    NetworksLoaded(Vec<NetworkConfig>),
    OperationComplete(String),
}

// Core wallet modules
pub mod api_service;
pub mod hd_wallet_service;
pub mod simple_transaction;
pub mod transaction_cancellation;
pub mod transaction_service;
pub mod wallet_messages;
pub mod working_wallet;

// UI components
pub mod components;
pub mod handlers;
pub mod views;

// Utilities and helpers
pub mod constants;
pub mod spinner;

pub mod theme;
pub mod transaction_errors;
pub mod tx_utils;
pub mod utils;
pub mod wallet_types;
pub mod widgets;

// Re-exports
pub use spinner::*;
pub use styles::*;
pub use theme::*;
pub use wallet_messages::Message;
pub use wallet_types::*;

// Utility functions

/// Validate transaction amount
pub fn validate_amount(amount: &str) -> bool {
    amount.parse::<f64>().map(|a| a > 0.0).unwrap_or(false)
}

/// Get network name from NetworkId
pub fn get_network_name(network_id: NetworkId) -> &'static str {
    match network_id.0 {
        1 => "Ethereum",
        56 => "BSC",
        137 => "Polygon",
        369 => "PulseChain",
        943 => "PulseChain Testnet v4",
        43114 => "Avalanche",
        250 => "Fantom",
        42161 => "Arbitrum",
        10 => "Optimism",
        _ => "Unknown",
    }
}

/// Get network currency symbol from NetworkId
pub fn get_network_currency(network_id: NetworkId) -> &'static str {
    match network_id.0 {
        1 => "ETH",
        56 => "BNB",
        137 => "MATIC",
        369 => "PLS",
        943 => "tPLS",
        43114 => "AVAX",
        250 => "FTM",
        42161 => "ETH",
        10 => "ETH",
        _ => "TOKEN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_validation() {
        assert!(validate_amount("1.0"));
        assert!(validate_amount("0.001"));
        assert!(!validate_amount("0"));
        assert!(!validate_amount("-1.0"));
        assert!(!validate_amount("invalid"));
    }
}

/// Load all available networks for the wallet
pub async fn load_all_networks() -> Vec<NetworkConfig> {
    tracing::info!("Loading all networks including custom ones...");

    // Start with default networks
    let mut networks = vec![
        NetworkConfig {
            id: NetworkId(1),
            name: "Ethereum".to_string(),
            chain_id: 1,
            rpc_url: "https://ethereum.publicnode.com".to_string(),
            symbol: "ETH".to_string(),
            block_explorer_url: "https://etherscan.io".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(369),
            name: "PulseChain".to_string(),
            chain_id: 369,
            rpc_url: "https://rpc.pulsechain.com".to_string(),
            symbol: "PLS".to_string(),
            block_explorer_url: "https://scan.pulsechain.com".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(943),
            name: "PulseChain Testnet v4".to_string(),
            chain_id: 943,
            rpc_url: "https://rpc.v4.testnet.pulsechain.com".to_string(),
            symbol: "tPLS".to_string(),
            block_explorer_url: "https://scan.v4.testnet.pulsechain.com".to_string(),
            is_testnet: true,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(56),
            name: "BSC".to_string(),
            chain_id: 56,
            rpc_url: "https://bsc-dataseed1.binance.org".to_string(),
            symbol: "BNB".to_string(),
            block_explorer_url: "https://bscscan.com".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(137),
            name: "Polygon".to_string(),
            chain_id: 137,
            rpc_url: "https://polygon-rpc.com".to_string(),
            symbol: "MATIC".to_string(),
            block_explorer_url: "https://polygonscan.com".to_string(),
            is_testnet: false,
            is_custom: false,
        },
    ];

    // Load custom networks from keystore and deduplicate
    match OSKeychain::new("vaughan-wallet".to_string()) {
        Ok(keychain) => {
            match SecureKeystoreImpl::new(Box::new(keychain)).await {
                Ok(keystore) => {
                    // Get custom networks from keystore
                    let custom_networks = keystore.get_custom_networks();
                    for (_, network) in custom_networks.iter() {
                        // Only add custom networks that don't duplicate hardcoded ones
                        if !networks.iter().any(|n| n.chain_id == network.chain_id) {
                            tracing::info!(
                                "Loaded custom network: {} (Chain ID: {})",
                                network.name,
                                network.chain_id
                            );
                            networks.push(network.clone());
                        } else {
                            tracing::info!(
                                "Skipping duplicate custom network: {} (Chain ID: {}) - already have hardcoded version",
                                network.name,
                                network.chain_id
                            );
                        }
                    }
                    tracing::info!(
                        "Loaded {} custom networks from storage (after deduplication)",
                        custom_networks
                            .iter()
                            .filter(|(_, n)| !networks.iter().any(|hn| hn.chain_id == n.chain_id))
                            .count()
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to initialize keystore for loading networks: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to initialize keychain for loading networks: {}", e);
        }
    }

    networks
}
