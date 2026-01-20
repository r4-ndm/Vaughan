//! Block Explorer Service
//!
//! Handles interactions with various block explorer APIs (Etherscan, PulseScan, etc.)
//! to fetch transaction history and other data.

use crate::gui::{Transaction, TransactionStatus};
use crate::network::NetworkId;
use chrono;
use reqwest;
use serde_json;

/// Comprehensive fetch from block explorer APIs
pub async fn fetch_from_block_explorer(network: NetworkId, address: &str) -> Result<Vec<Transaction>, String> {
    tracing::info!(
        "🔍 Fetching historical transactions for {} on network {}",
        address,
        network.0
    );

    // Try to fetch real historical transactions from block explorer APIs
    let transactions = match network.0 {
        369 => {
            // PulseChain mainnet
            tracing::info!("🔍 Trying PulseChain API for historical transactions");
            fetch_from_pulsescan(address, false).await.unwrap_or_else(|e| {
                tracing::warn!("PulseChain API failed: {}", e);
                Vec::new()
            })
        }
        943 => {
            // PulseChain testnet
            tracing::info!("🔍 Trying PulseChain testnet API for historical transactions");
            fetch_from_pulsescan(address, true).await.unwrap_or_else(|e| {
                tracing::warn!("PulseChain testnet API failed: {}", e);
                Vec::new()
            })
        }
        1 => {
            // Ethereum mainnet
            tracing::info!("🔍 Trying Ethereum API for historical transactions");
            fetch_from_etherscan_public(address).await.unwrap_or_else(|e| {
                tracing::warn!("Ethereum API failed: {}", e);
                Vec::new()
            })
        }
        56 => {
            // BSC
            tracing::info!("🔍 Trying BSC API for historical transactions");
            fetch_from_bscscan_public(address).await.unwrap_or_else(|e| {
                tracing::warn!("BSC API failed: {}", e);
                Vec::new()
            })
        }
        137 => {
            // Polygon
            tracing::info!("🔍 Trying Polygon API for historical transactions");
            fetch_from_polygonscan_public(address).await.unwrap_or_else(|e| {
                tracing::warn!("Polygon API failed: {}", e);
                Vec::new()
            })
        }
        _ => {
            tracing::info!("⚠️ Unsupported network {} for historical transactions", network.0);
            Vec::new()
        }
    };

    // If API calls returned empty results (likely due to missing API keys), that's fine
    if transactions.is_empty() {
        tracing::info!("📝 API returned empty results (likely no transactions or missing API keys)");
    }

    tracing::info!(
        "✅ Fetched {} historical transactions from block explorer",
        transactions.len()
    );
    Ok(transactions)
}

/// Fetch from Etherscan without API key (using public endpoints)
async fn fetch_from_etherscan_public(address: &str) -> Result<Vec<Transaction>, String> {
    // Note: Etherscan requires API keys for most requests now
    // This will likely fail, but we'll try anyway and fallback gracefully
    let url = format!("https://api.etherscan.io/api?module=account&action=txlist&address={address}&startblock=0&endblock=99999999&sort=desc&page=1&offset=50");

    // Try the request, but don't treat failures as critical errors
    match fetch_transactions_from_url(&url, "ETH").await {
        Ok(txs) => Ok(txs),
        Err(e) => {
            tracing::debug!("Etherscan API failed (expected without API key): {}", e);
            Err(format!("Etherscan API requires authentication: {e}"))
        }
    }
}

/// Fetch from BSCScan without API key
async fn fetch_from_bscscan_public(address: &str) -> Result<Vec<Transaction>, String> {
    let url = format!("https://api.bscscan.com/api?module=account&action=txlist&address={address}&startblock=0&endblock=99999999&sort=desc&page=1&offset=50");

    match fetch_transactions_from_url(&url, "BNB").await {
        Ok(txs) => Ok(txs),
        Err(e) => {
            tracing::debug!("BSCScan API failed (likely requires API key): {}", e);
            Err(format!("BSCScan API requires authentication: {e}"))
        }
    }
}

/// Fetch from Polygonscan without API key
async fn fetch_from_polygonscan_public(address: &str) -> Result<Vec<Transaction>, String> {
    let url = format!("https://api.polygonscan.com/api?module=account&action=txlist&address={address}&startblock=0&endblock=99999999&sort=desc&page=1&offset=50");

    match fetch_transactions_from_url(&url, "MATIC").await {
        Ok(txs) => Ok(txs),
        Err(e) => {
            tracing::debug!("Polygonscan API failed (likely requires API key): {}", e);
            Err(format!("Polygonscan API requires authentication: {e}"))
        }
    }
}

/// Fetch from PulseScan using Blockscout API (no API key required)
async fn fetch_from_pulsescan(address: &str, is_testnet: bool) -> Result<Vec<Transaction>, String> {
    // PulseChain uses Blockscout which has different API endpoints
    let base_url = if is_testnet {
        "https://scan.v4.testnet.pulsechain.com/api"
    } else {
        "https://scan.pulsechain.com/api" // Main PulseChain explorer
    };

    // Blockscout API v2 format for getting address transactions
    // This endpoint is typically free and doesn't require API keys
    let url = format!("{base_url}/v2/addresses/{address}/transactions?filter=to%%20%%7C%%20from&type=coin_transfer");

    // Try v2 API first
    match fetch_from_blockscout_v2(&url, "PLS").await {
        Ok(txs) if !txs.is_empty() => Ok(txs),
        _ => {
            // Fallback to v1 API format (Etherscan-compatible)
            let v1_url =
                format!("{base_url}/v1/result?module=account&action=txlist&address={address}&sort=desc&limit=50");
            fetch_transactions_from_url(&v1_url, "PLS").await
        }
    }
}

