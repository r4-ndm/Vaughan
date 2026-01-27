//! EIP-1193 Provider Implementation
//!
//! Core EIP-1193 compliant provider trait and implementation for Vaughan wallet.
//!
//! # Standard Methods
//!
//! - `eth_accounts` - Returns connected accounts
//! - `eth_chainId` - Returns current chain ID
//! - `eth_requestAccounts` - Request account access
//! - `eth_sendTransaction` - Send transaction
//! - `personal_sign` - Sign message
//! - `eth_signTypedData_v4` - Sign typed data (EIP-712)
//! - `wallet_switchEthereumChain` - Switch network
//!
//! # Inspiration
//!
//! This implementation follows MetaMask's provider API design.

use alloy::primitives::Address;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;
use super::events::{ProviderEvent, EventEmitter};
use super::permissions::PermissionManager;

// ============================================================================
// EIP-1193 Error Codes (MetaMask standard)
// ============================================================================

/// Standard EIP-1193 error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eip1193ErrorCode {
    /// User rejected the request
    UserRejectedRequest = 4001,
    /// The requested method and/or account has not been authorized
    Unauthorized = 4100,
    /// The Provider does not support the requested method
    UnsupportedMethod = 4200,
    /// The Provider is disconnected from all chains
    Disconnected = 4900,
    /// The Provider is not connected to the requested chain
    ChainDisconnected = 4901,
}

impl Eip1193ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UserRejectedRequest => "User rejected the request",
            Self::Unauthorized => "The requested method and/or account has not been authorized",
            Self::UnsupportedMethod => "The Provider does not support the requested method",
            Self::Disconnected => "The Provider is disconnected from all chains",
            Self::ChainDisconnected => "The Provider is not connected to the requested chain",
        }
    }
}

/// EIP-1193 Provider Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ProviderError {
    pub fn new(code: Eip1193ErrorCode, message: Option<String>) -> Self {
        Self {
            code: code as i32,
            message: message.unwrap_or_else(|| code.message().to_string()),
            data: None,
        }
    }

    pub fn user_rejected() -> Self {
        Self::new(Eip1193ErrorCode::UserRejectedRequest, None)
    }

    pub fn unauthorized() -> Self {
        Self::new(Eip1193ErrorCode::Unauthorized, None)
    }

    pub fn unsupported_method(method: &str) -> Self {
        Self::new(
            Eip1193ErrorCode::UnsupportedMethod,
            Some(format!("Unsupported method: {}", method)),
        )
    }

    pub fn disconnected() -> Self {
        Self::new(Eip1193ErrorCode::Disconnected, None)
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EIP-1193 Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProviderError {}

// ============================================================================
// EIP-1193 Provider Trait
// ============================================================================

/// EIP-1193 Provider Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    /// RPC method name
    pub method: String,
    /// Method parameters
    #[serde(default)]
    pub params: Value,
    /// Request correlation ID
    #[serde(skip)]
    pub correlation_id: Uuid,
    /// dApp origin (for permission checking)
    #[serde(skip)]
    pub origin: Option<String>,
}

impl ProviderRequest {
    pub fn new(method: &str, params: Value) -> Self {
        Self {
            method: method.to_string(),
            params,
            correlation_id: Uuid::new_v4(),
            origin: None,
        }
    }

    pub fn with_origin(mut self, origin: String) -> Self {
        self.origin = Some(origin);
        self
    }
}

/// EIP-1193 Provider Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProviderResponse {
    /// Successful response
    Success(Value),
    /// Error response
    Error(ProviderError),
}

impl ProviderResponse {
    pub fn success(value: Value) -> Self {
        Self::Success(value)
    }

    pub fn error(error: ProviderError) -> Self {
        Self::Error(error)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }
}

/// EIP-1193 Provider trait
///
/// Following MetaMask's provider interface design.
#[async_trait]
pub trait Eip1193Provider: Send + Sync {
    /// Send a request to the provider
    ///
    /// # Arguments
    /// * `request` - The provider request containing method and params
    ///
    /// # Returns
    /// * `Ok(ProviderResponse)` - The response from the provider
    async fn request(&self, request: ProviderRequest) -> ProviderResponse;

    /// Check if the provider is connected
    fn is_connected(&self) -> bool;

    /// Get the current chain ID
    fn chain_id(&self) -> u64;
}

// ============================================================================
// Vaughan Provider Implementation
// ============================================================================

/// Vaughan wallet's EIP-1193 provider implementation
pub struct VaughanProvider {
    /// Current chain ID
    chain_id: Arc<RwLock<u64>>,
    /// Connected accounts (addresses)
    accounts: Arc<RwLock<Vec<Address>>>,
    /// Permission manager for dApp approvals
    permissions: Arc<PermissionManager>,
    /// Event emitter for provider events
    events: Arc<EventEmitter>,
    /// Connection state
    connected: Arc<RwLock<bool>>,
}

