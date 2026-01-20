//! Account-specific error handling with correlation tracking
//!
//! This module provides error types and context structures for account management
//! operations, following Requirements 4.1 and 4.2 for structured error handling.
//!
//! ## Features
//!
//! - `AccountError`: Comprehensive error enum for all account operations
//! - `ErrorContext`: Rich context with correlation IDs for debugging
//! - Builder pattern for flexible context enrichment
//!
//! ## Correlation Tracking
//!
//! Every error includes a correlation ID (UUID v4) that can be used to trace
//! operations across async boundaries and multiple components.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

/// Account-specific error types with correlation tracking
///
/// Each error variant includes a correlation ID for tracking operations
/// across the system, as required by Requirements 4.1 and 4.2.
#[derive(Error, Debug, Clone)]
pub enum AccountError {
    /// Account not found in keystore
    #[error("Account not found: {address} [correlation: {correlation_id}]")]
    AccountNotFound {
        /// The account address that was not found
        address: String,
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },

    /// Invalid credentials (password or authentication failed)
    #[error("Invalid credentials [correlation: {correlation_id}]")]
    InvalidCredentials {
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },

    /// Account is locked and operation requires unlock
    #[error("Account is locked [correlation: {correlation_id}]")]
    AccountLocked {
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },

    /// Account import failed
    #[error("Import failed from {import_source}: {reason} [correlation: {correlation_id}]")]
    ImportFailed {
        /// Human-readable reason for the failure
        reason: String,
        /// Source of the import (e.g., "seed_phrase", "private_key", "metamask")
        import_source: String,
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },

    /// Account export failed
    #[error("Export failed: {reason} [correlation: {correlation_id}]")]
    ExportFailed {
        /// Human-readable reason for the failure
        reason: String,
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },

    /// Generic operation failure with context
    #[error("Operation '{operation}' failed: {reason} [correlation: {correlation_id}]")]
    OperationFailed {
        /// The operation that failed (e.g., "create", "unlock", "sign")
        operation: String,
        /// Human-readable reason for the failure
        reason: String,
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },

    /// Validation error for account data
    #[error("Validation failed: {message} [correlation: {correlation_id}]")]
    ValidationFailed {
        /// Human-readable validation error message
        message: String,
        /// Correlation ID for tracking this error
        correlation_id: Uuid,
    },
}

impl AccountError {
    /// Generate a new correlation ID for error creation
    #[inline]
    pub fn new_correlation_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Get the correlation ID from any error variant
    pub fn correlation_id(&self) -> Uuid {
        match self {
            Self::AccountNotFound { correlation_id, .. } => *correlation_id,
            Self::InvalidCredentials { correlation_id, .. } => *correlation_id,
            Self::AccountLocked { correlation_id, .. } => *correlation_id,
            Self::ImportFailed { correlation_id, .. } => *correlation_id,
            Self::ExportFailed { correlation_id, .. } => *correlation_id,
            Self::OperationFailed { correlation_id, .. } => *correlation_id,
            Self::ValidationFailed { correlation_id, .. } => *correlation_id,
        }
    }

    /// Create an AccountNotFound error with auto-generated correlation ID
    pub fn account_not_found(address: impl Into<String>) -> Self {
        Self::AccountNotFound {
            address: address.into(),
            correlation_id: Self::new_correlation_id(),
        }
    }

    /// Create an InvalidCredentials error with auto-generated correlation ID
    pub fn invalid_credentials() -> Self {
        Self::InvalidCredentials {
            correlation_id: Self::new_correlation_id(),
        }
    }

    /// Create an AccountLocked error with auto-generated correlation ID
    pub fn account_locked() -> Self {
        Self::AccountLocked {
            correlation_id: Self::new_correlation_id(),
        }
    }

    /// Create an ImportFailed error with auto-generated correlation ID
    pub fn import_failed(reason: impl Into<String>, import_source: impl Into<String>) -> Self {
        Self::ImportFailed {
            reason: reason.into(),
            import_source: import_source.into(),
            correlation_id: Self::new_correlation_id(),
        }
    }

    /// Create an ExportFailed error with auto-generated correlation ID
    pub fn export_failed(reason: impl Into<String>) -> Self {
        Self::ExportFailed {
            reason: reason.into(),
            correlation_id: Self::new_correlation_id(),
        }
    }

    /// Create an OperationFailed error with auto-generated correlation ID
    pub fn operation_failed(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::OperationFailed {
            operation: operation.into(),
            reason: reason.into(),
            correlation_id: Self::new_correlation_id(),
        }
    }

    /// Create a ValidationFailed error with auto-generated correlation ID
    pub fn validation_failed(message: impl Into<String>) -> Self {
        Self::ValidationFailed {
            message: message.into(),
            correlation_id: Self::new_correlation_id(),
        }
    }
}

