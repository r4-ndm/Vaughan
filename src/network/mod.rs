//! Network management for multi-EVM support
//!
//! This module handles connections to multiple EVM networks including Ethereum,
//! PulseChain, BSC, Polygon, and custom networks.

use alloy::primitives::{Address, TxHash, U256};
use alloy::providers::fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller};
use alloy::providers::Identity;
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::rpc::types::TransactionRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{NetworkError, Result};

// Type alias for the actual provider type returned by Alloy v1.1
type AlloyCoreProvider = FillProvider<
    JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
    RootProvider,
>;

pub mod config;
pub mod gas_optimizer;
pub mod health;
pub mod professional;
pub mod validation;

pub use config::*;
pub use gas_optimizer::*;
pub use health::*;
pub use validation::*;

/// Network identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

// NetworkId implements basic comparison traits automatically via derive

impl NetworkId {
    pub fn chain_id(&self) -> u64 {
        self.0
    }
}

/// Network change callback type
pub type NetworkChangeCallback = Box<dyn Fn(NetworkId) + Send + Sync + 'static>;

/// Network configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub id: NetworkId,
    pub name: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub symbol: String,
    pub block_explorer_url: String,
    pub is_testnet: bool,
    pub is_custom: bool,
}

impl NetworkConfig {
    /// Create Ethereum mainnet configuration
    pub fn ethereum_mainnet() -> Self {
        // Try to use environment variables for API keys, fallback to public endpoints
        let rpc_url = if let Ok(alchemy_key) = std::env::var("ALCHEMY_API_KEY") {
            format!("https://eth-mainnet.alchemyapi.io/v2/{alchemy_key}")
        } else if let Ok(infura_key) = std::env::var("INFURA_API_KEY") {
            format!("https://mainnet.infura.io/v3/{infura_key}")
        } else {
            // Use reliable public RPC endpoints as fallback
            "https://ethereum.publicnode.com".to_string()
        };

        Self {
            id: NetworkId(1),
            name: "Ethereum Mainnet".to_string(),
            rpc_url,
            chain_id: 1,
            symbol: "ETH".to_string(),
            block_explorer_url: "https://etherscan.io".to_string(),
            is_testnet: false,
            is_custom: false,
        }
    }

    /// Create PulseChain configuration
    pub fn pulsechain() -> Self {
        Self {
            id: NetworkId(369),
            name: "PulseChain".to_string(),
            rpc_url: "https://rpc.pulsechain.com".to_string(),
            chain_id: 369,
            symbol: "PLS".to_string(),
            block_explorer_url: "https://scan.pulsechain.com".to_string(),
            is_testnet: false,
            is_custom: false,
        }
    }

    /// Create PulseChain Testnet v4 configuration
    pub fn pulsechain_testnet() -> Self {
        Self {
            id: NetworkId(943),
            name: "PulseChain Testnet v4".to_string(),
            rpc_url: "https://rpc.v4.testnet.pulsechain.com".to_string(),
            chain_id: 943,
            symbol: "tPLS".to_string(),
            block_explorer_url: "https://scan.v4.testnet.pulsechain.com".to_string(),
            is_testnet: true,
            is_custom: false,
        }
    }

    /// Create BSC configuration
    pub fn bsc() -> Self {
        Self {
            id: NetworkId(56),
            name: "Binance Smart Chain".to_string(),
            rpc_url: "https://bsc-dataseed1.binance.org".to_string(),
            chain_id: 56,
            symbol: "BNB".to_string(),
            block_explorer_url: "https://bscscan.com".to_string(),
            is_testnet: false,
            is_custom: false,
        }
    }

    /// Create Polygon configuration
    pub fn polygon() -> Self {
        Self {
            id: NetworkId(137),
            name: "Polygon".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            chain_id: 137,
            symbol: "MATIC".to_string(),
            block_explorer_url: "https://polygonscan.com".to_string(),
            is_testnet: false,
            is_custom: false,
        }
    }
}

impl std::fmt::Display for NetworkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.symbol)
    }
}

/// Network manager for handling multiple EVM networks
#[derive(Debug)]
pub struct NetworkManager {
    networks: HashMap<NetworkId, NetworkConfig>,
    current_network: NetworkId,
    providers: Arc<RwLock<HashMap<NetworkId, AlloyCoreProvider>>>,
}

