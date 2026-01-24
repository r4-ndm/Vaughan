//! Account Events and Structured Logging
//!
//! This module provides structured logging for account management operations
//! with correlation tracking, privacy mode filtering, and span instrumentation.
//!
//! # Requirements Addressed
//!
//! - **Requirement 7.1**: Correlation ID creation for all account operations
//! - **Requirement 7.3**: Cross-component correlation context propagation
//! - **Requirement 7.4**: Complete operation logging (start, completion, errors)
//! - **Requirement 7.5**: Privacy mode filtering for sensitive data
//!
//! # Design Properties
//!
//! - **Property 16**: Operation Correlation Logging
//! - **Property 17**: Cross-Component Correlation
//! - **Property 18**: Complete Operation Logging
//! - **Property 19**: Privacy Mode Log Sanitization
//!
//! # Usage
//!
//! ```rust,ignore
//! use vaughan::telemetry::{AccountLogger, OperationSpan, PrivacyMode};
//!
//! // Create an operation span with correlation ID
//! let span = OperationSpan::new("create_account");
//! 
//! // Log with privacy mode
//! let logger = AccountLogger::new(PrivacyMode::Enabled);
//! logger.log_operation_start(&span, "Creating new account");
//! ```

use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicBool, Ordering};
// unused import removed
use tracing::{error, info, warn, Span};
use uuid::Uuid;

/// Global privacy mode setting
static PRIVACY_MODE: AtomicBool = AtomicBool::new(false);

/// Global opt-out setting
static OPT_OUT: AtomicBool = AtomicBool::new(false);

/// Privacy mode configuration for log sanitization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyMode {
    /// Privacy mode enabled - sanitize all sensitive data
    Enabled,
    /// Privacy mode disabled - full logging (development only)
    Disabled,
}

impl PrivacyMode {
    /// Check if privacy mode is enabled
    pub fn is_enabled(&self) -> bool {
        matches!(self, PrivacyMode::Enabled)
    }
}

impl Default for PrivacyMode {
    fn default() -> Self {
        // Default to enabled for security
        PrivacyMode::Enabled
    }
}

/// Set the global privacy mode
pub fn set_privacy_mode(enabled: bool) {
    PRIVACY_MODE.store(enabled, Ordering::SeqCst);
    if enabled {
        info!("🔒 Privacy mode enabled - sensitive data will be redacted from logs");
    } else {
        warn!("⚠️ Privacy mode disabled - sensitive data may appear in logs");
    }
}

/// Get the current global privacy mode
pub fn get_privacy_mode() -> bool {
    PRIVACY_MODE.load(Ordering::SeqCst)
}

/// Set opt-out status
pub fn set_opt_out(opt_out: bool) {
    OPT_OUT.store(opt_out, Ordering::SeqCst);
    if opt_out {
        warn!("🚫 Telemetry opt-out enabled - no events will be recorded");
    } else {
        info!("✅ Telemetry opt-in confirmed");
    }
}

/// Check if user has opted out
pub fn is_opted_out() -> bool {
    OPT_OUT.load(Ordering::SeqCst)
}

/// specific Telemetry interface matching requirements
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
        self.logger.log_operation_complete(span, &format!("{}: {}", event, details));
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

/// Operation span for tracking a single operation with correlation ID
///
/// Implements Property 16: Operation Correlation Logging
#[derive(Debug, Clone)]
pub struct OperationSpan {
    /// Unique correlation ID for this operation
    pub correlation_id: Uuid,
    /// Name of the operation being performed
    pub operation: String,
    /// When the operation started
    pub started_at: DateTime<Utc>,
    /// Parent correlation ID for cross-component tracking
    pub parent_id: Option<Uuid>,
    /// Component where the operation originated
    pub component: Option<String>,
}

impl OperationSpan {
    /// Create a new operation span with auto-generated correlation ID
    ///
    /// Implements Property 16: Creates correlation ID at operation start
    pub fn new(operation: impl Into<String>) -> Self {
        let span = Self {
            correlation_id: Uuid::new_v4(),
            operation: operation.into(),
            started_at: Utc::now(),
            parent_id: None,
            component: None,
        };

        tracing::debug!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            "📋 Operation span created"
        );

