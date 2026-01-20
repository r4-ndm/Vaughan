//! Network validation utilities

use crate::error::Result;
use crate::network::{NetworkConfig, NetworkId};

/// Network validation result
#[derive(Debug, Clone)]
pub struct NetworkValidation {
    pub is_valid: bool,
    pub is_secure: bool,
    pub chain_id_matches: bool,
    pub response_time_ms: u64,
    pub issues: Vec<NetworkValidationIssue>,
}

/// Network validation issues
#[derive(Debug, Clone)]
pub enum NetworkValidationIssue {
    InsecureConnection,
    ChainIdMismatch { expected: u64, actual: u64 },
    HighLatency(u64),
    NotResponding,
    InvalidResponse,
    UnsupportedNetwork,
    InvalidUrl,
    InvalidChainId,
    EmptyField(String),
    InvalidBlockExplorer,
}

/// Validate a network endpoint
pub async fn validate_network_endpoint(url: &str, expected_chain_id: u64) -> Result<NetworkValidation> {
    use alloy::providers::ProviderBuilder;
    use std::time::Instant;

    let mut issues = Vec::new();
    let start_time = Instant::now();

    // Check if URL is secure
    let is_secure = url.starts_with("https://");
    if !is_secure {
        issues.push(NetworkValidationIssue::InsecureConnection);
    }

    // Special handling for known problematic but legitimate networks
    // ETHW (Chain ID 10001) often has connectivity issues but is a legitimate network
    let is_known_problematic =
        expected_chain_id == 10001 && (url.contains("ethereumpow") || url.contains("ethw") || url.contains("eth_pow"));

    if is_known_problematic {
        tracing::warn!(
            "⚠️ Adding ETHW network - connectivity validation will be lenient due to known infrastructure limitations"
        );
    }

    // Try to create provider and test connection
    let parsed_url = match url.parse() {
        Ok(url) => url,
        Err(_) => {
            issues.push(NetworkValidationIssue::InvalidUrl);
            return Ok(NetworkValidation {
                is_valid: false,
                is_secure: false,
                chain_id_matches: false,
                response_time_ms: 0,
                issues,
            });
        }
    };

    let provider = ProviderBuilder::new().connect_http(parsed_url);

    // Test provider connectivity with timeout
    let connectivity_result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        test_provider_connectivity(&provider, expected_chain_id),
    )
    .await;

    let (chain_id_matches, actual_response_time) = match connectivity_result {
        Ok(Ok((matches, response_time))) => (matches, response_time),
        Ok(Err(_)) => {
            issues.push(NetworkValidationIssue::InvalidResponse);
            (false, start_time.elapsed().as_millis() as u64)
        }
        Err(_) => {
            issues.push(NetworkValidationIssue::NotResponding);
            (false, 10000) // Timeout
        }
    };

    // Check for high latency
    if actual_response_time > 5000 {
        issues.push(NetworkValidationIssue::HighLatency(actual_response_time));
    }

    // Special case for ETHW: Allow network addition even if not responding
    // ETHW infrastructure is known to be unstable but the network is legitimate
    let is_ethw = expected_chain_id == 10001;

    let is_valid = if is_ethw {
        // For ETHW, only block on invalid URL format
        issues
            .iter()
            .all(|issue| !matches!(issue, NetworkValidationIssue::InvalidUrl))
    } else {
        // For other networks, use standard validation
        issues.iter().all(|issue| {
            !matches!(
                issue,
                NetworkValidationIssue::InvalidUrl
                    | NetworkValidationIssue::NotResponding
                    | NetworkValidationIssue::InvalidResponse
            )
        })
    };

    Ok(NetworkValidation {
        is_valid,
        is_secure,
        chain_id_matches,
        response_time_ms: actual_response_time,
        issues,
    })
}

