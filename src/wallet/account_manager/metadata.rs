//! Account Metadata Management
//! 
//! Implements logic for account nicknames, avatars, tags, and activity tracking.
//! 
//! # Requirements
//! - Requirement 12.1: Nickname validation
//! - Requirement 12.2: Deterministic avatar generation
//! - Requirement 12.3: Tag management
//! - Requirement 12.4: Activity tracking

use crate::error::{Result, WalletError};
use crate::security::SecureAccount;
use alloy::primitives::Address;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use regex::Regex;
use std::collections::HashSet;
use sha2::{Digest, Sha256};

pub struct MetadataManager;

impl MetadataManager {
    /// Validate nickname uniqueness and format (Requirement 12.1)
    pub fn validate_nickname(name: &str, existing_names: &[String]) -> Result<()> {
        if name.trim().is_empty() {
            return Err(WalletError::Generic("Nickname cannot be empty".into()).into());
        }

        if name.len() > 32 {
            return Err(WalletError::Generic("Nickname too long (max 32 chars)".into()).into());
        }

        // Regex pattern is a constant and will never fail to compile
        #[allow(clippy::expect_used)]
        let re = Regex::new(r"^[a-zA-Z0-9_\-\s]+$")
            .expect("Regex pattern is valid");
        if !re.is_match(name) {
            return Err(WalletError::Generic("Nickname contains invalid characters".into()).into());
        }

        if existing_names.contains(&name.to_string()) {
            return Err(WalletError::Generic("Nickname already exists".into()).into());
        }

        Ok(())
    }

    /// Update tags for an account (Requirement 12.3)
    pub fn update_tags(account: &mut SecureAccount, tags: Vec<String>) -> Result<()> {
        let unique_tags: HashSet<_> = tags.iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        
        if unique_tags.len() > 10 {
             return Err(WalletError::Generic("Too many tags (max 10)".into()).into());
        }

        account.tags = unique_tags.into_iter().collect();
        account.tags.sort(); // Deterministic order
        Ok(())
    }

    /// Record account activity (Requirement 12.4)
    pub fn record_activity(account: &mut SecureAccount) {
        account.last_used = Some(chrono::Utc::now().timestamp());
        account.transaction_count += 1;
    }

    /// Generate deterministic avatar (SVG) based on address (Requirement 12.2)
    /// Inspired by MetaMask's Jazzicon/Blockies
    pub fn generate_avatar(address: Address) -> String {
        // Use address hash as seed for determinism
        let mut hasher = Sha256::new();
        hasher.update(address.as_slice());
        let hash = hasher.finalize();
        
        // Seed RNG with hash (hash is always 32 bytes from SHA256)
        #[allow(clippy::expect_used)]
        let seed = <[u8; 32]>::try_from(hash.as_slice())
            .expect("SHA256 hash is always 32 bytes");
        let mut rng = ChaCha20Rng::from_seed(seed);

        // Generate colors (HSL -> RGB hex)
        // MetaMask uses generally 3-4 colors. We'll use 3.
        let colors: Vec<String> = (0..3).map(|_| {
            let h = rng.gen_range(0..360);
            let s = rng.gen_range(50..100);
            let l = rng.gen_range(40..80);
            hsl_to_hex(h, s, l)
        }).collect();

        // Simple Geometric Pattern (5x5 grid like blockies)
        let mut rects = String::new();
        let size = 5;
        let scale = 10;
        
        // Generate grid with symmetry
        let mut grid = vec![vec![false; size]; size];
        let mut grid_colors = vec![vec![0; size]; size];

        for y in 0..size {
            for x in 0..=(size / 2) { // Iterate through left half + center
                if rng.gen_bool(0.5) {
                    let color_idx = rng.gen_range(0..colors.len());
                    
                    // Set left side
                    grid[x][y] = true;
                    grid_colors[x][y] = color_idx;

                    // Mirror to right side
                    let mirror_x = size - 1 - x;
                    if x != mirror_x {
                        grid[mirror_x][y] = true;
                        grid_colors[mirror_x][y] = color_idx;
                    }
                }
            }
        }

        for y in 0..size {
            for x in 0..size {
                if grid[x][y] {
                    let color = &colors[grid_colors[x][y]];
                    rects.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" />"#,
                        x * scale, y * scale, scale, scale, color
                    ));
                }
            }
        }

        let bg = &colors[rng.gen_range(0..colors.len())];
        
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 50 50" style="background-color: {};">{}</svg>"#,
            bg, rects
        )
    }
}

