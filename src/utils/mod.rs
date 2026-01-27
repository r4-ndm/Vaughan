//! Utility functions and common helpers
//!
//! This module provides common utilities used throughout the application.

use alloy::primitives::{Address, U256};
use std::str::FromStr;

use crate::error::{Result, SecurityError};

/// Format a U256 value as a human-readable string
pub fn format_token_amount(amount: U256, decimals: u8) -> String {
    let divisor = U256::from(10).pow(U256::from(decimals));
    let whole = amount / divisor;
    let remainder = amount % divisor;

    if remainder.is_zero() {
        whole.to_string()
    } else {
        // Convert remainder to string and pad with leading zeros
        let remainder_str = remainder.to_string();
        let padded_remainder = format!("{:0>width$}", remainder_str, width = decimals as usize);

        // Create decimal string and trim trailing zeros
        let decimal_str = format!("{}.{}", whole, padded_remainder)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        decimal_str
    }
}

/// Parse a token amount string to U256
pub fn parse_token_amount(amount_str: &str, decimals: u8) -> Result<U256> {
    let parts: Vec<&str> = amount_str.split('.').collect();

    match parts.len() {
        1 => {
            // No decimal point
            let whole: u128 = parts[0]
                .parse()
                .map_err(|_| SecurityError::InvalidAddress("Invalid amount format".to_string()))?;
            Ok(U256::from(whole) * U256::from(10).pow(U256::from(decimals)))
        }
        2 => {
            // Has decimal point
            let whole: u128 = parts[0]
                .parse()
                .map_err(|_| SecurityError::InvalidAddress("Invalid amount format".to_string()))?;
            let decimal_str = parts[1];

            if decimal_str.len() > decimals as usize {
                return Err(SecurityError::InvalidAddress("Too many decimal places".to_string()).into());
            }

            let decimal_padded = format!("{:0<width$}", decimal_str, width = decimals as usize);
            let decimal: u128 = decimal_padded
                .parse()
                .map_err(|_| SecurityError::InvalidAddress("Invalid decimal format".to_string()))?;

            let whole_part = U256::from(whole) * U256::from(10).pow(U256::from(decimals));
            let decimal_part = U256::from(decimal);

            Ok(whole_part + decimal_part)
        }
        _ => Err(SecurityError::InvalidAddress("Invalid amount format".to_string()).into()),
    }
}

/// Validate an Ethereum address
pub fn validate_address(address_str: &str) -> Result<Address> {
    Address::from_str(address_str).map_err(|_| SecurityError::InvalidAddress(address_str.to_string()).into())
}

/// Format an address for display (shortened)
pub fn format_address(address: Address) -> String {
    let addr_str = format!("{address:?}");
    if addr_str.len() > 10 {
        format!("{}...{}", &addr_str[0..6], &addr_str[addr_str.len() - 4..])
    } else {
        addr_str
    }
}

/// Calculate percentage change
pub fn calculate_percentage_change(old_value: f64, new_value: f64) -> f64 {
    if old_value == 0.0 {
        0.0
    } else {
        ((new_value - old_value) / old_value) * 100.0
    }
}

/// Format percentage with sign
pub fn format_percentage(percentage: f64) -> String {
    if percentage >= 0.0 {
        format!("+{percentage:.2}%")
    } else {
        format!("{percentage:.2}%")
    }
}

/// Generate a unique ID
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_nanos();
    format!("id_{timestamp}")
}

/// Sleep for a duration (async utility)
pub async fn sleep(duration: std::time::Duration) {
    tokio::time::sleep(duration).await;
}

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: usize,
    initial_delay: std::time::Duration,
) -> std::result::Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
{
    let mut delay = initial_delay;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries - 1 {
                    return Err(e);
                }
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}
