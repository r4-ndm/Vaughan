//! Seed phrase validation and word suggestion utilities
//!
//! This module provides comprehensive BIP39 seed phrase validation with:
//! - Word count verification
//! - BIP39 wordlist validation
//! - Checksum verification
//! - Typo correction with fuzzy matching
//! - Weak pattern detection (repeated words, sequential patterns)

use crate::error::{Result, SecurityError};
use bip39::Mnemonic;
use secrecy::{ExposeSecret, SecretString};
use std::collections::{HashMap, HashSet};

use super::types::{SeedImportConfig, SeedImportValidation, SeedStrength, WordSuggestion};

// ============================================================================
// Core Validation Functions
// ============================================================================

/// Validate a seed phrase using BIP39 standard
pub fn validate_seed_phrase(phrase: &SecretString) -> Result<()> {
    let phrase_str = phrase.expose_secret();

    // Validate using BIP39
    match Mnemonic::parse(phrase_str) {
        Ok(_) => {
            tracing::info!("Seed phrase validation successful");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Seed phrase validation failed: {}", e);
            Err(SecurityError::InvalidSeedPhrase {
                reason: format!("Invalid BIP39 mnemonic: {e}"),
            }
            .into())
        }
    }
}

/// Comprehensive seed phrase validation with detailed error reporting
pub fn validate_seed_phrase_comprehensive(phrase: &str, _config: &SeedImportConfig) -> Result<SeedImportValidation> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let word_count = words.len();

    // Check word count
    let strength = match word_count {
        12 => SeedStrength::Words12,
        15 => SeedStrength::Words15,
        18 => SeedStrength::Words18,
        21 => SeedStrength::Words21,
        24 => SeedStrength::Words24,
        _ => {
            return Ok(SeedImportValidation::invalid(
                word_count,
                vec![format!(
                    "Invalid word count: {}. Must be 12, 15, 18, 21, or 24 words.",
                    word_count
                )],
            ));
        }
    };

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Load BIP39 wordlist for validation
    let wordlist = get_bip39_wordlist_set();

    // Check each word against BIP39 wordlist
    for (index, word) in words.iter().enumerate() {
        if !wordlist.contains(word) {
            // Find suggestions for invalid words
            let word_suggestions = find_word_suggestions(word, &wordlist, 3);
            if !word_suggestions.is_empty() {
                suggestions.push(WordSuggestion {
                    position: index,
                    original_word: word.to_string(),
                    suggested_words: word_suggestions.clone(),
                    confidence: calculate_suggestion_confidence(word, &word_suggestions[0]),
                });
            }
            errors.push(format!("Invalid word at position {}: '{}'", index + 1, word));
        }
    }

    // If there are invalid words, return early with suggestions
    if !errors.is_empty() {
        return Ok(SeedImportValidation::invalid(word_count, errors).with_suggestions(suggestions));
    }

    // Validate BIP39 checksum
    let checksum_valid = match Mnemonic::parse(phrase) {
        Ok(_) => true,
        Err(e) => {
            errors.push(format!("BIP39 checksum validation failed: {e}"));
            false
        }
    };

    // Security warnings for weaker seed phrases
    match strength {
        SeedStrength::Words12 => {
            warnings.push(
                "12-word seed phrases provide standard security. Consider using 24 words for maximum security."
                    .to_string(),
            );
        }
        SeedStrength::Words15 | SeedStrength::Words18 | SeedStrength::Words21 => {
            warnings
                .push("Non-standard word count. While valid, 12 or 24 words are more commonly supported.".to_string());
        }
        SeedStrength::Words24 => {
            // No warnings for 24-word phrases
        }
    }

    // Check for common weak patterns
    check_weak_patterns(&words, &mut warnings);

    if errors.is_empty() && checksum_valid {
        Ok(SeedImportValidation::valid(word_count, strength).with_warnings(warnings))
    } else {
        Ok(SeedImportValidation::invalid(word_count, errors).with_warnings(warnings))
    }
}

/// Get BIP39 wordlist as a HashSet for fast lookup
pub fn get_bip39_wordlist_set() -> HashSet<&'static str> {
    bip39::Language::English.word_list().iter().cloned().collect()
}

// ============================================================================
// Word Suggestion Functions (Fuzzy Matching)
// ============================================================================

