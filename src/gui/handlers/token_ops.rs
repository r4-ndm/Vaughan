//! Token operation message handlers for WorkingWallet
//!
//! Handles token-related messages including:
//! - Token selection and balance display
//! - Custom token management (add, remove, fetch info)
//! - Token persistence (save, load)

use crate::gui::api_service::fetch_token_info;
use crate::gui::services::{load_custom_tokens, save_custom_tokens};
use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::{LogCategory, Message, SimpleTokenBalance, StatusMessageColor, TokenInfo};
use iced::Command;

impl WorkingWalletApp {
    /// Handle token operation-related messages
    pub fn handle_token_ops_message(&mut self, message: Message) -> Command<Message> {
        match message {
            // Token selection
            Message::BalanceTokenSelected(token) => self.handle_balance_token_selected(token),
            Message::BalanceTickerSelected(ticker) => self.handle_balance_ticker_selected(ticker),
            Message::ShowBalanceAddToken => self.handle_show_balance_add_token(),
            Message::TokenBalanceUpdateNeeded(ticker) => self.handle_token_balance_update_needed(ticker),

            // Send form token management
            Message::SendCustomTokenAddressChanged(address) => self.handle_send_custom_token_address_changed(address),
            Message::SendTxTypeChanged(tx_type) => self.handle_send_tx_type_changed(tx_type),
            Message::SendMaxFeeChanged(v) => self.handle_send_max_fee_changed(v),
            Message::SendMaxPriorityFeeChanged(v) => self.handle_send_max_priority_fee_changed(v),
            Message::SendNonceOverrideChanged(v) => self.handle_send_nonce_override_changed(v),
            Message::GasSpeedSelected(speed) => self.handle_gas_speed_selected(speed),
            Message::ToggleAdvancedSendOptions => self.handle_toggle_advanced_send_options(),
            Message::SendShowCustomTokenInput => self.handle_send_show_custom_token_input(),
            Message::HideCustomTokenInput => self.handle_hide_custom_token_input(),

            // Token info fetching
            Message::FetchTokenInfo(token_address) => self.handle_fetch_token_info(token_address),
            Message::TokenInfoFetched(result) => self.handle_token_info_fetched(result),
            Message::AddCustomToken(token_address) => self.handle_add_custom_token(token_address),
            Message::RemoveCustomToken(token_address) => self.handle_remove_custom_token(token_address),

            // Custom token screen
            Message::ShowCustomTokenScreen => self.handle_show_custom_token_screen(),
            Message::HideCustomTokenScreen => self.handle_hide_custom_token_screen(),
            Message::CustomTokenAddressChanged(address) => self.handle_custom_token_address_changed(address),
            Message::CustomTokenNameChanged(name) => self.handle_custom_token_name_changed(name),
            Message::CustomTokenSymbolChanged(symbol) => self.handle_custom_token_symbol_changed(symbol),
            Message::CustomTokenDecimalsChanged(decimals) => self.handle_custom_token_decimals_changed(decimals),
            Message::CreateCustomTokenManually => self.handle_create_custom_token_manually(),
            Message::AutoFetchTokenInfo => self.handle_auto_fetch_token_info(),

            // Clipboard operations for tokens
            Message::PasteTokenAddress => self.handle_paste_token_address(),
            Message::SendPasteFromClipboard => self.handle_send_paste_from_clipboard(),
            Message::SendPasteAddressFromClipboard => {
                tracing::info!("📋 SendPasteAddressFromClipboard message received in handler");
                self.handle_send_paste_address_from_clipboard()
            }
            Message::SendPasteAmountFromClipboard => self.handle_send_paste_amount_from_clipboard(),

            // Token persistence
            Message::LoadCustomTokens => self.handle_load_custom_tokens(),
            Message::CustomTokensLoaded(tokens) => self.handle_custom_tokens_loaded(tokens),
            Message::SaveCustomTokens => self.handle_save_custom_tokens(),

            // Send from account selection
            Message::SendFromAccountSelected(account_id) => self.handle_send_from_account_selected(account_id),

            _ => Command::none(),
        }
    }

