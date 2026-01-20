//! Security-safe logging for error handling
//!
//! This module provides logging utilities that automatically filter sensitive information.

use serde_json::json;
use std::sync::OnceLock;
use tracing::{debug, error, info, warn};

use super::{ErrorSeverity, VaughanError};

/// Security-safe error logger
#[derive(Default)]
pub struct ErrorLogger {
    log_sensitive: bool,
}

impl ErrorLogger {
    /// Create a new error logger
    pub fn new(log_sensitive: bool) -> Self {
        Self { log_sensitive }
    }

    /// Log an error with appropriate security filtering
    pub fn log_error(&self, error: &VaughanError, context: Option<&str>) {
        if !error.should_log() && !self.log_sensitive {
            // Log a generic message for sensitive errors
            warn!("Sensitive error occurred (details omitted for security)");
            return;
        }

        let error_context = error.context();
        let log_data = json!({
            "error_type": std::any::type_name_of_val(error),
            "severity": error_context.severity,
            "category": error_context.category,
            "support_code": error_context.support_code,
            "timestamp": error_context.timestamp,
            "context": context,
            "recoverable": error.is_recoverable(),
        });

        match error_context.severity {
            ErrorSeverity::Critical => {
                error!("Critical error: {} | Data: {}", error_context.user_message, log_data);
            }
            ErrorSeverity::High => {
                error!(
                    "High severity error: {} | Data: {}",
                    error_context.user_message, log_data
                );
            }
            ErrorSeverity::Medium => {
                warn!(
                    "Medium severity error: {} | Data: {}",
                    error_context.user_message, log_data
                );
            }
            ErrorSeverity::Low => {
                info!(
                    "Low severity error: {} | Data: {}",
                    error_context.user_message, log_data
                );
            }
        }
    }

    /// Log error recovery attempt
    pub fn log_recovery_attempt(&self, error: &VaughanError, attempt: usize) {
        debug!("Recovery attempt {} for error: {}", attempt, error.user_message());
    }

    /// Log successful error recovery
    pub fn log_recovery_success(&self, error: &VaughanError, attempts: usize) {
        info!(
            "Error recovery successful after {} attempts: {}",
            attempts,
            error.user_message()
        );
    }

    /// Log failed error recovery
    pub fn log_recovery_failure(&self, error: &VaughanError, attempts: usize) {
        error!(
            "Error recovery failed after {} attempts: {}",
            attempts,
            error.user_message()
        );
    }
}

/// Global error logger instance
static ERROR_LOGGER: OnceLock<ErrorLogger> = OnceLock::new();

/// Initialize the global error logger
pub fn init_error_logger(log_sensitive: bool) {
    ERROR_LOGGER.get_or_init(|| ErrorLogger::new(log_sensitive));
}

/// Get the global error logger
pub fn get_error_logger() -> &'static ErrorLogger {
    ERROR_LOGGER.get_or_init(ErrorLogger::default)
}

/// Convenience function to log an error
pub fn log_error(error: &VaughanError, context: Option<&str>) {
    get_error_logger().log_error(error, context);
}
