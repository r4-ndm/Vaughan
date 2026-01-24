//! Integrated Account Service
//!
//! This module provides a unified service layer that integrates:
//! - AccountManagerTrait for account operations
//! - BatchProcessor for efficient balance queries
//! - AccountTelemetry for structured logging with correlation tracking
//!
//! This service serves as the primary entry point for all account operations
//! in the GUI, ensuring consistent telemetry and error handling.

// VaughanError removed as unused
// Message removed as unused
use crate::performance::batch::{BatchConfig, BatchProcessor};
// Re-export Result for convenience
use crate::error::Result;
use crate::security::SecureAccount;
use crate::telemetry::account_events::{AccountTelemetry, OperationSpan};
use alloy::primitives::{Address, U256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Integrated account service combining account management, batch processing, and telemetry.
///
/// This service provides a unified interface for all account operations with:
/// - Automatic correlation tracking for debugging
/// - Batch balance queries for multi-account portfolios
/// - Structured logging for observability
///
/// # Design Inspiration
/// - **MetaMask**: Account management patterns and balance refresh strategies
/// - **Alloy**: RPC batching and provider patterns
#[derive(Debug)]
pub struct IntegratedAccountService {
    /// Batch processor for efficient RPC operations
    batch_processor: BatchProcessor,
    /// Telemetry service for structured logging
    telemetry: AccountTelemetry,
    /// Cached balances for quick access
    balance_cache: Arc<RwLock<HashMap<Address, CachedBalance>>>,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

/// Cached balance with timestamp
#[derive(Debug, Clone)]
pub struct CachedBalance {
    pub balance: U256,
    pub cached_at: std::time::Instant,
}

impl CachedBalance {
    pub fn new(balance: U256) -> Self {
        Self {
            balance,
            cached_at: std::time::Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl_secs: u64) -> bool {
        self.cached_at.elapsed().as_secs() > ttl_secs
    }
}

impl Default for IntegratedAccountService {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegratedAccountService {
    /// Create a new integrated account service with default configuration
    pub fn new() -> Self {
        Self {
            batch_processor: BatchProcessor::with_defaults(),
            telemetry: AccountTelemetry::new(),
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_secs: 30, // 30 second cache TTL
        }
    }

    /// Create a service with custom batch configuration
    pub fn with_config(batch_config: BatchConfig) -> Self {
        Self {
            batch_processor: BatchProcessor::new(batch_config),
            telemetry: AccountTelemetry::new(),
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_secs: 30,
        }
    }

    /// Create a span for tracking an operation
    pub fn start_operation(&self, operation: &str) -> OperationSpan {
        let span = OperationSpan::new(operation);
        self.telemetry.record_event(&span, "started", "");
        span
    }

    /// Record successful completion of an operation
    pub fn complete_operation(&self, span: &OperationSpan, details: &str) {
        self.telemetry.record_event(span, "completed", details);
        info!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            elapsed_ms = span.elapsed_ms(),
            "Operation completed successfully"
        );
    }

    /// Record operation failure
    pub fn fail_operation(&self, span: &OperationSpan, error: &str) {
        self.telemetry.record_error(span, error);
        error!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            elapsed_ms = span.elapsed_ms(),
            error = %error,
            "Operation failed"
        );
    }

    /// Get cached balance for an address if not expired
    pub async fn get_cached_balance(&self, address: &Address) -> Option<U256> {
        let cache = self.balance_cache.read().await;
        cache.get(address).and_then(|cached| {
            if cached.is_expired(self.cache_ttl_secs) {
                None
            } else {
                Some(cached.balance)
            }
        })
    }

    /// Update balance cache
    pub async fn update_balance_cache(&self, address: Address, balance: U256) {
        let mut cache = self.balance_cache.write().await;
        cache.insert(address, CachedBalance::new(balance));
    }

    /// Clear all cached balances
    pub async fn clear_balance_cache(&self) {
        let mut cache = self.balance_cache.write().await;
        cache.clear();
    }

    /// Refresh balances for multiple accounts using batch processing
    ///
    /// This method implements Requirement 6.1 (Batch RPC) by using the BatchProcessor
    /// to query multiple balances concurrently with controlled parallelism.
    ///
    /// # Arguments
    /// * `accounts` - List of accounts to refresh balances for
    /// * `fetch_fn` - Function to fetch balance for a single address
    ///
    /// # Returns
    /// A map of address to balance for all successfully queried accounts
    pub async fn refresh_account_balances<F, Fut>(
        &self,
        accounts: &[SecureAccount],
        fetch_fn: F,
    ) -> Result<HashMap<Address, U256>>
    where
        F: Fn(Address) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<U256>> + Send + 'static,
    {
        let span = self.start_operation("refresh_account_balances");
        
        if accounts.is_empty() {
            self.complete_operation(&span, "no accounts to refresh");
            return Ok(HashMap::new());
        }

        let addresses: Vec<Address> = accounts.iter().map(|a| a.address).collect();
        
        info!(
            correlation_id = %span.correlation_id,
            account_count = accounts.len(),
            "Starting batch balance refresh"
        );

        let batch_result = self.batch_processor
            .batch_balance_queries(addresses, fetch_fn)
            .await
            .map_err(|e| {
                self.fail_operation(&span, &e.to_string());
                e
            })?;

        // Convert results to map and update cache
        let balance_map = BatchProcessor::results_to_map(&batch_result.results);
        
        // Update cache for all successful results
        for (address, balance) in &balance_map {
            self.update_balance_cache(*address, *balance).await;
        }

        self.complete_operation(
            &span,
            &format!(
                "refreshed {} balances ({} succeeded, {} failed)",
                batch_result.results.len(),
                batch_result.success_count,
                batch_result.failure_count
            ),
        );

        Ok(balance_map)
    }

    /// Get balance for a single account, using cache if available
    pub async fn get_account_balance<F, Fut>(
        &self,
        account: &SecureAccount,
        fetch_fn: F,
    ) -> Result<U256>
    where
        F: FnOnce(Address) -> Fut,
        Fut: std::future::Future<Output = Result<U256>>,
    {
        // Check cache first
        if let Some(cached_balance) = self.get_cached_balance(&account.address).await {
            return Ok(cached_balance);
        }

        let span = self.start_operation("get_account_balance");
        
        let balance = fetch_fn(account.address)
            .await
            .map_err(|e| {
                self.fail_operation(&span, &e.to_string());
                e
            })?;

        self.update_balance_cache(account.address, balance).await;
        self.complete_operation(&span, &format!("balance: {}", balance));

        Ok(balance)
    }

    /// Instrument an account creation operation with telemetry
    pub fn instrument_create_account<T, E: std::fmt::Display>(
        &self,
        result: std::result::Result<T, E>,
        account_name: &str,
    ) -> std::result::Result<T, E> {
        let span = OperationSpan::new("create_account")
            .with_component("account_service");
        
        match &result {
            Ok(_) => {
                self.telemetry.record_event(
                    &span,
                    "account_created",
                    &format!("name: {}", account_name),
                );
            }
            Err(e) => {
                self.telemetry.record_error(&span, &e.to_string());
            }
        }
        
        result
    }

    /// Instrument an account import operation with telemetry
    pub fn instrument_import_account<T, E: std::fmt::Display>(
        &self,
        result: std::result::Result<T, E>,
        import_type: &str,
    ) -> std::result::Result<T, E> {
        let span = OperationSpan::new("import_account")
            .with_component("account_service");
        
        match &result {
            Ok(_) => {
                self.telemetry.record_event(
                    &span,
                    "account_imported",
                    &format!("type: {}", import_type),
                );
            }
            Err(e) => {
                self.telemetry.record_error(&span, &e.to_string());
            }
        }
        
        result
    }

    /// Instrument an account export operation with telemetry
    pub fn instrument_export_account<T, E: std::fmt::Display>(
        &self,
        result: std::result::Result<T, E>,
        export_type: &str,
    ) -> std::result::Result<T, E> {
        let span = OperationSpan::new("export_account")
            .with_component("account_service");
        
        match &result {
            Ok(_) => {
                self.telemetry.record_event(
                    &span,
                    "account_exported",
                    &format!("type: {}", export_type),
                );
            }
            Err(_) => {
                // Don't log the actual error for exports (could contain sensitive data)
                self.telemetry.record_error(&span, "export failed");
            }
        }
        
        result
    }

    /// Instrument a lock operation with telemetry
    pub fn instrument_lock(&self) {
        let span = OperationSpan::new("lock_wallet")
            .with_component("security");
        self.telemetry.record_event(&span, "wallet_locked", "");
        info!(correlation_id = %span.correlation_id, "Wallet locked");
    }

    /// Instrument an unlock operation with telemetry
    pub fn instrument_unlock(&self, success: bool) {
        let span = OperationSpan::new("unlock_wallet")
            .with_component("security");
        
        if success {
            self.telemetry.record_event(&span, "wallet_unlocked", "");
            info!(correlation_id = %span.correlation_id, "Wallet unlocked successfully");
        } else {
            self.telemetry.record_error(&span, "unlock failed - incorrect password");
            warn!(correlation_id = %span.correlation_id, "Wallet unlock failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = IntegratedAccountService::new();
        assert_eq!(service.cache_ttl_secs, 30);
    }

    #[tokio::test]
    async fn test_balance_caching() {
        let service = IntegratedAccountService::new();
        let address = Address::ZERO;
        let balance = U256::from(1000u64);

        // Initially no cached balance
        assert!(service.get_cached_balance(&address).await.is_none());

        // Cache the balance
        service.update_balance_cache(address, balance).await;

        // Now should be cached
        let cached = service.get_cached_balance(&address).await;
        assert_eq!(cached, Some(balance));

        // Clear cache
        service.clear_balance_cache().await;
        assert!(service.get_cached_balance(&address).await.is_none());
    }

    #[test]
    fn test_operation_span_creation() {
        let service = IntegratedAccountService::new();
        let span = service.start_operation("test_operation");
        assert_eq!(span.operation, "test_operation");
        assert!(!span.correlation_id.is_nil());
    }
}
