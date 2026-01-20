//! Error recovery mechanisms
//!
//! This module provides automatic error recovery and retry logic for various error types.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use super::{ConfigurationError, RecoveryAction, Result, VaughanError};

/// Error recovery manager
pub struct ErrorRecoveryManager {
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new(max_retries: usize, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }

    /// Attempt to recover from an error with automatic retry logic
    pub async fn recover_with_retry<F, Fut, T>(&self, operation: F, error: &VaughanError) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        if !error.is_recoverable() {
            warn!("Error is not recoverable: {}", error);
            return Err(error.clone());
        }

        let recovery_actions = error.recovery_actions();

        for (attempt, action) in recovery_actions.iter().enumerate() {
            if attempt >= self.max_retries {
                error!("Max recovery attempts exceeded for error: {}", error);
                break;
            }

            info!("Attempting recovery action {:?} (attempt {})", action, attempt + 1);

            match self.execute_recovery_action(action).await {
                Ok(_) => {
                    // Try the operation again after recovery action
                    match operation().await {
                        Ok(result) => {
                            info!("Recovery successful after {} attempts", attempt + 1);
                            return Ok(result);
                        }
                        Err(new_error) => {
                            warn!("Operation failed after recovery attempt {}: {}", attempt + 1, new_error);
                            // Continue to next recovery action
                        }
                    }
                }
                Err(recovery_error) => {
                    warn!("Recovery action failed: {}", recovery_error);
                    // Continue to next recovery action
                }
            }
        }

        error!("All recovery attempts failed for error: {}", error);
        Err(error.clone())
    }

    /// Execute a specific recovery action
    async fn execute_recovery_action(&self, action: &RecoveryAction) -> Result<()> {
        match action {
            RecoveryAction::Retry => {
                // Simple retry - no additional action needed
                Ok(())
            }

            RecoveryAction::RetryWithDelay { seconds } => {
                let delay = Duration::from_secs(*seconds).min(self.max_delay);
                info!("Waiting {} seconds before retry", delay.as_secs());
                sleep(delay).await;
                Ok(())
            }

            RecoveryAction::CheckConnection => {
                // This would typically involve checking network connectivity
                // For now, we'll just add a small delay
                info!("Checking network connection...");
                sleep(Duration::from_millis(1000)).await;
                Ok(())
            }

            RecoveryAction::SwitchNetwork => {
                // This would involve switching to a backup network
                info!("Attempting to switch to backup network...");
                sleep(Duration::from_millis(500)).await;
                Ok(())
            }

            RecoveryAction::UpdateConfiguration => {
                // This would involve reloading configuration
                info!("Updating configuration...");
                sleep(Duration::from_millis(500)).await;
                Ok(())
            }

            RecoveryAction::RestartApplication => {
                // This would involve restarting components
                warn!("Application restart required - this should be handled by the caller");
                Err(VaughanError::Configuration(ConfigurationError::ValidationFailed {
                    reason: "Application restart required".to_string(),
                }))
            }

            RecoveryAction::ContactSupport => {
                // This would involve logging the issue for support
                error!("Support contact required - logging issue for manual review");
                Ok(())
            }
        }
    }

    /// Retry an operation with exponential backoff
    pub async fn retry_with_exponential_backoff<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut delay = self.base_delay;

        for attempt in 0..self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt == self.max_retries - 1 {
                        error!("Final retry attempt failed: {}", error);
                        return Err(error);
                    }

                    if !error.is_recoverable() {
                        warn!("Error is not recoverable, stopping retries: {}", error);
                        return Err(error);
                    }

                    warn!("Attempt {} failed: {}, retrying in {:?}", attempt + 1, error, delay);
                    sleep(delay).await;

                    // Exponential backoff with jitter
                    delay = (delay * 2).min(self.max_delay);
                    let jitter = Duration::from_millis(fastrand::u64(0..=100));
                    delay += jitter;
                }
            }
        }

        unreachable!("Loop should have returned or broken before this point")
    }
}

/// Convenience function for simple retry with exponential backoff
pub async fn retry_operation<F, Fut, T>(operation: F, _max_retries: usize) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let recovery_manager = ErrorRecoveryManager::default();
    recovery_manager.retry_with_exponential_backoff(operation).await
}

/// Convenience function for network operations with automatic retry
pub async fn retry_network_operation<F, Fut, T>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let recovery_manager = ErrorRecoveryManager::new(
        5, // More retries for network operations
        Duration::from_millis(1000),
        Duration::from_secs(10),
    );
    recovery_manager.retry_with_exponential_backoff(operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{NetworkError, VaughanError};

    #[tokio::test]
    async fn test_retry_with_exponential_backoff() {
        let recovery_manager = ErrorRecoveryManager::new(3, Duration::from_millis(10), Duration::from_millis(100));

        let mut attempt_count = 0;
        let result = recovery_manager
            .retry_with_exponential_backoff(|| {
                attempt_count += 1;
                async move {
                    if attempt_count < 3 {
                        Err(VaughanError::Network(NetworkError::Timeout))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count, 3);
    }

    #[tokio::test]
    async fn test_non_recoverable_error() {
        let recovery_manager = ErrorRecoveryManager::default();

        let result: Result<()> = recovery_manager
            .retry_with_exponential_backoff(|| async {
                Err(VaughanError::Security(crate::error::SecurityError::InvalidPrivateKey))
            })
            .await;

        assert!(result.is_err());
    }
}
