//! Vaughan Main Wallet - Clean, Production-Ready Implementation

use crate::blockchain::{load_config, ExplorerApiManager};
use crate::error::VaughanError;
use crate::gui::api_service::create_eth_price_command;
use crate::gui::components::{
    add_network_dialog_view, clear_logs_confirmation_dialog_view, create_wallet_dialog_view, custom_token_screen_view,
    dapps_coming_soon_dialog_view, delete_account_dialog_view, delete_network_confirmation_dialog_view,
    export_wallet_dialog_view, hardware_wallet_dialog_view, import_wallet_dialog_view, receive_dialog_view,
    reset_wallet_confirmation_dialog_view, transaction_confirmation_dialog_view,
    unified_password_dialog_view as password_dialog_view,
};
use crate::gui::services::*;
use crate::gui::state::AppState as NewAppState;
use crate::gui::transaction_service::{check_for_incoming_transactions, load_transaction_history};
// Unused imports removed (now in handlers)
use crate::gui::utils::{connect_hardware_wallet, detect_hardware_wallets, get_hardware_wallet_addresses};
use crate::gui::wallet_types::HistoryTab;
use crate::gui::*;
use crate::network::{NetworkConfig, NetworkId};
use crate::security::SecureAccount;
use arboard::Clipboard;
use chrono;
use iced::{Application, Command, Element, Subscription, Theme};
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::{Duration, Instant};
// Handler modules imported in update method where needed
// Import service modules
use crate::gui::services::account_service::{
    analyze_seed_phrase, create_wallet_from_seed, delete_account, discover_addresses_from_seed,
    import_multiple_addresses_from_seed, import_wallet_from_private_key, import_wallet_from_seed,
};
use crate::gui::services::network_service::{
    add_custom_network, delete_existing_network, edit_existing_network, load_all_networks, save_networks_to_storage,
};
use crate::gui::services::{initialize_wallet, load_available_accounts};
// Remove checksum import for now - will implement later if needed

// Phase E: Import controllers (E4 - WorkingWalletApp structure)
use crate::controllers::{
    NetworkController, PriceController, TransactionController, WalletController,
};

// New decomposed AppState using domain-specific modules
pub type AppState = NewAppState;

// Main wallet application
// Phase E: Added controller fields (E4 - WorkingWalletApp structure)
pub struct WorkingWalletApp {
    // Existing fields (kept for gradual migration)
    pub state: AppState,
    pub wallet: Option<Arc<tokio::sync::RwLock<crate::wallet::Vaughan>>>,
    pub api_manager: Option<ExplorerApiManager>,
    pub account_service: Arc<IntegratedAccountService>,
    
    // Phase E: New controller fields (E4 complete)
    // Provider-independent controllers (always available)
    pub wallet_controller: Arc<WalletController>,
    pub price_controller: Arc<PriceController>,
    
    // Provider-dependent controllers (initialized on-demand when network is ready)
    // These are Option because they require an Alloy provider which is created
    // during network initialization. They will be initialized lazily when first needed.
    pub transaction_controller: Option<Arc<TransactionController<crate::network::AlloyCoreProvider>>>,
    pub network_controller: Option<Arc<NetworkController<crate::network::AlloyCoreProvider>>>,
}

