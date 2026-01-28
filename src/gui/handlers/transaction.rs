//! Transaction message handlers for WorkingWalletApp
//!
//! Handles all transaction-related messages using the simple Alloy-based service.

use crate::gui::simple_transaction::{estimate_gas, send_transaction};
use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::{LogCategory, Message, StatusMessageColor};
use iced::Command;
use std::time::Instant;

impl WorkingWalletApp {
    /// Handle transaction-related messages
    pub fn handle_transaction_message(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EstimateGas => self.handle_estimate_gas(),
            Message::GasEstimated(result) => self.handle_gas_estimated(result),
            Message::ShowTransactionConfirmation => self.handle_show_transaction_confirmation(),
            Message::HideTransactionConfirmation => self.handle_hide_transaction_confirmation(),
            Message::ConfirmTransaction => self.handle_confirm_transaction(),
            Message::SubmitTransaction => self.handle_submit_transaction(),
            Message::TransactionSubmitted(result) => self.handle_transaction_submitted(result),
            // Legacy/Unused messages that might still be emitted by UI
            _ => Command::none(),
        }
    }

    /// Validate transaction using TransactionFormService (Phase 5 - parallel implementation)
    ///
    /// This method runs alongside legacy validation for comparison and gradual rollout.
    /// When use_transaction_service flag is true, validation errors will block the transaction.
    ///
    /// # Returns
    /// - Ok(()) if validation passes
    /// - Err(String) with user-friendly error message if validation fails
    fn validate_transaction_with_service(&self) -> Result<(), String> {
        use crate::gui::services::TransactionFormServiceTrait;
        
        let service = self.state.services().transaction_form();
        let tx_state = self.state.transaction();
        
        // 1. Validate recipient address
        let recipient = &tx_state.send_to_address;
        if let Err(e) = service.validate_recipient(recipient) {
            tracing::warn!("❌ Service validation failed - recipient: {}", e);
            return Err(format!("Invalid recipient address: {}", e));
        }
        tracing::debug!("✅ Service validation passed - recipient");
        
        // 2. Validate amount
        let amount = &tx_state.send_amount;
        
        // Get current balance from state - try multiple sources
        let balance_str = self.state.account_balance.clone();
        tracing::debug!("🔍 Raw balance string for validation: '{}'", balance_str);
        
        // Parse balance - handle various formats
        let balance = if let Ok(balance_f64) = balance_str
            .replace(" ETH", "")
            .replace(" tPLS", "")
            .replace(" BNB", "")
            .replace(" ", "")
            .replace(",", "")
            .trim()
            .parse::<f64>()
        {
            use alloy::primitives::U256;
            tracing::debug!("✅ Parsed balance: {} (as f64)", balance_f64);
            U256::from((balance_f64 * 1e18) as u128)
        } else {
            // Balance parsing failed - check if it's an error state
            if balance_str.contains("Error") || balance_str.contains("loading") || balance_str.is_empty() {
                tracing::warn!("❌ Balance is in error state: '{}'", balance_str);
                return Err("Unable to verify balance. Please refresh your balance and try again.".to_string());
            }
            
            tracing::warn!("❌ Could not parse balance for validation: '{}'", balance_str);
            return Err("Could not determine account balance. Please refresh and try again.".to_string());
        };
        
        tracing::debug!("💰 Balance for validation: {} wei", balance);
        
        // Validate amount (18 decimals for ETH/native tokens)
        if let Err(e) = service.validate_amount(amount, balance, 18) {
            tracing::warn!("❌ Service validation failed - amount: {}", e);
            return Err(format!("Invalid amount: {}", e));
        }
        tracing::debug!("✅ Service validation passed - amount");
        
        // 3. Validate gas limit if provided
        if !tx_state.send_gas_limit.is_empty() {
            if let Err(e) = service.validate_gas_limit(&tx_state.send_gas_limit) {
                tracing::warn!("❌ Service validation failed - gas limit: {}", e);
                return Err(format!("Invalid gas limit: {}", e));
            }
            tracing::debug!("✅ Service validation passed - gas limit");
        }
        
        // 4. Validate gas price if provided
        if !tx_state.send_gas_price.is_empty() {
            if let Err(e) = service.validate_gas_price(&tx_state.send_gas_price) {
                tracing::warn!("❌ Service validation failed - gas price: {}", e);
                return Err(format!("Invalid gas price: {}", e));
            }
            tracing::debug!("✅ Service validation passed - gas price");
        }
        
        tracing::info!("✅ All service validations passed");
        Ok(())
    }

    /// Handle gas estimation request
    fn handle_estimate_gas(&mut self) -> Command<Message> {
        if self.state.transaction().estimating_gas {
            return Command::none();
        }

        self.state.transaction_mut().estimating_gas = true;
        self.state.transaction_mut().gas_estimation = None;

        let to_address = self.state.transaction().send_to_address.clone();
        let amount = self.state.transaction().send_amount.clone();

        // Get from address
        let from_address = if let Some(account_id) = &self.state.wallet().current_account_id {
            if let Some(account) = self
                .state
                .wallet()
                .available_accounts
                .iter()
                .find(|a| &a.id == account_id)
            {
                format!("{:#x}", account.address)
            } else {
                return Command::none();
            }
        } else {
            return Command::none();
        };

        // Get RPC URL
        let rpc_url = self.state.network().get_current_rpc_url();

        // Extract token contract address if this is an ERC-20 token
        let selected_token = &self.state.transaction().send_selected_token;
        tracing::info!("🔍 Selected token for gas estimation: '{}'", selected_token);

        let token_contract = if selected_token.contains("NATIVE") {
            // Native token (ETH, BNB, etc.)
            tracing::info!("💰 Using native token transfer");
            None
        } else {
            // ERC-20 token - extract contract address from format "TOKEN (0x123...)" or find by symbol
            tracing::info!("🪙 Attempting ERC-20 token transfer from: '{}'", selected_token);
            if let Some(paren_start) = selected_token.find('(') {
                if let Some(paren_end) = selected_token.find(')') {
                    let address_str = selected_token[paren_start + 1..paren_end].trim();
                    tracing::info!("📋 Extracted contract address: '{}'", address_str);
                    match address_str.parse::<alloy::primitives::Address>() {
                        Ok(address) => {
                            tracing::info!("✅ ERC-20 contract address parsed: {:#x}", address);
                            Some(address)
                        }
                        Err(_) => {
                            tracing::error!("❌ Invalid token contract address: {}", address_str);
                            None
                        }
                    }
                } else {
                    tracing::error!(
                        "❌ No closing parenthesis found in token selection: '{}'",
                        selected_token
                    );
                    None
                }
            } else {
                // Token name without parentheses - need to look up contract address
                tracing::info!(
                    "🔍 Token selection without parentheses format, looking up contract for: '{}'",
                    selected_token
                );

                // Look up contract address from token name in loaded token balances
                let token_balances = self.state.token_balances();
                if let Some(token_info) = token_balances.iter().find(|t| t.symbol == *selected_token) {
                    if let Some(contract_address) = token_info.contract_address {
                        tracing::info!(
                            "✅ Found token contract from lookup: {} = {:#x}",
                            selected_token,
                            contract_address
                        );
                        Some(contract_address)
                    } else {
                        tracing::error!(
                            "❌ Token info has no contract address (native token?): {}",
                            selected_token
                        );
                        None
                    }
                } else {
                    tracing::error!("❌ Token not found in loaded token balances: '{}'", selected_token);
                    None
                }
            }
        };

        tracing::info!(
            "⛽ Starting gas estimation: to={}, amount={}, token={}",
            to_address,
            amount,
            selected_token
        );

        Command::perform(
            async move {
                estimate_gas(&to_address, &amount, &from_address, &rpc_url, token_contract)
                    .await
                    .map(|gas| {
                        let cost = format!("{:.6}", (gas as f64 * 20.0 * 1e9) / 1e18);
                        crate::gui::GasEstimation {
                            estimated_gas: gas,
                            gas_price: "20.0".to_string(), // Default for now
                            estimated_cost: cost.clone(),
                            total_cost: cost,
                            currency: "ETH".to_string(),
                        }
                    })
            },
            Message::GasEstimated,
        )
    }

    /// Handle gas estimation result
    fn handle_gas_estimated(&mut self, result: Result<crate::gui::GasEstimation, String>) -> Command<Message> {
        self.state.transaction_mut().estimating_gas = false;
        match result {
            Ok(gas_estimation) => {
                tracing::info!("✅ Gas estimation successful: {} gas", gas_estimation.estimated_gas);
                self.state.transaction_mut().gas_estimation = Some(gas_estimation);
                self.state.transaction_mut().show_transaction_confirmation = true;
                Command::none()
            }
            Err(error_string) => {
                tracing::error!("❌ Gas estimation failed: {}", error_string);

                let selected_token = &self.state.transaction().send_selected_token;
                let is_erc20 = !selected_token.contains("NATIVE");
                let token_name = if is_erc20 {
                    selected_token.split('(').next().unwrap_or("Token").trim()
                } else {
                    "tPLS"
                };

                // Improve error message based on error type and token type
                let improved_message = if error_string.contains("insufficient") || error_string.contains("balance") {
                    // 🔧 CRITICAL FIX: Check if this is actual token balance issue vs technical error
                    let selected_token = &self.state.transaction().send_selected_token;
                    let is_custom_token = self
                        .state
                        .custom_tokens
                        .iter()
                        .any(|token| selected_token.contains(&token.symbol));

                    if is_custom_token {
                        // Check if token balance exists in our tracking
                        let token_symbol = if let Some(paren_pos) = selected_token.find('(') {
                            selected_token[..paren_pos].trim()
                        } else {
                            selected_token
                        };

                        let has_balance = self.state.token_balances.iter().any(|tb| {
                            tb.symbol == token_symbol && tb.balance != "0.000000" && !tb.balance.starts_with('0')
                        });

                        if has_balance {
                            format!("⚠️  Gas estimation failed: {}. This is likely a technical issue with token balance fetching. Try again or check your token balance directly.", error_string)
                        } else {
                            format!("Gas estimation failed: {}. You don't have enough {} tokens to send. Make sure you have added the custom token correctly and fetched its balance.", error_string, token_name)
                        }
                    } else {
                        format!("Gas estimation failed: {}. You don't have enough {} for gas fees. Add some {} to your wallet.", error_string, token_name, token_name)
                    }
                } else {
                    format!("Gas estimation failed: {}", error_string)
                };

                self.state.ui_mut().status_message = improved_message;
                self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                self.state.ui_mut().status_message_timer = Some(Instant::now());
                Command::none()
            }
        }
    }

    /// Handle show transaction confirmation
    fn handle_show_transaction_confirmation(&mut self) -> Command<Message> {
        self.state.transaction_mut().show_transaction_confirmation = true;
        Command::none()
    }

    /// Handle hide transaction confirmation
    fn handle_hide_transaction_confirmation(&mut self) -> Command<Message> {
        self.state.transaction_mut().show_transaction_confirmation = false;
        self.state.transaction_mut().gas_estimation = None;
        Command::none()
    }

    /// Handle transaction confirmation - DIRECT SENDING (No 2-tier security)
    fn handle_confirm_transaction(&mut self) -> Command<Message> {
        if self.state.transaction_mut().sending_transaction {
            return Command::none();
        }

        // PHASE 5: Parallel validation with TransactionFormService
        // Run service validation alongside legacy validation for comparison
        let service_validation_result = self.validate_transaction_with_service();
        
        if self.state.use_transaction_service {
            // Feature flag enabled: Block transaction if service validation fails
            if let Err(error_msg) = service_validation_result {
                tracing::error!("🚫 Transaction blocked by service validation: {}", error_msg);
                self.state.ui_mut().status_message = error_msg;
                self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                self.state.ui_mut().status_message_timer = Some(Instant::now());
                return Command::none();
            }
            tracing::info!("✅ Service validation passed - proceeding with transaction");
        } else {
            // Feature flag disabled: Log validation results but don't block
            match service_validation_result {
                Ok(()) => {
                    tracing::info!("✅ [PARALLEL] Service validation passed (not enforced)");
                }
                Err(error_msg) => {
                    tracing::warn!("⚠️ [PARALLEL] Service validation would have failed: {}", error_msg);
                    tracing::warn!("⚠️ [PARALLEL] Transaction proceeding with legacy validation");
                }
            }
        }

        // Check if we need master password authentication for seed-based accounts
        if let Some(_wallet_arc) = &self.wallet {
            // Check current account type - this needs to be synchronous
            let current_account_id = self.state.wallet().current_account_id.clone();
            let needs_master_password = if let Some(account_id) = current_account_id {
                if let Some(account) = self
                    .state
                    .wallet()
                    .available_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                {
                    if account.key_reference.service == "vaughan-wallet-encrypted-seeds" {
                        // This is a seed-based account - check if we have a temporary password
                        let needs_password = self.state.auth().session.temporary_key.is_none();
                        if needs_password {
                            Some((account.id.clone(), account.name.clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((_account_id, account_name)) = needs_master_password {
                // Need to prompt for master password
                tracing::info!("🔐 Seed-based account needs master password - showing dialog");

                // Clear any existing transaction state
                self.state.transaction_mut().show_transaction_confirmation = false;

                // Build transaction details string for password dialog
                let to_address = &self.state.transaction().send_to_address;
                let amount = &self.state.transaction().send_amount;
                let token = &self.state.transaction().send_selected_token;
                let tx_details = format!(
                    "From: {}\nTo: {}\nAmount: {} {}",
                    account_name,
                    to_address,
                    amount,
                    token
                );

                // Show unified password dialog with SignTransaction config
                // This ensures the transaction proceeds after password validation
                use crate::gui::state::auth_state::PasswordDialogConfig;
                self.state
                    .auth_mut()
                    .password_dialog
                    .show(PasswordDialogConfig::SignTransaction { tx_details });
                return Command::none();
            }
        }

        self.state.transaction_mut().sending_transaction = true;
        self.state.transaction_mut().show_transaction_confirmation = false;

        let to_address = self.state.transaction().send_to_address.clone();
        let amount = self.state.transaction().send_amount.clone();
        let rpc_url = self.state.network().get_current_rpc_url();
        let chain_id = self.state.network().current_network.0;

        // Extract token contract address if this is an ERC-20 token
        let selected_token = &self.state.transaction().send_selected_token;
        tracing::info!("🔍 Selected token for transaction: '{}'", selected_token);

        let token_contract = if selected_token.contains("NATIVE") {
            // Native token (ETH, BNB, etc.)
            tracing::info!("💰 Using native token transfer for transaction");
            None
        } else {
            // ERC-20 token - extract contract address from format "TOKEN (0x123...)"
            // or fallback to token name lookup if no parentheses format
            tracing::info!("🪙 Attempting ERC-20 token transfer for: '{}'", selected_token);
            if let Some(paren_start) = selected_token.find('(') {
                if let Some(paren_end) = selected_token.find(')') {
                    let address_str = selected_token[paren_start + 1..paren_end].trim();
                    tracing::info!("📋 Extracted contract address for transaction: '{}'", address_str);
                    match address_str.parse::<alloy::primitives::Address>() {
                        Ok(address) => {
                            tracing::info!("✅ ERC-20 contract address parsed for transaction: {:#x}", address);
                            Some(address)
                        }
                        Err(_) => {
                            tracing::error!("❌ Invalid token contract address for transaction: {}", address_str);
                            None
                        }
                    }
                } else {
                    tracing::error!(
                        "❌ No closing parenthesis found in token selection for transaction: '{}'",
                        selected_token
                    );
                    None
                }
            } else {
                // Token name without parentheses - need to look up contract address
                tracing::info!(
                    "🔍 Token selection without parentheses format, looking up contract for: '{}'",
                    selected_token
                );

                // Look up contract address from token name in loaded token balances
                let token_balances = self.state.token_balances();
                if let Some(token_info) = token_balances.iter().find(|t| t.symbol == *selected_token) {
                    if let Some(contract_address) = token_info.contract_address {
                        tracing::info!(
                            "✅ Found token contract from lookup: {} = {:#x}",
                            selected_token,
                            contract_address
                        );
                        Some(contract_address)
                    } else {
                        tracing::error!(
                            "❌ Token info has no contract address (native token?): {}",
                            selected_token
                        );
                        None
                    }
                } else {
                    tracing::error!("❌ Token not found in loaded token balances: '{}'", selected_token);
                    None
                }
            }
        };

        // Get wallet and account to retrieve seed phrase
        let wallet_arc = if let Some(w) = &self.wallet {
            w.clone()
        } else {
            self.state.transaction_mut().sending_transaction = false;
            self.state.ui_mut().status_message = "No wallet available".to_string();
            self.state.ui_mut().status_message_color = StatusMessageColor::Error;
            return Command::none();
        };

        // Get the temporary key for seed-based accounts
        let temporary_key = self.state.auth().session.temporary_key.clone();

        // Extract token decimals lookup data before entering async block
        let token_balances = self.state.token_balances.clone();
        let custom_tokens = self.state.custom_tokens.clone();
        // Extract gas estimation before async block
        let gas_estimation = self.state.transaction().gas_estimation.clone();

        tracing::info!("🔐 Retrieving seed phrase for transaction signing");

        Command::perform(
            async move {
                use secrecy::ExposeSecret;

                // Get private key - check if seed-based account first to avoid unnecessary keychain access
                let wallet_read = wallet_arc.read().await;
                let account = wallet_read
                    .current_account()
                    .await
                    .ok_or_else(|| "No account selected".to_string())?;

                let private_key_hex = if account.key_reference.service == "vaughan-wallet-encrypted-seeds" {
                    // This is a seed-based account - use the temporary password directly
                    if let Some(master_password) = temporary_key {
                        tracing::info!("🌱 Using stored master password for HD wallet derivation");

                        // Use the validated master password to derive HD wallet
                        let keychain = Box::new(
                            crate::security::keychain::OSKeychain::new("vaughan-wallet-encrypted-seeds".to_string())
                                .map_err(|e| format!("Failed to access keychain: {e}"))?,
                        );

                        let default_path = "m/44'/60'/0'/0/0".to_string();
                        let derivation_path = account.derivation_path.as_ref().unwrap_or(&default_path);

                        crate::gui::hd_wallet_service::HDWalletService::extract_private_key_from_encrypted_seed(
                            keychain.as_ref(),
                            &account.key_reference,
                            derivation_path,
                            &master_password,
                        )
                        .await
                        .map_err(|e| format!("HD wallet private key extraction failed: {e}"))?
                    } else {
                        // No master password available (should not happen due to pre-check)
                        return Err("Master password required for seed-based account".to_string());
                    }
                } else {
                    // For private key accounts, use the existing wallet method
                    match wallet_read.get_private_key_for_deployment().await {
                        Ok(private_key) => private_key.expose_secret().to_string(),
                        Err(e) => {
                            return Err(format!("Failed to get private key for private key account: {e}"));
                        }
                    }
                };

                tracing::info!("🚀 Sending transaction using Alloy");

                // Get token decimals if this is an ERC-20 transfer
                let token_decimals = if let Some(contract_addr) = token_contract {
                    // Look up decimals from token_balances or custom_tokens
                    token_balances
                        .iter()
                        .find(|token| token.contract_address == Some(contract_addr))
                        .map(|token| token.decimals)
                        .or_else(|| {
                            // Fallback: check custom_tokens
                            custom_tokens
                                .iter()
                                .find(|token| {
                                    token.address.parse::<alloy::primitives::Address>().unwrap_or_default()
                                        == contract_addr
                                })
                                .map(|token| token.decimals)
                        })
                } else {
                    None
                };

                tracing::info!(
                    "🔢 Using {} decimals for transaction",
                    token_decimals
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "default".to_string())
                );

                // Use gas limit from estimation (industry standard)
                let gas_limit = if let Some(gas_est) = &gas_estimation {
                    Some(gas_est.estimated_gas)
                } else {
                    // Fallback to conservative defaults if no estimation available
                    if token_contract.is_some() {
                        Some(65000) // Conservative ERC-20 default
                    } else {
                        Some(21000) // Native transfer default
                    }
                };

                tracing::info!(
                    "⛽ Using gas limit: {} (from estimation: {})",
                    gas_limit.unwrap_or(0),
                    gas_estimation.as_ref().map(|g| g.estimated_gas).unwrap_or(0)
                );

                // Send transaction using Alloy
                send_transaction(
                    &to_address,
                    &amount,
                    &private_key_hex,
                    &rpc_url,
                    chain_id,
                    gas_limit,      // Use estimated gas limit
                    Some(20.0),     // Default gas price
                    token_contract, // Pass token contract for ERC-20 transfers
                    token_decimals, // Pass token decimals for proper conversion
                )
                .await
                .map(|hash| (hash, None))
            },
            Message::TransactionSubmitted,
        )
    }

    /// Handle submit transaction (triggers gas estimation)
    fn handle_submit_transaction(&mut self) -> Command<Message> {
        tracing::info!("📝 Transaction form submitted - initiating gas estimation");
        self.handle_estimate_gas()
    }

    /// Handle transaction submission result
    fn handle_transaction_submitted(
        &mut self,
        result: Result<(String, Option<crate::gui::state::transaction_state::PendingTransaction>), String>,
    ) -> Command<Message> {
        self.state.transaction_mut().sending_transaction = false;

        // Clear temporary key for security (one-time use after transaction completion)
        self.state.auth_mut().session.temporary_key = None;

        match result {
            Ok((tx_hash, _)) => {
                tracing::info!("✅ Transaction submitted successfully: {tx_hash}");

                self.add_log_entry(
                    LogCategory::Wallet,
                    "Transaction submitted successfully".to_string(),
                    Some(format!("Transaction hash: {tx_hash}")),
                );

                self.add_transaction_to_history(tx_hash.clone());

                // Clear form
                self.state.transaction_mut().send_to_address.clear();
                self.state.transaction_mut().send_amount.clear();
                self.state.transaction_mut().gas_estimation = None;

                self.state.ui_mut().status_message = format!("Transaction submitted: {tx_hash}");
                self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                self.state.ui_mut().status_message_timer = Some(Instant::now());

                self.update_account_balance()
            }
            Err(error_string) => {
                tracing::error!("❌ Transaction failed: {}", error_string);
                self.state.ui_mut().status_message = format!("Transaction failed: {error_string}");
                self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                self.state.ui_mut().status_message_timer = Some(Instant::now());
                Command::none()
            }
        }
    }
}
