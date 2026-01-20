//! Transaction Cancellation Service
//!
//! This module provides comprehensive transaction cancellation functionality using Alloy.
//! It handles both Legacy and EIP-1559 transaction types with proper gas fee management.

use crate::gui::state::transaction_state::{PendingTransaction, TransactionType};
use crate::network::NetworkId;
use crate::wallet::Vaughan;
use alloy::primitives::{TxHash, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error types for transaction cancellation operations
#[derive(Debug, Clone)]
pub enum CancellationError {
    /// Transaction has already been confirmed/mined
    AlreadyConfirmed,
    /// Insufficient funds to pay for cancellation gas
    InsufficientFunds,
    /// Gas price too low for replacement (needs minimum 10% increase)
    GasPriceTooLow,
    /// Network/RPC connection issues
    NetworkError(String),
    /// Wallet signing failed
    WalletError(String),
    /// General cancellation error
    General(String),
}

impl std::fmt::Display for CancellationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancellationError::AlreadyConfirmed => write!(f, "Transaction already confirmed"),
            CancellationError::InsufficientFunds => write!(f, "Insufficient funds for gas"),
            CancellationError::GasPriceTooLow => write!(f, "Gas price too low for replacement"),
            CancellationError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            CancellationError::WalletError(msg) => write!(f, "Wallet error: {msg}"),
            CancellationError::General(msg) => write!(f, "Cancellation error: {msg}"),
        }
    }
}

impl std::error::Error for CancellationError {}

/// Gas settings for transaction cancellation (both Legacy and EIP-1559)
#[derive(Debug, Clone)]
pub struct GasSettings {
    pub gas_price: Option<U256>,                // For Legacy transactions
    pub max_fee_per_gas: Option<U256>,          // For EIP-1559 transactions
    pub max_priority_fee_per_gas: Option<U256>, // For EIP-1559 transactions
}

/// Transaction Cancellation Service using Alloy
pub struct TransactionCancellationService {
    provider_url: String,
    #[allow(dead_code)] // Reserved for future network-specific cancellation logic
    network_id: NetworkId,
}

impl TransactionCancellationService {
    /// Create a new cancellation service for the specified network
    pub fn new(provider_url: String, network_id: NetworkId) -> Self {
        Self {
            provider_url,
            network_id,
        }
    }

    /// Cancel a transaction by sending a 0 ETH replacement transaction with higher gas
    ///
    /// This is the battle-tested method used by MetaMask and other wallets.
    /// It works by sending a new transaction with the same nonce but higher gas price/fees.
    pub async fn cancel_transaction(
        &self,
        original_tx: &PendingTransaction,
        fee_multiplier: f64, // Usually 1.1 (10% increase) as required by nodes
        wallet: &Arc<RwLock<Vaughan>>,
    ) -> Result<TxHash, CancellationError> {
        // 1. Connect to the provider
        let provider = ProviderBuilder::new().connect_http(
            self.provider_url
                .parse()
                .map_err(|e| CancellationError::NetworkError(format!("Invalid RPC URL: {e}")))?,
        );

        // 2. Check if transaction is still pending
        if !self.is_cancellable_internal(&provider, &original_tx.tx_hash).await? {
            return Err(CancellationError::AlreadyConfirmed);
        }

        // 3. Calculate replacement gas settings
        let gas_settings = self
            .suggest_cancellation_gas_internal(&provider, original_tx, fee_multiplier)
            .await?;

        // 4. Validate user has sufficient balance for cancellation gas
        self.validate_cancellation_balance(wallet, &gas_settings).await?;

        // 5. Build replacement transaction (0 ETH to self)
        let mut cancel_tx = TransactionRequest::default()
            .from(original_tx.from)
            .to(original_tx.from) // Send to self
            .value(U256::ZERO) // 0 ETH value
            .nonce(original_tx.nonce) // Same nonce as original!
            .gas_limit(21000u64); // Standard ETH transfer gas limit

        // 6. Set appropriate gas pricing based on transaction type
        match original_tx.tx_type {
            TransactionType::Legacy => {
                if let Some(gas_price) = gas_settings.gas_price {
                    cancel_tx = cancel_tx.gas_price(gas_price.to::<u128>());
                } else {
                    return Err(CancellationError::General(
                        "No gas price for legacy transaction".to_string(),
                    ));
                }
            }
            TransactionType::Eip1559 => {
                if let (Some(max_fee), Some(priority_fee)) =
                    (gas_settings.max_fee_per_gas, gas_settings.max_priority_fee_per_gas)
                {
                    cancel_tx = cancel_tx
                        .max_fee_per_gas(max_fee.to::<u128>())
                        .max_priority_fee_per_gas(priority_fee.to::<u128>());
                } else {
                    return Err(CancellationError::General("Missing EIP-1559 gas settings".to_string()));
                }
            }
        }

        tracing::info!(
            "🚫 Cancel TX: Signing replacement transaction for {} with gas fees: {:?}",
            original_tx.tx_hash,
            gas_settings
        );

        // 7. Sign the replacement transaction with the wallet
        let signed_bytes = {
            let wallet_guard = wallet.read().await;
            wallet_guard.sign_transaction(&cancel_tx).await.map_err(|e| {
                tracing::error!("❌ Failed to sign cancellation transaction: {}", e);
                CancellationError::WalletError(format!("Failed to sign transaction: {e}"))
            })?
        };

        // 8. Submit the signed transaction to the network
        let pending_tx = provider.send_raw_transaction(&signed_bytes).await.map_err(|e| {
            tracing::error!("❌ Failed to broadcast cancellation transaction: {}", e);
            CancellationError::NetworkError(format!("Failed to broadcast transaction: {e}"))
        })?;

        // Wait for the transaction hash from the pending transaction
        let tx_hash = pending_tx.tx_hash();

        tracing::info!(
            "✅ Cancel TX: Successfully submitted replacement transaction {:?} for original {}",
            tx_hash,
            original_tx.tx_hash
        );

        Ok(*tx_hash)
    }

