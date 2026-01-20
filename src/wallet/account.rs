//! Account management functionality
//!
//! This module provides utilities for managing multiple accounts within the wallet,
//! including account switching, metadata management, and account organization.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Result, WalletError};
use crate::security::SecureAccount;

/// Account metadata for additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadata {
    pub name: String,
    pub created_at: std::time::SystemTime,
    pub last_used: Option<std::time::SystemTime>,
    pub is_hardware_wallet: bool,
    pub derivation_path: Option<String>,
    pub tags: Vec<String>,
}

impl Default for AccountMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed Account".to_string(),
            created_at: std::time::SystemTime::now(),
            last_used: None,
            is_hardware_wallet: false,
            derivation_path: None,
            tags: Vec::new(),
        }
    }
}

/// Account manager for handling multiple accounts
pub struct AccountManager {
    accounts: HashMap<Address, SecureAccount>,
    metadata: HashMap<Address, AccountMetadata>,
    active_account: Option<Address>,
    account_order: Vec<Address>, // For maintaining display order
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            metadata: HashMap::new(),
            active_account: None,
            account_order: Vec::new(),
        }
    }

    /// Add a new account to the manager
    pub fn add_account(&mut self, account: SecureAccount) -> Result<()> {
        let address = account.address;

        // Set as active if no active account
        if self.active_account.is_none() {
            self.active_account = Some(address);
        }

        // Create default metadata
        let metadata = AccountMetadata {
            name: account.name.clone(),
            created_at: std::time::SystemTime::now(),
            last_used: None,
            is_hardware_wallet: false,
            derivation_path: None,
            tags: Vec::new(),
        };

        self.accounts.insert(address, account);
        self.metadata.insert(address, metadata);
        self.account_order.push(address);

        Ok(())
    }

    /// Remove an account from the manager
    pub fn remove_account(&mut self, address: Address) -> Result<SecureAccount> {
        let account = self.accounts.remove(&address).ok_or(WalletError::AccountNotFound {
            address: format!("{address:?}"),
        })?;

        self.metadata.remove(&address);
        self.account_order.retain(|&addr| addr != address);

        // Clear active account if it was the removed one
        if self.active_account == Some(address) {
            self.active_account = self.account_order.first().copied();
        }

        Ok(account)
    }

    /// Get the currently active account
    pub fn get_active_account(&self) -> Option<&SecureAccount> {
        self.active_account.and_then(|addr| self.accounts.get(&addr))
    }

    /// Get account by address
    pub fn get_account(&self, address: Address) -> Option<&SecureAccount> {
        self.accounts.get(&address)
    }

    /// Switch to a different account
    pub fn switch_account(&mut self, address: Address) -> Result<()> {
        if !self.accounts.contains_key(&address) {
            return Err(WalletError::AccountNotFound {
                address: format!("{address:?}"),
            }
            .into());
        }

        self.active_account = Some(address);

        // Update last used timestamp
        if let Some(metadata) = self.metadata.get_mut(&address) {
            metadata.last_used = Some(std::time::SystemTime::now());
        }

        Ok(())
    }

    /// List all accounts in display order
    pub fn list_accounts(&self) -> Vec<&SecureAccount> {
        self.account_order
            .iter()
            .filter_map(|addr| self.accounts.get(addr))
            .collect()
    }

    /// Get account metadata
    pub fn get_metadata(&self, address: Address) -> Option<&AccountMetadata> {
        self.metadata.get(&address)
    }

    /// Update account metadata
    pub fn update_metadata(&mut self, address: Address, metadata: AccountMetadata) -> Result<()> {
        if !self.accounts.contains_key(&address) {
            return Err(WalletError::AccountNotFound {
                address: format!("{address:?}"),
            }
            .into());
        }

        self.metadata.insert(address, metadata);
        Ok(())
    }

    /// Update account name
    pub fn update_account_name(&mut self, address: Address, name: String) -> Result<()> {
        if let Some(metadata) = self.metadata.get_mut(&address) {
            metadata.name = name;
            Ok(())
        } else {
            Err(WalletError::AccountNotFound {
                address: format!("{address:?}"),
            }
            .into())
        }
    }

    /// Add tag to account
    pub fn add_account_tag(&mut self, address: Address, tag: String) -> Result<()> {
        if let Some(metadata) = self.metadata.get_mut(&address) {
            if !metadata.tags.contains(&tag) {
                metadata.tags.push(tag);
            }
            Ok(())
        } else {
            Err(WalletError::AccountNotFound {
                address: format!("{address:?}"),
            }
            .into())
        }
    }

    /// Remove tag from account
    pub fn remove_account_tag(&mut self, address: Address, tag: &str) -> Result<()> {
        if let Some(metadata) = self.metadata.get_mut(&address) {
            metadata.tags.retain(|t| t != tag);
            Ok(())
        } else {
            Err(WalletError::AccountNotFound {
                address: format!("{address:?}"),
            }
            .into())
        }
    }

    /// Get accounts by tag
    pub fn get_accounts_by_tag(&self, tag: &str) -> Vec<&SecureAccount> {
        self.accounts
            .iter()
            .filter(|(addr, _)| {
                self.metadata
                    .get(*addr)
                    .map(|meta| meta.tags.contains(&tag.to_string()))
                    .unwrap_or(false)
            })
            .map(|(_, account)| account)
            .collect()
    }

    /// Reorder accounts
    pub fn reorder_accounts(&mut self, new_order: Vec<Address>) -> Result<()> {
        // Validate that all addresses exist
        for addr in &new_order {
            if !self.accounts.contains_key(addr) {
                return Err(WalletError::AccountNotFound {
                    address: format!("{addr:?}"),
                }
                .into());
            }
        }

        // Ensure all accounts are included
        if new_order.len() != self.accounts.len() {
            return Err(WalletError::InvalidPrivateKey.into()); // Reuse error for invalid operation
        }

        self.account_order = new_order;
        Ok(())
    }

    /// Get account count
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Check if account exists
    pub fn has_account(&self, address: Address) -> bool {
        self.accounts.contains_key(&address)
    }

    /// Get active account address
    pub fn get_active_address(&self) -> Option<Address> {
        self.active_account
    }

    /// Clear all accounts (for wallet reset)
    pub fn clear_all(&mut self) {
        self.accounts.clear();
        self.metadata.clear();
        self.account_order.clear();
        self.active_account = None;
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::KeyReference;
    use anyhow::Context;

    fn create_test_account(address: Address, name: &str) -> SecureAccount {
        SecureAccount {
            id: uuid::Uuid::new_v4().to_string(),
            address,
            name: name.to_string(),
            key_reference: KeyReference {
                id: "test".to_string(),
                service: "test-service".to_string(),
                account: "test-account".to_string(),
            },
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: None,
        }
    }

    #[test]
    fn test_account_manager_basic_operations() -> Result<()> {
        let mut manager = AccountManager::new();
        let addr1 = Address::ZERO;
        let account1 = create_test_account(addr1, "Account 1");

        // Add account
        manager.add_account(account1).context("Operation failed")?;
        assert_eq!(manager.account_count(), 1);
        assert_eq!(manager.get_active_address(), Some(addr1));

        // Get account
        let retrieved = manager
            .get_account(addr1)
            .ok_or_else(|| anyhow::anyhow!("Failed to find retrieved"))?;
        assert_eq!(retrieved.name, "Account 1");

        // Update name
        manager
            .update_account_name(addr1, "Updated Account".to_string())
            .context("Operation failed")?;
        let metadata = manager
            .get_metadata(addr1)
            .ok_or_else(|| anyhow::anyhow!("Failed to find metadata"))?;
        assert_eq!(metadata.name, "Updated Account");
        Ok(())
    }

    #[test]
    fn test_account_tags() -> Result<()> {
        let mut manager = AccountManager::new();
        let addr1 = Address::ZERO;
        let account1 = create_test_account(addr1, "Account 1");

        manager.add_account(account1).context("Operation failed")?;

        // Add tags
        manager
            .add_account_tag(addr1, "trading".to_string())
            .context("Operation failed")?;
        manager
            .add_account_tag(addr1, "main".to_string())
            .context("Operation failed")?;

        let metadata = manager
            .get_metadata(addr1)
            .ok_or_else(|| anyhow::anyhow!("Failed to find metadata"))?;
        assert!(metadata.tags.contains(&"trading".to_string()));
        assert!(metadata.tags.contains(&"main".to_string()));

        // Get by tag
        let trading_accounts = manager.get_accounts_by_tag("trading");
        assert_eq!(trading_accounts.len(), 1);

        // Remove tag
        manager
            .remove_account_tag(addr1, "trading")
            .context("Operation failed")?;
        let metadata = manager
            .get_metadata(addr1)
            .ok_or_else(|| anyhow::anyhow!("Failed to find metadata"))?;
        assert!(!metadata.tags.contains(&"trading".to_string()));
        Ok(())
    }
}
