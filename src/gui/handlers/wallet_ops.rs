//! Wallet operation message handlers for WorkingWallet
//!
//! Handles wallet operation messages including account management,
//! balance updates, and core wallet functionality.

use crate::gui::utils::format_balance;
use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::{LogCategory, Message, StatusMessageColor, Transaction};

use iced::Command;
use std::time::Instant;
// Note: account creation functions need to be implemented

impl WorkingWalletApp {
    /// Handle wallet operation messages
    pub fn handle_wallet_ops_message(&mut self, message: Message) -> Command<Message> {
        match message {
            // Account management
            Message::CreateAccount => self.handle_create_account(),
            Message::AccountCreated(result) => self.handle_account_created(result),
            Message::ImportAccount => self.handle_import_account(),
            Message::AccountImported(result) => self.handle_account_imported(result),
            Message::AccountSelected(account_id) => self.handle_account_selected(account_id),
            Message::DeleteAccount(account_id) => self.handle_delete_account(account_id),

            // Balance and refresh operations
            Message::RefreshBalance => self.handle_refresh_balance(),
            Message::InternalRefreshBalance => self.handle_internal_refresh_balance(),
            Message::BalanceRefreshed(result) => self.handle_balance_refreshed(result),
            Message::TokenBalancesRefreshed(token_balances) => self.handle_token_balances_refreshed(token_balances),
            Message::UpdateAccountBalance => self.handle_update_account_balance(),

            // Transaction history
            Message::RefreshTransactionHistory => self.handle_refresh_transaction_history(),
            Message::TransactionHistoryRefreshed(result) => self.handle_transaction_history_refreshed(result),

            // Legacy balance fetched (for compatibility)
            Message::BalanceFetched(result) => self.handle_balance_refreshed(result),

            _ => Command::none(),
        }
    }

    /// Handle account creation - Following DEVELOPMENT_RULES.md using Alloy
    fn handle_create_account(&mut self) -> Command<Message> {
        if self.state.wallet().create_account_name.trim().is_empty() {
            self.state.ui_mut().status_message = "Please enter an account name".to_string();
            self.state.ui_mut().status_message_color = StatusMessageColor::Error;
            self.state.ui_mut().status_message_timer = Some(Instant::now());
            return Command::none();
        }

        // Get cached master password from wallet session (MetaMask pattern)
        // Wallet must be unlocked before creating accounts
        let master_password_str = match self
            .state
            .auth()
            .enhanced_session
            .wallet_session
            .cached_master_password
            .as_ref()
        {
            Some(password) => {
                use secrecy::ExposeSecret;
                let p: String = password.expose_secret().clone();
                p
            }
            None => {
                // Wallet not unlocked - show error
                tracing::error!("❌ Cannot create account: wallet not unlocked");
                self.state.ui_mut().status_message = "Please unlock wallet first".to_string();
                self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                self.state.ui_mut().status_message_timer = Some(Instant::now());
                return Command::none();
            }
        };

        self.state.wallet_mut().creating_account = true;
        let account_name = self.state.wallet().create_account_name.clone();
 
        tracing::info!("Creating new account: {}", account_name);
 
        // Use the standardized account service for creation
        Command::perform(
            async move {
                use crate::security::SeedStrength;
                
                // Generate a new 12-word seed phrase
                let seed_phrase = crate::gui::services::account_service::generate_seed_phrase_with_strength(SeedStrength::Words12).await;
                
                crate::gui::services::account_service::create_wallet_from_seed(
                    account_name,
                    seed_phrase,
                    master_password_str,
                )
                .await
            },
            Message::AccountCreated,
        )
    }

