//! Network health monitoring

use crate::error::Result;

/// Endpoint health status
#[derive(Debug, Clone)]
pub struct EndpointHealth {
    pub is_responsive: bool,
    pub latency_ms: u64,
    pub last_block_time: u64,
    pub is_syncing: bool,
    pub peer_count: u64,
}

/// Check endpoint health
pub async fn check_endpoint_health(url: &str) -> Result<EndpointHealth> {
    use alloy::providers::{Provider, ProviderBuilder};
    use std::time::Instant;

    let start_time = Instant::now();

    let parsed_url = url
        .parse()
        .map_err(|e| crate::error::VaughanError::Network(crate::error::NetworkError::RpcError {
            message: format!("Invalid URL: {}", e),
        }))?;
    let provider = ProviderBuilder::new().connect_http(parsed_url);

    // Test basic connectivity
    match provider.get_block_number().await {
        Ok(_block_number) => {
            let latency = start_time.elapsed().as_millis() as u64;

            // Mock sync status for now
            let is_syncing = false;

            // Get current time safely
            let last_block_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs();

            Ok(EndpointHealth {
                is_responsive: true,
                latency_ms: latency,
                last_block_time,
                is_syncing,
                peer_count: 0, // Most RPC endpoints don't expose peer count
            })
        }
        Err(_) => Ok(EndpointHealth {
            is_responsive: false,
            latency_ms: start_time.elapsed().as_millis() as u64,
            last_block_time: 0,
            is_syncing: false,
            peer_count: 0,
        }),
    }
}