    /// Check if a transaction can still be cancelled
    pub async fn is_cancellable(&self, tx_hash: &str) -> Result<bool, CancellationError> {
        let provider = ProviderBuilder::new().connect_http(
            self.provider_url
                .parse()
                .map_err(|e| CancellationError::NetworkError(format!("Invalid RPC URL: {e}")))?,
        );

        self.is_cancellable_internal(&provider, tx_hash).await
    }

    /// Get suggested gas settings for cancelling a transaction
    pub async fn suggest_cancellation_gas(
        &self,
        original_tx: &PendingTransaction,
        fee_multiplier: f64,
    ) -> Result<GasSettings, CancellationError> {
        let provider = ProviderBuilder::new().connect_http(
            self.provider_url
                .parse()
                .map_err(|e| CancellationError::NetworkError(format!("Invalid RPC URL: {e}")))?,
        );

        self.suggest_cancellation_gas_internal(&provider, original_tx, fee_multiplier)
            .await
    }

    // Internal helper methods

    async fn is_cancellable_internal<P>(&self, provider: &P, tx_hash: &str) -> Result<bool, CancellationError>
    where
        P: Provider,
    {
        // Parse the transaction hash
        let hash: TxHash = tx_hash
            .parse()
            .map_err(|e| CancellationError::General(format!("Invalid transaction hash: {e}")))?;

        // Check if transaction has been mined
        match provider.get_transaction_receipt(hash).await {
            Ok(Some(_receipt)) => {
                // Transaction is confirmed, cannot cancel
                Ok(false)
            }
            Ok(None) => {
                // Transaction not found in a block, still pending
                Ok(true)
            }
            Err(e) => Err(CancellationError::NetworkError(format!(
                "Failed to check transaction status: {e}"
            ))),
        }
    }