    /// Handle account creation result
    fn handle_account_created(&mut self, result: Result<String, String>) -> Command<Message> {
        self.state.wallet_mut().creating_account = false;
        match result {
            Ok(account_id) => {
                tracing::info!("✅ Account created successfully: {}", account_id);
                self.state.wallet_mut().show_create_dialog = false;
                self.state.wallet_mut().create_account_name = String::new();

                self.add_log_entry(
                    LogCategory::Wallet,
                    "Account created successfully".to_string(),
                    Some(format!("New account: {account_id}")),
                );

                self.state.ui_mut().status_message = "Account created successfully".to_string();
                self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                self.state.ui_mut().status_message_timer = Some(Instant::now());

                // Refresh accounts list and select the new account
                self.state.wallet_mut().current_account_id = Some(account_id.clone());
                // Also set send_from_account_id so send button works
                self.state.transaction_mut().send_from_account_id = Some(account_id);
                self.refresh_accounts_list()
            }
            Err(error) => {
                tracing::error!("❌ Account creation failed: {}", error);
                self.add_log_entry(
                    LogCategory::Error,
                    "Account creation failed".to_string(),
                    Some(error.clone()),
                );

                self.state.ui_mut().status_message = format!("Failed to create account: {error}");
                self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                self.state.ui_mut().status_message_timer = Some(Instant::now());

                Command::none()
            }
        }
    }

    /// Handle account import
    fn handle_import_account(&mut self) -> Command<Message> {
        if self.state.wallet().import_private_key.trim().is_empty() {
            self.state.ui_mut().status_message = "Please enter a private key".to_string();
            self.state.ui_mut().status_message_color = StatusMessageColor::Error;
            self.state.ui_mut().status_message_timer = Some(Instant::now());
            return Command::none();
        }

        if self.state.wallet().import_account_name.trim().is_empty() {
            self.state.ui_mut().status_message = "Please enter an account name".to_string();
            self.state.ui_mut().status_message_color = StatusMessageColor::Error;
            self.state.ui_mut().status_message_timer = Some(Instant::now());
            return Command::none();
        }

        self.state.wallet_mut().importing_account = true;
        let private_key = self.state.wallet().import_private_key.clone();
        let account_name = self.state.wallet().import_account_name.clone();
 
        tracing::info!("Importing account: {}", account_name);
 
        // Use the standardized account service for import
        Command::perform(
            async move {
                crate::gui::services::account_service::import_wallet_from_private_key(
                    account_name,
                    private_key,
                    String::new(), // Password not used for private key imports in this flow
                )
                .await
            },
            Message::AccountImported,
        )
    }

    /// Handle account import result
    fn handle_account_imported(&mut self, result: Result<String, String>) -> Command<Message> {
        self.state.wallet_mut().importing_account = false;
        match result {
            Ok(account_id) => {
                tracing::info!("✅ Account imported successfully: {}", account_id);
                self.state.wallet_mut().show_import_dialog = false;
                self.state.wallet_mut().import_private_key = String::new();
                self.state.wallet_mut().import_account_name = String::new();

                self.add_log_entry(
                    LogCategory::Wallet,
                    "Account imported successfully".to_string(),
                    Some(format!("Imported account: {account_id}")),
                );

                self.state.ui_mut().status_message = "Account imported successfully".to_string();
                self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                self.state.ui_mut().status_message_timer = Some(Instant::now());

                // Refresh accounts list and select the imported account
                self.state.wallet_mut().current_account_id = Some(account_id.clone());
                // Also set send_from_account_id so send button works
                self.state.transaction_mut().send_from_account_id = Some(account_id);
                self.refresh_accounts_list()
            }
            Err(error) => {
                tracing::error!("❌ Account import failed: {}", error);
                self.add_log_entry(
                    LogCategory::Error,
                    "Account import failed".to_string(),
                    Some(error.clone()),
                );

                self.state.ui_mut().status_message = format!("Failed to import account: {error}");
                self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                self.state.ui_mut().status_message_timer = Some(Instant::now());

                Command::none()
            }
        }
    }

