//! Input validation for security-sensitive operations
//!
//! This module provides validation functions for user inputs, network endpoints,
//! file paths, and other security-critical data to prevent injection attacks
//! and ensure data integrity.

use crate::error::{Result, SecurityError};
use std::path::{Path, PathBuf};
use url::Url;

/// Validates RPC endpoint URLs
pub fn validate_rpc_endpoint(url: &str) -> Result<Url> {
    // Parse the URL first
    let parsed_url = Url::parse(url).map_err(|_| SecurityError::KeystoreError {
        message: "Invalid URL format".to_string(),
    })?;

    // Validate scheme
    match parsed_url.scheme() {
        "http" | "https" | "ws" | "wss" => {}
        _ => {
            return Err(SecurityError::KeystoreError {
                message: "Invalid URL scheme. Only http, https, ws, and wss are allowed".to_string(),
            }
            .into())
        }
    }

    // Validate host
    let host = parsed_url.host_str().ok_or_else(|| SecurityError::KeystoreError {
        message: "URL must contain a valid host".to_string(),
    })?;

    // Block localhost and local network ranges in production unless explicitly allowed
    if is_local_address(host) {
        tracing::warn!("Using local network address: {}", host);
        // In production, you might want to block this entirely
        // return Err(SecurityError::KeystoreError {
        //     message: "Local network addresses are not allowed".to_string()
        // }.into());
    }

    // Validate port range
    if let Some(port) = parsed_url.port() {
        if !(80..=65535).contains(&port) {
            return Err(SecurityError::KeystoreError {
                message: "Port must be between 80 and 65535".to_string(),
            }
            .into());
        }
    }

    Ok(parsed_url)
}

/// Check if an address is a local/private network address
fn is_local_address(host: &str) -> bool {
    // Check for localhost
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return true;
    }

    // Check for private IP ranges
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // Private IPv4 ranges: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                return (octets[0] == 10)
                    || (octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31))
                    || (octets[0] == 192 && octets[1] == 168);
            }
            std::net::IpAddr::V6(ipv6) => {
                // Private IPv6 ranges
                return ipv6.is_loopback() ||
                       ipv6.segments()[0] == 0xfc00 || // fc00::/7
                       ipv6.segments()[0] == 0xfd00;
            }
        }
    }

    false
}

/// Validates file paths to prevent directory traversal attacks
pub fn validate_file_path(path: &str, base_dir: &Path) -> Result<PathBuf> {
    // Convert to PathBuf and canonicalize to resolve .. and other traversal attempts
    let path_buf = PathBuf::from(path);

    // Check for obvious traversal attempts
    if path.contains("..") || path.contains("~") {
        return Err(SecurityError::KeystoreError {
            message: "Path contains traversal sequences".to_string(),
        }
        .into());
    }

    // Check for null bytes (directory traversal attempt on some systems)
    if path.contains('\0') {
        return Err(SecurityError::KeystoreError {
            message: "Path contains null bytes".to_string(),
        }
        .into());
    }

    // Ensure path is within base directory
    let full_path = if path_buf.is_absolute() {
        path_buf
    } else {
        base_dir.join(path_buf)
    };

    // Canonicalize to resolve any remaining .. sequences
    let canonical_path = full_path.canonicalize().map_err(|_| SecurityError::KeystoreError {
        message: "Invalid path or path does not exist".to_string(),
    })?;

    let canonical_base = base_dir.canonicalize().map_err(|_| SecurityError::KeystoreError {
        message: "Base directory does not exist".to_string(),
    })?;

    // Ensure the canonical path is still within the base directory
    if !canonical_path.starts_with(&canonical_base) {
        return Err(SecurityError::KeystoreError {
            message: "Path escapes base directory".to_string(),
        }
        .into());
    }

    Ok(canonical_path)
}

/// Validates private key format
pub fn validate_private_key(private_key: &str) -> Result<String> {
    // Remove 0x prefix if present
    let clean_key = if private_key.starts_with("0x") {
        &private_key[2..]
    } else {
        private_key
    };

    // Check length (64 hex characters for 256-bit key)
    if clean_key.len() != 64 {
        return Err(SecurityError::InvalidPrivateKey.into());
    }

    // Check if all characters are valid hexadecimal
    if !clean_key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(SecurityError::InvalidPrivateKey.into());
    }

    // Check if key is not all zeros
    if clean_key.chars().all(|c| c == '0') {
        return Err(SecurityError::KeystoreError {
            message: "Private key cannot be all zeros".to_string(),
        }
        .into());
    }

    // Check if key is not the maximum value (invalid for secp256k1)
    let max_key = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141";
    if clean_key.to_uppercase().as_str() >= max_key {
        return Err(SecurityError::KeystoreError {
            message: "Private key exceeds maximum valid value".to_string(),
        }
        .into());
    }

    Ok(clean_key.to_string())
}