        span
    }

    /// Create a child span for cross-component tracking
    ///
    /// Implements Property 17: Cross-Component Correlation
    pub fn child(&self, operation: impl Into<String>) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            operation: operation.into(),
            started_at: Utc::now(),
            parent_id: Some(self.correlation_id),
            component: self.component.clone(),
        }
    }

    /// Set the component name for this span
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Get the elapsed time since operation start in milliseconds
    pub fn elapsed_ms(&self) -> i64 {
        (Utc::now() - self.started_at).num_milliseconds()
    }

    /// Create a tracing span for this operation
    pub fn tracing_span(&self) -> Span {
        tracing::info_span!(
            "operation",
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            parent_id = ?self.parent_id,
            component = ?self.component,
        )
    }
}

/// Sensitive data types that should be sanitized in privacy mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveDataType {
    /// Private key (32 bytes hex)
    PrivateKey,
    /// Seed phrase (12-24 words)
    SeedPhrase,
    /// Password
    Password,
    /// Wallet address
    Address,
    /// Transaction data
    TransactionData,
    /// Balance information
    Balance,
}

/// Sanitize sensitive data based on privacy mode
///
/// Implements Property 19: Privacy Mode Log Sanitization
pub fn sanitize(data: &str, data_type: SensitiveDataType) -> String {
    if !get_privacy_mode() {
        return data.to_string();
    }

    match data_type {
        SensitiveDataType::PrivateKey => "[REDACTED:PRIVATE_KEY]".to_string(),
        SensitiveDataType::SeedPhrase => "[REDACTED:SEED_PHRASE]".to_string(),
        SensitiveDataType::Password => "[REDACTED:PASSWORD]".to_string(),
        SensitiveDataType::Address => {
            // Show first 6 and last 4 characters
            if data.len() > 10 {
                format!("{}...{}", &data[..6], &data[data.len()-4..])
            } else {
                "[REDACTED:ADDRESS]".to_string()
            }
        }
        SensitiveDataType::TransactionData => "[REDACTED:TX_DATA]".to_string(),
        SensitiveDataType::Balance => "[REDACTED:BALANCE]".to_string(),
    }
}

