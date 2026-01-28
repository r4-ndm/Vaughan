//! Network Controller - Alloy provider management
//!
//! Follows MetaMask's NetworkController pattern for network state management.
//!
//! ## Design Principles
//!
//! 1. **Provider Management**: Creates and manages Alloy providers
//! 2. **Network Health**: Monitors network connectivity and chain ID
//! 3. **Type Safety**: Uses Alloy ChainId and Address types
//! 4. **MetaMask Patterns**: Network switching and validation
//!
//! ## MetaMask Inspiration
//!
//! This controller implements patterns from MetaMask's NetworkController:
//! - Provider lifecycle management
//! - Chain ID verification
//! - Network health checks
//! - Balance fetching with Alloy types

use super::{ControllerError, ControllerResult};
use alloy::primitives::{Address, ChainId, U256};
use alloy::providers::fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller};
use alloy::providers::{Identity, Provider, ProviderBuilder, RootProvider};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// Type alias for HTTP provider (matches existing codebase pattern)
///
/// This is the concrete type returned by `ProviderBuilder::new().connect_http(url)`.
/// Using a type alias ensures consistency with the rest of the codebase.
type HttpProvider = FillProvider<
    JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
    RootProvider,
>;

/// Network controller - manages providers and network state
///
/// This controller handles network switching, provider management, and health checks
/// using only Alloy providers. It is generic over the provider type to support
/// different Alloy provider configurations.
///
/// ## Example
///
/// ```rust,no_run
/// use vaughan::controllers::NetworkController;
/// use alloy::primitives::ChainId;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let controller = NetworkController::new(
///     "https://rpc.pulsechain.com".to_string(),
///     ChainId::from(369u64)
/// ).await?;
///
/// // Check network health
/// let is_healthy = controller.check_network_health().await?;
/// assert!(is_healthy);
/// # Ok(())
/// # }
/// ```
pub struct NetworkController<P> {
    current_provider: Arc<RwLock<P>>,
    current_chain_id: ChainId,
    rpc_url: String,
}

impl<P> NetworkController<P>
where
    P: Provider + Clone + 'static,
{
    /// Create new network controller from an existing provider
    ///
    /// This is the primary constructor that accepts any Alloy provider type.
    ///
    /// # Arguments
    ///
    /// * `provider` - Alloy provider instance
    /// * `chain_id` - Expected chain ID
    /// * `rpc_url` - RPC endpoint URL (for reference)
    pub fn from_provider(provider: P, chain_id: ChainId, rpc_url: String) -> Self {
        Self {
            current_provider: Arc::new(RwLock::new(provider)),
            current_chain_id: chain_id,
            rpc_url,
        }
    }

    /// Get current chain ID from network (Alloy provider)
    ///
    /// Queries the network to verify the actual chain ID matches expectations.
    ///
    /// # Returns
    ///
    /// * `Ok(ChainId)` - Current chain ID from network
    /// * `Err(ControllerError)` - Network error
    pub async fn get_chain_id(&self) -> ControllerResult<ChainId> {
        let provider = self.current_provider.read().await;

        let chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| ControllerError::Network(format!("Failed to get chain ID: {}", e)))?;

        Ok(ChainId::from(chain_id))
    }

    /// Validate network health (Alloy provider)
    ///
    /// Checks if the network is responsive by fetching the latest block number.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Network is healthy
    /// * `Ok(false)` - Network is unhealthy (block number is 0)
    /// * `Err(ControllerError)` - Network error
    pub async fn check_network_health(&self) -> ControllerResult<bool> {
        let provider = self.current_provider.read().await;

        // Try to get latest block number as health check
        let block_number = provider
            .get_block_number()
            .await
            .map_err(|e| ControllerError::Network(format!("Health check failed: {}", e)))?;

        Ok(block_number > 0)
    }

    /// Get balance for address (Alloy types)
    ///
    /// Fetches the native token balance for an address.
    ///
    /// # Arguments
    ///
    /// * `address` - Ethereum address (Alloy Address)
    ///
    /// # Returns
    ///
    /// * `Ok(U256)` - Balance in wei
    /// * `Err(ControllerError)` - Network error
    pub async fn get_balance(&self, address: Address) -> ControllerResult<U256> {
        let provider = self.current_provider.read().await;

        let balance = provider
            .get_balance(address)
            .await
            .map_err(|e| ControllerError::Network(format!("Failed to get balance: {}", e)))?;

        Ok(balance)
    }

    /// Get current RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Get current chain ID
    pub fn chain_id(&self) -> ChainId {
        self.current_chain_id
    }

    /// Get provider reference for advanced operations
    ///
    /// Returns a reference to the underlying Alloy provider for operations
    /// not covered by the controller API.
    pub fn provider(&self) -> Arc<RwLock<P>> {
        self.current_provider.clone()
    }
}