    async fn suggest_cancellation_gas_internal<P>(
        &self,
        provider: &P,
        original_tx: &PendingTransaction,
        fee_multiplier: f64,
    ) -> Result<GasSettings, CancellationError>
    where
        P: Provider,
    {
        match original_tx.tx_type {
            TransactionType::Legacy => {
                let old_price = original_tx.gas_price.unwrap_or_default();

                // Get current gas price from network
                let current_price = provider
                    .get_gas_price()
                    .await
                    .map_err(|e| CancellationError::NetworkError(format!("Failed to get gas price: {e}")))?;

                // Calculate new gas price with minimum 10% increase
                let multiplier_price = old_price * U256::from((fee_multiplier * 100.0) as u128) / U256::from(100);
                let network_price = U256::from(current_price) * U256::from(105) / U256::from(100); // 5% above current

                let new_price = std::cmp::max(multiplier_price, network_price);

                Ok(GasSettings {
                    gas_price: Some(new_price),
                    max_fee_per_gas: None,
                    max_priority_fee_per_gas: None,
                })
            }
            TransactionType::Eip1559 => {
                let old_max_fee = original_tx.max_fee_per_gas.unwrap_or_default();
                let old_priority = original_tx.max_priority_fee_per_gas.unwrap_or_default();

                // Get current base fee (approximated by gas price)
                let current_gas_price = provider
                    .get_gas_price()
                    .await
                    .map_err(|e| CancellationError::NetworkError(format!("Failed to get gas price: {e}")))?;

                // Calculate new priority fee with minimum 10% increase
                let new_priority = old_priority * U256::from((fee_multiplier * 100.0) as u128) / U256::from(100);

                // Calculate new max fee ensuring it's at least base_fee + new_priority
                let min_max_fee = U256::from(current_gas_price) + new_priority;
                let multiplier_max_fee = old_max_fee * U256::from((fee_multiplier * 100.0) as u128) / U256::from(100);
                let new_max_fee = std::cmp::max(min_max_fee, multiplier_max_fee);

                Ok(GasSettings {
                    gas_price: None,
                    max_fee_per_gas: Some(new_max_fee),
                    max_priority_fee_per_gas: Some(new_priority),
                })
            }
        }
    }

    /// Validate that the user has sufficient balance to pay for cancellation gas
    async fn validate_cancellation_balance(
        &self,
        wallet: &Arc<RwLock<Vaughan>>,
        gas_settings: &GasSettings,
    ) -> Result<(), CancellationError> {
        // Calculate total gas cost
        let gas_limit = U256::from(21000u64); // Standard ETH transfer gas limit
        let total_gas_cost = match (&gas_settings.gas_price, &gas_settings.max_fee_per_gas) {
            // Legacy transaction
            (Some(gas_price), _) => gas_limit * gas_price,
            // EIP-1559 transaction
            (_, Some(max_fee)) => gas_limit * max_fee,
            _ => {
                return Err(CancellationError::General(
                    "No gas pricing information available".to_string(),
                ))
            }
        };

        // Get current ETH balance from wallet
        let wallet_guard = wallet.read().await;
        let current_balance = wallet_guard
            .get_balance(None)
            .await
            .map_err(|e| CancellationError::WalletError(format!("Failed to get balance: {e}")))?;

        // Check if balance is sufficient
        if current_balance < total_gas_cost {
            // Convert U256 to string then parse to f64 for display
            let balance_str = current_balance.to_string();
            let required_str = total_gas_cost.to_string();

            let balance_eth = balance_str.parse::<f64>().unwrap_or(0.0) / 1_000_000_000_000_000_000.0;
            let required_eth = required_str.parse::<f64>().unwrap_or(0.0) / 1_000_000_000_000_000_000.0;

            tracing::warn!(
                "❌ Insufficient balance for cancellation: have {:.6} ETH, need {:.6} ETH",
                balance_eth,
                required_eth
            );

            return Err(CancellationError::InsufficientFunds);
        }

        // Convert for display purposes
        let balance_str = current_balance.to_string();
        let required_str = total_gas_cost.to_string();
        let balance_eth = balance_str.parse::<f64>().unwrap_or(0.0) / 1_000_000_000_000_000_000.0;
        let required_eth = required_str.parse::<f64>().unwrap_or(0.0) / 1_000_000_000_000_000_000.0;

        tracing::info!(
            "✅ Balance validation passed: have {:.6} ETH, need {:.6} ETH for cancellation",
            balance_eth,
            required_eth
        );

        Ok(())
    }
}

/// Progress callback type for cancellation steps
pub type ProgressCallback = Box<dyn Fn(crate::gui::state::transaction_state::CancellationProgress) + Send + Sync>;

