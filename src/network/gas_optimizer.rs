//! Advanced gas optimization using Alloy
//!
//! This module provides intelligent gas estimation and optimization strategies
//! leveraging Alloy's advanced features and real-time network data.

use alloy::primitives::U256;
use alloy::providers::fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller};
use alloy::providers::Identity;
use alloy::providers::{Provider, RootProvider};

// Type alias for the actual provider type returned by Alloy v1.1
type AlloyCoreProvider = FillProvider<
    JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
    RootProvider,
>;
use alloy::rpc::types::{FeeHistory, TransactionRequest};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::{NetworkError, Result};
use crate::network::{NetworkId, NetworkManager};

/// Advanced gas optimization strategies
#[derive(Debug, Clone)]
pub enum GasStrategy {
    /// Conservative: Use higher gas prices for guaranteed inclusion
    Conservative,
    /// Standard: Use market-average gas prices
    Standard,
    /// Aggressive: Use lower gas prices, accept longer wait times
    Aggressive,
    /// Dynamic: Adjust based on network congestion
    Dynamic,
    /// EIP-1559: Use priority fees for optimal inclusion
    Eip1559 { max_priority_fee: U256, max_fee: U256 },
}

/// Gas estimation result with optimization
#[derive(Debug, Clone)]
pub struct OptimizedGasEstimate {
    pub gas_limit: U256,
    pub gas_price: U256,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub estimated_cost: U256,
    pub strategy_used: GasStrategy,
    pub network_congestion: NetworkCongestion,
    pub time_estimate: TimeEstimate,
}

/// Network congestion levels
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkCongestion {
    Low,
    Medium,
    High,
    Critical,
}

/// Transaction time estimates
#[derive(Debug, Clone)]
pub struct TimeEstimate {
    pub fast: u64, // seconds
    pub standard: u64,
    pub safe: u64,
}

/// Advanced gas optimizer using Alloy
#[derive(Debug)]
pub struct GasOptimizer {
    providers: Arc<RwLock<std::collections::HashMap<NetworkId, AlloyCoreProvider>>>,
    fee_history_cache: Arc<RwLock<std::collections::HashMap<NetworkId, FeeHistory>>>,
}

