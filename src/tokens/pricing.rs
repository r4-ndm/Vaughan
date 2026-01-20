//! Token Pricing Module
//!
//! This module handles fetching real-time token prices from external APIs
//! like CoinGecko, CoinMarketCap, and other price feeds.

use super::{TokenManager, TokenPrice};
use crate::error::Result;
use alloy::primitives::Address;
use serde::Deserialize;
use std::collections::HashMap;

/// CoinGecko API response for simple price endpoint
#[allow(dead_code)] // Constructed by serde::Deserialize, not manually
#[derive(Debug, Deserialize)]
struct CoinGeckoSimplePrice {
    #[serde(flatten)]
    prices: HashMap<String, CoinGeckoPriceData>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoPriceData {
    usd: f64,
    usd_24h_change: Option<f64>,
}

/// CoinGecko token list for mapping addresses to coin IDs
#[allow(dead_code)] // Constructed by serde::Deserialize, not manually
#[derive(Debug, Deserialize)]
struct CoinGeckoTokenList {
    tokens: HashMap<String, CoinGeckoToken>,
}

#[allow(dead_code)] // Constructed by serde::Deserialize, not manually
#[derive(Debug, Deserialize)]
struct CoinGeckoToken {
    id: String,
    symbol: String,
    name: String,
}

/// Price provider trait for different APIs
#[async_trait::async_trait]
pub trait PriceProvider {
    async fn get_token_prices(&self, chain_id: u64, token_addresses: &[Address]) -> Result<Vec<TokenPrice>>;

    async fn get_native_token_price(&self, chain_id: u64) -> Result<Option<TokenPrice>>;
}

/// CoinGecko price provider implementation
pub struct CoinGeckoPriceProvider {
    client: reqwest::Client,
    api_key: Option<String>,
    base_url: String,
}

impl CoinGeckoPriceProvider {
    pub fn new(api_key: Option<String>) -> Self {
        let base_url = if api_key.is_some() {
            "https://pro-api.coingecko.com/api/v3".to_string()
        } else {
            "https://api.coingecko.com/api/v3".to_string()
        };

        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }

    /// Get CoinGecko platform ID for a chain
    fn get_platform_id(chain_id: u64) -> Option<&'static str> {
        match chain_id {
            1 => Some("ethereum"),
            137 => Some("polygon-pos"),
            56 => Some("binance-smart-chain"),
            369 => Some("pulsechain"), // May not be supported yet
            _ => None,
        }
    }

