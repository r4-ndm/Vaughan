//! Error reporting and metrics collection
//!
//! This module provides utilities for collecting error metrics and generating reports.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

use super::{ErrorCategory, ErrorSeverity, VaughanError};

/// Error statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    pub recovery_success_rate: f64,
    pub most_common_errors: Vec<(String, u64)>,
}

/// Error reporter for collecting metrics
pub struct ErrorReporter {
    stats: Arc<Mutex<ErrorStatsInternal>>,
}

#[derive(Debug, Default)]
struct ErrorStatsInternal {
    total_errors: u64,
    errors_by_severity: HashMap<ErrorSeverity, u64>,
    errors_by_category: HashMap<ErrorCategory, u64>,
    error_types: HashMap<String, u64>,
    recovery_attempts: u64,
    recovery_successes: u64,
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(ErrorStatsInternal::default())),
        }
    }

    /// Record an error occurrence
    pub fn record_error(&self, error: &VaughanError) {
        let mut stats = self.stats.lock().unwrap();

        stats.total_errors += 1;

        let context = error.context();
        *stats.errors_by_severity.entry(context.severity).or_insert(0) += 1;
        *stats.errors_by_category.entry(context.category).or_insert(0) += 1;

        let error_type = std::any::type_name_of_val(error).to_string();
        *stats.error_types.entry(error_type).or_insert(0) += 1;
    }

    /// Record a recovery attempt
    pub fn record_recovery_attempt(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.recovery_attempts += 1;
    }

    /// Record a successful recovery
    pub fn record_recovery_success(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.recovery_successes += 1;
    }

    /// Get current error statistics
    pub fn get_stats(&self) -> ErrorStats {
        let stats = self.stats.lock().unwrap();

        let recovery_success_rate = if stats.recovery_attempts > 0 {
            stats.recovery_successes as f64 / stats.recovery_attempts as f64
        } else {
            0.0
        };

        let mut most_common_errors: Vec<(String, u64)> =
            stats.error_types.iter().map(|(k, v)| (k.clone(), *v)).collect();
        most_common_errors.sort_by(|a, b| b.1.cmp(&a.1));
        most_common_errors.truncate(10); // Top 10 most common errors

        ErrorStats {
            total_errors: stats.total_errors,
            errors_by_severity: stats.errors_by_severity.clone(),
            errors_by_category: stats.errors_by_category.clone(),
            recovery_success_rate,
            most_common_errors,
        }
    }

    /// Reset all statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ErrorStatsInternal::default();
    }

    /// Generate a human-readable error report
    pub fn generate_report(&self) -> String {
        let stats = self.get_stats();

        let mut report = String::new();
        report.push_str("=== Vaughan Error Report ===\n\n");

        report.push_str(&format!("Total Errors: {}\n", stats.total_errors));
        report.push_str(&format!(
            "Recovery Success Rate: {:.1}%\n\n",
            stats.recovery_success_rate * 100.0
        ));

        report.push_str("Errors by Severity:\n");
        for (severity, count) in &stats.errors_by_severity {
            report.push_str(&format!("  {severity:?}: {count}\n"));
        }

        report.push_str("\nErrors by Category:\n");
        for (category, count) in &stats.errors_by_category {
            report.push_str(&format!("  {category:?}: {count}\n"));
        }

        if !stats.most_common_errors.is_empty() {
            report.push_str("\nMost Common Errors:\n");
            for (i, (error_type, count)) in stats.most_common_errors.iter().enumerate() {
                let short_name = error_type.split("::").last().unwrap_or(error_type);
                report.push_str(&format!("  {}. {}: {} occurrences\n", i + 1, short_name, count));
            }
        }

        report
    }
}

/// Global error reporter instance
static ERROR_REPORTER: OnceLock<ErrorReporter> = OnceLock::new();

/// Initialize the global error reporter
pub fn init_error_reporter() {
    ERROR_REPORTER.get_or_init(ErrorReporter::new);
}

/// Get the global error reporter
pub fn get_error_reporter() -> &'static ErrorReporter {
    ERROR_REPORTER.get_or_init(ErrorReporter::new)
}

/// Convenience function to record an error
pub fn record_error(error: &VaughanError) {
    get_error_reporter().record_error(error);
}

/// Convenience function to record a recovery attempt
pub fn record_recovery_attempt() {
    get_error_reporter().record_recovery_attempt();
}

/// Convenience function to record a recovery success
pub fn record_recovery_success() {
    get_error_reporter().record_recovery_success();
}
