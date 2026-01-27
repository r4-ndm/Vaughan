//! Privacy Mode and Data Sanitization
//!
//! Implements Property 19: Privacy Mode Log Sanitization
//!
//! This module provides privacy-aware logging with automatic sanitization
//! of sensitive data like private keys, seed phrases, and passwords.

use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

/// Global privacy mode setting
static PRIVACY_MODE: AtomicBool = AtomicBool::new(false);

/// Global opt-out setting
static OPT_OUT: AtomicBool = AtomicBool::new(false);

/// Privacy mode configuration for log sanitization
///
/// Controls whether sensitive data is redacted from logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PrivacyMode {
    /// Privacy mode enabled - sanitize all sensitive data
    #[default]
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


/// Set the global privacy mode
///
/// # Arguments
///
/// * `enabled` - Whether to enable privacy mode
pub fn set_privacy_mode(enabled: bool) {
    PRIVACY_MODE.store(enabled, Ordering::SeqCst);
    if enabled {
        info!("🔒 Privacy mode enabled - sensitive data will be redacted from logs");
    } else {
        warn!("⚠️ Privacy mode disabled - sensitive data may appear in logs");
    }
}

/// Get the current global privacy mode
///
/// Returns `true` if privacy mode is enabled.
pub fn get_privacy_mode() -> bool {
    PRIVACY_MODE.load(Ordering::SeqCst)
}

/// Set opt-out status for telemetry
///
/// # Arguments
///
/// * `opt_out` - Whether the user has opted out of telemetry
pub fn set_opt_out(opt_out: bool) {
    OPT_OUT.store(opt_out, Ordering::SeqCst);
    if opt_out {
        warn!("🚫 Telemetry opt-out enabled - no events will be recorded");
    } else {
        info!("✅ Telemetry opt-in confirmed");
    }
}

/// Check if user has opted out of telemetry
///
/// Returns `true` if the user has opted out.
pub fn is_opted_out() -> bool {
    OPT_OUT.load(Ordering::SeqCst)
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
///
/// # Arguments
///
/// * `data` - The data to potentially sanitize
/// * `data_type` - The type of sensitive data
///
/// # Returns
///
/// Either the original data (if privacy mode disabled) or a sanitized version
///
/// # Examples
///
/// ```ignore
/// use vaughan::telemetry::account_events::privacy::{sanitize, SensitiveDataType};
///
/// let key = "0x1234567890abcdef...";
/// let sanitized = sanitize(key, SensitiveDataType::PrivateKey);
/// // Returns: "[REDACTED:PRIVATE_KEY]"
/// ```
pub fn sanitize(data: &str, data_type: SensitiveDataType) -> String {
    if !get_privacy_mode() {
        return data.to_string();
    }

    match data_type {
        SensitiveDataType::PrivateKey => "[REDACTED:PRIVATE_KEY]".to_string(),
        SensitiveDataType::SeedPhrase => "[REDACTED:SEED_PHRASE]".to_string(),
        SensitiveDataType::Password => "[REDACTED:PASSWORD]".to_string(),
        SensitiveDataType::Address => {
            // Show first 6 and last 4 characters for debugging
            if data.len() > 10 {
                format!("{}...{}", &data[..6], &data[data.len() - 4..])
            } else {
                "[REDACTED:ADDRESS]".to_string()
            }
        }
        SensitiveDataType::TransactionData => "[REDACTED:TX_DATA]".to_string(),
        SensitiveDataType::Balance => "[REDACTED:BALANCE]".to_string(),
    }
}

/// Check if a string contains potentially sensitive patterns
///
/// Scans text for common sensitive data indicators.
///
/// # Arguments
///
/// * `text` - The text to check
///
/// # Returns
///
/// `true` if the text likely contains sensitive data
pub fn contains_sensitive_data(text: &str) -> bool {
    let lower = text.to_lowercase();

    // Check for common sensitive patterns
    let patterns = [
        "private",
        "seed",
        "mnemonic",
        "password",
        "secret",
        "key",
        "0x", // Hex prefixes often indicate keys
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_mode_default() {
        let mode = PrivacyMode::default();
        assert_eq!(mode, PrivacyMode::Enabled);
        assert!(mode.is_enabled());
    }

    #[test]
    fn test_privacy_mode_disabled() {
        let mode = PrivacyMode::Disabled;
        assert!(!mode.is_enabled());
    }

    #[test]
    fn test_set_get_privacy_mode() {
        set_privacy_mode(true);
        assert!(get_privacy_mode());

        set_privacy_mode(false);
        assert!(!get_privacy_mode());

        // Reset to default
        set_privacy_mode(true);
    }

    #[test]
    fn test_opt_out() {
        set_opt_out(false);
        assert!(!is_opted_out());

        set_opt_out(true);
        assert!(is_opted_out());

        // Reset to default
        set_opt_out(false);
    }

    #[test]
    fn test_sanitize_private_key() {
        set_privacy_mode(true);
        let key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let sanitized = sanitize(key, SensitiveDataType::PrivateKey);
        assert_eq!(sanitized, "[REDACTED:PRIVATE_KEY]");
    }

    #[test]
    fn test_sanitize_address() {
        set_privacy_mode(true);
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let sanitized = sanitize(addr, SensitiveDataType::Address);
        assert!(sanitized.starts_with("0x742d"));
        assert!(sanitized.ends_with("bEb"));
        assert!(sanitized.contains("..."));
    }

    #[test]
    fn test_sanitize_disabled() {
        set_privacy_mode(false);
        let key = "secret_key_123";
        let sanitized = sanitize(key, SensitiveDataType::PrivateKey);
        assert_eq!(sanitized, "secret_key_123");

        // Reset
        set_privacy_mode(true);
    }

    #[test]
    fn test_contains_sensitive_data() {
        assert!(contains_sensitive_data("my private key"));
        assert!(contains_sensitive_data("seed phrase here"));
        assert!(contains_sensitive_data("password123"));
        assert!(contains_sensitive_data("0x1234"));
        assert!(!contains_sensitive_data("hello world"));
    }
}
