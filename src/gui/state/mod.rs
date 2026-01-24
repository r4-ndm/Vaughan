//! State management modules for Vaughan wallet
//!
//! This module contains the decomposed state structs extracted from the monolithic AppState.
//! Each module focuses on a specific domain of the application.

pub mod auth_state;
pub mod network_state;
pub mod transaction_state;
pub mod ui_state;
pub mod wallet_state;

/// Types of wallet export operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    None,
    SeedPhrase,
    PrivateKey,
}

// Re-export auth_state types (including merged session types)
pub use auth_state::{
    AccountCreationType,
    AccountOperation,
    AccountSessionSettings,
    AccountSessionState,
    AccountSessionStatistics,
    AccountSessionSummary,
    AuthState,
    // Enhanced session types (merged from session_state.rs)
    EnhancedSessionState,
    GlobalSessionSettings,
    IdleDetectionSettings,
    LockReason,
    PasswordDialogConfig,
    PasswordDialogState,
    PasswordError,
    SessionCoordinator,
    SessionHealth,
    SessionHealthReport,
    SessionPriority,
    SessionState,
    SessionStatistics,
    SessionSummary,
    SessionValidationError,
    WalletOperation,
    WalletPasswordError,
    WalletSessionState,
};
pub use network_state::NetworkState;
pub use transaction_state::TransactionState;
pub use ui_state::UiState;
pub use wallet_state::WalletState;

use crate::gui::coordinators::{AccountCoordinator, LoadingCoordinator, NetworkCoordinator};
use crate::gui::wallet_types::CancelButtonState;
use crate::network::{NetworkConfig, NetworkId};
use std::time::Instant;

/// Token state - consolidated from token_state.rs
#[derive(Debug, Clone, Default)]
pub struct TokenState {
    /// Custom tokens (minimal implementation)
    pub custom_tokens: Vec<String>,
}

impl TokenState {
    /// Create new token state
    pub fn new() -> Self {
        Self::default()
    }
}

/// Core application state with decomposed modules but exposed fields for compatibility
#[derive(Debug, Clone)]
pub struct AppState {
    // Domain-specific state modules (private for now)
    network: NetworkState,
    wallet: WalletState,
    transaction: TransactionState,
    ui: UiState,
    token: TokenState,
    auth: AuthState,

    // Cross-cutting concern coordinators
    pub network_coordinator: NetworkCoordinator,
    pub account_coordinator: AccountCoordinator,
    pub loading_coordinator: LoadingCoordinator,

    // Core application-level fields (kept at top level for compatibility)
    pub is_loading: bool,
    pub last_activity: Instant,
    pub log_entries: Vec<crate::gui::LogEntry>,

    // Token fields
    pub custom_tokens: Vec<crate::gui::TokenInfo>,
    pub show_custom_token_screen: bool,

    // Export-related fields (non-deprecated)
    pub selected_export_account_id: Option<String>,
    pub exported_private_key: Option<String>,
    pub exported_seed_phrase: Option<String>,
    pub password_for_export: String,
    pub exporting_data: bool,
    pub export_result: Option<String>,
    pub export_loading: bool,
    pub pending_export_type: ExportType,
    pub export_error_message: Option<String>,

    // Custom token fields
    pub custom_token_address_input: String,
    pub custom_token_symbol_input: String,
    pub custom_token_name_input: String,
    pub custom_token_decimals_input: String,
    pub custom_token_validation_error: Option<String>,
    pub pending_token_address: String,
    pub fetching_token_info: bool,

