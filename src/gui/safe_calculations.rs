//! Safe calculation utilities for preventing NaN/Infinity propagation to UI widgets
//!
//! This module provides safety checks for all numeric calculations used in widget rendering
//! to prevent crashes due to invalid floating-point values from password bypass scenarios.

/// Validates a balance string and ensures it's safe for UI rendering
pub fn safe_balance(balance: &str) -> String {
    if balance.is_empty() {
        return "0.0000".to_string();
    }

    // Check for extremely large balances that would overflow f64
    if balance.len() > 30 {
        return "> 10^30".to_string();
    }

    // Check for scientific notation or extremely large values
    if balance.contains('e') || balance.contains('E') {
        return "> 10^30".to_string();
    }

    match balance.parse::<f64>() {
        Ok(val) => {
            if val.is_finite() && (0.0..1e30).contains(&val) {
                if val == 0.0 {
                    "0.0000".to_string()
                } else if val < 0.0001 {
                    format!("{val:.8}")
                } else if val < 1_000_000.0 {
                    format!("{val:.4}")
                } else if val < 1_000_000_000.0 {
                    format!("{:.2e}", val) // Scientific notation for large values
                } else {
                    "> 10^30".to_string()
                }
            } else {
                // Invalid value (NaN, infinity, negative, too large)
                "0.0000".to_string()
            }
        }
        Err(_) => {
            // Parsing failed - might be too large for f64
            if balance.len() > 50 {
                "> 10^30".to_string()
            } else {
                // Try to parse as decimal string manually for very large numbers
                match balance.find('.') {
                    Some(dot_pos) => {
                        let int_part = &balance[..dot_pos];
                        if int_part.len() > 12 {
                            format!("{}e{}", int_part.chars().next().unwrap_or('0'), int_part.len() - 1)
                        } else {
                            "0.0000".to_string() // Fallback
                        }
                    }
                    None => {
                        if balance.len() > 12 {
                            format!("{}e{}", balance.chars().next().unwrap_or('0'), balance.len() - 1)
                        } else {
                            "0.0000".to_string()
                        }
                    }
                }
            }
        }
    }
}

/// Validates USD price calculation and ensures safe output
pub fn safe_usd_value(balance: f64, price_per_unit: f64) -> f64 {
    if balance.is_finite() && price_per_unit.is_finite() && balance >= 0.0 && price_per_unit >= 0.0 {
        let result = balance * price_per_unit;
        if result.is_finite() && result >= 0.0 {
            result
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Validates widget dimension (width/height) to ensure it's safe for rendering
pub fn safe_dimension(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value.max(1.0) // Minimum 1 pixel
    } else {
        1.0 // Safe fallback
    }
}

/// Validates percentage values (0.0 to 1.0) for progress bars and animations
pub fn safe_percentage(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Validates time-based calculations for animations
pub fn safe_time_calculation(time_ms: f64) -> f64 {
    if time_ms.is_finite() && time_ms >= 0.0 {
        time_ms
    } else {
        0.0
    }
}

/// Validates token ticker/symbol strings
pub fn safe_ticker(ticker: &str) -> String {
    if ticker.is_empty() || ticker.trim().is_empty() {
        "ETH".to_string() // Safe default
    } else {
        ticker.trim().to_string()
    }
}

/// Validates list of tickers ensuring at least one valid option exists
pub fn safe_ticker_list(tickers: &[String]) -> Vec<String> {
    let valid_tickers: Vec<String> = tickers
        .iter()
        .filter_map(|t| {
            let safe = safe_ticker(t);
            if safe != "ETH" || t == "ETH" {
                // Keep ETH and other valid tickers
                Some(safe)
            } else {
                None // Filter out invalid tickers that got defaulted to ETH
            }
        })
        .collect();

    if valid_tickers.is_empty() {
        vec!["ETH".to_string()]
    } else {
        valid_tickers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_balance() {
        assert_eq!(safe_balance(""), "0.0000");
        assert_eq!(safe_balance("invalid"), "0.0000");
        assert_eq!(safe_balance("NaN"), "0.0000");
        assert_eq!(safe_balance("inf"), "0.0000");
        assert_eq!(safe_balance("-1.0"), "0.0000");
        assert_eq!(safe_balance("0"), "0.0000");
        assert_eq!(safe_balance("0.0001"), "0.0001");
        assert_eq!(safe_balance("123.456"), "123.4560");
    }

    #[test]
    fn test_safe_dimension() {
        assert_eq!(safe_dimension(0.0), 1.0);
        assert_eq!(safe_dimension(-10.0), 1.0);
        assert_eq!(safe_dimension(f32::NAN), 1.0);
        assert_eq!(safe_dimension(f32::INFINITY), 1.0);
        assert_eq!(safe_dimension(50.5), 50.5);
    }

    #[test]
    fn test_safe_ticker() {
        assert_eq!(safe_ticker(""), "ETH");
        assert_eq!(safe_ticker("  "), "ETH");
        assert_eq!(safe_ticker("BTC"), "BTC");
        assert_eq!(safe_ticker("  USDC  "), "USDC");
    }
}