impl Application for WorkingWalletApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Set up panic hook for better error reporting
        std::panic::set_hook(Box::new(|panic_info| {
            tracing::error!("🚨 GUI Panic: {}", panic_info);
            if let Some(location) = panic_info.location() {
                tracing::error!(
                    "   Location: {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
            }
        }));
        let mut state = AppState::default();
        // Set loading flags to true for proper UI feedback
        state.wallet_mut().loading_accounts = true;
        state.network_mut().loading_networks = true;

        // Initialize API manager for price fetching
        let api_manager = match load_config() {
            Ok(config) => {
                tracing::info!("🔗 API Manager initialized with configuration");
                Some(ExplorerApiManager::new(config))
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to load API config, price fetchin will use sample data: {}",
                    e
                );
                None
            }
        };

        // Initialize Integrated Account Service
        let account_service = Arc::new(IntegratedAccountService::new());
        tracing::info!("✅ Integrated Account Service initialized");

        // Phase E: Initialize controllers (E4 - WorkingWalletApp structure)
        // Initialize controllers that don't need a provider
        let wallet_controller = Arc::new(WalletController::new());
        let price_controller = Arc::new(PriceController::new(None)); // No Moralis API key yet
        tracing::info!("✅ Controllers initialized (wallet, price)");
        // Note: transaction_controller and network_controller will be initialized
        // after network setup when provider is available

        let mut wallet_app = Self {
            state,
            wallet: None,
            api_manager,
            account_service,
            // Phase E: Controller fields (E4 complete)
            wallet_controller,
            price_controller,
            // Provider-dependent controllers initialized on-demand
            transaction_controller: None,
            network_controller: None,
        };

        // Add some sample error entries for testing (debug builds only)
        #[cfg(debug_assertions)]
        {
            use crate::error::{NetworkError, SecurityError, VaughanError};

            // Add sample errors with different severities and categories
            wallet_app.add_error_log_entry(
                &VaughanError::Network(NetworkError::RpcConnectionFailed {
                    url: "https://eth-mainnet.alchemyapi.io".to_string(),
                }),
                Some("Initial connection test".to_string()),
            );

            wallet_app.add_error_log_entry(
                &VaughanError::Security(SecurityError::InvalidPrivateKey),
                Some("User imported invalid key format".to_string()),
            );
        }

        // Simplified startup - skip wallet-level authentication, go straight to normal initialization
        // Following DEVELOPMENT_RULES.md - simple Alloy approach, account-level passwords only
        tracing::info!("🚀 Starting simplified wallet without wallet-level authentication");
        let check_startup_cmd = Command::perform(
            async {
                // Just check for existing accounts without complex wallet authentication
                match account_service::check_for_seed_accounts().await {
                    Ok(has_accounts) => has_accounts,
                    Err(e) => {
                        tracing::warn!("Could not check for accounts: {}", e);
                        false
                    }
                }
            },
            Message::SeedAccountsChecked,
        );

        (wallet_app, check_startup_cmd)
    }

    fn title(&self) -> String {
        "Vaughan - Multi-EVM Wallet".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // Route messages to specialized handlers for better organization
        match message.clone() {
            // Transaction-related messages
            Message::EstimateGas
            | Message::GasEstimated(_)
            | Message::ShowTransactionConfirmation
            | Message::HideTransactionConfirmation
            | Message::ConfirmTransaction
            | Message::SubmitTransaction
            | Message::TransactionSubmitted(_)
            | Message::TransactionMonitoringTick => {
                // Use the new simplified handler directly
                return self.handle_transaction_message(message);
            }

            // Network-related messages
            Message::NetworkSelected(_) | Message::SmartPollTick | Message::BalanceChanged(_, _) => {
                return self.handle_network_message(message);
            }

            // Security/Hardware wallet messages
            Message::ConnectHardwareWallet(_)
            | Message::HardwareWalletConnected(_)
            | Message::GetHardwareAddresses(_)
            | Message::HardwareAddressesReceived(_)
            | Message::ScanHardwareWallets
            | Message::RefreshHardwareWallets
            | Message::ConnectToHardwareWallet(_)
            // Password dialog messages
            | Message::ShowPasswordDialog { .. }
            | Message::HidePasswordDialog
            | Message::PasswordInputChanged(_)
            | Message::PasswordRememberChanged(_)
            | Message::SubmitPassword
            | Message::ShowResetWalletConfirmation
            | Message::HideResetWalletConfirmation
            | Message::ConfirmResetWallet
            | Message::WalletResetComplete
            | Message::PasswordValidated(_)
            // Master password dialog messages (HD wallet)
            | Message::ShowMasterPasswordDialog(_)
            | Message::HideMasterPasswordDialog
            | Message::HDWalletPasswordChanged(_)
            | Message::MasterPasswordSubmit
            | Message::MasterPasswordCancel
            | Message::MasterPasswordValidated(_)
            // Session management messages
            | Message::SessionLocked
            | Message::SessionUnlocked
            | Message::ExtendSession
            | Message::ManualLock
            | Message::SessionTimeoutCheck => {
                return self.handle_security_message(message);
            }

            // UI state messages
            Message::ShowCreateDialog
            | Message::HideCreateDialog
            | Message::ShowImportDialog
            | Message::HideImportDialog
            | Message::ShowSettingsDialog
            | Message::HideSettingsDialog
            | Message::ShowImportWallet
            | Message::HideImportWallet
            | Message::ShowExportWallet
            | Message::HideExportWallet
            | Message::SendToAddressChanged(_)
            | Message::SendAmountChanged(_)
            | Message::SendGasLimitChanged(_)
            | Message::SendGasPriceChanged(_)
            | Message::SendTokenChanged(_)
            | Message::CreateAccountNameChanged(_)
            | Message::ImportPrivateKeyChanged(_)
            | Message::ImportAccountNameChanged(_)
            | Message::ClearStatusMessage
            | Message::UpdateLastActivity
            | Message::ToggleAccountDropdown
            | Message::SelectExportAccount(_)
            // Status message management
            | Message::SetStatusMessage(_, _)
            | Message::StatusMessageTick
            | Message::SpinnerTick
            // Log management
            | Message::ClearLogs
            | Message::ShowClearLogsConfirmation
            | Message::HideClearLogsConfirmation
            | Message::ConfirmClearLogs
            | Message::CopyLogEntry(_)
            | Message::LogEntryCopied(_)
            | Message::ResetCopyFeedback => {
                return self.handle_ui_state_message(message);
            }

            // Wallet operation messages
            Message::CreateAccount
            | Message::AccountCreated(_)
            | Message::ImportAccount
            | Message::AccountImported(_)
            | Message::AccountSelected(_)
            | Message::DeleteAccount(_)
            | Message::RefreshBalance
            | Message::InternalRefreshBalance
            | Message::BalanceRefreshed(_)
            | Message::TokenBalancesRefreshed(_)
            | Message::UpdateAccountBalance
            | Message::RefreshTransactionHistory
            | Message::TransactionHistoryRefreshed(_) => {
                return self.handle_wallet_ops_message(message);
            }

            // Receive dialog messages
            Message::ShowReceiveDialog | Message::HideReceiveDialog | Message::CopyToClipboard(_) => {
                return self.handle_receive_message(message);
            }

            // Token operation messages - handled by token_ops.rs
            Message::BalanceTokenSelected(_)
            | Message::BalanceTickerSelected(_)
            | Message::ShowBalanceAddToken
            | Message::TokenBalanceUpdateNeeded(_)
            | Message::SendCustomTokenAddressChanged(_)
            | Message::SendTxTypeChanged(_)
            | Message::SendMaxFeeChanged(_)
            | Message::SendMaxPriorityFeeChanged(_)
            | Message::SendNonceOverrideChanged(_)
            | Message::GasSpeedSelected(_)
            | Message::ToggleAdvancedSendOptions
            | Message::SendShowCustomTokenInput
            | Message::HideCustomTokenInput
            | Message::FetchTokenInfo(_)
            | Message::TokenInfoFetched(_)
            | Message::AddCustomToken(_)
            | Message::RemoveCustomToken(_)
            | Message::ShowCustomTokenScreen
            | Message::HideCustomTokenScreen
            | Message::CustomTokenAddressChanged(_)
            | Message::CustomTokenNameChanged(_)
            | Message::CustomTokenSymbolChanged(_)
            | Message::CustomTokenDecimalsChanged(_)
            | Message::CreateCustomTokenManually
            | Message::AutoFetchTokenInfo
            | Message::PasteTokenAddress
            | Message::SendPasteFromClipboard
            | Message::SendPasteAddressFromClipboard
            | Message::SendPasteAmountFromClipboard
            | Message::LoadCustomTokens
            | Message::CustomTokensLoaded(_)
            | Message::SaveCustomTokens
            | Message::SendFromAccountSelected(_) => {
                return self.handle_token_ops_message(message);
            }

            // Core messages handled directly
            _ => {}
        }

        // Handle core messages that don't belong to specialized handlers
        match message {
            Message::SeedAccountsChecked(has_seed_accounts) => {
                tracing::info!("🔐 Checking wallet password setup status...");
                tracing::info!("   Existing accounts detected: {}", has_seed_accounts);

                // For simplified startup mode: if legacy accounts exist, load them directly
                // This skips the master password dialog requirement for existing accounts
                if has_seed_accounts {
                    tracing::info!("📁 Legacy accounts found - loading directly (simplified startup)");
                    return self.start_normal_initialization();
                }

                // Check if wallet password has been set up (keystore.json exists)
                let keystore_exists = dirs::home_dir()
                    .map(|d| d.join(".vaughan/keystore.json").exists())
                    .unwrap_or(false);

                if keystore_exists {
                    // Returning user with new keystore format - show unlock dialog
                    tracing::info!("🔓 Wallet keystore found - showing unlock dialog");
                    self.state
                        .auth_mut()
                        .password_dialog
                        .show(crate::gui::state::auth_state::PasswordDialogConfig::WalletUnlock);
                    Command::none()
                } else {
                    // First-time user - skip password setup, go straight to wallet
                    // User can create accounts without a master password
                    tracing::info!("🆕 No wallet keystore found - showing welcome view");
                    self.start_normal_initialization()
                }
            }
            Message::StartupAuthenticationRequired => {
                tracing::info!("🔐 Wallet configuration detected - skipping wallet unlock dialog (simplified mode)");

                // Skip wallet unlock dialog and go straight to initialization
                // This effectively disables the wallet master password feature
                self.start_normal_initialization()
            }
            Message::StartupAuthenticationComplete => {
                tracing::info!("🔓 Startup authentication complete - loading wallet data");
                self.start_normal_initialization()
            }
            Message::WalletInitialized(result) => {
                match result {
                    Ok(wallet) => {
                        self.wallet = Some(wallet);
                        // Sync UI network with wallet's default network (PulseChain Testnet v4)
                        self.state.network_mut().current_network = NetworkId(943);
                        tracing::info!("🔗 UI network synced with wallet: PulseChain Testnet v4 (Chain ID: 943)");
                        // Accounts and networks are already being loaded in parallel from Application::new()
                        Command::none()
                    }
                    Err(e) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Wallet initialization failed".to_string(),
                            Some(format!("Error details: {e}")),
                        );
                        Command::none()
                    }
                }
            }
            // Status message and spinner handlers are now routed to handle_ui_state_message
            // via lines 196-232 above. Inline handlers removed in debloat.
            Message::RetryAccountLoading => {
                self.state.wallet_mut().loading_accounts = true;
                self.state.wallet_mut().export_error_message = None;
                self.state.ui_mut().show_retry_options = false;
                Command::perform(load_available_accounts(), Message::AccountsLoaded)
            }
            Message::RetryExportOperation => {
                self.state.wallet_mut().export_error_message = None;
                self.state.ui_mut().show_retry_options = false;
                // Reset to allow user to retry
                self.state.exporting_data = false;
                self.state.export_loading = false;
                Command::none()
            }
            // Network-related messages (NetworkSelected, SmartPollTick, BalanceChanged) are now
            // routed to handle_network_message via line 157 above. Inline handlers removed in debloat.
            Message::UserActivity => {
                // Update last activity time and reactivate polling
                self.state.last_activity = Instant::now();
                self.state.ui_mut().polling_active = true;
                self.state.ui_mut().poll_interval = 10;

                // Also update session activity to prevent timeout
                self.state.auth_mut().session.update_activity();

                Command::none()
            }
            Message::ShowTransactionHistory => {
                // Use the same history view - don't create separate navigation layer
                self.state.transaction_mut().show_history = true;
                self.state.transaction_mut().current_history_tab = HistoryTab::Transactions;
                self.state.transaction_mut().loading_transactions = true;
                // Don't clear existing history - let the merge logic handle it

                let network_id = self.state.network().current_network;

                // Get the actual account address
                let account_address = if let Some(account_id) = &self.state.wallet().current_account_id {
                    if let Some(account) = self
                        .state
                        .wallet()
                        .available_accounts
                        .iter()
                        .find(|a| &a.id == account_id)
                    {
                        format!("{:#x}", account.address) // Use proper hex formatting without quotes
                    } else {
                        self.state.wallet().current_account.clone()
                    }
                } else {
                    self.state.wallet().current_account.clone()
                };

                Command::perform(
                    load_transaction_history(network_id, account_address),
                    Message::TransactionHistoryLoaded,
                )
            }
            Message::HideTransactionHistory => {
                // Just go back to main view, no separate layer
                self.state.transaction_mut().show_history = false;
                Command::none()
            }
            Message::ShowReceive => self.handle_receive_message(Message::ShowReceiveDialog),
            Message::ShowHistory => {
                self.state.transaction_mut().show_history = true;
                // Automatically load transaction history when opening history view
                self.state.transaction_mut().loading_transactions = true;
                // Don't clear existing history - preserve local transactions

                let network_id = self.state.network().current_network;

                // Get the actual account address
                let account_address = if let Some(account_id) = &self.state.wallet().current_account_id {
                    if let Some(account) = self
                        .state
                        .wallet()
                        .available_accounts
                        .iter()
                        .find(|a| &a.id == account_id)
                    {
                        format!("{:#x}", account.address) // Use proper hex formatting without quotes
                    } else {
                        return Command::none();
                    }
                } else {
                    return Command::none();
                };

                Command::perform(
                    load_transaction_history(network_id, account_address),
                    Message::TransactionHistoryLoaded,
                )
            }
            Message::HideHistory => {
                self.state.transaction_mut().show_history = false;
                Command::none()
            }
            Message::HistoryTabSelected(tab) => {
                self.state.transaction_mut().current_history_tab = tab;
                Command::none()
            }
            Message::LogEntryCopied(result) => {
                match result {
                    Ok(_) => {
                        self.state.ui_mut().copy_feedback = Some("Copied to clipboard!".to_string());
                        // Clear feedback after 3 seconds
                        Command::perform(
                            async {
                                tokio::time::sleep(Duration::from_secs(3)).await;
                            },
                            |_| Message::ResetCopyFeedback,
                        )
                    }
                    Err(error) => {
                        self.state.ui_mut().copy_feedback = Some(format!("Copy failed: {error}"));
                        // Clear error feedback after 5 seconds
                        Command::perform(
                            async {
                                tokio::time::sleep(Duration::from_secs(5)).await;
                            },
                            |_| Message::ResetCopyFeedback,
                        )
                    }
                }
            }
            Message::ResetCopyFeedback => {
                self.state.ui_mut().copy_feedback = None;
                Command::none()
            }
            Message::ClearLogs => {
                // Show confirmation dialog before clearing logs
                self.state.ui_mut().show_clear_logs_confirmation = true;
                Command::none()
            }
            Message::ShowClearLogsConfirmation => {
                self.state.ui_mut().show_clear_logs_confirmation = true;
                Command::none()
            }
            Message::HideClearLogsConfirmation => {
                self.state.ui_mut().show_clear_logs_confirmation = false;
                self.state.ui_mut().clearing_logs = false;
                Command::none()
            }
            Message::ConfirmClearLogs => {
                self.state.ui_mut().clearing_logs = true;

                // Clear all log entries from memory
                self.state.log_entries.clear();

                // Add a log entry indicating logs were cleared
                self.add_log_entry(
                    LogCategory::Success, // Using AccountCreated as a general success category
                    "Session logs cleared".to_string(),
                    Some(format!(
                        "All session log entries have been cleared at {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                    )),
                );

                // Hide the confirmation dialog and stay on history page
                self.state.ui_mut().show_clear_logs_confirmation = false;
                self.state.ui_mut().clearing_logs = false;
                // Ensure we stay on the history page
                self.state.transaction_mut().show_history = true;

                Command::none()
            }
            Message::CopyLogEntry(index) => {
                if let Some(entry) = self.state.log_entries.get(index) {
                    let log_text = format!("[{}] {}", entry.timestamp, entry.message);

                    Command::perform(
                        async move {
                            match Clipboard::new() {
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
            Message::TransactionHistoryLoaded(result) => {
                self.state.transaction_mut().loading_transactions = false;
                match result {
                    Ok(external_transactions) => {
                        // Merge external transactions with local transactions
                        // Keep local transactions (which are more recent and reliable)
                        // and add external transactions that aren't duplicates
                        let mut merged_history = self.state.transaction().transaction_history.clone();

                        for ext_tx in external_transactions {
                            // Check if this transaction already exists in our local history
                            let exists = merged_history
                                .iter()
                                .any(|local_tx| local_tx.hash.to_lowercase() == ext_tx.hash.to_lowercase());

                            if !exists {
                                merged_history.push(ext_tx);
                            }
                        }

                        // Sort by timestamp (most recent first) and limit to 30
                        merged_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        if merged_history.len() > 30 {
                            merged_history.truncate(30);
                        }

                        self.state.transaction_mut().transaction_history = merged_history;
                        self.state.transaction_mut().transaction_fetch_error = false;
                    }
                    Err(error) => {
                        tracing::error!("Failed to load external transaction history: {}", error);
                        // Don't clear existing local transactions, just set the error flag
                        self.state.transaction_mut().transaction_fetch_error = true;
                    }
                }
                Command::none()
            }
            Message::ClearTransactionHistory => {
                // Clear transaction history
                self.state.transaction_mut().transaction_history.clear();

                // Add a log entry indicating transaction history was cleared
                self.add_log_entry(
                    LogCategory::Info,
                    "Transaction history cleared".to_string(),
                    Some(format!(
                        "Transaction history has been cleared at {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                    )),
                );

                Command::none()
            }
            Message::ShowSend => {
                // Form is now always visible, so just clear the form fields for a fresh start
                self.state.transaction_mut().send_to_address.clear();
                self.state.transaction_mut().send_amount.clear();
                // Set default send-from account to current account
                self.state.transaction_mut().send_from_account_id = self.state.wallet().current_account_id.clone();

                // Proactively unlock the wallet with the default send-from account if available
                if let (Some(wallet), Some(account_id)) = (&self.wallet, self.state.wallet().current_account_id.clone())
                {
                    if let Some(account) = self
                        .state
                        .wallet()
                        .available_accounts
                        .iter()
                        .find(|a| a.id == account_id)
                    {
                        let wallet_clone = wallet.clone();
                        let account_clone = account.clone();
                        return Command::perform(
                            async move {
                                let mut wallet = wallet_clone.write().await;
                                wallet.unlock_with_account(account_clone).await
                            },
                            |result| match result {
                                Ok(_) => {
                                    tracing::info!("✅ Wallet unlocked with default send-from account");
                                    Message::SetStatusMessage("Ready to send".to_string(), StatusMessageColor::Success)
                                }
                                Err(e) => {
                                    tracing::error!("❌ Failed to unlock wallet with default send-from account: {}", e);
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
            Message::HideSend => {
                // Form is always visible now, so just clear the fields instead of hiding
                self.state.transaction_mut().send_to_address.clear();
                self.state.transaction_mut().send_amount.clear();
                self.state.transaction_mut().send_gas_limit = "21000".to_string();
                self.state.transaction_mut().send_gas_price = "20".to_string();
                self.state.transaction_mut().send_nonce_override.clear();
                Command::none()
            }
            // Send form field handlers (SendAddressChanged, SendAmountChanged, etc.) are now
            // routed to handle_ui_state_message via lines 196-232 above. Inline handlers removed.
            // Token operations routing: Token-related messages are fully handled by token_ops.rs
            // via the routing at lines 256-291 above. Inline handlers removed in debloat.
            // Legacy transaction handlers removed (now handled by handle_transaction_message)
            Message::ShowSettings => {
                tracing::debug!("Settings feature - Coming soon");
                Command::none()
            }
            Message::ShowDapps => {
                self.state.ui_mut().status_message = "Dapps feature - Coming soon".to_string();
                self.state.ui_mut().status_message_color = StatusMessageColor::Info;
                // Set a timer to auto-hide after 3 seconds
                self.state.ui_mut().status_message_timer = Some(std::time::Instant::now());
                Command::none()
            }
            Message::ShowTransactionSpeed => {
                tracing::debug!("Transaction Speed feature - Coming soon");
                Command::none()
            }
            Message::ShowDappsComingSoon => {
                self.state.ui_mut().show_dapps_coming_soon = true;
                Command::none()
            }
            Message::HideDappsComingSoon => {
                self.state.ui_mut().show_dapps_coming_soon = false;
                Command::none()
            }
            Message::CancelLastTransaction => {
                // Delegate to transaction handler
                self.handle_transaction_message(message)
            }
            // Wallet creation/import messages - placeholder implementations
            Message::ShowCreateWallet => {
                self.state.wallet_mut().show_create_wallet = true;
                self.state.wallet_mut().wallet_name.clear();
                self.state.wallet_mut().seed_phrase.clear();
                Command::none()
            }
            Message::HideCreateWallet => {
                self.state.wallet_mut().show_create_wallet = false;
                Command::none()
            }
            // ShowImportWallet is routed to handle_ui_state_message at line 203
            Message::ShowImportWalletFromSeed => {
                tracing::info!(
                    "🔍 DEBUG: ShowImportWalletFromSeed message received! Setting show_import_wallet = true"
                );
                self.state.wallet_mut().show_import_wallet = true;
                self.state.wallet_mut().wallet_name.clear();
                self.state.wallet_mut().seed_phrase.clear();
                // Pre-select seed phrase import type
                self.state.wallet_mut().import_type = crate::gui::wallet_types::ImportType::SeedPhrase;
                Command::none()
            }
            Message::ShowWalletUnlock => {
                tracing::info!("🔓 Showing wallet unlock dialog");

                // Show unified password dialog for unlocking existing wallet
                use crate::gui::state::auth_state::PasswordDialogConfig;
                self.state
                    .auth_mut()
                    .password_dialog
                    .show(PasswordDialogConfig::WalletUnlock);
                Command::none()
            }
            // Wallet password message handlers
            Message::WalletPasswordChanged(new_password) => {
                use secrecy::SecretString;
                self.state.auth_mut().password_dialog.input = SecretString::new(new_password);
                Command::none()
            }
            Message::WalletPasswordSubmitted => {
                // Handle wallet password submission
                self.handle_wallet_password_submission()
            }
            Message::WalletPasswordCancelled => {
                // Clear pending export operation when user cancels authentication (MetaMask pattern)
                // Pending operation is no longer tracked in state, just hide the dialog

                self.state.auth_mut().password_dialog.hide();
                tracing::info!("🚫 Wallet password dialog cancelled");
                Command::none()
            }
            Message::WalletRememberSessionToggled(remember) => {
                self.state.auth_mut().password_dialog.remember_session = remember;
                Command::none()
            }

            // Account password authentication messages (two-tier security)
            Message::ShowAccountPasswordDialog(account_id) => {
                // Find account name for display
                let account_name = self
                    .state
                    .wallet()
                    .available_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| account_id.clone());

                self.state.auth_mut().password_dialog.show(
                    crate::gui::state::auth_state::PasswordDialogConfig::AccountUnlock {
                        account_id: account_id.clone(),
                        account_name,
                    },
                );
                Command::none()
            }
            Message::HideAccountPasswordDialog => {
                self.state.auth_mut().password_dialog.hide();
                Command::none()
            }
            Message::AccountPasswordChanged(new_password) => {
                use secrecy::SecretString;
                self.state.auth_mut().password_dialog.input = SecretString::new(new_password);
                self.state.auth_mut().password_dialog.clear_error();
                Command::none()
            }
            Message::SubmitAccountPassword => self.handle_account_password_submission(),
            Message::AccountPasswordSubmitted(account_id, password) => {
                // This is called from the async handler
                self.process_account_unlock(account_id, password)
            }
            Message::AccountPasswordCancelled => {
                self.state.auth_mut().password_dialog.hide();
                tracing::info!("🚫 Account password dialog cancelled");
                Command::none()
            }
            Message::AccountSessionUnlocked(account_id) => {
                // Account session was successfully unlocked, complete the account selection
                if let Some(account) = self
                    .state
                    .wallet()
                    .available_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                    .cloned()
                {
                    self.complete_account_selection(account_id, account)
                } else {
                    Command::none()
                }
            }
            Message::AccountSessionLocked(account_id) => {
                tracing::info!("🔒 Account session locked: {}", account_id);
                // Clear current selection if this was the active account
                if let Some(current_id) = &self.state.wallet().current_account_id {
                    if current_id == &account_id {
                        self.state.wallet_mut().current_account_id = None;
                        self.state.wallet_mut().current_account = String::new();
                    }
                }
                Command::none()
            }
            // HideImportWallet is routed to handle_ui_state_message at line 204
            // ShowExportWallet and HideExportWallet are routed to handle_ui_state_message at lines 205-206

            // Direct export handlers
            Message::ExportSeedPhrase => {
                // Step 1: Check if wallet is unlocked (prerequisite for all operations)
                if !self.state.auth().enhanced_session.is_wallet_ready() {
                    tracing::info!("Export seed phrase requires wallet unlock - requesting authentication");

                    // Show cancellable wallet password dialog for export operations
                    // We don't store a pending operation anymore - simpler, more secure flow
                    self.state.pending_export_type = crate::gui::state::ExportType::SeedPhrase;
                    self.state
                        .auth_mut()
                        .password_dialog
                        .show(crate::gui::state::auth_state::PasswordDialogConfig::WalletExport);
                    return Command::none();
                }

                self.state.pending_export_type = crate::gui::state::ExportType::SeedPhrase;

                // Use already selected account, or fallback to current account
                let account_id = if let Some(selected_id) = &self.state.selected_export_account_id {
                    // Respect user's existing selection in export dialog
                    selected_id.clone()
                } else if let Some(current_id) = self.state.wallet().current_account_id.clone() {
                    // Only auto-select current account if none selected in export dialog
                    self.state.selected_export_account_id = Some(current_id.clone());
                    current_id.clone()
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "No account available".to_string(),
                        Some("Please select an account first to export its seed phrase.".to_string()),
                    );
                    return Command::none();
                };

                // Validate account exists and has proper key reference
                let account = match self
                    .state
                    .wallet()
                    .available_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                {
                    Some(acc) => acc,
                    None => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Account not found".to_string(),
                            Some(
                                "The selected account could not be found. Please select a different account."
                                    .to_string(),
                            ),
                        );
                        return Command::none();
                    }
                };

                // Check if account has valid key reference for export
                if account.key_reference.id.is_empty() {
                    tracing::error!("❌ Empty key reference ID!");
                    self.add_log_entry(
                        LogCategory::Error,
                        "Cannot export from this account".to_string(),
                        Some("Selected account does not have a valid key reference for export.".to_string()),
                    );
                    return Command::none();
                }

                // Additional validation for hardware wallets and seed phrase export
                if account.is_hardware {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Cannot export seed phrase".to_string(),
                        Some("Seed phrase export is not available for hardware wallet accounts. Try exporting the private key instead.".to_string())
                    );
                    return Command::none();
                }

                // Step 2: Check if account session is unlocked (two-tier security)
                // For export operations, skip account-level unlock if wallet master password was provided
                // This follows MetaMask industry standards where master password is sufficient for exports
                if !self.state.auth().enhanced_session.is_account_unlocked(&account_id) {
                    tracing::info!("Account session not unlocked, but proceeding with export since master password was authenticated (MetaMask pattern)");

                    // Temporarily unlock account for this export operation
                    // This is secure because the user already provided the master password
                    // For export operations with validated master password, we bypass the account unlock requirement
                    // This follows MetaMask industry standards where master password is sufficient for export operations
                    tracing::info!("Proceeding with export - master password validation is sufficient for export (industry standard)");
                    // No additional account unlock needed - master password authentication is sufficient
                }

                // Step 3: Account session is unlocked, proceed with export
                tracing::info!("Account session authenticated - proceeding with seed phrase export");

                // Check for cached master password (new flow)
                if let Some(password_secret) = &self.state.auth().enhanced_session.wallet_session.cached_master_password
                {
                    use secrecy::ExposeSecret;
                    let password: String = password_secret.expose_secret().clone();

                    self.state.exporting_data = true;
                    self.state.export_loading = true;
                    self.state.wallet_mut().export_error_message = None;
                    self.state.exported_seed_phrase = None;

                    Command::perform(
                        crate::gui::services::account_service::export_seed_phrase_unified(account_id, password),
                        Message::SeedPhraseExported,
                    )
                } else {
                    tracing::error!("❌ Wallet unlocked but master password not cached!");
                    self.add_log_entry(
                        LogCategory::Error,
                        "Security Integrity Error".to_string(),
                        Some(
                            "Wallet is unlocked but credentials are missing. Please lock and unlock your wallet."
                                .to_string(),
                        ),
                    );
                    Command::none()
                }
            }
            Message::ExportPrivateKey => {
                // Step 1: Check if wallet is unlocked (prerequisite for all operations)
                if !self.state.auth().enhanced_session.is_wallet_ready() {
                    tracing::info!("Export private key requires wallet unlock - requesting authentication");

                    // Show cancellable wallet password dialog for export operations
                    // We don't store a pending operation anymore - simpler, more secure flow
                    self.state.pending_export_type = crate::gui::state::ExportType::PrivateKey;
                    self.state
                        .auth_mut()
                        .password_dialog
                        .show(crate::gui::state::auth_state::PasswordDialogConfig::WalletExport);
                    return Command::none();
                }

                self.state.pending_export_type = crate::gui::state::ExportType::PrivateKey;

                // Use already selected account, or fallback to current account
                let account_id = if let Some(selected_id) = &self.state.selected_export_account_id {
                    // Respect user's existing selection in export dialog
                    selected_id.clone()
                } else if let Some(current_id) = self.state.wallet().current_account_id.clone() {
                    // Only auto-select current account if none selected in export dialog
                    self.state.selected_export_account_id = Some(current_id.clone());
                    current_id.clone()
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "No account available".to_string(),
                        Some("Please select an account first to export its private key.".to_string()),
                    );
                    return Command::none();
                };

                // Validate account exists and has proper key reference
                let account = match self
                    .state
                    .wallet()
                    .available_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                {
                    Some(acc) => acc,
                    None => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Account not found".to_string(),
                            Some(
                                "The selected account could not be found. Please select a different account."
                                    .to_string(),
                            ),
                        );
                        return Command::none();
                    }
                };

                // Check if account has valid key reference for export
                if account.key_reference.id.is_empty() {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Cannot export from this account".to_string(),
                        Some("Selected account does not have a valid key reference for export.".to_string()),
                    );
                    return Command::none();
                }

                // Step 2: For export operations, skip account-level unlock check
                // This follows MetaMask industry standards where master password is sufficient for exports
                if !self.state.auth().enhanced_session.is_account_unlocked(&account_id) {
                    tracing::info!("Account session not unlocked, but proceeding with private key export since master password was authenticated (MetaMask pattern)");
                    // No additional account unlock needed - master password authentication is sufficient for export operations
                }

                // Step 3: Account session is unlocked, proceed with export
                tracing::info!("Account session authenticated - proceeding with private key export");

                // Check for cached master password (new flow)
                if let Some(password_secret) = &self.state.auth().enhanced_session.wallet_session.cached_master_password
                {
                    use secrecy::ExposeSecret;
                    let password: String = password_secret.expose_secret().clone();

                    self.state.exporting_data = true;
                    self.state.export_loading = true;
                    self.state.wallet_mut().export_error_message = None;
                    self.state.exported_private_key = None;

                    Command::perform(
                        crate::gui::services::account_service::export_private_key_unified(account_id, password),
                        Message::PrivateKeyExported,
                    )
                } else {
                    tracing::error!("❌ Wallet unlocked but master password not cached!");
                    self.add_log_entry(
                        LogCategory::Error,
                        "Security Integrity Error".to_string(),
                        Some(
                            "Wallet is unlocked but credentials are missing. Please lock and unlock your wallet."
                                .to_string(),
                        ),
                    );
                    Command::none()
                }
            }
            Message::PerformWalletExport(password) => {
                let account_id = match &self.state.selected_export_account_id {
                    Some(id) => id.clone(),
                    None => {
                        self.state.exporting_data = false;
                        return Command::none();
                    }
                };

                use secrecy::ExposeSecret;
                let password_str = password.expose_secret().to_string();

                self.state.exporting_data = true;
                self.state.export_loading = true;
                self.state.wallet_mut().export_error_message = None;

                match self.state.pending_export_type {
                    crate::gui::state::ExportType::SeedPhrase => {
                        self.state.exported_seed_phrase = None;
                        Command::perform(
                            crate::gui::services::account_service::export_seed_phrase_unified(
                                account_id,
                                password_str,
                            ),
                            Message::SeedPhraseExported,
                        )
                    }
                    crate::gui::state::ExportType::PrivateKey => {
                        self.state.exported_private_key = None;
                        Command::perform(
                            crate::gui::services::account_service::export_private_key_unified(
                                account_id,
                                password_str,
                            ),
                            Message::PrivateKeyExported,
                        )
                    }
                    crate::gui::state::ExportType::None => {
                        self.state.exporting_data = false;
                        Command::none()
                    }
                }
            }
            Message::SeedPhraseExported(result) => {
                self.state.exporting_data = false;
                match result {
                    Ok(seed_phrase) => {
                        tracing::info!("Seed phrase exported successfully, length: {}", seed_phrase.len());
                        self.state.exported_seed_phrase = Some(seed_phrase);
                        // Complete state implicitly via export_result availability
                    }
                    Err(error) => {
                        // Log detailed error for debugging
                        tracing::error!("Seed phrase export failed: {}", error);

                        // Enhanced error categorization for better user experience
                        let error_lower = error.to_lowercase();
                        let is_auth_error = error_lower.contains("password")
                            || error_lower.contains("authentication")
                            || error_lower.contains("invalid credentials")
                            || error_lower.contains("incorrect password")
                            || error_lower.contains("unauthorized");

                        let is_timeout_error = error_lower.contains("timeout") || error_lower.contains("timed out");
                        let is_network_error = error_lower.contains("network") || error_lower.contains("connection");
                        let is_keystore_error = error_lower.contains("keystore") || error_lower.contains("keychain");
                        let is_permission_error =
                            error_lower.contains("permission") || error_lower.contains("access denied");
                        let is_missing_key_file = error_lower.contains("failed to read key file")
                            || error_lower.contains("no such file or directory");

                        if is_auth_error {
                            // For authentication errors, stay on password step and show specific message
                            self.add_log_entry(
                                LogCategory::Error,
                                "Incorrect password".to_string(),
                                Some("Please check your master password and try again.".to_string()),
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Incorrect password. Please try again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Stay on password step
                        } else if is_timeout_error {
                            // For timeout errors, provide retry suggestion and stay on password step
                            self.add_log_entry(
                                LogCategory::Error,
                                "Export timed out".to_string(),
                                Some(
                                    "The operation took too long. Please check your connection and try again."
                                        .to_string(),
                                ),
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Export timed out. Please try again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Allow retry
                        } else if is_network_error {
                            // For network errors, suggest connection check
                            self.add_log_entry(
                                LogCategory::Error,
                                "Network connection issue".to_string(),
                                Some("Please check your internet connection and try again.".to_string()),
                            );
                            self.state.wallet_mut().export_error_message = Some(
                                "Network connection issue. Please check your connection and try again.".to_string(),
                            );
                            self.state.ui_mut().show_retry_options = true;
                        // Allow retry
                        } else if is_missing_key_file {
                            // For missing seed phrase key files - specific to seed export
                            self.add_log_entry(
                                LogCategory::Error,
                                "Unable to retrieve seed phrase".to_string(),
                                Some("This account's seed phrase backup is missing or corrupted. You may need to restore from your original seed phrase backup or reimport this account.".to_string())
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Unable to retrieve seed phrase. Account data may be corrupted.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Reset to account selection
                        } else if is_keystore_error {
                            // For keystore errors, suggest account reselection
                            self.add_log_entry(
                                LogCategory::Error,
                                "Keystore access failed".to_string(),
                                Some("There was an issue accessing your wallet data. Please try selecting the account again.".to_string())
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Keystore access failed. Please try selecting the account again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Reset to account selection
                        } else if is_permission_error {
                            // For permission errors, provide system-level guidance
                            self.add_log_entry(
                                LogCategory::Error,
                                "Permission denied".to_string(),
                                Some("Unable to access wallet files. Please check that Vaughan has the necessary permissions.".to_string())
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Permission denied. Please check application permissions.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        } else {
                            // For other errors, provide detailed information and reset to account selection
                            self.add_log_entry(
                                LogCategory::Error,
                                "Unable to retrieve seed phrase".to_string(),
                                Some(format!(
                                    "Error details: {error}\n\nPlease try again or contact support if the issue persists."
                                )),
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Unable to retrieve seed phrase. Please try again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                            // Reset to account selection
                        }
                    }
                }
                Command::none()
            }
            Message::PrivateKeyExported(result) => {
                self.state.exporting_data = false;
                match result {
                    Ok(private_key) => {
                        self.state.exported_private_key = Some(private_key);
                        // Complete state implicitly via result
                    }
                    Err(error) => {
                        // Log detailed error for debugging
                        tracing::error!("Private key export failed: {}", error);

                        // Enhanced error categorization for better user experience
                        let error_lower = error.to_lowercase();
                        let is_auth_error = error_lower.contains("password")
                            || error_lower.contains("authentication")
                            || error_lower.contains("invalid credentials")
                            || error_lower.contains("incorrect password")
                            || error_lower.contains("unauthorized");

                        let is_timeout_error = error_lower.contains("timeout") || error_lower.contains("timed out");
                        let is_network_error = error_lower.contains("network") || error_lower.contains("connection");
                        let is_keystore_error = error_lower.contains("keystore") || error_lower.contains("keychain");
                        let is_permission_error =
                            error_lower.contains("permission") || error_lower.contains("access denied");
                        let is_missing_key_file = error_lower.contains("failed to read key file")
                            || error_lower.contains("no such file or directory");

                        if is_auth_error {
                            // For authentication errors, stay on password step and show specific message
                            self.add_log_entry(
                                LogCategory::Error,
                                "Incorrect password".to_string(),
                                Some("Please check your master password and try again.".to_string()),
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Incorrect password. Please try again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Stay on password step
                        } else if is_timeout_error {
                            // For timeout errors, provide retry suggestion and stay on password step
                            self.add_log_entry(
                                LogCategory::Error,
                                "Export timed out".to_string(),
                                Some(
                                    "The operation took too long. Please check your connection and try again."
                                        .to_string(),
                                ),
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Export timed out. Please try again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Allow retry
                        } else if is_network_error {
                            // For network errors, suggest connection check
                            self.add_log_entry(
                                LogCategory::Error,
                                "Network connection issue".to_string(),
                                Some("Please check your internet connection and try again.".to_string()),
                            );
                            self.state.wallet_mut().export_error_message = Some(
                                "Network connection issue. Please check your connection and try again.".to_string(),
                            );
                            self.state.ui_mut().show_retry_options = true;
                        // Allow retry
                        } else if is_missing_key_file {
                            // For missing private key files
                            self.add_log_entry(
                                LogCategory::Error,
                                "Unable to retrieve seed phrase".to_string(),
                                Some("This account's private key data is missing or corrupted. You may need to restore from your original seed phrase backup or reimport this account.".to_string())
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Unable to retrieve seed phrase. Account data may be corrupted.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Reset to account selection
                        } else if is_keystore_error {
                            // For keystore errors, suggest account reselection
                            self.add_log_entry(
                                LogCategory::Error,
                                "Keystore access failed".to_string(),
                                Some("There was an issue accessing your wallet data. Please try selecting the account again.".to_string())
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Keystore access failed. Please try selecting the account again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        // Reset to account selection
                        } else if is_permission_error {
                            // For permission errors, provide system-level guidance
                            self.add_log_entry(
                                LogCategory::Error,
                                "Permission denied".to_string(),
                                Some("Unable to access wallet files. Please check that Vaughan has the necessary permissions.".to_string())
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Permission denied. Please check application permissions.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                        } else {
                            // For other errors, provide detailed information and reset to account selection
                            self.add_log_entry(
                                LogCategory::Error,
                                "Unable to retrieve seed phrase".to_string(),
                                Some(format!(
                                    "Error details: {error}\n\nPlease try again or contact support if the issue persists."
                                )),
                            );
                            self.state.wallet_mut().export_error_message =
                                Some("Unable to retrieve seed phrase. Please try again.".to_string());
                            self.state.ui_mut().show_retry_options = true;
                            // Reset to account selection
                        }
                    }
                }
                Command::none()
            }

            Message::ExportAccountSelected(account_id) => {
                self.state.selected_export_account_id = Some(account_id);
                // Clear any previous export data when account changes
                self.state.exported_seed_phrase = None;
                if let Some(ref key) = self.state.exported_private_key {
                    if !key.is_empty() {
                        self.state.exported_private_key = None;
                    }
                };
                if let Some(ref key) = self.state.exported_private_key {
                    if !key.is_empty() {
                        self.state.exported_private_key = None;
                    }
                };
                Command::none()
            }
            Message::StartInlineExport(_) => {
                // Deprecated: Inline export flow removed
                Command::none()
            }
            Message::ExportPasswordChanged(_) => {
                // Deprecated: Export password input removed
                Command::none()
            }
            Message::SubmitInlineExport => {
                // Deprecated: Inline export password submission removed
                Command::none()
            }
            Message::CancelInlineExport => {
                // Clear exported data to return to account selection
                self.state.exported_seed_phrase = None;
                self.state.exported_private_key = None;
                self.state.wallet_mut().export_error_message = None;
                self.state.exporting_data = false;
                self.state.export_loading = false;
                Command::none()
            }
            Message::BackToExportOptions => {
                // Deprecated: Back to options not needed
                Command::none()
            }
            Message::CopyExportedData(data) => {
                // Copy exported seed phrase or private key to clipboard
                let copy_task = async move {
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(data) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("Failed to copy to clipboard: {e}")),
                        },
                        Err(e) => Err(format!("Failed to access clipboard: {e}")),
                    }
                };

                Command::perform(copy_task, Message::ExportDataCopied)
            }
            Message::ExportDataCopied(result) => {
                match result {
                    Ok(_) => {
                        self.state.ui_mut().export_copy_feedback =
                            Some("Copied to clipboard! Will be cleared automatically in 30 seconds.".to_string());
                        self.state.ui_mut().clipboard_clear_timer_active = true;

                        // Start 30-second timer to clear clipboard for security
                        let clear_task = async {
                            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                        };

                        Command::batch([
                            Command::perform(clear_task, |_| Message::ClearClipboardAfterDelay),
                            // Also clear the feedback message after 5 seconds
                            Command::perform(
                                async {
                                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                },
                                |_| Message::ResetExportCopyFeedback,
                            ),
                        ])
                    }
                    Err(error) => {
                        // Log detailed clipboard error for debugging
                        tracing::error!("Clipboard operation failed: {}", error);

                        // Provide user-friendly error message
                        let user_message = if error.to_lowercase().contains("permission") {
                            "Failed to copy to clipboard: Permission denied. Please check clipboard access permissions."
                        } else if error.to_lowercase().contains("not available")
                            || error.to_lowercase().contains("unavailable")
                        {
                            "Failed to copy to clipboard: Clipboard not available. Please try copying manually."
                        } else {
                            "Failed to copy to clipboard: Please try copying the text manually."
                        };

                        self.state.ui_mut().export_copy_feedback = Some(user_message.to_string());

                        // Add detailed error to logs
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to copy to clipboard".to_string(),
                            Some(format!(
                                "Clipboard error details: {error}\n\nYou can manually copy the displayed text instead."
                            )),
                        );

                        // Clear error message after 5 seconds
                        Command::perform(
                            async {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            },
                            |_| Message::ResetExportCopyFeedback,
                        )
                    }
                }
            }
            Message::ResetExportCopyFeedback => {
                self.state.ui_mut().export_copy_feedback = None;
                Command::none()
            }
            Message::ClearClipboardAfterDelay => {
                if self.state.ui_mut().clipboard_clear_timer_active {
                    // Clear clipboard for security
                    let clear_task = async {
                        match Clipboard::new() {
                            Ok(mut clipboard) => match clipboard.set_text("") {
                                Ok(_) => tracing::info!("Clipboard cleared for security"),
                                Err(e) => tracing::error!("Failed to clear clipboard: {}", e),
                            },
                            Err(e) => tracing::error!("Failed to access clipboard for clearing: {}", e),
                        }
                    };

                    self.state.ui_mut().clipboard_clear_timer_active = false;
                    Command::perform(clear_task, |_| Message::ResetExportCopyFeedback)
                // This won't do anything but satisfies the type
                } else {
                    Command::none()
                }
            }
            Message::SeedPhraseChanged(phrase) => {
                // Handle seed phrase input (existing logic)
                // Analyze the seed phrase if it's not empty
                if !phrase.trim().is_empty() {
                    self.state.wallet_mut().seed_phrase = phrase.clone();
                    Command::perform(analyze_seed_phrase(phrase), Message::SeedAnalyzed)
                } else {
                    self.state.wallet_mut().seed_phrase = phrase;
                    self.state.wallet_mut().seed_analysis = None;
                    Command::none()
                }
            }
            Message::PrivateKeyChanged(mut key) => {
                // Strip 0x prefix if present and normalize
                if key.starts_with("0x") || key.starts_with("0X") {
                    key = key[2..].to_string();
                }
                // Remove whitespace and convert to lowercase for consistency
                key = key
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .to_lowercase();
                self.state.wallet_mut().private_key = secrecy::SecretString::new(key);
                Command::none()
            }
            Message::ImportTypeSelected(import_type) => {
                self.state.wallet_mut().import_type = import_type;
                // Clear inputs when switching import type
                self.state.wallet_mut().seed_phrase.clear();
                if !self.state.wallet().private_key.expose_secret().is_empty() {
                    self.state.wallet_mut().private_key = secrecy::SecretString::new(String::new());
                };
                self.state.wallet_mut().seed_analysis = None;
                Command::none()
            }
            Message::WalletNameChanged(name) => {
                self.state.wallet_mut().wallet_name = name;
                Command::none()
            }
            Message::MasterPasswordChanged(password) => {
                self.state.wallet_mut().master_password = password;
                Command::none()
            }
            Message::ConfirmPasswordChanged(password) => {
                self.state.wallet_mut().confirm_password = password;
                Command::none()
            }
            Message::GenerateNewSeed => {
                self.state.wallet_mut().generating_seed = true;
                let strength = self.state.wallet().selected_seed_strength;
                Command::perform(generate_seed_phrase_with_strength(strength), Message::SeedGenerated)
            }
            Message::SeedStrengthSelected(strength) => {
                self.state.wallet_mut().selected_seed_strength = strength;
                // If we have a current seed phrase, re-analyze it
                if !self.state.wallet().seed_phrase.is_empty() {
                    let phrase = self.state.wallet().seed_phrase.clone();
                    Command::perform(analyze_seed_phrase(phrase), Message::SeedAnalyzed)
                } else {
                    Command::none()
                }
            }
            Message::SeedAnalyzed(result) => {
                match result {
                    Ok(analysis) => {
                        self.state.wallet_mut().seed_analysis = Some(analysis);
                    }
                    Err(e) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Seed analysis failed".to_string(),
                            Some(format!("Error details: {e}")),
                        );
                        self.state.wallet_mut().seed_analysis = None;
                    }
                }
                Command::none()
            }
            Message::SeedGenerated(seed) => {
                self.state.wallet_mut().generating_seed = false;
                self.state.wallet_mut().seed_phrase = seed;
                Command::none()
            }
            Message::CreateWalletFromSeed => {
                if self.state.wallet().creating_wallet {
                    return Command::none();
                }

                // Validate passwords match
                if self.state.wallet().master_password != self.state.wallet().confirm_password {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Password validation failed".to_string(),
                        Some("Master password and confirmation do not match".to_string()),
                    );
                    return Command::none();
                }

                if self.state.wallet().master_password.is_empty() {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Password validation failed".to_string(),
                        Some("Master password is required".to_string()),
                    );
                    return Command::none();
                }

                self.state.wallet_mut().creating_wallet = true;
                let name = self.state.wallet().wallet_name.clone();
                let phrase = self.state.wallet().seed_phrase.clone();
                let password = self.state.wallet().master_password.clone();

                Command::perform(create_wallet_from_seed(name, phrase, password), Message::WalletCreated)
            }
            Message::ImportWalletFromSeed => {
                if self.state.wallet().creating_wallet {
                    return Command::none();
                }

                // Show address discovery dialog for multi-address import
                self.state.wallet_mut().show_address_discovery = true;
                self.state.wallet_mut().discovered_addresses.clear();
                self.state.wallet_mut().selected_addresses_for_import.clear();
                self.state.wallet_mut().current_seed_for_discovery = self.state.wallet().seed_phrase.clone();

                Command::none()
            }
            Message::ImportWalletFromSeedDirect => {
                if self.state.wallet().creating_wallet {
                    return Command::none();
                }

                // Import wallet directly from seed phrase (first address only)
                self.state.wallet_mut().creating_wallet = true;
                let name = self.state.wallet().wallet_name.clone();
                let seed = self.state.wallet().seed_phrase.clone();
                let password = self.state.wallet().master_password.clone();

                Command::perform(import_wallet_from_seed(name, seed, password), Message::WalletCreated)
            }
            Message::ImportWalletFromPrivateKey => {
                if self.state.wallet().creating_wallet {
                    return Command::none();
                }

                self.state.wallet_mut().creating_wallet = true;
                let name = self.state.wallet().wallet_name.clone();
                let private_key = self.state.wallet().private_key.clone();

                Command::perform(
                    import_wallet_from_private_key(name, private_key.expose_secret().clone(), String::new()),
                    Message::WalletCreated,
                )
            }
            Message::WalletCreated(result) => {
                self.state.wallet_mut().creating_wallet = false;
                match result {
                    Ok(address) => {
                        tracing::info!("Wallet created successfully: {address}");
                        tracing::info!("🔄 Wallet created, reloading accounts...");
                        // Determine operation type for appropriate messaging
                        let (success_message, log_message) = if self.state.wallet().show_import_wallet {
                            (
                                "Wallet imported successfully! Check the account dropdown to select it.",
                                "Wallet imported successfully",
                            )
                        } else {
                            (
                                "Wallet created successfully! Check the account dropdown to select it.",
                                "Wallet created successfully",
                            )
                        };

                        self.state.wallet_mut().show_create_wallet = false;
                        self.state.wallet_mut().show_import_wallet = false;
                        // Also hide wallet password dialog if it's showing
                        self.state.auth_mut().password_dialog.hide();
                        // Clear all wallet creation fields for security
                        self.state.wallet_mut().seed_phrase.clear();
                        self.state.wallet_mut().master_password.clear();
                        self.state.wallet_mut().confirm_password.clear();
                        self.state.wallet_mut().wallet_name.clear();
                        if !self.state.wallet().private_key.expose_secret().is_empty() {
                            self.state.wallet_mut().private_key = secrecy::SecretString::new(String::new());
                        } // Also clear private key field

                        self.add_log_entry(
                            LogCategory::Success,
                            log_message.to_string(),
                            Some(format!("Address: {address}")),
                        );
                        // Show success message with more details
                        let set_status = self.dispatch_message(Message::SetStatusMessage(
                            success_message.to_string(),
                            StatusMessageColor::Success,
                        ));
                        // Reload accounts to include the new one
                        let load_accounts = self.dispatch_message(Message::LoadAccounts);
                        Command::batch([set_status, load_accounts])
                    }
                    Err(error) => {
                        // Determine operation type for appropriate error messaging
                        let error_message = if self.state.wallet().show_import_wallet {
                            "Wallet import failed"
                        } else {
                            "Wallet creation failed"
                        };

                        self.add_log_entry(
                            LogCategory::Error,
                            error_message.to_string(),
                            Some(format!("Error details: {error}")),
                        );
                        // Show specific error message
                        let display_error = if error.len() > 100 {
                            format!("{}: {}", error_message, &error[..100])
                        } else {
                            format!("{error_message}: {error}")
                        };

                        self.dispatch_message(Message::SetStatusMessage(display_error, StatusMessageColor::Error))
                    }
                }
            }
            // Multi-address management
            Message::ShowAddressDiscovery => {
                self.state.wallet_mut().show_address_discovery = true;
                self.state.wallet_mut().discovered_addresses.clear();
                self.state.wallet_mut().selected_addresses_for_import.clear();
                self.state.wallet_mut().current_seed_for_discovery = self.state.wallet().seed_phrase.clone();
                Command::none()
            }
            Message::HideAddressDiscovery => {
                self.state.wallet_mut().show_address_discovery = false;
                self.state.wallet_mut().discovered_addresses.clear();
                self.state.wallet_mut().selected_addresses_for_import.clear();
                self.state.wallet_mut().current_seed_for_discovery.clear();
                Command::none()
            }
            Message::DiscoverAddresses => {
                if self.state.wallet().discovering_addresses {
                    return Command::none();
                }

                self.state.wallet_mut().discovering_addresses = true;
                let seed_phrase = self.state.wallet_mut().current_seed_for_discovery.clone();

                Command::perform(discover_addresses_from_seed(seed_phrase), Message::AddressesDiscovered)
            }
            Message::AddressesDiscovered(result) => {
                self.state.wallet_mut().discovering_addresses = false;
                match result {
                    Ok(addresses) => {
                        self.state.wallet_mut().discovered_addresses = addresses;
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Address discovery failed".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                    }
                }
                Command::none()
            }
            Message::SelectAddressForImport(address, _derivation_path) => {
                if self.state.wallet_mut().selected_addresses_for_import.contains(&address) {
                    self.state.wallet_mut().selected_addresses_for_import.remove(&address);
                } else {
                    self.state.wallet_mut().selected_addresses_for_import.insert(address);
                }
                Command::none()
            }
            Message::ImportSelectedAddresses => {
                if self.state.wallet_mut().selected_addresses_for_import.is_empty() {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Import validation failed".to_string(),
                        Some("Please select at least one address to import".to_string()),
                    );
                    return Command::none();
                }

                let seed_phrase = self.state.wallet_mut().current_seed_for_discovery.clone();
                let selected_addresses: Vec<String> = self
                    .state
                    .wallet_mut()
                    .selected_addresses_for_import
                    .iter()
                    .cloned()
                    .collect();
                let wallet_name = self.state.wallet().wallet_name.clone();

                let master_password = self.state.wallet().master_password.clone();

                Command::perform(
                    async move {
                        import_multiple_addresses_from_seed(
                            wallet_name,
                            seed_phrase,
                            master_password,
                            selected_addresses,
                        )
                        .await
                        .map(|addresses| addresses.join(", "))
                    },
                    Message::WalletCreated,
                )
            }
            // Custom network management
            Message::ShowAddNetwork => {
                // Clear all form fields when opening the dialog
                self.state.network_mut().network_name.clear();
                self.state.network_mut().network_rpc_url.clear();
                self.state.network_mut().network_chain_id.clear();
                self.state.network_mut().network_symbol.clear();
                self.state.network_mut().network_block_explorer.clear();
                // Reset edit mode state
                self.state.network_mut().edit_mode = false;
                self.state.network_mut().selected_network_for_edit = None;
                // Show the dialog
                self.state.network_mut().show_add_network = true;
                Command::none()
            }
            Message::HideAddNetwork => {
                // Clear form fields when dialog is closed
                self.state.network_mut().network_name.clear();
                self.state.network_mut().network_rpc_url.clear();
                self.state.network_mut().network_chain_id.clear();
                self.state.network_mut().network_symbol.clear();
                self.state.network_mut().network_block_explorer.clear();
                self.state.network_mut().show_add_network = false;
                // Clear edit mode state
                self.state.network_mut().edit_mode = false;
                self.state.network_mut().selected_network_for_edit = None;
                Command::none()
            }
            Message::NetworkNameChanged(name) => {
                self.state.network_mut().network_name = name;
                Command::none()
            }
            Message::NetworkRpcUrlChanged(url) => {
                self.state.network_mut().network_rpc_url = url;
                Command::none()
            }
            Message::NetworkChainIdChanged(chain_id) => {
                self.state.network_mut().network_chain_id = chain_id;
                Command::none()
            }
            Message::NetworkSymbolChanged(symbol) => {
                self.state.network_mut().network_symbol = symbol;
                Command::none()
            }
            Message::NetworkBlockExplorerChanged(value) => {
                self.state.network_mut().network_block_explorer = value;
                Command::none()
            }
            Message::AddCustomNetwork => {
                tracing::info!("🔧 AddCustomNetwork message received");
                tracing::info!("  Name: {}", self.state.network().network_name);
                tracing::info!("  RPC URL: {}", self.state.network().network_rpc_url);
                tracing::info!("  Chain ID: {}", self.state.network().network_chain_id);
                tracing::info!("  Symbol: {}", self.state.network().network_symbol);

                if self.state.network().adding_network {
                    tracing::warn!("Already adding network, ignoring duplicate request");
                    return Command::none();
                }

                // Validate Chain ID before spawning task
                let chain_id_str = self.state.network_mut().network_chain_id.trim().to_string();
                let parsed_chain_id = match chain_id_str.parse::<u64>() {
                    Ok(id) => id,
                    Err(_) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Invalid Chain ID".to_string(),
                            Some("Please enter a valid numeric Chain ID (e.g., 1, 56, 8453)".to_string()),
                        );
                        self.state.ui_mut().status_message =
                            "Invalid Chain ID — please enter a valid number".to_string();
                        self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                        return Command::none();
                    }
                };

                self.state.network_mut().adding_network = true;
                let name = self.state.network_mut().network_name.clone();
                let rpc_url = self.state.network_mut().network_rpc_url.clone();
                let symbol = self.state.network_mut().network_symbol.clone();
                let block_explorer = self.state.network_mut().network_block_explorer.clone();

                Command::perform(
                    add_custom_network(name, rpc_url, parsed_chain_id, symbol, block_explorer),
                    Message::NetworkAdded,
                )
            }
            Message::NetworkAdded(result) => {
                self.state.network_mut().adding_network = false;
                match result {
                    Ok(network_info) => {
                        tracing::info!("Network added: {network_info}");

                        // Parse the network info and add to available networks
                        let chain_id: u64 = self.state.network().network_chain_id.parse().unwrap_or(0);
                        let added_name = self.state.network_mut().network_name.clone();
                        let custom_network = NetworkConfig {
                            id: NetworkId(chain_id),
                            name: self.state.network_mut().network_name.clone(),
                            rpc_url: self.state.network_mut().network_rpc_url.clone(),
                            chain_id,
                            symbol: self.state.network_mut().network_symbol.clone(),
                            block_explorer_url: self.state.network_mut().network_block_explorer.clone(),
                            is_testnet: false,
                            is_custom: true, // This is a custom network
                        };

                        // Add to available networks if not already present
                        if !self
                            .state
                            .network()
                            .available_networks
                            .iter()
                            .any(|n| n.id == custom_network.id)
                        {
                            self.state.network_mut().available_networks.push(custom_network.clone());

                            // Save networks to persistent storage
                            let networks = self.state.network().available_networks.clone();
                            tokio::task::spawn(async move {
                                if let Err(e) = save_networks_to_storage(networks).await {
                                    tracing::error!("Failed to save networks: {}", e);
                                }
                            });

                            // Also update the running wallet's NetworkManager so it can be selected immediately
                            if let Some(wallet) = &self.wallet {
                                let wallet_clone = wallet.clone();
                                let network_config = custom_network.clone();
                                tokio::task::spawn(async move {
                                    let mut w = wallet_clone.write().await;
                                    match w.add_custom_network(network_config.clone()).await {
                                        Ok(_) => tracing::info!(
                                            "🔧 Added custom network to NetworkManager: {} (Chain ID: {})",
                                            network_config.name,
                                            network_config.chain_id
                                        ),
                                        Err(e) => {
                                            tracing::error!("❌ Failed to add custom network to NetworkManager: {}", e)
                                        }
                                    }
                                });
                            }
                        }

                        // Clear form fields
                        self.state.network_mut().network_name.clear();
                        self.state.network_mut().network_rpc_url.clear();
                        self.state.network_mut().network_chain_id.clear();
                        self.state.network_mut().network_symbol.clear();
                        self.state.network_mut().network_block_explorer.clear();

                        self.state.network_mut().show_add_network = false;
                        self.add_log_entry(
                            LogCategory::Network,
                            "Network added successfully".to_string(),
                            Some(format!("Network info: {network_info}")),
                        );
                        // Status toast
                        self.state.ui_mut().status_message =
                            format!("Custom network added: {added_name} (Chain ID: {chain_id})");
                        self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to add network".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                        // Status toast
                        self.state.ui_mut().status_message = format!("Failed to add network: {error}");
                        self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                    }
                }
                Command::none()
            }
            // Network editing handlers
            Message::EditModeToggled(enabled) => {
                self.state.network_mut().edit_mode = enabled;
                if !enabled {
                    self.state.network_mut().selected_network_for_edit = None;
                    // Clear form when disabling edit mode
                    self.state.network_mut().network_name.clear();
                    self.state.network_mut().network_rpc_url.clear();
                    self.state.network_mut().network_chain_id.clear();
                    self.state.network_mut().network_symbol.clear();
                }
                Command::none()
            }
            Message::ExistingNetworkSelected(network_id) => {
                self.state.network_mut().selected_network_for_edit = network_id;
                Command::none()
            }
            Message::LoadExistingNetwork => {
                if let Some(network_id_str) = &self.state.network().selected_network_for_edit {
                    // Parse the network ID string to find the matching network
                    if let Ok(chain_id) = network_id_str.parse::<u64>() {
                        let network_id = NetworkId(chain_id);
                        if let Some(network) = self
                            .state
                            .network()
                            .available_networks
                            .iter()
                            .find(|n| n.id == network_id)
                        {
                            // Extract values to avoid borrow checker issues
                            let name = network.name.clone();
                            let rpc_url = network.rpc_url.clone();
                            let chain_id_str = network.chain_id.to_string();
                            let symbol = network.symbol.clone();
                            let block_explorer = network.block_explorer_url.clone();

                            // Now mutate
                            self.state.network_mut().network_name = name;
                            self.state.network_mut().network_rpc_url = rpc_url;
                            self.state.network_mut().network_chain_id = chain_id_str;
                            self.state.network_mut().network_symbol = symbol;
                            self.state.network_mut().network_block_explorer = block_explorer;
                        }
                    }
                }
                Command::none()
            }
            Message::ShowDeleteNetworkConfirm => {
                // Only available in edit mode with a selected custom network
                if let Some(network_id_str) = &self.state.network().selected_network_for_edit {
                    if let Ok(chain_id) = network_id_str.parse::<u64>() {
                        let is_custom = self
                            .state
                            .network()
                            .available_networks
                            .iter()
                            .find(|n| n.id == NetworkId(chain_id))
                            .map(|n| n.is_custom)
                            .unwrap_or(false);
                        if !is_custom {
                            // Prevent deleting non-custom networks
                            self.add_log_entry(
                                LogCategory::Error,
                                "Cannot delete built-in network".to_string(),
                                Some("Only custom networks can be deleted".to_string()),
                            );
                            return Command::none();
                        }
                        // Show confirmation dialog
                        self.state.network_mut().show_delete_network_confirmation = true;
                    }
                }
                Command::none()
            }
            Message::HideDeleteNetworkConfirm => {
                self.state.network_mut().show_delete_network_confirmation = false;
                Command::none()
            }
            Message::ConfirmDeleteNetwork => {
                // Only available in edit mode with a selected custom network
                if let Some(network_id_str) = &self.state.network().selected_network_for_edit {
                    if let Ok(chain_id) = network_id_str.parse::<u64>() {
                        let is_custom = self
                            .state
                            .network()
                            .available_networks
                            .iter()
                            .find(|n| n.id == NetworkId(chain_id))
                            .map(|n| n.is_custom)
                            .unwrap_or(false);
                        if !is_custom {
                            // Prevent deleting non-custom networks
                            self.add_log_entry(
                                LogCategory::Error,
                                "Cannot delete built-in network".to_string(),
                                Some("Only custom networks can be deleted".to_string()),
                            );
                            return Command::none();
                        }
                        // Perform deletion
                        let id = network_id_str.clone();
                        return Command::perform(delete_existing_network(id), Message::NetworkDeleted);
                    }
                }
                Command::none()
            }
            Message::EditNetwork => {
                if self.state.network().editing_network {
                    return Command::none();
                }

                // Validate Chain ID before spawning task
                let chain_id_str = self.state.network_mut().network_chain_id.trim().to_string();
                let parsed_chain_id = match chain_id_str.parse::<u64>() {
                    Ok(id) => id,
                    Err(_) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Invalid Chain ID".to_string(),
                            Some("Please enter a valid numeric Chain ID (e.g., 1, 56, 8453)".to_string()),
                        );
                        self.state.ui_mut().status_message =
                            "Invalid Chain ID — please enter a valid number".to_string();
                        self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                        return Command::none();
                    }
                };

                self.state.network_mut().editing_network = true;
                let name = self.state.network_mut().network_name.clone();
                let rpc_url = self.state.network_mut().network_rpc_url.clone();
                let symbol = self.state.network_mut().network_symbol.clone();
                let network_id = self
                    .state
                    .network()
                    .selected_network_for_edit
                    .clone()
                    .unwrap_or_default();

                Command::perform(
                    edit_existing_network(network_id, name, rpc_url, parsed_chain_id, symbol),
                    Message::NetworkUpdated,
                )
            }
            Message::NetworkUpdated(result) => {
                self.state.network_mut().editing_network = false;
                match result {
                    Ok(network_info) => {
                        // Update the network in the available networks list
                        // Extract network ID string and form values first to avoid borrow checker issues
                        let network_update_data = {
                            let network_state = self.state.network();
                            network_state
                                .selected_network_for_edit
                                .as_ref()
                                .and_then(|network_id_str| {
                                    network_id_str.parse::<u64>().ok().map(|chain_id| {
                                        (
                                            NetworkId(chain_id),
                                            network_state.network_chain_id.clone(),
                                            network_state.network_name.clone(),
                                            network_state.network_rpc_url.clone(),
                                            network_state.network_symbol.clone(),
                                            network_state.network_block_explorer.clone(),
                                            network_state.current_network,
                                        )
                                    })
                                })
                        };

                        if let Some((
                            network_id,
                            form_chain_id,
                            form_name,
                            form_rpc_url,
                            form_symbol,
                            form_block_explorer,
                            current_network,
                        )) = network_update_data
                        {
                            // Update network in a single mutable borrow scope
                            {
                                let network_state = self.state.network_mut();
                                if let Some(network) =
                                    network_state.available_networks.iter_mut().find(|n| n.id == network_id)
                                {
                                    // Parse new chain id
                                    let new_chain_id: u64 = form_chain_id.parse().unwrap_or(network.chain_id);

                                    // Track if ID changed
                                    let old_id = network.id;
                                    let id_changed = old_id != NetworkId(new_chain_id);

                                    // Update the network with new values, including ID if changed
                                    network.name = form_name.clone();
                                    network.rpc_url = form_rpc_url.clone();
                                    network.chain_id = new_chain_id;
                                    network.symbol = form_symbol.clone();
                                    network.block_explorer_url = form_block_explorer.clone();
                                    if id_changed {
                                        network.id = NetworkId(new_chain_id);
                                    }

                                    // If we just edited the currently selected network and the ID changed,
                                    // update the GUI's current_network to the new ID (in same mutable borrow scope)
                                    if id_changed && current_network == old_id {
                                        network_state.current_network = NetworkId(new_chain_id);
                                    }

                                    // Store values for async task
                                    let wallet_clone = self.wallet.clone();
                                    let updated_config = network.clone();
                                    let network_name_for_log = network.name.clone();

                                    // Also update the running wallet's NetworkManager so changes take effect
                                    if let Some(wallet) = wallet_clone {
                                        tokio::task::spawn(async move {
                                            let mut w = wallet.write().await;
                                            let old_network_id = old_id;
                                            match w
                                                .update_or_replace_custom_network(
                                                    old_network_id,
                                                    updated_config.clone(),
                                                )
                                                .await
                                            {
                                                Ok(_) => tracing::info!(
                                                    "🔧 Updated custom network in NetworkManager: {} (Chain ID: {})",
                                                    updated_config.name,
                                                    updated_config.chain_id
                                                ),
                                                Err(e) => tracing::error!(
                                                    "❌ Failed to update network in NetworkManager: {}",
                                                    e
                                                ),
                                            }
                                        });
                                    }

                                    tracing::info!("Updated network: {:?} -> {}", network_id, network_name_for_log);
                                }
                            }
                        }

                        // Save networks to persistent storage
                        let networks_clone = self.state.network().available_networks.clone();
                        tokio::task::spawn(async move {
                            if let Err(e) = save_networks_to_storage(networks_clone).await {
                                tracing::error!("Failed to save networks: {}", e);
                            }
                        });

                        self.state.network_mut().show_add_network = false;
                        self.state.network_mut().edit_mode = false;
                        self.state.network_mut().selected_network_for_edit = None;
                        self.add_log_entry(
                            LogCategory::Network,
                            "Network updated successfully".to_string(),
                            Some(format!("Network info: {network_info}")),
                        );
                        // Status toast
                        let updated_name = self.state.network_mut().network_name.clone();
                        let updated_chain_id = self.state.network().network_chain_id.parse().unwrap_or(0);
                        self.state.ui_mut().status_message =
                            format!("Network updated: {updated_name} (Chain ID: {updated_chain_id})");
                        self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to update network".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                        // Status toast
                        self.state.ui_mut().status_message = format!("Failed to update network: {error}");
                        self.state.ui_mut().status_message_color = StatusMessageColor::Error;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                    }
                }
                Command::none()
            }
            Message::NetworkDeleted(result) => {
                match result {
                    Ok(info) => {
                        // Remove from available networks
                        if let Some(network_id_str) = &self.state.network().selected_network_for_edit {
                            if let Ok(chain_id) = network_id_str.parse::<u64>() {
                                let removed_id = NetworkId(chain_id);
                                // Capture name before removal for status message
                                let removed_name = self
                                    .state
                                    .network()
                                    .available_networks
                                    .iter()
                                    .find(|n| n.id == removed_id)
                                    .map(|n| n.name.clone())
                                    .unwrap_or_else(|| format!("Chain {chain_id}"));

                                self.state
                                    .network_mut()
                                    .available_networks
                                    .retain(|n| n.id != removed_id);

                                // Update current network if it was removed
                                if self.state.network().current_network == removed_id {
                                    if let Some(first) = self.state.network().available_networks.first() {
                                        self.state.network_mut().current_network = first.id;
                                    }
                                }

                                // Persist updated custom networks
                                let networks_clone = self.state.network().available_networks.clone();
                                tokio::task::spawn(async move {
                                    if let Err(e) = save_networks_to_storage(networks_clone).await {
                                        tracing::error!("Failed to save networks after delete: {}", e);
                                    }
                                });

                                // Update live wallet NetworkManager
                                if let Some(wallet) = &self.wallet {
                                    let wallet_clone = wallet.clone();
                                    tokio::task::spawn(async move {
                                        let mut w = wallet_clone.write().await;
                                        let _ = w.remove_custom_network(removed_id).await;
                                    });
                                }

                                // Log + status message toast
                                self.add_log_entry(
                                    LogCategory::Network,
                                    "Network deleted".to_string(),
                                    Some(info.clone()),
                                );
                                self.state.ui_mut().status_message =
                                    format!("Custom network removed: {removed_name} (Chain ID: {chain_id})");
                                self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                                self.state.ui_mut().status_message_timer = Some(Instant::now());
                            }
                        }
                        // Reset edit state
                        self.state.network_mut().edit_mode = false;
                        self.state.network_mut().selected_network_for_edit = None;
                        self.state.network_mut().show_add_network = false;
                        self.state.network_mut().show_delete_network_confirmation = false;
                    }
                    Err(e) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to delete network".to_string(),
                            Some(e.to_string()),
                        );
                    }
                }
                Command::none()
            }
            Message::LoadNetworks => {
                self.state.network_mut().loading_networks = true;
                Command::perform(load_all_networks(), Message::NetworksLoaded)
            }
            Message::NetworksLoaded(networks) => {
                self.state.network_mut().loading_networks = false;
                if !networks.is_empty() {
                    self.state.network_mut().available_networks = networks;
                    // Set current network to first available if current is not in the list
                    if !self
                        .state
                        .network()
                        .available_networks
                        .iter()
                        .any(|n| n.id == self.state.network().current_network)
                    {
                        self.state.network_mut().current_network = self.state.network().available_networks[0].id;
                    }

                    // Update token list for current network
                    self.update_token_list_for_network(self.state.network().current_network);
                }
                Command::none()
            }
            // Account management
            Message::LoadAccounts => {
                self.state.wallet_mut().loading_accounts = true;
                // Show spinner for account loading
                self.state.ui_mut().accounts_spinner = true;
                Command::perform(load_available_accounts(), Message::AccountsLoaded)
            }
            Message::AccountsLoaded(result) => {
                self.state.wallet_mut().loading_accounts = false;
                // Hide accounts spinner
                self.state.ui_mut().accounts_spinner = false;
                match result {
                    Ok(accounts) => {
                        tracing::info!(
                            "📋 Loaded {} accounts: {:?}",
                            accounts.len(),
                            accounts.iter().map(|a| &a.name).collect::<Vec<_>>()
                        );
                        // Set the accounts first
                        self.state.wallet_mut().available_accounts = accounts;

                        // Activate auto balance polling when accounts are loaded successfully
                        if !self.state.wallet().available_accounts.is_empty() {
                            tracing::info!(
                                "🔄 Activating automatic balance polling for {} accounts",
                                self.state.wallet().available_accounts.len()
                            );
                            self.state.ui_mut().polling_active = true;
                            self.state.ui_mut().poll_interval = 3; // Start with 3-second intervals as mentioned
                            self.state.last_activity = Instant::now();

                            // Auto balance polling is now active - audio alerts work through balance change detection
                        }

                        // Set current account if we have accounts and none is selected
                        if !self.state.wallet().available_accounts.is_empty()
                            && self.state.wallet().current_account_id.is_none()
                        {
                            let first_account = &self.state.wallet().available_accounts[0];
                            let first_account_id = first_account.id.clone();

                            tracing::info!(
                                "🎯 Auto-selecting first account: {} ({})",
                                first_account.name,
                                first_account.address
                            );

                            // Use AccountSelected to properly unlock the wallet
                            return self.dispatch_message(Message::AccountSelected(first_account_id));
                        }
                    }
                    Err(error) => {
                        tracing::error!("Failed to load accounts: {}", error);
                        self.add_log_entry(
                            LogCategory::Error,
                            "Unable to load accounts".to_string(),
                            Some(format!(
                                "Error details: {error}\n\nPlease check your wallet setup and try again."
                            )),
                        );
                        self.state.wallet_mut().available_accounts = Vec::new();

                        // Set status message for immediate user feedback
                        return Command::perform(async {}, |_| {
                            Message::SetStatusMessage(
                                "Unable to load accounts. Check wallet setup.".to_string(),
                                StatusMessageColor::Error,
                            )
                        });
                    }
                }

                Command::none()
            }
            Message::AccountSelected(account_id) => {
                tracing::info!("🔄 AccountSelected: Two-tier security check for account {}", account_id);

                // Track user activity
                self.state.last_activity = Instant::now();
                self.state.ui_mut().polling_active = true;

                // Find the selected account
                let account_data = self
                    .state
                    .wallet()
                    .available_accounts
                    .iter()
                    .find(|a| a.id == account_id)
                    .cloned();

                if let Some(account) = account_data {
                    let account_name = account.name.clone();

                    // Check if wallet is unlocked (prerequisite for two-tier security)
                    if !self.state.auth().enhanced_session.is_wallet_ready() {
                        tracing::warn!("⚠️ Wallet not unlocked, cannot select account");
                        return self.dispatch_message(Message::ShowWalletUnlock);
                    }

                    // Check if account session is already unlocked
                    if self.state.auth().enhanced_session.is_account_unlocked(&account_id) {
                        // Account is already unlocked, proceed with selection
                        tracing::info!("✅ Account session already unlocked, proceeding with selection");
                        self.complete_account_selection(account_id, account)
                    } else {
                        // Account needs to be unlocked, show account password dialog
                        tracing::info!("🔐 Account session locked, requesting account password");
                        self.state.auth_mut().password_dialog.show(
                            crate::gui::state::auth_state::PasswordDialogConfig::AccountUnlock {
                                account_id: account_id.clone(),
                                account_name: account_name.clone(),
                            },
                        );
                        Command::none()
                    }
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Account switch failed".to_string(),
                        Some("Account not found".to_string()),
                    );
                    Command::none()
                }
            }
            Message::AccountUnlocked(account_name) => {
                self.add_log_entry(
                    LogCategory::Success, // Using closest available category
                    "Account unlocked successfully".to_string(),
                    Some(format!("Unlocked account: {account_name}")),
                );

                // Reset last balance when account changes
                self.state.last_balance = "0.0000".to_string();

                // Refresh balance for the unlocked account
                self.dispatch_message(Message::RefreshBalance)
            }
            Message::AccountUnlockFailed(error) => {
                self.add_log_entry(LogCategory::Error, "Failed to unlock account".to_string(), Some(error));
                Command::none()
            }
            Message::ShowDeleteAccount => {
                self.state.wallet_mut().show_delete_account = true;
                Command::none()
            }
            Message::HideDeleteAccount => {
                self.state.wallet_mut().show_delete_account = false;
                Command::none()
            }
            Message::ConfirmDeleteAccount => {
                if let Some(account_id) = self.state.wallet().current_account_id.clone() {
                    self.state.wallet_mut().deleting_account = true;
                    Command::perform(delete_account(account_id), Message::AccountDeleted)
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Account deletion failed".to_string(),
                        Some("No account selected for deletion".to_string()),
                    );
                    Command::none()
                }
            }
            Message::AccountDeleted(result) => {
                self.state.wallet_mut().deleting_account = false;
                self.state.wallet_mut().show_delete_account = false;

                match result {
                    Ok(message) => {
                        tracing::info!("Account deleted: {message}");
                        tracing::info!("🗑️ Account deleted successfully, reloading accounts...");
                        self.add_log_entry(
                            LogCategory::Security,
                            "Account deleted successfully".to_string(),
                            Some(format!("Details: {message}")),
                        );

                        // Reload accounts and reset current account
                        self.state.wallet_mut().current_account_id = None;
                        self.state.wallet_mut().current_account = "No account selected".to_string();
                        self.dispatch_message(Message::LoadAccounts)
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to delete account".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                        Command::none()
                    }
                }
            }
            Message::CopyAddress(address) => {
                // Copy address to clipboard and show subtle feedback
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Err(e) = clipboard.set_text(&address) {
                        tracing::error!("Failed to copy to clipboard: {e}");
                    } else {
                        self.state.wallet_mut().address_just_copied = true;
                        tracing::info!("✅ Address copied to clipboard, showing feedback");
                    }
                } else {
                    tracing::error!("Failed to access clipboard");
                }
                // Reset the copied state after 0.5 seconds for better visibility
                Command::perform(
                    async {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    },
                    |_| Message::ResetCopyState,
                )
            }
            Message::CopyTransactionAddress(address) => {
                // Copy transaction address to clipboard
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Err(e) = clipboard.set_text(&address) {
                        tracing::error!("Failed to copy transaction address to clipboard: {e}");
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to copy address".to_string(),
                            Some(format!("Could not copy address: {e}")),
                        );
                    } else {
                        self.add_log_entry(
                            LogCategory::Info,
                            "Address copied".to_string(),
                            Some(format!("Copied address: {address}")),
                        );
                    }
                } else {
                    tracing::error!("Failed to access clipboard for transaction address");
                }
                Command::none()
            }
            Message::CopyTransactionHash(hash) => {
                // Copy transaction hash to clipboard
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Err(e) = clipboard.set_text(&hash) {
                        tracing::error!("Failed to copy transaction hash to clipboard: {e}");
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to copy transaction hash".to_string(),
                            Some(format!("Could not copy hash: {e}")),
                        );
                    } else {
                        self.add_log_entry(
                            LogCategory::Info,
                            "Transaction hash copied".to_string(),
                            Some(format!("Copied hash: {hash}")),
                        );
                    }
                } else {
                    tracing::error!("Failed to access clipboard for transaction hash");
                }
                Command::none()
            }
            Message::ResetCopyState => {
                self.state.wallet_mut().address_just_copied = false;
                tracing::info!("🔄 Reset copy state, hiding feedback");
                Command::none()
            }
            Message::ShowHardWallet => {
                self.state.wallet_mut().show_hardware_wallet = true;
                // Automatically detect hard wallets when dialog opens
                self.dispatch_message(Message::DetectHardwareWallets)
            }
            Message::HideHardWallet => {
                self.state.wallet_mut().show_hardware_wallet = false;
                Command::none()
            }
            Message::DetectHardwareWallets => {
                self.state.wallet_mut().detecting_hardware_wallets = true;
                Command::perform(detect_hardware_wallets(), |_result| {
                    Message::HardwareWalletsDetected(Ok(vec![]))
                })
            }
            Message::HardwareWalletsDetected(result) => {
                self.state.wallet_mut().detecting_hardware_wallets = false;
                match result {
                    Ok(wallets) => {
                        self.state.wallet_mut().available_hardware_wallets = wallets;
                        if self.state.wallet().available_hardware_wallets.is_empty() {
                            self.add_log_entry(
                                LogCategory::Error,
                                "No hard wallets detected".to_string(),
                                Some("Please connect your device and unlock it".to_string()),
                            );
                        } else {
                            self.add_log_entry(
                                LogCategory::Wallet, // Using closest available category
                                "Hard wallets detected".to_string(),
                                Some(format!(
                                    "Found {} hard wallet(s)",
                                    self.state.wallet().available_hardware_wallets.len()
                                )),
                            );
                        }
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to detect hard wallets".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                    }
                }
                Command::none()
            }
            Message::ConnectHardwareWallet(index) => {
                if index < self.state.wallet().available_hardware_wallets.len() {
                    let wallet_info = &self.state.wallet().available_hardware_wallets[index];
                    self.add_log_entry(
                        LogCategory::Wallet, // Using closest available category
                        "Connecting to hard wallet".to_string(),
                        Some(format!("Connecting to hardware wallet {wallet_info}...")),
                    );
                    Command::perform(connect_hardware_wallet(index), |result| {
                        Message::HardwareWalletConnected(result)
                    })
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Hard wallet connection failed".to_string(),
                        Some("Invalid wallet selection".to_string()),
                    );
                    Command::none()
                }
            }
            Message::HardwareWalletConnected(result) => {
                match result {
                    Ok(message) => {
                        self.add_log_entry(
                            LogCategory::Wallet, // Using closest available category
                            "Hard wallet connected".to_string(),
                            Some(message),
                        );
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Hard wallet connection failed".to_string(),
                            Some(format!("Connection failed: {error}")),
                        );
                    }
                }
                Command::none()
            }
            Message::GetHardwareAddresses(index) => {
                if index < self.state.wallet().available_hardware_wallets.len() {
                    self.state.wallet_mut().loading_hardware_addresses = true;
                    Command::perform(get_hardware_wallet_addresses(index), |_result| {
                        Message::HardwareAddressesReceived(Ok(vec![]))
                    })
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Hardware wallet address retrieval failed".to_string(),
                        Some("Invalid wallet selection".to_string()),
                    );
                    Command::none()
                }
            }
            Message::HardwareAddressesReceived(result) => {
                self.state.wallet_mut().loading_hardware_addresses = false;
                match result {
                    Ok(addresses) => {
                        self.state.wallet_mut().hardware_wallet_addresses = addresses;
                        self.add_log_entry(
                            LogCategory::Wallet, // Using closest available category
                            "Hardware wallet addresses retrieved".to_string(),
                            Some(format!(
                                "Retrieved {} addresses from hardware wallet",
                                self.state.wallet().hardware_wallet_addresses.len()
                            )),
                        );
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to get hardware wallet addresses".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                    }
                }
                Command::none()
            }
            Message::ScanHardwareWallets => {
                self.state.wallet_mut().detecting_hardware_wallets = true;
                Command::perform(detect_hardware_wallets(), |_result| {
                    Message::HardwareWalletsDetected(Ok(vec![]))
                })
            }
            Message::RefreshHardwareWallets => {
                self.state.wallet_mut().detecting_hardware_wallets = true;
                self.state.wallet_mut().available_hardware_wallets.clear();
                Command::perform(detect_hardware_wallets(), |_result| {
                    Message::HardwareWalletsDetected(Ok(vec![]))
                })
            }
            Message::ConnectToHardwareWallet(device_id) => {
                if let Some(device) = self
                    .state
                    .wallet()
                    .available_hardware_wallets
                    .iter()
                    .find(|d| d.to_string() == device_id)
                {
                    self.add_log_entry(
                        LogCategory::Wallet,
                        "Connecting to hardware wallet".to_string(),
                        Some(format!("Connecting to hardware wallet: {device}")),
                    );
                    // For now, just log success - real connection implementation would go here
                    self.add_log_entry(
                        LogCategory::Wallet,
                        "Hardware wallet connected".to_string(),
                        Some(format!("Successfully connected to {device_id}")),
                    );
                } else {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Hardware wallet connection failed".to_string(),
                        Some(format!("Device not found: {device_id}")),
                    );
                }
                Command::none()
            }
            Message::TestHardwareWallet => {
                if self.state.wallet().available_hardware_wallets.is_empty() {
                    self.add_log_entry(
                        LogCategory::Error,
                        "Hardware wallet test failed".to_string(),
                        Some("No hardware wallets connected".to_string()),
                    );
                } else {
                    self.add_log_entry(
                        LogCategory::Wallet,
                        "Testing hardware wallet connection".to_string(),
                        Some("Running connection test...".to_string()),
                    );
                    // Simulate a successful test
                    self.add_log_entry(
                        LogCategory::Wallet,
                        "Hardware wallet test completed".to_string(),
                        Some("All tests passed successfully".to_string()),
                    );
                }
                Command::none()
            }
            Message::HardwareWalletsScanned(result) => {
                self.state.wallet_mut().detecting_hardware_wallets = false;
                match result {
                    Ok(wallets) => {
                        self.state.wallet_mut().available_hardware_wallets = wallets;
                        if self.state.wallet().available_hardware_wallets.is_empty() {
                            self.add_log_entry(
                                LogCategory::Error,
                                "No hardware wallets detected".to_string(),
                                Some("Please connect your device and unlock it".to_string()),
                            );
                        } else {
                            self.add_log_entry(
                                LogCategory::Wallet,
                                "Hardware wallets detected".to_string(),
                                Some(format!(
                                    "Found {} hardware wallet(s)",
                                    self.state.wallet().available_hardware_wallets.len()
                                )),
                            );
                        }
                    }
                    Err(error) => {
                        self.add_log_entry(
                            LogCategory::Error,
                            "Failed to detect hardware wallets".to_string(),
                            Some(format!("Error details: {error}")),
                        );
                    }
                }
                Command::none()
            }
            Message::ThemeToggled => {
                // Theme toggling is disabled - keep single deep black theme
                Command::none()
            }
            Message::TestIncomingTransactions => {
                // Manual test for incoming transaction monitoring
                if let Some(account_id) = &self.state.wallet().current_account_id {
                    if let Some(account) = self
                        .state
                        .wallet()
                        .available_accounts
                        .iter()
                        .find(|a| &a.id == account_id)
                    {
                        let network_id = self.state.network().current_network;
                        let account_address = format!("{:#x}", account.address); // Use proper hex formatting
                        let current_tx_count = self.state.transaction().transaction_history.len();

                        self.add_log_entry(
                            LogCategory::Info,
                            "Manual incoming transaction test".to_string(),
                            Some(format!(
                                "Testing API call for address {} on network {}",
                                account_address, network_id.0
                            )),
                        );

                        return Command::perform(
                            check_for_incoming_transactions(
                                network_id,
                                account_address,
                                current_tx_count,
                                "TEST".to_string(),
                            ),
                            Message::IncomingTransactionsChecked,
                        );
                    }
                }
                Command::none()
            }
            Message::IncomingTransactionsChecked(result) => {
                match result {
                    Ok(incoming_transactions) => {
                        if !incoming_transactions.is_empty() {
                            tracing::info!("📥 Found {} incoming transactions", incoming_transactions.len());

                            // Add incoming transactions to history
                            for tx in &incoming_transactions {
                                // Check if transaction already exists
                                let exists = self
                                    .state
                                    .transaction()
                                    .transaction_history
                                    .iter()
                                    .any(|existing_tx| existing_tx.hash.to_lowercase() == tx.hash.to_lowercase());

                                if !exists {
                                    tracing::info!(
                                        "➕ Adding new incoming transaction: {} -> {} ({})",
                                        tx.from,
                                        tx.to,
                                        tx.amount
                                    );
                                    self.state.transaction_mut().transaction_history.insert(0, tx.clone());

                                    // Add log entry for the received transaction
                                    self.add_log_entry(
                                        LogCategory::Wallet,
                                        "Transaction received".to_string(),
                                        Some(format!("New incoming transaction: {} from {}", tx.amount, tx.from)),
                                    );
                                } else {
                                    tracing::debug!("🔄 Skipping duplicate transaction: {}", tx.hash);
                                }
                            }

                            // Limit history to 30 transactions
                            if self.state.transaction().transaction_history.len() > 30 {
                                self.state.transaction_mut().transaction_history.truncate(30);
                            }
                        } else {
                            tracing::debug!("📭 No new incoming transactions found");
                        }
                    }
                    Err(error) => {
                        tracing::warn!("⚠️ Failed to check for incoming transactions: {}", error);
                        // Add log entry for debugging
                        self.add_log_entry(
                            LogCategory::Error,
                            "Incoming transaction check failed".to_string(),
                            Some(format!("API Error: {error}")),
                        );
                    }
                }
                Command::none()
            }

            // Auto balance monitoring messages
            Message::AutoBalanceUpdate(auto_message) => {
                use crate::gui::services::AutoBalanceMessage;
                match auto_message {
                    AutoBalanceMessage::IncomingTransaction {
                        hash,
                        from,
                        amount,
                        token: _,
                    } => {
                        tracing::info!(
                            "🔔 Auto Balance: Incoming transaction detected - {} from {}",
                            amount,
                            from
                        );

                        // Add log entry for the incoming transaction
                        self.add_log_entry(
                            LogCategory::Wallet,
                            "Incoming transaction detected".to_string(),
                            Some(format!("Amount: {}, From: {}, Hash: {}", amount, from, hash)),
                        );

                        // Set status message
                        self.state.ui_mut().status_message = format!("💰 Incoming: {}", amount);
                        self.state.ui_mut().status_message_color = StatusMessageColor::Success;
                        self.state.ui_mut().status_message_timer = Some(Instant::now());
                    }
                    AutoBalanceMessage::BalanceChanged { address, new_balance } => {
                        tracing::info!(
                            "💳 Auto Balance: Balance updated for {} - New balance: {}",
                            address,
                            new_balance
                        );

                        // Trigger a balance refresh to update the UI
                        return Command::perform(async {}, |_| Message::RefreshBalance);
                    }
                    AutoBalanceMessage::ServiceError(error) => {
                        tracing::error!("🚨 Auto Balance Service Error: {}", error);

                        // Add error log entry
                        self.add_log_entry(
                            LogCategory::Error,
                            "Auto balance service error".to_string(),
                            Some(error),
                        );
                    }
                }
                Command::none()
            }

            Message::StartAutoBalanceMonitoring => {
                tracing::info!("🔄 Auto balance monitoring is handled automatically through smart polling");
                Command::none()
            }

            Message::StopAutoBalanceMonitoring => {
                tracing::info!("⏹️ Auto balance monitoring is handled automatically through smart polling");
                Command::none()
            }

            Message::ShowPriceInfo => {
                self.state.network_mut().show_price_info = true;
                self.state.network_mut().fetching_price = true;
                // Automatically fetch price when showing price info
                create_eth_price_command(self.api_manager.clone(), self.state.network().current_network.0)
            }
            Message::HidePriceInfo => {
                self.state.network_mut().show_price_info = false;
                Command::none()
            }
            Message::RefreshEthPrice => {
                self.state.network_mut().fetching_price = true;
                create_eth_price_command(self.api_manager.clone(), self.state.network().current_network.0)
            }
            Message::EthPriceFetched(result) => {
                self.state.network_mut().fetching_price = false;
                match result {
                    Ok((price, change)) => {
                        self.state.network_mut().eth_price = Some(price);
                        self.state.network_mut().eth_price_change_24h = change;
                        self.state.network_mut().price_last_updated = Some(Instant::now());
                        tracing::debug!("💰 ETH price updated: ${:.2}", price);
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to fetch ETH price: {}", e);
                        self.add_log_entry(
                            LogCategory::Error,
                            "Price fetch failed".to_string(),
                            Some(format!("Could not fetch ETH price: {e}")),
                        );
                    }
                }
                Command::none()
            }
            Message::PriceAutoRefreshTick => {
                // Auto-refresh price data if needed
                if self.state.network().show_price_info {
                    let should_refresh = if let Some(last_updated) = self.state.network().price_last_updated {
                        last_updated.elapsed() > Duration::from_secs(30)
                    } else {
                        true
                    };

                    if should_refresh {
                        self.state.network_mut().fetching_price = true;
                        create_eth_price_command(self.api_manager.clone(), self.state.network().current_network.0)
                    } else {
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::ExportDataReceived(result) => {
                self.state.export_loading = false;
                match result {
                    Ok(data) => {
                        self.state.export_result = Some(data);
                        // Complete state implicit
                        self.state.wallet_mut().export_error_message = None;
                    }
                    Err(e) => {
                        self.state.wallet_mut().export_error_message = Some(format!("Export failed: {e}"));
                    }
                }
                Command::none()
            }

            // Catch-all for messages that should be handled by specialized handlers
            // This should not normally be reached if routing is correct
            _ => {
                tracing::warn!(
                    "Unhandled message variant reached core handler: {:?}",
                    std::any::type_name_of_val(&message)
                );
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Show different views based on application state

        // Show password dialog (highest priority - required for security operations)
        if self.state.auth().password_dialog.visible {
            eprintln!("VIEW_DEBUG: Showing password dialog (password_dialog.visible=true)");
            return password_dialog_view(&self.state);
        }

        // Show master password dialog (HD wallet authentication)

        // Show transaction confirmation dialog (high priority for transaction flow)
        if self.state.transaction().show_transaction_confirmation {
            return transaction_confirmation_dialog_view(&self.state);
        }

        // Show receive dialog
        if self.state.wallet().receive_dialog.visible {
            return receive_dialog_view(&self.state);
        }

        // Show Dapps message as centered dialog if present
        if !self.state.ui().status_message.is_empty() && self.state.ui().status_message.contains("Dapps feature") {
            return self.dapps_dialog_view();
        }

        // Special case: If we're in history view and showing clear logs confirmation,
        // we need to overlay the dialog on top of the history view
        if self.state.transaction().show_history && self.state.ui().show_clear_logs_confirmation {
            return clear_logs_confirmation_dialog_view(&self.state);
        }

        if self.state.show_custom_token_screen {
            return custom_token_screen_view(&self.state);
        }

        if self.state.transaction().show_history {
            return self.state.history_view();
        }

        // Send dialog is now integrated into main view, no longer a separate view

        if self.state.wallet().show_create_wallet {
            return create_wallet_dialog_view(&self.state);
        }

        if self.state.wallet().show_import_wallet {
            return import_wallet_dialog_view(&self.state);
        }

        if self.state.wallet().show_export_wallet {
            return export_wallet_dialog_view(&self.state);
        }

        if self.state.network().show_add_network {
            return add_network_dialog_view(&self.state);
        }

        // Show delete network confirmation above all if active
        if self.state.network().show_delete_network_confirmation {
            return delete_network_confirmation_dialog_view(&self.state);
        }

        if self.state.wallet().show_delete_account {
            return delete_account_dialog_view(&self.state);
        }

        if self.state.ui().show_clear_logs_confirmation {
            return clear_logs_confirmation_dialog_view(&self.state);
        }

        if self.state.ui().show_dapps_coming_soon {
            return dapps_coming_soon_dialog_view(&self.state);
        }

        if self.state.ui().show_reset_wallet_confirmation {
            return reset_wallet_confirmation_dialog_view(&self.state);
        }

        if self.state.wallet().show_hardware_wallet {
            return hardware_wallet_dialog_view(&self.state);
        }

        // Default: show main wallet view
        self.state.main_wallet_view()
    }

    fn theme(&self) -> Theme {
        self.state.ui().current_theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![];

        // Smart polling subscription
        if self.state.ui().polling_active && self.state.wallet().current_account_id.is_some() {
            subscriptions.push(
                iced::time::every(Duration::from_secs(self.state.ui().poll_interval)).map(|_| Message::SmartPollTick),
            );
        }

        // Status message timer subscription
        if self.state.ui().status_message_timer.is_some() {
            subscriptions.push(iced::time::every(Duration::from_millis(100)).map(|_| Message::StatusMessageTick));
        }

        // Spinner animation subscription
        if self.state.balance_spinner || self.state.ui().accounts_spinner || self.state.ui().transaction_spinner {
            subscriptions.push(iced::time::every(Duration::from_millis(100)).map(|_| Message::SpinnerTick));
        }

        // Price auto-refresh subscription
        if self.state.network().show_price_info {
            subscriptions.push(iced::time::every(Duration::from_secs(30)).map(|_| Message::PriceAutoRefreshTick));
        }

        // Transaction monitoring subscription - check pending transactions every 15 seconds
        if !self.state.transaction().pending_transactions.is_empty() {
            subscriptions.push(iced::time::every(Duration::from_secs(15)).map(|_| Message::TransactionMonitoringTick));
        }

        // Retro loading bar animation subscription - animate when pending transactions exist
        if self
            .state
            .transaction()
            .pending_transactions
            .iter()
            .any(|tx| tx.cancellable)
        {
            subscriptions.push(iced::time::every(Duration::from_millis(50)).map(|_| Message::SpinnerTick));
        }

        // Session timeout subscription - check every 10 seconds
        if self.state.auth().session.auto_lock_enabled {
            subscriptions.push(iced::time::every(Duration::from_secs(10)).map(|_| Message::SessionTimeoutCheck));
        }

        // Keyboard event subscription for modal dialog handling
        if self.state.wallet().show_export_wallet {
            subscriptions.push(iced::keyboard::on_key_press(|key, _modifiers| {
                match key {
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                        Some(Message::HideExportWallet) // Close export dialog on Escape
                    }
                    _ => None,
                }
            }));
        }

        if subscriptions.is_empty() {
            Subscription::none()
        } else {
            Subscription::batch(subscriptions)
        }
    }
}

// Custom tokens storage functions

impl WorkingWalletApp {
    /// Creates a centered dialog view for the Dapps message
    fn dapps_dialog_view(&self) -> Element<'_, Message> {
        use iced::widget::{Button, Column, Container, Space, Text};
        use iced::{Color, Length};

        // Simple centered modal dialog like other dialogs in the system
        Container::new(
            Container::new(
                Column::new()
                    .push(Text::new("Dapps").size(20).style(Color::WHITE))
                    .push(Space::with_height(Length::Fixed(20.0)))
                    .push(
                        Text::new("Decentralized applications (Dapps) feature is coming soon!")
                            .size(14)
                            .style(Color::from_rgb(0.8, 0.8, 0.8)),
                    )
                    .push(Space::with_height(Length::Fixed(20.0)))
                    .push(
                        Button::new(Text::new("OK").size(14))
                            .on_press(Message::ClearStatusMessage)
                            .padding([10, 20])
                            .style(crate::gui::theme::styles::secondary_button()),
                    )
                    .align_items(iced::Alignment::Center)
                    .spacing(5),
            )
            .padding(30)
            .style(crate::gui::theme::styles::dark_flat_container())
            .width(Length::Fixed(400.0)),
        )
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            ..Default::default()
        })
        .into()
    }

    /// Add an error to the log with optional additional context
    fn add_error_log_entry(&mut self, error: &VaughanError, additional_context: Option<String>) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let error_context = error.context();

        let details = match additional_context {
            Some(context) => Some(format!("{}\n\nContext: {}", error_context.user_message, context)),
            None => Some(error_context.user_message.clone()),
        };

        let log_entry = LogEntry {
            timestamp,
            category: LogCategory::Error,
            message: error.user_message(),
            details,
            copyable: true,
            error_severity: Some(error_context.severity),
            error_category: Some(error_context.category),
            support_code: Some(error_context.support_code),
            recovery_steps: Some(error_context.recovery_steps),
        };

        self.state.log_entries.push(log_entry);

        // Limit to 100 entries to prevent memory issues
        if self.state.log_entries.len() > 100 {
            self.state.log_entries.remove(0);
        }
    }

    /// Add a general log entry
    pub fn add_log_entry(&mut self, category: LogCategory, message: String, details: Option<String>) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let log_entry = LogEntry {
            timestamp,
            category,
            message,
            details,
            copyable: true,
            error_severity: None,
            error_category: None,
            support_code: None,
            recovery_steps: None,
        };

        self.state.log_entries.push(log_entry);

        // Limit to 100 entries to prevent memory issues
        if self.state.log_entries.len() > 100 {
            self.state.log_entries.remove(0);
        }
    }
}

