//! Service Layer for GUI Operations
//!
//! This module provides a clean separation between UI views and business logic.
//! Services handle file I/O, validation, data formatting, and other non-UI concerns.
//!
//! # Architecture
//! - Each service has a trait interface for testability (mocking)
//! - Services are stateless where possible
//! - ServiceRegistry provides lazy initialization and dependency injection
//!
//! # Services
//! - `account_service`: Account creation, import, export operations
//! - `network_service`: Network configuration and management
//! - `token_service`: Custom token persistence
//! - `wallet_service`: Wallet initialization and account loading
//! - `explorer_service`: Block explorer API integration
//! - `auto_balance_service`: Automatic balance monitoring
//! - `qr_service`: QR code generation (feature-gated)
//! - `integrated_account_service`: Unified account service with telemetry
//! - `asset_service`: Asset loading and availability checks (NEW)
//! - `account_display_service`: Account formatting and display logic (NEW)

// Existing services
pub mod account_service;
pub mod auto_balance_service;
pub mod explorer_service;
pub mod integrated_account_service;
pub mod network_service;
#[cfg(feature = "qr")]
pub mod qr_service;
pub mod token_service;
pub mod wallet_service;

// New services for business logic extraction
pub mod asset_service;
pub mod account_display_service;

// Re-exports from existing services
pub use account_service::*;
pub use auto_balance_service::AutoBalanceMessage;
pub use integrated_account_service::IntegratedAccountService;
pub use network_service::*;
pub use token_service::{load_custom_tokens, save_custom_tokens};
pub use wallet_service::{initialize_wallet, load_available_accounts};

// Re-exports from new services
pub use asset_service::{AssetService, AssetServiceTrait};
pub use account_display_service::{AccountDisplayService, AccountDisplayServiceTrait, AccountDisplayInfo};

use std::sync::{Arc, OnceLock};

/// Service registry providing lazy initialization and access to all services.
/// 
/// Services are created on first access and shared via Arc for thread safety.
#[derive(Debug, Default)]
pub struct ServiceRegistry {
    asset_service: OnceLock<Arc<AssetService>>,
    account_display_service: OnceLock<Arc<AccountDisplayService>>,
}

impl Clone for ServiceRegistry {
    fn clone(&self) -> Self {
        // Create a new registry - services will be lazily initialized again
        // This is intentional as OnceLock doesn't implement Clone
        Self::new()
    }
}

impl ServiceRegistry {
    /// Create a new service registry with lazy initialization.
    pub fn new() -> Self {
        Self {
            asset_service: OnceLock::new(),
            account_display_service: OnceLock::new(),
        }
    }

    /// Get the asset service, creating it if necessary.
    pub fn asset(&self) -> Arc<AssetService> {
        self.asset_service
            .get_or_init(|| Arc::new(AssetService::default()))
            .clone()
    }

    /// Get the account display service, creating it if necessary.
    pub fn account_display(&self) -> Arc<AccountDisplayService> {
        self.account_display_service
            .get_or_init(|| Arc::new(AccountDisplayService::new()))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        // Services should be lazily initialized
        assert!(registry.asset_service.get().is_none());
        assert!(registry.account_display_service.get().is_none());
    }

    #[test]
    fn test_service_registry_lazy_init() {
        let registry = ServiceRegistry::new();
        
        // First access should initialize
        let asset1 = registry.asset();
        assert!(registry.asset_service.get().is_some());
        
        // Second access should return same instance
        let asset2 = registry.asset();
        assert!(Arc::ptr_eq(&asset1, &asset2));
    }
}
