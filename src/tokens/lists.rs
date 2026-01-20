//! Token List Management
//!
//! This module handles loading and caching token lists from external APIs
//! following the Uniswap token list standard.

use super::{TokenInfo, TokenManager};
use crate::error::Result;
use crate::network::NetworkId;
use alloy::primitives::Address;
use std::str::FromStr;

/// Well-known token list URLs for different chains
pub struct TokenListUrls;

impl TokenListUrls {
    /// Get default token list URLs for a network
    pub fn get_default_urls(network_id: NetworkId) -> Vec<&'static str> {
        match network_id.chain_id() {
            // Ethereum Mainnet
            1 => vec![
                "https://tokens.uniswap.org",
                "https://raw.githubusercontent.com/compound-finance/token-list/master/compound.tokenlist.json",
                "https://raw.githubusercontent.com/Uniswap/default-token-list/main/src/tokens/mainnet.json",
            ],
            // Polygon
            137 => vec![
                "https://unpkg.com/quickswap-default-token-list@1.2.28/build/quickswap-default.tokenlist.json",
                "https://raw.githubusercontent.com/sushiswap/default-token-list/master/tokens/polygon.json",
            ],
            // BSC
            56 => vec![
                "https://raw.githubusercontent.com/pancakeswap/token-list/main/lists/pancakeswap-extended.json",
                "https://raw.githubusercontent.com/sushiswap/default-token-list/master/tokens/bsc.json",
            ],
            // PulseChain (custom list)
            369 => vec![
                // Will need to find/create PulseChain token lists
                "https://tokens.app.pulse.eco/pulsechain.tokenlist.json", // Example - may not exist
            ],
            _ => vec![],
        }
    }

    /// Get comprehensive multi-chain token lists
    pub fn get_multichain_urls() -> Vec<&'static str> {
        vec![
            "https://tokens.uniswap.org",                    // Official Uniswap list
            "https://tokens.coingecko.com/uniswap/all.json", // CoinGecko comprehensive list
            "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/tokenlist.json", // Trust Wallet
        ]
    }
}

impl TokenManager {
    /// Load token lists specifically for a network
    pub async fn load_token_lists_for_network(&mut self, network_id: NetworkId) -> Result<usize> {
        let urls = TokenListUrls::get_default_urls(network_id);
        let mut loaded_count = 0;

        for url in urls {
            match self.load_token_list_from_url(url).await {
                Ok(token_list) => {
                    let network_tokens = token_list
                        .tokens
                        .iter()
                        .filter(|token| token.chain_id == network_id.chain_id())
                        .count();
                    loaded_count += network_tokens;
                    tracing::info!(
                        "Loaded {} tokens for chain {} from {}",
                        network_tokens,
                        network_id.chain_id(),
                        url
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load token list for chain {} from {}: {}",
                        network_id.chain_id(),
                        url,
                        e
                    );
                }
            }
        }

        // Always add the native token for this network
        self.ensure_native_token(network_id);

        Ok(loaded_count)
    }

    /// Ensure native token exists for a network
    fn ensure_native_token(&mut self, network_id: NetworkId) {
        let tokens = self.token_lists.entry(network_id).or_default();

        // Check if native token already exists
        if tokens.iter().any(|token| token.is_native) {
            return;
        }

        // Add native token based on network
        let native_token = match network_id.chain_id() {
            1 => TokenInfo {
                address: Address::ZERO,
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
                chain_id: 1,
                logo_uri: Some(
                    "https://ethereum.org/static/6b935ac0e6194247347855dc3d328e83/6ed5f/eth-diamond-black.webp"
                        .to_string(),
                ),
                tags: vec!["native".to_string()],
                is_native: true,
            },
            369 => TokenInfo {
                address: Address::ZERO,
                name: "Pulse".to_string(),
                symbol: "PLS".to_string(),
                decimals: 18,
                chain_id: 369,
                logo_uri: Some("https://scan.pulsechain.com/favicon-32x32.png".to_string()),
                tags: vec!["native".to_string()],
                is_native: true,
            },
            56 => TokenInfo {
                address: Address::ZERO,
                name: "BNB".to_string(),
                symbol: "BNB".to_string(),
                decimals: 18,
                chain_id: 56,
                logo_uri: Some("https://cryptologos.cc/logos/bnb-bnb-logo.png".to_string()),
                tags: vec!["native".to_string()],
                is_native: true,
            },
            137 => TokenInfo {
                address: Address::ZERO,
                name: "Polygon".to_string(),
                symbol: "MATIC".to_string(),
                decimals: 18,
                chain_id: 137,
                logo_uri: Some("https://wallet-asset.matic.network/img/tokens/matic.svg".to_string()),
                tags: vec!["native".to_string()],
                is_native: true,
            },
            _ => TokenInfo {
                address: Address::ZERO,
                name: "Native Token".to_string(),
                symbol: "NATIVE".to_string(),
                decimals: 18,
                chain_id: network_id.chain_id(),
                logo_uri: None,
                tags: vec!["native".to_string()],
                is_native: true,
            },
        };

        // Insert native token at the beginning
        tokens.insert(0, native_token);
    }

