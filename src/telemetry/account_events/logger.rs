//! Account Logger and Telemetry Operations
//!
//! Implements Property 18: Complete Operation Logging
//!
//! This module provides structured logging for account operations with
//! privacy-aware sanitization and correlation tracking.

use super::privacy::{contains_sensitive_data, get_privacy_mode, is_opted_out, PrivacyMode};
use super::spans::OperationSpan;
use tracing::{error, info, warn};

/// Specific Telemetry interface matching requirements
#[derive(Debug, Clone)]
pub struct AccountTelemetry {
    logger: AccountLogger,
}

impl AccountTelemetry {
    pub fn new() -> Self {
        Self {
            logger: AccountLogger::default(),
        }
    }

    pub fn record_event(&self, span: &OperationSpan, event: &str, details: &str) {
        if is_opted_out() {
            return;
        }
        self.logger
            .log_operation_complete(span, &format!("{}: {}", event, details));
    }

    pub fn record_error(&self, span: &OperationSpan, error: &str) {
        if is_opted_out() {
            return;
        }
        self.logger.log_operation_error(span, error);
    }
}

impl Default for AccountTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

/// Account logger with correlation tracking and privacy mode
///
/// Implements Properties 16-19 for structured logging
#[derive(Debug, Clone)]
pub struct AccountLogger {
    /// Privacy mode for this logger instance
    privacy_mode: PrivacyMode,
    /// Default component name
    component: String,
}

impl AccountLogger {
    /// Create a new account logger with privacy mode
    pub fn new(privacy_mode: PrivacyMode) -> Self {
        Self {
            privacy_mode,
            component: "account_manager".to_string(),
        }
    }

    /// Create a logger with default settings (privacy enabled)
    pub fn default_logger() -> Self {
        Self::new(PrivacyMode::Enabled)
    }

    /// Set the component name for logging context
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = component.into();
        self
    }

    /// Check if privacy mode is enabled for this logger
    pub fn is_privacy_enabled(&self) -> bool {
        self.privacy_mode.is_enabled() || get_privacy_mode()
    }

    /// Log operation start
    ///
    /// Implements Property 18: Complete Operation Logging
    pub fn log_operation_start(&self, span: &OperationSpan, details: &str) {
        info!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            component = %self.component,
            parent_id = ?span.parent_id,
            details = %self.sanitize_if_needed(details),
            "▶️ Operation started"
        );
    }

    /// Log operation completion
    ///
    /// Implements Property 18: Complete Operation Logging
    pub fn log_operation_complete(&self, span: &OperationSpan, details: &str) {
        let elapsed = span.elapsed_ms();
        info!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            component = %self.component,
            elapsed_ms = elapsed,
            details = %self.sanitize_if_needed(details),
            "✅ Operation completed"
        );
    }

    /// Log operation warning (recoverable issue)
    ///
    /// Implements Property 18: Complete Operation Logging
    pub fn log_operation_warning(&self, span: &OperationSpan, message: &str) {
        warn!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            component = %self.component,
            message = %self.sanitize_if_needed(message),
            "⚠️ Operation warning"
        );
    }

    /// Log operation error (failure)
    ///
    /// Implements Property 18: Complete Operation Logging
    pub fn log_operation_error(&self, span: &OperationSpan, error: &str) {
        let elapsed = span.elapsed_ms();
        error!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            component = %self.component,
            elapsed_ms = elapsed,
            error = %self.sanitize_if_needed(error),
            "❌ Operation failed"
        );
    }

    /// Log a child operation for cross-component tracking
    ///
    /// Implements Property 17: Cross-Component Correlation
    pub fn log_child_operation(
        &self,
        parent: &OperationSpan,
        child: &OperationSpan,
        details: &str,
    ) {
        info!(
            parent_correlation_id = %parent.correlation_id,
            child_correlation_id = %child.correlation_id,
            parent_operation = %parent.operation,
            child_operation = %child.operation,
            component = %self.component,
            details = %self.sanitize_if_needed(details),
            "🔗 Child operation started"
        );
    }

    /// Log account-specific event with address sanitization
    pub fn log_account_event(
        &self,
        span: &OperationSpan,
        event_type: &str,
        address: &str,
        details: &str,
    ) {
        use super::privacy::{sanitize, SensitiveDataType};

        let sanitized_address = if self.is_privacy_enabled() {
            sanitize(address, SensitiveDataType::Address)
        } else {
            address.to_string()
        };

        info!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            event_type = %event_type,
            address = %sanitized_address,
            details = %self.sanitize_if_needed(details),
            "📝 Account event"
        );
    }

    /// Sanitize text if privacy mode is enabled
    fn sanitize_if_needed(&self, text: &str) -> String {
        if self.is_privacy_enabled() && contains_sensitive_data(text) {
            "[SANITIZED]".to_string()
        } else {
            text.to_string()
        }
    }
}

impl Default for AccountLogger {
    fn default() -> Self {
        Self::default_logger()
    }
}

/// Operation result for tracking success/failure with logging
#[derive(Debug)]
pub struct TrackedOperation<T> {
    /// The operation span
    pub span: OperationSpan,
    /// The result of the operation
    pub result: Result<T, String>,
    /// Whether the operation was logged
    pub logged: bool,
}

impl<T> TrackedOperation<T> {
    /// Create a new tracked operation
    pub fn new(span: OperationSpan) -> Self {
        Self {
            span,
            result: Err("Not completed".to_string()),
            logged: false,
        }
    }

    /// Mark operation as successful
    pub fn success(mut self, value: T) -> Self {
        self.result = Ok(value);
        self
    }

    /// Mark operation as failed
    pub fn failure(mut self, error: String) -> Self {
        self.result = Err(error);
        self
    }

    /// Log and consume the operation
    pub fn log_and_finish(mut self, logger: &AccountLogger) -> Result<T, String> {
        self.logged = true;
        match &self.result {
            Ok(_) => logger.log_operation_complete(&self.span, "Operation successful"),
            Err(e) => logger.log_operation_error(&self.span, e),
        }
        self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::privacy::set_privacy_mode;

    #[test]
    fn test_account_logger_privacy_mode() {
        let logger = AccountLogger::new(PrivacyMode::Enabled);
        assert!(logger.is_privacy_enabled());

        let logger2 = AccountLogger::new(PrivacyMode::Disabled);
        // Still enabled because global mode check
        set_privacy_mode(true);
        assert!(logger2.is_privacy_enabled());
        set_privacy_mode(false);
    }

    #[test]
    fn test_tracked_operation_success() {
        let span = OperationSpan::new("test");
        let op: TrackedOperation<i32> = TrackedOperation::new(span);
        let op = op.success(42);

        assert!(op.result.is_ok());
        assert_eq!(op.result.unwrap(), 42);
    }

    #[test]
    fn test_tracked_operation_failure() {
        let span = OperationSpan::new("test");
        let op: TrackedOperation<()> = TrackedOperation::new(span);
        let op = op.failure("test error".to_string());

        assert!(op.result.is_err());
    }

    #[test]
    fn test_account_telemetry() {
        let telemetry = AccountTelemetry::new();
        let span = OperationSpan::new("test_operation");

        // These should not panic
        telemetry.record_event(&span, "test_event", "test details");
        telemetry.record_error(&span, "test error");
    }
}