/// Fetch from Blockscout v2 API format
async fn fetch_from_blockscout_v2(url: &str, symbol: &str) -> Result<Vec<Transaction>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {e}"))?;

    let mut transactions = Vec::new();

    // Parse Blockscout v2 response format
    if let Some(items) = data["items"].as_array() {
        for tx in items.iter().take(50) {
            let hash = tx["hash"].as_str().unwrap_or("").to_string();
            if hash.is_empty() {
                continue;
            }

            let from = tx["from"]["hash"]
                .as_str()
                .or_else(|| tx["from"].as_str())
                .unwrap_or("")
                .to_string();

            let to = tx["to"]["hash"]
                .as_str()
                .or_else(|| tx["to"].as_str())
                .unwrap_or("")
                .to_string();

            // Parse value
            let value = tx["value"].as_str().unwrap_or("0");
            let value_native = if let Ok(wei) = value.parse::<u128>() {
                format!("{:.6} {}", wei as f64 / 1e18, symbol)
            } else {
                format!("0 {symbol}")
            };

            // Parse timestamp
            let timestamp = tx["timestamp"]
                .as_str()
                .or_else(|| tx["block_timestamp"].as_str())
                .map(|ts| ts.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Parse status
            let status = match tx["status"].as_str() {
                Some("ok") => TransactionStatus::Confirmed,
                Some("error") => TransactionStatus::Failed,
                Some("pending") => TransactionStatus::Pending,
                _ => TransactionStatus::Confirmed,
            };

            transactions.push(Transaction {
                hash,
                from,
                to,
                amount: value_native,
                timestamp,
                status,
            });
        }
    }

    Ok(transactions)
}

/// Generic function to fetch and parse transactions from block explorer API
async fn fetch_transactions_from_url(url: &str, symbol: &str) -> Result<Vec<Transaction>, String> {
    tracing::debug!("Fetching transactions from: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status();
    tracing::debug!("API response status: {}", status);

    // Get the raw response text first for debugging
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    // Log first 200 chars for debugging (without sensitive data)
    tracing::debug!(
        "API response preview: {}",
        response_text
            .chars()
            .take(200)
            .collect::<String>()
            .replace("\n", " ")
            .replace("\r", "")
    );

    // Check if this looks like HTML (common for rate-limited responses)
    if response_text.trim_start().starts_with("<") {
        return Err("API returned HTML instead of JSON - likely rate limited or blocked".to_string());
    }

    // Try to parse as JSON
    let data: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
        tracing::error!(
            "JSON parse error: {}. Response was: {}",
            e,
            response_text.chars().take(500).collect::<String>()
        );
        format!("Failed to parse JSON response: {e}")
    })?;

    // Check if API returned success with empty result (address has no transactions)
    if data["status"] == "1" && data["result"].as_array().is_some_and(|a| a.is_empty()) {
        // Valid response but no transactions - this is OK, return empty vector
        return Ok(Vec::new());
    }

    // Check various response formats
    let transactions_data = if let Some(result) = data["result"].as_array() {
        // Standard Etherscan format
        result.clone()
    } else if let Some(txs) = data["transactions"].as_array() {
        // Alternative format
        txs.clone()
    } else if data["status"] == "0" && data["message"].as_str() == Some("NOTOK") {
        // API error - likely rate limited or requires API key
        return Err("API request failed - rate limited or authentication required".to_string());
    } else if data["status"] == "0" && data["message"].as_str() == Some("No transactions found") {
        // Valid response but no transactions
        return Ok(Vec::new());
    } else {
        return Err("Unexpected API response format".to_string());
    };

    let mut transactions = Vec::new();

    for tx in transactions_data.iter().take(50) {
        // Try to parse with multiple field name variations
        let hash = tx["hash"]
            .as_str()
            .or_else(|| tx["txhash"].as_str())
            .or_else(|| tx["transactionHash"].as_str())
            .unwrap_or("")
            .to_string();

        if hash.is_empty() {
            continue; // Skip invalid entries
        }

        let from = tx["from"].as_str().unwrap_or("").to_string();
        let to = tx["to"].as_str().unwrap_or("").to_string();

        // Convert value from wei to native token
        let value_wei = tx["value"].as_str().unwrap_or("0");
        let value_native = if let Ok(wei) = value_wei.parse::<u128>() {
            format!("{:.6} {}", wei as f64 / 1e18, symbol)
        } else {
            format!("0 {symbol}")
        };

        // Convert timestamp
        let timestamp = if let Some(ts_str) = tx["timeStamp"].as_str() {
            if let Ok(ts) = ts_str.parse::<i64>() {
                chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            }
        } else if let Some(ts_num) = tx["timestamp"].as_i64() {
            chrono::DateTime::from_timestamp(ts_num, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        // Determine status
        let status = if tx["isError"] == "1" || tx["status"] == "0" {
            TransactionStatus::Failed
        } else if let Some(confirmations) = tx["confirmations"].as_str() {
            if confirmations == "0" {
                TransactionStatus::Pending
            } else {
                TransactionStatus::Confirmed
            }
        } else {
            TransactionStatus::Confirmed // Assume confirmed if no status info
        };

        transactions.push(Transaction {
            hash,
            from,
            to,
            amount: value_native,
            timestamp,
            status,
        });
    }

    // Return the transactions (can be empty if address has no history)
    Ok(transactions)
}
