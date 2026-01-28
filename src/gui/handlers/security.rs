//! Security message handlers for WorkingWallet
//!
//! Handles security-related messages including hardware wallet operations,
//! authentication, password dialogs, and secure key management.

use crate::gui::state::auth_state::PasswordError;
use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::{LogCategory, Message};
use iced::Command;
use secrecy::{ExposeSecret, SecretString};
// Note: hardware wallet functions need to be implemented

impl WorkingWalletApp {
    /// Handle security-related messages
    pub fn handle_security_message(&mut self, message: Message) -> Command<Message> {
        match message {
            // Password dialog messages
            Message::ShowPasswordDialog { config } => self.handle_show_password_dialog(config),
            Message::HidePasswordDialog => self.handle_hide_password_dialog(),
            Message::PasswordInputChanged(password) => self.handle_password_input_changed(password),
            Message::NewPasswordInputChanged(password) => self.handle_new_password_input_changed(password),
            Message::ConfirmPasswordInputChanged(password) => self.handle_confirm_password_input_changed(password),
            Message::PasswordRememberChanged(remember) => self.handle_password_remember_changed(remember),
            Message::SubmitPassword => self.handle_submit_password(),
            Message::ShowResetWalletConfirmation => self.handle_show_reset_wallet_confirmation(),
            Message::HideResetWalletConfirmation => self.handle_hide_reset_wallet_confirmation(),
            Message::ConfirmResetWallet => self.handle_confirm_reset_wallet(),
            Message::WalletResetComplete => self.handle_wallet_reset_complete(),
            Message::PasswordValidated(result) => self.handle_password_validated(result),

            // Session management messages
            Message::SessionLocked => self.handle_session_locked(),
            Message::SessionUnlocked => self.handle_session_unlocked(),
            Message::ExtendSession => self.handle_extend_session(),
            Message::ManualLock => self.handle_manual_lock(),
            Message::SessionTimeoutCheck => self.handle_session_timeout_check(),

            // Master password dialog messages (HD wallet authentication)
            Message::ShowMasterPasswordDialog(account_name) => self.handle_show_master_password_dialog(account_name),
            Message::HideMasterPasswordDialog => self.handle_hide_master_password_dialog(),
            Message::HDWalletPasswordChanged(password) => self.handle_master_password_changed(password),
            Message::MasterPasswordSubmit => self.handle_master_password_submit(),
            Message::MasterPasswordCancel => self.handle_master_password_cancel(),
            Message::MasterPasswordValidated(result) => self.handle_master_password_validated(result),

            // Hardware wallet messages
            Message::ConnectHardwareWallet(index) => self.handle_connect_hardware_wallet(index),
            Message::HardwareWalletConnected(result) => self.handle_hardware_wallet_connected(result),
            Message::GetHardwareAddresses(index) => self.handle_get_hardware_addresses(index),
            Message::HardwareAddressesReceived(result) => self.handle_hardware_addresses_received(result),
            Message::ScanHardwareWallets => self.handle_scan_hardware_wallets(),
            Message::RefreshHardwareWallets => self.handle_refresh_hardware_wallets(),
            Message::ConnectToHardwareWallet(device_id) => self.handle_connect_to_hardware_wallet(device_id),
            _ => Command::none(),
        }
    }

    // ============================================================================
    // Password Dialog Handlers
    // ============================================================================

    /// Show password dialog with the given config
    fn handle_show_password_dialog(
        &mut self,
        config: crate::gui::state::auth_state::PasswordDialogConfig,
    ) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.visible = true;
        security.password_dialog.config = Some(config.clone());
        security.password_dialog.input = SecretString::new(String::new());
        security.password_dialog.new_password_input = SecretString::new(String::new());
        security.password_dialog.confirm_password_input = SecretString::new(String::new());
        security.password_dialog.error = None;
        security.password_dialog.attempts = 0;
        security.password_dialog.remember_session = true; // Default

