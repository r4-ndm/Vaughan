//! Token Management Module
//!
//! This module handles token lists, metadata, and price information
//! for ERC20 and native tokens across multiple networks.

use crate::error::Result;
use crate::error::TokenError;
use crate::network::NetworkId;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub mod lists;
pub mod pricing;

/// Token metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub chain_id: u64,
    pub logo_uri: Option<String>,
    pub tags: Vec<String>,
    pub is_native: bool,
}

impl TokenInfo {
    pub fn new(address: Address, chain_id: u64, name: String, symbol: String, decimals: u8) -> Self {
        Self {
            address,
            name,
            symbol,
            decimals,
            chain_id,
            logo_uri: None,
            tags: Vec::new(),
            is_native: address == Address::ZERO,
        }
    }

    /// Check if this is a stablecoin
    pub fn is_stablecoin(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag.to_lowercase().contains("stablecoin") || tag.to_lowercase().contains("stable"))
            || [
                "USDT", "USDC", "DAI", "BUSD", "TUSD", "USDP", "USDD", "FRAX", "LUSD", "sUSD", "FDUSD",
            ]
            .iter()
            .any(|stable| stable.to_lowercase() == self.symbol.to_lowercase())
    }

    /// Check if this is an LP token
    pub fn is_lp_token(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag.to_lowercase().contains("lp") || tag.to_lowercase().contains("liquidity"))
            || self.name.to_lowercase().contains("lp token")
            || self.name.to_lowercase().contains("liquidity pool")
    }

    /// Check if this is a bridged token
    pub fn is_bridged(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag.to_lowercase().contains("bridged") || tag.to_lowercase().contains("multichain"))
            || self.symbol.ends_with(".e") // Arbitrum bridged
            || self.symbol.starts_with("h") // Harmony bridged
            || self.name.to_lowercase().contains("bridged")
    }

    /// Get formatted price string
    pub fn format_price(&self, price: f64) -> String {
        if price < 0.000001 {
            format!("${:.8}", price)
        } else if price < 0.01 {
            format!("${:.6}", price)
        } else if price < 1.0 {
            format!("${:.4}", price)
        } else {
            format!("${:.2}", price)
        }
    }

    /// Get display name with symbol
    pub fn display_name(&self) -> String {
        if self.is_native {
            self.symbol.clone()
        } else {
            format!("{} ({})", self.name, self.symbol)
        }
    }

    /// Add logo URI
    pub fn with_logo(mut self, logo_uri: String) -> Self {
        self.logo_uri = Some(logo_uri);
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Token filter criteria
#[derive(Debug, Clone)]
pub struct TokenFilter {
    pub chain_id: Option<u64>,
    pub search_query: Option<String>,
    pub include_native: bool,
    pub include_stable: bool,
    pub include_lp: bool,
    pub include_verified_only: bool,
    pub tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub min_liquidity: Option<f64>,
    pub max_results: Option<usize>,
}

impl Default for TokenFilter {
    fn default() -> Self {
        Self {
            chain_id: None,
            search_query: None,
            include_native: true,
            include_stable: true,
            include_lp: false,
            include_verified_only: false,
            tags: Vec::new(),
            exclude_tags: Vec::new(),
            min_liquidity: None,
            max_results: Some(100),
        }
    }
}

impl TokenFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    pub fn search(mut self, query: String) -> Self {
        self.search_query = Some(query);
        self
    }

    pub fn include_stable(mut self, include: bool) -> Self {
        self.include_stable = include;
        self
    }

    pub fn include_lp(mut self, include: bool) -> Self {
        self.include_lp = include;
        self
    }

    pub fn verified_only(mut self, verified_only: bool) -> Self {
        self.include_verified_only = verified_only;
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn exclude_tag(mut self, tag: String) -> Self {
        self.exclude_tags.push(tag);
        self
    }

    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Check if a token matches the filter criteria
    pub fn matches(&self, token: &TokenInfo) -> bool {
        // Chain filter
        if let Some(chain_id) = self.chain_id {
            if token.chain_id != chain_id {
                return false;
            }
        }

        // Native token filter
        if token.is_native && !self.include_native {
            return false;
        }

        // Stablecoin filter
        if token.is_stablecoin() && !self.include_stable {
            return false;
        }

        // LP token filter
        if token.is_lp_token() && !self.include_lp {
            return false;
        }

        // Verified only filter
        if self.include_verified_only && !token.tags.contains(&"verified".to_string()) {
            return false;
        }

        // Required tags
        if !self.tags.is_empty() {
            let has_all_tags = self.tags.iter().all(|required_tag| {
                token
                    .tags
                    .iter()
                    .any(|token_tag| token_tag.to_lowercase() == required_tag.to_lowercase())
            });
            if !has_all_tags {
                return false;
            }
        }

        // Excluded tags
        if !self.exclude_tags.is_empty() {
            let has_excluded_tags = self.exclude_tags.iter().any(|excluded_tag| {
                token
                    .tags
                    .iter()
                    .any(|token_tag| token_tag.to_lowercase() == excluded_tag.to_lowercase())
            });
            if has_excluded_tags {
                return false;
            }
        }

        // Search query filter
        if let Some(query) = &self.search_query {
            let query_lower = query.to_lowercase();
            let matches_search =
                token.name.to_lowercase().contains(&query_lower) || token.symbol.to_lowercase().contains(&query_lower);
            if !matches_search {
                return false;
            }
        }

        true
    }

    /// Sort tokens by relevance (native first, then verified, then search relevance)
    pub fn sort_by_relevance(&self, tokens: &mut Vec<TokenInfo>) {
        tokens.sort_by(|a, b| {
            // Native tokens first
            match (a.is_native, b.is_native) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            // Verified tokens next
            let a_verified = a.tags.contains(&"verified".to_string());
            let b_verified = b.tags.contains(&"verified".to_string());
            match (a_verified, b_verified) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            // Search relevance
            if let Some(query) = &self.search_query {
                let query_lower = query.to_lowercase();

                let a_exact_name = a.name.to_lowercase() == query_lower;
                let b_exact_name = b.name.to_lowercase() == query_lower;
                match (a_exact_name, b_exact_name) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                let a_exact_symbol = a.symbol.to_lowercase() == query_lower;
                let b_exact_symbol = b.symbol.to_lowercase() == query_lower;
                match (a_exact_symbol, b_exact_symbol) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                // Alphabetical order
                return a.name.cmp(&b.name);
            }

            // Default alphabetical order
            a.name.cmp(&b.name)
        });
    }
}

/// Token balance with USD value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token: TokenInfo,
    pub balance: String,        // Raw balance as string to preserve precision
    pub formatted: String,      // Human-readable balance (e.g., "1.234567")
    pub usd_value: Option<f64>, // USD value if price is available
}

