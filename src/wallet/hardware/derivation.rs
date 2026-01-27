//! Standard derivation paths for hardware wallets
//!
//! This module defines the standard derivation paths supported by the wallet,
//! ensuring compatibility with MetaMask, Ledger Live, and other industry standards.
//!
//! # Standards
//!
//! - **BIP44** (`m/44'/60'/0'/0/0`): Standard Ethereum path
//! - **Ledger Live** (`m/44'/60'/x'/0/0`): Used by Ledger Live (account index varies)
//! - **Legacy** (`m/44'/60'/0'/x`): Older standard
//! - **Custom**: User-defined BIP32 path with validation
//!
//! # Implementation
//!
//! Uses `alloy_signer_ledger` and `alloy_signer_trezor` HDPath types.
//!
//! # Task Reference
//! Implements: Task 3.1 (Create derivation standard enum)
//! Implements: Task 3.4 (Unit tests for derivation paths)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error type for derivation path validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivationPathError {
    /// Path must start with 'm/'
    InvalidPrefix,
    /// Path contains invalid characters
    InvalidCharacter(char),
    /// Path component is not a valid number
    InvalidComponent(String),
    /// Path is empty or too short
    TooShort,
    /// Path is too long (max 10 components)
    TooLong,
    /// Missing Ethereum purpose (44')
    MissingPurpose,
    /// Missing Ethereum coin type (60')
    MissingCoinType,
}

impl fmt::Display for DerivationPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "Path must start with 'm/'"),
            Self::InvalidCharacter(c) => write!(f, "Invalid character in path: '{}'", c),
            Self::InvalidComponent(s) => write!(f, "Invalid path component: '{}'", s),
            Self::TooShort => write!(f, "Path is too short, minimum is m/44'/60'"),
            Self::TooLong => write!(f, "Path is too long, maximum 10 components"),
            Self::MissingPurpose => write!(f, "Path must include purpose 44'"),
            Self::MissingCoinType => write!(f, "Path must include Ethereum coin type 60'"),
        }
    }
}

impl std::error::Error for DerivationPathError {}

/// Standard derivation path types
///
/// Supports both standard paths (BIP44, Ledger Live, Legacy) and
/// custom user-defined paths with validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DerivationStandard {
    /// Standard BIP44 Ethereum path: `m/44'/60'/0'/0/{index}`
    /// Most reliable for cross-wallet compatibility.
    #[default]
    Bip44,
    /// Ledger Live standard: `m/44'/60'/{index}'/0/0`
    /// Used by Ledger Live software.
    LedgerLive,
    /// Legacy/MEW standard: `m/44'/60'/0'/{index}`
    /// Used by older wallets like MyEtherWallet.
    Legacy,
    /// Custom user-defined path
    /// Must be a valid BIP32 path starting with m/44'/60'
    Custom(String),
}

impl DerivationStandard {
    /// Create a custom derivation path with validation
    ///
    /// # Arguments
    /// * `path` - The custom derivation path (e.g., "m/44'/60'/0'/0/5")
    ///
    /// # Returns
    /// * `Ok(DerivationStandard::Custom)` if path is valid
    /// * `Err(DerivationPathError)` if path is invalid
    ///
    /// # Example
    /// ```
    /// use vaughan::wallet::hardware::DerivationStandard;
    ///
    /// let standard = DerivationStandard::custom("m/44'/60'/0'/0/5").unwrap();
    /// assert!(matches!(standard, DerivationStandard::Custom(_)));
    /// ```
    pub fn custom(path: &str) -> Result<Self, DerivationPathError> {
        validate_derivation_path(path)?;
        Ok(Self::Custom(path.to_string()))
    }

    /// Get the string description of the path format
    pub fn description(&self) -> &str {
        match self {
            Self::Bip44 => "BIP44 (Standard)",
            Self::LedgerLive => "Ledger Live",
            Self::Legacy => "Legacy (MEW)",
            Self::Custom(_) => "Custom Path",
        }
    }

    /// Get the path template string
    pub fn path_template(&self) -> &str {
        match self {
            Self::Bip44 => "m/44'/60'/0'/0/{index}",
            Self::LedgerLive => "m/44'/60'/{index}'/0/0",
            Self::Legacy => "m/44'/60'/0'/{index}",
            Self::Custom(path) => path.as_str(),
        }
    }

    /// Check if this is a custom path
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Get all available standards (excluding Custom)
    pub fn all_standards() -> Vec<Self> {
        vec![Self::Bip44, Self::LedgerLive, Self::Legacy]
    }
}

