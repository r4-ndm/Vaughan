//! Transaction Service Module
//!
//! This module handles all transaction-related business logic including
//! transaction history loading, caching, and transaction submission.
//!
//! Consolidated from transaction_service.rs and transaction_service_extended.rs

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::gui::services::explorer_service::fetch_from_block_explorer;
use crate::gui::state::transaction_state::{PendingTransaction, TransactionType};
use crate::gui::Transaction;
use crate::network::NetworkId;
use alloy::primitives::{Address, U256};

/// Cache for transaction history to avoid repeated API calls
static TRANSACTION_CACHE: OnceLock<Mutex<HashMap<String, (std::time::Instant, Vec<Transaction>)>>> = OnceLock::new();
const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(300); // 5 minutes

/// Load transaction history for a given network and address with caching
pub async fn load_transaction_history(network: NetworkId, address: String) -> Result<Vec<Transaction>, String> {
    tracing::info!(
        "📊 Loading transaction history for {} on network {}",
        address,
        network.0
    );

    // Parse the address string to get the actual address
    let address = if address.starts_with("0x") {
        address
    } else {
        format!("0x{address}")
    };

    // Create cache key - avoid extra allocation for cache key
    let cache_key = format!("{}_{}", network.0, address);

    // Check cache first
    let cache_mutex = TRANSACTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = cache_mutex.lock() {
        if let Some((cached_time, cached_transactions)) = cache.get(&cache_key) {
            if cached_time.elapsed() < CACHE_DURATION {
                tracing::info!(
                    "🔄 Using cached transaction history ({} transactions)",
                    cached_transactions.len()
                );
                return Ok(cached_transactions.clone());
            }
        }
    }

    // Cache miss or expired, fetch from API
    tracing::info!("🌐 Cache miss/expired, fetching fresh transaction history");
    match fetch_from_block_explorer(network, &address).await {
        Ok(txs) => {
            tracing::info!("✅ Fetched {} transactions from block explorer", txs.len());

            // Update cache
            let cache_mutex = TRANSACTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
            if let Ok(mut cache) = cache_mutex.lock() {
                cache.insert(cache_key, (std::time::Instant::now(), txs.clone()));
            }

            Ok(txs)
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to fetch transaction history from block explorer: {}", e);

            // Try to return cached data even if expired
            let cache_mutex = TRANSACTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
            if let Ok(cache) = cache_mutex.lock() {
                if let Some((_, cached_transactions)) = cache.get(&cache_key) {
                    tracing::info!("🔄 Using stale cached data as fallback");
                    return Ok(cached_transactions.clone());
                }
            }

            tracing::info!("📝 API unavailable, returning empty transaction list");
            // Return empty list when APIs are unavailable - no fake data
            Ok(Vec::new())
        }
    }
}

/// Check for incoming transactions for a wallet address
pub async fn check_for_incoming_transactions(
    network: NetworkId,
    address: String,
    _current_tx_count: usize,
    _token_name: String,
) -> Result<Vec<Transaction>, String> {
    tracing::info!(
        "📥 Checking for incoming transactions for {} on network {}",
        address,
        network.0
    );

    // Parse the address string to get the actual address
    let address = if address.starts_with("0x") {
        address.clone()
    } else {
        format!("0x{address}")
    };

    tracing::info!("🔍 Final address for API call: {}", address);

    // Try to fetch recent transactions from block explorer API
    match fetch_from_block_explorer(network, &address).await {
        Ok(all_transactions) => {
            tracing::info!("📊 Fetched {} total transactions from API", all_transactions.len());

            // Log first few transactions for debugging
            for (i, tx) in all_transactions.iter().take(3).enumerate() {
                tracing::info!("🔎 Transaction #{}: {} -> {} ({})", i + 1, tx.from, tx.to, tx.amount);
            }

            // Filter for incoming transactions only (where 'to' field matches our address)
            let incoming_transactions: Vec<Transaction> = all_transactions
                .into_iter()
                .filter(|tx| {
                    let matches = tx.to.to_lowercase() == address.to_lowercase();
                    if matches {
                        tracing::info!("✓ Incoming transaction found: {} from {}", tx.amount, tx.from);
                    }
                    matches
                })
                .take(5) // Only get the 5 most recent incoming transactions
                .collect();

            tracing::info!(
                "✅ Found {} incoming transactions after filtering",
                incoming_transactions.len()
            );
            Ok(incoming_transactions)
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to check for incoming transactions: {}", e);
            // Return empty vec instead of error to avoid disrupting the UI
            Ok(Vec::new())
        }
    }
}

/// Create a pending transaction for tracking cancellation
#[allow(clippy::too_many_arguments)]
pub fn create_pending_transaction(
    tx_hash: String,
    nonce: u64,
    from: Address,
    to: Option<Address>,
    value: U256,
    tx_type: TransactionType,
    gas_limit: u64,
    gas_price: Option<U256>,
    max_fee_per_gas: Option<U256>,
    max_priority_fee_per_gas: Option<U256>,
    network: NetworkId,
) -> PendingTransaction {
    PendingTransaction {
        tx_hash,
        nonce,
        from,
        to,
        value,
        tx_type,
        gas_limit,
        gas_price,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        network,
        timestamp: std::time::Instant::now(),
        cancellable: true,
    }
}
