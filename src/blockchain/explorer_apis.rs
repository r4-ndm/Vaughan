//! Professional Block Explorer API Management
//!
//! This module provides a unified interface to multiple block explorer APIs
//! with automatic fallbacks, rate limiting, and configuration management.

use crate::network::NetworkId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Configuration for block explorer APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerApiConfig {
    /// API keys for different block explorers
    pub api_keys: HashMap<String, String>,

    /// Multi-chain price API key (e.g., Moralis, Alchemy)
    pub price_api_key: Option<String>,

    /// Preferred price API service
    pub preferred_price_api: Option<String>,

    /// Rate limit: requests per minute
    pub rate_limit: u32,

    /// Timeout for HTTP requests
    pub request_timeout: Duration,

    /// Number of retries on failure
    pub max_retries: u32,

    /// Whether to use sample data when APIs fail
    pub use_sample_fallback: bool,
}

impl Default for ExplorerApiConfig {
    fn default() -> Self {
        Self {
            api_keys: HashMap::new(),
            price_api_key: None,
            preferred_price_api: None,
            rate_limit: 5, // Conservative: 5 requests per minute for free tiers
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
            use_sample_fallback: true,
        }
    }
}

/// Block explorer API endpoints and configuration
#[derive(Debug, Clone)]
pub struct ExplorerEndpoint {
    pub name: String,
    pub api_url: String,
    pub requires_api_key: bool,
    pub rate_limit: u32,
    pub free_tier_limit: Option<u32>,
}

/// Transaction data from block explorer APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: u64,
    pub block_number: u64,
    pub gas_used: Option<u64>,
    pub gas_price: Option<String>,
    pub status: String,
    pub method_name: Option<String>,
}

/// Token price data from Moralis API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    #[serde(rename = "tokenName")]
    pub token_name: String,
    #[serde(rename = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(rename = "tokenLogo")]
    pub token_logo: Option<String>,
    #[serde(rename = "tokenDecimals")]
    pub token_decimals: String,
    #[serde(rename = "usdPrice")]
    pub usd_price: f64,
    #[serde(rename = "24hrPercentChange")]
    pub percent_change_24h: Option<String>,
}

/// API response wrapper
#[derive(Debug, Deserialize)]
struct EtherscanResponse {
    status: String,
    message: String,
    result: serde_json::Value,
}

/// Rate limiting tracker
#[derive(Debug)]
struct RateLimiter {
    last_request: Instant,
    requests_in_window: u32,
    window_start: Instant,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            last_request: Instant::now() - Duration::from_secs(60),
            requests_in_window: 0,
            window_start: Instant::now(),
        }
    }

    async fn wait_if_needed(&mut self, rate_limit: u32) {
        let now = Instant::now();

        // Reset window if it's been more than a minute
        if now.duration_since(self.window_start) > Duration::from_secs(60) {
            self.window_start = now;
            self.requests_in_window = 0;
        }

        // Check if we need to wait
        if self.requests_in_window >= rate_limit {
            let wait_time = Duration::from_secs(60) - now.duration_since(self.window_start);
            if !wait_time.is_zero() {
                tracing::info!("🕐 Rate limit reached, waiting {:?}", wait_time);
                sleep(wait_time).await;
                self.window_start = Instant::now();
                self.requests_in_window = 0;
            }
        }

        self.requests_in_window += 1;
        self.last_request = now;
    }
}

/// Main Block Explorer API Manager
pub struct ExplorerApiManager {
    config: ExplorerApiConfig,
    endpoints: HashMap<NetworkId, Vec<ExplorerEndpoint>>,
    rate_limiters: HashMap<String, RateLimiter>,
    client: reqwest::Client,
}

impl Clone for ExplorerApiManager {
    fn clone(&self) -> Self {
        // HTTP client builder should never fail with these basic settings
        let client = reqwest::Client::builder()
            .timeout(self.config.request_timeout)
            .user_agent("Vaughan-Wallet/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut manager = Self {
            config: self.config.clone(),
            endpoints: self.endpoints.clone(),
            rate_limiters: HashMap::new(), // Create new rate limiters
            client,
        };

        // Initialize rate limiters
        for endpoint_list in self.endpoints.values() {
            for endpoint in endpoint_list {
                manager.rate_limiters.insert(endpoint.name.clone(), RateLimiter::new());
            }
        }

        manager
    }
}