// Convenience constructors for HTTP providers
impl NetworkController<HttpProvider> {
    /// Create new network controller with HTTP provider
    ///
    /// This is a convenience method that creates an HTTP provider internally.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - RPC endpoint URL
    /// * `chain_id` - Expected chain ID
    ///
    /// # Returns
    ///
    /// * `Ok(NetworkController)` - Successfully created controller
    /// * `Err(ControllerError)` - Invalid URL or connection failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vaughan::controllers::NetworkController;
    /// # use alloy::primitives::ChainId;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let controller = NetworkController::new(
    ///     "https://rpc.pulsechain.com".to_string(),
    ///     ChainId::from(369u64)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(rpc_url: String, chain_id: ChainId) -> ControllerResult<Self> {
        // Parse and validate URL
        let url = Url::parse(&rpc_url)
            .map_err(|e| ControllerError::Network(format!("Invalid RPC URL: {}", e)))?;

        // Create Alloy provider with HTTP transport (following existing codebase pattern)
        // Note: connect_http() is the correct method in Alloy v1.5
        let provider = ProviderBuilder::new().connect_http(url);

        Ok(Self::from_provider(provider, chain_id, rpc_url))
    }

    /// Switch to new network (Alloy provider)
    ///
    /// Creates a new provider for the specified network and verifies the chain ID.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - New RPC endpoint URL
    /// * `chain_id` - Expected chain ID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully switched networks
    /// * `Err(ControllerError)` - Invalid URL, connection failed, or chain ID mismatch
    pub async fn switch_network(
        &mut self,
        rpc_url: String,
        chain_id: ChainId,
    ) -> ControllerResult<()> {
        // Parse and validate URL
        let url = Url::parse(&rpc_url)
            .map_err(|e| ControllerError::Network(format!("Invalid RPC URL: {}", e)))?;

        // Create new provider with HTTP transport
        // Note: connect_http() is the correct method in Alloy v1.5
        let provider = ProviderBuilder::new().connect_http(url);

        // Verify chain ID matches (MetaMask pattern)
        let actual_chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| ControllerError::Network(format!("Failed to verify chain ID: {}", e)))?;

        if ChainId::from(actual_chain_id) != chain_id {
            return Err(ControllerError::Network(format!(
                "Chain ID mismatch: expected {}, got {}",
                chain_id, actual_chain_id
            )));
        }

        // Update state
        *self.current_provider.write().await = provider;
        self.current_chain_id = chain_id;
        self.rpc_url = rpc_url;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_controller_creation() {
        // Use a public RPC endpoint for testing
        let result = NetworkController::new(
            "https://rpc.pulsechain.com".to_string(),
            ChainId::from(369u64),
        )
        .await;

        assert!(result.is_ok());
        let controller = result.unwrap();
        assert_eq!(controller.chain_id(), ChainId::from(369u64));
        assert_eq!(controller.rpc_url(), "https://rpc.pulsechain.com");
    }

    #[tokio::test]
    async fn test_invalid_url_rejected() {
        let result = NetworkController::new(
            "not-a-valid-url".to_string(),
            ChainId::from(1u64),
        )
        .await;

        assert!(result.is_err());
        if let Err(ControllerError::Network(msg)) = result {
            assert!(msg.contains("Invalid RPC URL"));
        } else {
            panic!("Expected Network error");
        }
    }

    #[tokio::test]
    async fn test_get_balance() {
        // Use a public RPC endpoint
        let controller = NetworkController::new(
            "https://rpc.pulsechain.com".to_string(),
            ChainId::from(369u64),
        )
        .await
        .unwrap();

        // Test with a known address (zero address should have 0 balance)
        let balance = controller.get_balance(Address::ZERO).await;

        // Balance fetch should succeed (even if balance is 0)
        assert!(balance.is_ok());
    }

    #[tokio::test]
    async fn test_check_network_health() {
        let controller = NetworkController::new(
            "https://rpc.pulsechain.com".to_string(),
            ChainId::from(369u64),
        )
        .await
        .unwrap();

        let health = controller.check_network_health().await;

        // Health check should succeed for public endpoint
        assert!(health.is_ok());
        if let Ok(is_healthy) = health {
            assert!(is_healthy);
        }
    }

    #[tokio::test]
    async fn test_get_chain_id() {
        let controller = NetworkController::new(
            "https://rpc.pulsechain.com".to_string(),
            ChainId::from(369u64),
        )
        .await
        .unwrap();

        let chain_id = controller.get_chain_id().await;

        assert!(chain_id.is_ok());
        if let Ok(id) = chain_id {
            assert_eq!(id, ChainId::from(369u64));
        }
    }
}