    /// Handle account selection
    fn handle_account_selected(&mut self, account_id: String) -> Command<Message> {
        self.state.wallet_mut().current_account_id = Some(account_id.clone());
        // Also set the send_from_account_id so the send button becomes enabled
        self.state.transaction_mut().send_from_account_id = Some(account_id.clone());
        self.state.last_activity = Instant::now();

        // Clear cached balances when switching accounts
        self.state.token_balances.clear();
        self.state.account_balance = "0.0000".to_string();

        // Set flag to skip audio on next balance change (this is a switch, not incoming coins)
        self.state.account_just_switched = true;

        // Clear any password errors when switching accounts
        self.state.auth_mut().password_dialog.error = None;

        // Track activity for session management
        self.state.auth_mut().session.update_activity();

        // Check account type and adjust authentication requirements (Requirement 5.5)
        if let Some(account) = self
            .state
            .wallet()
            .available_accounts
            .iter()
            .find(|a| a.id == account_id)
        {
            use crate::gui::services::account_service::get_account_type;
            use crate::gui::wallet_types::AccountType;

            let account_type = get_account_type(account);

            match account_type {
                AccountType::SeedBased => {
                    // Switching to seed-based account - session state remains as is
                    // If session is locked, user will be prompted for password on next transaction
                    tracing::info!(
                        "Switched to seed-based account: {} (session: {})",
                        account_id,
                        if self.state.auth().session.is_unlocked {
                            "unlocked"
                        } else {
                            "locked"
                        }
                    );
                }
                AccountType::PrivateKey => {
                    // Switching to private-key account - clear cached keys for security
                    // Private-key accounts don't need password authentication
                    if self.state.auth().session.cached_password.is_some() {
                        tracing::info!(
                            "Switching from seed-based to private-key account: {} - clearing cached keys",
                            account_id
                        );
                        self.state.auth_mut().session.cached_password = None;
                    } else {
                        tracing::info!("Switched to private-key account: {}", account_id);
                    }
                }
            }
        }

        tracing::info!(
            "Account selected: {} (also set as send_from account, balances cleared)",
            account_id
        );

        // Find the selected account and switch to it in the wallet
        if let Some(account) = self
            .state
            .wallet()
            .available_accounts
            .iter()
            .find(|a| a.id == account_id)
        {
            let account_address = account.address;
            let wallet = self.wallet.clone();

            // Switch the wallet to use this account
            let switch_cmd = Command::perform(
                async move {
                    if let Some(wallet_arc) = wallet {
                        // First, ensure the keystore has accounts and debug what's available
                        {
                            let wallet_read = wallet_arc.read().await;
                            let keystore_arc = wallet_read.keystore();
                            let mut keystore = keystore_arc.write().await;
                            if let Err(e) = keystore.ensure_unlocked().await {
                                tracing::error!("Failed to ensure keystore unlocked: {}", e);
                                return Err(format!("Failed to unlock keystore: {e}"));
                            }

                            let account_count = keystore.list_accounts().await.unwrap_or_default().len();
                            let accounts = keystore.list_accounts().await.unwrap_or_default();
                            tracing::info!(
                                "✅ Keystore has {} accounts. Addresses: {:?}",
                                account_count,
                                accounts.iter().map(|a| format!("{:#x}", a.address)).collect::<Vec<_>>()
                            );
                            tracing::info!("Looking for account address: {:#x}", account_address);
                        }

                        // Now try to switch to the account
                        let mut wallet_write = wallet_arc.write().await;
                        if let Err(e) = wallet_write.switch_account(account_address).await {
                            tracing::error!("Failed to switch wallet account: {}", e);
                            return Err(format!("Failed to switch account: {e}"));
                        }
                        tracing::info!("✅ Wallet switched to account: {}", account_address);
                    }
                    Ok(())
                },
                |result: Result<(), String>| {
                    if let Err(e) = result {
                        Message::SetStatusMessage(e, StatusMessageColor::Error)
                    } else {
                        Message::RefreshBalance
                    }
                },
            );

            switch_cmd
        } else {
            // Account not found, just refresh balance
            self.dispatch_message(Message::RefreshBalance)
        }
    }

    /// Handle account deletion
    fn handle_delete_account(&mut self, account_id: String) -> Command<Message> {
        // Implementation would go here for account deletion
        tracing::info!("Delete account requested: {}", account_id);

        self.add_log_entry(
            LogCategory::Wallet,
            "Account deletion requested".to_string(),
            Some(format!("Account: {account_id}")),
        );

        // For now, just return none - full implementation would handle actual deletion
        Command::none()
    }

