//! State Coordinators for Cross-Cutting Concerns
//!
//! Phase 5 - Stage 3: Isolation of shared state management
//! This module provides centralized coordination for cross-cutting concerns

use crate::gui::Message;
use crate::network::{NetworkConfig, NetworkId};
use crate::security::SecureAccount;
use iced::Command;
use std::time::Instant;

/// Network state coordinator - manages network-related cross-cutting concerns
#[derive(Debug, Clone)]
pub struct NetworkCoordinator {
    /// Current active network
    pub current_network: NetworkId,
    /// Available networks for user selection
    pub available_networks: Vec<NetworkConfig>,
    /// Current balance for the selected account on this network
    pub current_balance: String,
    /// Network loading state
    pub is_loading: bool,
    /// Last successful balance update
    pub last_balance_update: Option<Instant>,
    /// Subscribers to network changes (for notification pattern)
    subscribers: Vec<String>, // Component IDs that need network change notifications
}

impl NetworkCoordinator {
    /// Create a new network coordinator with default values
    pub fn new() -> Self {
        Self {
            current_network: NetworkId(943), // Default to PulseChain Testnet v4
            available_networks: Vec::new(),
            current_balance: "0.0000".to_string(),
            is_loading: false,
            last_balance_update: None,
            subscribers: Vec::new(),
        }
    }

    /// Change the current network and notify subscribers
    pub fn change_network(&mut self, network_id: NetworkId) -> Vec<Command<Message>> {
        tracing::info!("🔄 NetworkCoordinator: Switching network to {:?}", network_id);

        let _old_network = self.current_network;
        self.current_network = network_id;
        self.is_loading = true;
        self.current_balance = "0.0000".to_string(); // Reset balance during switch

        // Generate commands for network change sequence
        let mut commands = vec![Command::perform(async move { network_id }, Message::NetworkSelected)];

        // Add balance refresh after network change
        commands.push(Command::perform(
            async {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            },
            |_| Message::RefreshBalance,
        ));

        tracing::info!(
            "🔄 NetworkCoordinator: Generated {} commands for network change",
            commands.len()
        );
        commands
    }

    /// Update balance and mark last update time
    pub fn update_balance(&mut self, balance: String) {
        tracing::info!(
            "💰 NetworkCoordinator: Updating balance from '{}' to '{}'",
            self.current_balance,
            balance
        );
        self.current_balance = balance;
        self.last_balance_update = Some(Instant::now());
        self.is_loading = false;
    }

    /// Check if balance is stale and needs refresh
    pub fn is_balance_stale(&self, max_age_seconds: u64) -> bool {
        match self.last_balance_update {
            Some(last_update) => last_update.elapsed().as_secs() > max_age_seconds,
            None => true, // No update ever = definitely stale
        }
    }

    /// Get current network configuration
    pub fn current_network_config(&self) -> Option<&NetworkConfig> {
        self.available_networks.iter().find(|n| n.id == self.current_network)
    }

    /// Subscribe a component to network change notifications
    pub fn subscribe(&mut self, component_id: String) {
        if !self.subscribers.contains(&component_id) {
            self.subscribers.push(component_id);
        }
    }

    /// Get the current network context summary
    pub fn network_context(&self) -> (NetworkId, &str, bool) {
        let network_name = self
            .current_network_config()
            .map(|n| n.name.as_str())
            .unwrap_or("Unknown Network");
        (self.current_network, network_name, self.is_loading)
    }
}

impl Default for NetworkCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Account state coordinator - manages account-related cross-cutting concerns
#[derive(Debug, Clone)]
pub struct AccountCoordinator {
    /// Currently selected account ID
    pub current_account_id: Option<String>,
    /// All available accounts
    pub available_accounts: Vec<SecureAccount>,
    /// Account loading state
    pub is_loading: bool,
    /// Last account operation timestamp
    pub last_operation: Option<Instant>,
    /// Subscribers to account changes
    subscribers: Vec<String>,
}