    // Token selection handlers
    fn handle_balance_token_selected(&mut self, token: String) -> Command<Message> {
        self.state.balance_selected_token = token.clone();

        // Synchronize with send form selector
        let matching_send_token = self
            .state
            .transaction_mut()
            .send_available_tokens
            .iter()
            .find(|send_token| {
                send_token.contains(&token)
                    || (token.contains('(') && send_token.contains(token.split('(').next().unwrap_or(&token).trim()))
            })
            .cloned();

        if let Some(matching_token) = matching_send_token {
            self.state.transaction_mut().send_selected_token = matching_token;
            self.state.transaction_mut().send_amount.clear();
        }

        // Log token balance for debugging
        if !token.contains("NATIVE") {
            if let Some(token_balance) = self
                .state
                .token_balances
                .iter()
                .find(|tb| tb.symbol == token.split('(').next().unwrap_or(&token).trim())
            {
                tracing::info!(
                    "📊 Token {} selected for sending, balance available: {}",
                    token_balance.symbol,
                    token_balance.balance
                );
            } else {
                tracing::warn!("⚠️ Could not find balance for token: {}", token);
            }
        }
        Command::none()
    }

    fn handle_balance_ticker_selected(&mut self, ticker: String) -> Command<Message> {
        self.state.balance_selected_ticker = ticker.clone();
        self.state.balance_selected_token = ticker.clone();
        self.state.transaction_mut().send_selected_token = ticker.clone();
        self.state.transaction_mut().send_amount.clear();
        let _ = self.handle_wallet_ops_message(Message::UpdateAccountBalance);
        Command::none()
    }

    fn handle_show_balance_add_token(&mut self) -> Command<Message> {
        self.state.show_custom_token_screen = true;
        Command::none()
    }

    fn handle_token_balance_update_needed(&mut self, ticker: String) -> Command<Message> {
        tracing::info!("💰 Updating token balance for: {}", ticker);
        let _ = self.handle_wallet_ops_message(Message::UpdateAccountBalance);
        Command::none()
    }

    // Send form token management handlers
    fn handle_send_custom_token_address_changed(&mut self, address: String) -> Command<Message> {
        self.state.transaction_mut().send_custom_token_address = address;
        Command::none()
    }

    fn handle_send_tx_type_changed(&mut self, tx_type: String) -> Command<Message> {
        self.state.transaction_mut().send_tx_type = tx_type;
        Command::none()
    }

    fn handle_send_max_fee_changed(&mut self, v: String) -> Command<Message> {
        self.state.transaction_mut().send_max_fee_gwei = v;
        Command::none()
    }

    fn handle_send_max_priority_fee_changed(&mut self, v: String) -> Command<Message> {
        self.state.transaction_mut().send_max_priority_fee_gwei = v;
        Command::none()
    }

    fn handle_send_nonce_override_changed(&mut self, v: String) -> Command<Message> {
        self.state.transaction_mut().send_nonce_override = v;
        Command::none()
    }

    fn handle_gas_speed_selected(&mut self, speed: crate::gui::wallet_types::GasSpeed) -> Command<Message> {
        self.state.transaction_mut().gas_speed = speed;

        // Network-specific base gas prices (Gwei)
        let network_base_price: f64 = match self.state.network().current_network.chain_id() {
            1 => 25.0,    // Ethereum Mainnet
            56 => 5.0,    // BSC
            137 => 30.0,  // Polygon
            369 => 1.0,   // PulseChain
            943 => 1.0,   // PulseChain Testnet
            42161 => 0.1, // Arbitrum One
            10 => 0.001,  // Optimism
            _ => 20.0,    // Default fallback
        };

        // Apply speed multiplier
        let multiplier = self.state.transaction().gas_speed.multiplier();
        let adjusted_price = network_base_price * multiplier;
        self.state.transaction_mut().send_gas_price = format!("{:.1}", adjusted_price.max(0.1f64));

        // Update EIP-1559 max fee
        let max_fee = adjusted_price * 1.5;
        self.state.transaction_mut().send_max_fee_gwei = format!("{:.1}", max_fee.max(0.1f64));

        // Set priority fee for EIP-1559
        if self.state.transaction().send_tx_type == "EIP-1559" {
            let priority_fee = max_fee * 0.1;
            self.state.transaction_mut().send_max_priority_fee_gwei = format!("{:.1}", priority_fee.max(0.1f64));
        }

        Command::none()
    }