    /// Handle balance refresh
    fn handle_refresh_balance(&mut self) -> Command<Message> {
        // Track activity for session management
        self.state.auth_mut().session.update_activity();

        tracing::info!(
            "🔄 RefreshBalance requested - wallet: {}, account_id: {:?}",
            self.wallet.is_some(),
            self.state.wallet().current_account_id.as_deref().unwrap_or("None")
        );

        if let (Some(wallet), Some(account_id)) = (&self.wallet, &self.state.wallet().current_account_id) {
            tracing::info!("🔍 Looking for account address for ID: {}", account_id);

            // Use proper accessor method instead of deprecated field
            let available_accounts = &self.state.wallet().available_accounts;
            
            // Find the account to get the SecureAccount object
            let account = if let Some(acc) = available_accounts.iter().find(|a| &a.id == account_id) {
                tracing::info!("✅ Found account address: {} for ID: {}", acc.address, account_id);
                acc.clone()
            } else {
                tracing::error!("❌ Could not find address for account ID: {}", account_id);
                return Command::none();
            };

            self.state.is_loading = true;
            let wallet_clone = wallet.clone();
            let network_id = self.state.network().current_network;
            let account_service = self.account_service.clone();

            tracing::info!(
                "🚀 Starting async wallet switch operation for account: {}",
                account.address
            );

            Command::perform(
                async move {
                    tracing::info!("🔒 Attempting to acquire wallet write lock for account switch");
                    // First, switch to the selected account in the wallet
                    let mut wallet_write = wallet_clone.write().await;
                    if let Err(e) = wallet_write.switch_account(account.address).await {
                        tracing::error!("Failed to switch to account {}: {}", account.address, e);
                        return Err(format!("Failed to switch to account: {e}"));
                    }
                    drop(wallet_write); // Release write lock

                    // Use IntegratedAccountService to get balance (utilizing cache and telemetry)
                    // Define fetch function that uses the network manager directly (bypassing current account check)
                    let wallet_for_fetch = wallet_clone.clone();
                    let fetch_fn = move |address: alloy::primitives::Address| {
                        let inner_wallet = wallet_for_fetch.clone();
                        async move {
                            let wallet_read = inner_wallet.read().await;
                            let network_manager = wallet_read.network_manager();
                            let nm_read = network_manager.read().await;
                            
                            // NetworkManager::get_balance returns Result<U256, VaughanError> (if implicit into)
                            // or Result<U256, NetworkError>. 
                            // If it's Result<U256, VaughanError>, we don't need map_err.
                            nm_read.get_balance(address, None).await
                        }
                    };

                    match account_service.get_account_balance(&account, fetch_fn).await {
                        Ok(balance_wei) => {
                            tracing::info!(
                                "✅ Retrieved balance for account {}: {} wei",
                                account.address,
                                balance_wei
                            );
                            // Format the balance from wei to human-readable format with currency symbol
                            let formatted_balance = format_balance(balance_wei, network_id);
                            Ok(formatted_balance)
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to get balance for account {}: {}", account.address, e);
                            Err(format!("Failed to get balance: {e}"))
                        }
                    }
                },
                Message::BalanceRefreshed,
            )
        } else {
            tracing::warn!("⚠️ Cannot refresh balance: wallet or account_id missing");
            Command::none()
        }
    }

    /// Handle internal balance refresh (for polling)
    fn handle_internal_refresh_balance(&mut self) -> Command<Message> {
        // Track activity for session management
        self.state.auth_mut().session.update_activity();

        if let (Some(wallet), Some(account_id)) = (&self.wallet, &self.state.wallet().current_account_id) {
            let available_accounts = &self.state.wallet().available_accounts;
            let account_address = if let Some(account) = available_accounts.iter().find(|a| &a.id == account_id) {
                account.address
            } else {
                return Command::none();
            };

            // DON'T set self.state.is_loading = true for background refresh
            let wallet_clone = wallet.clone();
            let network_id = self.state.network().current_network;

            Command::perform(
                async move {
                    let mut wallet_write = wallet_clone.write().await;
                    if let Err(e) = wallet_write.switch_account(account_address).await {
                        return Err(format!("Failed to switch to account: {e}"));
                    }
                    drop(wallet_write);

                    let wallet_read = wallet_clone.read().await;
                    match wallet_read.get_balance(None).await {
                        Ok(balance_wei) => {
                            let formatted_balance = format_balance(balance_wei, network_id);
                            Ok(formatted_balance)
                        }
                        Err(e) => Err(format!("Failed to get balance: {e}")),
                    }
                },
                Message::BalanceRefreshed,
            )
        } else {
            Command::none()
        }
    }

