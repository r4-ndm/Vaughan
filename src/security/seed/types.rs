//! Core types for seed phrase management
//!
//! This module contains data structures, enums, and configuration types
//! used throughout the seed management system.

use crate::error::{Result, SecurityError};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;

use alloy::primitives::Address;

// ============================================================================
// SecureSeed - Wrapper with automatic zeroization
// ============================================================================

/// Secure wrapper for seed phrases with automatic zeroization
#[derive(Clone)]
pub struct SecureSeed(Secret<[u8; 64]>);

impl SecureSeed {
    /// Create from raw seed bytes
    pub fn from_bytes(seed_bytes: [u8; 64]) -> Self {
        Self(Secret::new(seed_bytes))
    }

    /// Expose the seed bytes (use with extreme caution)
    pub fn expose_seed(&self) -> &[u8; 64] {
        self.0.expose_secret()
    }
}

impl fmt::Debug for SecureSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureSeed").field("seed", &"[REDACTED]").finish()
    }
}

// ============================================================================
// SeedStrength - Entropy levels for seed phrases
// ============================================================================

/// Seed phrase strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedStrength {
    /// 12 words (128 bits entropy)
    Words12 = 12,
    /// 15 words (160 bits entropy)
    Words15 = 15,
    /// 18 words (192 bits entropy)
    Words18 = 18,
    /// 21 words (224 bits entropy)
    Words21 = 21,
    /// 24 words (256 bits entropy) - Most secure
    Words24 = 24,
}

impl SeedStrength {
    pub fn word_count(self) -> usize {
        self as usize
    }

    pub fn entropy_bits(self) -> usize {
        match self {
            SeedStrength::Words12 => 128,
            SeedStrength::Words15 => 160,
            SeedStrength::Words18 => 192,
            SeedStrength::Words21 => 224,
            SeedStrength::Words24 => 256,
        }
    }

    /// Create SeedStrength from word count
    pub fn from_word_count(count: usize) -> Result<Self> {
        match count {
            12 => Ok(SeedStrength::Words12),
            15 => Ok(SeedStrength::Words15),
            18 => Ok(SeedStrength::Words18),
            21 => Ok(SeedStrength::Words21),
            24 => Ok(SeedStrength::Words24),
            _ => Err(SecurityError::InvalidSeedPhrase {
                reason: format!("Invalid word count: {count}. Must be 12, 15, 18, 21, or 24 words"),
            }
            .into()),
        }
    }

    /// Get security level description
    pub fn security_level(self) -> &'static str {
        match self {
            SeedStrength::Words12 => "Standard Security",
            SeedStrength::Words15 => "Enhanced Security",
            SeedStrength::Words18 => "High Security",
            SeedStrength::Words21 => "Very High Security",
            SeedStrength::Words24 => "Maximum Security",
        }
    }

    /// Get security recommendation
    pub fn security_recommendation(self) -> &'static str {
        match self {
            SeedStrength::Words12 => "Suitable for most users. Provides strong security for typical cryptocurrency holdings.",
            SeedStrength::Words15 => "Enhanced security with additional entropy. Good for users with larger holdings.",
            SeedStrength::Words18 => "High security level. Recommended for institutional or high-value accounts.",
            SeedStrength::Words21 => "Very high security. Suitable for long-term storage of significant assets.",
            SeedStrength::Words24 => "Maximum security with 256-bit entropy. Recommended for maximum protection and future-proofing against quantum computing advances.",
        }
    }

    /// Check if this strength is recommended for the given use case
    pub fn is_recommended_for_use_case(self, use_case: SecurityUseCase) -> bool {
        match use_case {
            SecurityUseCase::Personal => matches!(self, SeedStrength::Words12 | SeedStrength::Words24),
            SecurityUseCase::Business => matches!(
                self,
                SeedStrength::Words18 | SeedStrength::Words21 | SeedStrength::Words24
            ),
            SecurityUseCase::Institutional => matches!(self, SeedStrength::Words21 | SeedStrength::Words24),
            SecurityUseCase::LongTermStorage => matches!(self, SeedStrength::Words24),
        }
    }

    /// Get estimated time to brute force (for educational purposes)
    pub fn brute_force_time_estimate(self) -> &'static str {
        match self {
            SeedStrength::Words12 => "~37 million years (with current technology)",
            SeedStrength::Words15 => "~2.4 billion years",
            SeedStrength::Words18 => "~160 trillion years",
            SeedStrength::Words21 => "~10 quintillion years",
            SeedStrength::Words24 => "~13 septillion years (quantum-resistant)",
        }
    }
}

