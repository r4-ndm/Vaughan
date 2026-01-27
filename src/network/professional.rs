#![cfg_attr(not(feature = "professional"), allow(dead_code))]

use crate::error::{NetworkError, Result};
use crate::network::{NetworkConfig, NetworkId};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use url::Url;

/// Professional network manager with enterprise-grade features
#[derive(Debug)]
pub struct ProfessionalNetworkManager {
    /// Network configurations with multiple endpoints
    networks: Arc<RwLock<HashMap<NetworkId, NetworkCluster>>>,
    /// HTTP client with connection pooling
    http_client: Client,
    /// Load balancer for RPC endpoints
    load_balancer: LoadBalancer,
    /// Health monitor for endpoints
    health_monitor: Arc<NetworkHealthMonitor>,
    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,
    /// Circuit breaker for failed endpoints
    circuit_breaker: Arc<CircuitBreaker>,
    /// Request cache
    cache: Arc<RequestCache>,
    /// Network statistics
    stats: Arc<RwLock<NetworkStatistics>>,
    /// Current network
    current_network: Arc<RwLock<NetworkId>>,
}

/// Network cluster with multiple RPC endpoints
#[derive(Debug, Clone)]
pub struct NetworkCluster {
    pub primary_config: NetworkConfig,
    pub endpoints: Vec<RpcEndpoint>,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub health_check_config: HealthCheckConfig,
}

/// RPC endpoint with health and performance metrics
#[derive(Debug, Clone)]
pub struct RpcEndpoint {
    pub url: String,
    pub priority: u32,
    pub weight: f32,
    pub max_concurrent_requests: u32,
    pub timeout: Duration,
    pub health_status: EndpointHealth,
    pub performance_metrics: PerformanceMetrics,
    pub auth_config: Option<AuthConfig>,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AuthType {
    None,
    ApiKey,
    BasicAuth,
    Bearer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LatencyBased,
    HealthBased,
}

#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub unhealthy_threshold: u32,
    pub healthy_threshold: u32,
    pub method: HealthCheckMethod,
}

#[derive(Debug, Clone)]
pub enum HealthCheckMethod {
    EthBlockNumber,
    EthChainId,
    NetVersion,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EndpointHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_latency: Duration,
    pub success_rate: f32,
    pub requests_per_second: f32,
    pub last_successful_request: Option<SystemTime>,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub total_failures: u64,
}

/// Load balancer for distributing requests across endpoints
#[derive(Debug)]
pub struct LoadBalancer {
    strategies: HashMap<LoadBalancingStrategy, Box<dyn LoadBalancingAlgorithm>>,
    active_connections: Arc<RwLock<HashMap<String, u32>>>,
}

#[async_trait::async_trait]
pub trait LoadBalancingAlgorithm: Send + Sync + std::fmt::Debug {
    async fn select_endpoint(
        &self,
        endpoints: &[RpcEndpoint],
        active_connections: &HashMap<String, u32>,
    ) -> Option<usize>;
}

/// Network health monitoring system
#[derive(Debug)]
pub struct NetworkHealthMonitor {
    health_checks: Arc<RwLock<HashMap<String, HealthCheckState>>>,
    #[allow(dead_code)] // Planned for future monitoring features
    monitoring_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone)]
struct HealthCheckState {
    last_check: SystemTime,
    consecutive_failures: u32,
    consecutive_successes: u32,
    current_status: EndpointHealth,
}

/// Rate limiting for API calls
#[derive(Debug)]
pub struct RateLimiter {
    limiters: RwLock<HashMap<String, TokenBucket>>,
    global_limiter: TokenBucket,
}

#[derive(Debug)]
struct TokenBucket {
    capacity: u32,
    tokens: Arc<RwLock<u32>>,
    refill_rate: u32, // tokens per second
    last_refill: Arc<RwLock<Instant>>,
}

/// Circuit breaker for failing endpoints
#[derive(Debug)]
pub struct CircuitBreaker {
    breakers: RwLock<HashMap<String, CircuitBreakerState>>,
    failure_threshold: u32,
    timeout: Duration,
    half_open_max_calls: u32,
}