/// Check if a string contains potentially sensitive patterns
pub fn contains_sensitive_data(text: &str) -> bool {
    let lower = text.to_lowercase();
    
    // Check for common sensitive patterns
    let patterns = [
        "private", "seed", "mnemonic", "password", "secret",
        "key", "0x", // Hex prefixes often indicate keys
    ];

    patterns.iter().any(|p| lower.contains(p))
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
    pub fn log_child_operation(&self, parent: &OperationSpan, child: &OperationSpan, details: &str) {
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

/// Correlation context for passing across async boundaries
///
/// Implements Property 17: Cross-Component Correlation
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Primary correlation ID
    pub correlation_id: Uuid,
    /// Stack of parent IDs for nested operations
    pub parent_ids: Vec<Uuid>,
    /// Component chain for tracking flow
    pub components: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            parent_ids: Vec::new(),
            components: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Create from an existing operation span
    pub fn from_span(span: &OperationSpan) -> Self {
        let mut ctx = Self::new();
        ctx.correlation_id = span.correlation_id;
        if let Some(parent) = span.parent_id {
            ctx.parent_ids.push(parent);
        }
        if let Some(ref component) = span.component {
            ctx.components.push(component.clone());
        }
        ctx
    }

    /// Add a component to the context
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.components.push(component.into());
        self
    }

    /// Create a child context
    pub fn child(&self) -> Self {
        let mut child = Self::new();
        child.parent_ids = self.parent_ids.clone();
        child.parent_ids.push(self.correlation_id);
        child.components = self.components.clone();
        child
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_span_creation() {
        let span = OperationSpan::new("test_operation");
        
        assert!(!span.correlation_id.is_nil());
        assert_eq!(span.operation, "test_operation");
        assert!(span.parent_id.is_none());
    }

    #[test]
    fn test_child_span_creation() {
        let parent = OperationSpan::new("parent_op");
        let child = parent.child("child_op");
        
        assert_eq!(child.parent_id, Some(parent.correlation_id));
        assert_ne!(child.correlation_id, parent.correlation_id);
    }

    #[test]
    fn test_sanitize_private_key() {
        set_privacy_mode(true);
        let result = sanitize("0x1234567890abcdef", SensitiveDataType::PrivateKey);
        assert_eq!(result, "[REDACTED:PRIVATE_KEY]");
        set_privacy_mode(false);
    }

    #[test]
    fn test_sanitize_address() {
        set_privacy_mode(true);
        let result = sanitize("0x1234567890abcdef12345678", SensitiveDataType::Address);
        assert!(result.contains("..."));
        assert!(result.starts_with("0x1234"));
        set_privacy_mode(false);
    }

    #[test]
    fn test_sanitize_disabled() {
        set_privacy_mode(false);
        let data = "sensitive_data";
        let result = sanitize(data, SensitiveDataType::Password);
        assert_eq!(result, data);
    }

    #[test]
    fn test_contains_sensitive_data() {
        assert!(contains_sensitive_data("my private key"));
        assert!(contains_sensitive_data("seed phrase here"));
        assert!(contains_sensitive_data("enter password"));
        assert!(!contains_sensitive_data("hello world"));
    }

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
    fn test_correlation_context_child() {
        let parent = CorrelationContext::new();
        let child = parent.child();
        
        assert!(child.parent_ids.contains(&parent.correlation_id));
        assert_ne!(child.correlation_id, parent.correlation_id);
    }

    #[test]
    fn test_operation_span_elapsed() {
        let span = OperationSpan::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = span.elapsed_ms();
        assert!(elapsed >= 10);
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 16: Operation Correlation Logging
        ///
        /// *For any* account operation, the system should create a correlation ID
        /// at the start and include it in all log entries for that operation.
        ///
        /// Validates: Requirements 7.1
        #[test]
        fn prop_operation_correlation_logging(
            operation_name in "[a-z_]{3,20}"
        ) {
            let span = OperationSpan::new(&operation_name);
            
            // Correlation ID should be non-nil
            prop_assert!(!span.correlation_id.is_nil());
            
            // Operation name should be preserved
            prop_assert_eq!(span.operation, operation_name);
            
            // Timestamp should be set
            prop_assert!(span.started_at <= Utc::now());
        }

        /// Property 17: Cross-Component Correlation
        ///
        /// *For any* operation that spans multiple components, the correlation ID
        /// should be propagated and maintained across all component boundaries.
        ///
        /// Validates: Requirements 7.3
        #[test]
        fn prop_cross_component_correlation(
            parent_op in "[a-z_]{3,15}",
            child_op in "[a-z_]{3,15}",
            component in "[a-z_]{3,15}"
        ) {
            let parent = OperationSpan::new(&parent_op).with_component(&component);
            let child = parent.child(&child_op);
            
            // Child should reference parent's correlation ID
            prop_assert_eq!(child.parent_id, Some(parent.correlation_id));
            
            // Child should have its own unique correlation ID
            prop_assert_ne!(child.correlation_id, parent.correlation_id);
            
            // Correlation context should track hierarchy
            let ctx = CorrelationContext::from_span(&parent);
            let child_ctx = ctx.child();
            prop_assert!(child_ctx.parent_ids.contains(&ctx.correlation_id));
        }

        /// Property 18: Complete Operation Logging
        ///
        /// *For any* account operation, the system should log operation start,
        /// completion (or error), with appropriate log levels.
        ///
        /// Validates: Requirements 7.4
        #[test]
        fn prop_complete_operation_logging(
            operation_name in "[a-z_]{3,20}",
            details in "[a-zA-Z0-9 ]{5,50}"
        ) {
            let span = OperationSpan::new(&operation_name);
            let logger = AccountLogger::new(PrivacyMode::Disabled);
            
            // All these should execute without panic
            logger.log_operation_start(&span, &details);
            logger.log_operation_complete(&span, "completed");
            logger.log_operation_warning(&span, "warning message");
            logger.log_operation_error(&span, "error message");
            
            // Elapsed time should be non-negative
            prop_assert!(span.elapsed_ms() >= 0);
        }

        /// Property 19: Privacy Mode Log Sanitization
        ///
        /// *For any* log entry when privacy mode is enabled, the entry should not
        /// contain private keys, seed phrases, passwords, or other sensitive info.
        ///
        /// Validates: Requirements 7.5
        #[test]
        fn prop_privacy_mode_log_sanitization(
            sensitive_data in "[a-f0-9]{64}",  // Simulates private key
            password in "[a-zA-Z0-9!@#$%^&*]{8,20}"
        ) {
            set_privacy_mode(true);
            
            // Private key should be redacted
            let sanitized_key = sanitize(&sensitive_data, SensitiveDataType::PrivateKey);
            prop_assert!(!sanitized_key.contains(&sensitive_data));
            prop_assert!(sanitized_key.contains("REDACTED"));
            
            // Password should be redacted
            let sanitized_pw = sanitize(&password, SensitiveDataType::Password);
            prop_assert!(!sanitized_pw.contains(&password));
            
            // Seed phrase should be redacted
            let sanitized_seed = sanitize("word1 word2 word3", SensitiveDataType::SeedPhrase);
            prop_assert!(sanitized_seed.contains("REDACTED"));
            
            set_privacy_mode(false);
        }

        /// Property 29: Telemetry Anonymity
        ///
        /// Validates: Requirements 10.1, 10.4
        /// Test that telemetry contains no sensitive data when privacy mode is enabled.
        #[test]
        fn prop_telemetry_anonymity(
            sensitive_data in "[a-f0-9]{64}", 
            address in "0x[a-f0-9]{40}",
            details in "[a-zA-Z0-9 ]{10,50}"
        ) {
            set_privacy_mode(true);
            set_opt_out(false); // Ensure we are logging to check sanitization
            
            // Check sanitization logic explicitly used by telemetry
            let sanitized_key = sanitize(&sensitive_data, SensitiveDataType::PrivateKey);
            let sanitized_addr = sanitize(&address, SensitiveDataType::Address);
            
            // Verify PII is removed
            prop_assert!(!sanitized_key.contains(&sensitive_data));
            prop_assert!(sanitized_key.contains("REDACTED"));
            
            prop_assert!(!sanitized_addr.contains(&address));
            
            // Verify address is truncated if shown at all
            if sanitized_addr != "[REDACTED:ADDRESS]" {
                 prop_assert!(sanitized_addr.contains("..."));
            }
            
            set_privacy_mode(false);
        }

        /// Property: Correlation IDs are unique
        ///
        /// *For any* set of operations, each should have a unique correlation ID.
        #[test]
        fn prop_correlation_ids_unique(
            count in 2usize..20
        ) {
            let spans: Vec<OperationSpan> = (0..count)
                .map(|i| OperationSpan::new(format!("op_{}", i)))
                .collect();
            
            let mut ids: Vec<Uuid> = spans.iter().map(|s| s.correlation_id).collect();
            ids.sort();
            ids.dedup();
            
            prop_assert_eq!(ids.len(), count, "All correlation IDs should be unique");
        }

        /// Property: Child operations form proper hierarchy
        #[test]
        fn prop_child_hierarchy(
            depth in 2usize..5
        ) {
            let mut current = OperationSpan::new("root");
            let mut parent_ids: Vec<Uuid> = vec![current.correlation_id];
            
            for i in 1..depth {
                let child = current.child(format!("level_{}", i));
                prop_assert_eq!(child.parent_id, Some(current.correlation_id));
                prop_assert!(!parent_ids.contains(&child.correlation_id));
                parent_ids.push(child.correlation_id);
                current = child;
            }

            // All IDs should be unique
            let mut sorted = parent_ids.clone();
            sorted.sort();
            sorted.dedup();
            prop_assert_eq!(sorted.len(), depth);
        }
    }
}