    /// Handle balance refresh result
    fn handle_balance_refreshed(&mut self, result: Result<String, String>) -> Command<Message> {
        // Only reset loading state for user-initiated refresh
        if self.state.is_loading {
            self.state.is_loading = false;
        }
        match result {
            Ok(balance) => {
                let old_balance = self.state.network().balance.clone();
                tracing::info!(
                    "💰 Balance refresh successful: {} -> {} (wallet loading: {})",
                    if old_balance.is_empty() { "Empty" } else { &old_balance },
                    balance,
                    self.state.is_loading
                );
                self.state.network_mut().balance = balance.clone();
                self.state.account_balance = balance.clone(); // Keep both in sync

                // Update token_balances map for all tokens, not just the selected one
                let numeric_balance = balance.split_whitespace().next().unwrap_or("0.0000").to_string();

                // Update the native token balance first
                let native_ticker = self
                    .state
                    .network()
                    .available_networks
                    .iter()
                    .find(|n| n.id == self.state.network().current_network)
                    .map(|n| n.symbol.clone())
                    .unwrap_or_else(|| "ETH".to_string());

                if let Some(token) = self.state.token_balances.iter_mut().find(|t| t.symbol == native_ticker) {
                    token.balance = numeric_balance.clone();
                } else {
                    self.state.token_balances.push(crate::gui::SimpleTokenBalance {
                        symbol: native_ticker,
                        name: "Native Token".to_string(),
                        contract_address: None,
                        balance: numeric_balance,
                        decimals: 18,
                    });
                }

                // Check if balance changed for notifications BEFORE any early returns
                let notification_command = if !old_balance.is_empty() && old_balance != balance {
                    tracing::error!(
                        "🔥🔥🔥 NOTIFICATION: Sending BalanceChanged message: '{}' → '{}'",
                        old_balance,
                        balance
                    );
                    self.dispatch_message(Message::BalanceChanged(old_balance.clone(), balance.clone()))
                } else {
                    tracing::error!("🔥🔥🔥 NO_NOTIFICATION: No balance change notification to send");
                    Command::none()
                };

                // Now fetch balances for all other tokens that have contract addresses
                if let (Some(wallet), Some(account_id)) = (&self.wallet, &self.state.wallet().current_account_id) {
                    let wallet_clone = wallet.clone();
                    let _account_id_clone = account_id.clone();
                    let tokens_to_update = self.state.token_balances.clone();

                    let token_update_command = Command::perform(
                        async move {
                            let mut results = Vec::new();
                            let wallet_read = wallet_clone.read().await;

                            for token in &tokens_to_update {
                                if let Some(contract_address) = token.contract_address {
                                    match wallet_read.get_balance(Some(contract_address)).await {
                                        Ok(balance_wei) => {
                                            // Convert using Alloy's format_units handles decimals correctly
                                            let formatted_balance =
                                                alloy::primitives::utils::format_units(balance_wei, token.decimals)
                                                    .unwrap_or_else(|_| "0.0000".to_string());

                                            // Truncate to avoid excessive decimals in UI (limit to 6)
                                            let final_balance = if let Some(idx) = formatted_balance.find('.') {
                                                if idx + 1 + 6 < formatted_balance.len() {
                                                    formatted_balance[..idx + 1 + 6].to_string()
                                                } else {
                                                    formatted_balance
                                                }
                                            } else {
                                                formatted_balance
                                            };
                                            results.push((token.symbol.clone(), final_balance));
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to fetch balance for token {}: {}", token.symbol, e);
                                            results.push((token.symbol.clone(), "0.0000".to_string()));
                                        }
                                    }
                                }
                            }
                            results
                        },
                        Message::TokenBalancesRefreshed,
                    );

                    return Command::batch(vec![notification_command, token_update_command]);
                }

                // If no tokens to update, just return the notification command
                notification_command
            }
            Err(error) => {
                tracing::error!("❌ Failed to refresh balance: {}", error);
                self.state.network_mut().balance = "Error loading balance".to_string();
                self.state.account_balance = "Error loading balance".to_string(); // Keep both in sync
                                                                                  // Update token_balances map for native token with error state
                                                                                  // Extract just the symbol from the selected ticker (handle both "SYMBOL" and "SYMBOL (ADDRESS)" formats)
                let selected_symbol = if self.state.balance_selected_ticker.contains('(') {
                    self.state
                        .balance_selected_ticker
                        .split('(')
                        .next()
                        .unwrap_or(&self.state.balance_selected_ticker)
                        .trim()
                        .to_string()
                } else {
                    self.state.balance_selected_ticker.clone()
                };

                if let Some(token) = self
                    .state
                    .token_balances
                    .iter_mut()
                    .find(|t| t.symbol == selected_symbol)
                {
                    token.balance = "Error".to_string();
                } else {
                    self.state.token_balances.push(crate::gui::SimpleTokenBalance {
                        symbol: selected_symbol.clone(),
                        name: selected_symbol.clone(),
                        contract_address: None,
                        balance: "Error".to_string(),
                        decimals: 18,
                    });
                }
                Command::none()
            }
        }
    }

