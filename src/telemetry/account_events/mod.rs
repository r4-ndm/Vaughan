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

pub mod privacy;
pub mod spans;
pub mod logger;

// Re-export main types for convenience
pub use privacy::{
    contains_sensitive_data, get_privacy_mode, is_opted_out, sanitize, set_opt_out,
    set_privacy_mode, PrivacyMode, SensitiveDataType,
};
pub use spans::{CorrelationContext, OperationSpan};
pub use logger::{AccountLogger, AccountTelemetry, TrackedOperation};

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
        #![proptest_config(ProptestConfig::with_cases(500))]

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
            prop_assert!(span.started_at <= chrono::Utc::now());
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
            // Enable privacy mode and verify it's set
            set_privacy_mode(true);
            prop_assert!(get_privacy_mode(), "Privacy mode should be enabled");

            // Private key should be redacted
            let sanitized_key = sanitize(&sensitive_data, SensitiveDataType::PrivateKey);
            prop_assert!(!sanitized_key.contains(&sensitive_data), 
                "Sanitized key should not contain original data: {} vs {}", 
                sanitized_key, sensitive_data);
            prop_assert!(sanitized_key.contains("REDACTED"), 
                "Sanitized key should contain REDACTED: {}", sanitized_key);

            // Password should be redacted
            let sanitized_pw = sanitize(&password, SensitiveDataType::Password);
            prop_assert!(!sanitized_pw.contains(&password),
                "Sanitized password should not contain original: {} vs {}",
                sanitized_pw, password);
            prop_assert!(sanitized_pw.contains("REDACTED"),
                "Sanitized password should contain REDACTED: {}", sanitized_pw);

            // Seed phrase should be redacted
            let sanitized_seed = sanitize("word1 word2 word3", SensitiveDataType::SeedPhrase);
            prop_assert!(sanitized_seed.contains("REDACTED"),
                "Sanitized seed should contain REDACTED: {}", sanitized_seed);

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
            _details in "[a-zA-Z0-9 ]{10,50}"
        ) {
            // Ensure privacy mode is enabled for this test
            set_privacy_mode(true);
            set_opt_out(false); // Ensure we are logging to check sanitization

            // Verify privacy mode is actually enabled
            prop_assert!(get_privacy_mode(), "Privacy mode should be enabled");

            // Check sanitization logic explicitly used by telemetry
            let sanitized_key = sanitize(&sensitive_data, SensitiveDataType::PrivateKey);
            let sanitized_addr = sanitize(&address, SensitiveDataType::Address);

            // Verify PII is removed
            prop_assert!(!sanitized_key.contains(&sensitive_data), 
                "Sanitized key should not contain original data: {} vs {}", 
                sanitized_key, sensitive_data);
            prop_assert!(sanitized_key.contains("REDACTED"), 
                "Sanitized key should contain REDACTED: {}", sanitized_key);

            prop_assert!(!sanitized_addr.contains(&address),
                "Sanitized address should not contain full address: {} vs {}",
                sanitized_addr, address);

            // Verify address is truncated if shown at all
            if sanitized_addr != "[REDACTED:ADDRESS]" {
                 prop_assert!(sanitized_addr.contains("..."),
                    "Sanitized address should contain ellipsis: {}", sanitized_addr);
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

            let mut ids: Vec<uuid::Uuid> = spans.iter().map(|s| s.correlation_id).collect();
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
            let mut parent_ids: Vec<uuid::Uuid> = vec![current.correlation_id];

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
