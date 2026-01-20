//! API Service Module
//!
//! This module handles external API calls for price data and token information.

use crate::blockchain::explorer_apis::ExplorerApiManager;
use crate::gui::wallet_messages::Message;
use crate::gui::wallet_types::TokenInfo;
use crate::network::NetworkId;
use iced::Command;

/// Create command to fetch ETH price from API
pub fn create_eth_price_command(api_manager: Option<ExplorerApiManager>, chain_id: u64) -> Command<Message> {
    if let Some(api_manager) = api_manager {
        // Determine chain based on current network
        let chain = match chain_id {
            1 => "eth",
            56 => "bsc",
            137 => "polygon",
            42161 => "arbitrum",
            10 => "optimism",
            _ => "eth", // Default to Ethereum
        }
        .to_string();

        Command::perform(
            async move { api_manager.get_eth_price(&chain).await },
            |result| match result {
                Ok(price_data) => Message::EthPriceFetched(Ok((price_data.usd_price, price_data.percent_change_24h))),
                Err(e) => Message::EthPriceFetched(Err(e.to_string())),
            },
        )
    } else {
        // No API manager available, return sample data
        Command::perform(async { (2500.0, Some("+2.5".to_string())) }, |data| {
            Message::EthPriceFetched(Ok(data))
        })
    }
}

/// Token information fetching (simplified version for now)
pub async fn fetch_token_info(token_address: String, network_id: NetworkId) -> Result<TokenInfo, String> {
    use alloy::primitives::{Address, U256};
    use alloy::providers::{Provider, ProviderBuilder};
    use alloy::rpc::types::TransactionRequest;
    use std::str::FromStr;

    // Parse the token address
    let address = Address::from_str(&token_address).map_err(|e| format!("Invalid token address: {}", e))?;

    // Get RPC URL for the network
    let rpc_url = match network_id.chain_id() {
        943 => "https://rpc.v4.testnet.pulsechain.com", // PulseChain Testnet
        369 => "https://rpc.pulsechain.com",            // PulseChain Mainnet
        1 => "https://eth.llamarpc.com",                // Ethereum
        _ => return Err("Unsupported network".to_string()),
    };

    // Create provider
    let url = rpc_url.parse().unwrap();
    let provider = ProviderBuilder::new().connect_http(url);

    // Helper function to call contract method
    let call_contract = |selector: &[u8]| {
        let call_data = selector.to_vec();
        TransactionRequest::default().to(address).input(call_data.into())
    };

    // Helper function to parse string result
    let parse_string = |result: &[u8]| -> String {
        if result.len() < 64 {
            return "Unknown".to_string();
        }
        // Skip offset (32 bytes) and length (32 bytes), then read string data
        let length = U256::from_be_slice(&result[32..64]).to::<usize>().min(100);
        if result.len() >= 64 + length {
            String::from_utf8_lossy(&result[64..64 + length]).to_string()
        } else {
            "Unknown".to_string()
        }
    };

    // Fetch token symbol
    let symbol_call = call_contract(&[0x95, 0xd8, 0x9b, 0x41]); // symbol()
    let symbol = match provider.call(symbol_call).await {
        Ok(result) => {
            let parsed = parse_string(&result);
            if parsed == "Unknown" || parsed.is_empty() {
                // Fallback: try to guess from address
                if token_address.to_lowercase().contains("usdc") {
                    "USDC".to_string()
                } else if token_address.to_lowercase().contains("usdt") {
                    "USDT".to_string()
                } else if token_address.to_lowercase().contains("pepe") {
                    "PEPE".to_string()
                } else {
                    "TOKEN".to_string()
                }
            } else {
                parsed
            }
        }
        Err(_) => "TOKEN".to_string(),
    };

    // Fetch token name
    let name_call = call_contract(&[0x06, 0xfd, 0xde, 0x03]); // name()
    let name = match provider.call(name_call).await {
        Ok(result) => {
            let parsed = parse_string(&result);
            if parsed == "Unknown" || parsed.is_empty() {
                format!("{} Token", symbol)
            } else {
                parsed
            }
        }
        Err(_) => format!("{} Token", symbol),
    };

    // Fetch token decimals
    let decimals_call = call_contract(&[0x31, 0x3c, 0xe5, 0x67]); // decimals()
    let decimals = match provider.call(decimals_call).await {
        Ok(result) => {
            if result.len() >= 32 {
                U256::from_be_slice(&result[result.len() - 32..]).to::<u8>()
            } else {
                18
            }
        }
        Err(_) => 18,
    };

    Ok(TokenInfo {
        address: token_address,
        name,
        symbol,
        decimals,
        balance: None,
    })
}
