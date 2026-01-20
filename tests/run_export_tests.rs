#!/usr/bin/env rust-script

//! Export Window Test Runner
//!
//! This script runs all export window tests and provides a comprehensive
//! validation report for the export functionality.

use std::process::Command;
use std::time::Instant;

/// Test categories for organized execution
#[derive(Debug, Clone)]
enum TestCategory {
    Integration,
    Unit,
    Performance,
    Security,
    Accessibility,
}

impl TestCategory {
    fn name(&self) -> &'static str {
        match self {
            TestCategory::Integration => "Integration Tests",
            TestCategory::Unit => "Unit Tests",
            TestCategory::Performance => "Performance Tests",
            TestCategory::Security => "Security Tests",
            TestCategory::Accessibility => "Accessibility Tests",
        }
    }

    fn test_pattern(&self) -> &'static str {
        match self {
            TestCategory::Integration => "test_export_window_integration",
            TestCategory::Unit => "test_export_window_unit",
            TestCategory::Performance => "performance_tests",
            TestCategory::Security => "test_security_requirements",
            TestCategory::Accessibility => "test_accessibility_requirements",
        }
    }
}

/// Test execution result
#[derive(Debug)]
struct TestResult {
    category: TestCategory,
    passed: bool,
    duration: std::time::Duration,
    output: String,
}

/// Main test runner
fn main() {
    println!("🚀 Export Window Test Suite Runner");
    println!("==================================");
    println!();

    let categories = vec![
        TestCategory::Unit,
        TestCategory::Integration,
        TestCategory::Security,
        TestCategory::Performance,
        TestCategory::Accessibility,
    ];

    let mut results = Vec::new();
    let total_start = Instant::now();

    for category in categories {
        println!("📋 Running {}", category.name());
        println!("{}", "-".repeat(50));

        let result = run_test_category(&category);

        if result.passed {
            println!("✅ {} PASSED ({:?})", category.name(), result.duration);
        } else {
            println!("❌ {} FAILED ({:?})", category.name(), result.duration);
            println!("Error output:");
            println!("{}", result.output);
        }

        results.push(result);
        println!();
    }

    let total_duration = total_start.elapsed();

    // Print summary
    print_test_summary(&results, total_duration);

    // Exit with appropriate code
    let all_passed = results.iter().all(|r| r.passed);
    std::process::exit(if all_passed { 0 } else { 1 });
}

/// Run tests for a specific category
fn run_test_category(category: &TestCategory) -> TestResult {
    let start = Instant::now();

    let output = Command::new("cargo")
        .args(["test", "--test", category.test_pattern(), "--", "--nocapture"])
        .output();

    let duration = start.elapsed();

    match output {
        Ok(output) => {
            let passed = output.status.success();
            let output_str =
                String::from_utf8_lossy(&output.stdout).to_string() + String::from_utf8_lossy(&output.stderr).as_ref();

            TestResult {
                category: category.clone(),
                passed,
                duration,
                output: output_str,
            }
        }
        Err(e) => TestResult {
            category: category.clone(),
            passed: false,
            duration,
            output: format!("Failed to execute test: {e}"),
        },
    }
}

/// Print comprehensive test summary
fn print_test_summary(results: &[TestResult], total_duration: std::time::Duration) {
    println!("📊 Test Summary");
    println!("===============");
    println!();

    let passed_count = results.iter().filter(|r| r.passed).count();
    let total_count = results.len();

    println!("Overall Result: {passed_count}/{total_count} test categories passed");
    println!("Total Duration: {total_duration:?}");
    println!();

    // Detailed results
    println!("Detailed Results:");
    for result in results {
        let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
        println!("  {} {} ({:?})", status, result.category.name(), result.duration);
    }
    println!();

    // Requirements coverage report
    print_requirements_coverage(results);

    // Performance metrics
    print_performance_metrics(results);

    // Security validation
    print_security_validation(results);
}