impl GasOptimizer {
    /// Create a new gas optimizer
    pub fn new(_network_manager: Arc<NetworkManager>) -> Self {
        Self {
            providers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            fee_history_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get optimized gas estimate for a transaction
    pub async fn estimate_optimized_gas(
        &self,
        tx: &TransactionRequest,
        network_id: NetworkId,
        strategy: GasStrategy,
    ) -> Result<OptimizedGasEstimate> {
        let providers = self.providers.read().await;
        let provider = providers.get(&network_id).ok_or(NetworkError::UnsupportedNetwork {
            network_id: network_id.chain_id(),
        })?;

        info!(
            "🔍 Starting optimized gas estimation for Chain ID {}",
            network_id.chain_id()
        );

        // Get base gas estimate
        let gas_limit = self.estimate_gas_limit(provider, tx).await?;

        // Analyze network congestion
        let congestion = self.analyze_network_congestion(provider, network_id).await?;

        // Get fee history for EIP-1559 networks
        let fee_history = self.get_cached_fee_history(provider, network_id).await?;

        // Calculate optimal gas pricing based on strategy
        let (gas_price, max_fee, max_priority_fee) = self
            .calculate_optimal_pricing(&strategy, &congestion, &fee_history)
            .await?;

        // Estimate transaction times
        let time_estimate = self.estimate_transaction_times(&congestion, gas_price).await;

        // Calculate total estimated cost
        let estimated_cost = gas_limit * gas_price;

        info!("✅ Optimized gas estimation completed:");
        info!("   Gas Limit: {} units", gas_limit);
        info!(
            "   Gas Price: {} wei ({:.2} Gwei)",
            gas_price,
            gas_price.to::<u128>() as f64 / 1e9
        );
        info!("   Estimated Cost: {} wei", estimated_cost);
        info!("   Network Congestion: {:?}", congestion);

        Ok(OptimizedGasEstimate {
            gas_limit,
            gas_price,
            max_fee_per_gas: max_fee,
            max_priority_fee_per_gas: max_priority_fee,
            estimated_cost,
            strategy_used: strategy,
            network_congestion: congestion,
            time_estimate,
        })
    }

    /// Estimate gas limit with buffer
    async fn estimate_gas_limit(&self, provider: &AlloyCoreProvider, tx: &TransactionRequest) -> Result<U256> {
        match provider.estimate_gas(tx.clone()).await {
            Ok(estimate) => {
                let estimate_u256 = U256::from(estimate);
                // Add 10% buffer for safety
                let buffered = estimate_u256 * U256::from(110) / U256::from(100);
                info!("📊 Gas limit estimated: {} (with 10% buffer)", buffered);
                Ok(buffered)
            }
            Err(e) => {
                warn!("⚠️ Gas limit estimation failed: {}", e);
                // Return sensible defaults based on transaction type
                Ok(U256::from(
                    if tx.input.input.as_ref().is_some_and(|data| !data.is_empty()) {
                        200_000u64 // Contract interaction
                    } else {
                        21_000u64 // Simple transfer
                    },
                ))
            }
        }
    }

    /// Analyze network congestion
    async fn analyze_network_congestion(
        &self,
        provider: &AlloyCoreProvider,
        _network_id: NetworkId,
    ) -> Result<NetworkCongestion> {
        // Get recent block gas usage
        match provider.get_block_number().await {
            Ok(latest_block) => {
                // Analyze last 5 blocks for congestion patterns
                let mut total_gas_used = 0u64;
                let mut total_gas_limit = 0u64;

                for i in 0..5 {
                    if let Ok(Some(block)) = provider.get_block_by_number((latest_block - i).into()).await {
                        total_gas_used += block.header.gas_used;
                        total_gas_limit += block.header.gas_limit;
                    }
                }

                let utilization = if total_gas_limit > 0 {
                    (total_gas_used as f64 / total_gas_limit as f64) * 100.0
                } else {
                    50.0 // Default assumption
                };

                let congestion = match utilization {
                    x if x >= 95.0 => NetworkCongestion::Critical,
                    x if x >= 80.0 => NetworkCongestion::High,
                    x if x >= 60.0 => NetworkCongestion::Medium,
                    _ => NetworkCongestion::Low,
                };

                info!(
                    "📈 Network utilization: {:.1}% - Congestion: {:?}",
                    utilization, congestion
                );
                Ok(congestion)
            }
            Err(e) => {
                warn!("⚠️ Could not analyze network congestion: {}", e);
                Ok(NetworkCongestion::Medium) // Safe default
            }
        }
    }

    /// Get cached fee history for EIP-1559 optimization
    async fn get_cached_fee_history(&self, provider: &AlloyCoreProvider, network_id: NetworkId) -> Result<FeeHistory> {
        // Check cache first
        let cache = self.fee_history_cache.read().await;
        if let Some(cached) = cache.get(&network_id) {
            return Ok(cached.clone());
        }
        drop(cache);

        // Fetch fresh fee history
        match provider
            .get_fee_history(
                20,
                alloy::rpc::types::BlockNumberOrTag::Latest,
                &[10.0, 25.0, 50.0, 75.0, 90.0],
            )
            .await
        {
            Ok(fee_history) => {
                // Cache the result
                let mut cache = self.fee_history_cache.write().await;
                cache.insert(network_id, fee_history.clone());
                Ok(fee_history)
            }
            Err(e) => {
                warn!("⚠️ Fee history fetch failed: {}", e);
                // Return empty fee history
                Ok(alloy::rpc::types::FeeHistory {
                    oldest_block: 0u64,
                    base_fee_per_gas: vec![20_000_000_000u128; 20],
                    gas_used_ratio: vec![0.5; 19],
                    reward: Some(vec![vec![1_000_000_000u128; 5]; 19]),
                    base_fee_per_blob_gas: vec![1_000_000_000u128; 20],
                    blob_gas_used_ratio: vec![0.1; 19],
                })
            }
        }
    }

    /// Calculate optimal gas pricing
    async fn calculate_optimal_pricing(
        &self,
        strategy: &GasStrategy,
        congestion: &NetworkCongestion,
        fee_history: &FeeHistory,
    ) -> Result<(U256, Option<U256>, Option<U256>)> {
        let base_fee = fee_history
            .base_fee_per_gas
            .last()
            .copied()
            .unwrap_or(20_000_000_000u128); // 20 Gwei default

        match strategy {
            GasStrategy::Conservative => {
                let gas_price = U256::from(base_fee * 150u128 / 100u128); // 1.5x base fee
                Ok((gas_price, Some(gas_price), Some(U256::from(2_000_000_000u64))))
            }
            GasStrategy::Standard => {
                let gas_price = U256::from(base_fee * 120u128 / 100u128); // 1.2x base fee
                Ok((gas_price, Some(gas_price), Some(U256::from(1_500_000_000u64))))
            }
            GasStrategy::Aggressive => {
                let gas_price = U256::from(base_fee * 105u128 / 100u128); // 1.05x base fee
                Ok((gas_price, Some(gas_price), Some(U256::from(1_000_000_000u64))))
            }
            GasStrategy::Dynamic => {
                let multiplier = match congestion {
                    NetworkCongestion::Critical => 200, // 2.0x
                    NetworkCongestion::High => 150,     // 1.5x
                    NetworkCongestion::Medium => 120,   // 1.2x
                    NetworkCongestion::Low => 110,      // 1.1x
                };
                let gas_price = U256::from(base_fee * multiplier as u128 / 100u128);
                let priority_fee = match congestion {
                    NetworkCongestion::Critical => U256::from(5_000_000_000u64),
                    NetworkCongestion::High => U256::from(3_000_000_000u64),
                    NetworkCongestion::Medium => U256::from(2_000_000_000u64),
                    NetworkCongestion::Low => U256::from(1_000_000_000u64),
                };
                Ok((gas_price, Some(gas_price), Some(priority_fee)))
            }
            GasStrategy::Eip1559 {
                max_priority_fee,
                max_fee,
            } => Ok((
                U256::from(base_fee) + *max_priority_fee,
                Some(*max_fee),
                Some(*max_priority_fee),
            )),
        }
    }

    /// Estimate transaction confirmation times
    async fn estimate_transaction_times(&self, congestion: &NetworkCongestion, _gas_price: U256) -> TimeEstimate {
        match congestion {
            NetworkCongestion::Low => TimeEstimate {
                fast: 15,     // 15 seconds
                standard: 30, // 30 seconds
                safe: 60,     // 1 minute
            },
            NetworkCongestion::Medium => TimeEstimate {
                fast: 30,     // 30 seconds
                standard: 60, // 1 minute
                safe: 120,    // 2 minutes
            },
            NetworkCongestion::High => TimeEstimate {
                fast: 60,      // 1 minute
                standard: 180, // 3 minutes
                safe: 300,     // 5 minutes
            },
            NetworkCongestion::Critical => TimeEstimate {
                fast: 180,     // 3 minutes
                standard: 600, // 10 minutes
                safe: 900,     // 15 minutes
            },
        }
    }

    /// Get real-time gas price recommendations
    pub async fn get_gas_recommendations(
        &self,
        network_id: NetworkId,
    ) -> Result<std::collections::HashMap<String, U256>> {
        let providers = self.providers.read().await;
        let provider = providers.get(&network_id).ok_or(NetworkError::UnsupportedNetwork {
            network_id: network_id.chain_id(),
        })?;

        let congestion = self.analyze_network_congestion(provider, network_id).await?;
        let fee_history = self.get_cached_fee_history(provider, network_id).await?;

        let mut recommendations = std::collections::HashMap::new();

        // Calculate recommendations for different strategies
        for (name, strategy) in [
            ("conservative", GasStrategy::Conservative),
            ("standard", GasStrategy::Standard),
            ("aggressive", GasStrategy::Aggressive),
            ("dynamic", GasStrategy::Dynamic),
        ] {
            if let Ok((gas_price, _, _)) = self
                .calculate_optimal_pricing(&strategy, &congestion, &fee_history)
                .await
            {
                recommendations.insert(name.to_string(), gas_price);
            }
        }

        Ok(recommendations)
    }
}