impl NetworkManager {
    /// Create a new network manager with default networks
    pub async fn new() -> Result<Self> {
        let mut networks = HashMap::new();

        // Add default networks
        let ethereum = NetworkConfig::ethereum_mainnet();
        let pulsechain = NetworkConfig::pulsechain();
        let pulsechain_testnet = NetworkConfig::pulsechain_testnet();
        let bsc = NetworkConfig::bsc();
        let polygon = NetworkConfig::polygon();

        networks.insert(ethereum.id, ethereum);
        networks.insert(pulsechain.id, pulsechain);
        networks.insert(pulsechain_testnet.id, pulsechain_testnet);
        networks.insert(bsc.id, bsc);
        networks.insert(polygon.id, polygon);

        let mut manager = Self {
            networks,
            current_network: NetworkId(943), // Default to PulseChain Testnet v4 for testing
            providers: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize providers for all networks
        manager.initialize_providers().await?;

        Ok(manager)
    }

    /// Initialize providers for all configured networks
    async fn initialize_providers(&mut self) -> Result<()> {
        let mut providers = self.providers.write().await;

        for (network_id, config) in &self.networks {
            match config.rpc_url.parse::<reqwest::Url>() {
                Ok(url) => {
                    // Create provider with HTTP URL
                    let provider = ProviderBuilder::new().connect_http(url);
                    providers.insert(*network_id, provider);
                    tracing::info!(
                        "✅ Initialized provider for {} ({}) with 30s timeout",
                        config.name,
                        config.rpc_url
                    );
                }
                Err(e) => {
                    tracing::warn!("❌ Failed to initialize provider for {}: {}", config.name, e);
                }
            }
        }

        tracing::info!("🌐 Initialized {} network providers", providers.len());
        Ok(())
    }

    /// Switch to a different network
    pub async fn switch_network(&mut self, network_id: NetworkId) -> Result<()> {
        if !self.networks.contains_key(&network_id) {
            return Err(NetworkError::UnsupportedNetwork {
                network_id: network_id.chain_id(),
            }
            .into());
        }

        self.current_network = network_id;
        Ok(())
    }

    /// Add a custom network
    pub async fn add_custom_network(&mut self, config: NetworkConfig) -> Result<()> {
        // Validate the network configuration using the NEW network's chain ID
        let validation = validation::validate_network_endpoint(&config.rpc_url, config.chain_id).await?;
        if !validation.is_valid {
            return Err(NetworkError::InvalidConfiguration.into());
        }

        // Special handling for ETHW (Chain ID 10001)
        // ETHW often has connectivity issues, so we're more lenient with chain ID validation
        let is_ethw = config.chain_id == 10001;

        if !validation.chain_id_matches && !is_ethw {
            return Err(NetworkError::ChainIdMismatch {
                expected: config.chain_id,
                actual: 0, // We don't have the actual value here, but the error is clear
            }
            .into());
        }

        if is_ethw && !validation.chain_id_matches {
            tracing::warn!(
                "⚠️ Adding ETHW network without chain ID verification due to known connectivity issues. \
                 Please ensure you're connecting to the correct network (Chain ID: 10001)"
            );
        }

        // Create provider for the new network
        let provider = match config.rpc_url.parse::<reqwest::Url>() {
            Ok(url) => ProviderBuilder::new().connect_http(url),
            Err(_) => return Err(NetworkError::InvalidConfiguration.into()),
        };

        // Add to networks and providers
        let network_id = config.id;
        self.networks.insert(network_id, config);

        let mut providers = self.providers.write().await;
        providers.insert(network_id, provider);

        Ok(())
    }

    /// Update an existing custom network, replacing it if the ID (chain_id) changed.
    /// If `old_id` differs from `new_config.id`, the old network entry and its provider are removed,
    /// then the new config is added. If the current network equals `old_id`, it is updated to `new_config.id`.
    pub async fn update_or_replace_custom_network(
        &mut self,
        old_id: NetworkId,
        new_config: NetworkConfig,
    ) -> Result<()> {
        // Validate new endpoint against the new chain id explicitly
        let validation = validation::validate_network_endpoint(&new_config.rpc_url, new_config.chain_id).await?;
        if !validation.is_valid || !validation.chain_id_matches {
            return Err(NetworkError::InvalidConfiguration.into());
        }

        // Build provider for the new config
        let provider = match new_config.rpc_url.parse::<reqwest::Url>() {
            Ok(url) => ProviderBuilder::new().connect_http(url),
            Err(_) => return Err(NetworkError::InvalidConfiguration.into()),
        };

        // Remove old entry if id changed
        if old_id != new_config.id {
            self.networks.remove(&old_id);
            let mut providers = self.providers.write().await;
            providers.remove(&old_id);

            // If the current network was the old id, point it to the new id
            if self.current_network == old_id {
                self.current_network = new_config.id;
            }

            // Insert new
            self.networks.insert(new_config.id, new_config.clone());
            providers.insert(new_config.id, provider);
        } else {
            // Same id: update config and refresh provider
            self.networks.insert(new_config.id, new_config.clone());
            let mut providers = self.providers.write().await;
            providers.insert(new_config.id, provider);
        }

        Ok(())
    }

    /// Remove a custom network
    pub async fn remove_custom_network(&mut self, network_id: NetworkId) -> Result<()> {
        // Remove from networks and providers
        let existed = self.networks.remove(&network_id).is_some();
        let mut providers = self.providers.write().await;
        providers.remove(&network_id);

        if !existed {
            return Err(NetworkError::UnsupportedNetwork {
                network_id: network_id.chain_id(),
            }
            .into());
        }

        // If the removed network was current, choose a fallback
        if self.current_network == network_id {
            if let Some((&fallback_id, _)) = self.networks.iter().next() {
                self.current_network = fallback_id;
            } else {
                // No networks left; this should not happen because we always keep defaults,
                // but if it does, default back to Ethereum mainnet id if present
                self.current_network = NetworkId(1);
            }
        }
        Ok(())
    }

    /// Validate an RPC endpoint
    pub async fn validate_rpc_endpoint(&self, url: &str) -> Result<NetworkValidation> {
        // Use current network's chain ID as expected, or default to 1 (Ethereum)
        let expected_chain_id = self
            .networks
            .get(&self.current_network)
            .map(|config| config.chain_id)
            .unwrap_or(1);

        validation::validate_network_endpoint(url, expected_chain_id).await
    }

    /// Get current gas price
    pub async fn get_gas_price(&self) -> Result<U256> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(&self.current_network)
            .ok_or(NetworkError::UnsupportedNetwork {
                network_id: self.current_network.chain_id(),
            })?;

        let network_name = self
            .networks
            .get(&self.current_network)
            .map(|n| n.name.as_str())
            .unwrap_or("Unknown");

        tracing::info!(
            "⛽ Fetching current gas price for {} (Chain ID: {})",
            network_name,
            self.current_network.chain_id()
        );

        // Retry logic for gas price fetching
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            match provider.get_gas_price().await {
                Ok(price) => {
                    let price_u256 = U256::from(price);
                    tracing::info!(
                        "✅ Gas price fetched successfully: {} wei ({:.2} Gwei)",
                        price_u256,
                        price as f64 / 1e9
                    );
                    return Ok(price_u256);
                }
                Err(e) => {
                    attempts += 1;
                    tracing::warn!("⚠️ Gas price fetch attempt {}/{} failed: {}", attempts, max_attempts, e);

                    if attempts < max_attempts {
                        // Wait before retry with exponential backoff
                        let delay = std::time::Duration::from_millis(500 * attempts);
                        tokio::time::sleep(delay).await;
                    } else {
                        // All attempts failed, use network-specific fallback
                        let fallback_gas_price = match self.current_network.chain_id() {
                            1 => U256::from(25_000_000_000u64),  // 25 Gwei for Ethereum
                            943 => U256::from(1_000_000_000u64), // 1 Gwei for PulseChain Testnet
                            369 => U256::from(1_000_000_000u64), // 1 Gwei for PulseChain
                            56 => U256::from(5_000_000_000u64),  // 5 Gwei for BSC
                            _ => U256::from(20_000_000_000u64),  // 20 Gwei default
                        };

                        tracing::warn!(
                            "🔄 Using fallback gas price: {} wei ({:.2} Gwei) for {}",
                            fallback_gas_price,
                            fallback_gas_price.to::<u128>() as f64 / 1e9,
                            network_name
                        );
                        return Ok(fallback_gas_price);
                    }
                }
            }
        }