/// Test provider connectivity and chain ID
async fn test_provider_connectivity(
    provider: &alloy::providers::fillers::FillProvider<
        alloy::providers::fillers::JoinFill<
            alloy::providers::Identity,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::GasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::BlobGasFiller,
                    alloy::providers::fillers::JoinFill<
                        alloy::providers::fillers::NonceFiller,
                        alloy::providers::fillers::ChainIdFiller,
                    >,
                >,
            >,
        >,
        alloy::providers::RootProvider,
    >,
    expected_chain_id: u64,
) -> Result<(bool, u64)> {
    use alloy::providers::Provider;
    use std::time::Instant;

    let start = Instant::now();

    // Test basic connectivity by getting chain ID
    let actual_chain_id = provider
        .get_chain_id()
        .await
        .map_err(|e| crate::error::NetworkError::RpcError {
            message: format!("Failed to get chain ID: {e}"),
        })?;

    let response_time = start.elapsed().as_millis() as u64;
    let chain_id_matches = actual_chain_id == expected_chain_id;

    Ok((chain_id_matches, response_time))
}

/// Validate a complete custom network configuration
pub fn validate_custom_network_config(
    name: &str,
    rpc_url: &str,
    chain_id_str: &str,
    symbol: &str,
    block_explorer: &str,
) -> NetworkValidation {
    let mut issues = Vec::new();

    // Validate required fields
    if name.trim().is_empty() {
        issues.push(NetworkValidationIssue::EmptyField("Network Name".to_string()));
    }

    if rpc_url.trim().is_empty() {
        issues.push(NetworkValidationIssue::EmptyField("RPC URL".to_string()));
    } else {
        // Validate URL format
        if !rpc_url.starts_with("http://") && !rpc_url.starts_with("https://") {
            issues.push(NetworkValidationIssue::InvalidUrl);
        } else if !rpc_url.starts_with("https://") {
            // Check if it's localhost - only exception for HTTP
            let is_localhost =
                rpc_url.contains("127.0.0.1") || rpc_url.contains("localhost") || rpc_url.contains("[::1]"); // IPv6 localhost

            if !is_localhost {
                // HTTP is not allowed for remote endpoints - this is a blocking error
                issues.push(NetworkValidationIssue::InsecureConnection);
            }
        }

        // Basic URL format validation
        if url::Url::parse(rpc_url).is_err() {
            issues.push(NetworkValidationIssue::InvalidUrl);
        }
    }

    if chain_id_str.trim().is_empty() {
        issues.push(NetworkValidationIssue::EmptyField("Chain ID".to_string()));
    } else {
        // Validate chain ID is a valid number
        if chain_id_str.parse::<u64>().is_err() {
            issues.push(NetworkValidationIssue::InvalidChainId);
        }
    }

    if symbol.trim().is_empty() {
        issues.push(NetworkValidationIssue::EmptyField("Currency Symbol".to_string()));
    }

    // Validate block explorer URL if provided
    if !block_explorer.trim().is_empty() {
        if !block_explorer.starts_with("http://") && !block_explorer.starts_with("https://") {
            issues.push(NetworkValidationIssue::InvalidBlockExplorer);
        } else if url::Url::parse(block_explorer).is_err() {
            issues.push(NetworkValidationIssue::InvalidBlockExplorer);
        }
    }

    let is_valid = issues.is_empty();
    let is_secure = rpc_url.starts_with("https://");

    NetworkValidation {
        is_valid,
        is_secure,
        chain_id_matches: true, // Will be validated later with actual network test
        response_time_ms: 0,    // Not applicable for form validation
        issues,
    }
}

/// Create a NetworkConfig from validated form inputs
pub fn create_network_config(
    name: String,
    rpc_url: String,
    chain_id: u64,
    symbol: String,
    block_explorer: String,
    is_testnet: bool,
) -> NetworkConfig {
    let network_id = NetworkId(chain_id);

    NetworkConfig {
        id: network_id,
        name,
        rpc_url,
        chain_id,
        symbol,
        block_explorer_url: if block_explorer.trim().is_empty() {
            "https://etherscan.io".to_string() // Default fallback
        } else {
            block_explorer
        },
        is_testnet,
        is_custom: true, // All networks created via this function are custom
    }
}