#[derive(Debug, Clone)]
struct CircuitBreakerState {
    state: BreakerState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    half_open_successes: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum BreakerState {
    Closed,   // Normal operation
    Open,     // Failing fast
    HalfOpen, // Testing if service recovered
}

/// Request caching system
#[derive(Debug)]
pub struct RequestCache {
    cache: RwLock<HashMap<String, CachedResponse>>,
    max_size: usize,
    #[allow(dead_code)] // Planned for future cache TTL features
    default_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    data: String,
    timestamp: SystemTime,
    ttl: Duration,
}

/// Network statistics collection
#[derive(Debug, Clone, Default)]
pub struct NetworkStatistics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub cached_responses: u64,
    pub average_response_time: Duration,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub endpoint_stats: HashMap<String, EndpointStats>,
}

#[derive(Debug, Clone, Default)]
pub struct EndpointStats {
    pub requests: u64,
    pub failures: u64,
    pub average_latency: Duration,
    pub last_used: Option<SystemTime>,
}

impl ProfessionalNetworkManager {
    /// Create a new professional network manager
    #[cfg(feature = "professional")]
    pub async fn new() -> Result<Self> {
        info!("🌐 Initializing Professional Network Manager");

        // Build HTTP client with optimizations
        let http_client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .user_agent("VaughanWallet/1.0")
            .build()
            .map_err(|e| NetworkError::NetworkError {
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        let manager = Self {
            networks: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            load_balancer: LoadBalancer::new(),
            health_monitor: Arc::new(NetworkHealthMonitor::new()),
            rate_limiter: Arc::new(RateLimiter::new(100, 1000)), // 100 RPS, 1000 burst
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(60), 3)),
            cache: Arc::new(RequestCache::new(1000, Duration::from_secs(300))),
            stats: Arc::new(RwLock::new(NetworkStatistics::default())),
            current_network: Arc::new(RwLock::new(NetworkId(1))), // Default to Ethereum
        };

        // Initialize default networks with multiple endpoints
        manager.initialize_default_networks().await?;

        // Start health monitoring
        manager.start_health_monitoring().await;

        info!("✅ Professional Network Manager initialized successfully");
        Ok(manager)
    }