    // Balance and token display fields
    pub balance_selected_token: String,
    pub balance_selected_ticker: String,
    pub balance_available_tokens: Vec<String>,
    pub balance_available_tickers: Vec<String>,
    pub balance_spinner: bool,
    pub account_balance: String,
    pub token_balances: Vec<crate::gui::SimpleTokenBalance>,
    pub last_balance: String,
    /// Flag to indicate an account switch just occurred (skip audio on next balance change)
    pub account_just_switched: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let mut state = Self {
            network: NetworkState::default(),
            wallet: WalletState::default(),
            transaction: TransactionState::default(),
            ui: UiState::default(),
            token: TokenState::default(),
            auth: AuthState::default(),
            network_coordinator: NetworkCoordinator::default(),
            account_coordinator: AccountCoordinator::default(),
            loading_coordinator: LoadingCoordinator::default(),
            is_loading: false,
            last_activity: Instant::now(),
            log_entries: Vec::new(),

            custom_tokens: Vec::new(),
            show_custom_token_screen: false,

            // Export-related fields (non-deprecated)
            selected_export_account_id: None,
            exported_private_key: None,
            exported_seed_phrase: None,
            password_for_export: String::new(),
            exporting_data: false,
            export_result: None,
            export_loading: false,
            pending_export_type: ExportType::None,
            export_error_message: None,

            // Custom token fields
            custom_token_address_input: String::new(),
            custom_token_symbol_input: String::new(),
            custom_token_name_input: String::new(),
            custom_token_decimals_input: "18".to_string(),
            custom_token_validation_error: None,
            pending_token_address: String::new(),
            fetching_token_info: false,

            // Balance and token display fields
            balance_selected_token: "NATIVE (ETH)".to_string(),
            balance_selected_ticker: "ETH".to_string(),
            balance_available_tokens: vec!["NATIVE (ETH)".to_string()],
            balance_available_tickers: vec!["ETH".to_string()],
            balance_spinner: false,
            account_balance: "0.0000".to_string(),
            token_balances: Vec::new(),
            last_balance: "0.0000".to_string(),
            account_just_switched: false,
        };

        // Synchronize coordinators with flattened fields after initialization
        state.sync_coordinators_with_flat_fields();
        state
    }
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        Self::default()
    }
}

/// Compatibility methods for gradual migration from monolithic AppState
/// These methods provide access to nested fields using the original field names
impl AppState {
    // Network state accessors
    pub fn current_network(&self) -> &NetworkId {
        &self.network.current_network
    }
    pub fn current_network_mut(&mut self) -> &mut NetworkId {
        &mut self.network.current_network
    }

    pub fn available_networks(&self) -> &Vec<NetworkConfig> {
        &self.network.available_networks
    }
    pub fn available_networks_mut(&mut self) -> &mut Vec<NetworkConfig> {
        &mut self.network.available_networks
    }

    pub fn balance(&self) -> &String {
        &self.network.balance
    }
    pub fn balance_mut(&mut self) -> &mut String {
        &mut self.network.balance
    }

    pub fn loading_networks(&self) -> bool {
        self.network.loading_networks
    }
    pub fn loading_networks_mut(&mut self) -> &mut bool {
        &mut self.network.loading_networks
    }

    // Additional network state accessors
    pub fn show_add_network(&self) -> bool {
        self.network.show_add_network
    }
    pub fn show_add_network_mut(&mut self) -> &mut bool {
        &mut self.network.show_add_network
    }

    pub fn network_name(&self) -> &String {
        &self.network.network_name
    }
    pub fn network_name_mut(&mut self) -> &mut String {
        &mut self.network.network_name
    }

    pub fn polling_active(&self) -> bool {
        self.network.polling_active
    }
    pub fn polling_active_mut(&mut self) -> &mut bool {
        &mut self.network.polling_active
    }

    // Wallet state accessors
    pub fn current_account(&self) -> &String {
        &self.wallet.current_account
    }
    pub fn current_account_mut(&mut self) -> &mut String {
        &mut self.wallet.current_account
    }

    pub fn current_account_id(&self) -> &Option<String> {
        &self.wallet.current_account_id
    }
    pub fn current_account_id_mut(&mut self) -> &mut Option<String> {
        &mut self.wallet.current_account_id
    }