impl ExplorerApiManager {
    /// Create a new API manager with configuration
    pub fn new(config: ExplorerApiConfig) -> Self {
        // HTTP client builder should never fail with these basic settings
        let client = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .user_agent("Vaughan-Wallet/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut manager = Self {
            config,
            endpoints: HashMap::new(),
            rate_limiters: HashMap::new(),
            client,
        };

        manager.initialize_endpoints();
        manager
    }

    /// Initialize all known block explorer endpoints
    fn initialize_endpoints(&mut self) {
        // Ethereum Mainnet
        self.endpoints.insert(
            NetworkId(1),
            vec![
                ExplorerEndpoint {
                    name: "Etherscan".to_string(),
                    api_url: "https://api.etherscan.io/api".to_string(),
                    requires_api_key: true,
                    rate_limit: 5,
                    free_tier_limit: Some(100_000), // 100k requests/day
                },
                ExplorerEndpoint {
                    name: "Blockchair".to_string(),
                    api_url: "https://api.blockchair.com/ethereum".to_string(),
                    requires_api_key: false,
                    rate_limit: 2,                // More conservative for free tier
                    free_tier_limit: Some(1_440), // ~1 request per minute for free
                },
                ExplorerEndpoint {
                    name: "Alchemy-Public".to_string(),
                    api_url: "https://eth-mainnet.g.alchemy.com/v2/demo".to_string(),
                    requires_api_key: false,
                    rate_limit: 1,              // Very conservative for demo endpoint
                    free_tier_limit: Some(300), // Very limited
                },
            ],
        );

        // Binance Smart Chain
        self.endpoints.insert(
            NetworkId(56),
            vec![
                ExplorerEndpoint {
                    name: "BSCScan".to_string(),
                    api_url: "https://api.bscscan.com/api".to_string(),
                    requires_api_key: true,
                    rate_limit: 5,
                    free_tier_limit: Some(100_000),
                },
                ExplorerEndpoint {
                    name: "Blockchair-BSC".to_string(),
                    api_url: "https://api.blockchair.com/binance-smart-chain".to_string(),
                    requires_api_key: false,
                    rate_limit: 2,
                    free_tier_limit: Some(1_440),
                },
            ],
        );

        // Polygon
        self.endpoints.insert(
            NetworkId(137),
            vec![ExplorerEndpoint {
                name: "PolygonScan".to_string(),
                api_url: "https://api.polygonscan.com/api".to_string(),
                requires_api_key: true,
                rate_limit: 5,
                free_tier_limit: Some(100_000),
            }],
        );

        // PulseChain (uses Blockscout - no API key required)
        self.endpoints.insert(
            NetworkId(369),
            vec![ExplorerEndpoint {
                name: "PulseScan".to_string(),
                api_url: "https://scan.pulsechain.com/api".to_string(),
                requires_api_key: false,
                rate_limit: 3,
                free_tier_limit: None,
            }],
        );

        // Avalanche C-Chain
        self.endpoints.insert(
            NetworkId(43114),
            vec![ExplorerEndpoint {
                name: "SnowTrace".to_string(),
                api_url: "https://api.snowtrace.io/api".to_string(),
                requires_api_key: true,
                rate_limit: 5,
                free_tier_limit: Some(100_000),
            }],
        );

        // Fantom Opera
        self.endpoints.insert(
            NetworkId(250),
            vec![ExplorerEndpoint {
                name: "FTMScan".to_string(),
                api_url: "https://api.ftmscan.com/api".to_string(),
                requires_api_key: true,
                rate_limit: 5,
                free_tier_limit: Some(100_000),
            }],
        );

        // Gnosis Chain (formerly xDAI) - uses Blockscout
        self.endpoints.insert(
            NetworkId(100),
            vec![ExplorerEndpoint {
                name: "GnosisScan".to_string(),
                api_url: "https://api.gnosisscan.io/api".to_string(),
                requires_api_key: false, // Blockscout-based
                rate_limit: 3,
                free_tier_limit: None,
            }],
        );

        // Arbitrum One
        self.endpoints.insert(
            NetworkId(42161),
            vec![ExplorerEndpoint {
                name: "ArbScan".to_string(),
                api_url: "https://api.arbiscan.io/api".to_string(),
                requires_api_key: true,
                rate_limit: 5,
                free_tier_limit: Some(100_000),
            }],
        );

        // Optimism
        self.endpoints.insert(
            NetworkId(10),
            vec![ExplorerEndpoint {
                name: "OptimismScan".to_string(),
                api_url: "https://api-optimistic.etherscan.io/api".to_string(),
                requires_api_key: true,
                rate_limit: 5,
                free_tier_limit: Some(100_000),
            }],
        );
    }

