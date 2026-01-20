//! UI state message handlers for WorkingWallet
//!
//! Handles UI state management messages including dialog visibility,
//! form updates, and interface transitions.

use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::Message;
use iced::Command;
use std::time::Instant;

impl WorkingWalletApp {
    /// Handle UI state-related messages
    pub fn handle_ui_state_message(&mut self, message: Message) -> Command<Message> {
        match message {
            // Dialog visibility management
            Message::ShowCreateDialog => self.handle_show_create_dialog(),
            Message::HideCreateDialog => self.handle_hide_create_dialog(),
            Message::ShowImportDialog => self.handle_show_import_dialog(),
            Message::HideImportDialog => self.handle_hide_import_dialog(),
            Message::ShowSettingsDialog => self.handle_show_settings_dialog(),
            Message::HideSettingsDialog => self.handle_hide_settings_dialog(),
            Message::ShowDappsDialog => self.handle_show_dapps_dialog(),
            Message::HideDappsDialog => self.handle_hide_dapps_dialog(),
            Message::ShowImportWallet => self.handle_show_import_wallet(),
            Message::HideImportWallet => self.handle_hide_import_wallet(),
            Message::ShowExportWallet => self.handle_show_export_wallet(),
            Message::HideExportWallet => self.handle_hide_export_wallet(),

            // Form field updates
            Message::SendToAddressChanged(address) => self.handle_send_to_address_changed(address),
            Message::SendAmountChanged(amount) => self.handle_send_amount_changed(amount),
            Message::SendGasLimitChanged(gas_limit) => self.handle_send_gas_limit_changed(gas_limit),
            Message::SendGasPriceChanged(gas_price) => self.handle_send_gas_price_changed(gas_price),
            Message::SendTokenChanged(token) => self.handle_send_token_changed(token),

            // Account form updates
            Message::CreateAccountNameChanged(name) => self.handle_create_account_name_changed(name),
            Message::ImportPrivateKeyChanged(key) => self.handle_import_private_key_changed(key),
            Message::ImportAccountNameChanged(name) => self.handle_import_account_name_changed(name),

            // Status updates
            Message::ClearStatusMessage => self.handle_clear_status_message(),
            Message::UpdateLastActivity => self.handle_update_last_activity(),
            Message::SetStatusMessage(message, color) => self.handle_set_status_message(message, color),
            Message::StatusMessageTick => self.handle_status_message_tick(),
            Message::SpinnerTick => self.handle_spinner_tick(),

            // Log management
            Message::ClearLogs => self.handle_clear_logs(),
            Message::ShowClearLogsConfirmation => self.handle_show_clear_logs_confirmation(),
            Message::HideClearLogsConfirmation => self.handle_hide_clear_logs_confirmation(),
            Message::ConfirmClearLogs => self.handle_confirm_clear_logs(),
            Message::CopyLogEntry(index) => self.handle_copy_log_entry(index),
            Message::LogEntryCopied(result) => self.handle_log_entry_copied(result),
            Message::ResetCopyFeedback => self.handle_reset_copy_feedback(),

            // Export account selection
            Message::ToggleAccountDropdown => self.handle_toggle_account_dropdown(),
            Message::SelectExportAccount(account_id) => self.handle_select_export_account(account_id),

            // Transaction cancellation dialogs
            Message::ShowCancelConfirmation => self.handle_show_cancel_confirmation(),
            Message::HideCancelConfirmation => self.handle_hide_cancel_confirmation(),

            _ => Command::none(),
        }
    }