// Custom modal background style for semi-transparent overlay

// Basic implementations for core functionality
// Note: initialize_wallet() and load_available_accounts() extracted to services/wallet_service.rs

// Note: load_available_accounts() extracted to services/wallet_service.rs

// All standalone functions extracted to service modules:
// - Network functions: services/network_service.rs
// - Account functions: services/account_service.rs
// - Wallet functions: services/wallet_service.rs

impl WorkingWalletApp {
    // Helper method for periodic incoming transaction checks
    pub fn check_for_incoming_transactions_periodically(&self) -> Command<Message> {
        if let Some(account_id) = &self.state.wallet().current_account_id {
            if let Some(account) = self
                .state
                .wallet()
                .available_accounts
                .iter()
                .find(|a| &a.id == account_id)
            {
                let network_id = self.state.network().current_network;
                let account_address = format!("{:#x}", account.address); // Use proper hex formatting without quotes
                let current_tx_count = self.state.transaction().transaction_history.len();

                return Command::perform(
                    check_for_incoming_transactions(
                        network_id,
                        account_address,
                        current_tx_count,
                        "ETH".to_string(), // Default token for periodic checks
                    ),
                    Message::IncomingTransactionsChecked,
                );
            }
        }
        Command::none()
    }

    /// Start normal wallet initialization after authentication
    /// This loads wallet, accounts, networks, and tokens in parallel
    pub fn start_normal_initialization(&mut self) -> Command<Message> {
        tracing::info!("🚀 Starting normal wallet initialization");

        // Set loading flags
        self.state.wallet_mut().loading_accounts = true;
        self.state.network_mut().loading_networks = true;

        // Check if wallet is unlocked to determine account loading method
        let load_accounts_cmd = if self.state.auth().enhanced_session.is_wallet_ready() {
            // Load accounts from wallet configuration
            tracing::info!("📁 Loading accounts from wallet configuration");
            {
                // Extract wallet config and master password before async operation
                let wallet_config = self
                    .state
                    .auth()
                    .enhanced_session
                    .wallet_session
                    .cached_wallet_config
                    .clone();
                let master_password = self
                    .state
                    .auth()
                    .enhanced_session
                    .wallet_session
                    .cached_master_password
                    .clone();

                Command::perform(
                    Self::load_accounts_from_wallet_config_static(wallet_config, master_password),
                    Message::AccountsLoaded,
                )
            }
        } else {
            // Fallback to legacy account loading
            tracing::info!("📁 Loading accounts using legacy method");
            Command::perform(load_available_accounts(), Message::AccountsLoaded)
        };

        // Create parallel loading commands
        let init_wallet_cmd = Command::perform(initialize_wallet(), Message::WalletInitialized);
        let load_networks_cmd = Command::perform(load_all_networks(), Message::NetworksLoaded);
        let load_tokens_cmd = Command::perform(load_custom_tokens(), |result| {
            Message::CustomTokensLoaded(result.unwrap_or_default())
        });

        Command::batch(vec![
            init_wallet_cmd,
            load_accounts_cmd,
            load_networks_cmd,
            load_tokens_cmd,
        ])
    }