/// Rich error context with correlation tracking for debugging
///
/// Implements Requirements 4.1 and 4.2 by including:
/// - Correlation ID (UUID v4) for cross-component tracking
/// - Operation name describing what was being attempted
/// - Optional account ID for account-related operations
/// - Optional user action context
/// - Optional network context
/// - Timestamp using chrono::Utc
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Unique correlation ID for tracking across components
    pub correlation_id: Uuid,
    /// Name of the operation that generated this error
    pub operation: String,
    /// Account address if applicable
    pub account_id: Option<String>,
    /// User action that triggered the operation
    pub user_action: Option<String>,
    /// Network name or ID if applicable
    pub network: Option<String>,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
}

impl ErrorContext {
    /// Create a new ErrorContext with automatic UUID generation
    ///
    /// # Arguments
    /// * `operation` - Name of the operation being performed
    ///
    /// # Example
    /// ```ignore
    /// let ctx = ErrorContext::new("create_account");
    /// ```
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            operation: operation.into(),
            account_id: None,
            user_action: None,
            network: None,
            timestamp: Utc::now(),
        }
    }

    /// Add account context to the error
    ///
    /// # Arguments
    /// * `address` - The account address involved in the operation
    pub fn with_account(mut self, address: impl Into<String>) -> Self {
        self.account_id = Some(address.into());
        self
    }

    /// Add user action context to the error
    ///
    /// # Arguments
    /// * `action` - Description of what the user was doing
    pub fn with_user_action(mut self, action: impl Into<String>) -> Self {
        self.user_action = Some(action.into());
        self
    }

    /// Add network context to the error
    ///
    /// # Arguments
    /// * `network` - Network name or chain ID
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Operation: {} | Correlation: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.operation,
            self.correlation_id
        )?;

        if let Some(ref account) = self.account_id {
            write!(f, " | Account: {}", account)?;
        }
        if let Some(ref action) = self.user_action {
            write!(f, " | Action: {}", action)?;
        }
        if let Some(ref network) = self.network {
            write!(f, " | Network: {}", network)?;
        }

        Ok(())
    }
}

/// Result type alias for account operations
pub type AccountResult<T> = Result<T, AccountError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_correlation_ids_are_unique() {
        let err1 = AccountError::account_not_found("0x123");
        let err2 = AccountError::account_not_found("0x456");
        assert_ne!(err1.correlation_id(), err2.correlation_id());
    }

    #[test]
    fn test_error_display() {
        let err = AccountError::account_not_found("0x1234567890abcdef");
        let display = err.to_string();
        assert!(display.contains("Account not found"));
        assert!(display.contains("0x1234567890abcdef"));
        assert!(display.contains("correlation"));
    }

    #[test]
    fn test_error_context_builder() {
        let ctx = ErrorContext::new("create_account")
            .with_account("0xabcd")
            .with_user_action("clicked create button")
            .with_network("mainnet");

        assert_eq!(ctx.operation, "create_account");
        assert_eq!(ctx.account_id, Some("0xabcd".to_string()));
        assert_eq!(ctx.user_action, Some("clicked create button".to_string()));
        assert_eq!(ctx.network, Some("mainnet".to_string()));
    }

    #[test]
    fn test_error_context_display() {
        let ctx = ErrorContext::new("unlock_account")
            .with_account("0x1234");

        let display = ctx.to_string();
        assert!(display.contains("unlock_account"));
        assert!(display.contains("0x1234"));
        assert!(display.contains("Correlation"));
    }

    #[test]
    fn test_all_error_variants_have_correlation_id() {
        let errors = vec![
            AccountError::account_not_found("addr"),
            AccountError::invalid_credentials(),
            AccountError::account_locked(),
            AccountError::import_failed("reason", "source"),
            AccountError::export_failed("reason"),
            AccountError::operation_failed("op", "reason"),
            AccountError::validation_failed("message"),
        ];

        for err in errors {
            // Correlation ID should be non-nil
            assert_ne!(err.correlation_id(), Uuid::nil());
        }
    }
}