/// Find word suggestions using fuzzy matching
pub fn find_word_suggestions(word: &str, wordlist: &HashSet<&'static str>, max_suggestions: usize) -> Vec<String> {
    let mut suggestions = Vec::new();

    // First, try exact prefix matches
    for &valid_word in wordlist {
        if valid_word.starts_with(word) {
            suggestions.push(valid_word.to_string());
            if suggestions.len() >= max_suggestions {
                return suggestions;
            }
        }
    }

    // Then try Levenshtein distance matching
    let mut scored_suggestions: Vec<(String, usize)> = Vec::new();

    for &valid_word in wordlist {
        let distance = levenshtein_distance(word, valid_word);
        if distance <= 2 {
            // Allow up to 2 character differences
            scored_suggestions.push((valid_word.to_string(), distance));
        }
    }

    // Sort by distance (lower is better)
    scored_suggestions.sort_by_key(|(_, distance)| *distance);

    // Take the best suggestions
    for (suggestion, _) in scored_suggestions.into_iter().take(max_suggestions) {
        if !suggestions.contains(&suggestion) {
            suggestions.push(suggestion);
        }
    }

    suggestions
}

/// Calculate Levenshtein distance between two strings
#[allow(clippy::needless_range_loop)]
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    matrix[i + 1][j] + 1, // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Calculate confidence score for a word suggestion
pub fn calculate_suggestion_confidence(original: &str, suggestion: &str) -> f32 {
    let distance = levenshtein_distance(original, suggestion);
    let max_len = std::cmp::max(original.len(), suggestion.len());

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f32 / max_len as f32)
}

/// Apply corrections to seed phrase based on suggestions
pub fn apply_corrections(phrase: &str, validation: &SeedImportValidation, config: &SeedImportConfig) -> Result<String> {
    let mut words: Vec<String> = phrase.split_whitespace().map(|s| s.to_string()).collect();
    let mut corrections_applied = 0;

    for suggestion in &validation.suggestions {
        if corrections_applied >= config.max_typo_corrections {
            break;
        }

        if suggestion.confidence >= 0.7 && !suggestion.suggested_words.is_empty() {
            // Apply the best suggestion
            words[suggestion.position] = suggestion.suggested_words[0].clone();
            corrections_applied += 1;

            tracing::info!(
                "Applied correction at position {}: '{}' -> '{}'",
                suggestion.position + 1,
                suggestion.original_word,
                suggestion.suggested_words[0]
            );
        }
    }

    Ok(words.join(" "))
}

// ============================================================================
// Weak Pattern Detection
// ============================================================================

/// Check for weak patterns in seed phrases
pub fn check_weak_patterns(words: &[&str], warnings: &mut Vec<String>) {
    // Check for repeated words
    let mut word_counts = HashMap::new();
    for word in words {
        *word_counts.entry(*word).or_insert(0) += 1;
    }

    for (word, count) in word_counts {
        if count > 1 {
            warnings.push(format!("Word '{word}' appears {count} times. This reduces entropy."));
        }
    }

    // Check for sequential patterns (basic check)
    let wordlist: Vec<&str> = bip39::Language::English.word_list().to_vec();
    for i in 0..words.len().saturating_sub(2) {
        if let (Some(pos1), Some(pos2)) = (
            wordlist.iter().position(|&w| w == words[i]),
            wordlist.iter().position(|&w| w == words[i + 1]),
        ) {
            if pos2 == pos1 + 1 {
                warnings.push("Sequential words detected. This may indicate a non-random seed phrase.".to_string());
                break;
            }
        }
    }
}

/// Check seed phrase security requirements
pub fn check_seed_phrase_security(_phrase: &str, validation: &SeedImportValidation) -> Result<()> {
    if let Some(strength) = validation.strength {
        match strength {
            SeedStrength::Words12 => {
                // 12 words is acceptable for most use cases
            }
            SeedStrength::Words15 | SeedStrength::Words18 | SeedStrength::Words21 => {
                tracing::warn!("Non-standard seed phrase length: {} words", validation.word_count);
            }
            SeedStrength::Words24 => {
                // 24 words is the most secure
            }
        }
    }

    // Check for critical warnings that should block import
    for warning in &validation.warnings {
        if warning.contains("appears") && warning.contains("times") {
            return Err(SecurityError::InvalidSeedPhrase {
                reason: "Seed phrase contains repeated words, which significantly reduces security.".to_string(),
            }
            .into());
        }
    }

    Ok(())
}

// ============================================================================
// Preprocessing
// ============================================================================

/// Preprocess seed phrase by cleaning whitespace and normalizing format
pub fn preprocess_seed_phrase(phrase: &str) -> String {
    phrase.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}