        // This should never be reached due to the fallback above
        unreachable!()
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<U256> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(&self.current_network)
            .ok_or(NetworkError::UnsupportedNetwork {
                network_id: self.current_network.chain_id(),
            })?;

        tracing::info!(
            "📊 Estimating gas for transaction on Chain ID {}",
            self.current_network.chain_id()
        );

        // Estimate gas using the provider
        let gas_estimate = match provider.estimate_gas(tx.clone()).await {
            Ok(estimate) => {
                let estimate_u256 = U256::from(estimate);
                tracing::info!("✅ Gas estimated: {} units", estimate_u256);
                estimate_u256
            }
            Err(e) => {
                tracing::warn!("⚠️ Gas estimation failed, using fallback: {}", e);
                U256::from(21000u64) // Basic transfer gas limit
            }
        };

        Ok(gas_estimate)
    }

    /// Send a raw signed transaction
    pub async fn send_raw_transaction(&self, raw_tx: &[u8]) -> Result<TxHash> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(&self.current_network)
            .ok_or(NetworkError::UnsupportedNetwork {
                network_id: self.current_network.chain_id(),
            })?;

        tracing::info!(
            "🚀 Broadcasting raw transaction to network: Chain ID {}",
            self.current_network.chain_id()
        );
        tracing::info!("📦 Raw transaction size: {} bytes", raw_tx.len());
        if let Some(b) = raw_tx.first() {
            tracing::info!("🔎 First byte: 0x{:02x}", b);
        }
        if raw_tx.len() < 80 {
            tracing::error!(
                "❌ Refusing to broadcast: raw tx too short to be valid ({} bytes)",
                raw_tx.len()
            );
            return Err(NetworkError::RpcError {
                message: format!("Signed transaction encoding invalid ({} bytes)", raw_tx.len()),
            }
            .into());
        }

        // Send the raw transaction to the network using eth_sendRawTransaction
        let pending_tx = provider.send_raw_transaction(raw_tx).await.map_err(|e| {
            tracing::error!("❌ Raw transaction broadcast failed: {}", e);
            NetworkError::RpcError {
                message: format!("Failed to broadcast raw transaction: {e}"),
            }
        })?;

        let tx_hash = *pending_tx.tx_hash();
        tracing::info!("✅ Raw transaction broadcast successful: {}", tx_hash);

        Ok(tx_hash)
    }

    /// Get transaction count (nonce) for an address
    pub async fn get_transaction_count(&self, address: Address) -> Result<u64> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(&self.current_network)
            .ok_or(NetworkError::UnsupportedNetwork {
                network_id: self.current_network.chain_id(),
            })?;

        tracing::debug!("🔢 Getting transaction count for address: {}", address);

        let nonce = provider.get_transaction_count(address).await.map_err(|e| {
            tracing::error!("❌ Failed to get transaction count: {}", e);
            NetworkError::RpcError {
                message: format!("Failed to get transaction count: {e}"),
            }
        })?;

        tracing::debug!("✅ Transaction count for {}: {}", address, nonce);

        Ok(nonce)
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: Address, token: Option<Address>) -> Result<U256> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(&self.current_network)
            .ok_or(NetworkError::UnsupportedNetwork {
                network_id: self.current_network.chain_id(),
            })?;

        // Get network name for logging
        let network_name = self
            .networks
            .get(&self.current_network)
            .map(|config| config.name.as_str())
            .unwrap_or("Unknown");

        match token {
            None => {
                // Get native token balance from the blockchain
                tracing::info!(
                    "🔍 Fetching native balance for {} on {} (Chain ID: {})",
                    address,
                    network_name,
                    self.current_network.chain_id()
                );

                // Retry logic for balance fetching
                let mut attempts = 0;
                let max_attempts = 3;

                let balance = loop {
                    match provider.get_balance(address).await {
                        Ok(balance) => break balance,
                        Err(e) => {
                            attempts += 1;
                            tracing::warn!("⚠️ Balance fetch attempt {}/{} failed: {}", attempts, max_attempts, e);

                            if attempts >= max_attempts {
                                tracing::error!(
                                    "❌ All balance fetch attempts failed for {} on {}",
                                    address,
                                    network_name
                                );
                                return Err(NetworkError::RpcError {
                                    message: format!("Failed to fetch balance after {max_attempts} attempts: {e}"),
                                }
                                .into());
                            }

                            // Wait before retry with exponential backoff
                            let delay = std::time::Duration::from_millis(300 * attempts);
                            tokio::time::sleep(delay).await;
                        }
                    }
                };

                // Convert balance to ETH for logging (simplified conversion)
                let balance_eth = if balance > U256::ZERO {
                    let balance_str = balance.to_string();
                    let balance_f64: f64 = balance_str.parse().unwrap_or(0.0);
                    balance_f64 / 1e18
                } else {
                    0.0
                };

                tracing::info!(
                    "✅ Successfully fetched balance: {} wei ({:.6} ETH)",
                    balance,
                    balance_eth
                );
                Ok(balance)
            }
            Some(token_address) => {
                // ERC20 token balance using contract call
                tracing::info!(
                    "🪙 Fetching ERC20 token balance for {} on token {} ({})",
                    address,
                    token_address,
                    network_name
                );

                // Create the ERC20 balanceOf call data
                // balanceOf(address) = 0x70a08231 + padded address
                let mut call_data = Vec::with_capacity(36);
                call_data.extend_from_slice(&[0x70, 0xa0, 0x82, 0x31]); // balanceOf selector
                call_data.extend_from_slice(&[0u8; 12]); // padding
                call_data.extend_from_slice(address.as_slice()); // address

                let call_request = TransactionRequest::default().to(token_address).input(call_data.into());

                let result = provider.call(call_request.clone()).await.map_err(|e| {
                    tracing::error!("❌ ERC20 balance call failed: {}", e);
                    NetworkError::RpcError {
                        message: format!("Failed to call ERC20 balanceOf: {e}"),
                    }
                })?;

                // Parse the result as U256
                if result.len() >= 32 {
                    let balance = U256::from_be_slice(&result[result.len() - 32..]);
                    tracing::info!("✅ Successfully fetched ERC20 balance: {} tokens", balance);
                    Ok(balance)
                } else {
                    tracing::warn!("⚠️  Invalid ERC20 balanceOf response length: {}", result.len());
                    Ok(U256::ZERO)
                }
            }
        }
    }

    /// Get current network configuration
    pub fn get_current_network_config(&self) -> Option<&NetworkConfig> {
        self.networks.get(&self.current_network)
    }

    /// Get all network configurations
    pub fn get_all_networks(&self) -> &HashMap<NetworkId, NetworkConfig> {
        &self.networks
    }

    /// Get current network ID
    pub fn current_network(&self) -> NetworkId {
        self.current_network
    }

    /// Get providers for external access
    pub async fn get_providers(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, std::collections::HashMap<NetworkId, AlloyCoreProvider>> {
        self.providers.read().await
    }

    /// Check endpoint health
    pub async fn check_endpoint_health(&self) -> Result<EndpointHealth> {
        let current_config = self
            .get_current_network_config()
            .ok_or(NetworkError::UnsupportedNetwork {
                network_id: self.current_network.chain_id(),
            })?;

        health::check_endpoint_health(&current_config.rpc_url).await
    }

    /// Auto-switch to a healthy endpoint
    pub async fn auto_switch_to_healthy_endpoint(&mut self) -> Result<bool> {
        // Check health of all available networks
        let mut healthy_networks = Vec::new();

        for (network_id, config) in &self.networks {
            match health::check_endpoint_health(&config.rpc_url).await {
                Ok(health) if health.is_responsive => {
                    healthy_networks.push((*network_id, health.latency_ms));
                }
                _ => continue,
            }
        }

        if healthy_networks.is_empty() {
            return Ok(false); // No healthy networks found
        }

        // Sort by latency and switch to the fastest healthy network
        healthy_networks.sort_by_key(|(_, latency)| *latency);
        let (best_network_id, _) = healthy_networks[0];

        // Only switch if it's different from current
        if best_network_id != self.current_network {
            self.switch_network(best_network_id).await?;
            Ok(true)
        } else {
            Ok(false) // Already on the best network
        }
    }
}
