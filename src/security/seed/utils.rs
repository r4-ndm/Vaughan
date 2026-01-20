//! Utility functions for seed phrase operations

use secrecy::{ExposeSecret, SecretString};

/// Split seed phrase into individual words
pub fn split_seed_words(phrase: &SecretString) -> Vec<String> {
    phrase
        .expose_secret()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Check if a word is in the BIP39 wordlist
pub fn is_valid_bip39_word(word: &str) -> bool {
    bip39::Language::English.word_list().contains(&word)
}

/// Get BIP39 wordlist for autocomplete
pub fn get_bip39_wordlist() -> &'static [&'static str] {
    bip39::Language::English.word_list()
}

/// Calculate entropy bits from word count
pub fn entropy_bits_from_words(word_count: usize) -> Option<u32> {
    match word_count {
        12 => Some(128),
        15 => Some(160),
        18 => Some(192),
        21 => Some(224),
        24 => Some(256),
        _ => None,
    }
}
