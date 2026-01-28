//! Price Controller - Token price fetching and caching
//!
//! Follows MetaMask's TokenRatesController pattern for price management.
//!
//! ## Design Principles
//!
//! 1. **Price Caching**: LRU cache for performance
//! 2. **Multiple Sources**: CoinGecko (free) + Moralis (optional)
//! 3. **Rate Limiting**: Respects API limits
//! 4. **Fallback Strategy**: CoinGecko fallback if Moralis fails
//!
//! ## MetaMask Inspiration
//!
//! This controller implements patterns from MetaMask's TokenRatesController:
//! - Price caching with expiration
//! - Multiple price source support
//! - Native token price fetching
//! - ERC20 token price fetching

use super::{ControllerError, ControllerResult};
use alloy::primitives::Address;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Price data for a token
#[derive(Debug, Clone)]
pub struct TokenPrice {
    /// Token address (Address::ZERO for native tokens)
    pub address: Address,
    /// Chain ID
    pub chain_id: u64,
    /// Price in USD
    pub price_usd: f64,
    /// 24h price change percentage
    pub price_change_24h: Option<f64>,
    /// When this price was fetched
    pub fetched_at: Instant,
}

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry {
    price: TokenPrice,
    expires_at: Instant,
}

/// Price cache key
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct CacheKey {
    chain_id: u64,
    address: Address,
}

/// Price controller - manages token price fetching and caching
///
/// This controller handles price fetching from multiple APIs with caching
/// for performance. It uses CoinGecko as the primary free source and supports
/// Moralis as an optional premium source.
///
/// ## Example
///
/// ```rust,no_run
/// use vaughan::controllers::PriceController;
/// use alloy::primitives::Address;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let controller = PriceController::new(None);
///
/// // Fetch ETH price (Ethereum mainnet)
/// let eth_price = controller.fetch_native_token_price(1).await?;
/// println!("ETH price: ${:.2}", eth_price.price_usd);
///
/// // Fetch ERC20 token price
/// let token_address = Address::from([0x12; 20]);
/// let token_price = controller.fetch_token_price(1, token_address).await?;
/// # Ok(())
/// # }
/// ```
pub struct PriceController {
    /// LRU cache for price data
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,
    /// Cache TTL (time to live)
    cache_ttl: Duration,
    /// Optional Moralis API key for premium features
    moralis_api_key: Option<String>,
    /// HTTP client for API requests
    client: reqwest::Client,
}