    fn handle_toggle_advanced_send_options(&mut self) -> Command<Message> {
        self.state.transaction_mut().send_show_advanced = !self.state.transaction().send_show_advanced;
        Command::none()
    }

    fn handle_send_show_custom_token_input(&mut self) -> Command<Message> {
        self.state.transaction_mut().send_show_custom_token_input = true;
        Command::none()
    }

    fn handle_hide_custom_token_input(&mut self) -> Command<Message> {
        self.state.transaction_mut().send_show_custom_token_input = false;
        self.state.transaction_mut().send_custom_token_address.clear();
        Command::none()
    }

    // Token info fetching handlers
    fn handle_fetch_token_info(&mut self, token_address: String) -> Command<Message> {
        // Validate address format
        if token_address.len() != 42 || !token_address.starts_with("0x") {
            self.add_log_entry(
                LogCategory::Error,
                "Invalid token address".to_string(),
                Some("Token address must be a valid Ethereum address (0x followed by 40 hex characters)".to_string()),
            );
            return Command::none();
        }

        // Check if already exists
        let already_exists = self
            .state
            .custom_tokens
            .iter()
            .any(|token| token.address.to_lowercase() == token_address.to_lowercase());

        if already_exists {
            self.add_log_entry(
                LogCategory::Error,
                "Token already added".to_string(),
                Some("This token is already in your custom tokens list".to_string()),
            );
            return Command::none();
        }

        self.state.fetching_token_info = true;
        self.state.pending_token_address = token_address.clone();
        let network_id = self.state.network().current_network;

        Command::perform(fetch_token_info(token_address, network_id), Message::TokenInfoFetched)
    }