fn hsl_to_hex(h: i32, s: i32, l: i32) -> String {
    let s = s as f32 / 100.0;
    let l = l as f32 / 100.0;
    
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h as f32 / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = match h {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    let r = ((r + m) * 255.0) as u8;
    let g = ((g + m) * 255.0) as u8;
    let b = ((b + m) * 255.0) as u8;
    
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_nickname_validation() {
        let existing = vec!["MyAccount".to_string()];
        
        assert!(MetadataManager::validate_nickname("NewAccount", &existing).is_ok());
        assert!(MetadataManager::validate_nickname("", &existing).is_err()); // Empty
        assert!(MetadataManager::validate_nickname("MyAccount", &existing).is_err()); // Duplicate
        assert!(MetadataManager::validate_nickname("Invalid!", &existing).is_err()); // Special chars
        assert!(MetadataManager::validate_nickname("A".repeat(33).as_str(), &existing).is_err()); // Too long
    }

    #[test]
    fn test_avatar_determinism() {
        let addr1 = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let svg1 = MetadataManager::generate_avatar(addr1);
        let svg2 = MetadataManager::generate_avatar(addr1);
        
        assert_eq!(svg1, svg2);
        
        let addr2 = Address::from_str("0x0000000000000000000000000000000000000002").unwrap();
        let svg3 = MetadataManager::generate_avatar(addr2);
        
        assert_ne!(svg1, svg3);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::security::KeyReference;
    use chrono::Utc;
    use proptest::prelude::*;
    use uuid::Uuid;

    // Helper to create a dummy secure account
    fn dummy_account() -> SecureAccount {
        SecureAccount {
            id: Uuid::new_v4().to_string(),
            address: Address::ZERO,
            name: "Test".to_string(),
            key_reference: KeyReference {
                id: "test".to_string(),
                service: "test".to_string(),
                account: "test".to_string(),
            },
            created_at: Utc::now(),
            is_hardware: false,
            derivation_path: None,
            tags: Vec::new(),
            last_used: None,
            transaction_count: 0,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        /// Property 33: Nickname Uniqueness and Format
        /// Validates that nicknames meeting criteria are accepted, and invalid ones rejected.
        #[test]
        fn prop_nickname_validation(name in "[a-zA-Z0-9_\\-\\s]{0,40}") {
            let existing = vec!["ExistingUser".to_string()];
            let result = MetadataManager::validate_nickname(&name, &existing);

            let is_valid_char = Regex::new(r"^[a-zA-Z0-9_\-\s]+$").unwrap().is_match(&name);
            let is_empty = name.trim().is_empty();
            let is_too_long = name.len() > 32;
            let exists = existing.contains(&name);

            if !is_empty && !is_too_long && is_valid_char && !exists {
                prop_assert!(result.is_ok(), "Valid nickname rejected: {}", name);
            } else {
                prop_assert!(result.is_err(), "Invalid nickname accepted: {}", name);
            }
        }

        /// Property 34: Avatar Determinism
        /// Generating an avatar for the same address must always produce the same SVG.
        #[test]
        fn prop_avatar_determinism(addr_bytes in proptest::array::uniform20(0u8..255)) {
            let address = Address::from(addr_bytes);
            let svg1 = MetadataManager::generate_avatar(address);
            let svg2 = MetadataManager::generate_avatar(address);
            
            prop_assert_eq!(&svg1, &svg2);
            prop_assert!(svg1.contains("<svg"));
            prop_assert!(svg1.contains("</svg>"));
        }

        /// Property 35: Tag Management Consistency
        /// Tags must be unique, trimmed, non-empty, and limited to 10.
        #[test]
        fn prop_tag_management(tags in proptest::collection::vec("[a-z ]{0,10}", 0..20)) {
            let mut account = dummy_account();
            let result = MetadataManager::update_tags(&mut account, tags.clone());

            let valid_tags: Vec<String> = tags.iter()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            
            let unique_count = valid_tags.iter().collect::<HashSet<_>>().len();

            if unique_count > 10 {
                prop_assert!(result.is_err());
            } else {
                prop_assert!(result.is_ok());
                // Verify tags in account
                prop_assert_eq!(account.tags.len(), unique_count);
                // Verify distinctness
                let account_tags_set: HashSet<_> = account.tags.iter().collect();
                prop_assert_eq!(account_tags_set.len(), unique_count);
            }
        }
    }
}