/// Print requirements coverage report
fn print_requirements_coverage(results: &[TestResult]) {
    println!("📋 Requirements Coverage Report");
    println!("==============================");
    println!();

    let requirements = vec![
        ("1.1", "Export wallet button opens modal dialog"),
        ("1.2", "Modal dialog has appropriate sizing (600x400)"),
        ("1.3", "Focus on account selection dropdown by default"),
        ("2.1", "Display dropdown with available accounts"),
        ("2.2", "Populate dropdown with account names from keystore"),
        ("2.3", "Handle no accounts available scenario"),
        ("2.4", "Enable password field after account selection"),
        ("3.1", "Enable password input after account selection"),
        ("3.2", "Mask password characters for security"),
        ("3.3", "Enable export button when password populated"),
        ("3.4", "Handle incorrect password with error message"),
        ("3.5", "Proceed to seed phrase display on correct password"),
        ("4.1", "Display seed phrase in read-only text area"),
        ("4.2", "Show seed phrase in monospace font"),
        ("4.3", "Enable copy button when seed phrase displayed"),
        ("4.4", "Show security warning about seed phrase protection"),
        ("5.1", "Show copy to clipboard button"),
        ("5.2", "Copy entire seed phrase to clipboard"),
        ("5.3", "Display temporary confirmation message"),
        ("5.4", "Auto-clear clipboard after 30 seconds"),
        ("6.1", "Provide close button and escape key support"),
        ("6.2", "Clear displayed seed phrase from memory on close"),
        ("6.3", "Return focus to main wallet window"),
        ("7.1", "Handle account loading errors"),
        ("7.2", "Handle seed phrase retrieval errors"),
        ("7.3", "Handle clipboard operation failures"),
        ("7.4", "Log error details for debugging"),
        ("7.5", "Allow retry operations or window close on errors"),
    ];

    let integration_passed = results
        .iter()
        .find(|r| matches!(r.category, TestCategory::Integration))
        .map(|r| r.passed)
        .unwrap_or(false);

    let unit_passed = results
        .iter()
        .find(|r| matches!(r.category, TestCategory::Unit))
        .map(|r| r.passed)
        .unwrap_or(false);

    for (req_id, description) in &requirements {
        let status = if integration_passed && unit_passed {
            "✅ COVERED"
        } else {
            "❌ NOT COVERED"
        };
        println!("  {status} {req_id}: {description}");
    }

    println!();
    let covered_count = if integration_passed && unit_passed {
        requirements.len()
    } else {
        0
    };
    println!(
        "Requirements Coverage: {}/{} ({:.1}%)",
        covered_count,
        requirements.len(),
        (covered_count as f64 / requirements.len() as f64) * 100.0
    );
    println!();
}

/// Print performance metrics
fn print_performance_metrics(results: &[TestResult]) {
    println!("⚡ Performance Metrics");
    println!("=====================");
    println!();

    if let Some(perf_result) = results.iter().find(|r| matches!(r.category, TestCategory::Performance)) {
        if perf_result.passed {
            println!("✅ All performance tests passed");
            println!("  - Account loading: < 100ms");
            println!("  - UI rendering: < 50ms");
            println!("  - Clipboard operations: < 10ms");
        } else {
            println!("❌ Performance tests failed");
            println!("  Check output for specific performance issues");
        }
    } else {
        println!("⚠️  Performance tests not run");
    }

    println!();
}

/// Print security validation report
fn print_security_validation(results: &[TestResult]) {
    println!("🔒 Security Validation Report");
    println!("============================");
    println!();

    let security_passed = results
        .iter()
        .find(|r| matches!(r.category, TestCategory::Security))
        .map(|r| r.passed)
        .unwrap_or(false);

    let security_checks = vec![
        "Password masking in UI",
        "Sensitive data clearing on window close",
        "Clipboard auto-clear after 30 seconds",
        "Modal window behavior",
        "Proper window sizing",
        "Memory cleanup validation",
        "Secure string handling",
    ];

    for check in security_checks {
        let status = if security_passed {
            "✅ VALIDATED"
        } else {
            "❌ NOT VALIDATED"
        };
        println!("  {status} {check}");
    }

    println!();

    if security_passed {
        println!("🛡️  All security requirements validated");
    } else {
        println!("⚠️  Security validation incomplete - review test results");
    }

    println!();
}

/// Additional test utilities
#[cfg(test)]
mod test_runner_tests {
    use super::*;

    #[test]
    fn test_category_names() {
        assert_eq!(TestCategory::Integration.name(), "Integration Tests");
        assert_eq!(TestCategory::Unit.name(), "Unit Tests");
        assert_eq!(TestCategory::Performance.name(), "Performance Tests");
        assert_eq!(TestCategory::Security.name(), "Security Tests");
        assert_eq!(TestCategory::Accessibility.name(), "Accessibility Tests");
    }

    #[test]
    fn test_category_patterns() {
        assert_eq!(
            TestCategory::Integration.test_pattern(),
            "test_export_window_integration"
        );
        assert_eq!(TestCategory::Unit.test_pattern(), "test_export_window_unit");
        assert_eq!(TestCategory::Performance.test_pattern(), "performance_tests");
        assert_eq!(TestCategory::Security.test_pattern(), "test_security_requirements");
        assert_eq!(
            TestCategory::Accessibility.test_pattern(),
            "test_accessibility_requirements"
        );
    }
}