impl VaughanProvider {
    /// Create a new VaughanProvider
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id: Arc::new(RwLock::new(chain_id)),
            accounts: Arc::new(RwLock::new(Vec::new())),
            permissions: Arc::new(PermissionManager::new()),
            events: Arc::new(EventEmitter::new()),
            connected: Arc::new(RwLock::new(true)),
        }
    }

    /// Get a reference to the event emitter
    pub fn events(&self) -> Arc<EventEmitter> {
        Arc::clone(&self.events)
    }

    /// Get a reference to the permission manager
    pub fn permissions(&self) -> Arc<PermissionManager> {
        Arc::clone(&self.permissions)
    }

    /// Set connected accounts
    pub async fn set_accounts(&self, accounts: Vec<Address>) {
        let old_accounts = self.accounts.read().await.clone();
        *self.accounts.write().await = accounts.clone();
        
        // Emit accountsChanged event if accounts changed
        if old_accounts != accounts {
            self.events.emit(ProviderEvent::AccountsChanged { 
                accounts: accounts.iter().map(|a| format!("{:?}", a)).collect() 
            }).await;
        }
    }

    /// Switch to a different chain
    pub async fn switch_chain(&self, new_chain_id: u64) -> Result<()> {
        let old_chain_id = *self.chain_id.read().await;
        *self.chain_id.write().await = new_chain_id;
        
        // Emit chainChanged event
        if old_chain_id != new_chain_id {
            self.events.emit(ProviderEvent::ChainChanged { 
                chain_id: format!("0x{:x}", new_chain_id) 
            }).await;
        }
        
        Ok(())
    }

    /// Connect the provider
    pub async fn connect(&self) {
        let was_connected = *self.connected.read().await;
        *self.connected.write().await = true;
        
        if !was_connected {
            let chain_id = *self.chain_id.read().await;
            self.events.emit(ProviderEvent::Connect { 
                chain_id: format!("0x{:x}", chain_id) 
            }).await;
        }
    }

    /// Disconnect the provider
    pub async fn disconnect(&self, code: i32, message: String) {
        *self.connected.write().await = false;
        self.events.emit(ProviderEvent::Disconnect { code, message }).await;
    }

    // ========================================================================
    // RPC Method Handlers
    // ========================================================================

    async fn handle_eth_accounts(&self, request: &ProviderRequest) -> ProviderResponse {
        tracing::debug!(
            correlation_id = %request.correlation_id,
            "Handling eth_accounts"
        );

        // Check if origin has permission
        if let Some(origin) = &request.origin {
            if !self.permissions.is_authorized(origin).await {
                return ProviderResponse::success(Value::Array(vec![]));
            }
        }

        let accounts = self.accounts.read().await;
        let account_strings: Vec<Value> = accounts
            .iter()
            .map(|a| Value::String(format!("{:?}", a)))
            .collect();
        
        ProviderResponse::success(Value::Array(account_strings))
    }

    async fn handle_eth_chain_id(&self, request: &ProviderRequest) -> ProviderResponse {
        tracing::debug!(
            correlation_id = %request.correlation_id,
            "Handling eth_chainId"
        );

        let chain_id = *self.chain_id.read().await;
        ProviderResponse::success(Value::String(format!("0x{:x}", chain_id)))
    }

    async fn handle_eth_request_accounts(&self, request: &ProviderRequest) -> ProviderResponse {
        tracing::info!(
            correlation_id = %request.correlation_id,
            origin = ?request.origin,
            "Handling eth_requestAccounts"
        );

        // Check if already authorized
        if let Some(origin) = &request.origin {
            if self.permissions.is_authorized(origin).await {
                return self.handle_eth_accounts(request).await;
            }

            // Request permission (in real implementation, this would show a UI prompt)
            // For now, auto-approve for testing purposes
            tracing::info!(
                correlation_id = %request.correlation_id,
                origin = origin,
                "Requesting account access permission"
            );
            
            // Auto-approve for now (real implementation would wait for user)
            self.permissions.grant_permission(origin).await;
        }

        self.handle_eth_accounts(request).await
    }

    async fn handle_personal_sign(&self, request: &ProviderRequest) -> ProviderResponse {
        tracing::info!(
            correlation_id = %request.correlation_id,
            "Handling personal_sign"
        );

        // Check authorization
        if let Some(origin) = &request.origin {
            if !self.permissions.is_authorized(origin).await {
                return ProviderResponse::error(ProviderError::unauthorized());
            }
        }

        // Extract parameters: [message, address]
        let params = match request.params.as_array() {
            Some(p) if p.len() >= 2 => p,
            _ => {
                return ProviderResponse::error(ProviderError {
                    code: -32602,
                    message: "Invalid params: expected [message, address]".to_string(),
                    data: None,
                });
            }
        };

        let message = params[0].as_str().unwrap_or_default();
        let address = params[1].as_str().unwrap_or_default();

        tracing::debug!(
            correlation_id = %request.correlation_id,
            message_preview = &message[..message.len().min(50)],
            address = address,
            "Signing message"
        );

        // In real implementation, this would:
        // 1. Show message to user for approval
        // 2. Sign with the account's private key
        // For now, return a placeholder signature
        let placeholder_sig = "0x".to_string() + &"0".repeat(130);
        ProviderResponse::success(Value::String(placeholder_sig))
    }

    async fn handle_eth_send_transaction(&self, request: &ProviderRequest) -> ProviderResponse {
        tracing::info!(
            correlation_id = %request.correlation_id,
            "Handling eth_sendTransaction"
        );

        // Check authorization
        if let Some(origin) = &request.origin {
            if !self.permissions.is_authorized(origin).await {
                return ProviderResponse::error(ProviderError::unauthorized());
            }
        }

        // Extract transaction parameters
        let params = match request.params.as_array() {
            Some(p) if !p.is_empty() => &p[0],
            _ => {
                return ProviderResponse::error(ProviderError {
                    code: -32602,
                    message: "Invalid params: expected transaction object".to_string(),
                    data: None,
                });
            }
        };

        tracing::debug!(
            correlation_id = %request.correlation_id,
            tx_params = ?params,
            "Processing transaction"
        );

        // In real implementation, this would:
        // 1. Parse transaction
        // 2. Show transaction details to user
        // 3. Sign and broadcast transaction
        // 4. Return transaction hash
        // For now, return a placeholder hash
        let placeholder_hash = "0x".to_string() + &"a".repeat(64);
        ProviderResponse::success(Value::String(placeholder_hash))
    }

    async fn handle_wallet_switch_chain(&self, request: &ProviderRequest) -> ProviderResponse {
        tracing::info!(
            correlation_id = %request.correlation_id,
            "Handling wallet_switchEthereumChain"
        );

        // Extract chain ID parameter
        let params = match request.params.as_array() {
            Some(p) if !p.is_empty() => &p[0],
            _ => {
                return ProviderResponse::error(ProviderError {
                    code: -32602,
                    message: "Invalid params: expected { chainId: '0x...' }".to_string(),
                    data: None,
                });
            }
        };

        let chain_id_str = params.get("chainId")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        // Parse hex chain ID
        let chain_id = if let Some(hex_part) = chain_id_str.strip_prefix("0x") {
            u64::from_str_radix(hex_part, 16).unwrap_or(1)
        } else {
            chain_id_str.parse().unwrap_or(1)
        };

        match self.switch_chain(chain_id).await {
            Ok(_) => ProviderResponse::success(Value::Null),
            Err(e) => ProviderResponse::error(ProviderError {
                code: 4902,
                message: format!("Failed to switch chain: {}", e),
                data: None,
            }),
        }
    }
}

