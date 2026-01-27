//! Batch Processing Configuration
//!
//! Configuration types for batch RPC operations with concurrency limiting
//! and retry logic.

/// Configuration for batch processing operations
///
/// Controls concurrency, retry behavior, and timeouts for batch RPC operations.
///
/// # Examples
///
/// ```ignore
/// use vaughan::performance::batch::BatchConfig;
///
/// // Default configuration
/// let config = BatchConfig::default();
///
/// // Custom concurrency limit
/// let config = BatchConfig::with_concurrency(5);
/// ```
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of concurrent RPC requests (default: 10)
    pub max_concurrent: usize,
    /// Maximum number of retry attempts (default: 3)
    pub max_retries: u32,
    /// Base delay for exponential backoff in milliseconds (default: 1000)
    pub base_delay_ms: u64,
    /// Maximum delay for backoff in milliseconds (default: 8000)
    pub max_delay_ms: u64,
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 8000,
            timeout_secs: 30,
        }
    }
}

impl BatchConfig {
    /// Create a new batch config with custom concurrency limit
    ///
    /// Other parameters use default values.
    ///
    /// # Arguments
    ///
    /// * `max_concurrent` - Maximum number of concurrent requests
    pub fn with_concurrency(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            ..Default::default()
        }
    }

    /// Create a config optimized for testing (faster retries, lower concurrency)
    ///
    /// This configuration uses:
    /// - max_concurrent: 5
    /// - max_retries: 2
    /// - base_delay_ms: 100
    /// - max_delay_ms: 400
    /// - timeout_secs: 5
    #[cfg(test)]
    pub fn for_testing() -> Self {
        Self {
            max_concurrent: 5,
            max_retries: 2,
            base_delay_ms: 100,
            max_delay_ms: 400,
            timeout_secs: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 8000);
    }

    #[test]
    fn test_batch_config_with_concurrency() {
        let config = BatchConfig::with_concurrency(5);
        assert_eq!(config.max_concurrent, 5);
        // Other defaults should remain
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_batch_config_for_testing() {
        let config = BatchConfig::for_testing();
        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 400);
        assert_eq!(config.timeout_secs, 5);
    }
}
