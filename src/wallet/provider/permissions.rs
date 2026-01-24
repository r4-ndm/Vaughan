//! Permission Management for EIP-1193 Provider
//!
//! Manages dApp permissions and authorization for wallet access.
//!
//! # Task Reference
//!
//! Implements: Task 5.3 (Add permission management)
//!
//! # Inspiration
//!
//! This permission model follows MetaMask's dApp permission system.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Permission grant for a dApp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// Origin of the dApp (e.g., "https://app.example.com")
    pub origin: String,
    /// When the permission was granted
    pub granted_at: DateTime<Utc>,
    /// Accounts the dApp has access to
    pub accounts: Vec<String>,
    /// Whether the permission is currently active
    pub active: bool,
}

impl PermissionGrant {
    pub fn new(origin: &str) -> Self {
        Self {
            origin: origin.to_string(),
            granted_at: Utc::now(),
            accounts: Vec::new(),
            active: true,
        }
    }

    pub fn with_accounts(mut self, accounts: Vec<String>) -> Self {
        self.accounts = accounts;
        self
    }
}

/// Permission manager for tracking dApp authorizations
#[derive(Debug)]
pub struct PermissionManager {
    /// Set of authorized origins
    authorized: Arc<RwLock<HashSet<String>>>,
    /// Detailed permission grants
    grants: Arc<RwLock<Vec<PermissionGrant>>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        Self {
            authorized: Arc::new(RwLock::new(HashSet::new())),
            grants: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if an origin is authorized
    pub async fn is_authorized(&self, origin: &str) -> bool {
        self.authorized.read().await.contains(origin)
    }

    /// Grant permission to an origin
    pub async fn grant_permission(&self, origin: &str) {
        tracing::info!(origin = origin, "Granting permission to dApp");
        
        self.authorized.write().await.insert(origin.to_string());
        
        let grant = PermissionGrant::new(origin);
        self.grants.write().await.push(grant);
    }

    /// Grant permission with specific accounts
    pub async fn grant_permission_with_accounts(&self, origin: &str, accounts: Vec<String>) {
        tracing::info!(
            origin = origin,
            accounts = ?accounts,
            "Granting permission with accounts"
        );
        
        self.authorized.write().await.insert(origin.to_string());
        
        let grant = PermissionGrant::new(origin).with_accounts(accounts);
        self.grants.write().await.push(grant);
    }

    /// Revoke permission from an origin
    pub async fn revoke_permission(&self, origin: &str) {
        tracing::info!(origin = origin, "Revoking permission from dApp");
        
        self.authorized.write().await.remove(origin);
        
        // Mark grants as inactive
        let mut grants = self.grants.write().await;
        for grant in grants.iter_mut() {
            if grant.origin == origin {
                grant.active = false;
            }
        }
    }

    /// Revoke all permissions
    pub async fn revoke_all(&self) {
        tracing::info!("Revoking all permissions");
        
        self.authorized.write().await.clear();
        
        let mut grants = self.grants.write().await;
        for grant in grants.iter_mut() {
            grant.active = false;
        }
    }

    /// Get all authorized origins
    pub async fn get_authorized_origins(&self) -> Vec<String> {
        self.authorized.read().await.iter().cloned().collect()
    }

    /// Get all permission grants
    pub async fn get_grants(&self) -> Vec<PermissionGrant> {
        self.grants.read().await.clone()
    }

    /// Get active grants only
    pub async fn get_active_grants(&self) -> Vec<PermissionGrant> {
        self.grants
            .read()
            .await
            .iter()
            .filter(|g| g.active)
            .cloned()
            .collect()
    }

    /// Get grant for a specific origin
    pub async fn get_grant(&self, origin: &str) -> Option<PermissionGrant> {
        self.grants
            .read()
            .await
            .iter()
            .find(|g| g.origin == origin && g.active)
            .cloned()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_manager_creation() {
        let manager = PermissionManager::new();
        assert!(manager.get_authorized_origins().await.is_empty());
    }

    #[tokio::test]
    async fn test_grant_permission() {
        let manager = PermissionManager::new();
        
        assert!(!manager.is_authorized("example.com").await);
        
        manager.grant_permission("example.com").await;
        
        assert!(manager.is_authorized("example.com").await);
    }

    #[tokio::test]
    async fn test_revoke_permission() {
        let manager = PermissionManager::new();
        
        manager.grant_permission("example.com").await;
        assert!(manager.is_authorized("example.com").await);
        
        manager.revoke_permission("example.com").await;
        assert!(!manager.is_authorized("example.com").await);
    }

    #[tokio::test]
    async fn test_multiple_permissions() {
        let manager = PermissionManager::new();
        
        manager.grant_permission("app1.com").await;
        manager.grant_permission("app2.com").await;
        manager.grant_permission("app3.com").await;
        
        assert!(manager.is_authorized("app1.com").await);
        assert!(manager.is_authorized("app2.com").await);
        assert!(manager.is_authorized("app3.com").await);
        
        manager.revoke_permission("app2.com").await;
        
        assert!(manager.is_authorized("app1.com").await);
        assert!(!manager.is_authorized("app2.com").await);
        assert!(manager.is_authorized("app3.com").await);
    }

    #[tokio::test]
    async fn test_revoke_all() {
        let manager = PermissionManager::new();
        
        manager.grant_permission("app1.com").await;
        manager.grant_permission("app2.com").await;
        
        manager.revoke_all().await;
        
        assert!(!manager.is_authorized("app1.com").await);
        assert!(!manager.is_authorized("app2.com").await);
        assert!(manager.get_authorized_origins().await.is_empty());
    }

    #[tokio::test]
    async fn test_get_grants() {
        let manager = PermissionManager::new();
        
        manager.grant_permission("app1.com").await;
        manager.grant_permission_with_accounts(
            "app2.com",
            vec!["0x1234".to_string(), "0x5678".to_string()],
        ).await;
        
        let grants = manager.get_grants().await;
        assert_eq!(grants.len(), 2);
        
        let active_grants = manager.get_active_grants().await;
        assert_eq!(active_grants.len(), 2);
    }

    #[tokio::test]
    async fn test_get_grant_for_origin() {
        let manager = PermissionManager::new();
        
        manager.grant_permission("example.com").await;
        
        let grant = manager.get_grant("example.com").await;
        assert!(grant.is_some());
        assert_eq!(grant.unwrap().origin, "example.com");
        
        let missing = manager.get_grant("unknown.com").await;
        assert!(missing.is_none());
    }

    #[test]
    fn test_permission_grant_creation() {
        let grant = PermissionGrant::new("example.com")
            .with_accounts(vec!["0x1234".to_string()]);
        
        assert_eq!(grant.origin, "example.com");
        assert!(grant.active);
        assert_eq!(grant.accounts.len(), 1);
    }
}