    pub fn available_accounts(&self) -> &Vec<crate::security::SecureAccount> {
        &self.wallet.available_accounts
    }
    pub fn available_accounts_mut(&mut self) -> &mut Vec<crate::security::SecureAccount> {
        &mut self.wallet.available_accounts
    }

    pub fn loading_accounts(&self) -> bool {
        self.wallet.loading_accounts
    }
    pub fn loading_accounts_mut(&mut self) -> &mut bool {
        &mut self.wallet.loading_accounts
    }

    // Transaction state accessors
    pub fn show_history(&self) -> bool {
        self.transaction.show_history
    }
    pub fn show_history_mut(&mut self) -> &mut bool {
        &mut self.transaction.show_history
    }

    pub fn show_transaction_history(&self) -> bool {
        self.transaction.show_transaction_history
    }
    pub fn show_transaction_history_mut(&mut self) -> &mut bool {
        &mut self.transaction.show_transaction_history
    }

    pub fn transaction_history(&self) -> &Vec<crate::gui::Transaction> {
        &self.transaction.transaction_history
    }
    pub fn transaction_history_mut(&mut self) -> &mut Vec<crate::gui::Transaction> {
        &mut self.transaction.transaction_history
    }

    pub fn loading_transactions(&self) -> bool {
        self.transaction.loading_transactions
    }
    pub fn loading_transactions_mut(&mut self) -> &mut bool {
        &mut self.transaction.loading_transactions
    }

    // UI state accessors
    pub fn status_message(&self) -> &String {
        &self.ui.status_message
    }
    pub fn status_message_mut(&mut self) -> &mut String {
        &mut self.ui.status_message
    }

    pub fn copy_feedback(&self) -> &Option<String> {
        &self.ui.copy_feedback
    }
    pub fn copy_feedback_mut(&mut self) -> &mut Option<String> {
        &mut self.ui.copy_feedback
    }

    // Cross-cutting concern accessors (Phase 5 - Stage 1.5)
    /// Get current loading state across all domains
    pub fn is_loading_state(&self) -> bool {
        self.is_loading
            || self.network.loading_networks
            || self.wallet.loading_accounts
            || self.transaction.loading_transactions
    }

    /// Get core user context (network + account combination)
    pub fn user_context(&self) -> (NetworkId, Option<String>) {
        (self.network.current_network, self.wallet.current_account_id.clone())
    }

    /// Check if user has a complete context (network + account selected)
    pub fn has_complete_context(&self) -> bool {
        self.wallet.current_account_id.is_some() && !self.is_loading_state()
    }

    /// Get activity timestamp for session management
    pub fn activity_timestamp(&self) -> &std::time::Instant {
        &self.last_activity
    }

    /// Update activity timestamp (for user interaction tracking)
    pub fn update_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    // Domain state accessors (new API)
    /// Get the network state module
    pub fn network(&self) -> &NetworkState {
        &self.network
    }

    /// Get mutable access to network state module
    pub fn network_mut(&mut self) -> &mut NetworkState {
        &mut self.network
    }

    /// Get the wallet state module
    pub fn wallet(&self) -> &WalletState {
        &self.wallet
    }

    /// Get mutable access to wallet state module
    pub fn wallet_mut(&mut self) -> &mut WalletState {
        &mut self.wallet
    }

    /// Get the transaction state module
    pub fn transaction(&self) -> &TransactionState {
        &self.transaction
    }

    /// Get mutable access to transaction state module
    pub fn transaction_mut(&mut self) -> &mut TransactionState {
        &mut self.transaction
    }

    /// Get the UI state module
    pub fn ui(&self) -> &UiState {
        &self.ui
    }

    /// Get mutable access to UI state module
    pub fn ui_mut(&mut self) -> &mut UiState {
        &mut self.ui
    }

    /// Get the token state module
    pub fn token(&self) -> &TokenState {
        &self.token
    }

    /// Get mutable access to token state module
    pub fn token_mut(&mut self) -> &mut TokenState {
        &mut self.token
    }