/// Validates an Ethereum address
pub fn validate_ethereum_address(address: &str) -> Result<()> {
    // Check if it starts with 0x (optional but recommended for clarity)
    let clean_address = if address.starts_with("0x") {
        &address[2..]
    } else {
        address
    };

    // Check length (40 hex characters)
    if clean_address.len() != 40 {
        return Err(SecurityError::KeystoreError {
            message: "Invalid Ethereum address length".to_string(),
        }
        .into());
    }

    // Check if all characters are valid hexadecimal
    if !clean_address.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(SecurityError::KeystoreError {
            message: "Invalid Ethereum address format".to_string(),
        }
        .into());
    }

    Ok(())
}

/// Validates seed phrase words
pub fn validate_seed_phrase_words(words: &[String]) -> Result<()> {
    // Check word count
    let valid_counts = [12, 15, 18, 21, 24];
    if !valid_counts.contains(&words.len()) {
        return Err(SecurityError::InvalidSeedPhrase {
            reason: format!(
                "Invalid word count: {}. Must be 12, 15, 18, 21, or 24 words",
                words.len()
            ),
        }
        .into());
    }

    // Check for empty words
    if words.iter().any(|word| word.is_empty()) {
        return Err(SecurityError::InvalidSeedPhrase {
            reason: "Seed phrase contains empty words".to_string(),
        }
        .into());
    }

    // Check for non-ASCII characters (BIP39 wordlist is ASCII only)
    for word in words {
        if !word.is_ascii() {
            return Err(SecurityError::InvalidSeedPhrase {
                reason: "Seed phrase contains non-ASCII characters".to_string(),
            }
            .into());
        }

        // Check for whitespace within words
        if word.contains(char::is_whitespace) {
            return Err(SecurityError::InvalidSeedPhrase {
                reason: "Individual words cannot contain whitespace".to_string(),
            }
            .into());
        }
    }

    Ok(())
}

/// Validates password strength
pub fn validate_password_strength(password: &str) -> Result<()> {
    if password.len() < 12 {
        return Err(SecurityError::KeystoreError {
            message: "Password must be at least 12 characters long".to_string(),
        }
        .into());
    }

    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    let criteria_met = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if criteria_met < 3 {
        return Err(SecurityError::KeystoreError {
            message: "Password must contain at least 3 of: lowercase, uppercase, digit, special character".to_string(),
        }
        .into());
    }

    Ok(())
}

/// Sanitize input strings to prevent injection attacks
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "-_.".contains(*c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_rpc_endpoint() {
        // Valid URLs
        assert!(validate_rpc_endpoint("https://mainnet.infura.io/v3/your-project-id").is_ok());
        assert!(validate_rpc_endpoint("wss://ws-mainnet.infura.io/v3/your-project-id").is_ok());

        // Invalid URLs
        assert!(validate_rpc_endpoint("ftp://example.com").is_err());
        assert!(validate_rpc_endpoint("not-a-url").is_err());
        assert!(validate_rpc_endpoint("http://").is_err());
    }

    #[test]
    fn test_validate_file_path() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Valid paths
        let test_file = base_path.join("test.txt");
        std::fs::write(&test_file, "test").unwrap();
        assert!(validate_file_path("test.txt", base_path).is_ok());

        // Invalid traversal attempts
        assert!(validate_file_path("../etc/passwd", base_path).is_err());
        assert!(validate_file_path("~/secrets", base_path).is_err());
        assert!(validate_file_path("test\0file", base_path).is_err());
    }

    #[test]
    fn test_validate_private_key() {
        // Valid private key
        assert!(validate_private_key("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").is_ok());
        assert!(validate_private_key("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").is_ok());

        // Invalid keys
        assert!(validate_private_key("short").is_err());
        assert!(validate_private_key("0000000000000000000000000000000000000000000000000000000000000000").is_err());
        assert!(validate_private_key("xyz123").is_err());
    }

    #[test]
    fn test_validate_ethereum_address() {
        // Valid address
        assert!(validate_ethereum_address("0x742d35Cc6635C0532925a3b8D35d6C1a23e1a2A0").is_ok());
        assert!(validate_ethereum_address("742d35Cc6635C0532925a3b8D35d6C1a23e1a2A0").is_ok());

        // Invalid addresses
        assert!(validate_ethereum_address("short").is_err());
        assert!(validate_ethereum_address("0x742d35Cc6635C0532925a3b8D35d6C1a23e1a2Az").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        // Strong passwords
        assert!(validate_password_strength("MyStr0ngP@ssw0rd!").is_ok());
        assert!(validate_password_strength("ComplexPassword123!").is_ok());

        // Weak passwords
        assert!(validate_password_strength("weak").is_err());
        assert!(validate_password_strength("onlylowercase").is_err());
        assert!(validate_password_strength("12345678901").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("hello@world!"), "helloworld");
        assert_eq!(sanitize_input("test-file_name.txt"), "test-file_name.txt");
        assert_eq!(sanitize_input("../../../etc/passwd"), "......etcpasswd");
    }
}