    /// Initialize default networks with enterprise-grade endpoints
    async fn initialize_default_networks(&self) -> Result<()> {
        let mut networks = self.networks.write().await;

        // Ethereum Mainnet
        networks.insert(
            NetworkId(1),
            NetworkCluster {
                primary_config: NetworkConfig {
                    id: NetworkId(1),
                    name: "Ethereum Mainnet".to_string(),
                    rpc_url: "https://eth.llamarpc.com".to_string(),
                    chain_id: 1,
                    symbol: "ETH".to_string(),
                    block_explorer_url: "https://etherscan.io".to_string(),
                    is_testnet: false,
                    is_custom: false,
                },
                endpoints: vec![
                    RpcEndpoint {
                        url: "https://eth.llamarpc.com".to_string(),
                        priority: 1,
                        weight: 1.0,
                        max_concurrent_requests: 100,
                        timeout: Duration::from_secs(10),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                    RpcEndpoint {
                        url: "https://rpc.ankr.com/eth".to_string(),
                        priority: 2,
                        weight: 0.8,
                        max_concurrent_requests: 80,
                        timeout: Duration::from_secs(12),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                    RpcEndpoint {
                        url: "https://ethereum.publicnode.com".to_string(),
                        priority: 3,
                        weight: 0.6,
                        max_concurrent_requests: 60,
                        timeout: Duration::from_secs(15),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                ],
                load_balancing_strategy: LoadBalancingStrategy::HealthBased,
                health_check_config: HealthCheckConfig {
                    interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(5),
                    unhealthy_threshold: 3,
                    healthy_threshold: 2,
                    method: HealthCheckMethod::EthBlockNumber,
                },
            },
        );

        // Polygon
        networks.insert(
            NetworkId(137),
            NetworkCluster {
                primary_config: NetworkConfig {
                    id: NetworkId(137),
                    name: "Polygon Mainnet".to_string(),
                    rpc_url: "https://polygon.llamarpc.com".to_string(),
                    chain_id: 137,
                    symbol: "MATIC".to_string(),
                    block_explorer_url: "https://polygonscan.com".to_string(),
                    is_testnet: false,
                    is_custom: false,
                },
                endpoints: vec![
                    RpcEndpoint {
                        url: "https://polygon.llamarpc.com".to_string(),
                        priority: 1,
                        weight: 1.0,
                        max_concurrent_requests: 100,
                        timeout: Duration::from_secs(8),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                    RpcEndpoint {
                        url: "https://rpc.ankr.com/polygon".to_string(),
                        priority: 2,
                        weight: 0.8,
                        max_concurrent_requests: 80,
                        timeout: Duration::from_secs(10),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                ],
                load_balancing_strategy: LoadBalancingStrategy::LatencyBased,
                health_check_config: HealthCheckConfig {
                    interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(5),
                    unhealthy_threshold: 3,
                    healthy_threshold: 2,
                    method: HealthCheckMethod::EthBlockNumber,
                },
            },
        );

        // BSC
        networks.insert(
            NetworkId(56),
            NetworkCluster {
                primary_config: NetworkConfig {
                    id: NetworkId(56),
                    name: "BSC Mainnet".to_string(),
                    rpc_url: "https://bsc.llamarpc.com".to_string(),
                    chain_id: 56,
                    symbol: "BNB".to_string(),
                    block_explorer_url: "https://bscscan.com".to_string(),
                    is_testnet: false,
                    is_custom: false,
                },
                endpoints: vec![
                    RpcEndpoint {
                        url: "https://bsc.llamarpc.com".to_string(),
                        priority: 1,
                        weight: 1.0,
                        max_concurrent_requests: 100,
                        timeout: Duration::from_secs(8),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                    RpcEndpoint {
                        url: "https://rpc.ankr.com/bsc".to_string(),
                        priority: 2,
                        weight: 0.8,
                        max_concurrent_requests: 80,
                        timeout: Duration::from_secs(10),
                        health_status: EndpointHealth::Unknown,
                        performance_metrics: PerformanceMetrics::default(),
                        auth_config: None,
                    },
                ],
                load_balancing_strategy: LoadBalancingStrategy::WeightedRoundRobin,
                health_check_config: HealthCheckConfig {
                    interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(5),
                    unhealthy_threshold: 3,
                    healthy_threshold: 2,
                    method: HealthCheckMethod::EthBlockNumber,
                },
            },
        );

        info!(
            "📡 Initialized {} default networks with redundant endpoints",
            networks.len()
        );
        Ok(())
    }

    /// Error when trying to use professional features without feature enabled
    #[cfg(not(feature = "professional"))]
    pub async fn new() -> Result<Self> {
        Err(NetworkError::InvalidConfiguration.into())
    }

    /// Start health monitoring for all endpoints
    async fn start_health_monitoring(&self) {
        let networks = self.networks.clone();
        let health_monitor = self.health_monitor.clone();
        let http_client = self.http_client.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;

                let networks_guard = networks.read().await;
                for (_network_id, cluster) in networks_guard.iter() {
                    for endpoint in &cluster.endpoints {
                        let endpoint_url = endpoint.url.clone();
                        let client = http_client.clone();
                        let monitor = health_monitor.clone();
                        let config = cluster.health_check_config.clone();

                        tokio::spawn(async move {
                            monitor.check_endpoint_health(&endpoint_url, &client, &config).await;
                        });
                    }
                }
            }
        });

        info!("🔍 Health monitoring started for all network endpoints");
    }

    /// Get the best available endpoint for a network
    pub async fn get_best_endpoint(&self, network_id: NetworkId) -> Result<String> {
        let networks = self.networks.read().await;
        let cluster = networks.get(&network_id).ok_or(NetworkError::UnsupportedNetwork {
            network_id: network_id.0,
        })?;

        // Check rate limits
        if !self.rate_limiter.check_rate_limit("global").await {
            return Err(NetworkError::NetworkError {
                message: "Rate limit exceeded".to_string(),
            }
            .into());
        }

        // Get active connections
        let active_connections = self.load_balancer.active_connections.read().await;

        // Filter healthy endpoints
        let healthy_endpoints: Vec<&RpcEndpoint> = cluster
            .endpoints
            .iter()
            .filter(|ep| ep.health_status == EndpointHealth::Healthy || ep.health_status == EndpointHealth::Unknown)
            .collect();

        if healthy_endpoints.is_empty() {
            warn!(
                "⚠️ No healthy endpoints available for network {}, using fallback",
                network_id.0
            );
            return Ok(cluster.primary_config.rpc_url.clone());
        }

        // Use load balancer to select endpoint
        let strategy = &cluster.load_balancing_strategy;
        if let Some(algorithm) = self.load_balancer.strategies.get(strategy) {
            if let Some(index) = algorithm.select_endpoint(&cluster.endpoints, &active_connections).await {
                return Ok(cluster.endpoints[index].url.clone());
            }
        }

        // Fallback to first healthy endpoint
        Ok(healthy_endpoints[0].url.clone())
    }

    /// Make a request with automatic failover and retries
    pub async fn request_with_failover<T>(
        &self,
        network_id: NetworkId,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            attempts += 1;

            match self.make_request_internal(network_id, method, params.clone()).await {
                Ok(response) => {
                    // Update statistics
                    self.update_success_stats(network_id).await;
                    return Ok(response);
                }
                Err(e) => {
                    self.update_failure_stats(network_id).await;

                    if attempts >= max_attempts {
                        error!(
                            "❌ All {} attempts failed for {} on network {}: {}",
                            max_attempts, method, network_id.0, e
                        );
                        return Err(e);
                    }

                    warn!("⚠️ Request failed (attempt {}), retrying: {}", attempts, e);

                    // Exponential backoff
                    let delay = Duration::from_millis(100 * (1 << (attempts - 1)));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(NetworkError::NetworkError {
            message: "Maximum retry attempts exceeded".to_string(),
        }
        .into())
    }

    async fn make_request_internal<T>(
        &self,
        network_id: NetworkId,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Check cache first
        let cache_key = format!(
            "{}:{}:{}",
            network_id.0,
            method,
            serde_json::to_string(&params).unwrap_or_default()
        );

        if let Some(cached) = self.cache.get(&cache_key).await {
            debug!("📦 Cache hit for {}", cache_key);
            return serde_json::from_str(&cached).map_err(|e| {
                NetworkError::NetworkError {
                    message: format!("Cache deserialization error: {e}"),
                }
                .into()
            });
        }

        // Get best endpoint
        let endpoint_url = self.get_best_endpoint(network_id).await?;

        // Check circuit breaker
        if !self.circuit_breaker.allow_request(&endpoint_url).await {
            return Err(NetworkError::NetworkError {
                message: format!("Circuit breaker open for endpoint: {endpoint_url}"),
            }
            .into());
        }

        let request_start = Instant::now();

        // Build JSON-RPC request
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        // Make the request
        let response = self
            .http_client
            .post(&endpoint_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| NetworkError::RpcError {
                message: format!("HTTP request failed: {e}"),
            })?;

        let response_text = response.text().await.map_err(|e| NetworkError::RpcError {
            message: format!("Failed to read response: {e}"),
        })?;

        let elapsed = request_start.elapsed();

        // Parse JSON-RPC response
        let rpc_response: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| NetworkError::RpcError {
                message: format!("JSON parsing failed: {e}"),
            })?;

        if let Some(error) = rpc_response.get("error") {
            self.circuit_breaker.record_failure(&endpoint_url).await;
            return Err(NetworkError::RpcError {
                message: format!("RPC error: {error}"),
            }
            .into());
        }

        let result = rpc_response.get("result").ok_or_else(|| NetworkError::RpcError {
            message: "No result field in response".to_string(),
        })?;

        // Record success
        self.circuit_breaker.record_success(&endpoint_url).await;

        // Update performance metrics
        self.update_endpoint_metrics(&endpoint_url, elapsed, true).await;

        // Cache the result for cacheable methods
        if self.is_cacheable_method(method) {
            self.cache.set(cache_key, response_text, Duration::from_secs(60)).await;
        }

        serde_json::from_value(result.clone()).map_err(|e| {
            NetworkError::RpcError {
                message: format!("Result deserialization failed: {e}"),
            }
            .into()
        })
    }

    fn is_cacheable_method(&self, method: &str) -> bool {
        matches!(
            method,
            "eth_chainId" | "net_version" | "eth_getBlockByNumber" | "eth_getCode"
        )
    }

    async fn update_endpoint_metrics(&self, endpoint_url: &str, latency: Duration, success: bool) {
        let mut stats = self.stats.write().await;

        let endpoint_stats = stats
            .endpoint_stats
            .entry(endpoint_url.to_string())
            .or_insert_with(EndpointStats::default);

        endpoint_stats.requests += 1;
        endpoint_stats.last_used = Some(SystemTime::now());

        if success {
            // Update average latency (simple moving average)
            let current_avg = endpoint_stats.average_latency;
            let new_avg = if endpoint_stats.requests == 1 {
                latency
            } else {
                Duration::from_nanos((current_avg.as_nanos() as u64 + latency.as_nanos() as u64) / 2)
            };
            endpoint_stats.average_latency = new_avg;
        } else {
            endpoint_stats.failures += 1;
        }
    }

    async fn update_success_stats(&self, _network_id: NetworkId) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.successful_requests += 1;
    }

    async fn update_failure_stats(&self, _network_id: NetworkId) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.failed_requests += 1;
    }

    /// Get network statistics
    pub async fn get_network_statistics(&self) -> NetworkStatistics {
        self.stats.read().await.clone()
    }

    /// Add a custom network with multiple endpoints
    pub async fn add_network_cluster(&self, cluster: NetworkCluster) -> Result<()> {
        let network_id = cluster.primary_config.id;
        let mut networks = self.networks.write().await;

        // Validate endpoints
        for endpoint in &cluster.endpoints {
            if let Err(_e) = Url::parse(&endpoint.url) {
                return Err(NetworkError::InvalidConfiguration.into());
            }
        }

        networks.insert(network_id, cluster);

        let endpoint_count = networks.get(&network_id).map(|n| n.endpoints.len()).unwrap_or(0);
        info!(
            "📡 Added network cluster for {} with {} endpoints",
            network_id.0, endpoint_count
        );

        Ok(())
    }

    /// Get current network
    pub async fn current_network(&self) -> NetworkId {
        *self.current_network.read().await
    }

    /// Switch to a different network
    pub async fn switch_network(&self, network_id: NetworkId) -> Result<()> {
        let networks = self.networks.read().await;
        if !networks.contains_key(&network_id) {
            return Err(NetworkError::UnsupportedNetwork {
                network_id: network_id.0,
            }
            .into());
        }

        *self.current_network.write().await = network_id;
        info!("🔄 Switched to network {}", network_id.0);

        Ok(())
    }
}

// Implementation details for supporting structures
impl LoadBalancer {
    fn new() -> Self {
        let mut strategies: HashMap<LoadBalancingStrategy, Box<dyn LoadBalancingAlgorithm>> = HashMap::new();

        strategies.insert(LoadBalancingStrategy::RoundRobin, Box::new(RoundRobinBalancer::new()));
        strategies.insert(
            LoadBalancingStrategy::WeightedRoundRobin,
            Box::new(WeightedRoundRobinBalancer::new()),
        );
        strategies.insert(
            LoadBalancingStrategy::LeastConnections,
            Box::new(LeastConnectionsBalancer::new()),
        );
        strategies.insert(
            LoadBalancingStrategy::LatencyBased,
            Box::new(LatencyBasedBalancer::new()),
        );
        strategies.insert(LoadBalancingStrategy::HealthBased, Box::new(HealthBasedBalancer::new()));

        Self {
            strategies,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl NetworkHealthMonitor {
    fn new() -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            monitoring_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn check_endpoint_health(&self, endpoint_url: &str, client: &Client, config: &HealthCheckConfig) {
        let start_time = Instant::now();

        let health_result = match config.method {
            HealthCheckMethod::EthBlockNumber => {
                self.check_eth_block_number(endpoint_url, client, config.timeout).await
            }
            HealthCheckMethod::EthChainId => self.check_eth_chain_id(endpoint_url, client, config.timeout).await,
            HealthCheckMethod::NetVersion => self.check_net_version(endpoint_url, client, config.timeout).await,
            HealthCheckMethod::Custom(ref method) => {
                self.check_custom_method(endpoint_url, client, method, config.timeout)
                    .await
            }
        };

        let latency = start_time.elapsed();

        let mut health_checks = self.health_checks.write().await;
        let state = health_checks
            .entry(endpoint_url.to_string())
            .or_insert_with(|| HealthCheckState {
                last_check: SystemTime::now(),
                consecutive_failures: 0,
                consecutive_successes: 0,
                current_status: EndpointHealth::Unknown,
            });

        state.last_check = SystemTime::now();

        match health_result {
            Ok(_) => {
                state.consecutive_successes += 1;
                state.consecutive_failures = 0;

                if state.consecutive_successes >= config.healthy_threshold {
                    state.current_status = EndpointHealth::Healthy;
                }

                debug!("✅ Health check passed for {} (latency: {:?})", endpoint_url, latency);
            }
            Err(e) => {
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;

                if state.consecutive_failures >= config.unhealthy_threshold {
                    state.current_status = EndpointHealth::Unhealthy;
                } else if state.consecutive_failures > 0 {
                    state.current_status = EndpointHealth::Degraded;
                }

                warn!("❌ Health check failed for {}: {}", endpoint_url, e);
            }
        }
    }

    async fn check_eth_block_number(&self, endpoint_url: &str, client: &Client, timeout: Duration) -> Result<u64> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        });

        let response = tokio::time::timeout(timeout, client.post(endpoint_url).json(&request).send())
            .await
            .map_err(|_| NetworkError::Timeout)?
            .map_err(|e| NetworkError::NetworkError { message: e.to_string() })?;

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| NetworkError::NetworkError { message: e.to_string() })?;

        if let Some(result) = response_json.get("result") {
            if let Some(block_hex) = result.as_str() {
                let block_number = u64::from_str_radix(block_hex.trim_start_matches("0x"), 16).map_err(|e| {
                    NetworkError::NetworkError {
                        message: format!("Invalid block number: {e}"),
                    }
                })?;

                return Ok(block_number);
            }
        }

        Err(NetworkError::NetworkError {
            message: "Invalid health check response".to_string(),
        }
        .into())
    }

    async fn check_eth_chain_id(&self, _endpoint_url: &str, _client: &Client, _timeout: Duration) -> Result<u64> {
        // Similar implementation for chain ID check
        Ok(1) // Placeholder
    }

    async fn check_net_version(&self, _endpoint_url: &str, _client: &Client, _timeout: Duration) -> Result<u64> {
        // Similar implementation for net version check - returning as u64 for consistency
        Ok(1) // Placeholder
    }

    async fn check_custom_method(
        &self,
        _endpoint_url: &str,
        _client: &Client,
        _method: &str,
        _timeout: Duration,
    ) -> Result<u64> {
        // Custom health check method implementation - returning as u64 for consistency
        Ok(1) // Placeholder
    }
}

impl RateLimiter {
    pub fn new(global_rate: u32, global_burst: u32) -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
            global_limiter: TokenBucket::new(global_rate, global_burst),
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> bool {
        if key == "global" {
            return self.global_limiter.consume(1).await;
        }

        let limiters = self.limiters.read().await;
        if let Some(limiter) = limiters.get(key) {
            limiter.consume(1).await
        } else {
            // Create new limiter for this key
            drop(limiters);
            let mut limiters = self.limiters.write().await;
            let limiter = TokenBucket::new(60, 100); // 60 RPS, 100 burst per endpoint
            let result = limiter.consume(1).await;
            limiters.insert(key.to_string(), limiter);
            result
        }
    }
}

impl TokenBucket {
    fn new(refill_rate: u32, capacity: u32) -> Self {
        Self {
            capacity,
            tokens: Arc::new(RwLock::new(capacity)),
            refill_rate,
            last_refill: Arc::new(RwLock::new(Instant::now())),
        }
    }