    /// Get CoinGecko coin ID for native tokens
    fn get_native_coin_id(chain_id: u64) -> Option<&'static str> {
        match chain_id {
            1 => Some("ethereum"),
            137 => Some("matic-network"),
            56 => Some("binancecoin"),
            369 => Some("pulsechain"), // May not be supported yet
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl PriceProvider for CoinGeckoPriceProvider {
    async fn get_token_prices(&self, chain_id: u64, token_addresses: &[Address]) -> Result<Vec<TokenPrice>> {
        let platform_id = match Self::get_platform_id(chain_id) {
            Some(id) => id,
            None => {
                tracing::warn!("Unsupported chain ID for CoinGecko: {}", chain_id);
                return Ok(vec![]);
            }
        };

        if token_addresses.is_empty() {
            return Ok(vec![]);
        }

        // Convert addresses to comma-separated string
        let addresses_str = token_addresses
            .iter()
            .map(|addr| addr.to_string().to_lowercase())
            .collect::<Vec<_>>()
            .join(",");

        let mut url = format!(
            "{}/simple/token_price/{}?contract_addresses={}&vs_currencies=usd&include_24hr_change=true",
            self.base_url, platform_id, addresses_str
        );

        // Add API key if available
        if let Some(api_key) = &self.api_key {
            url.push_str(&format!("&x_cg_pro_api_key={api_key}"));
        }

        tracing::debug!("Fetching token prices from CoinGecko: {}", url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| crate::error::NetworkError::RpcError {
                message: format!("Failed to fetch token prices: {e}"),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::NetworkError::RpcError {
                message: format!("CoinGecko API error {status}: {error_text}"),
            }
            .into());
        }

        let price_data: HashMap<String, CoinGeckoPriceData> =
            response
                .json()
                .await
                .map_err(|e| crate::error::NetworkError::RpcError {
                    message: format!("Failed to parse price response: {e}"),
                })?;

        let now = chrono::Utc::now();
        let mut prices = Vec::new();

        for (address_str, price_info) in price_data {
            if let Ok(address) = address_str.parse::<Address>() {
                prices.push(TokenPrice {
                    token_address: address,
                    chain_id,
                    price_usd: price_info.usd,
                    price_change_24h: price_info.usd_24h_change,
                    last_updated: now,
                });
            }
        }

        tracing::info!("Fetched {} token prices for chain {}", prices.len(), chain_id);
        Ok(prices)
    }

    async fn get_native_token_price(&self, chain_id: u64) -> Result<Option<TokenPrice>> {
        let coin_id = match Self::get_native_coin_id(chain_id) {
            Some(id) => id,
            None => {
                tracing::warn!("Unsupported native token for chain ID: {}", chain_id);
                return Ok(None);
            }
        };

        let mut url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
            self.base_url, coin_id
        );

        if let Some(api_key) = &self.api_key {
            url.push_str(&format!("&x_cg_pro_api_key={api_key}"));
        }

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| crate::error::NetworkError::RpcError {
                message: format!("Failed to fetch native token price: {e}"),
            })?;

        let price_data: HashMap<String, CoinGeckoPriceData> =
            response
                .json()
                .await
                .map_err(|e| crate::error::NetworkError::RpcError {
                    message: format!("Failed to parse native price response: {e}"),
                })?;

        if let Some(price_info) = price_data.get(coin_id) {
            Ok(Some(TokenPrice {
                token_address: Address::ZERO, // Native tokens use zero address
                chain_id,
                price_usd: price_info.usd,
                price_change_24h: price_info.usd_24h_change,
                last_updated: chrono::Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }
}

/// Moralis price provider implementation
pub struct MoralisPriceProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl MoralisPriceProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://deep-index.moralis.io/api/v2".to_string(),
        }
    }

    /// Get Moralis chain identifier for a chain ID
    fn get_moralis_chain(chain_id: u64) -> Option<&'static str> {
        match chain_id {
            1 => Some("eth"),
            137 => Some("polygon"),
            56 => Some("bsc"),
            369 => Some("pulsechain"),
            943 => Some("pulsechain-testnet"),
            10001 => Some("ethw"),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MoralisTokenPriceResponse {
    #[serde(rename = "usdPrice")]
    usd_price: f64,
    #[serde(rename = "usdPriceFormatted")]
    #[allow(dead_code)] // Used by serde for deserialization
    usd_price_formatted: Option<String>,
    #[serde(rename = "24hrPercentChange")]
    percent_change_24h: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoralisNativePriceResponse {
    usd: f64,
    #[serde(rename = "usd24hChange")]
    usd_24h_change: Option<f64>,
}

#[async_trait::async_trait]
impl PriceProvider for MoralisPriceProvider {
    async fn get_token_prices(&self, chain_id: u64, token_addresses: &[Address]) -> Result<Vec<TokenPrice>> {
        let chain = match Self::get_moralis_chain(chain_id) {
            Some(chain) => chain,
            None => {
                tracing::warn!("Unsupported chain ID for Moralis: {}", chain_id);
                return Ok(vec![]);
            }
        };

        if token_addresses.is_empty() {
            return Ok(vec![]);
        }

        let mut prices = Vec::new();
        let now = chrono::Utc::now();

        // Moralis requires individual requests for each token
        for token_address in token_addresses {
            let url = format!("{}/erc20/{}/price?chain={}", self.base_url, token_address, chain);

            tracing::debug!("Fetching token price from Moralis: {}", url);

            match self
                .client
                .get(&url)
                .header("X-API-Key", &self.api_key)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<MoralisTokenPriceResponse>().await {
                            Ok(price_data) => {
                                // Parse 24h change if available
                                let price_change_24h =
                                    price_data.percent_change_24h.and_then(|s| s.parse::<f64>().ok());

                                prices.push(TokenPrice {
                                    token_address: *token_address,
                                    chain_id,
                                    price_usd: price_data.usd_price,
                                    price_change_24h,
                                    last_updated: now,
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse Moralis price response for {}: {}", token_address, e);
                            }
                        }
                    } else {
                        tracing::warn!("Moralis API error {} for token {}", response.status(), token_address);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch price from Moralis for {}: {}", token_address, e);
                }
            }

            // Rate limiting - Moralis allows 100 requests per minute
            tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        }

        tracing::info!(
            "Fetched {} token prices from Moralis for chain {}",
            prices.len(),
            chain_id
        );
        Ok(prices)
    }

    async fn get_native_token_price(&self, chain_id: u64) -> Result<Option<TokenPrice>> {
        let chain = match Self::get_moralis_chain(chain_id) {
            Some(chain) => chain,
            None => {
                tracing::warn!("Unsupported native token for chain ID: {}", chain_id);
                return Ok(None);
            }
        };

        let url = format!("{}/market-data/evm/{}/price", self.base_url, chain);

        match self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<MoralisNativePriceResponse>().await {
                        Ok(price_data) => {
                            Ok(Some(TokenPrice {
                                token_address: Address::ZERO, // Native tokens use zero address
                                chain_id,
                                price_usd: price_data.usd,
                                price_change_24h: price_data.usd_24h_change,
                                last_updated: chrono::Utc::now(),
                            }))
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse Moralis native price response: {}", e);
                            Ok(None)
                        }
                    }
                } else {
                    tracing::warn!("Moralis API error {} for native token", response.status());
                    Ok(None)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch native price from Moralis: {}", e);
                Ok(None)
            }
        }
    }
}

impl TokenManager {
    /// Update token prices for a specific network with hybrid providers
    pub async fn update_prices_for_network(&mut self, chain_id: u64) -> Result<usize> {
        self.update_prices_for_network_with_api_key(chain_id, None).await
    }

    /// Update token prices for a specific network with optional Moralis API key
    pub async fn update_prices_for_network_with_api_key(
        &mut self,
        chain_id: u64,
        moralis_api_key: Option<String>,
    ) -> Result<usize> {
        let mut updated_count = 0;

        // Try Moralis first if API key is available
        if let Some(api_key) = moralis_api_key {
            tracing::info!("Attempting to fetch prices from Moralis for chain {}", chain_id);
            let moralis = MoralisPriceProvider::new(api_key);

            // Get native token price from Moralis
            match moralis.get_native_token_price(chain_id).await {
                Ok(Some(native_price)) => {
                    self.token_prices.insert((chain_id, Address::ZERO), native_price);
                    updated_count += 1;
                    tracing::info!("✅ Updated native token price from Moralis for chain {}", chain_id);
                }
                Ok(None) => {
                    tracing::debug!("No native token price available from Moralis for chain {}", chain_id);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch native token price from Moralis: {}", e);
                }
            }

            // Get ERC20 token prices from Moralis
            let network_id = crate::network::NetworkId(chain_id);
            if let Some(tokens) = self.token_lists.get(&network_id) {
                let token_addresses: Vec<Address> = tokens
                    .iter()
                    .filter(|token| !token.is_native)
                    .map(|token| token.address)
                    .collect();

                if !token_addresses.is_empty() {
                    match moralis.get_token_prices(chain_id, &token_addresses).await {
                        Ok(prices) => {
                            for price in prices {
                                self.token_prices.insert((price.chain_id, price.token_address), price);
                                updated_count += 1;
                            }
                            tracing::info!(
                                "✅ Updated {} token prices from Moralis for chain {}",
                                token_addresses.len(),
                                chain_id
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Failed to fetch token prices from Moralis: {}", e);
                        }
                    }
                }
            }
        } else {
            tracing::debug!("No Moralis API key provided, skipping Moralis price fetch");
        }

        // Use CoinGecko as fallback for missing prices
        let coingecko = CoinGeckoPriceProvider::new(None);

        // Get native token price from CoinGecko if not already fetched
        if let std::collections::hash_map::Entry::Vacant(e) = self.token_prices.entry((chain_id, Address::ZERO)) {
            if let Ok(Some(native_price)) = coingecko.get_native_token_price(chain_id).await {
                e.insert(native_price);
                updated_count += 1;
                tracing::info!(
                    "📈 Updated native token price from CoinGecko fallback for chain {}",
                    chain_id
                );
            }
        }

        // Get ERC20 token prices from CoinGecko for tokens not fetched from Moralis
        let network_id = crate::network::NetworkId(chain_id);
        if let Some(tokens) = self.token_lists.get(&network_id) {
            // Filter out native tokens and tokens already fetched from Moralis
            let missing_token_addresses: Vec<Address> = tokens
                .iter()
                .filter(|token| !token.is_native)
                .filter(|token| !self.token_prices.contains_key(&(chain_id, token.address)))
                .map(|token| token.address)
                .collect();

            if !missing_token_addresses.is_empty() {
                tracing::info!(
                    "Fetching {} missing token prices from CoinGecko fallback for chain {}",
                    missing_token_addresses.len(),
                    chain_id
                );

                // Batch request for multiple tokens (CoinGecko allows up to 30 per request)
                const BATCH_SIZE: usize = 30;

                for chunk in missing_token_addresses.chunks(BATCH_SIZE) {
                    match coingecko.get_token_prices(chain_id, chunk).await {
                        Ok(prices) => {
                            let count = prices.len();
                            for price in prices {
                                self.token_prices.insert((price.chain_id, price.token_address), price);
                                updated_count += 1;
                            }
                            tracing::info!("📈 Updated {} token prices from CoinGecko fallback", count);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to fetch price batch from CoinGecko for chain {}: {}",
                                chain_id,
                                e
                            );
                        }
                    }

                    // Rate limiting - CoinGecko free tier allows 30 calls per minute
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                }
            } else {
                tracing::debug!("All token prices already fetched for chain {}", chain_id);
            }
        }

        tracing::info!("Updated {} token prices for chain {}", updated_count, chain_id);
        Ok(updated_count)
    }

    /// Update prices for all cached tokens
    pub async fn update_all_prices(&mut self) -> Result<usize> {
        self.update_all_prices_with_api_key(None).await
    }

    /// Update prices for all cached tokens with optional Moralis API key
    pub async fn update_all_prices_with_api_key(&mut self, moralis_api_key: Option<String>) -> Result<usize> {
        let mut total_updated = 0;

        // Get all unique chain IDs from token lists
        let chain_ids: Vec<u64> = self
            .token_lists
            .keys()
            .map(|network_id| network_id.chain_id())
            .collect();

        for chain_id in chain_ids {
            match self
                .update_prices_for_network_with_api_key(chain_id, moralis_api_key.clone())
                .await
            {
                Ok(count) => total_updated += count,
                Err(e) => tracing::warn!("Failed to update prices for chain {}: {}", chain_id, e),
            }
        }

        Ok(total_updated)
    }

    /// Clear old price data (older than specified duration)
    pub fn clear_old_prices(&mut self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;

        self.token_prices.retain(|_, price| price.last_updated > cutoff);

        tracing::debug!("Cleared old price data older than {:?}", max_age);
    }

    /// Get price cache statistics
    pub fn get_price_cache_stats(&self) -> (usize, Option<chrono::DateTime<chrono::Utc>>) {
        let count = self.token_prices.len();
        let oldest = self.token_prices.values().map(|price| price.last_updated).min();

        (count, oldest)
    }
}