    /// Get the auth state module
    pub fn auth(&self) -> &AuthState {
        &self.auth
    }

    /// Get mutable access to auth state module
    pub fn auth_mut(&mut self) -> &mut AuthState {
        &mut self.auth
    }

    // Coordinator access methods
    /// Get the network coordinator for network state management
    pub fn network_coordinator(&self) -> &NetworkCoordinator {
        &self.network_coordinator
    }

    /// Get mutable access to network coordinator
    pub fn network_coordinator_mut(&mut self) -> &mut NetworkCoordinator {
        &mut self.network_coordinator
    }

    /// Get the account coordinator for account state management
    pub fn account_coordinator(&self) -> &AccountCoordinator {
        &self.account_coordinator
    }

    /// Get mutable access to account coordinator
    pub fn account_coordinator_mut(&mut self) -> &mut AccountCoordinator {
        &mut self.account_coordinator
    }

    /// Get the loading coordinator for global loading state management
    pub fn loading_coordinator(&self) -> &LoadingCoordinator {
        &self.loading_coordinator
    }

    /// Get mutable access to loading coordinator
    pub fn loading_coordinator_mut(&mut self) -> &mut LoadingCoordinator {
        &mut self.loading_coordinator
    }

    // Coordinator-based state change methods
    /// Perform a coordinated network change with notification system
    pub fn change_network_coordinated(&mut self, network_id: NetworkId) -> Vec<iced::Command<crate::gui::Message>> {
        // Update both the coordinator and domain state
        let commands = self.network_coordinator.change_network(network_id);
        self.network.current_network = network_id;

        // Update loading state through coordinator
        self.loading_coordinator.set_network_loading(true);

        tracing::info!("🔄 AppState: Coordinated network change to {:?} initiated", network_id);
        commands
    }

    /// Perform a coordinated account change with notification system
    pub fn change_account_coordinated(&mut self, account_id: String) -> Vec<iced::Command<crate::gui::Message>> {
        // Update both the coordinator and domain state
        let commands = self.account_coordinator.change_account(account_id.clone());
        self.wallet.current_account_id = Some(account_id.clone());
        // Also set send_from_account_id so send button works
        self.transaction.send_from_account_id = Some(account_id);

        // Update loading state through coordinator
        self.loading_coordinator.set_account_loading(true);

        tracing::info!("🔄 AppState: Coordinated account change initiated");
        commands
    }

    /// Sync coordinator state with domain state (updated for Phase E)
    pub fn sync_coordinators_with_flat_fields(&mut self) {
        // Sync network coordinator from network domain
        self.network_coordinator.current_network = self.network.current_network;
        self.network_coordinator.available_networks = self.network.available_networks.clone();
        if !self.network.balance.is_empty() && !self.network.balance.starts_with("0.0") {
            self.network_coordinator.update_balance(self.network.balance.clone());
        }

        // Sync account coordinator from wallet domain
        self.account_coordinator.current_account_id = self.wallet.current_account_id.clone();
        self.account_coordinator.available_accounts = self.wallet.available_accounts.clone();

        // Sync loading coordinator from domain states
        self.loading_coordinator
            .set_network_loading(self.network.loading_networks);
        self.loading_coordinator
            .set_account_loading(self.wallet.loading_accounts);
        self.loading_coordinator
            .set_transaction_loading(self.transaction.loading_transactions);
    }

    /// Apply coordinator state changes back to domain states (updated for Phase E)
    pub fn apply_coordinator_changes(&mut self) {
        // Apply network coordinator changes to network domain
        self.network.current_network = self.network_coordinator.current_network;
        if !self.network_coordinator.current_balance.is_empty() {
            self.network.balance = self.network_coordinator.current_balance.clone();
        }

        // Apply loading coordinator changes to domain states
        self.network.loading_networks = self.loading_coordinator.network_loading;
        self.wallet.loading_accounts = self.loading_coordinator.account_loading;
        self.transaction.loading_transactions = self.loading_coordinator.transaction_loading;

        // Update global loading state
        self.is_loading = self.loading_coordinator.is_any_loading();
    }