    // Dialog visibility handlers
    fn handle_show_create_dialog(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_create_dialog = true;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_create_dialog(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_create_dialog = false;
        self.state.wallet_mut().create_account_name = String::new();
        Command::none()
    }

    fn handle_show_import_dialog(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_import_dialog = true;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_import_dialog(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_import_dialog = false;
        self.state.wallet_mut().import_private_key = String::new();
        self.state.wallet_mut().import_account_name = String::new();
        Command::none()
    }

    fn handle_show_settings_dialog(&mut self) -> Command<Message> {
        self.state.ui_mut().show_settings_dialog = true;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_settings_dialog(&mut self) -> Command<Message> {
        self.state.ui_mut().show_settings_dialog = false;
        Command::none()
    }

    fn handle_show_dapps_dialog(&mut self) -> Command<Message> {
        self.state.ui_mut().show_dapps_dialog = true;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_dapps_dialog(&mut self) -> Command<Message> {
        self.state.ui_mut().show_dapps_dialog = false;
        Command::none()
    }

    // Form field update handlers
    fn handle_send_to_address_changed(&mut self, address: String) -> Command<Message> {
        self.state.transaction_mut().send_to_address = address;
        self.state.ui_mut().last_activity = Instant::now();
        // Track activity for session management
        self.state.auth_mut().session.update_activity();
        Command::none()
    }

    fn handle_send_amount_changed(&mut self, amount: String) -> Command<Message> {
        self.state.transaction_mut().send_amount = amount;
        self.state.ui_mut().last_activity = Instant::now();
        // Track activity for session management
        self.state.auth_mut().session.update_activity();
        Command::none()
    }

    fn handle_send_gas_limit_changed(&mut self, gas_limit: String) -> Command<Message> {
        self.state.transaction_mut().send_gas_limit = gas_limit;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_send_gas_price_changed(&mut self, gas_price: String) -> Command<Message> {
        self.state.transaction_mut().send_gas_price = gas_price;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_send_token_changed(&mut self, token: String) -> Command<Message> {
        self.state.transaction_mut().send_selected_token = token;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    // Account form update handlers
    fn handle_create_account_name_changed(&mut self, name: String) -> Command<Message> {
        self.state.wallet_mut().create_account_name = name;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_import_private_key_changed(&mut self, key: String) -> Command<Message> {
        self.state.wallet_mut().import_private_key = key;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_import_account_name_changed(&mut self, name: String) -> Command<Message> {
        self.state.wallet_mut().import_account_name = name;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    // Status and activity handlers
    fn handle_clear_status_message(&mut self) -> Command<Message> {
        self.state.ui_mut().status_message = String::new();
        self.state.ui_mut().status_message_timer = None;
        Command::none()
    }

    fn handle_update_last_activity(&mut self) -> Command<Message> {
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    // Import/Export wallet dialog handlers
    fn handle_show_import_wallet(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_import_wallet = true;
        self.state.wallet_mut().wallet_name.clear();
        self.state.wallet_mut().seed_phrase.clear();
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_import_wallet(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_import_wallet = false;
        Command::none()
    }

    fn handle_show_export_wallet(&mut self) -> Command<Message> {
        tracing::info!("🔍 ShowExportWallet: Setting show_export_wallet = true");
        self.state.wallet_mut().show_export_wallet = true;

        // Complete state reset when window is reopened
        self.state.exported_seed_phrase = None;
        if let Some(ref key) = self.state.exported_private_key {
            if !key.is_empty() {
                self.state.exported_private_key = None;
            }
        }
        self.state.exporting_data = false;
        self.state.export_loading = false;

        // Clear any previous error messages and retry options
        self.state.wallet_mut().export_error_message = None;
        self.state.ui_mut().show_retry_options = false;

        // Clear export copy feedback and disable clipboard timer
        self.state.ui_mut().export_copy_feedback = None;
        self.state.ui_mut().clipboard_clear_timer_active = false;

        // Log for debugging
        tracing::info!(
            "Export wallet dialog opened - available accounts: {}",
            self.state.wallet().available_accounts.len()
        );

        // Filter accounts that are suitable for export
        let exportable_accounts: Vec<crate::security::SecureAccount> = self
            .state
            .wallet()
            .available_accounts
            .iter()
            .filter(|account| {
                // Filter out accounts that don't have valid key references
                !account.key_reference.id.is_empty()
            })
            .cloned()
            .collect();

        // Auto-select account based on available options
        if exportable_accounts.is_empty() {
            // No exportable accounts available
            self.state.selected_export_account_id = None;
            tracing::warn!("No exportable accounts found");
        } else if exportable_accounts.len() == 1 {
            // Auto-select the only available account
            self.state.selected_export_account_id = Some(exportable_accounts[0].id.clone());
            tracing::info!("Auto-selected single account: {}", exportable_accounts[0].name);
        } else {
            // Multiple accounts available - ALWAYS auto-select one to show export buttons
            if self.state.selected_export_account_id.is_some() {
                // Keep existing selection (set by export button handlers)
                tracing::info!("🎯 Keeping existing account selection for export");
            } else if let Some(current_id) = self.state.wallet().current_account_id.clone() {
                if exportable_accounts.iter().any(|acc| acc.id == current_id) {
                    // Current account is exportable - auto-select it
                    self.state.selected_export_account_id = Some(current_id.clone());
                    tracing::info!("🎯 Auto-selected current account for export: {}", current_id);
                } else {
                    // Current account not exportable - select first available
                    self.state.selected_export_account_id = Some(exportable_accounts[0].id.clone());
                    tracing::info!(
                        "🎯 Current account not exportable, selected first available: {}",
                        exportable_accounts[0].name
                    );
                }
            } else {
                // No current account - select first available
                self.state.selected_export_account_id = Some(exportable_accounts[0].id.clone());
                tracing::info!(
                    "🎯 No current account, selected first available: {}",
                    exportable_accounts[0].name
                );
            }
        }

        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_export_wallet(&mut self) -> Command<Message> {
        self.state.wallet_mut().show_export_wallet = false;

        // Secure cleanup of sensitive data
        if let Some(ref seed) = self.state.exported_seed_phrase {
            if !seed.is_empty() {
                // Clear the seed phrase
                self.state.exported_seed_phrase = None;
            }
        }

        if let Some(ref key) = self.state.exported_private_key {
            if !key.is_empty() {
                // Clear the private key
                self.state.exported_private_key = None;
            }
        }

        // Reset all export flow state to initial step for next opening
        self.state.selected_export_account_id = None;
        self.state.exporting_data = false;
        self.state.export_loading = false;

        // Clear export copy feedback and disable clipboard timer
        self.state.ui_mut().export_copy_feedback = None;
        self.state.ui_mut().clipboard_clear_timer_active = false;

        // Clear error messages and retry options
        self.state.wallet_mut().export_error_message = None;
        self.state.ui_mut().show_retry_options = false;

        // Log the secure cleanup for audit purposes
        tracing::info!("Export dialog closed - sensitive data securely cleared from memory, state reset");

        Command::none()
    }

    // Export account selection handlers
    fn handle_toggle_account_dropdown(&mut self) -> Command<Message> {
        self.state.ui_mut().show_account_dropdown = !self.state.ui().show_account_dropdown;
        Command::none()
    }

    fn handle_select_export_account(&mut self, account_id: String) -> Command<Message> {
        self.state.selected_export_account_id = Some(account_id);
        self.state.ui_mut().show_account_dropdown = false; // Close dropdown after selection
        Command::none()
    }

    // Transaction cancellation dialog handlers
    fn handle_show_cancel_confirmation(&mut self) -> Command<Message> {
        self.state.ui_mut().show_cancel_confirmation = true;
        self.state.ui_mut().last_activity = Instant::now();
        Command::none()
    }

    fn handle_hide_cancel_confirmation(&mut self) -> Command<Message> {
        self.state.ui_mut().show_cancel_confirmation = false;
        self.state.ui_mut().pending_cancel_tx = None;
        Command::none()
    }

    // Status message handlers
    fn handle_set_status_message(
        &mut self,
        message: String,
        color: crate::gui::StatusMessageColor,
    ) -> Command<Message> {
        self.state.ui_mut().status_message = message;
        self.state.ui_mut().status_message_color = color;
        self.state.ui_mut().status_message_timer = Some(Instant::now());
        Command::none()
    }

    fn handle_status_message_tick(&mut self) -> Command<Message> {
        if let Some(timer) = self.state.ui().status_message_timer {
            if timer.elapsed() > std::time::Duration::from_secs(5) {
                self.state.ui_mut().status_message.clear();
                self.state.ui_mut().status_message_color = crate::gui::StatusMessageColor::Default;
                self.state.ui_mut().status_message_timer = None;
            }
        }
        Command::none()
    }

    fn handle_spinner_tick(&mut self) -> Command<Message> {
        // Update spinner animations - just triggers redraw
        Command::none()
    }

    // Log management handlers
    fn handle_clear_logs(&mut self) -> Command<Message> {
        self.state.ui_mut().show_clear_logs_confirmation = true;
        Command::none()
    }

    fn handle_show_clear_logs_confirmation(&mut self) -> Command<Message> {
        self.state.ui_mut().show_clear_logs_confirmation = true;
        Command::none()
    }

    fn handle_hide_clear_logs_confirmation(&mut self) -> Command<Message> {
        self.state.ui_mut().show_clear_logs_confirmation = false;
        self.state.ui_mut().clearing_logs = false;
        Command::none()
    }

    fn handle_confirm_clear_logs(&mut self) -> Command<Message> {
        self.state.ui_mut().clearing_logs = true;
        self.state.log_entries.clear();

        self.add_log_entry(
            crate::gui::LogCategory::Success,
            "Session logs cleared".to_string(),
            Some(format!(
                "All session log entries have been cleared at {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            )),
        );

        self.state.ui_mut().show_clear_logs_confirmation = false;
        self.state.ui_mut().clearing_logs = false;
        self.state.transaction_mut().show_history = true;
        Command::none()
    }

    fn handle_copy_log_entry(&mut self, index: usize) -> Command<Message> {
        if let Some(entry) = self.state.log_entries.get(index) {
            let log_text = format!("[{}] {}", entry.timestamp, entry.message);
            Command::perform(
                async move {
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(log_text) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("Failed to copy to clipboard: {e}")),
                        },
                        Err(e) => Err(format!("Failed to access clipboard: {e}")),
                    }
                },
                Message::LogEntryCopied,
            )
        } else {
            Command::none()
        }
    }

    fn handle_log_entry_copied(&mut self, result: Result<(), String>) -> Command<Message> {
        match result {
            Ok(_) => {
                self.state.ui_mut().copy_feedback = Some("Copied to clipboard!".to_string());
                Command::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::ResetCopyFeedback,
                )
            }
            Err(error) => {
                self.state.ui_mut().copy_feedback = Some(format!("Copy failed: {error}"));
                Command::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    },
                    |_| Message::ResetCopyFeedback,
                )
            }
        }
    }

    fn handle_reset_copy_feedback(&mut self) -> Command<Message> {
        self.state.ui_mut().copy_feedback = None;
        Command::none()
    }
}