/// Token price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub token_address: Address,
    pub chain_id: u64,
    pub price_usd: f64,
    pub price_change_24h: Option<f64>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Token list metadata following Uniswap token list standard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenList {
    pub name: String,
    pub version: TokenListVersion,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tokens: Vec<TokenInfo>,
    pub keywords: Vec<String>,
    pub tags: HashMap<String, TokenListTag>,
    pub logo_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListTag {
    pub name: String,
    pub description: String,
}

/// Main token manager for handling all token operations
#[derive(Debug)]
pub struct TokenManager {
    /// Cached token lists by network
    token_lists: HashMap<NetworkId, Vec<TokenInfo>>,
    /// Cached prices by (chain_id, token_address)
    token_prices: HashMap<(u64, Address), TokenPrice>,
    /// HTTP client for API calls
    client: reqwest::Client,
    /// Custom tokens added by user
    custom_tokens: HashMap<NetworkId, Vec<TokenInfo>>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new() -> Self {
        Self {
            token_lists: HashMap::new(),
            token_prices: HashMap::new(),
            client: reqwest::Client::new(),
            custom_tokens: HashMap::new(),
        }
    }

    /// Get token list for a specific network
    pub fn get_tokens_for_network(&self, network_id: NetworkId) -> Vec<TokenInfo> {
        let mut tokens = self.token_lists.get(&network_id).cloned().unwrap_or_default();

        // Add custom tokens
        if let Some(custom_tokens) = self.custom_tokens.get(&network_id) {
            tokens.extend_from_slice(custom_tokens);
        }

        tokens
    }

    /// Search tokens by name or symbol
    pub async fn search_tokens(&self, network_id: NetworkId, query: String) -> Vec<TokenInfo> {
        let tokens = self.get_tokens_for_network(network_id);
        let query_lower = query.to_lowercase();

        tokens
            .into_iter()
            .filter(|token| {
                token.name.to_lowercase().contains(&query_lower) || token.symbol.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Filter tokens using custom filter criteria
    pub async fn filter_tokens(&self, network_id: NetworkId, filter: TokenFilter) -> Vec<TokenInfo> {
        let tokens = self.get_tokens_for_network(network_id);

        let mut filtered: Vec<TokenInfo> = tokens.into_iter().filter(|token| filter.matches(token)).collect();

        // Sort by relevance
        filter.sort_by_relevance(&mut filtered);

        // Limit results
        if let Some(max_results) = filter.max_results {
            filtered.truncate(max_results);
        }

        filtered
    }

    /// Get trending tokens (high volume tokens)
    pub async fn get_trending_tokens(&self, network_id: NetworkId, limit: usize) -> Vec<TokenInfo> {
        let filter = TokenFilter::new().with_tag("trending".to_string()).max_results(limit);

        self.filter_tokens(network_id, filter).await
    }

    /// Get popular tokens (verified tokens with good liquidity)
    pub async fn get_popular_tokens(&self, network_id: NetworkId, limit: usize) -> Vec<TokenInfo> {
        let tokens = self.get_tokens_for_network(network_id);

        let mut popular: Vec<TokenInfo> = tokens
            .into_iter()
            .filter(|token| {
                token.is_native
                    || token.tags.contains(&"verified".to_string())
                    || token.tags.contains(&"popular".to_string())
            })
            .collect();

        // Sort: native first, then verified, then popular
        popular.sort_by(|a, b| match (a.is_native, b.is_native) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_verified = a.tags.contains(&"verified".to_string());
                let b_verified = b.tags.contains(&"verified".to_string());
                match (a_verified, b_verified) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            }
        });

        popular.truncate(limit);
        popular
    }

    /// Get stablecoins for a network
    pub async fn get_stablecoins(&self, network_id: NetworkId) -> Vec<TokenInfo> {
        let tokens = self.get_tokens_for_network(network_id);

        tokens.into_iter().filter(|token| token.is_stablecoin()).collect()
    }

    /// Get LP tokens for a network
    pub async fn get_lp_tokens(&self, network_id: NetworkId) -> Vec<TokenInfo> {
        let tokens = self.get_tokens_for_network(network_id);

        tokens.into_iter().filter(|token| token.is_lp_token()).collect()
    }

    /// Get tokens by tag
    pub async fn get_tokens_by_tag(&self, network_id: NetworkId, tag: String) -> Vec<TokenInfo> {
        let filter = TokenFilter::new().with_tag(tag);
        self.filter_tokens(network_id, filter).await
    }

    /// Get all supported network IDs
    pub async fn get_supported_networks(&self) -> Vec<NetworkId> {
        let mut networks: Vec<NetworkId> = self.token_lists.keys().copied().collect();
        // Sort by chain ID (since NetworkId doesn't implement Ord)
        networks.sort_by_key(|n| n.0);
        networks
    }

    /// Check if token is verified
    pub async fn is_token_verified(&self, network_id: NetworkId, address: Address) -> bool {
        if let Some(token) = self.get_token_info(network_id, address) {
            token.tags.contains(&"verified".to_string())
        } else {
            false
        }
    }

    /// Get token with price information
    pub async fn get_token_with_price(
        &self,
        network_id: NetworkId,
        address: Address,
    ) -> Option<(TokenInfo, Option<TokenPrice>)> {
        let chain_id = network_id.0; // Access the inner u64

        if let Some(token) = self.get_token_info(network_id, address) {
            let price = self.get_token_price(chain_id, address).cloned();
            Some((token.clone(), price))
        } else {
            None
        }
    }

    /// Get token info by address and network
    pub fn get_token_info(&self, network_id: NetworkId, address: Address) -> Option<&TokenInfo> {
        // Check standard tokens
        if let Some(tokens) = self.token_lists.get(&network_id) {
            if let Some(token) = tokens.iter().find(|t| t.address == address) {
                return Some(token);
            }
        }

        // Check custom tokens
        if let Some(custom_tokens) = self.custom_tokens.get(&network_id) {
            custom_tokens.iter().find(|t| t.address == address)
        } else {
            None
        }
    }

    /// Get token price
    pub fn get_token_price(&self, chain_id: u64, address: Address) -> Option<&TokenPrice> {
        self.token_prices.get(&(chain_id, address))
    }

    /// Add custom token with automatic metadata discovery
    pub async fn add_custom_token_with_metadata(
        &mut self,
        network_id: NetworkId,
        address: Address,
    ) -> Result<TokenInfo> {
        tracing::info!("Adding custom token: {:?} on network: {:?}", address, network_id);

        // First check if token already exists
        if self.get_token_info(network_id, address).is_some() {
            return self
                .get_token_info(network_id, address)
                .cloned()
                .ok_or(crate::error::VaughanError::Token(TokenError::NotFound(address)));
        }

        // Create basic token info for now (metadata discovery can be added later)
        let token_info = TokenInfo::new(
            address,
            network_id.0,
            "Custom Token".to_string(),
            "CUSTOM".to_string(),
            18, // Default to 18 decimals
        );

        // Add to custom tokens
        self.add_custom_token(network_id, token_info.clone());

        Ok(token_info)
    }

    /// Add a custom token
    pub fn add_custom_token(&mut self, network_id: NetworkId, token: TokenInfo) {
        tracing::info!("Adding custom token {} on {:?}", token.symbol, network_id);
        self.custom_tokens.entry(network_id).or_default().push(token);
    }

    /// Load token list from URL
    pub async fn load_token_list_from_url(&mut self, url: &str) -> Result<TokenList> {
        tracing::info!("Loading token list from: {}", url);

        let response = self
            .client
            .get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| crate::error::NetworkError::RpcError {
                message: format!("Failed to fetch token list: {e}"),
            })?;

        let token_list: TokenList = response
            .json()
            .await
            .map_err(|e| crate::error::NetworkError::RpcError {
                message: format!("Failed to parse token list JSON: {e}"),
            })?;

        tracing::info!("Loaded {} tokens from {}", token_list.tokens.len(), token_list.name);

        // Group tokens by chain ID
        for token in &token_list.tokens {
            let network_id = NetworkId(token.chain_id);
            self.token_lists.entry(network_id).or_default().push(token.clone());
        }

        Ok(token_list)
    }

    /// Load default token lists for all supported networks
    pub async fn load_default_token_lists(&mut self) -> Result<()> {
        let default_lists = vec![
            // Uniswap default list (multi-chain)
            "https://tokens.uniswap.org",
            // CoinGecko token list
            "https://tokens.coingecko.com/uniswap/all.json",
            // 1inch token list
            "https://wispy-bird-88a7.uniswap.workers.dev/?url=http://tokens.1inch.eth.link",
        ];

        for url in default_lists {
            match self.load_token_list_from_url(url).await {
                Ok(_) => {
                    tracing::info!("Successfully loaded token list from {}", url);
                }
                Err(e) => {
                    tracing::warn!("Failed to load token list from {}: {}", url, e);
                    // Continue with other lists even if one fails
                }
            }
        }

        // Add native tokens for each network
        self.add_native_tokens();

        Ok(())
    }

    /// Add native tokens for each supported network
    fn add_native_tokens(&mut self) {
        let native_tokens = vec![
            // Ethereum
            (
                NetworkId(1),
                TokenInfo {
                    address: Address::ZERO, // Native ETH uses zero address
                    name: "Ethereum".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                    chain_id: 1,
                    logo_uri: Some(
                        "https://tokens.uniswap.org/images/0x0000000000000000000000000000000000000000.png".to_string(),
                    ),
                    tags: vec!["native".to_string()],
                    is_native: true,
                },
            ),
            // PulseChain
            (
                NetworkId(369),
                TokenInfo {
                    address: Address::ZERO,
                    name: "Pulse".to_string(),
                    symbol: "PLS".to_string(),
                    decimals: 18,
                    chain_id: 369,
                    logo_uri: Some("https://scan.pulsechain.com/images/pls.png".to_string()),
                    tags: vec!["native".to_string()],
                    is_native: true,
                },
            ),
            // Binance Smart Chain
            (
                NetworkId(56),
                TokenInfo {
                    address: Address::ZERO,
                    name: "BNB".to_string(),
                    symbol: "BNB".to_string(),
                    decimals: 18,
                    chain_id: 56,
                    logo_uri: Some(
                        "https://tokens.uniswap.org/images/0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c.png".to_string(),
                    ),
                    tags: vec!["native".to_string()],
                    is_native: true,
                },
            ),
            // Polygon
            (
                NetworkId(137),
                TokenInfo {
                    address: Address::ZERO,
                    name: "Matic".to_string(),
                    symbol: "MATIC".to_string(),
                    decimals: 18,
                    chain_id: 137,
                    logo_uri: Some("https://wallet-asset.matic.network/img/tokens/matic.svg".to_string()),
                    tags: vec!["native".to_string()],
                    is_native: true,
                },
            ),
        ];

        for (network_id, token) in native_tokens {
            self.token_lists.entry(network_id).or_default().insert(0, token); // Insert native token at the beginning
        }
    }

    /// Clear all cached token lists and prices
    pub fn clear_cache(&mut self) {
        self.token_lists.clear();
        self.token_prices.clear();
    }

    /// Get total number of cached tokens
    pub fn get_total_token_count(&self) -> usize {
        self.token_lists.values().map(|list| list.len()).sum::<usize>()
            + self.custom_tokens.values().map(|list| list.len()).sum::<usize>()
    }

    /// Get Multicall3 address for network
    pub fn get_multicall_address(&self, chain_id: u64) -> Address {
        match chain_id {
            1 => Address::from_str("0x5BA1e109517A9Db676D3435833F2FB74ea86faB9").unwrap(), // Ethereum Mainnet
            56 => Address::from_str("0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5").unwrap(), // BSC
            137 => Address::from_str("0x910eFc8Ff6c998353354eE51D7942c27F5A8D1EE").unwrap(), // Polygon (corrected)
            369 => Address::from_str("0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5").unwrap(), // PulseChain (using BSC address)
            _ => Address::from_str("0x5BA1e109517A9Db676D3435833F2FB74ea86faB9").unwrap(),   // Default to Ethereum
        }
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}