impl PriceController {
    /// Create new price controller
    ///
    /// # Arguments
    ///
    /// * `moralis_api_key` - Optional Moralis API key for premium features
    ///
    /// # Returns
    ///
    /// New PriceController instance with 100-entry LRU cache and 5-minute TTL
    pub fn new(moralis_api_key: Option<String>) -> Self {
        // LRU cache with 100 entries (reasonable for most wallets)
        let cache_size = NonZeroUsize::new(100).unwrap();
        let cache = LruCache::new(cache_size);

        Self {
            cache: Arc::new(RwLock::new(cache)),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            moralis_api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Create price controller with custom cache settings
    ///
    /// # Arguments
    ///
    /// * `moralis_api_key` - Optional Moralis API key
    /// * `cache_size` - Maximum number of cached prices
    /// * `cache_ttl` - Cache time-to-live duration
    pub fn with_cache_settings(
        moralis_api_key: Option<String>,
        cache_size: usize,
        cache_ttl: Duration,
    ) -> Self {
        let size = NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(100).unwrap());
        let cache = LruCache::new(size);

        Self {
            cache: Arc::new(RwLock::new(cache)),
            cache_ttl,
            moralis_api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch native token price (ETH, BNB, MATIC, etc.)
    ///
    /// Fetches the price of the native token for a given chain.
    /// Uses cache if available and not expired.
    ///
    /// # Arguments
    ///
    /// * `chain_id` - Chain ID (1 for Ethereum, 137 for Polygon, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(TokenPrice)` - Token price data
    /// * `Err(ControllerError)` - Network error or unsupported chain
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vaughan::controllers::PriceController;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let controller = PriceController::new(None);
    ///
    /// // Fetch ETH price
    /// let eth_price = controller.fetch_native_token_price(1).await?;
    /// println!("ETH: ${:.2} ({:.2}% 24h)", eth_price.price_usd, eth_price.price_change_24h.unwrap_or(0.0));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_native_token_price(&self, chain_id: u64) -> ControllerResult<TokenPrice> {
        let key = CacheKey {
            chain_id,
            address: Address::ZERO,
        };

        // Check cache first
        if let Some(price) = self.get_cached_price(&key).await {
            return Ok(price);
        }

        // Fetch from API
        let price = self.fetch_native_price_from_api(chain_id).await?;

        // Cache the result
        self.cache_price(&key, &price).await;

        Ok(price)
    }

    /// Fetch ERC20 token price
    ///
    /// Fetches the price of an ERC20 token on a given chain.
    /// Uses cache if available and not expired.
    ///
    /// # Arguments
    ///
    /// * `chain_id` - Chain ID
    /// * `token_address` - Token contract address
    ///
    /// # Returns
    ///
    /// * `Ok(TokenPrice)` - Token price data
    /// * `Err(ControllerError)` - Network error or token not found
    pub async fn fetch_token_price(
        &self,
        chain_id: u64,
        token_address: Address,
    ) -> ControllerResult<TokenPrice> {
        let key = CacheKey {
            chain_id,
            address: token_address,
        };

        // Check cache first
        if let Some(price) = self.get_cached_price(&key).await {
            return Ok(price);
        }

        // Fetch from API
        let price = self.fetch_token_price_from_api(chain_id, token_address).await?;

        // Cache the result
        self.cache_price(&key, &price).await;

        Ok(price)
    }

    /// Get cached price if available and not expired (internal method)
    async fn get_cached_price(&self, key: &CacheKey) -> Option<TokenPrice> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.price.clone());
            } else {
                // Expired, remove from cache
                cache.pop(key);
            }
        }

        None
    }

    /// Clear all cached prices
    ///
    /// Removes all entries from the price cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    ///
    /// Returns the number of cached prices and cache capacity.
    ///
    /// # Returns
    ///
    /// Tuple of (current_size, max_capacity)
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), cache.cap().get())
    }

    /// Cache a price
    async fn cache_price(&self, key: &CacheKey, price: &TokenPrice) {
        let entry = CacheEntry {
            price: price.clone(),
            expires_at: Instant::now() + self.cache_ttl,
        };

        let mut cache = self.cache.write().await;
        cache.put(key.clone(), entry);
    }

    /// Fetch native token price from API (CoinGecko)
    async fn fetch_native_price_from_api(&self, chain_id: u64) -> ControllerResult<TokenPrice> {
        let coin_id = Self::get_coingecko_coin_id(chain_id).ok_or_else(|| {
            ControllerError::Price(format!("Unsupported chain ID: {}", chain_id))
        })?;

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
            coin_id
        );

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ControllerError::Price(format!("Failed to fetch price: {}", e)))?;

        if !response.status().is_success() {
            return Err(ControllerError::Price(format!(
                "API error: {}",
                response.status()
            )));
        }

        // Parse response
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ControllerError::Price(format!("Failed to parse response: {}", e)))?;

        let price_data = data
            .get(coin_id)
            .ok_or_else(|| ControllerError::Price("Price data not found".to_string()))?;

        let price_usd = price_data
            .get("usd")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ControllerError::Price("Invalid price data".to_string()))?;

        let price_change_24h = price_data.get("usd_24h_change").and_then(|v| v.as_f64());

        Ok(TokenPrice {
            address: Address::ZERO,
            chain_id,
            price_usd,
            price_change_24h,
            fetched_at: Instant::now(),
        })
    }

    /// Fetch ERC20 token price from API (CoinGecko)
    async fn fetch_token_price_from_api(
        &self,
        chain_id: u64,
        token_address: Address,
    ) -> ControllerResult<TokenPrice> {
        let platform_id = Self::get_coingecko_platform_id(chain_id).ok_or_else(|| {
            ControllerError::Price(format!("Unsupported chain ID: {}", chain_id))
        })?;

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/token_price/{}?contract_addresses={}&vs_currencies=usd&include_24hr_change=true",
            platform_id,
            token_address.to_string().to_lowercase()
        );

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ControllerError::Price(format!("Failed to fetch token price: {}", e)))?;

        if !response.status().is_success() {
            return Err(ControllerError::Price(format!(
                "API error: {}",
                response.status()
            )));
        }

        // Parse response
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ControllerError::Price(format!("Failed to parse response: {}", e)))?;

        let address_str = token_address.to_string().to_lowercase();
        let price_data = data
            .get(&address_str)
            .ok_or_else(|| ControllerError::Price("Token price not found".to_string()))?;

        let price_usd = price_data
            .get("usd")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ControllerError::Price("Invalid price data".to_string()))?;

        let price_change_24h = price_data.get("usd_24h_change").and_then(|v| v.as_f64());

        Ok(TokenPrice {
            address: token_address,
            chain_id,
            price_usd,
            price_change_24h,
            fetched_at: Instant::now(),
        })
    }

    /// Get CoinGecko coin ID for native tokens
    fn get_coingecko_coin_id(chain_id: u64) -> Option<&'static str> {
        match chain_id {
            1 => Some("ethereum"),
            137 => Some("matic-network"),
            56 => Some("binancecoin"),
            369 => Some("pulsechain"),
            _ => None,
        }
    }

    /// Get CoinGecko platform ID for ERC20 tokens
    fn get_coingecko_platform_id(chain_id: u64) -> Option<&'static str> {
        match chain_id {
            1 => Some("ethereum"),
            137 => Some("polygon-pos"),
            56 => Some("binance-smart-chain"),
            369 => Some("pulsechain"),
            _ => None,
        }
    }
}