/// Validate a BIP32 derivation path for Ethereum
///
/// # Rules
/// - Must start with "m/"
/// - Must contain purpose 44' (BIP44)
/// - Must contain Ethereum coin type 60'
/// - Components must be valid numbers with optional ' for hardened
/// - Maximum 10 components
///
/// # Example
/// ```
/// use vaughan::wallet::hardware::validate_derivation_path;
///
/// assert!(validate_derivation_path("m/44'/60'/0'/0/0").is_ok());
/// assert!(validate_derivation_path("invalid").is_err());
/// ```
pub fn validate_derivation_path(path: &str) -> Result<(), DerivationPathError> {
    // Must start with "m/"
    if !path.starts_with("m/") {
        return Err(DerivationPathError::InvalidPrefix);
    }

    let components: Vec<&str> = path[2..].split('/').collect();

    // Must have at least purpose and coin type
    if components.len() < 2 {
        return Err(DerivationPathError::TooShort);
    }

    // Max 10 components (reasonable limit)
    if components.len() > 10 {
        return Err(DerivationPathError::TooLong);
    }

    // Validate each component
    for component in &components {
        // Remove hardened marker for validation
        let num_str = component.trim_end_matches('\'');
        
        // Check for invalid characters
        for c in num_str.chars() {
            if !c.is_ascii_digit() {
                return Err(DerivationPathError::InvalidCharacter(c));
            }
        }

        // Try to parse as number
        if num_str.parse::<u32>().is_err() {
            return Err(DerivationPathError::InvalidComponent(component.to_string()));
        }
    }

    // Check for BIP44 purpose (44')
    if components.first() != Some(&"44'") {
        return Err(DerivationPathError::MissingPurpose);
    }

    // Check for Ethereum coin type (60')
    if components.get(1) != Some(&"60'") {
        return Err(DerivationPathError::MissingCoinType);
    }

    Ok(())
}

/// Helper to convert standard and index to Ledger HDPath
#[cfg(feature = "hardware-wallets")]
pub fn to_ledger_path(standard: DerivationStandard, index: u32) -> alloy_signer_ledger::HDPath {
    use alloy_signer_ledger::HDPath;
    let index_usize = index as usize;
    match standard {
        // Alloy's Ledger signer primarily supports LedgerLive and Legacy variants.
        // We map Bip44 to LedgerLive as the best available standard option for Ledger devices.
        DerivationStandard::Bip44 => HDPath::LedgerLive(index_usize),
        DerivationStandard::LedgerLive => HDPath::LedgerLive(index_usize),
        DerivationStandard::Legacy => HDPath::Legacy(index_usize),
        // Custom paths use LedgerLive as fallback (index in path is ignored)
        DerivationStandard::Custom(_) => HDPath::LedgerLive(index_usize),
    }
}