/// Execute complete transaction cancellation process with progress feedback
///
/// This is the main entry point for cancelling transactions from the UI.
/// It includes all necessary validation and error handling with step-by-step progress.
pub async fn execute_cancellation_with_progress(
    tx_to_cancel: PendingTransaction, // Take ownership instead of borrowing
    wallet: Arc<RwLock<Vaughan>>,     // Take ownership instead of borrowing
    network: NetworkId,
    provider_url: String,
    progress_callback: Option<ProgressCallback>,
) -> Result<String, String> {
    let service = TransactionCancellationService::new(provider_url.clone(), network);

    // Helper to send progress updates
    let send_progress = |step: crate::gui::state::transaction_state::CancellationProgress| {
        if let Some(ref callback) = progress_callback {
            callback(step);
        }
    };

    // Step 1: Validate transaction
    send_progress(crate::gui::state::transaction_state::CancellationProgress::ValidatingTransaction);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await; // Brief pause for UI feedback

    // Connect to provider
    let _provider = alloy::providers::ProviderBuilder::new()
        .connect_http(provider_url.parse().map_err(|e| format!("Invalid RPC URL: {e}"))?);

    // Check if transaction is still cancellable
    if !service
        .is_cancellable(&tx_to_cancel.tx_hash)
        .await
        .map_err(|e| e.to_string())?
    {
        return Err("Transaction has already been confirmed and cannot be cancelled".to_string());
    }

    // Step 2: Calculate gas settings
    send_progress(crate::gui::state::transaction_state::CancellationProgress::CalculatingGasSettings);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let gas_settings = service
        .suggest_cancellation_gas(&tx_to_cancel, 1.10)
        .await
        .map_err(|e| e.to_string())?;

    // Step 3: Check balance
    send_progress(crate::gui::state::transaction_state::CancellationProgress::CheckingBalance);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    service
        .validate_cancellation_balance(&wallet, &gas_settings)
        .await
        .map_err(|e| e.to_string())?;

    // Step 4: Sign replacement transaction
    send_progress(crate::gui::state::transaction_state::CancellationProgress::SigningReplacement);
    tokio::time::sleep(std::time::Duration::from_millis(800)).await; // Longer pause for signing

    // Execute the cancellation with 10% fee increase (minimum required)
    let tx_hash = service
        .cancel_transaction(&tx_to_cancel, 1.10, &wallet)
        .await
        .map_err(|e| e.to_string())?; // Convert CancellationError to String

    // Step 5: Broadcasting replacement
    send_progress(crate::gui::state::transaction_state::CancellationProgress::BroadcastingReplacement);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 6: Waiting for confirmation
    send_progress(crate::gui::state::transaction_state::CancellationProgress::WaitingConfirmation);
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Return the cancellation transaction hash
    Ok(format!("{tx_hash:?}"))
}

/// Execute complete transaction cancellation process (legacy wrapper)
///
/// This is the main entry point for cancelling transactions from the UI.
/// It includes all necessary validation and error handling.
pub async fn execute_cancellation(
    tx_to_cancel: PendingTransaction, // Take ownership instead of borrowing
    wallet: Arc<RwLock<Vaughan>>,     // Take ownership instead of borrowing
    network: NetworkId,
    provider_url: String,
) -> Result<String, String> {
    execute_cancellation_with_progress(tx_to_cancel, wallet, network, provider_url, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::time::Instant;

    fn create_test_pending_tx() -> PendingTransaction {
        PendingTransaction {
            tx_hash: "0x1234567890abcdef".to_string(),
            nonce: 42,
            from: Address::ZERO,
            to: Some(Address::ZERO),
            value: U256::from(1000000000000000000u64), // 1 ETH
            tx_type: TransactionType::Legacy,
            gas_limit: 21000,
            gas_price: Some(U256::from(20000000000u64)), // 20 gwei
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            timestamp: Instant::now(),
            network: NetworkId(1), // Ethereum mainnet
            cancellable: true,
        }
    }

    #[test]
    fn test_gas_settings_creation() {
        let gas_settings = GasSettings {
            gas_price: Some(U256::from(25000000000u64)), // 25 gwei
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        };

        assert!(gas_settings.gas_price.is_some());
        assert_eq!(gas_settings.gas_price.unwrap(), U256::from(25000000000u64));
    }

    #[test]
    fn test_cancellation_error_display() {
        let error = CancellationError::AlreadyConfirmed;
        assert_eq!(error.to_string(), "Transaction already confirmed");

        let error = CancellationError::NetworkError("Connection failed".to_string());
        assert_eq!(error.to_string(), "Network error: Connection failed");
    }

    #[test]
    fn test_pending_transaction_creation() {
        let tx = create_test_pending_tx();
        assert_eq!(tx.nonce, 42);
        assert_eq!(tx.gas_limit, 21000);
        assert!(matches!(tx.tx_type, TransactionType::Legacy));
    }
}
