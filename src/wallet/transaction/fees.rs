
//! EIP-1559 Fee Estimation Module
//!
//! Provides utilities for estimating gas fees using EIP-1559 (London Hardfork) 
//! standards with fallback for legacy networks.
//!
//! # Features
//!
//! - **Priority Levels**: Slow, Standard, Fast
//! - **EIP-1559 Support**: Calculates `max_fee_per_gas` and `max_priority_fee_per_gas`
//! - **History Analysis**: Uses `eth_feeHistory` for accurate trends (TODO)
//! - **Legacy Fallback**: Supports `gas_price` for older networks

use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use alloy::rpc::types::BlockNumberOrTag;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::error::{Result, NetworkError, VaughanError};

/// Priority level for fee estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeePriority {
    /// Low priority, slower inclusion (10th percentile)
    Slow,
    /// Standard priority, average inclusion (50th percentile)
    Standard,
    /// High priority, fast inclusion (90th percentile)
    Fast,
    /// Custom fee settings
    Custom {
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
    },
}

/// Result of a fee estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    /// Maximum fee per gas (EIP-1559) or Legacy Gas Price
    pub max_fee_per_gas: U256,
    /// Maximum priority fee per gas (EIP-1559 only)
    pub max_priority_fee_per_gas: U256,
    /// Estimated wait time (seconds) - optional/simulated
    pub estimated_seconds: u64,
}

/// Fee Estimator Service
pub struct FeeEstimator;

impl FeeEstimator {
    /// Estimate EIP-1559 fees
    /// 
    /// improved logic from metamask:
    /// 1. Get base fee from latest block
    /// 2. Get priority fee estimate from provider
    /// 3. Apply multipliers based on priority
    pub async fn estimate_eip1559_fees<P, N>(
        provider: &P,
        priority: FeePriority,
    ) -> Result<FeeEstimate>
    where
        P: Provider<N>,
        N: Network,
    {
        let correlation_id = Uuid::new_v4();
        tracing::debug!(%correlation_id, ?priority, "Estimating EIP-1559 fees");

        // Handle Custom priority early
        if let FeePriority::Custom { max_fee_per_gas, max_priority_fee_per_gas } = priority {
             return Ok(FeeEstimate {
                max_fee_per_gas,
                max_priority_fee_per_gas,
                estimated_seconds: 0,
            });
        }

        // Get suggested gas fees from provider (usually gives average/standard)
        let fees = provider.estimate_eip1559_fees().await
            .map_err(|e| VaughanError::Network(NetworkError::RpcError { message: e.to_string() }))?;

        let base_fee = U256::from(fees.max_fee_per_gas.saturating_sub(fees.max_priority_fee_per_gas));
        let priority_fee = U256::from(fees.max_priority_fee_per_gas);

        let (max_fee, adjusted_priority, seconds) = Self::calculate_fee_parameters(base_fee, priority_fee, priority);

        tracing::info!(
            %correlation_id,
            ?priority,
            base_fee = %base_fee,
            adjusted_priority = %adjusted_priority,
            max_fee = %max_fee,
            "Fee estimation calculated"
        );

        Ok(FeeEstimate {
            max_fee_per_gas: max_fee,
            max_priority_fee_per_gas: adjusted_priority,
            estimated_seconds: seconds,
        })
    }

    /// Pure function to calculate fee parameters based on base fee, priority fee, and user priority
    pub fn calculate_fee_parameters(
        base_fee: U256,
        priority_fee: U256,
        priority: FeePriority,
    ) -> (U256, U256, u64) {
        // Multipliers based on priority (MetaMask-style heuristic)
        // Standard is roughly what the provider returns.
        // Slow: 0.8x priority
        // Fast: 1.2x priority + 1.1x base fee buffer
        
        let (prio_mult, base_mult, seconds) = match priority {
            FeePriority::Slow => (80, 100, 60),    // 0.8x prio, 1.0x base
            FeePriority::Standard => (100, 100, 12), // 1.0x, 1.0x
            FeePriority::Fast => (120, 110, 3),    // 1.2x prio, 1.1x base (buffer for next block surge)
            FeePriority::Custom { max_fee_per_gas, max_priority_fee_per_gas } => {
                return (max_fee_per_gas, max_priority_fee_per_gas, 0);
            }
        };

        let adjusted_priority = priority_fee
            .checked_mul(U256::from(prio_mult)).unwrap_or(priority_fee)
            .checked_div(U256::from(100)).unwrap_or(priority_fee);
            
        // Calculate max_fee = (base_fee * base_mult) + adjusted_priority
        // We buffer base_fee for fast transactions to ensure inclusion even if base fee rises
        let adjusted_base = base_fee
             .checked_mul(U256::from(base_mult)).unwrap_or(base_fee)
             .checked_div(U256::from(100)).unwrap_or(base_fee);
             
        let max_fee = adjusted_base.checked_add(adjusted_priority).unwrap_or(U256::MAX);
        
        (max_fee, adjusted_priority, seconds)
    }