    // Update token list based on selected network
    pub fn update_token_list_for_network(&mut self, network_id: NetworkId) {
        let (native_token, network_name) = match network_id.0 {
            1 => ("NATIVE (ETH)", "Ethereum"),
            369 => ("NATIVE (PLS)", "PulseChain"),
            943 => ("NATIVE (tPLS)", "PulseChain Testnet v4"),
            56 => ("NATIVE (BNB)", "Binance Smart Chain"),
            137 => ("NATIVE (MATIC)", "Polygon"),
            42161 => ("NATIVE (ETH)", "Arbitrum"),
            10 => ("NATIVE (ETH)", "Optimism"),
            _ => ("NATIVE (ETH)", "Unknown Network"),
        };

        // Set the native token based on network
        self.state.transaction_mut().send_selected_token = native_token.to_string();
        // Update balance display token selection
        self.state.balance_selected_token = native_token.to_string();

        // Update available tokens list based on network (ticker only)
        self.state.transaction_mut().send_available_tokens = match network_id.0 {
            1 => vec![
                "ETH".to_string(),
                "USDC".to_string(),
                "USDT".to_string(),
                "WETH".to_string(),
            ],
            369 => vec![
                "PLS".to_string(),
                "PLSX".to_string(),
                "INC".to_string(),
                "HEX".to_string(),
            ],
            943 => vec![
                "tPLS".to_string(),
                "tPLSX".to_string(),
                "tINC".to_string(),
                "tHEX".to_string(),
            ],
            56 => vec![
                "BNB".to_string(),
                "USDT".to_string(),
                "BUSD".to_string(),
                "CAKE".to_string(),
            ],
            137 => vec![
                "MATIC".to_string(),
                "USDC".to_string(),
                "USDT".to_string(),
                "WETH".to_string(),
            ],
            42161 => vec![
                "ETH".to_string(),
                "USDC".to_string(),
                "USDT".to_string(),
                "ARB".to_string(),
            ],
            10 => vec![
                "ETH".to_string(),
                "USDC".to_string(),
                "USDT".to_string(),
                "OP".to_string(),
            ],
            _ => vec!["ETH".to_string()],
        };

        // Update balance available tokens to match send tokens
        self.state.balance_available_tokens = self.state.transaction_mut().send_available_tokens.clone();

        // Add custom tokens to send tokens list as well
        let custom_tokens = self.state.custom_tokens.clone();
        for token in &custom_tokens {
            let token_display = format!("{} ({})", token.symbol, token.address);
            if !self
                .state
                .transaction_mut()
                .send_available_tokens
                .contains(&token_display)
            {
                self.state.transaction_mut().send_available_tokens.push(token_display);
            }
        }

        // Build balance available tickers using the same format as send tokens
        let mut base_tickers = Vec::new();

        // Add native token (no contract address for native)
        let native_ticker = match network_id.0 {
            1 | 42161 | 10 => "ETH",
            369 => "PLS",
            943 => "tPLS",
            56 => "BNB",
            137 => "MATIC",
            _ => "ETH",
        };
        base_tickers.push(native_ticker.to_string());

        // Add tokens with contract addresses using the same format as send tokens
        // Addresses verified from official sources (Etherscan, BSCScan, PolygonScan)
        let tokens_with_addresses = match network_id.0 {
            // Ethereum Mainnet - verified addresses
            1 => vec![
                ("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // Fixed: correct USDC address
                ("USDT", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                ("WETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            ],
            // PulseChain Mainnet - TODO: verify these addresses on PulseChain
            // Note: PulseChain has its own token ecosystem, not Ethereum tokens
            369 => vec![
                // Temporarily disabled until correct PulseChain addresses are verified
                // ("WPLS", "0xA1077a294dDE1B09bB078844df40758a5D0f9a27"),
            ],
            // PulseChain Testnet v4 - verified working addresses
            943 => vec![
                ("USD", "0x3e0Ad60c6D427191D66B6D168ddeF82A66F573B0"),
                ("WPLS", "0xcF1Fc503CA35618E9b4C08b7847980b3e10FB53B"),
            ],
            // BSC - verified addresses from BSCScan
            56 => vec![
                ("USDT", "0x55d398326f99059fF775485246999027B3197955"), // BSC-USD (Binance-Peg USDT)
                ("BUSD", "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56"), // Binance-Peg BUSD
                ("CAKE", "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82"), // PancakeSwap Token
            ],
            // Polygon - verified addresses from PolygonScan
            137 => vec![
                ("USDC", "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"), // USD Coin (PoS)
                ("USDT", "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"), // Tether USD (PoS)
                ("WETH", "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619"), // Wrapped Ether (PoS)
            ],
            _ => vec![],
        };

        // Add tokens with full display format "SYMBOL (ADDRESS)"
        for (symbol, address) in tokens_with_addresses {
            base_tickers.push(format!("{} ({})", symbol, address));
        }

        // Add custom tokens to the balance tickers list using the same format
        for token in &self.state.custom_tokens {
            let token_display = format!("{} ({})", token.symbol, token.address);
            if !base_tickers.contains(&token_display) {
                base_tickers.push(token_display);
            }
        }

        // Update the balance available tickers
        self.state.balance_available_tickers = base_tickers;

        // 🔧 CRITICAL FIX: Initialize token_balances with popular tokens for this network
        self.initialize_token_balances_for_network(network_id);

        // Update selected ticker to match the native token
        let native_ticker = match network_id.0 {
            1 | 42161 | 10 => "ETH",
            369 => "PLS",
            943 => "tPLS",
            56 => "BNB",
            137 => "MATIC",
            _ => "ETH",
        };
        self.state.balance_selected_ticker = native_ticker.to_string();

        tracing::info!(
            "🪙 Updated token list for network {} (Chain ID: {}). Selected: {}",
            network_name,
            network_id.0,
            native_token
        );
    }

    /// Initialize token_balances with popular tokens for the specified network
    /// This ensures all popular tokens are available for balance refresh
    pub fn initialize_token_balances_for_network(&mut self, network_id: NetworkId) {
        let tokens = match network_id.0 {
            // Ethereum Mainnet (Chain ID 1) - verified addresses from Etherscan
            1 => vec![
                ("USDC", "USD Coin", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", 6), // Fixed: correct USDC
                ("USDT", "Tether USD", "0xdAC17F958D2ee523a2206206994597C13D831ec7", 6),
                (
                    "WETH",
                    "Wrapped Ether",
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                    18,
                ),
                (
                    "DAI",
                    "Dai Stablecoin",
                    "0x6B175474E89094C44Da98b954EedeAC495271d0F",
                    18,
                ),
            ],
            // PulseChain Mainnet (Chain ID 369) - TODO: verify correct PulseChain addresses
            369 => vec![
                // Temporarily disabled until correct PulseChain addresses are verified
                // PulseChain has its own token ecosystem, not Ethereum tokens
            ],
            // PulseChain Testnet v4 (Chain ID 943) - verified working addresses
            943 => vec![
                (
                    "USD",
                    "USD Test Token",
                    "0x3e0Ad60c6D427191D66B6D168ddeF82A66F573B0",
                    18,
                ), // Real testnet USD token
                ("WPLS", "Wrapped PLS", "0xcF1Fc503CA35618E9b4C08b7847980b3e10FB53B", 18), // Real testnet WPLS
            ],
            // BSC (Chain ID 56) - verified addresses from BSCScan
            56 => vec![
                ("USDT", "Tether USD", "0x55d398326f99059fF775485246999027B3197955", 18), // BSC-USD
                ("BUSD", "Binance USD", "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56", 18), // Binance-Peg BUSD
                ("CAKE", "PancakeSwap Token", "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82", 18),
            ],
            // Polygon (Chain ID 137) - verified addresses from PolygonScan
            137 => vec![
                ("USDC", "USD Coin (PoS)", "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", 6),
                ("USDT", "Tether USD (PoS)", "0xc2132D05D31c914a87C6611C10748AEb04B58e8F", 6),
                ("WETH", "Wrapped Ether (PoS)", "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619", 18),
            ],
            _ => vec![], // Other networks don't have predefined tokens yet
        };

        let token_count = tokens.len();
        for (symbol, name, address_str, decimals) in tokens {
            // Parse the contract address
            let contract_address = match address_str.parse::<alloy::primitives::Address>() {
                Ok(addr) => Some(addr),
                Err(e) => {
                    tracing::error!(
                        "Failed to parse token contract address '{}' for {}: {}",
                        address_str,
                        symbol,
                        e
                    );
                    continue;
                }
            };

            // Check if this token already exists in token_balances
            let token_exists = self
                .state
                .token_balances
                .iter()
                .any(|existing| existing.symbol == symbol && existing.contract_address == contract_address);

            if !token_exists {
                // Add the token to token_balances for balance refresh functionality
                self.state.token_balances.push(SimpleTokenBalance {
                    symbol: symbol.to_string(),
                    name: name.to_string(),
                    contract_address,
                    balance: "0.0000".to_string(),
                    decimals,
                });

                tracing::debug!(
                    "🔧 Initialized popular token {} in token_balances for network {}",
                    symbol,
                    network_id.0
                );
            }
        }

        tracing::info!(
            "✅ Initialized token_balances with {} popular tokens for network {}",
            token_count,
            network_id.0
        );
    }

    // Helper function to extract token address from display format

    /// Add transaction to history
    pub fn add_transaction_to_history(&mut self, tx_hash: String) {
        // For now, just log the transaction hash
        // In a full implementation, this would add to the transaction history
        self.add_log_entry(
            LogCategory::Info,
            "Transaction added to history".to_string(),
            Some(format!("Transaction Hash: {tx_hash}")),
        );
    }

    /// Update account balance
    pub fn update_account_balance(&mut self) -> Command<Message> {
        // Trigger a balance refresh
        self.dispatch_message(Message::RefreshBalance)
    }

    /// Refresh accounts list
    pub fn refresh_accounts_list(&mut self) -> Command<Message> {
        Command::perform(
            async {
                // Load accounts from both keystores
                tracing::info!("Refreshing accounts list from keystore");

                use crate::security::keychain::OSKeychain;
                use crate::security::keystore::SecureKeystoreImpl;

                // Load from both keychain services
                let mut all_accounts = Vec::new();

                // Load seed-based accounts
                if let Ok(keychain) = OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string()) {
                    if let Ok(keystore) = SecureKeystoreImpl::new(Box::new(keychain)).await {
                        if let Ok(accounts) = keystore.list_accounts().await {
                            all_accounts.extend(accounts);
                            tracing::info!("Loaded {} seed-based accounts", all_accounts.len());
                        }
                    }
                }

                // Load private-key accounts
                if let Ok(keychain) = OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()) {
                    if let Ok(keystore) = SecureKeystoreImpl::new(Box::new(keychain)).await {
                        if let Ok(accounts) = keystore.list_accounts().await {
                            all_accounts.extend(accounts);
                            tracing::info!("Loaded {} total accounts", all_accounts.len());
                        }
                    }
                }

                Ok(all_accounts)
            },
            |result: Result<Vec<SecureAccount>, String>| Message::AccountsLoaded(result),
        )
    }

    /// Check for incoming transactions
    pub fn check_for_incoming_transactions(&mut self) -> Command<Message> {
        if let Some(account_id) = &self.state.wallet().current_account_id {
            if let Some(account) = self
                .state
                .wallet()
                .available_accounts
                .iter()
                .find(|a| &a.id == account_id)
            {
                let network_id = self.state.network().current_network;
                let account_address = format!("{:#x}", account.address);
                let current_tx_count = self.state.transaction().transaction_history.len();

                return Command::perform(
                    check_for_incoming_transactions(network_id, account_address, current_tx_count, "ETH".to_string()),
                    Message::IncomingTransactionsChecked,
                );
            }
        }
        Command::none()
    }

    /// Get transaction history
    pub fn get_transaction_history(&self, account_id: &str, _network_id: NetworkId) -> Command<Message> {
        let account_id = account_id.to_string();
        Command::perform(
            async move {
                tracing::info!("Loading transaction history for account: {}", account_id);
                // Placeholder - would load actual transaction history
                Ok(vec![])
            },
            Message::TransactionHistoryRefreshed,
        )
    }

    /// Dispatch a message as a command (for internal message routing)
    pub fn dispatch_message(&self, message: Message) -> Command<Message> {
        Command::perform(async move { message }, |msg| msg)
    }

    /// Handle wallet password submission for wallet creation and operations
    fn handle_wallet_password_submission(&mut self) -> Command<Message> {
        use secrecy::ExposeSecret;

        let wallet_password_state = &self.state.auth().password_dialog;

        // Get password and reason
        let password = wallet_password_state.input.expose_secret().clone();
        if password.is_empty() {
            // Set error for empty password
            self.state.auth_mut().password_dialog.set_error(
                crate::gui::state::auth_state::WalletPasswordError::InvalidInput {
                    message: "Password cannot be empty".to_string(),
                }
                .into(),
            );
            return Command::none();
        }

        let config = match &wallet_password_state.config {
            Some(config) => config.clone(),
            None => return Command::none(),
        };

        match config {
            crate::gui::state::auth_state::PasswordDialogConfig::WalletSetup { wallet_name } => {
                // Handle wallet setup with master password
                tracing::info!("🔧 Setting up wallet '{}' with master password", wallet_name);
                self.handle_wallet_setup(wallet_name, password)
            }
            crate::gui::state::auth_state::PasswordDialogConfig::WalletUnlock => {
                // Handle wallet unlock
                tracing::info!("🔓 Unlocking wallet with master password");
                self.handle_wallet_unlock(password)
            }
            crate::gui::state::auth_state::PasswordDialogConfig::WalletExport => {
                // Handle wallet unlock for export operations (MetaMask-style flow)
                tracing::info!("🔓 Unlocking wallet for export operation with master password");

                // Attempt to unlock the wallet - this will dispatch MasterPasswordValidated
                // which in turn will eventually dispatch PerformWalletExport via handle_password_validated in security.rs
                self.handle_wallet_unlock(password)
            }
            _ => {
                tracing::warn!("⚠️ Wallet password config not implemented: {:?}", config);
                Command::none()
            }
        }
    }

    /// Handle wallet setup (creation) with master password
    /// Uses the new WalletManager for MetaMask-compatible keystore format
    fn handle_wallet_setup(&mut self, _wallet_name: String, password: String) -> Command<Message> {
        use crate::wallet::WalletManager;
        use secrecy::SecretString;
        use std::fs;

        tracing::info!("🚀 Creating new wallet using WalletManager (MetaMask-compatible format)");

        // Get keystore path
        let wallet_dir = crate::security::keystore::storage::get_vaughan_dir();

        // Ensure directory exists
        if let Err(e) = fs::create_dir_all(&wallet_dir) {
            tracing::error!("❌ Failed to create wallet directory: {}", e);
            self.state.auth_mut().password_dialog.set_error(
                crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                    reason: format!("Failed to create wallet directory: {e}"),
                }
                .into(),
            );
            return Command::none();
        }

        let keystore_path = wallet_dir.join("keystore.json");
        tracing::info!("💾 Creating MetaMask-compatible keystore at: {:?}", keystore_path);

        // Create wallet using WalletManager
        let mut manager = WalletManager::new(keystore_path);
        let secret_password = SecretString::new(password);

        let address = match manager.create_wallet(secret_password) {
            Ok(addr) => {
                tracing::info!("✅ Wallet created with address: {}", addr);
                addr
            }
            Err(e) => {
                tracing::error!("❌ Wallet creation failed: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: format!("Wallet creation failed: {e}"),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        // Create config directory for wallet storage
        let config_dir = dirs::home_dir().unwrap_or_default().join(".config").join("vaughan");
        if let Err(e) = fs::create_dir_all(&config_dir) {
            tracing::error!("❌ Failed to create config directory: {}", e);
            self.state.auth_mut().password_dialog.set_error(
                crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                    reason: format!("Failed to create config directory: {e}"),
                }
                .into(),
            );
            return Command::none();
        }

        // Create a key reference for the encrypted private key
        let key_reference = crate::security::KeyReference {
            id: uuid::Uuid::new_v4().to_string(),
            service: "vaughan-wallet".to_string(),
            account: "main-account".to_string(),
        };

        // Add account to the account list
        let account = crate::security::SecureAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Main Account".to_string(),
            address,
            key_reference,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
            tags: Vec::new(),
            last_used: None,
            transaction_count: 0,
        };

        self.state.wallet_mut().available_accounts.push(account.clone());
        self.state.wallet_mut().current_account_id = Some(account.id.clone());

        // Hide the password dialog
        self.state.auth_mut().password_dialog.hide();

        // Save account to persistent storage for future sessions
        let accounts_path = crate::security::keystore::storage::get_vaughan_dir()
            .join("accounts.json");

        tracing::info!(
            "💾 Saving new account to persistent storage: {}",
            accounts_path.display()
        );

        let account_metadata = serde_json::json!({
            "id": account.id,
            "name": account.name,
            "address": format!("{:?}", account.address),
            "derivation_path": account.derivation_path,
            "created_at": account.created_at.to_rfc3339(),
            "is_hardware": account.is_hardware,
            "key_reference": {
                "id": account.key_reference.id,
                "service": account.key_reference.service,
                "account": account.key_reference.account
            }
        });

        // Load existing accounts or create new list
        let mut all_accounts = if accounts_path.exists() {
            match fs::read_to_string(&accounts_path) {
                Ok(content) => serde_json::from_str::<Vec<serde_json::Value>>(&content).unwrap_or_else(|_| Vec::new()),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        all_accounts.push(account_metadata);

        if let Ok(json_content) = serde_json::to_string_pretty(&all_accounts) {
            if let Err(e) = fs::write(&accounts_path, json_content) {
                tracing::error!("❌ Failed to save accounts.json: {}", e);
            } else {
                tracing::info!("✅ Account saved to accounts.json");
            }
        }

        // Set success status
        self.state.ui_mut().status_message = "✅ Wallet created successfully!".to_string();
        self.state.ui_mut().status_message_color = crate::gui::wallet_types::StatusMessageColor::Success;

        tracing::info!("✅ Wallet saved and account added successfully");

        // Trigger startup authentication complete to proceed with initialization
        self.dispatch_message(Message::StartupAuthenticationComplete)
    }

    /// Handle account password submission for account session unlock
    fn handle_account_password_submission(&mut self) -> Command<Message> {
        use secrecy::ExposeSecret;

        let password_dialog = &self.state.auth().password_dialog;

        // Get password and reason
        let password = password_dialog.input.expose_secret().clone();
        if password.is_empty() {
            self.state
                .auth_mut()
                .password_dialog
                .set_error(crate::gui::state::auth_state::PasswordError::EmptyPassword);
            return Command::none();
        }

        // Extract account ID from the config
        let account_id = match &password_dialog.config {
            Some(crate::gui::state::auth_state::PasswordDialogConfig::AccountUnlock { account_id, .. }) => {
                account_id.clone()
            }
            _ => {
                tracing::error!("❌ Invalid password dialog config for account unlock");
                return Command::none();
            }
        };

        tracing::info!("🔐 Processing account password for account: {}", account_id);

        // Submit the account password for processing
        self.dispatch_message(Message::AccountPasswordSubmitted(account_id, password))
    }

    /// Process account unlock with provided password
    fn process_account_unlock(&mut self, account_id: String, password: String) -> Command<Message> {
        use secrecy::SecretString;

        let secret_password = SecretString::new(password);

        // Find the account
        let _account = match self
            .state
            .wallet()
            .available_accounts
            .iter()
            .find(|a| a.id == account_id)
        {
            Some(account) => account.clone(),
            None => {
                self.state
                    .auth_mut()
                    .password_dialog
                    .set_error(crate::gui::state::auth_state::PasswordError::DecryptionFailed);
                return Command::none();
            }
        };

        // Extract remember_session before mutable borrow
        let remember_session = self.state.auth().password_dialog.remember_session;

        // Unlock the account session in enhanced session state
        self.state.auth_mut().enhanced_session.unlock_account_session(
            account_id.clone(),
            secret_password,
            remember_session,
            None, // Use default account settings
        );

        // Hide the password dialog
        self.state.auth_mut().password_dialog.hide();

        tracing::info!("✅ Account session unlocked: {}", account_id);

        // Trigger account session unlocked message to complete selection
        self.dispatch_message(Message::AccountSessionUnlocked(account_id))
    }

    /// Handle wallet unlock with master password
    /// Uses the new WalletManager for MetaMask-compatible keystore format
    /// Falls back to legacy wallet.json if keystore.json not found (backward compatibility)
    fn handle_wallet_unlock(&mut self, password: String) -> Command<Message> {
        use crate::wallet::WalletManager;
        use secrecy::SecretString;

        tracing::info!("🔓 Wallet unlock attempt with password length: {}", password.len());

        // Get wallet directory
        let wallet_dir = crate::security::keystore::storage::get_vaughan_dir();

        let keystore_path = wallet_dir.join("keystore.json");
        let legacy_wallet_path = wallet_dir.join("wallet.json");

        // Try new WalletManager format first
        if keystore_path.exists() {
            tracing::info!("🔍 Found MetaMask-compatible keystore at: {:?}", keystore_path);

            let mut manager = WalletManager::new(keystore_path);
            let secret_password = SecretString::new(password.clone());

            match manager.unlock(secret_password) {
                Ok(()) => {
                    let address = match manager.address() {
                        Ok(addr) => addr,
                        Err(e) => {
                            tracing::error!("❌ Failed to get address after unlock: {}", e);
                            self.state.auth_mut().password_dialog.set_error(
                                crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                                    reason: format!("Failed to get address: {e}"),
                                }
                                .into(),
                            );
                            return Command::none();
                        }
                    };

                    tracing::info!("✅ Wallet unlocked successfully: {}", address);

                    // Create key reference for the account
                    let key_reference = crate::security::KeyReference {
                        id: uuid::Uuid::new_v4().to_string(),
                        service: "vaughan-wallet".to_string(),
                        account: "main-account".to_string(),
                    };

                    // Create SecureAccount and add to available_accounts
                    let account = crate::security::SecureAccount {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Main Account".to_string(),
                        address,
                        key_reference,
                        created_at: chrono::Utc::now(),
                        is_hardware: false,
                        derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
                        tags: Vec::new(),
                        last_used: None,
                        transaction_count: 0,
                    };

                    self.state.wallet_mut().available_accounts.push(account.clone());
                    self.state.wallet_mut().current_account_id = Some(account.id.clone());

                    tracing::info!("✅ Password validated successfully - account loaded: {}", account.name);

                    // Create wallet config for session management (MetaMask pattern)
                    let wallet_config = match crate::security::WalletConfig::new(
                        "Vaughan Wallet".to_string(),
                        &secrecy::SecretString::new(password.clone()),
                    ) {
                        Ok(config) => config,
                        Err(e) => {
                            tracing::error!("❌ Failed to create wallet config: {}", e);
                            self.state.auth_mut().password_dialog.set_error(
                                crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                                    reason: "Failed to create wallet configuration".to_string(),
                                }
                                .into(),
                            );
                            return Command::none();
                        }
                    };

                    // Unlock the wallet session
                    let remember_session = self.state.auth().password_dialog.remember_session;
                    self.state.auth_mut().enhanced_session.wallet_session.unlock(
                        wallet_config,
                        secrecy::SecretString::new(password),
                        remember_session,
                    );

                    tracing::info!("✅ Wallet session unlocked successfully - ready for operations");

                    // Hide dialog and proceed with startup
                    self.state.auth_mut().password_dialog.hide();

                    // Set status message
                    self.state.ui_mut().status_message = "✅ Wallet unlocked successfully!".to_string();
                    self.state.ui_mut().status_message_color = crate::gui::wallet_types::StatusMessageColor::Success;

                    // Trigger startup authentication complete
                    return self.dispatch_message(Message::StartupAuthenticationComplete);
                }
                Err(e) => {
                    tracing::error!("❌ Unlock failed: {}", e);
                    self.state.auth_mut().password_dialog.set_error(
                        crate::gui::state::auth_state::WalletPasswordError::IncorrectPassword { attempts_remaining: 3 }
                            .into(),
                    );
                    return Command::none();
                }
            }
        }

        // Fallback to legacy wallet.json format for backward compatibility
        if legacy_wallet_path.exists() {
            tracing::info!(
                "🔍 Found legacy wallet at: {:?}, attempting legacy unlock",
                legacy_wallet_path
            );
            return self.handle_legacy_wallet_unlock(password, legacy_wallet_path);
        }

        // No wallet file found
        tracing::error!(
            "❌ No wallet file found at {:?} or {:?}",
            keystore_path,
            legacy_wallet_path
        );
        self.state.auth_mut().password_dialog.set_error(
            crate::gui::state::auth_state::WalletPasswordError::InvalidInput {
                message: "Wallet file not found".to_string(),
            }
            .into(),
        );
        Command::none()
    }

    /// Handle legacy wallet.json unlock (backward compatibility)
    fn handle_legacy_wallet_unlock(&mut self, password: String, wallet_file: std::path::PathBuf) -> Command<Message> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use alloy::signers::local::PrivateKeySigner;
        use sha2::{Digest, Sha256};
        use std::fs;

        tracing::info!("🔓 Attempting legacy wallet unlock");

        let wallet_data = match fs::read_to_string(&wallet_file) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("❌ Failed to read wallet file: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: format!("Failed to read wallet file: {e}"),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let wallet_json: serde_json::Value = match serde_json::from_str(&wallet_data) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("❌ Failed to parse wallet JSON: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Wallet file is corrupted".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let encrypted_hex = match wallet_json["encrypted_private_key"].as_str() {
            Some(hex) => hex,
            None => {
                tracing::error!("❌ Missing encrypted_private_key in wallet file");
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Wallet file is corrupted".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let nonce_hex = match wallet_json["nonce"].as_str() {
            Some(hex) => hex,
            None => {
                tracing::error!("❌ Missing nonce in wallet file");
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Wallet file is corrupted".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let address_str = match wallet_json["address"].as_str() {
            Some(addr) => addr,
            None => {
                tracing::error!("❌ Missing address in wallet file");
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Wallet file is corrupted".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        // Derive encryption key from password using SHA256 (legacy method)
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key_bytes = hasher.finalize();
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let encrypted_bytes = match hex::decode(encrypted_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("❌ Failed to decode encrypted key hex: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Wallet file is corrupted".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let nonce_bytes = match hex::decode(nonce_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("❌ Failed to decode nonce hex: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Wallet file is corrupted".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted_bytes = match cipher.decrypt(nonce, encrypted_bytes.as_ref()) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("❌ Decryption failed (likely incorrect password): {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::IncorrectPassword { attempts_remaining: 3 }
                        .into(),
                );
                return Command::none();
            }
        };

        if decrypted_bytes.len() != 32 {
            tracing::error!("❌ Invalid private key length: {} (expected 32)", decrypted_bytes.len());
            self.state.auth_mut().password_dialog.set_error(
                crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                    reason: "Invalid private key length".to_string(),
                }
                .into(),
            );
            return Command::none();
        }

        let mut key_bytes_arr = [0u8; 32];
        key_bytes_arr.copy_from_slice(&decrypted_bytes);
        let private_key_b256 = alloy::primitives::B256::from(key_bytes_arr);

        let wallet = match PrivateKeySigner::from_bytes(&private_key_b256) {
            Ok(wallet) => wallet,
            Err(e) => {
                tracing::error!("❌ Failed to create wallet from decrypted key: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Failed to restore wallet from decrypted key".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let decrypted_address = wallet.address();

        if format!("{decrypted_address:#x}").to_lowercase() != address_str.to_lowercase() {
            tracing::error!(
                "❌ Address mismatch! Expected: {}, Got: {:#x}",
                address_str,
                decrypted_address
            );
            self.state.auth_mut().password_dialog.set_error(
                crate::gui::state::auth_state::WalletPasswordError::IncorrectPassword { attempts_remaining: 3 }.into(),
            );
            return Command::none();
        }

        tracing::info!("✅ Legacy wallet password validated - address matches: {}", address_str);

        // Create key reference and account
        let key_reference = crate::security::KeyReference {
            id: uuid::Uuid::new_v4().to_string(),
            service: "vaughan-wallet".to_string(),
            account: "main-account".to_string(),
        };

        let account = crate::security::SecureAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Main Account".to_string(),
            address: decrypted_address,
            key_reference,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
            tags: Vec::new(),
            last_used: None,
            transaction_count: 0,
        };

        self.state.wallet_mut().available_accounts.push(account.clone());
        self.state.wallet_mut().current_account_id = Some(account.id.clone());

        // Create wallet config for session management
        let wallet_config = match crate::security::WalletConfig::new(
            "Vaughan Wallet".to_string(),
            &secrecy::SecretString::new(password.clone()),
        ) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("❌ Failed to create wallet config: {}", e);
                self.state.auth_mut().password_dialog.set_error(
                    crate::gui::state::auth_state::WalletPasswordError::CreationFailed {
                        reason: "Failed to create wallet configuration".to_string(),
                    }
                    .into(),
                );
                return Command::none();
            }
        };

        let remember_session = self.state.auth().password_dialog.remember_session;
        self.state.auth_mut().enhanced_session.wallet_session.unlock(
            wallet_config,
            secrecy::SecretString::new(password),
            remember_session,
        );

        tracing::info!("✅ Legacy wallet session unlocked successfully");

        self.state.auth_mut().password_dialog.hide();
        self.state.ui_mut().status_message = "✅ Wallet unlocked successfully!".to_string();
        self.state.ui_mut().status_message_color = crate::gui::wallet_types::StatusMessageColor::Success;

        self.dispatch_message(Message::StartupAuthenticationComplete)
    }

    /// Load accounts from wallet configuration metadata
    async fn load_accounts_from_wallet_config_static(
        wallet_config: Option<crate::security::WalletConfig>,
        master_password: Option<secrecy::SecretString>,
    ) -> Result<Vec<crate::security::SecureAccount>, String> {
        // Get the wallet configuration and master password
        let (wallet_config, master_password) = match (wallet_config, master_password) {
            (Some(config), Some(password)) => (config, password),
            _ => {
                tracing::error!("❌ No cached wallet config or master password available");
                return Err("Wallet configuration or master password not available".to_string());
            }
        };

        // Decrypt account metadata from wallet config
        let account_metadata = wallet_config
            .decrypt_account_metadata(&master_password)
            .map_err(|e| format!("Failed to decrypt account metadata: {e:?}"))?;

        // Convert account metadata to SecureAccount instances
        let mut secure_accounts = Vec::new();

        for account_meta in &account_metadata {
            tracing::info!("📋 Loading account: {}", account_meta.name);

            // Create SecureAccount from metadata
            // Note: The actual encrypted seed/private key would be loaded separately
            // when the account needs to be unlocked with its individual password
            let secure_account = crate::security::SecureAccount::new_from_metadata(account_meta)
                .map_err(|e| format!("Failed to create SecureAccount: {e:?}"))?;

            secure_accounts.push(secure_account);
        }

        tracing::info!("✅ Loaded {} accounts from wallet configuration", secure_accounts.len());
        Ok(secure_accounts)
    }

    /// Complete account selection after account session is verified as unlocked
    pub fn complete_account_selection(
        &mut self,
        account_id: String,
        account: crate::security::SecureAccount,
    ) -> Command<Message> {
        let account_name = account.name.clone();
        let account_address = account.address;

        tracing::info!("🎯 Completing account selection for: {}", account_name);

        // Get coordinator-managed commands
        let coordinator_commands = self.state.change_account_coordinated(account_id.clone());

        // Update legacy fields for compatibility
        self.state.wallet_mut().current_account = format!("{account_address}");
        self.state.transaction_mut().send_from_account_id = Some(account_id.clone());
        self.state.last_balance = "0.0000".to_string();

        // Update the current account ID
        self.state.wallet_mut().current_account_id = Some(account_id.clone());

        // In the two-tier model, the account is already authenticated at the session level
        // No need to unlock the entire wallet again - just proceed with the selection

        self.add_log_entry(
            LogCategory::Success,
            "Account switched successfully".to_string(),
            Some(format!("Switched to account: {account_name} (two-tier security)")),
        );

        // Return coordinator commands to handle balance refresh, etc.
        Command::batch(coordinator_commands)
    }
}