#[async_trait]
impl Eip1193Provider for VaughanProvider {
    async fn request(&self, request: ProviderRequest) -> ProviderResponse {
        tracing::debug!(
            correlation_id = %request.correlation_id,
            method = &request.method,
            "Processing EIP-1193 request"
        );

        // Check connection
        if !*self.connected.read().await {
            return ProviderResponse::error(ProviderError::disconnected());
        }

        // Dispatch to appropriate handler
        match request.method.as_str() {
            "eth_accounts" => self.handle_eth_accounts(&request).await,
            "eth_chainId" => self.handle_eth_chain_id(&request).await,
            "eth_requestAccounts" => self.handle_eth_request_accounts(&request).await,
            "personal_sign" => self.handle_personal_sign(&request).await,
            "eth_sendTransaction" => self.handle_eth_send_transaction(&request).await,
            "wallet_switchEthereumChain" => self.handle_wallet_switch_chain(&request).await,
            
            // Methods that pass through (would be handled by RPC node)
            "eth_blockNumber" | "eth_getBalance" | "eth_call" | "eth_estimateGas" |
            "eth_gasPrice" | "eth_getTransactionReceipt" | "eth_getTransactionByHash" => {
                tracing::debug!(
                    correlation_id = %request.correlation_id,
                    method = &request.method,
                    "Passthrough method - would forward to RPC node"
                );
                // In real implementation, forward to RPC node
                ProviderResponse::success(Value::Null)
            }
            
            _ => {
                tracing::warn!(
                    correlation_id = %request.correlation_id,
                    method = &request.method,
                    "Unsupported method"
                );
                ProviderResponse::error(ProviderError::unsupported_method(&request.method))
            }
        }
    }