    fn handle_token_info_fetched(&mut self, result: Result<TokenInfo, String>) -> Command<Message> {
        self.state.fetching_token_info = false;
        self.state.pending_token_address = String::new();

        match result {
            Ok(token_info) => {
                // Add to custom tokens
                self.state.custom_tokens.push(token_info.clone());

                // Parse and add to token_balances
                let contract_address = match token_info.address.parse::<alloy::primitives::Address>() {
                    Ok(addr) => Some(addr),
                    Err(e) => {
                        tracing::error!("Failed to parse contract address '{}': {}", token_info.address, e);
                        None
                    }
                };

                self.state.token_balances.push(SimpleTokenBalance {
                    symbol: token_info.symbol.clone(),
                    name: token_info.name.clone(),
                    contract_address,
                    balance: "0.0000".to_string(),
                    decimals: token_info.decimals,
                });

                // Add to balance available tickers
                let token_display = format!("{} ({})", token_info.symbol, token_info.address);
                if !self.state.balance_available_tickers.contains(&token_display) {
                    self.state.balance_available_tickers.push(token_display.clone());
                }

                // Add to available tokens dropdown
                self.state
                    .transaction_mut()
                    .send_available_tokens
                    .push(token_display.clone());
                self.state.transaction_mut().send_selected_token = token_display;

                // Update custom token screen if open
                if self.state.show_custom_token_screen {
                    self.state.custom_token_name_input = token_info.name.clone();
                    self.state.custom_token_symbol_input = token_info.symbol.clone();
                    self.state.custom_token_decimals_input = token_info.decimals.to_string();
                    self.state.custom_token_validation_error = None;
                } else {
                    self.state.transaction_mut().send_custom_token_address.clear();
                    self.state.transaction_mut().send_show_custom_token_input = false;
                }

                self.state.transaction_mut().send_amount.clear();

                self.add_log_entry(
                    LogCategory::Success,
                    "Custom token added successfully".to_string(),
                    Some(format!(
                        "Added {} ({}) - {} decimals",
                        token_info.name, token_info.symbol, token_info.decimals
                    )),
                );

                // Save tokens and trigger balance update
                let tokens_to_save = self.state.custom_tokens.clone();
                let ticker = token_info.symbol.clone();

                Command::batch([
                    Command::perform(async move { save_custom_tokens(&tokens_to_save[..]).await }, |result| {
                        if let Err(e) = result {
                            tracing::error!("Failed to save custom tokens: {}", e);
                        }
                        Message::UserActivity
                    }),
                    Command::perform(
                        async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            ticker
                        },
                        Message::TokenBalanceUpdateNeeded,
                    ),
                ])
            }
            Err(error) => {
                if self.state.show_custom_token_screen {
                    self.state.custom_token_validation_error = Some(format!("Failed to fetch token info: {error}"));
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Failed to fetch token information".to_string(),
                        Some(format!("Error: {error}")),
                    );
                }
                Command::none()
            }
        }
    }

    fn handle_add_custom_token(&mut self, token_address: String) -> Command<Message> {
        self.dispatch_message(Message::FetchTokenInfo(token_address))
    }

    fn handle_remove_custom_token(&mut self, token_address: String) -> Command<Message> {
        let token_to_remove = self
            .state
            .custom_tokens
            .iter()
            .find(|t| t.address == token_address)
            .cloned();

        self.state.custom_tokens.retain(|token| token.address != token_address);
        self.state
            .transaction_mut()
            .send_available_tokens
            .retain(|token_str| !token_str.contains(&token_address));

        if let Some(token) = token_to_remove {
            self.state
                .balance_available_tickers
                .retain(|ticker| ticker != &token.symbol);
        }

        if self
            .state
            .transaction_mut()
            .send_selected_token
            .contains(&token_address)
        {
            self.state.transaction_mut().send_selected_token = "NATIVE (ETH)".to_string();
            self.state.transaction_mut().send_amount.clear();
        }

        self.add_log_entry(
            LogCategory::Success,
            "Custom token removed".to_string(),
            Some(format!("Removed token {token_address}")),
        );

        let tokens_to_save = self.state.custom_tokens.clone();
        Command::perform(async move { save_custom_tokens(&tokens_to_save[..]).await }, |result| {
            if let Err(e) = result {
                tracing::error!("Failed to save custom tokens: {}", e);
            }
            Message::UserActivity
        })
    }

    // Custom token screen handlers
    fn handle_show_custom_token_screen(&mut self) -> Command<Message> {
        self.state.show_custom_token_screen = true;
        self.state.custom_token_address_input.clear();
        self.state.custom_token_name_input.clear();
        self.state.custom_token_symbol_input.clear();
        self.state.custom_token_decimals_input = "18".to_string();
        self.state.custom_token_validation_error = None;
        Command::none()
    }

    fn handle_hide_custom_token_screen(&mut self) -> Command<Message> {
        self.state.show_custom_token_screen = false;
        self.state.custom_token_address_input.clear();
        self.state.custom_token_name_input.clear();
        self.state.custom_token_symbol_input.clear();
        self.state.custom_token_decimals_input = "18".to_string();
        self.state.custom_token_validation_error = None;
        Command::none()
    }

    fn handle_custom_token_address_changed(&mut self, address: String) -> Command<Message> {
        self.state.custom_token_address_input = address;
        self.state.custom_token_validation_error = None;
        Command::none()
    }

    fn handle_custom_token_name_changed(&mut self, name: String) -> Command<Message> {
        self.state.custom_token_name_input = name;
        Command::none()
    }

    fn handle_custom_token_symbol_changed(&mut self, symbol: String) -> Command<Message> {
        self.state.custom_token_symbol_input = symbol;
        Command::none()
    }

    fn handle_custom_token_decimals_changed(&mut self, decimals: String) -> Command<Message> {
        if decimals.is_empty() || decimals.parse::<u8>().is_ok() {
            self.state.custom_token_decimals_input = decimals;
        }
        Command::none()
    }

    fn handle_create_custom_token_manually(&mut self) -> Command<Message> {
        let address = self.state.custom_token_address_input.clone();
        let name = self.state.custom_token_name_input.clone();
        let symbol = self.state.custom_token_symbol_input.clone();
        let decimals_str = self.state.custom_token_decimals_input.clone();

        // Validate inputs
        if address.len() != 42 || !address.starts_with("0x") {
            self.state.custom_token_validation_error =
                Some("Invalid token address format. Must be 42 characters starting with 0x".to_string());
            return Command::none();
        }

        if name.trim().is_empty() {
            self.state.custom_token_validation_error = Some("Token name is required".to_string());
            return Command::none();
        }

        if symbol.trim().is_empty() {
            self.state.custom_token_validation_error = Some("Token symbol is required".to_string());
            return Command::none();
        }

        let decimals = match decimals_str.parse::<u8>() {
            Ok(d) => d,
            Err(_) => {
                self.state.custom_token_validation_error =
                    Some("Invalid decimals value. Must be a number between 0 and 255".to_string());
                return Command::none();
            }
        };

        // Check if token already exists
        let already_exists = self
            .state
            .custom_tokens
            .iter()
            .any(|token| token.address.to_lowercase() == address.to_lowercase());

        if already_exists {
            self.state.custom_token_validation_error =
                Some("Token already exists in your custom tokens list".to_string());
            return Command::none();
        }

        // Create the token info
        let token_info = TokenInfo {
            address: address.clone(),
            name: name.clone(),
            symbol: symbol.clone(),
            decimals,
            balance: None,
        };

        self.state.custom_tokens.push(token_info);

        let token_display = format!("{symbol} ({address})");
        self.state
            .transaction_mut()
            .send_available_tokens
            .push(token_display.clone());

        if !self.state.balance_available_tickers.contains(&token_display) {
            self.state.balance_available_tickers.push(token_display.clone());
        }

        self.state.transaction_mut().send_selected_token = token_display;

        // Clear inputs
        self.state.custom_token_address_input.clear();
        self.state.custom_token_name_input.clear();
        self.state.custom_token_symbol_input.clear();
        self.state.custom_token_decimals_input = "18".to_string();

        self.add_log_entry(
            LogCategory::Success,
            "Custom token added manually".to_string(),
            Some(format!("Added {name} ({symbol}) - {decimals} decimals")),
        );

        let tokens_to_save = self.state.custom_tokens.clone();
        Command::perform(async move { save_custom_tokens(&tokens_to_save[..]).await }, |result| {
            if let Err(e) = result {
                tracing::error!("Failed to save custom tokens: {}", e);
            }
            Message::UserActivity
        })
    }

    fn handle_auto_fetch_token_info(&mut self) -> Command<Message> {
        let token_address = self.state.custom_token_address_input.clone();

        if token_address.len() != 42 || !token_address.starts_with("0x") {
            self.state.custom_token_validation_error =
                Some("Invalid token address format. Must be 42 characters starting with 0x".to_string());
            return Command::none();
        }

        let already_exists = self
            .state
            .custom_tokens
            .iter()
            .any(|token| token.address.to_lowercase() == token_address.to_lowercase());

        if already_exists {
            self.state.custom_token_validation_error =
                Some("Token already exists in your custom tokens list".to_string());
            return Command::none();
        }

        self.state.fetching_token_info = true;
        self.state.custom_token_validation_error = None;
        self.state.pending_token_address = token_address.clone();

        let network_id = self.state.network().current_network;
        Command::perform(fetch_token_info(token_address, network_id), Message::TokenInfoFetched)
    }

    // Clipboard operation handlers
    fn handle_paste_token_address(&mut self) -> Command<Message> {
        Command::perform(
            async {
                match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
                    Ok(text) => Ok(text.trim().to_string()),
                    Err(e) => Err(format!("Failed to access clipboard: {e}")),
                }
            },
            |result| match result {
                Ok(text) => Message::CustomTokenAddressChanged(text),
                Err(_) => {
                    Message::SetStatusMessage("Failed to paste from clipboard".to_string(), StatusMessageColor::Error)
                }
            },
        )
    }

    fn handle_send_paste_from_clipboard(&mut self) -> Command<Message> {
        Command::perform(
            async {
                match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
                    Ok(text) => Ok(text.trim().to_string()),
                    Err(e) => Err(format!("Failed to access clipboard: {e}")),
                }
            },
            |result| match result {
                Ok(text) => Message::SendCustomTokenAddressChanged(text),
                Err(_) => {
                    Message::SetStatusMessage("Failed to paste from clipboard".to_string(), StatusMessageColor::Error)
                }
            },
        )
    }

    fn handle_send_paste_address_from_clipboard(&mut self) -> Command<Message> {
        tracing::info!("📋 Paste address button clicked - attempting to read clipboard");
        Command::perform(
            async {
                tracing::info!("📋 Inside async clipboard read task");
                match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
                    Ok(text) => {
                        tracing::info!("📋 Successfully read from clipboard: {}", text);
                        Ok(text.trim().to_string())
                    }
                    Err(e) => {
                        tracing::error!("📋 Failed to access clipboard: {}", e);
                        Err(format!("Failed to access clipboard: {e}"))
                    }
                }
            },
            |result| match result {
                Ok(text) => {
                    tracing::info!("📋 Sending SendToAddressChanged message with: {}", text);
                    Message::SendToAddressChanged(text)
                }
                Err(e) => {
                    tracing::error!("📋 Clipboard error: {}", e);
                    Message::SetStatusMessage("Failed to paste from clipboard".to_string(), StatusMessageColor::Error)
                }
            },
        )
    }

    fn handle_send_paste_amount_from_clipboard(&mut self) -> Command<Message> {
        Command::perform(
            async {
                match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
                    Ok(text) => Ok(text.trim().to_string()),
                    Err(e) => Err(format!("Failed to access clipboard: {e}")),
                }
            },
            |result| match result {
                Ok(text) => Message::SendAmountChanged(text),
                Err(_) => {
                    Message::SetStatusMessage("Failed to paste from clipboard".to_string(), StatusMessageColor::Error)
                }
            },
        )
    }

    // Token persistence handlers
    fn handle_load_custom_tokens(&mut self) -> Command<Message> {
        Command::perform(load_custom_tokens(), |result| {
            Message::CustomTokensLoaded(result.unwrap_or_default())
        })
    }

    fn handle_custom_tokens_loaded(&mut self, tokens: Vec<TokenInfo>) -> Command<Message> {
        self.state.custom_tokens = tokens.clone();

        for token in &tokens {
            let token_display = token.symbol.clone();

            if !self
                .state
                .transaction_mut()
                .send_available_tokens
                .contains(&token_display)
            {
                self.state
                    .transaction_mut()
                    .send_available_tokens
                    .push(token_display.clone());
            }

            let token_display = format!("{} ({})", token.symbol, token.address);
            if !self.state.balance_available_tickers.contains(&token_display) {
                self.state.balance_available_tickers.push(token_display);
            }

            // Add to token_balances for balance refresh
            let contract_address = match token.address.parse::<alloy::primitives::Address>() {
                Ok(addr) => Some(addr),
                Err(e) => {
                    tracing::error!("Failed to parse token contract address '{}': {}", token.address, e);
                    None
                }
            };

            let token_exists = self.state.token_balances.iter().any(|t| t.symbol == token.symbol);
            if !token_exists {
                self.state.token_balances.push(SimpleTokenBalance {
                    symbol: token.symbol.clone(),
                    name: token.name.clone(),
                    contract_address,
                    balance: "0.0000".to_string(),
                    decimals: token.decimals,
                });
                tracing::info!("📊 Added {} to token_balances for balance tracking", token.symbol);
            }
        }

        tracing::info!(
            "📦 Loaded {} custom tokens into both send and balance dropdowns",
            tokens.len()
        );
        Command::none()
    }

    fn handle_save_custom_tokens(&mut self) -> Command<Message> {
        let tokens = self.state.custom_tokens.clone();
        Command::perform(
            async move {
                if let Err(e) = save_custom_tokens(&tokens).await {
                    tracing::error!("Failed to save custom tokens: {}", e);
                }
            },
            |_| Message::UserActivity,
        )
    }

    fn handle_send_from_account_selected(&mut self, account_id: String) -> Command<Message> {
        self.state.transaction_mut().send_from_account_id = Some(account_id.clone());

        if let Some(account) = self
            .state
            .wallet()
            .available_accounts
            .iter()
            .find(|a| a.id == account_id)
        {
            if let Some(wallet) = &self.wallet {
                let wallet_clone = wallet.clone();
                let account_clone = account.clone();
                return Command::perform(
                    async move {
                        let mut wallet = wallet_clone.write().await;
                        wallet.unlock_with_account(account_clone).await
                    },
                    |result| match result {
                        Ok(_) => {
                            tracing::info!("✅ Wallet unlocked with send-from account for transaction");
                            Message::SetStatusMessage("Send account selected".to_string(), StatusMessageColor::Success)
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to unlock wallet with send-from account: {}", e);
                            Message::SetStatusMessage(
                                "Failed to select send account".to_string(),
                                StatusMessageColor::Error,
                            )
                        }
                    },
                );
            }
        }
        Command::none()
    }
}
