//! Command Orchestration Helpers
//!
//! Phase 5 - Stage 2: Command pattern extraction for reduced complexity
//! This module provides reusable command patterns to simplify working_wallet.rs

use crate::gui::Message;
use crate::network::NetworkId;
use iced::Command;

/// Balance refresh command patterns
pub mod balance {
    use super::*;

    /// Create a standard balance refresh command
    pub fn refresh_balance() -> Command<Message> {
        Command::perform(async {}, |_| Message::RefreshBalance)
    }

    /// Create an internal balance refresh command (no loading UI)
    pub fn refresh_balance_internal() -> Command<Message> {
        Command::perform(async {}, |_| Message::InternalRefreshBalance)
    }

    /// Create a delayed balance refresh command
    pub fn refresh_balance_delayed(delay_ms: u64) -> Command<Message> {
        Command::perform(
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            },
            |_| Message::RefreshBalance,
        )
    }

    /// Create a balance refresh command with account update
    pub fn refresh_with_account_update() -> Command<Message> {
        Command::batch([
            refresh_balance(),
            Command::perform(async {}, |_| Message::UpdateAccountBalance),
        ])
    }
}

/// Account management command patterns
pub mod account {
    use super::*;

    /// Create a complete account switch sequence
    pub fn switch_account(account_id: String) -> Command<Message> {
        Command::batch([
            Command::perform(async move { account_id }, Message::AccountSelected),
            balance::refresh_balance(),
            Command::perform(async {}, |_| Message::RefreshTransactionHistory),
        ])
    }

    /// Create account creation sequence
    pub fn create_account_sequence() -> Command<Message> {
        Command::perform(async {}, |_| Message::CreateAccount)
    }

    /// Create account import sequence
    pub fn import_account_sequence() -> Command<Message> {
        Command::perform(async {}, |_| Message::ImportAccount)
    }
}

/// Network management command patterns
pub mod network {
    use super::*;

    /// Create a complete network switch sequence
    pub fn switch_network(network_id: NetworkId) -> Command<Message> {
        Command::batch([
            Command::perform(async move { network_id }, Message::NetworkSelected),
            balance::refresh_balance_delayed(500), // Small delay for network switch
            Command::perform(async {}, |_| Message::RefreshTransactionHistory),
        ])
    }

    /// Create network addition sequence
    pub fn add_network_sequence() -> Command<Message> {
        Command::batch([Command::perform(async {}, |_| Message::ShowAddNetwork)])
    }
}

/// Transaction command patterns
pub mod transaction {
    use super::*;

    /// Create transaction history refresh
    pub fn refresh_history() -> Command<Message> {
        Command::perform(async {}, |_| Message::RefreshTransactionHistory)
    }

    /// Create gas estimation command
    pub fn estimate_gas() -> Command<Message> {
        Command::perform(async {}, |_| Message::EstimateGas)
    }

    /// Create complete transaction submission sequence
    pub fn submit_transaction_sequence() -> Command<Message> {
        Command::batch([
            Command::perform(async {}, |_| Message::ShowTransactionConfirmation),
            // Additional validation and preparation commands can be added here
        ])
    }
}

/// UI state management command patterns
pub mod ui {
    use super::*;

    /// Create status message clear command with delay
    pub fn clear_status_message_delayed(delay_ms: u64) -> Command<Message> {
        Command::perform(
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            },
            |_| Message::ClearStatusMessage,
        )
    }

    /// Create user activity update command
    pub fn update_activity() -> Command<Message> {
        Command::perform(async {}, |_| Message::UpdateLastActivity)
    }

    /// Create copy feedback sequence with auto-clear
    pub fn copy_feedback_sequence() -> Command<Message> {
        Command::batch([
            Command::perform(async {}, |_| Message::ResetCopyFeedback),
            clear_status_message_delayed(3000), // Clear after 3 seconds
        ])
    }
}

/// Composite command patterns for common workflows
pub mod workflows {
    use super::*;

    /// Complete wallet initialization workflow
    pub fn initialize_wallet_workflow() -> Command<Message> {
        Command::batch([
            Command::perform(async {}, |_| Message::LoadAccounts),
            Command::perform(async {}, |_| Message::LoadNetworks),
        ])
    }

    /// Complete account context refresh (network + account + balance)
    pub fn refresh_user_context() -> Command<Message> {
        Command::batch([
            balance::refresh_balance(),
            transaction::refresh_history(),
            ui::update_activity(),
        ])
    }

    /// Error recovery workflow
    pub fn error_recovery_workflow() -> Command<Message> {
        Command::batch([
            balance::refresh_balance_delayed(1000),
            ui::clear_status_message_delayed(5000),
        ])
    }
}