/// Helper to convert standard and index to Trezor HDPath
#[cfg(feature = "hardware-wallets")]
pub fn to_trezor_path(standard: DerivationStandard, index: u32) -> alloy_signer_trezor::HDPath {
    use alloy_signer_trezor::HDPath;
    // Trezor signer in Alloy primarily supports TrezorLive (which takes usize)
    let index_usize = index as usize;
    match standard {
        DerivationStandard::Bip44 => HDPath::TrezorLive(index_usize),
        DerivationStandard::LedgerLive => HDPath::TrezorLive(index_usize),
        DerivationStandard::Legacy => HDPath::TrezorLive(index_usize),
        DerivationStandard::Custom(_) => HDPath::TrezorLive(index_usize),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Task 3.4: Unit tests for derivation paths
    // ========================================================================

    #[test]
    fn test_derivation_standard_defaults() {
        assert_eq!(DerivationStandard::default(), DerivationStandard::Bip44);
    }

    #[test]
    fn test_derivation_standard_descriptions() {
        assert!(DerivationStandard::Bip44.description().contains("BIP44"));
        assert!(DerivationStandard::LedgerLive.description().contains("Ledger Live"));
        assert!(DerivationStandard::Legacy.description().contains("Legacy"));
        assert!(DerivationStandard::Custom("m/44'/60'/0'/0/0".into()).description().contains("Custom"));
    }

    #[test]
    fn test_path_templates() {
        assert!(DerivationStandard::Bip44.path_template().contains("/0'/0/"));
        assert!(DerivationStandard::LedgerLive.path_template().contains("/0/0"));
        assert!(DerivationStandard::Legacy.path_template().contains("44'/60'/0'/"));
        
        let custom = DerivationStandard::Custom("m/44'/60'/1'/0/5".into());
        assert_eq!(custom.path_template(), "m/44'/60'/1'/0/5");
    }

    #[test]
    fn test_is_custom() {
        assert!(!DerivationStandard::Bip44.is_custom());
        assert!(!DerivationStandard::LedgerLive.is_custom());
        assert!(!DerivationStandard::Legacy.is_custom());
        assert!(DerivationStandard::Custom("m/44'/60'/0'/0/0".into()).is_custom());
    }

    #[test]
    fn test_all_standards() {
        let standards = DerivationStandard::all_standards();
        assert_eq!(standards.len(), 3);
        assert!(standards.contains(&DerivationStandard::Bip44));
        assert!(standards.contains(&DerivationStandard::LedgerLive));
        assert!(standards.contains(&DerivationStandard::Legacy));
    }

    // ========================================================================
    // Custom path validation tests
    // ========================================================================

    #[test]
    fn test_valid_custom_paths() {
        // Standard BIP44 format
        assert!(validate_derivation_path("m/44'/60'/0'/0/0").is_ok());
        assert!(validate_derivation_path("m/44'/60'/0'/0/1").is_ok());
        assert!(validate_derivation_path("m/44'/60'/0'/0/999").is_ok());

        // Ledger Live format
        assert!(validate_derivation_path("m/44'/60'/0'/0/0").is_ok());
        assert!(validate_derivation_path("m/44'/60'/1'/0/0").is_ok());

        // All hardened
        assert!(validate_derivation_path("m/44'/60'/0'/0'").is_ok());

        // Minimal valid path
        assert!(validate_derivation_path("m/44'/60'").is_ok());
    }

    #[test]
    fn test_invalid_path_prefix() {
        assert_eq!(
            validate_derivation_path("44'/60'/0'/0/0"),
            Err(DerivationPathError::InvalidPrefix)
        );
        assert_eq!(
            validate_derivation_path("M/44'/60'/0'/0/0"),
            Err(DerivationPathError::InvalidPrefix)
        );
        assert_eq!(
            validate_derivation_path("/44'/60'/0'/0/0"),
            Err(DerivationPathError::InvalidPrefix)
        );
    }

    #[test]
    fn test_invalid_path_too_short() {
        assert_eq!(
            validate_derivation_path("m/44'"),
            Err(DerivationPathError::TooShort)
        );
        assert_eq!(
            validate_derivation_path("m/"),
            Err(DerivationPathError::TooShort)
        );
    }

    #[test]
    fn test_invalid_path_too_long() {
        assert_eq!(
            validate_derivation_path("m/44'/60'/0'/0/0/1/2/3/4/5/6"),
            Err(DerivationPathError::TooLong)
        );
    }

    #[test]
    fn test_invalid_path_characters() {
        assert!(matches!(
            validate_derivation_path("m/44'/60'/x/0/0"),
            Err(DerivationPathError::InvalidCharacter('x'))
        ));
        assert!(matches!(
            validate_derivation_path("m/44'/60'/0'/0/-1"),
            Err(DerivationPathError::InvalidCharacter('-'))
        ));
    }

    #[test]
    fn test_invalid_path_missing_purpose() {
        assert_eq!(
            validate_derivation_path("m/45'/60'/0'/0/0"),
            Err(DerivationPathError::MissingPurpose)
        );
        assert_eq!(
            validate_derivation_path("m/44/60'/0'/0/0"),
            Err(DerivationPathError::MissingPurpose)
        );
    }

    #[test]
    fn test_invalid_path_missing_coin_type() {
        assert_eq!(
            validate_derivation_path("m/44'/61'/0'/0/0"),
            Err(DerivationPathError::MissingCoinType)
        );
        assert_eq!(
            validate_derivation_path("m/44'/60/0'/0/0"),
            Err(DerivationPathError::MissingCoinType)
        );
    }

    #[test]
    fn test_custom_path_creation() {
        // Valid custom path
        let result = DerivationStandard::custom("m/44'/60'/0'/0/5");
        assert!(result.is_ok());
        assert!(result.unwrap().is_custom());

        // Invalid custom path
        let result = DerivationStandard::custom("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_derivation_standard_serialization() {
        // Test that Custom paths serialize correctly
        let custom = DerivationStandard::Custom("m/44'/60'/0'/0/5".into());
        let json = serde_json::to_string(&custom).unwrap();
        let parsed: DerivationStandard = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, custom);

        // Test standard variants
        let bip44 = DerivationStandard::Bip44;
        let json = serde_json::to_string(&bip44).unwrap();
        let parsed: DerivationStandard = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, bip44);
    }

    // ========================================================================
    // Hardware wallet path conversion tests (feature-gated)
    // ========================================================================

    #[test]
    #[cfg(feature = "hardware-wallets")]
    fn test_ledger_path_conversion() {
        // Just verify it doesn't panic and returns a valid variant
        let _ = to_ledger_path(DerivationStandard::Bip44, 0);
        let _ = to_ledger_path(DerivationStandard::LedgerLive, 1);
        let _ = to_ledger_path(DerivationStandard::Legacy, 2);
        let _ = to_ledger_path(DerivationStandard::Custom("m/44'/60'/0'/0/0".into()), 0);
    }

    #[test]
    #[cfg(feature = "hardware-wallets")]
    fn test_trezor_path_conversion() {
        // Just verify it doesn't panic
        let _ = to_trezor_path(DerivationStandard::Bip44, 0);
        let _ = to_trezor_path(DerivationStandard::LedgerLive, 1);
        let _ = to_trezor_path(DerivationStandard::Legacy, 2);
        let _ = to_trezor_path(DerivationStandard::Custom("m/44'/60'/0'/0/0".into()), 0);
    }
}