    /// Estimate Legacy fees (Gas Price)
    pub async fn estimate_legacy_fees<P, N>(
        provider: &P,
        priority: FeePriority,
    ) -> Result<FeeEstimate>
    where
        P: Provider<N>,
        N: Network,
    {
        let correlation_id = Uuid::new_v4();
        tracing::debug!(%correlation_id, ?priority, "Estimating Legacy fees");
        
        // Handle Custom priority
        if let FeePriority::Custom { max_fee_per_gas, .. } = priority {
             return Ok(FeeEstimate {
                max_fee_per_gas,
                max_priority_fee_per_gas: max_fee_per_gas, // Legacy txs effectively use same price
                estimated_seconds: 0,
            });
        }

        // Convert u128 directly to U256 if needed, depends on alloy version
        let gas_price_u128 = provider.get_gas_price().await
             .map_err(|e| VaughanError::Network(NetworkError::RpcError { message: e.to_string() }))?;
        let gas_price = U256::from(gas_price_u128);

        // Multipliers
        let mult = match priority {
            FeePriority::Slow => 90,
            FeePriority::Standard => 100,
            FeePriority::Fast => 110,
            FeePriority::Custom { .. } => unreachable!(),
        };

        let adjusted_price = gas_price
            .checked_mul(U256::from(mult)).unwrap_or(gas_price)
            .checked_div(U256::from(100)).unwrap_or(gas_price);

        Ok(FeeEstimate {
            max_fee_per_gas: adjusted_price,
            max_priority_fee_per_gas: adjusted_price,
            estimated_seconds: match priority {
                FeePriority::Slow => 60,
                FeePriority::Standard => 12,
                FeePriority::Fast => 3,
                _ => 0,
            },
        })
    }

    /// Estimate fees (auto-detects EIP-1559 support)
    pub async fn estimate_fees<P, N>(
        provider: &P,
        priority: FeePriority,
    ) -> Result<FeeEstimate>
    where
        P: Provider<N>,
        N: Network,
    {
        // Try EIP-1559 first, fall back to legacy if it fails or if not supported
        // Note: A more robust check would be to check the chain ID or block header, 
        // but trying the call is a reliable feature detection.
        match Self::estimate_eip1559_fees(provider, priority).await {
            Ok(estimate) => {
                tracing::debug!("Using EIP-1559 fee estimation");
                Ok(estimate)
            },
            Err(_) => {
                tracing::warn!("EIP-1559 estimation failed, falling back to legacy gas price");
                Self::estimate_legacy_fees(provider, priority).await
            }
        }
    }

    /// Analyze fee history for trend detection
    /// Performs a simple 10-block history check to see if base fee is trending up or down.
    pub async fn analyze_fee_history<P, N>(
        provider: &P,
    ) -> Result<FeeHistoryAnalysis>
    where
        P: Provider<N>,
        N: Network,
    {
        let correlation_id = Uuid::new_v4();
        tracing::debug!(%correlation_id, "Analyzing fee history");

        // Request 10 blocks history, 25th, 50th, 75th percentiles
        let percentiles = &[25.0, 50.0, 75.0];
        let history = provider.get_fee_history(10, BlockNumberOrTag::Latest, percentiles)
            .await
            .map_err(|e| VaughanError::Network(NetworkError::RpcError { message: e.to_string() }))?;

        // Base fees usually has count + 1 items (including next block)
        let base_fees = &history.base_fee_per_gas;
        
        if base_fees.len() < 2 {
             return Ok(FeeHistoryAnalysis {
                trend: FeeTrend::Stable,
                suggested_base_fee: base_fees.first().cloned().map(U256::from).unwrap_or(U256::ZERO),
            });
        }

        // Compare oldest available vs newest (next block base fee)
        let oldest = U256::from(*base_fees.first().unwrap());
        let newest = U256::from(*base_fees.last().unwrap());

        // Simple trend analysis
        // Rising: Newest > Oldest + 10%
        // Falling: Oldest > Newest + 10%
        // Stable: Otherwise
        
        // 10% threshold calculation
        let threshold_rising = oldest.saturating_add(oldest / U256::from(10));
        let threshold_falling = newest.saturating_add(newest / U256::from(10));

        let trend = if newest > threshold_rising {
            FeeTrend::Rising
        } else if oldest > threshold_falling {
            FeeTrend::Falling
        } else {
            FeeTrend::Stable
        };

        // Use the projected next block base fee as suggested
        let suggested_base_fee = newest;

        tracing::info!(
            %correlation_id, 
            ?trend, 
            %suggested_base_fee, 
            oldest = %oldest,
            newest = %newest,
            "Fee history analyzed"
        );

        Ok(FeeHistoryAnalysis {
            trend,
            suggested_base_fee,
        })
    }
}

/// Analysis result of fee history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeHistoryAnalysis {
    pub trend: FeeTrend,
    pub suggested_base_fee: U256,
}

/// Fee market trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeeTrend {
    Rising,
    Falling,
    Stable,
}

#[cfg(test)]
#[path = "fees_tests.rs"]
mod tests;