// Display implementation for GUI integration
impl std::fmt::Display for SeedStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} words", self.word_count())
    }
}

// ============================================================================
// SecurityUseCase - Use case categories for seed phrase selection
// ============================================================================

/// Security use cases for seed phrase strength selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityUseCase {
    Personal,
    Business,
    Institutional,
    LongTermStorage,
}

// ============================================================================
// SeedImportValidation - Validation result for seed phrase imports
// ============================================================================

/// Seed phrase import validation result
#[derive(Debug, Clone)]
pub struct SeedImportValidation {
    pub is_valid: bool,
    pub word_count: usize,
    pub strength: Option<SeedStrength>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<WordSuggestion>,
    pub checksum_valid: bool,
    pub entropy_bits: Option<usize>,
}

impl SeedImportValidation {
    pub fn valid(word_count: usize, strength: SeedStrength) -> Self {
        Self {
            is_valid: true,
            word_count,
            strength: Some(strength),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            checksum_valid: true,
            entropy_bits: Some(strength.entropy_bits()),
        }
    }

    pub fn invalid(word_count: usize, errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            word_count,
            strength: None,
            errors,
            warnings: Vec::new(),
            suggestions: Vec::new(),
            checksum_valid: false,
            entropy_bits: None,
        }
    }

    pub fn with_suggestions(mut self, suggestions: Vec<WordSuggestion>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Get overall import confidence score (0-100)
    pub fn confidence_score(&self) -> u8 {
        if !self.is_valid {
            return 0;
        }

        let mut score = 100u8;

        // Reduce score for warnings
        score = score.saturating_sub((self.warnings.len() * 10).min(30) as u8);

        // Reduce score if suggestions were needed
        score = score.saturating_sub((self.suggestions.len() * 5).min(20) as u8);

        score
    }
}

/// Word suggestion for typo correction
#[derive(Debug, Clone)]
pub struct WordSuggestion {
    pub position: usize,
    pub original_word: String,
    pub suggested_words: Vec<String>,
    pub confidence: f32, // 0.0 to 1.0
}

// ============================================================================
// Export Types - Formats and options for seed phrase export
// ============================================================================

/// Export format for seed phrases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Plain text seed phrase (most common)
    PlainText,
    /// QR code compatible format
    QrCode,
    /// JSON format with metadata
    Json,
    /// Encrypted export with custom password
    EncryptedJson,
}

/// Export options for seed phrase export
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_derivation_paths: bool,
    pub custom_export_name: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::PlainText,
            include_metadata: false,
            include_derivation_paths: false,
            custom_export_name: None,
        }
    }
}

/// Export result containing the exported data and metadata
#[derive(Debug)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub data: SecretString,
    pub metadata: Option<ExportMetadata>,
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
    pub security_warnings: Vec<String>,
}

/// Metadata included with exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub wallet_name: String,
    pub seed_strength: SeedStrength,
    pub creation_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub derivation_paths: Vec<DerivationPathConfig>,
    pub export_version: String,
}

// ============================================================================
// Derivation Path Types - Configuration for HD wallet derivation
// ============================================================================

/// Derivation path configuration for seed phrase import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationPathConfig {
    pub path: String,
    pub description: String,
    pub is_standard: bool,
}

impl DerivationPathConfig {
    /// Standard Ethereum derivation path
    pub fn ethereum_standard() -> Self {
        Self {
            path: "m/44'/60'/0'/0/0".to_string(),
            description: "Standard Ethereum (BIP44)".to_string(),
            is_standard: true,
        }
    }