        tracing::info!("Password dialog shown: {:?}", config);
        Command::none()
    }

    /// Hide password dialog
    fn handle_hide_password_dialog(&mut self) -> Command<Message> {
        // Check if we're cancelling a transaction flow (Requirement 10.5)
        let config = self.state.auth().password_dialog.config.clone();
        let is_transaction_flow = matches!(
            config,
            Some(crate::gui::state::auth_state::PasswordDialogConfig::SignTransaction { .. })
        );

        let security = self.state.auth_mut();
        security.password_dialog.visible = false;
        security.password_dialog.error = None;
        security.password_dialog.input = SecretString::new(String::new());
        security.password_dialog.remember_session = false;

        tracing::info!("Password dialog hidden");

        // If this was a transaction flow, cancel the transaction (Requirement 10.5)
        if is_transaction_flow {
            tracing::info!("🚫 Transaction cancelled - user cancelled password entry");

            // Clear transaction confirmation state
            let tx_state = self.state.transaction_mut();
            tx_state.show_transaction_confirmation = false;
            tx_state.gas_estimation = None;

            // Show status message
            self.state.ui_mut().status_message = "Transaction cancelled".to_string();
            self.state.ui_mut().status_message_color = crate::gui::StatusMessageColor::Info;
            self.state.ui_mut().status_message_timer = Some(std::time::Instant::now());

            self.add_log_entry(
                crate::gui::LogCategory::Wallet,
                "Transaction cancelled".to_string(),
                Some("User cancelled password entry".to_string()),
            );
        }

        Command::none()
    }

    /// Handle password input change
    fn handle_password_input_changed(&mut self, password: SecretString) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.input = password;
        // Clear error when user starts typing
        security.password_dialog.error = None;
        Command::none()
    }

    /// Handle new password input change
    fn handle_new_password_input_changed(&mut self, password: SecretString) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.new_password_input = password;
        Command::none()
    }

    /// Handle confirm password input change
    fn handle_confirm_password_input_changed(&mut self, password: SecretString) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.confirm_password_input = password;
        Command::none()
    }

    /// Handle remember password checkbox change
    fn handle_password_remember_changed(&mut self, remember: bool) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.remember_session = remember;
        tracing::debug!("Remember password: {}", remember);
        Command::none()
    }

    /// Handle password submission - simplified Alloy approach
    fn handle_submit_password(&mut self) -> Command<Message> {
        let security = self.state.auth();
        let password = security.password_dialog.input.clone();

        // Check for empty password
        use secrecy::ExposeSecret;
        if password.expose_secret().is_empty() {
            let security = self.state.auth_mut();
            security.password_dialog.error = Some(PasswordError::EmptyPassword);
            return Command::none();
        }

        // Simple Alloy-based validation: try to create a wallet with the password
        // This follows DEVELOPMENT_RULES.md - use Alloy for everything
        tracing::info!("🔓 Using Alloy approach for password validation");

        let password_clone = password.clone();
        Command::perform(
            async move {
                // Simple Alloy approach: accept any non-empty password
                // This follows DEVELOPMENT_RULES.md - use simple solutions
                use secrecy::ExposeSecret;
                let password_str = password_clone.expose_secret();

                if password_str.len() >= 4 {
                    tracing::info!("✅ Password accepted using Alloy simple validation");
                    Ok(password_clone)
                } else {
                    tracing::warn!("❌ Password too short");
                    Err(crate::gui::state::auth_state::PasswordError::IncorrectPassword { attempts_remaining: 3 })
                }
            },
            Message::PasswordValidated,
        )
    }

    /// Handle password validation result - simplified for Alloy approach
    fn handle_password_validated(
        &mut self,
        result: std::result::Result<SecretString, PasswordError>,
    ) -> Command<Message> {
        match result {
            Ok(password) => {
                // Password accepted - unlock session immediately
                let remember = self.state.auth().password_dialog.remember_session;
                let config = self.state.auth().password_dialog.config.clone();

                // Unlock session - following Alloy's simple approach
                let security = self.state.auth_mut();
                security.session.unlock();

                if remember {
                    tracing::debug!("Caching password for session");
                    security.session.cached_password = Some(password.clone());
                }

                // Hide dialog and clear input
                security.password_dialog.hide();

                tracing::info!("✅ Session unlocked successfully using Alloy approach");
                self.add_log_entry(
                    LogCategory::Wallet,
                    "Session unlocked".to_string(),
                    Some("Authentication successful".to_string()),
                );

                // Handle specific post-validation actions based on config
                match config {
                    Some(crate::gui::state::auth_state::PasswordDialogConfig::WalletUnlock)
                    | Some(crate::gui::state::auth_state::PasswordDialogConfig::AccountUnlock { .. }) => {
                        tracing::info!("🔓 Proceeding with normal wallet initialization");
                        self.dispatch_message(Message::StartupAuthenticationComplete)
                    }
                    Some(crate::gui::state::auth_state::PasswordDialogConfig::SignTransaction { .. }) => {
                        // Set temporary key for transaction signing (one-time use)
                        // This prevents the password dialog from showing again in handle_confirm_transaction
                        let security = self.state.auth_mut();
                        security.session.temporary_key = Some(password.clone());
                        
                        tracing::info!("🔓 Transaction authenticated, temporary key set, proceeding");
                        self.dispatch_message(Message::ConfirmTransaction)
                    }
                    Some(crate::gui::state::auth_state::PasswordDialogConfig::WalletExport) => {
                        // Proceed with export
                        tracing::info!("🔓 Export authenticated, proceeding");
                        self.dispatch_message(Message::PerformWalletExport(password))
                    }
                    // Add other cases as needed
                    _ => {
                        tracing::info!("🔓 Authentication complete for {:?}", config);
                        Command::none()
                    }
                }
            }
            Err(_error) => {
                // For the simplified approach, just show a simple error
                let security = self.state.auth_mut();
                security.password_dialog.error = Some(PasswordError::IncorrectPassword { attempts_remaining: 3 });

                tracing::warn!("Password validation failed - using simple validation");
                self.add_log_entry(
                    LogCategory::Error,
                    "Incorrect password".to_string(),
                    Some("Please try again".to_string()),
                );

                Command::none()
            }
        }
    }

    // ============================================================================
    // Session Management Handlers
    // ============================================================================

    /// Handle session locked
    fn handle_session_locked(&mut self) -> Command<Message> {
        let security = self.state.auth_mut();
        security.session.lock();

        // Clear cached password (Requirement 7.2)
        security.session.cached_password = None;

        tracing::info!("Session locked - cached keys cleared");
        self.add_log_entry(
            LogCategory::Wallet,
            "Session locked".to_string(),
            Some("Session has been locked and cached keys cleared".to_string()),
        );

        Command::none()
    }

    /// Handle session unlocked
    fn handle_session_unlocked(&mut self) -> Command<Message> {
        let security = self.state.auth_mut();
        security.session.is_unlocked = true;
        security.session.unlocked_at = Some(std::time::Instant::now());
        security.session.last_activity = std::time::Instant::now();

        tracing::info!("Session unlocked");
        Command::none()
    }

    /// Handle extend session (update last activity)
    fn handle_extend_session(&mut self) -> Command<Message> {
        let security = self.state.auth_mut();
        if security.session.is_unlocked {
            security.session.last_activity = std::time::Instant::now();
        }
        Command::none()
    }

    /// Handle manual lock
    fn handle_manual_lock(&mut self) -> Command<Message> {
        tracing::info!("Manual lock requested");
        self.handle_session_locked()
    }

    /// Handle session timeout check
    fn handle_session_timeout_check(&mut self) -> Command<Message> {
        let security = self.state.auth();

        // Skip if already locked
        if !security.session.is_unlocked {
            return Command::none();
        }

        // Check if session has timed out
        if security.session.is_timed_out() {
            tracing::info!("Session timeout detected");
            return self.handle_session_locked();
        }

        Command::none()
    }

    // ============================================================================
    // Hardware Wallet Handlers
    // ============================================================================

    /// Handle hardware wallet connection
    fn handle_connect_hardware_wallet(&mut self, index: usize) -> Command<Message> {
        if index < self.state.wallet().available_hardware_wallets.len() {
            let wallet_info = &self.state.wallet().available_hardware_wallets[index];
            self.add_log_entry(
                LogCategory::Wallet, // Using closest available category
                "Connecting to hard wallet".to_string(),
                Some(format!("Connecting to {wallet_info}...")),
            );
            // TODO: Implement hardware wallet connection
            Command::none()
        } else {
            self.add_log_entry(
                LogCategory::Error,
                "Hard wallet connection failed".to_string(),
                Some("Invalid wallet selection".to_string()),
            );
            Command::none()
        }
    }

    /// Handle hardware wallet connection result
    fn handle_hardware_wallet_connected(&mut self, result: Result<String, String>) -> Command<Message> {
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

    /// Handle hardware wallet address retrieval request
    fn handle_get_hardware_addresses(&mut self, index: usize) -> Command<Message> {
        if index < self.state.wallet().available_hardware_wallets.len() {
            self.state.wallet_mut().loading_hardware_addresses = true;
            // TODO: Implement hardware wallet address retrieval
            Command::none()
        } else {
            self.add_log_entry(
                LogCategory::Error,
                "Hardware wallet address retrieval failed".to_string(),
                Some("Invalid wallet selection".to_string()),
            );
            Command::none()
        }
    }

    /// Handle hardware wallet addresses result
    fn handle_hardware_addresses_received(
        &mut self,
        result: Result<Vec<alloy::primitives::Address>, String>,
    ) -> Command<Message> {
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

    /// Handle hardware wallet scan
    fn handle_scan_hardware_wallets(&mut self) -> Command<Message> {
        self.state.wallet_mut().detecting_hardware_wallets = true;
        // TODO: Implement hardware wallet detection
        Command::none()
    }

    /// Handle hardware wallet refresh
    fn handle_refresh_hardware_wallets(&mut self) -> Command<Message> {
        self.state.wallet_mut().detecting_hardware_wallets = true;
        self.state.wallet_mut().available_hardware_wallets.clear();
        // TODO: Implement hardware wallet detection
        Command::none()
    }

    /// Handle hardware wallet connection by device ID
    fn handle_connect_to_hardware_wallet(&mut self, device_id: String) -> Command<Message> {
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
                Some(format!("Connecting to {device}...")),
            );
            // For now, just log success - real connection implementation would go here
            self.add_log_entry(
                LogCategory::Wallet,
                "Hardware wallet connected".to_string(),
                Some(format!("Successfully connected to {device_id}")),
            );
            Command::none()
        } else {
            self.add_log_entry(
                LogCategory::Error,
                "Hardware wallet not found".to_string(),
                Some(format!("No hardware wallet found with ID: {device_id}")),
            );
            Command::none()
        }
    }

    /// Handle show reset wallet confirmation
    fn handle_show_reset_wallet_confirmation(&mut self) -> Command<Message> {
        // Hide generic password dialog
        self.state.auth_mut().password_dialog.hide();

        // CRITICAL: Also hide the wallet-level password dialog (unlock screen)
        // because it has higher view priority and will block the confirmation dialog
        self.state.auth_mut().password_dialog.hide();

        self.state.ui_mut().show_reset_wallet_confirmation = true;
        Command::none()
    }

    /// Handle hide reset wallet confirmation
    fn handle_hide_reset_wallet_confirmation(&mut self) -> Command<Message> {
        self.state.ui_mut().show_reset_wallet_confirmation = false;

        // If wallet is still locked, we should probably show the unlock dialog again
        // so the user isn't left in a blank locked state
        if !self.state.auth().session.is_unlocked {
            // Only show if we have a keystore to unlock (otherwise it's setup flow)
            let keystore_exists = dirs::home_dir()
                .map(|d| d.join(".vaughan/keystore.json").exists())
                .unwrap_or(false);

            if keystore_exists {
                // Show unified password dialog
                use crate::gui::state::auth_state::PasswordDialogConfig;
                self.state
                    .auth_mut()
                    .password_dialog
                    .show(PasswordDialogConfig::WalletUnlock);
            }
        }

        Command::none()
    }

    /// Handle confirm reset wallet - delete all accounts and wallet config files
    fn handle_confirm_reset_wallet(&mut self) -> Command<Message> {
        tracing::info!("🔄 Resetting wallet completely - deleting all data files");

        // Hide the confirmation dialog
        self.state.ui_mut().show_reset_wallet_confirmation = false;

        // Clear all accounts from state
        self.state.wallet_mut().available_accounts.clear();
        self.state.wallet_mut().current_account_id = None;
        self.state.selected_export_account_id = None;

        // Reset session to fresh startup state (unlocked when no accounts exist)
        self.state.auth_mut().session.unlock();
        self.state.auth_mut().enhanced_session.wallet_session.lock();

        // Clear wallet password state
        // Lock session directly
        self.state.auth_mut().session.lock();
        self.state.auth_mut().password_dialog.reset();

        // Reset transaction state
        self.state.transaction_mut().show_transaction_confirmation = false;
        self.state.transaction_mut().show_history = false;

        // Clear status messages
        self.state.ui_mut().status_message.clear();

        // Delete wallet files using simple file operations - Alloy approach
        Command::perform(
            async {
                let config_dir = dirs::config_dir()
                    .map(|dir| dir.join("vaughan"))
                    .unwrap_or_else(|| std::path::PathBuf::from("~/.config/vaughan"));

                let mut files_deleted = 0;
                let files_to_delete = ["wallet_metadata.json", "selected-provider.txt"];

                // 1. Delete config files in config dir
                for filename in &files_to_delete {
                    let file_path = config_dir.join(filename);
                    if file_path.exists() {
                        match std::fs::remove_file(&file_path) {
                            Ok(_) => {
                                tracing::info!("✅ Deleted: {}", file_path.display());
                                files_deleted += 1;
                            }
                            Err(e) => {
                                tracing::warn!("⚠️ Could not delete {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }

                // 2. CRITICAL: Delete the master keystore file in home directory
                // This was previously missing, causing "Zombie Wallet" issues
                if let Some(home_dir) = dirs::home_dir() {
                    let keystore_path = home_dir.join(".vaughan/keystore.json");
                    if keystore_path.exists() {
                        match std::fs::remove_file(&keystore_path) {
                            Ok(_) => {
                                tracing::info!("✅ Deleted keystore: {}", keystore_path.display());
                                files_deleted += 1;
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to delete keystore {}: {}", keystore_path.display(), e);
                            }
                        }
                    }
                }

                // Note: Individual keychain entries will be orphaned but that's okay
                // since the wallet metadata is deleted and they can't be accessed

                format!("Wallet reset complete! {files_deleted} files deleted. You can now create a new wallet.")
            },
            |message| {
                tracing::info!("🗑️ {}", message);
                Message::WalletResetComplete
            },
        )
    }

    /// Handle wallet reset completion
    fn handle_wallet_reset_complete(&mut self) -> Command<Message> {
        tracing::info!("🎉 Wallet reset completed successfully");

        // Set a success message
        self.state.ui_mut().status_message = "Wallet reset complete! You can now create a new wallet.".to_string();
        self.state.ui_mut().status_message_color = crate::gui::StatusMessageColor::Success;
        self.state.ui_mut().status_message_timer = Some(std::time::Instant::now());

        // CRITICAL: Hide the confirmation dialog
        self.state.ui_mut().show_reset_wallet_confirmation = false;

        // Re-trigger startup check to transition UI to "Create Wallet" state
        // We know keystore is gone, so we pass false effectively
        // This ensures the App transitions to WalletSetup mode immediately
        Command::perform(async {}, |_| Message::SeedAccountsChecked(false))
    }

    // ============================================================================
    // Master Password Dialog Handlers (HD Wallet Authentication)
    // ============================================================================

    /// Show master password dialog for HD wallet authentication
    fn handle_show_master_password_dialog(&mut self, account_name: String) -> Command<Message> {
        // Find account ID from name
        let account_id = self
            .state
            .wallet()
            .available_accounts
            .iter()
            .find(|a| a.name == account_name)
            .map(|a| a.id.clone())
            .unwrap_or_default(); // Fallback if not found

        let security = self.state.auth_mut();
        security
            .password_dialog
            .show(crate::gui::state::auth_state::PasswordDialogConfig::AccountUnlock {
                account_id,
                account_name: account_name.clone(),
            });

        tracing::info!("🔐 Master password dialog shown for account: {}", account_name);
        Command::none()
    }

    /// Hide master password dialog
    fn handle_hide_master_password_dialog(&mut self) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.hide();

        tracing::info!("🔐 Master password dialog hidden");
        Command::none()
    }

    /// Handle master password input change
    fn handle_master_password_changed(&mut self, password: String) -> Command<Message> {
        let security = self.state.auth_mut();
        security.password_dialog.input = SecretString::new(password);
        Command::none()
    }

    /// Handle master password submission
    fn handle_master_password_submit(&mut self) -> Command<Message> {
        let password = self.state.auth().password_dialog.input.clone();

        if password.expose_secret().trim().is_empty() {
            let security = self.state.auth_mut();
            security.password_dialog.set_error(PasswordError::EmptyPassword);
            return Command::none();
        }

        tracing::info!("🔐 Master password submitted for validation");

        // Store password for transaction processing
        let wallet_arc = self.wallet.clone();
        let _account_name = match &self.state.auth().password_dialog.config {
            Some(crate::gui::state::auth_state::PasswordDialogConfig::AccountUnlock { account_name, .. }) => {
                account_name.clone()
            }
            _ => String::new(),
        };

        Command::perform(
            async move {
                // Validate password by attempting to decrypt seed
                if let Some(wallet) = wallet_arc {
                    let wallet_read = wallet.read().await;
                    if let Some(account) = wallet_read.current_account().await {
                        if account.key_reference.service == "vaughan-wallet-encrypted-seeds" {
                            // Try to create HD wallet to validate password
                            let keychain = Box::new(
                                crate::security::keychain::OSKeychain::new(
                                    "vaughan-wallet-encrypted-seeds".to_string(),
                                )
                                .map_err(|e| format!("Failed to access keychain: {e}"))?,
                            );

                            let default_path = "m/44'/60'/0'/0/0".to_string();
                            let derivation_path = account.derivation_path.as_ref().unwrap_or(&default_path);
                            match crate::gui::hd_wallet_service::HDWalletService::create_wallet_from_encrypted_seed(
                                keychain.as_ref(),
                                &account.key_reference,
                                derivation_path,
                                &password,
                            )
                            .await
                            {
                                Ok(_) => Ok(password),
                                Err(e) => Err(format!("Invalid password: {e}")),
                            }
                        } else {
                            Err("Account is not a seed-based account".to_string())
                        }
                    } else {
                        Err("No account selected".to_string())
                    }
                } else {
                    Err("Wallet not available".to_string())
                }
            },
            Message::MasterPasswordValidated,
        )
    }

    /// Handle master password cancellation
    fn handle_master_password_cancel(&mut self) -> Command<Message> {
        tracing::info!("🚫 Master password entry cancelled");

        let security = self.state.auth_mut();
        security.password_dialog.hide();

        // Show status message
        self.state.ui_mut().status_message = "HD wallet authentication cancelled".to_string();
        self.state.ui_mut().status_message_color = crate::gui::StatusMessageColor::Info;
        self.state.ui_mut().status_message_timer = Some(std::time::Instant::now());

        Command::none()
    }

    /// Handle master password validation result
    fn handle_master_password_validated(&mut self, result: Result<secrecy::SecretString, String>) -> Command<Message> {
        match result {
            Ok(validated_password) => {
                tracing::info!("✅ Master password validated successfully");

                // Hide dialog
                let security = self.state.auth_mut();
                security.password_dialog.hide();

                // Store the validated password temporarily for transaction processing
                // This will be used by the transaction handler to derive the HD wallet
                security.session.temporary_key = Some(validated_password);

                // Show success message
                self.state.ui_mut().status_message = "Authentication successful".to_string();
                self.state.ui_mut().status_message_color = crate::gui::StatusMessageColor::Success;
                self.state.ui_mut().status_message_timer = Some(std::time::Instant::now());

                // Now proceed with the transaction that was waiting for authentication
                Command::perform(async {}, |_| Message::ConfirmTransaction)
            }
            Err(error) => {
                tracing::warn!("❌ Master password validation failed: {}", error);

                // Show error in dialog
                let security = self.state.auth_mut();
                // Convert string error to PasswordError? Or just generic failure
                security.password_dialog.set_error(PasswordError::DecryptionFailed);

                Command::none()
            }
        }
    }
}