    /// Get unified state summary from all coordinators
    pub fn get_coordinator_summary(&self) -> (NetworkId, Option<String>, bool) {
        let (network_id, network_name, _network_loading) = self.network_coordinator.network_context();
        let (_account_id, account_name, _account_loading) = self.account_coordinator.account_context();
        let (is_loading, _operations) = self.loading_coordinator.loading_summary();

        tracing::debug!(
            "📊 Coordinator Summary - Network: {:?} ({}), Account: {}, Loading: {}",
            network_id,
            network_name,
            account_name.as_deref().unwrap_or("None"),
            is_loading
        );

        (network_id, account_name, is_loading)
    }

    // State validation methods for comprehensive state validation
    /// Validate that the application state is internally consistent
    pub fn validate_state_consistency(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate network state consistency
        if let Err(mut network_errors) = self.validate_network_state() {
            errors.append(&mut network_errors);
        }

        // Validate account state consistency
        if let Err(mut account_errors) = self.validate_account_state() {
            errors.append(&mut account_errors);
        }

        // Validate cross-coordinator consistency
        if let Err(mut coordinator_errors) = self.validate_coordinator_consistency() {
            errors.append(&mut coordinator_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate network-related state consistency
    fn validate_network_state(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check that current network exists in available networks
        if !self
            .network
            .available_networks
            .iter()
            .any(|n| n.id == self.network.current_network)
        {
            errors.push(format!(
                "Current network {:?} not found in available networks",
                self.network.current_network
            ));
        }

        // Validate coordinator consistency with domain state
        if self.network_coordinator.current_network != self.network.current_network {
            errors.push(format!(
                "Network coordinator mismatch: coordinator has {:?}, state has {:?}",
                self.network_coordinator.current_network, self.network.current_network
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate account-related state consistency
    fn validate_account_state(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check that current account ID exists in available accounts (if set)
        if let Some(ref account_id) = self.wallet.current_account_id {
            if !self.wallet.available_accounts.iter().any(|a| &a.id == account_id) {
                errors.push(format!(
                    "Current account ID '{account_id}' not found in available accounts"
                ));
            }
        }

        // Validate coordinator consistency with domain state
        if self.account_coordinator.current_account_id != self.wallet.current_account_id {
            errors.push(format!(
                "Account coordinator mismatch: coordinator has {:?}, state has {:?}",
                self.account_coordinator.current_account_id, self.wallet.current_account_id
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate cross-coordinator state consistency
    fn validate_coordinator_consistency(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check loading state consistency
        let coordinator_any_loading = self.loading_coordinator.is_any_loading();
        if coordinator_any_loading != self.is_loading {
            errors.push(format!(
                "Loading state mismatch: coordinator reports {}, state has {}",
                coordinator_any_loading, self.is_loading
            ));
        }

        // Check that coordinators have up-to-date data
        if self.network_coordinator.available_networks.len() != self.network.available_networks.len() {
            errors.push("Network coordinator has outdated available_networks list".to_string());
        }

        if self.account_coordinator.available_accounts.len() != self.wallet.available_accounts.len() {
            errors.push("Account coordinator has outdated available_accounts list".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Force synchronization of coordinators with current state (for emergency recovery)
    pub fn force_coordinator_sync(&mut self) {
        tracing::warn!("🔄 Force synchronizing coordinators with flattened state");
        self.sync_coordinators_with_flat_fields();
        tracing::info!("✅ Coordinator synchronization completed");
    }

    /// Validate and optionally fix state consistency issues
    pub fn validate_and_repair_state(&mut self) -> (bool, Vec<String>) {
        match self.validate_state_consistency() {
            Ok(()) => {
                tracing::debug!("✅ State validation passed");
                (true, vec!["State validation successful".to_string()])
            }
            Err(errors) => {
                tracing::warn!("⚠️ State validation failed with {} errors", errors.len());
                for error in &errors {
                    tracing::warn!("   - {}", error);
                }

                // Attempt automatic repair
                self.force_coordinator_sync();

                // Re-validate
                match self.validate_state_consistency() {
                    Ok(()) => {
                        tracing::info!("✅ State automatically repaired");
                        (
                            true,
                            vec![
                                format!("Found {} errors, but automatically repaired", errors.len()),
                                "State is now consistent".to_string(),
                            ],
                        )
                    }
                    Err(remaining_errors) => {
                        tracing::error!("❌ Could not automatically repair state");
                        (false, remaining_errors)
                    }
                }
            }
        }
    }

    /// Get the current state of the Cancel TX button for smart UI behavior
    pub fn get_cancel_button_state(&self) -> CancelButtonState {
        // Check if cancellation is currently in progress
        if self.transaction.cancellation_in_progress {
            return CancelButtonState::InProgress;
        }

        // Count cancellable pending transactions
        let cancellable_count = self
            .transaction
            .pending_transactions
            .iter()
            .filter(|tx| tx.cancellable)
            .count();

        // Check if there are any pending transactions that are no longer cancellable
        let has_confirmed = self.transaction.pending_transactions.iter().any(|tx| !tx.cancellable);

        match (cancellable_count, has_confirmed) {
            (0, false) => CancelButtonState::NoPending,
            (0, true) => CancelButtonState::TooLate,
            (count, _) => CancelButtonState::Cancellable(count),
        }
    }

    // Transaction state accessor methods
    pub fn send_to_address(&self) -> &String {
        &self.transaction.send_to_address
    }

    pub fn send_amount(&self) -> &String {
        &self.transaction.send_amount
    }

    pub fn send_selected_token(&self) -> &String {
        &self.transaction.send_selected_token
    }

    pub fn send_available_tokens(&self) -> &Vec<String> {
        &self.transaction.send_available_tokens
    }

    pub fn send_gas_limit(&self) -> &String {
        &self.transaction.send_gas_limit
    }

    pub fn send_gas_price(&self) -> &String {
        &self.transaction.send_gas_price
    }

    pub fn send_tx_type(&self) -> &String {
        &self.transaction.send_tx_type
    }

    pub fn send_max_fee_gwei(&self) -> &String {
        &self.transaction.send_max_fee_gwei
    }

    pub fn send_max_priority_fee_gwei(&self) -> &String {
        &self.transaction.send_max_priority_fee_gwei
    }

    pub fn send_nonce_override(&self) -> &String {
        &self.transaction.send_nonce_override
    }

    pub fn gas_speed(&self) -> &crate::gui::GasSpeed {
        &self.transaction.gas_speed
    }

    pub fn sending_transaction(&self) -> bool {
        self.transaction.sending_transaction
    }

    pub fn send_from_account_id(&self) -> &Option<String> {
        &self.transaction.send_from_account_id
    }

    // UI state accessor methods
    pub fn status_message_color(&self) -> &crate::gui::StatusMessageColor {
        &self.ui.status_message_color
    }

    pub fn balance_spinner(&self) -> bool {
        self.ui.balance_spinner
    }

    pub fn address_just_copied(&self) -> bool {
        self.wallet.address_just_copied
    }

    // Network state accessor methods for price info
    pub fn show_price_info(&self) -> bool {
        self.network.show_price_info
    }

    pub fn current_eth_price(&self) -> Option<f64> {
        self.network.eth_price
    }

    pub fn eth_price_loading(&self) -> bool {
        self.network.fetching_price
    }

    // Token state accessor methods
    pub fn balance_selected_ticker(&self) -> &String {
        &self.balance_selected_ticker
    }

    pub fn balance_available_tickers(&self) -> &Vec<String> {
        &self.balance_available_tickers
    }

    pub fn token_balances(&self) -> &Vec<crate::gui::SimpleTokenBalance> {
        &self.token_balances
    }
}