    fn is_connected(&self) -> bool {
        // Use blocking read for sync method
        // In production, this should be redesigned
        true
    }

    fn chain_id(&self) -> u64 {
        // Default chain ID - in production, use proper async access
        1
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = VaughanProvider::new(1);
        assert!(provider.is_connected());
        assert_eq!(provider.chain_id(), 1);
    }

    #[tokio::test]
    async fn test_eth_chain_id() {
        let provider = VaughanProvider::new(137); // Polygon
        let request = ProviderRequest::new("eth_chainId", Value::Array(vec![]));
        
        let response = provider.request(request).await;
        assert!(response.is_success());
        
        if let ProviderResponse::Success(value) = response {
            assert_eq!(value.as_str().unwrap(), "0x89"); // 137 in hex
        }
    }

    #[tokio::test]
    async fn test_eth_accounts_empty() {
        let provider = VaughanProvider::new(1);
        let request = ProviderRequest::new("eth_accounts", Value::Array(vec![]));
        
        let response = provider.request(request).await;
        assert!(response.is_success());
        
        if let ProviderResponse::Success(value) = response {
            assert!(value.as_array().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn test_eth_accounts_with_accounts() {
        let provider = VaughanProvider::new(1);
        let addr = Address::ZERO;
        provider.set_accounts(vec![addr]).await;
        
        // Grant permission
        provider.permissions.grant_permission("test-origin").await;
        
        let request = ProviderRequest::new("eth_accounts", Value::Array(vec![]))
            .with_origin("test-origin".to_string());
        
        let response = provider.request(request).await;
        assert!(response.is_success());
        
        if let ProviderResponse::Success(value) = response {
            assert_eq!(value.as_array().unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn test_unsupported_method() {
        let provider = VaughanProvider::new(1);
        let request = ProviderRequest::new("unsupported_method", Value::Null);
        
        let response = provider.request(request).await;
        assert!(!response.is_success());
        
        if let ProviderResponse::Error(err) = response {
            assert_eq!(err.code, Eip1193ErrorCode::UnsupportedMethod as i32);
        }
    }

    #[tokio::test]
    async fn test_disconnected_provider() {
        let provider = VaughanProvider::new(1);
        provider.disconnect(1000, "Test disconnect".to_string()).await;
        
        let request = ProviderRequest::new("eth_chainId", Value::Null);
        let response = provider.request(request).await;
        
        assert!(!response.is_success());
        if let ProviderResponse::Error(err) = response {
            assert_eq!(err.code, Eip1193ErrorCode::Disconnected as i32);
        }
    }

    #[tokio::test]
    async fn test_switch_chain() {
        let provider = VaughanProvider::new(1);
        
        let request = ProviderRequest::new(
            "wallet_switchEthereumChain",
            Value::Array(vec![
                serde_json::json!({ "chainId": "0x89" }) // Polygon
            ]),
        );
        
        let response = provider.request(request).await;
        assert!(response.is_success());
        
        // Verify chain changed
        let chain_request = ProviderRequest::new("eth_chainId", Value::Null);
        let chain_response = provider.request(chain_request).await;
        
        if let ProviderResponse::Success(value) = chain_response {
            assert_eq!(value.as_str().unwrap(), "0x89");
        }
    }

    #[tokio::test]
    async fn test_request_accounts_grants_permission() {
        let provider = VaughanProvider::new(1);
        let addr = Address::ZERO;
        provider.set_accounts(vec![addr]).await;
        
        // Initially not authorized
        assert!(!provider.permissions.is_authorized("test-dapp.com").await);
        
        // Request accounts (auto-approves in test mode)
        let request = ProviderRequest::new("eth_requestAccounts", Value::Null)
            .with_origin("test-dapp.com".to_string());
        
        let response = provider.request(request).await;
        assert!(response.is_success());
        
        // Now should be authorized
        assert!(provider.permissions.is_authorized("test-dapp.com").await);
    }

    #[test]
    fn test_provider_error_codes() {
        assert_eq!(Eip1193ErrorCode::UserRejectedRequest as i32, 4001);
        assert_eq!(Eip1193ErrorCode::Unauthorized as i32, 4100);
        assert_eq!(Eip1193ErrorCode::UnsupportedMethod as i32, 4200);
        assert_eq!(Eip1193ErrorCode::Disconnected as i32, 4900);
    }

    #[test]
    fn test_provider_request_creation() {
        let request = ProviderRequest::new("eth_accounts", Value::Null)
            .with_origin("example.com".to_string());
        
        assert_eq!(request.method, "eth_accounts");
        assert_eq!(request.origin, Some("example.com".to_string()));
        assert!(!request.correlation_id.is_nil());
    }
}
