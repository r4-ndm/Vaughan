//! Batch Processor Implementation
//!
//! Core batch processing logic with concurrency limiting, retry logic,
//! and exponential backoff.

use alloy::primitives::{Address, U256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::error::Result;

use super::config::BatchConfig;
use super::results::{BalanceResult, BatchResult};

/// Batch processor for efficient multi-account operations
///
/// Uses Alloy providers for all blockchain operations with:
/// - Semaphore-based concurrency limiting (Property 12)
/// - Exponential backoff retry logic (Property 15)
/// - Correlation tracking for all operations
/// - Graceful partial failure handling (Property 13)
///
/// # Requirements Addressed
///
/// - **Requirement 6.1**: Batch RPC calls using Alloy's provider capabilities
/// - **Requirement 6.2**: Concurrency limiting with semaphore-based control
/// - **Requirement 6.3**: Graceful handling of partial failures
/// - **Requirement 6.5**: Retry logic with exponential backoff
///
/// # Example
///
/// ```ignore
/// use vaughan::performance::batch::{BatchProcessor, BatchConfig};
///
/// let processor = BatchProcessor::new(BatchConfig::default());
/// let balances = processor.batch_balance_queries(addresses, fetch_fn).await?;
/// ```
#[derive(Debug)]
pub struct BatchProcessor {
    /// Configuration for batch operations
    config: BatchConfig,
    /// Semaphore for concurrency limiting
    semaphore: Arc<Semaphore>,
}

impl BatchProcessor {
    /// Create a new BatchProcessor with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Batch processing configuration
    pub fn new(config: BatchConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        tracing::info!(
            max_concurrent = config.max_concurrent,
            max_retries = config.max_retries,
            base_delay_ms = config.base_delay_ms,
            "🚀 Creating BatchProcessor"
        );

        Self { config, semaphore }
    }

    /// Create a BatchProcessor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BatchConfig::default())
    }

    /// Get the current configuration
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }

    /// Execute a batch of balance queries using the provided fetch function
    ///
    /// This method implements Property 11 (Batch RPC Efficiency) by processing
    /// multiple addresses concurrently with controlled parallelism.
    ///
    /// # Arguments
    ///
    /// * `addresses` - List of addresses to query
    /// * `fetch_fn` - Async function to fetch balance for a single address
    ///
    /// # Returns
    ///
    /// A BatchResult containing all balance results and metadata
    ///
    /// # Example
    ///
    /// ```ignore
    /// let addresses = vec![addr1, addr2, addr3];
    /// let result = processor.batch_balance_queries(addresses, |addr| async move {
    ///     provider.get_balance(addr).await
    /// }).await?;
    /// ```
    pub async fn batch_balance_queries<F, Fut>(
        &self,
        addresses: Vec<Address>,
        fetch_fn: F,
    ) -> Result<BatchResult<BalanceResult>>
    where
        F: Fn(Address) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<U256>> + Send,
    {
        let batch_correlation_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();
        let total_addresses = addresses.len();

        tracing::info!(
            correlation_id = %batch_correlation_id,
            address_count = total_addresses,
            max_concurrent = self.config.max_concurrent,
            "📦 Starting batch balance queries"
        );

        if addresses.is_empty() {
            return Ok(BatchResult::new(Vec::new(), 0, 0, batch_correlation_id, 0));
        }

        // Create futures for all addresses with semaphore-based concurrency limiting
        let futures: Vec<_> = addresses
            .into_iter()
            .map(|address| {
                let semaphore = Arc::clone(&self.semaphore);
                let fetch = fetch_fn.clone();
                let config = self.config.clone();
                let query_correlation_id = Uuid::new_v4();

                async move {
                    // Acquire semaphore permit (Property 12: Concurrency Limiting)
                    let _permit = semaphore
                        .acquire()
                        .await
                        .map_err(|e| {
                            tracing::error!(
                                correlation_id = %query_correlation_id,
                                error = %e,
                                "❌ Failed to acquire semaphore"
                            );
                            e
                        })
                        .ok();

                    // Execute with retry logic (Property 15: Retry with Backoff)
                    Self::execute_with_retry(
                        address,
                        fetch,
                        config.max_retries,
                        config.base_delay_ms,
                        config.max_delay_ms,
                        query_correlation_id,
                    )
                    .await
                }
            })
            .collect();

        // Execute all futures concurrently
        let results: Vec<BalanceResult> = futures_util::future::join_all(futures).await;

        // Count successes and failures
        let success_count = results.iter().filter(|r| r.is_success()).count();
        let failure_count = results.len() - success_count;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            correlation_id = %batch_correlation_id,
            total = total_addresses,
            success = success_count,
            failure = failure_count,
            duration_ms = duration_ms,
            "✅ Batch balance queries completed"
        );

        // Property 13: Partial failures are handled gracefully
        if failure_count > 0 {
            tracing::warn!(
                correlation_id = %batch_correlation_id,
                failed_count = failure_count,
                "⚠️ Some balance queries failed, returning partial results"
            );
        }

        Ok(BatchResult::new(
            results,
            success_count,
            failure_count,
            batch_correlation_id,
            duration_ms,
        ))
    }

    /// Execute a single query with retry logic and exponential backoff
    ///
    /// Implements Property 15: Retry with Backoff
    ///
    /// # Arguments
    ///
    /// * `address` - Address to query
    /// * `fetch_fn` - Function to fetch the balance
    /// * `max_retries` - Maximum number of retry attempts
    /// * `base_delay_ms` - Base delay for exponential backoff
    /// * `max_delay_ms` - Maximum delay cap
    /// * `correlation_id` - Correlation ID for tracking
    async fn execute_with_retry<F, Fut>(
        address: Address,
        fetch_fn: F,
        max_retries: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
        correlation_id: Uuid,
    ) -> BalanceResult
    where
        F: Fn(Address) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<U256>> + Send,
    {
        let mut attempts = 0;

        loop {
            match fetch_fn(address).await {
                Ok(balance) => {
                    tracing::debug!(
                        correlation_id = %correlation_id,
                        address = %address,
                        balance = %balance,
                        attempts = attempts + 1,
                        "✅ Balance fetched successfully"
                    );
                    return BalanceResult::success(address, balance, correlation_id);
                }
                Err(e) => {
                    attempts += 1;

                    if attempts >= max_retries {
                        tracing::warn!(
                            correlation_id = %correlation_id,
                            address = %address,
                            attempts = attempts,
                            error = %e,
                            "❌ Max retries exceeded"
                        );
                        return BalanceResult::failure(
                            address,
                            format!("Failed after {} attempts: {}", attempts, e),
                            correlation_id,
                        );
                    }

                    // Calculate exponential backoff delay
                    let delay_ms = calculate_backoff_delay(attempts, base_delay_ms, max_delay_ms);

                    tracing::debug!(
                        correlation_id = %correlation_id,
                        address = %address,
                        attempt = attempts,
                        delay_ms = delay_ms,
                        error = %e,
                        "🔄 Retrying after backoff"
                    );

                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    /// Convert batch results to a HashMap for easy lookup
    ///
    /// Only includes successful results.
    ///
    /// # Arguments
    ///
    /// * `results` - Slice of balance results
    ///
    /// # Returns
    ///
    /// HashMap mapping addresses to their balances
    pub fn results_to_map(results: &[BalanceResult]) -> HashMap<Address, U256> {
        results
            .iter()
            .filter_map(|r| r.balance.map(|b| (r.address, b)))
            .collect()
    }

    /// Get count statistics from batch results
    ///
    /// # Arguments
    ///
    /// * `results` - Slice of balance results
    ///
    /// # Returns
    ///
    /// Tuple of (success_count, failure_count)
    pub fn count_results(results: &[BalanceResult]) -> (usize, usize) {
        let success = results.iter().filter(|r| r.is_success()).count();
        let failure = results.len() - success;
        (success, failure)
    }
}

/// Calculate exponential backoff delay
///
/// Uses the formula: min(base_delay * 2^(attempt-1), max_delay)
///
/// # Arguments
///
/// * `attempt` - Current attempt number (1-indexed)
/// * `base_delay_ms` - Base delay in milliseconds
/// * `max_delay_ms` - Maximum delay cap in milliseconds
///
/// # Returns
///
/// Calculated delay in milliseconds
pub fn calculate_backoff_delay(attempt: u32, base_delay_ms: u64, max_delay_ms: u64) -> u64 {
    let delay = base_delay_ms.saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));
    delay.min(max_delay_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_backoff_calculation() {
        // First retry: base_delay * 2^0 = 1000
        assert_eq!(calculate_backoff_delay(1, 1000, 8000), 1000);
        // Second retry: base_delay * 2^1 = 2000
        assert_eq!(calculate_backoff_delay(2, 1000, 8000), 2000);
        // Third retry: base_delay * 2^2 = 4000
        assert_eq!(calculate_backoff_delay(3, 1000, 8000), 4000);
        // Fourth retry: capped at max_delay
        assert_eq!(calculate_backoff_delay(4, 1000, 8000), 8000);
        // Beyond max is still capped
        assert_eq!(calculate_backoff_delay(10, 1000, 8000), 8000);
    }

    #[tokio::test]
    async fn test_batch_processor_creation() {
        let processor = BatchProcessor::with_defaults();
        assert_eq!(processor.config().max_concurrent, 10);
    }

    #[tokio::test]
    async fn test_batch_empty_addresses() {
        let processor = BatchProcessor::with_defaults();

        let result = processor
            .batch_balance_queries(vec![], |_addr| async { Ok(U256::ZERO) })
            .await
            .expect("Empty batch should succeed");

        assert_eq!(result.results.len(), 0);
        assert_eq!(result.success_count, 0);
        assert_eq!(result.failure_count, 0);
    }

    #[tokio::test]
    async fn test_batch_balance_queries_success() {
        let processor = BatchProcessor::new(BatchConfig::for_testing());

        let addresses: Vec<Address> = (0..5)
            .map(|i| {
                let mut bytes = [0u8; 20];
                bytes[19] = i as u8;
                Address::from(bytes)
            })
            .collect();

        let result = processor
            .batch_balance_queries(addresses.clone(), |addr| async move {
                // Return a balance based on the last byte of the address
                let last_byte = addr.0[19] as u64;
                Ok(U256::from(last_byte * 1000))
            })
            .await
            .expect("Batch should succeed");

        assert_eq!(result.results.len(), 5);
        assert_eq!(result.success_count, 5);
        assert_eq!(result.failure_count, 0);
        assert!(result.all_succeeded());
    }

    #[tokio::test]
    async fn test_batch_partial_failure() {
        let processor = BatchProcessor::new(BatchConfig::for_testing());

        let addresses: Vec<Address> = (0..4)
            .map(|i| {
                let mut bytes = [0u8; 20];
                bytes[19] = i as u8;
                Address::from(bytes)
            })
            .collect();

        let result = processor
            .batch_balance_queries(addresses, |addr| async move {
                // Fail for addresses ending in 1 or 3
                let last_byte = addr.0[19];
                if last_byte % 2 == 1 {
                    Err(crate::error::VaughanError::Network(
                        crate::error::NetworkError::RpcError {
                            message: "Simulated failure".to_string(),
                        },
                    ))
                } else {
                    Ok(U256::from(last_byte as u64 * 1000))
                }
            })
            .await
            .expect("Batch should handle partial failures");

        // 0 and 2 succeed, 1 and 3 fail
        assert_eq!(result.results.len(), 4);
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failure_count, 2);
        assert!(!result.all_succeeded());
    }

    #[tokio::test]
    async fn test_concurrency_limiting() {
        let processor = BatchProcessor::new(BatchConfig {
            max_concurrent: 2,
            ..BatchConfig::for_testing()
        });

        let concurrent_count = Arc::new(AtomicUsize::new(0));
        let max_concurrent_seen = Arc::new(AtomicUsize::new(0));

        let addresses: Vec<Address> = (0..10)
            .map(|i| {
                let mut bytes = [0u8; 20];
                bytes[19] = i as u8;
                Address::from(bytes)
            })
            .collect();

        let cc = Arc::clone(&concurrent_count);
        let mcs = Arc::clone(&max_concurrent_seen);

        let result = processor
            .batch_balance_queries(addresses, move |_addr| {
                let cc = Arc::clone(&cc);
                let mcs = Arc::clone(&mcs);
                async move {
                    // Increment concurrent count
                    let current = cc.fetch_add(1, Ordering::SeqCst) + 1;

                    // Update max seen
                    let mut max = mcs.load(Ordering::SeqCst);
                    while current > max {
                        match mcs.compare_exchange(
                            max,
                            current,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        ) {
                            Ok(_) => break,
                            Err(actual) => max = actual,
                        }
                    }

                    // Simulate some work
                    tokio::time::sleep(Duration::from_millis(10)).await;

                    // Decrement concurrent count
                    cc.fetch_sub(1, Ordering::SeqCst);

                    Ok(U256::ZERO)
                }
            })
            .await
            .expect("Batch should succeed");

        assert_eq!(result.success_count, 10);
        // Max concurrent should not exceed the limit
        assert!(max_concurrent_seen.load(Ordering::SeqCst) <= 2);
    }

    #[tokio::test]
    async fn test_results_to_map() {
        let addr1 = Address::ZERO;
        let addr2 = {
            let mut bytes = [0u8; 20];
            bytes[19] = 1;
            Address::from(bytes)
        };

        let results = vec![
            BalanceResult::success(addr1, U256::from(100u64), Uuid::new_v4()),
            BalanceResult::failure(addr2, "Error".to_string(), Uuid::new_v4()),
        ];

        let map = BatchProcessor::results_to_map(&results);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&addr1), Some(&U256::from(100u64)));
        assert!(map.get(&addr2).is_none());
    }

    #[tokio::test]
    async fn test_count_results() {
        let results = vec![
            BalanceResult::success(Address::ZERO, U256::from(100u64), Uuid::new_v4()),
            BalanceResult::failure(Address::ZERO, "Error".to_string(), Uuid::new_v4()),
            BalanceResult::success(Address::ZERO, U256::from(200u64), Uuid::new_v4()),
        ];

        let (success, failure) = BatchProcessor::count_results(&results);
        assert_eq!(success, 2);
        assert_eq!(failure, 1);
    }
}