    /// Ethereum account base derivation path
    pub fn ethereum_account_base() -> Self {
        Self {
            path: "m/44'/60'/0'/0".to_string(),
            description: "Ethereum Account Base".to_string(),
            is_standard: true,
        }
    }

    /// Ledger Live derivation path
    pub fn ledger_live() -> Self {
        Self {
            path: "m/44'/60'/0'/0".to_string(),
            description: "Ledger Live".to_string(),
            is_standard: true,
        }
    }

    /// Legacy derivation path
    pub fn legacy() -> Self {
        Self {
            path: "m/44'/60'/0'".to_string(),
            description: "Legacy".to_string(),
            is_standard: false,
        }
    }

    /// Custom derivation path
    pub fn custom(path: String, description: String) -> Self {
        Self {
            path,
            description,
            is_standard: false,
        }
    }

    /// Get all standard derivation paths
    pub fn standard_paths() -> Vec<Self> {
        vec![Self::ethereum_standard(), Self::ledger_live(), Self::legacy()]
    }
}

/// Multiple account derivation result
#[derive(Debug, Clone)]
pub struct MultiAccountDerivation {
    pub accounts: Vec<DerivedAccount>,
    pub derivation_path: String,
    pub total_derived: usize,
}

/// Single derived account information
#[derive(Debug, Clone)]
pub struct DerivedAccount {
    pub index: u32,
    pub address: Address,
    pub derivation_path: String,
    pub public_key: Option<String>,
}

// ============================================================================
// Import Configuration - Settings for seed phrase import
// ============================================================================

/// Seed phrase import configuration
#[derive(Debug, Clone)]
pub struct SeedImportConfig {
    pub allow_weak_phrases: bool,
    pub max_typo_corrections: usize,
    pub require_checksum_validation: bool,
    pub enable_fuzzy_matching: bool,
    pub derivation_paths: Vec<DerivationPathConfig>,
    pub max_accounts_to_derive: u32,
}

impl Default for SeedImportConfig {
    fn default() -> Self {
        Self {
            allow_weak_phrases: false,
            max_typo_corrections: 3,
            require_checksum_validation: true,
            enable_fuzzy_matching: true,
            derivation_paths: DerivationPathConfig::standard_paths(),
            max_accounts_to_derive: 10,
        }
    }
}

// ============================================================================
// SeedAnalysis - Comprehensive analysis of a seed phrase
// ============================================================================

/// Comprehensive analysis of a seed phrase
#[derive(Debug, Clone)]
pub struct SeedAnalysis {
    pub strength: SeedStrength,
    pub entropy_bits: usize,
    pub security_level: String,
    pub recommendation: String,
    pub brute_force_estimate: String,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

impl SeedAnalysis {
    /// Check if the seed phrase is suitable for the given use case
    pub fn is_suitable_for_use_case(&self, use_case: SecurityUseCase) -> bool {
        self.strength.is_recommended_for_use_case(use_case) && self.warnings.is_empty()
    }

    /// Get overall security score (0-100)
    pub fn security_score(&self) -> u8 {
        let base_score: u8 = match self.strength {
            SeedStrength::Words12 => 70,
            SeedStrength::Words15 => 80,
            SeedStrength::Words18 => 85,
            SeedStrength::Words21 => 90,
            SeedStrength::Words24 => 100,
        };

        // Reduce score for warnings
        let warning_penalty = (self.warnings.len() * 10).min(30) as u8;
        base_score.saturating_sub(warning_penalty)
    }
}

// ============================================================================
// Backup Types - Structures for seed phrase backup
// ============================================================================

/// Seed phrase backup structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedBackup {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; 32],
    pub nonce: [u8; 12],
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub version: u32,
}

impl SeedBackup {
    /// Get backup age in days
    pub fn age_days(&self) -> i64 {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.created_at).num_days()
    }

    /// Check if backup is recent (less than 30 days old)
    pub fn is_recent(&self) -> bool {
        self.age_days() < 30
    }
}