impl AccountCoordinator {
    /// Create a new account coordinator
    pub fn new() -> Self {
        Self {
            current_account_id: None,
            available_accounts: Vec::new(),
            is_loading: false,
            last_operation: None,
            subscribers: Vec::new(),
        }
    }

    /// Change the current account and notify subscribers
    pub fn change_account(&mut self, account_id: String) -> Vec<Command<Message>> {
        tracing::info!("🔄 AccountCoordinator: Switching account to {}", account_id);

        self.current_account_id = Some(account_id.clone());
        self.last_operation = Some(Instant::now());

        // Generate commands for account change sequence
        vec![
            Command::perform(async {}, |_| Message::RefreshBalance),
            Command::perform(async {}, |_| Message::RefreshTransactionHistory),
        ]
    }

    /// Get current account information
    pub fn current_account(&self) -> Option<&SecureAccount> {
        if let Some(account_id) = &self.current_account_id {
            self.available_accounts.iter().find(|a| &a.id == account_id)
        } else {
            None
        }
    }

    /// Check if user has a valid account selected
    pub fn has_active_account(&self) -> bool {
        self.current_account_id.is_some() && self.current_account().is_some() && !self.is_loading
    }

    /// Subscribe a component to account change notifications
    pub fn subscribe(&mut self, component_id: String) {
        if !self.subscribers.contains(&component_id) {
            self.subscribers.push(component_id);
        }
    }

    /// Get account context summary
    pub fn account_context(&self) -> (Option<String>, Option<String>, bool) {
        let account_name = self.current_account().map(|a| a.name.clone());
        (self.current_account_id.clone(), account_name, self.is_loading)
    }
}

impl Default for AccountCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Loading state coordinator - manages global loading states
#[derive(Debug, Clone)]
pub struct LoadingCoordinator {
    /// Network operations loading
    pub network_loading: bool,
    /// Account operations loading
    pub account_loading: bool,
    /// Transaction operations loading
    pub transaction_loading: bool,
    /// General application loading
    pub app_loading: bool,
    /// Loading operation history for debugging
    pub loading_history: Vec<(String, bool, Instant)>,
}

impl LoadingCoordinator {
    /// Create a new loading coordinator
    pub fn new() -> Self {
        Self {
            network_loading: false,
            account_loading: false,
            transaction_loading: false,
            app_loading: false,
            loading_history: Vec::new(),
        }
    }

    /// Set network loading state
    pub fn set_network_loading(&mut self, loading: bool) {
        self.network_loading = loading;
        self.loading_history
            .push(("network".to_string(), loading, Instant::now()));
        tracing::info!("🔄 LoadingCoordinator: Network loading = {}", loading);
    }

    /// Set account loading state
    pub fn set_account_loading(&mut self, loading: bool) {
        self.account_loading = loading;
        self.loading_history
            .push(("account".to_string(), loading, Instant::now()));
        tracing::info!("🔄 LoadingCoordinator: Account loading = {}", loading);
    }

    /// Set transaction loading state
    pub fn set_transaction_loading(&mut self, loading: bool) {
        self.transaction_loading = loading;
        self.loading_history
            .push(("transaction".to_string(), loading, Instant::now()));
        tracing::info!("🔄 LoadingCoordinator: Transaction loading = {}", loading);
    }

    /// Check if any operations are currently loading
    pub fn is_any_loading(&self) -> bool {
        self.network_loading || self.account_loading || self.transaction_loading || self.app_loading
    }

    /// Get loading state summary
    pub fn loading_summary(&self) -> (bool, Vec<String>) {
        let mut active_operations = Vec::new();

        if self.network_loading {
            active_operations.push("network".to_string());
        }
        if self.account_loading {
            active_operations.push("account".to_string());
        }
        if self.transaction_loading {
            active_operations.push("transaction".to_string());
        }
        if self.app_loading {
            active_operations.push("app".to_string());
        }

        (self.is_any_loading(), active_operations)
    }
}

impl Default for LoadingCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