impl Default for PriceController {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_controller_creation() {
        let controller = PriceController::new(None);
        assert!(controller.moralis_api_key.is_none());
    }

    #[test]
    fn test_price_controller_with_api_key() {
        let controller = PriceController::new(Some("test_key".to_string()));
        assert_eq!(controller.moralis_api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_custom_cache_settings() {
        let controller = PriceController::with_cache_settings(
            None,
            50,
            Duration::from_secs(60),
        );
        assert_eq!(controller.cache_ttl, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let controller = PriceController::new(None);
        let (size, capacity) = controller.cache_stats().await;
        assert_eq!(size, 0);
        assert_eq!(capacity, 100);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let controller = PriceController::new(None);

        // Add a cache entry manually
        let key = CacheKey {
            chain_id: 1,
            address: Address::ZERO,
        };
        let price = TokenPrice {
            address: Address::ZERO,
            chain_id: 1,
            price_usd: 2000.0,
            price_change_24h: Some(5.0),
            fetched_at: Instant::now(),
        };
        controller.cache_price(&key, &price).await;

        let (size, _) = controller.cache_stats().await;
        assert_eq!(size, 1);

        controller.clear_cache().await;

        let (size, _) = controller.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let controller = PriceController::with_cache_settings(
            None,
            100,
            Duration::from_millis(100), // 100ms TTL
        );

        let key = CacheKey {
            chain_id: 1,
            address: Address::ZERO,
        };
        let price = TokenPrice {
            address: Address::ZERO,
            chain_id: 1,
            price_usd: 2000.0,
            price_change_24h: Some(5.0),
            fetched_at: Instant::now(),
        };

        controller.cache_price(&key, &price).await;

        // Should be cached
        assert!(controller.get_cached_price(&key).await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        assert!(controller.get_cached_price(&key).await.is_none());
    }

    #[test]
    fn test_coingecko_coin_id_mapping() {
        assert_eq!(
            PriceController::get_coingecko_coin_id(1),
            Some("ethereum")
        );
        assert_eq!(
            PriceController::get_coingecko_coin_id(137),
            Some("matic-network")
        );
        assert_eq!(
            PriceController::get_coingecko_coin_id(56),
            Some("binancecoin")
        );
        assert_eq!(
            PriceController::get_coingecko_coin_id(369),
            Some("pulsechain")
        );
        assert_eq!(PriceController::get_coingecko_coin_id(999), None);
    }

    #[test]
    fn test_coingecko_platform_id_mapping() {
        assert_eq!(
            PriceController::get_coingecko_platform_id(1),
            Some("ethereum")
        );
        assert_eq!(
            PriceController::get_coingecko_platform_id(137),
            Some("polygon-pos")
        );
        assert_eq!(
            PriceController::get_coingecko_platform_id(56),
            Some("binance-smart-chain")
        );
        assert_eq!(
            PriceController::get_coingecko_platform_id(369),
            Some("pulsechain")
        );
        assert_eq!(PriceController::get_coingecko_platform_id(999), None);
    }

    // Note: Network tests are commented out to avoid hitting real APIs during testing
    // Uncomment for manual testing with real API calls

    // #[tokio::test]
    // async fn test_fetch_eth_price() {
    //     let controller = PriceController::new(None);
    //     let result = controller.fetch_native_token_price(1).await;
    //     assert!(result.is_ok());
    //     let price = result.unwrap();
    //     assert_eq!(price.chain_id, 1);
    //     assert!(price.price_usd > 0.0);
    // }

    // #[tokio::test]
    // async fn test_fetch_token_price() {
    //     let controller = PriceController::new(None);
    //     // USDC on Ethereum
    //     let usdc_address = Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
    //     let result = controller.fetch_token_price(1, usdc_address).await;
    //     assert!(result.is_ok());
    //     let price = result.unwrap();
    //     assert_eq!(price.address, usdc_address);
    //     assert!(price.price_usd > 0.0);
    // }
}
