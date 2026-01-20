//! Network message handlers for WorkingWallet
//!
//! Handles network-related messages including network switching,
//! provider management, and connectivity operations.

use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::{LogCategory, Message};
use crate::network::NetworkId;
use iced::Command;
use std::time::Duration;

impl WorkingWalletApp {
    /// Handle network-related messages
    pub fn handle_network_message(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NetworkSelected(network_id) => self.handle_network_selected(network_id),
            Message::SmartPollTick => self.handle_smart_poll_tick(),
            Message::BalanceChanged(old_balance, new_balance) => self.handle_balance_changed(old_balance, new_balance),
            _ => Command::none(),
        }
    }

    /// Handle network selection
    fn handle_network_selected(&mut self, network_id: NetworkId) -> Command<Message> {
        self.state.network_mut().current_network = network_id;
        // Reset last balance when network changes
        // Reset last balance when network changes - handled by coordinator

        // Update token list for the selected network
        self.update_token_list_for_network(network_id);

        // Switch the wallet's network manager to the selected network
        if let Some(wallet) = &self.wallet {
            let wallet_clone = wallet.clone();
            let network_id_for_log = network_id.0; // Capture the value for the closure
            return Command::perform(
                async move {
                    let mut wallet = wallet_clone.write().await;
                    wallet.switch_network(network_id, None).await
                },
                move |result| {
                    match result {
                        Ok(_) => {
                            tracing::info!("✅ Successfully switched wallet to network {}", network_id_for_log);
                            Message::RefreshBalance
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to switch wallet network: {}", e);
                            Message::RefreshBalance // Still try to refresh balance
                        }
                    }
                },
            );
        }

        self.dispatch_message(Message::RefreshBalance)
    }

    /// Handle smart polling tick for periodic updates
    fn handle_smart_poll_tick(&mut self) -> Command<Message> {
        // Only poll if we have an account and wallet is not loading
        if !self.state.is_loading && self.state.wallet().current_account_id.is_some() && self.wallet.is_some() {
            // Update poll interval based on activity
            let time_since_activity = self.state.ui().last_activity.elapsed();

            if time_since_activity > Duration::from_secs(1800) {
                // 30 minutes - Stop polling after 30 minutes of inactivity
                self.state.ui_mut().polling_active = false;
                Command::none()
            } else if time_since_activity > Duration::from_secs(300) {
                // 5 minutes - Slow polling after 5 minutes
                self.state.ui_mut().poll_interval = 60;

                // Check for both balance updates and incoming transactions
                let balance_cmd = self.dispatch_message(Message::InternalRefreshBalance);
                let tx_check_cmd = self.check_for_incoming_transactions_periodically();

                Command::batch(vec![balance_cmd, tx_check_cmd])
            } else {
                // Active polling for first 5 minutes
                self.state.ui_mut().poll_interval = 10;

                // Check for both balance updates and incoming transactions more frequently
                let balance_cmd = self.dispatch_message(Message::InternalRefreshBalance);
                let tx_check_cmd = self.check_for_incoming_transactions_periodically();

                Command::batch(vec![balance_cmd, tx_check_cmd])
            }
        } else {
            Command::none()
        }
    }

    /// Handle balance change notifications
    fn handle_balance_changed(&mut self, old_balance: String, new_balance: String) -> Command<Message> {
        use crate::gui::utils::parse_balance;

        // Check if this is a balance change after an account switch (not incoming coins)
        if self.state.account_just_switched {
            tracing::debug!("⏩ Skipping audio alert - account was just switched");
            self.state.account_just_switched = false; // Clear the flag
            return Command::none();
        }

        // Enhanced balance change notification with incoming transaction detection
        if let (Ok(old_val), Ok(new_val)) = (parse_balance(&old_balance), parse_balance(&new_balance)) {
            if new_val > old_val {
                // Check if this is a legitimate incoming transaction vs initial load
                if !Self::is_legitimate_balance_increase(&old_balance, &new_balance, old_val, new_val) {
                    tracing::debug!("⏩ Skipping audio alert - appears to be initial balance load");
                    return Command::none();
                }
                // Determine the token type from the balance string
                let token_name = if new_balance.contains("ETH") {
                    "ETH"
                } else if new_balance.contains("PLS") || new_balance.contains("tPLS") {
                    "tPLS"
                } else if new_balance.contains("BNB") {
                    "BNB"
                } else if new_balance.contains("MATIC") {
                    "MATIC"
                } else {
                    // Default for unknown tokens
                    "tokens"
                };

                // Simple sound notification for incoming coins
                tracing::error!("🔥🔥🔥 AUDIO: Playing notification sound for balance increase!");
                if let Err(e) = crate::gui::utils::play_notification_sound() {
                    tracing::error!("❌ Failed to play notification sound: {}", e);
                } else {
                    tracing::error!("✅ Audio notification played successfully!");
                }

                // Log the balance change for debugging
                tracing::info!(
                    "💰 Balance increased: {} → {} - checking for incoming transactions",
                    old_balance,
                    new_balance
                );

                // Add to wallet logs
                self.add_log_entry(
                    LogCategory::Info,
                    format!("Balance increased: {token_name}"),
                    Some(format!("Old: {old_balance} → New: {new_balance}")),
                );

                // Check for incoming transactions when balance increases
                self.check_for_incoming_transactions()
            } else if new_val < old_val {
                // Balance decreased - likely outgoing transaction
                tracing::info!("📤 Balance decreased: {} → {}", old_balance, new_balance);
                Command::none()
            } else {
                // No significant change
                Command::none()
            }
        } else {
            // Could not parse balance values
            tracing::warn!("⚠️ Could not parse balance change: {} → {}", old_balance, new_balance);
            Command::none()
        }
    }

    /// Check if a balance increase is legitimate (real incoming transaction) vs initial load
    pub fn is_legitimate_balance_increase(old_balance: &str, new_balance: &str, old_val: f64, new_val: f64) -> bool {
        // Skip audio alerts for these cases (likely initial loads):

        // 1. If old balance looks like default/zero values
        if old_balance == "0.0000" || old_balance == "0" || old_balance.is_empty() {
            tracing::debug!(
                "🔍 Skipping - old balance appears to be default value: '{}'",
                old_balance
            );
            return false;
        }

        // 2. If the old balance contains default placeholder text
        if old_balance.contains("Loading") || old_balance.contains("...") || old_balance.contains("--") {
            tracing::debug!(
                "🔍 Skipping - old balance appears to be loading placeholder: '{}'",
                old_balance
            );
            return false;
        }

        // 3. If this is a very large increase (likely initial load, not a typical transaction)
        if old_val == 0.0 && new_val > 0.0 {
            tracing::debug!(
                "🔍 Skipping - balance change from zero to {} (likely initial load)",
                new_val
            );
            return false;
        }

        // 4. If the increase is suspiciously large (> 1000 units), might be initial load
        let increase_ratio = if old_val > 0.0 {
            new_val / old_val
        } else {
            f64::INFINITY
        };
        if increase_ratio > 1000.0 {
            tracing::debug!(
                "🔍 Skipping - very large increase ratio ({:.2}x) suggests initial load",
                increase_ratio
            );
            return false;
        }

        // If we get here, this looks like a legitimate incoming transaction
        tracing::info!(
            "🔔 Legitimate balance increase detected: {} → {} ({:.6} increase)",
            old_balance,
            new_balance,
            new_val - old_val
        );
        true
    }
}