    /// Load popular tokens for a network (fallback if main lists fail)
    pub async fn load_popular_tokens_for_network(&mut self, network_id: NetworkId) -> Result<()> {
        let popular_tokens = match network_id.chain_id() {
            // Ethereum popular tokens
            1 => vec![
                ("USDC", "USD Coin", "0xA0b86a33E642D6C02F6085C9D7E245845C1C7627", 6),
                ("USDT", "Tether USD", "0xdAC17F958D2ee523a2206206994597C13D831ec7", 6),
                (
                    "WETH",
                    "Wrapped Ether",
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                    18,
                ),
                (
                    "DAI",
                    "Dai Stablecoin",
                    "0x6B175474E89094C44Da98b954EedeAC495271d0F",
                    18,
                ),
                ("UNI", "Uniswap", "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984", 18),
                ("LINK", "Chainlink", "0x514910771AF9Ca656af840dff83E8264EcF986CA", 18),
            ],
            // Polygon popular tokens
            137 => vec![
                ("USDC", "USD Coin", "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", 6),
                ("USDT", "Tether USD", "0xc2132D05D31c914a87C6611C10748AEb04B58e8F", 6),
                (
                    "WETH",
                    "Wrapped Ether",
                    "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619",
                    18,
                ),
                (
                    "WMATIC",
                    "Wrapped Matic",
                    "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270",
                    18,
                ),
                ("AAVE", "Aave", "0xD6DF932A45C0f255f85145f286eA0b292B21C90B", 18),
                ("QUICK", "Quickswap", "0x831753DD7087CaC61aB5644b308642cc1c33Dc13", 18),
            ],
            // BSC popular tokens
            56 => vec![
                ("USDT", "Tether USD", "0x55d398326f99059fF775485246999027B3197955", 18),
                ("USDC", "USD Coin", "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d", 18),
                ("BUSD", "Binance USD", "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56", 18),
                ("WBNB", "Wrapped BNB", "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c", 18),
                ("CAKE", "PancakeSwap", "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82", 18),
                ("ADA", "Cardano", "0x3EE2200Efb3400fAbB9AacF31297cBdD1d435D47", 18),
            ],
            // PulseChain popular tokens (examples - may need updating)
            369 => vec![
                ("WPLS", "Wrapped PLS", "0xA1077a294dDE1B09bB078844df40758a5D0f9a27", 18),
                ("PLSX", "PulseX", "0x95B303987A60C71504D99Aa1b13B4DA07b0790ab", 18),
                ("INC", "Incentive", "0x2fa878Ab3F87CC1C9737Fc071108F904c0B0C95d", 18),
                ("TONI", "Toni", "0x77327a6D70dB9C392aB86ACF1C493e2b8e3a5c6e", 18),
            ],
            // PulseChain Testnet v4 (943) popular tokens
            943 => vec![
                (
                    "USD",
                    "USD Test Token",
                    "0x3e0Ad60c6D427191D66B6D168ddeF82A66F573B0",
                    18,
                ), // Real testnet USD token
                ("WPLS", "Wrapped PLS", "0xcF1Fc503CA35618E9b4C08b7847980b3e10FB53B", 18), // Real testnet WPLS
            ],
            _ => vec![],
        };

        for (symbol, name, address_str, decimals) in popular_tokens {
            if let Ok(address) = Address::from_str(address_str) {
                let token = TokenInfo {
                    address,
                    name: name.to_string(),
                    symbol: symbol.to_string(),
                    decimals,
                    chain_id: network_id.chain_id(),
                    logo_uri: None,
                    tags: vec!["popular".to_string()],
                    is_native: false,
                };

                self.token_lists.entry(network_id).or_default().push(token);
            }
        }

        Ok(())
    }

    /// Get token count for a specific network
    pub fn get_token_count_for_network(&self, network_id: NetworkId) -> usize {
        self.token_lists
            .get(&network_id)
            .map(|tokens| tokens.len())
            .unwrap_or(0)
            + self
                .custom_tokens
                .get(&network_id)
                .map(|tokens| tokens.len())
                .unwrap_or(0)
    }

    /// Search tokens by symbol or name
    pub fn search_tokens_legacy(&self, network_id: NetworkId, query: &str) -> Vec<&TokenInfo> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        // Search in standard tokens
        if let Some(tokens) = self.token_lists.get(&network_id) {
            results.extend(tokens.iter().filter(|token| {
                token.symbol.to_lowercase().contains(&query_lower) || token.name.to_lowercase().contains(&query_lower)
            }));
        }

        // Search in custom tokens
        if let Some(custom_tokens) = self.custom_tokens.get(&network_id) {
            results.extend(custom_tokens.iter().filter(|token| {
                token.symbol.to_lowercase().contains(&query_lower) || token.name.to_lowercase().contains(&query_lower)
            }));
        }

        // Sort by exact matches first, then partial matches
        results.sort_by(|a, b| {
            let a_exact = a.symbol.to_lowercase() == query_lower || a.name.to_lowercase() == query_lower;
            let b_exact = b.symbol.to_lowercase() == query_lower || b.name.to_lowercase() == query_lower;

            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.symbol.cmp(&b.symbol),
            }
        });

        results
    }
}