    async fn consume(&self, tokens: u32) -> bool {
        self.refill_tokens().await;

        let mut current_tokens = self.tokens.write().await;
        if *current_tokens >= tokens {
            *current_tokens -= tokens;
            true
        } else {
            false
        }
    }

    async fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_secs(1) {
            let tokens_to_add = (elapsed.as_secs() as u32) * self.refill_rate;
            let mut current_tokens = self.tokens.write().await;
            *current_tokens = (*current_tokens + tokens_to_add).min(self.capacity);
            *last_refill = now;
        }
    }
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration, half_open_max_calls: u32) -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            failure_threshold,
            timeout,
            half_open_max_calls,
        }
    }

    pub async fn allow_request(&self, endpoint: &str) -> bool {
        let breakers = self.breakers.read().await;
        if let Some(state) = breakers.get(endpoint) {
            match state.state {
                BreakerState::Closed => true,
                BreakerState::Open => {
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() > self.timeout {
                            // Transition to half-open
                            drop(breakers);
                            self.transition_to_half_open(endpoint).await;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                BreakerState::HalfOpen => state.half_open_successes < self.half_open_max_calls,
            }
        } else {
            true // No breaker state, allow request
        }
    }

    pub async fn record_success(&self, endpoint: &str) {
        let mut breakers = self.breakers.write().await;
        let state = breakers
            .entry(endpoint.to_string())
            .or_insert_with(|| CircuitBreakerState {
                state: BreakerState::Closed,
                failure_count: 0,
                last_failure_time: None,
                half_open_successes: 0,
            });

        match state.state {
            BreakerState::HalfOpen => {
                state.half_open_successes += 1;
                if state.half_open_successes >= self.half_open_max_calls {
                    // Transition back to closed
                    state.state = BreakerState::Closed;
                    state.failure_count = 0;
                    state.half_open_successes = 0;
                }
            }
            _ => {
                state.failure_count = 0;
            }
        }
    }

    pub async fn record_failure(&self, endpoint: &str) {
        let mut breakers = self.breakers.write().await;
        let state = breakers
            .entry(endpoint.to_string())
            .or_insert_with(|| CircuitBreakerState {
                state: BreakerState::Closed,
                failure_count: 0,
                last_failure_time: None,
                half_open_successes: 0,
            });

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        if state.failure_count >= self.failure_threshold {
            state.state = BreakerState::Open;
        }
    }

    async fn transition_to_half_open(&self, endpoint: &str) {
        let mut breakers = self.breakers.write().await;
        if let Some(state) = breakers.get_mut(endpoint) {
            state.state = BreakerState::HalfOpen;
            state.half_open_successes = 0;
        }
    }
}

impl RequestCache {
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            default_ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed().unwrap_or_default() < cached.ttl {
                return Some(cached.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: String, ttl: Duration) {
        let mut cache = self.cache.write().await;

        // Evict expired entries if cache is full
        if cache.len() >= self.max_size {
            self.evict_expired(&mut cache);

            // If still full, evict oldest entry
            if cache.len() >= self.max_size {
                if let Some(oldest_key) = cache.keys().next().cloned() {
                    cache.remove(&oldest_key);
                }
            }
        }

        cache.insert(
            key,
            CachedResponse {
                data: value,
                timestamp: SystemTime::now(),
                ttl,
            },
        );
    }

    fn evict_expired(&self, cache: &mut HashMap<String, CachedResponse>) {
        let _now = SystemTime::now();
        cache.retain(|_, cached| cached.timestamp.elapsed().unwrap_or_default() < cached.ttl);
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_latency: Duration::from_millis(0),
            success_rate: 1.0,
            requests_per_second: 0.0,
            last_successful_request: None,
            consecutive_failures: 0,
            total_requests: 0,
            total_failures: 0,
        }
    }
}

// Load balancing algorithm implementations
#[derive(Debug)]
struct RoundRobinBalancer {
    counter: Arc<RwLock<usize>>,
}

impl RoundRobinBalancer {
    fn new() -> Self {
        Self {
            counter: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancingAlgorithm for RoundRobinBalancer {
    async fn select_endpoint(
        &self,
        endpoints: &[RpcEndpoint],
        _active_connections: &HashMap<String, u32>,
    ) -> Option<usize> {
        if endpoints.is_empty() {
            return None;
        }

        let mut counter = self.counter.write().await;
        let index = *counter % endpoints.len();
        *counter = (*counter + 1) % endpoints.len();
        Some(index)
    }
}

#[derive(Debug)]
struct WeightedRoundRobinBalancer {
    weights: Arc<RwLock<Vec<f32>>>,
    current_weights: Arc<RwLock<Vec<f32>>>,
}

impl WeightedRoundRobinBalancer {
    fn new() -> Self {
        Self {
            weights: Arc::new(RwLock::new(Vec::new())),
            current_weights: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancingAlgorithm for WeightedRoundRobinBalancer {
    async fn select_endpoint(
        &self,
        endpoints: &[RpcEndpoint],
        _active_connections: &HashMap<String, u32>,
    ) -> Option<usize> {
        if endpoints.is_empty() {
            return None;
        }

        // Initialize weights if needed
        let mut weights = self.weights.write().await;
        let mut current_weights = self.current_weights.write().await;

        if weights.len() != endpoints.len() {
            *weights = endpoints.iter().map(|ep| ep.weight).collect();
            *current_weights = vec![0.0; endpoints.len()];
        }

        // Update current weights
        for (i, &weight) in weights.iter().enumerate() {
            current_weights[i] += weight;
        }

        // Find endpoint with highest current weight
        let mut best_index = 0;
        let mut best_weight = current_weights[0];

        for (i, &weight) in current_weights.iter().enumerate().skip(1) {
            if weight > best_weight {
                best_weight = weight;
                best_index = i;
            }
        }

        // Reduce selected endpoint's current weight by total weight
        let total_weight: f32 = weights.iter().sum();
        current_weights[best_index] -= total_weight;

        Some(best_index)
    }
}

#[derive(Debug)]
struct LeastConnectionsBalancer;

impl LeastConnectionsBalancer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LoadBalancingAlgorithm for LeastConnectionsBalancer {
    async fn select_endpoint(
        &self,
        endpoints: &[RpcEndpoint],
        active_connections: &HashMap<String, u32>,
    ) -> Option<usize> {
        if endpoints.is_empty() {
            return None;
        }

        let mut best_index = 0;
        let mut min_connections = active_connections.get(&endpoints[0].url).unwrap_or(&0);

        for (i, endpoint) in endpoints.iter().enumerate().skip(1) {
            let connections = active_connections.get(&endpoint.url).unwrap_or(&0);
            if connections < min_connections {
                min_connections = connections;
                best_index = i;
            }
        }

        Some(best_index)
    }
}

#[derive(Debug)]
struct LatencyBasedBalancer;

impl LatencyBasedBalancer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LoadBalancingAlgorithm for LatencyBasedBalancer {
    async fn select_endpoint(
        &self,
        endpoints: &[RpcEndpoint],
        _active_connections: &HashMap<String, u32>,
    ) -> Option<usize> {
        if endpoints.is_empty() {
            return None;
        }

        let mut best_index = 0;
        let mut best_latency = endpoints[0].performance_metrics.average_latency;

        for (i, endpoint) in endpoints.iter().enumerate().skip(1) {
            let latency = endpoint.performance_metrics.average_latency;
            if latency < best_latency {
                best_latency = latency;
                best_index = i;
            }
        }

        Some(best_index)
    }
}

#[derive(Debug)]
struct HealthBasedBalancer;

impl HealthBasedBalancer {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LoadBalancingAlgorithm for HealthBasedBalancer {
    async fn select_endpoint(
        &self,
        endpoints: &[RpcEndpoint],
        _active_connections: &HashMap<String, u32>,
    ) -> Option<usize> {
        if endpoints.is_empty() {
            return None;
        }

        // Prefer healthy endpoints, then degraded, avoid unhealthy
        for (i, endpoint) in endpoints.iter().enumerate() {
            if endpoint.health_status == EndpointHealth::Healthy {
                return Some(i);
            }
        }

        for (i, endpoint) in endpoints.iter().enumerate() {
            if endpoint.health_status == EndpointHealth::Degraded {
                return Some(i);
            }
        }

        // Fallback to first endpoint if all are unhealthy
        Some(0)
    }
}