    /// Handle account balance update
    fn handle_update_account_balance(&mut self) -> Command<Message> {
        // Trigger a balance refresh
        self.dispatch_message(Message::RefreshBalance)
    }

    /// Handle transaction history refresh
    fn handle_refresh_transaction_history(&mut self) -> Command<Message> {
        if let (Some(wallet), Some(account_id)) = (&self.wallet, &self.state.wallet().current_account_id) {
            let _wallet_clone = wallet.clone();
            let account_id_clone = account_id.clone();
            let _network_id = self.state.network().current_network;

            Command::perform(
                async move {
                    // Placeholder transaction history loading
                    tracing::info!("Loading transaction history for account: {}", account_id_clone);
                    // In a full implementation, this would call the wallet's transaction history method
                    Ok(vec![])
                },
                Message::TransactionHistoryRefreshed,
            )
        } else {
            Command::none()
        }
    }

    /// Handle transaction history refresh result
    fn handle_transaction_history_refreshed(&mut self, result: Result<Vec<Transaction>, String>) -> Command<Message> {
        match result {
            Ok(transactions) => {
                self.state.transaction_mut().transaction_history = transactions;
                tracing::info!(
                    "✅ Transaction history refreshed: {} transactions",
                    self.state.transaction().transaction_history.len()
                );
                Command::none()
            }
            Err(error) => {
                tracing::error!("❌ Failed to refresh transaction history: {}", error);
                Command::none()
            }
        }
    }

    /// Handle token balances refresh result
    fn handle_token_balances_refreshed(&mut self, token_balances: Vec<(String, String)>) -> Command<Message> {
        tracing::info!("✅ Received updated token balances: {:?}", token_balances);

        // Track notifications to send after the borrow ends
        let mut notifications = Vec::new();

        // Update all token balances in the state
        for (token_symbol, balance) in token_balances {
            if let Some(token) = self.state.token_balances.iter_mut().find(|t| t.symbol == token_symbol) {
                // Check if balance increased for audio alert
                let old_balance = token.balance.clone();
                let new_balance = balance.clone();

                // Only play sound if balance actually changed and increased
                if old_balance != new_balance {
                    use crate::gui::utils::parse_balance;

                    if let (Ok(old_val), Ok(new_val)) = (parse_balance(&old_balance), parse_balance(&new_balance)) {
                        if new_val > old_val {
                            // Check if this is a legitimate incoming transaction vs initial load
                            // Reuse the logic from network handler
                            if WorkingWalletApp::is_legitimate_balance_increase(
                                &old_balance,
                                &new_balance,
                                old_val,
                                new_val,
                            ) {
                                notifications.push((token_symbol.clone(), old_balance, new_balance));
                            }
                        }
                    }
                }

                token.balance = balance.clone();
                tracing::info!("📊 Updated {} balance to: {}", token_symbol, balance);
            }
        }

        // Process audio and logs after the mutable borrow on state ends
        for (token_symbol, old_balance, new_balance) in notifications {
            tracing::error!(
                "🔥🔥🔥 TOKEN AUDIO: Playing notification sound for {} balance increase!",
                token_symbol
            );
            if let Err(e) = crate::gui::utils::play_notification_sound() {
                tracing::error!("❌ Failed to play notification sound: {}", e);
            } else {
                tracing::error!("✅ Token audio notification played successfully!");
            }

            // Add log entry
            self.add_log_entry(
                LogCategory::Info,
                format!("Token Received: {}", token_symbol),
                Some(format!("{} → {}", old_balance, new_balance)),
            );
        }

        Command::none()
    }
}