/// Property-based tests for error context completeness
///
/// These tests validate **Property 8: Error Context Completeness** from design.md
/// and **Requirements 4.1, 4.2** from requirements.md:
///
/// - WHEN any error occurs, THE System SHALL create an Error_Context with correlation tracking
/// - THE Error_Context SHALL include operation name, timestamp, account ID (if applicable), and Correlation_ID
///
/// Uses proptest with minimum 100 iterations as specified in design.md.
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Strategy for generating random operation names
    fn operation_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z_]{1,30}")
            .expect("Valid regex")
    }

    /// Strategy for generating random account addresses (Ethereum-style)
    fn address_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("0x[a-fA-F0-9]{40}")
            .expect("Valid regex")
    }

    /// Strategy for generating random reason/message strings
    fn reason_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z0-9 ]{1,100}")
            .expect("Valid regex")
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 8: Error Context Completeness - Correlation ID Non-Nil
        ///
        /// *For any* error that occurs in the system, the error should include
        /// a complete ErrorContext with a non-nil correlation ID.
        ///
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop_account_not_found_has_valid_correlation_id(
            address in address_strategy()
        ) {
            let err = AccountError::account_not_found(&address);
            let correlation_id = err.correlation_id();

            // Correlation ID must be non-nil (Property 8)
            prop_assert_ne!(correlation_id, Uuid::nil());

            // Error display must contain correlation ID
            let display = err.to_string();
            prop_assert!(display.contains(&correlation_id.to_string()));
        }

        /// Property 8: Error Context Completeness - All Variants Have Correlation IDs
        ///
        /// *For any* error variant created with random inputs, the error should
        /// include a complete correlation ID.
        ///
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop_all_error_variants_have_correlation_id(
            address in address_strategy(),
            reason in reason_strategy(),
            operation in operation_strategy(),
            message in reason_strategy(),
        ) {
            // Create all error variants with generated inputs
            let errors = vec![
                AccountError::account_not_found(&address),
                AccountError::invalid_credentials(),
                AccountError::account_locked(),
                AccountError::import_failed(&reason, "seed_phrase"),
                AccountError::import_failed(&reason, "private_key"),
                AccountError::import_failed(&reason, "metamask"),
                AccountError::export_failed(&reason),
                AccountError::operation_failed(&operation, &reason),
                AccountError::validation_failed(&message),
            ];

            for err in errors {
                // All errors must have non-nil correlation ID
                prop_assert_ne!(err.correlation_id(), Uuid::nil());
            }
        }

        /// Property 8: Error Context Completeness - Unique Correlation IDs
        ///
        /// *For any* set of errors created, each should have a unique correlation ID.
        /// This ensures correlation IDs can be used for debugging across components.
        ///
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop_correlation_ids_are_unique(
            address in address_strategy(),
            reason in reason_strategy(),
        ) {
            let mut correlation_ids = HashSet::new();

            // Create multiple errors and collect their correlation IDs
            let errors = vec![
                AccountError::account_not_found(&address),
                AccountError::account_not_found(&address), // Same input, different instance
                AccountError::invalid_credentials(),
                AccountError::invalid_credentials(),
                AccountError::import_failed(&reason, "source1"),
                AccountError::import_failed(&reason, "source2"),
            ];

            for err in errors {
                let id = err.correlation_id();
                // Each correlation ID should be unique
                prop_assert!(correlation_ids.insert(id), "Duplicate correlation ID found: {}", id);
            }
        }

        /// Property 8: Error Context Completeness - ErrorContext Has Required Fields
        ///
        /// *For any* ErrorContext created, it must include operation name, timestamp,
        /// and correlation ID. Account ID is optional per requirements.
        ///
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop_error_context_has_required_fields(
            operation in operation_strategy()
        ) {
            let before = Utc::now();
            let ctx = ErrorContext::new(&operation);
            let after = Utc::now();

            // Operation name must match input
            prop_assert_eq!(&ctx.operation, &operation);

            // Correlation ID must be non-nil
            prop_assert_ne!(ctx.correlation_id, Uuid::nil());

            // Timestamp must be between before and after
            prop_assert!(ctx.timestamp >= before);
            prop_assert!(ctx.timestamp <= after);
        }

        /// Property 8: Error Context Completeness - Builder Enrichment Preserves Fields
        ///
        /// *For any* ErrorContext enriched with builder methods, all fields should
        /// be preserved and accessible.
        ///
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop_error_context_builder_preserves_fields(
            operation in operation_strategy(),
            account in address_strategy(),
            user_action in reason_strategy(),
            network in prop::string::string_regex("[a-z]{1,20}").expect("Valid regex"),
        ) {
            let ctx = ErrorContext::new(&operation)
                .with_account(&account)
                .with_user_action(&user_action)
                .with_network(&network);

            // All fields must be preserved
            prop_assert_eq!(&ctx.operation, &operation);
            prop_assert_eq!(ctx.account_id, Some(account));
            prop_assert_eq!(ctx.user_action, Some(user_action));
            prop_assert_eq!(ctx.network, Some(network));

            // Correlation ID must still be valid
            prop_assert_ne!(ctx.correlation_id, Uuid::nil());
        }

        /// Property 8: Error Context Completeness - Display Includes Correlation
        ///
        /// *For any* ErrorContext, its Display output must include the correlation ID
        /// for debugging purposes.
        ///
        /// Validates: Requirements 4.1, 4.2
        #[test]
        fn prop_error_context_display_includes_correlation(
            operation in operation_strategy()
        ) {
            let ctx = ErrorContext::new(&operation);
            let display = ctx.to_string();

            // Display must contain correlation ID
            prop_assert!(display.contains(&ctx.correlation_id.to_string()));

            // Display must contain operation name
            prop_assert!(display.contains(&operation));

            // Display must contain "Correlation" keyword
            prop_assert!(display.contains("Correlation"));
        }
    }
}