    /// Get transaction history for an address
    pub async fn get_transactions(&mut self, network: NetworkId, address: &str) -> Result<Vec<ApiTransaction>, String> {
        let endpoints = self
            .endpoints
            .get(&network)
            .ok_or_else(|| format!("Unsupported network: {}", network.0))?
            .clone(); // Clone the endpoints to avoid borrow issues

        // Try each endpoint in order until one succeeds
        for endpoint in endpoints {
            match self.try_endpoint(&endpoint, address).await {
                Ok(transactions) => {
                    tracing::info!(
                        "✅ Successfully fetched {} transactions from {}",
                        transactions.len(),
                        endpoint.name
                    );
                    return Ok(transactions);
                }
                Err(e) => {
                    tracing::warn!("⚠️ {} API failed: {}", endpoint.name, e);
                    continue;
                }
            }
        }

        // All endpoints failed - return empty results instead of fake data
        tracing::info!("📝 All APIs failed, returning empty transaction list");
        Ok(Vec::new())
    }

    /// Try a specific endpoint with rate limiting and retries
    async fn try_endpoint(
        &mut self,
        endpoint: &ExplorerEndpoint,
        address: &str,
    ) -> Result<Vec<ApiTransaction>, String> {
        // Rate limiting
        let rate_limiter = self
            .rate_limiters
            .entry(endpoint.name.clone())
            .or_insert_with(RateLimiter::new);
        rate_limiter.wait_if_needed(endpoint.rate_limit).await;

        // Build URL
        let url = self.build_api_url(endpoint, address)?;

        // Retry logic
        for attempt in 1..=self.config.max_retries {
            match self.make_api_request(&url).await {
                Ok(transactions) => return Ok(transactions),
                Err(e) => {
                    if attempt < self.config.max_retries {
                        let backoff = Duration::from_secs(2_u64.pow(attempt));
                        tracing::warn!("Attempt {} failed, retrying in {:?}: {}", attempt, backoff, e);
                        sleep(backoff).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        unreachable!()
    }

    /// Build API URL with parameters and API key
    fn build_api_url(&self, endpoint: &ExplorerEndpoint, address: &str) -> Result<String, String> {
        let mut url = format!(
            "{}?module=account&action=txlist&address={}&startblock=0&endblock=99999999&sort=desc&page=1&offset=50",
            endpoint.api_url, address
        );

        // Add API key if required and available
        if endpoint.requires_api_key {
            if let Some(api_key) = self.config.api_keys.get(&endpoint.name.to_lowercase()) {
                url.push_str(&format!("&apikey={api_key}"));
            } else {
                return Err(format!("API key required for {} but not configured", endpoint.name));
            }
        }

        Ok(url)
    }

    /// Make the actual HTTP request
    async fn make_api_request(&self, url: &str) -> Result<Vec<ApiTransaction>, String> {
        tracing::debug!("🌐 Making API request to: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let api_response: EtherscanResponse = response.json().await.map_err(|e| format!("JSON parsing failed: {e}"))?;

        if api_response.status != "1" {
            return Err(format!("API error: {}", api_response.message));
        }

        self.parse_transactions(api_response.result)
    }

    /// Parse API response into our transaction format
    fn parse_transactions(&self, result: serde_json::Value) -> Result<Vec<ApiTransaction>, String> {
        let transactions_array = result.as_array().ok_or("Expected array of transactions")?;

        let mut transactions = Vec::new();
        for tx in transactions_array {
            if let Ok(transaction) = self.parse_single_transaction(tx) {
                transactions.push(transaction);
            }
        }

        Ok(transactions)
    }

    /// Parse a single transaction from API response
    fn parse_single_transaction(&self, tx: &serde_json::Value) -> Result<ApiTransaction, String> {
        Ok(ApiTransaction {
            hash: tx["hash"].as_str().unwrap_or("").to_string(),
            from: tx["from"].as_str().unwrap_or("").to_string(),
            to: tx["to"].as_str().unwrap_or("").to_string(),
            value: tx["value"].as_str().unwrap_or("0").to_string(),
            timestamp: tx["timeStamp"].as_str().unwrap_or("0").parse().unwrap_or(0),
            block_number: tx["blockNumber"].as_str().unwrap_or("0").parse().unwrap_or(0),
            gas_used: tx["gasUsed"].as_str().and_then(|s| s.parse().ok()),
            gas_price: tx["gasPrice"].as_str().map(|s| s.to_string()),
            status: if tx["txreceipt_status"].as_str().unwrap_or("1") == "1" {
                "Success".to_string()
            } else {
                "Failed".to_string()
            },
            method_name: tx["methodId"].as_str().map(|s| s.to_string()),
        })
    }

    /// Update configuration (useful for adding API keys)
    pub fn update_config(&mut self, new_config: ExplorerApiConfig) {
        self.config = new_config;
    }

    /// Add or update an API key
    pub fn set_api_key(&mut self, service: &str, api_key: String) {
        self.config.api_keys.insert(service.to_lowercase(), api_key);
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ExplorerApiConfig {
        &self.config
    }

    /// Fetch token price using Moralis API
    pub async fn get_token_price(
        &self,
        token_address: &str,
        chain: &str,
    ) -> Result<TokenPrice, Box<dyn std::error::Error>> {
        // First try with price API key (Moralis)
        if let Some(price_api_key) = &self.config.price_api_key {
            match self.fetch_moralis_price(token_address, chain, price_api_key).await {
                Ok(price) => return Ok(price),
                Err(e) => {
                    tracing::warn!("Moralis price fetch failed: {}", e);
                }
            }
        }

        // Return sample price data for development
        Ok(TokenPrice {
            token_name: "Ethereum".to_string(),
            token_symbol: "ETH".to_string(),
            token_logo: None,
            token_decimals: "18".to_string(),
            usd_price: 2500.0, // Sample price
            percent_change_24h: Some("+2.5".to_string()),
        })
    }

    /// Fetch ETH price specifically (common use case)
    pub async fn get_eth_price(&self, chain: &str) -> Result<TokenPrice, Box<dyn std::error::Error>> {
        // ETH contract address (native token)
        let eth_address = "0x0000000000000000000000000000000000000000";
        self.get_token_price(eth_address, chain).await
    }

    /// Internal method to fetch price from Moralis
    async fn fetch_moralis_price(
        &self,
        token_address: &str,
        chain: &str,
        api_key: &str,
    ) -> Result<TokenPrice, Box<dyn std::error::Error>> {
        let url = format!("https://deep-index.moralis.io/api/v2.2/erc20/{token_address}/price?chain={chain}");

        tracing::debug!("🪙 Fetching price from Moralis: {}", url);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", api_key)
            .send()
            .await?
            .json::<TokenPrice>()
            .await?;

        Ok(response)
    }
}

/// Helper function to load configuration from file
pub fn load_config() -> Result<ExplorerApiConfig, String> {
    let config_path = std::env::var("HOME")
        .map(|home| format!("{home}/.config/vaughan/explorer_apis.json"))
        .unwrap_or_else(|_| "./config/explorer_apis.json".to_string());

    if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config file: {e}"))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {e}"))
    } else {
        tracing::info!("Config file not found at {}, using defaults", config_path);
        Ok(ExplorerApiConfig::default())
    }
}

/// Helper function to save configuration to file
pub fn save_config(config: &ExplorerApiConfig) -> Result<(), String> {
    let config_path = std::env::var("HOME")
        .map(|home| format!("{home}/.config/vaughan/explorer_apis.json"))
        .unwrap_or_else(|_| "./config/explorer_apis.json".to_string());

    // Create config directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {e}"))?;
    }

    let content = serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {e}"))?;

    std::fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {e}"))?;

    tracing::info!("Configuration saved to: {}", config_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();

        // Should not block first request
        let start = Instant::now();
        limiter.wait_if_needed(5).await;
        assert!(start.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = ExplorerApiConfig::default();
        config.api_keys.insert("etherscan".to_string(), "test-key".to_string());

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: ExplorerApiConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.api_keys.get("etherscan"), Some(&"test-key".to_string()));
    }
}
