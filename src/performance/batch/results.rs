//! Batch Processing Result Types
//!
//! Result types for batch RPC operations, including individual query results
//! and aggregate batch results with metadata.

use alloy::primitives::{Address, U256};
use uuid::Uuid;

/// Result of a single balance query in a batch
///
/// Contains the query result (success or failure) along with correlation tracking.
#[derive(Debug, Clone)]
pub struct BalanceResult {
    /// The queried address
    pub address: Address,
    /// The balance if successful
    pub balance: Option<U256>,
    /// Error message if failed
    pub error: Option<String>,
    /// Correlation ID for this specific query
    pub correlation_id: Uuid,
}

impl BalanceResult {
    /// Create a successful result
    ///
    /// # Arguments
    ///
    /// * `address` - The queried address
    /// * `balance` - The retrieved balance
    /// * `correlation_id` - Correlation ID for tracking
    pub fn success(address: Address, balance: U256, correlation_id: Uuid) -> Self {
        Self {
            address,
            balance: Some(balance),
            error: None,
            correlation_id,
        }
    }

    /// Create a failed result
    ///
    /// # Arguments
    ///
    /// * `address` - The queried address
    /// * `error` - Error message describing the failure
    /// * `correlation_id` - Correlation ID for tracking
    pub fn failure(address: Address, error: String, correlation_id: Uuid) -> Self {
        Self {
            address,
            balance: None,
            error: Some(error),
            correlation_id,
        }
    }

    /// Check if the result is successful
    ///
    /// Returns `true` if the balance was retrieved successfully.
    pub fn is_success(&self) -> bool {
        self.balance.is_some()
    }
}

/// Batch result containing all query results and metadata
///
/// Provides aggregate statistics and correlation tracking for a batch operation.
///
/// # Type Parameters
///
/// * `T` - The type of individual results in the batch
#[derive(Debug)]
pub struct BatchResult<T> {
    /// Individual results
    pub results: Vec<T>,
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failure_count: usize,
    /// Batch correlation ID
    pub correlation_id: Uuid,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl<T> BatchResult<T> {
    /// Create a new batch result
    ///
    /// # Arguments
    ///
    /// * `results` - Individual operation results
    /// * `success_count` - Number of successful operations
    /// * `failure_count` - Number of failed operations
    /// * `correlation_id` - Batch correlation ID
    /// * `duration_ms` - Total batch duration in milliseconds
    pub fn new(
        results: Vec<T>,
        success_count: usize,
        failure_count: usize,
        correlation_id: Uuid,
        duration_ms: u64,
    ) -> Self {
        Self {
            results,
            success_count,
            failure_count,
            correlation_id,
            duration_ms,
        }
    }

    /// Check if all operations succeeded
    ///
    /// Returns `true` if there were no failures.
    pub fn all_succeeded(&self) -> bool {
        self.failure_count == 0
    }

    /// Get the success rate as a percentage
    ///
    /// Returns a value between 0.0 and 100.0.
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            100.0
        } else {
            (self.success_count as f64 / total as f64) * 100.0
        }
    }
}

/// Batch error types for more specific error handling
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    /// All requests in the batch failed
    #[error("All {count} requests failed [correlation: {correlation_id}]")]
    AllFailed {
        /// Number of failed requests
        count: usize,
        /// Batch correlation ID
        correlation_id: Uuid,
    },

    /// Batch was cancelled
    #[error("Batch operation cancelled [correlation: {correlation_id}]")]
    Cancelled {
        /// Batch correlation ID
        correlation_id: Uuid,
    },

    /// Configuration error
    #[error("Invalid batch configuration: {message}")]
    InvalidConfig {
        /// Error message
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_result_success() {
        let addr = Address::ZERO;
        let balance = U256::from(1000u64);
        let corr_id = Uuid::new_v4();

        let result = BalanceResult::success(addr, balance, corr_id);
        assert!(result.is_success());
        assert_eq!(result.balance, Some(balance));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_balance_result_failure() {
        let addr = Address::ZERO;
        let corr_id = Uuid::new_v4();

        let result = BalanceResult::failure(addr, "Network error".to_string(), corr_id);
        assert!(!result.is_success());
        assert!(result.balance.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_batch_result_success_rate() {
        let corr_id = Uuid::new_v4();

        let result: BatchResult<()> = BatchResult::new(vec![], 8, 2, corr_id, 100);
        assert!((result.success_rate() - 80.0).abs() < 0.01);
        assert!(!result.all_succeeded());

        let all_success: BatchResult<()> = BatchResult::new(vec![], 10, 0, corr_id, 100);
        assert!((all_success.success_rate() - 100.0).abs() < 0.01);
        assert!(all_success.all_succeeded());
    }

    #[test]
    fn test_batch_result_empty() {
        let corr_id = Uuid::new_v4();
        let result: BatchResult<()> = BatchResult::new(vec![], 0, 0, corr_id, 0);
        
        assert_eq!(result.success_rate(), 100.0);
        assert!(result.all_succeeded());
    }
}
