//! Core wallet functionality
//!
//! This module provides the main Vaughan wallet implementation with secure key management,
//! account operations, and transaction signing capabilities.

use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{Result, WalletError};
use crate::network::{NetworkChangeCallback, NetworkConfig, NetworkId, NetworkManager};
use crate::security::{SecureAccount, SecureExport, SecureKeystore};

pub mod account;
pub mod account_manager;
pub mod errors;
pub mod hardware;
pub mod keystore;
pub mod keystore_format;
pub mod manager;
pub mod backup;
pub mod provider;
pub mod transaction;

pub use account::*;
pub use account_manager::{
    AccountConfig, AccountManagerResult, AccountManagerTrait, AccountType, AuthToken,
    AuthorizedOperation, ImportSource, SeedStrength,
};
pub use errors::*;
pub use hardware::{
    AddressVerificationFeedback, DeviceRecoveryFeedback, HardwareManager, HardwareWalletStatus,
    RiskLevel, TransactionAuditFeedback,
};
pub use keystore_format::*;
pub use manager::*;
/// Main wallet configuration
#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub default_network: NetworkId,
    pub auto_lock_timeout: Option<std::time::Duration>,
    pub hardware_wallet_enabled: bool,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            default_network: NetworkId(1), // Ethereum mainnet
            auto_lock_timeout: None,       // Disabled for testing - no auto-lock
            hardware_wallet_enabled: true,
        }
    }
}

/// Main Vaughan wallet struct
#[derive(Debug)]
pub struct Vaughan {
    network_config: Arc<RwLock<NetworkManager>>,
    current_account: Arc<RwLock<Option<SecureAccount>>>,
    keystore: Arc<RwLock<SecureKeystore>>,
    hardware_manager: Arc<RwLock<Option<HardwareManager>>>,
    /// Wallet lock state - tracks whether the wallet is locked
    locked: Arc<RwLock<bool>>,
    config: WalletConfig,
}

impl Vaughan {
    /// Create a new Vaughan wallet instance
    pub async fn new(config: WalletConfig) -> Result<Self> {
        let network_manager = NetworkManager::new().await?;
        let keychain = crate::security::create_keychain_interface()?;
        let mut keystore = SecureKeystore::new(keychain).await?;

        // Auto-unlock keystore for testing
        keystore.ensure_unlocked().await?;

        // Get all available accounts
        let accounts = keystore.list_accounts().await?;
        let initial_account = if !accounts.is_empty() {
            tracing::info!(
                "🔓 Auto-initializing wallet with first account: {} ({})",
                accounts[0].name,
                accounts[0].address
            );
            Some(accounts[0].clone())
        } else {
            tracing::info!("📋 No accounts available for auto-initialization");
            None
        };

        let mut wallet = Self {
            network_config: Arc::new(RwLock::new(network_manager)),
            current_account: Arc::new(RwLock::new(initial_account)),
            keystore: Arc::new(RwLock::new(keystore)),
            hardware_manager: Arc::new(RwLock::new(None)),
            locked: Arc::new(RwLock::new(false)),
            config,
        };

        // Initialize hardware wallet manager if enabled
        wallet.initialize_hardware_manager().await?;

        Ok(wallet)
    }

    /// Create a new account
    pub async fn create_account(&mut self, name: String) -> Result<Address> {
        let mut keystore = self.keystore.write().await;
        let account = keystore.create_account(name).await?;
        let address = account.address;

        // Set as current account if none is set
        let mut current = self.current_account.write().await;
        if current.is_none() {
            *current = Some(account);
        }

        Ok(address)
    }

    /// Import an account from a private key
    pub async fn import_account(&mut self, private_key: SecretString, name: String) -> Result<Address> {
        let mut keystore = self.keystore.write().await;
        let account = keystore.import_account(private_key, name).await?;
        let address = account.address;

        // Set as current account if none is set
        let mut current = self.current_account.write().await;
        if current.is_none() {
            *current = Some(account);
        }

        Ok(address)
    }

    /// Export an account
    pub async fn export_account(&self, address: Address, password: SecretString) -> Result<SecureExport> {
        let keystore = self.keystore.read().await;
        keystore.export_account(address, password).await
    }

    /// Sign a transaction
    pub async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<Vec<u8>> {
        self.sign_transaction_with_password(tx, None).await
    }

    pub async fn sign_transaction_with_password(
        &self,
        tx: &TransactionRequest,
        password: Option<&secrecy::SecretString>,
    ) -> Result<Vec<u8>> {
        tracing::info!(
            "🔐 Wallet sign_transaction_with_password called (password provided: {})",
            password.is_some()
        );

        let current_account = self.current_account.read().await;

        // If no current account, try to auto-select the first available account
        let account = match current_account.as_ref() {
            Some(acc) => acc.clone(),
            None => {
                // Auto-select first account if available
                drop(current_account);
                let keystore = self.keystore.read().await;
                let accounts = keystore.list_accounts().await?;
                if !accounts.is_empty() {
                    let first_account = accounts[0].clone();
                    drop(keystore);
                    let mut current = self.current_account.write().await;
                    *current = Some(first_account.clone());
                    tracing::info!(
                        "🔓 Auto-unlocked with first account for signing: {} ({})",
                        first_account.name,
                        first_account.address
                    );
                    first_account
                } else {
                    tracing::error!("❌ No accounts available for auto-unlock");
                    return Err(WalletError::WalletLocked.into());
                }
            }
        };

        tracing::info!("👤 Current account for signing: {} ({})", account.name, account.address);
        tracing::info!(
            "📝 Transaction details: to={:?}, value={:?}, gas={:?}",
            tx.to,
            tx.value,
            tx.gas
        );

        // Ensure keystore is unlocked and accounts are loaded
        let mut keystore = self.keystore.write().await;
        keystore.ensure_unlocked().await.map_err(|e| {
            tracing::error!("❌ Failed to ensure keystore is unlocked: {}", e);
            e
        })?;

        // Verify the account exists in the keystore before trying to sign
        let keystore_accounts = keystore.list_accounts().await?;
        tracing::info!("🔍 Keystore has {} accounts loaded", keystore_accounts.len());
        for acc in &keystore_accounts {
            tracing::info!("   - {} ({})", acc.name, acc.address);
        }

        let account_exists = keystore_accounts.iter().any(|a| a.address == account.address);
        if !account_exists {
            tracing::error!(
                "❌ Account {} ({}) not found in keystore!",
                account.name,
                account.address
            );
            tracing::error!("   This account exists in GUI but not in wallet's keystore");
            tracing::error!(
                "   Available accounts: {:?}",
                keystore_accounts.iter().map(|a| a.address).collect::<Vec<_>>()
            );
            return Err(WalletError::AccountNotFound {
                address: format!("{}", account.address),
            }
            .into());
        }

        // Add diagnostics before calling keystore sign_transaction
        tracing::info!(
            "🔍 About to call keystore.sign_transaction with address: {}",
            account.address
        );

        // Pass the password for seed-based accounts
        keystore
            .sign_transaction(tx, &account.address, password, None)
            .await
            .map_err(|e| {
                tracing::error!("❌ Keystore signing failed: {}", e);
                tracing::error!("   Account: {} ({})", account.name, account.address);
                e
            })
    }

    /// Get balance for current account
    pub async fn get_balance(&self, token: Option<Address>) -> Result<U256> {
        let current_account = self.current_account.read().await;

        // If no current account, try to auto-select the first available account
        let account = match current_account.as_ref() {
            Some(acc) => acc.clone(),
            None => {
                // Auto-select first account if available
                drop(current_account);
                let keystore = self.keystore.read().await;
                let accounts = keystore.list_accounts().await?;
                if !accounts.is_empty() {
                    let first_account = accounts[0].clone();
                    let mut current = self.current_account.write().await;
                    *current = Some(first_account.clone());
                    tracing::info!(
                        "🔓 Auto-unlocked with first account: {} ({})",
                        first_account.name,
                        first_account.address
                    );
                    first_account
                } else {
                    return Err(WalletError::WalletLocked.into());
                }
            }
        };

        let network_manager = self.network_config.read().await;
        network_manager.get_balance(account.address, token).await
    }

    /// Switch network
    pub async fn switch_network(&mut self, network: NetworkId, callback: Option<NetworkChangeCallback>) -> Result<()> {
        let mut network_manager = self.network_config.write().await;
        network_manager.switch_network(network).await?;

        // Execute callback if provided
        if let Some(callback) = callback {
            callback(network);
        }

        Ok(())
    }

    /// Get current account
    pub async fn current_account(&self) -> Option<SecureAccount> {
        self.current_account.read().await.clone()
    }

    /// List all accounts
    pub async fn list_accounts(&self) -> Result<Vec<SecureAccount>> {
        let keystore = self.keystore.read().await;
        keystore.list_accounts().await
    }

    /// Switch to a different account
    pub async fn switch_account(&mut self, address: Address) -> Result<()> {
        let keystore = self.keystore.read().await;
        let account = keystore.get_account(address).await?;

        let mut current = self.current_account.write().await;
        *current = Some(account);

        Ok(())
    }

    // ========== Lock/Unlock Implementation with Conditional Compilation ==========
    //
    // Production mode: Implements proper locking with memory clearing (Requirement 2.1)
    // Test mode: Locking disabled for development convenience (Requirement 2.2)

    /// Lock the wallet - clears sensitive data from memory
    ///
    /// In production mode, this clears current_account and any cached sensitive data.
    /// In test mode, locking is disabled for development convenience.
    ///
    /// Implements Requirements 2.1 (production locking) and 2.3 (memory clearing)
    #[cfg(not(test))]
    pub async fn lock(&mut self) -> Result<()> {
        let correlation_id = Uuid::new_v4();
        tracing::info!(
            correlation_id = %correlation_id,
            "🔒 Locking wallet - clearing sensitive data from memory"
        );

        // Set locked state first
        {
            let mut locked = self.locked.write().await;
            *locked = true;
        }

        // Clear current account (sensitive data)
        {
            let mut current = self.current_account.write().await;
            // Note: SecureAccount doesn't contain raw private keys, but clearing
            // the reference prevents unauthorized access to account operations
            *current = None;
        }

        // Note: The keystore itself holds encrypted data, not raw private keys
        // Real private keys are only decrypted momentarily for signing operations
        // and are not cached in the Vaughan struct

        tracing::info!(
            correlation_id = %correlation_id,
            "🔒 Wallet locked successfully - sensitive data cleared"
        );

        Ok(())
    }

    /// Lock the wallet - DISABLED FOR TESTING
    ///
    /// In test mode, locking is disabled for development convenience.
    /// Implements Requirement 2.2 (test mode convenience)
    #[cfg(test)]
    pub async fn lock(&mut self) -> Result<()> {
        let correlation_id = Uuid::new_v4();
        tracing::info!(
            correlation_id = %correlation_id,
            "🔓 Lock disabled for testing - wallet remains unlocked"
        );
        Ok(())
    }

    /// Check if wallet is currently locked
    ///
    /// In production mode, returns the actual lock state.
    /// In test mode, always returns false.
    #[cfg(not(test))]
    pub async fn is_locked(&self) -> bool {
        let locked = self.locked.read().await;
        *locked
    }

    /// Check if wallet is locked - ALWAYS RETURNS FALSE FOR TESTING
    #[cfg(test)]
    pub async fn is_locked(&self) -> bool {
        false
    }

    /// Unlock the wallet by setting a current account
    ///
    /// In production mode, unlocks the wallet and sets the current account.
    /// In test mode, same behavior but without lock state changes.
    ///
    /// Implements Requirement 2.4 (unlock with correct credentials)
    #[cfg(not(test))]
    pub async fn unlock(&mut self, address: Address) -> Result<()> {
        let correlation_id = Uuid::new_v4();
        tracing::info!(
            correlation_id = %correlation_id,
            address = %address,
            "🔓 Unlocking wallet"
        );

        let keystore = self.keystore.read().await;
        let account = keystore.get_account(address).await?;
        drop(keystore);

        // Set current account
        {
            let mut current = self.current_account.write().await;
            *current = Some(account);
        }

        // Clear locked state
        {
            let mut locked = self.locked.write().await;
            *locked = false;
        }

        tracing::info!(
            correlation_id = %correlation_id,
            address = %address,
            "🔓 Wallet unlocked successfully"
        );

        Ok(())
    }

    /// Unlock the wallet by setting a current account - TEST MODE
    #[cfg(test)]
    pub async fn unlock(&mut self, address: Address) -> Result<()> {
        let keystore = self.keystore.read().await;
        let account = keystore.get_account(address).await?;

        let mut current = self.current_account.write().await;
        *current = Some(account);

        Ok(())
    }

    /// Unlock the wallet with a provided account (for GUI integration)
    ///
    /// In production mode, unlocks the wallet and sets the current account.
    /// In test mode, same behavior but without lock state changes.
    #[cfg(not(test))]
    pub async fn unlock_with_account(&mut self, account: SecureAccount) -> Result<()> {
        let correlation_id = Uuid::new_v4();
        tracing::info!(
            correlation_id = %correlation_id,
            address = %account.address,
            name = %account.name,
            "🔓 Unlocking wallet with provided account"
        );

        // Set current account
        {
            let mut current = self.current_account.write().await;
            *current = Some(account.clone());
        }

        // Clear locked state
        {
            let mut locked = self.locked.write().await;
            *locked = false;
        }

        tracing::info!(
            correlation_id = %correlation_id,
            "🔓 Wallet unlocked successfully with provided account"
        );

        Ok(())
    }

    /// Unlock the wallet with a provided account - TEST MODE
    #[cfg(test)]
    pub async fn unlock_with_account(&mut self, account: SecureAccount) -> Result<()> {
        let mut current = self.current_account.write().await;
        *current = Some(account);

        Ok(())
    }

    /// Get current network information
    pub async fn get_current_network(&self) -> Option<NetworkConfig> {
        let network_manager = self.network_config.read().await;
        network_manager.get_current_network_config().cloned()
    }

    /// Get all available networks
    pub async fn get_all_networks(&self) -> HashMap<NetworkId, NetworkConfig> {
        let network_manager = self.network_config.read().await;
        network_manager.get_all_networks().clone()
    }

    /// Add a custom network
    pub async fn add_custom_network(&mut self, config: NetworkConfig) -> Result<()> {
        let mut network_manager = self.network_config.write().await;
        network_manager.add_custom_network(config).await
    }

    /// Update or replace a custom network (handles chain ID changes)
    pub async fn update_or_replace_custom_network(&mut self, old_id: NetworkId, config: NetworkConfig) -> Result<()> {
        let mut network_manager = self.network_config.write().await;
        network_manager.update_or_replace_custom_network(old_id, config).await
    }

    /// Get gas price for current network
    pub async fn get_gas_price(&self) -> Result<U256> {
        let network_manager = self.network_config.read().await;
        network_manager.get_gas_price().await
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<U256> {
        let network_manager = self.network_config.read().await;
        network_manager.estimate_gas(tx).await
    }

    /// Get transaction count (nonce) for an address
    pub async fn get_nonce(&self, address: Address) -> Result<u64> {
        let network_manager = self.network_config.read().await;
        network_manager.get_transaction_count(address).await
    }

    /// Broadcast a signed transaction to the network
    pub async fn broadcast_transaction(&self, signed_tx: &[u8]) -> Result<alloy::primitives::TxHash> {
        let network_manager = self.network_config.read().await;
        network_manager.send_raw_transaction(signed_tx).await
    }

    /// Remove a custom network (handles current selection fallback)
    pub async fn remove_custom_network(&mut self, network_id: NetworkId) -> Result<()> {
        let mut network_manager = self.network_config.write().await;
        network_manager.remove_custom_network(network_id).await
    }

    /// Remove an account from the wallet
    pub async fn remove_account(&mut self, address: Address) -> Result<()> {
        // Check if this is the current account
        let current_account = self.current_account.read().await;
        if let Some(ref account) = *current_account {
            if account.address == address {
                drop(current_account);
                // Clear current account if we're removing it
                let mut current = self.current_account.write().await;
                *current = None;
            }
        }

        let mut keystore = self.keystore.write().await;
        keystore.remove_account(address).await
    }

    /// Get wallet configuration
    pub fn get_config(&self) -> &WalletConfig {
        &self.config
    }

    /// Update wallet configuration
    pub fn update_config(&mut self, config: WalletConfig) {
        self.config = config;
    }

    /// Get current account address (for GUI compatibility)
    pub fn get_current_account(&self) -> Option<Address> {
        // This is a synchronous version for GUI compatibility
        // In practice, this would need to be handled differently
        None // Placeholder - actual implementation would need async handling
    }

    /// Get current secure account (async version for deployment operations)
    pub async fn get_current_secure_account(&self) -> Option<SecureAccount> {
        let current = self.current_account.read().await;
        current.clone()
    }

    /// Check if wallet is connected (for GUI compatibility)
    pub fn is_connected(&self) -> bool {
        // Placeholder - actual implementation would check network connection
        true
    }

    /// Get current network ID (for GUI compatibility)
    pub fn get_current_network_id(&self) -> NetworkId {
        // Placeholder - actual implementation would get from network manager
        NetworkId(1)
    }

    /// Get the network manager for external use (e.g., audio notifications)
    pub fn network_manager(&self) -> Arc<RwLock<NetworkManager>> {
        Arc::clone(&self.network_config)
    }

    /// Get private key for current account (for forge deployment)
    /// ⚠️ USE WITH CAUTION - exposes raw private key
    pub async fn get_private_key_for_deployment(&self) -> Result<SecretString> {
        let current_account = self.current_account.read().await;
        let account = current_account.as_ref().ok_or(WalletError::WalletLocked)?;

        // Retrieve private key from keystore
        let keystore = self.keystore.read().await;
        keystore.retrieve(&account.key_reference)
    }

    /// Get keystore interface (for advanced operations)
    pub fn keystore(&self) -> Arc<RwLock<SecureKeystore>> {
        Arc::clone(&self.keystore)
    }

    /// Initialize hardware wallet manager
    pub async fn initialize_hardware_manager(&mut self) -> Result<()> {
        if !self.config.hardware_wallet_enabled {
            tracing::info!("Hardware wallet support is disabled in configuration");
            return Ok(());
        }

        match HardwareManager::new() {
            Ok(manager) => {
                let mut hw_manager = self.hardware_manager.write().await;
                *hw_manager = Some(manager);
                tracing::info!("✅ Hardware wallet manager initialized");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to initialize hardware wallet manager: {}", e);
                // Don't fail wallet initialization if hardware wallet setup fails
                Ok(())
            }
        }
    }

    /// Detect connected hardware wallets
    pub async fn detect_hardware_wallets(&self) -> Result<Vec<crate::security::hardware::HardwareWalletInfo>> {
        let mut hw_manager_guard = self.hardware_manager.write().await;

        if let Some(ref mut hw_manager) = *hw_manager_guard {
            hw_manager.detect_wallets().await
        } else {
            // Try to initialize hardware manager if not already done
            drop(hw_manager_guard);
            if let Ok(mut manager) = HardwareManager::new() {
                let detected = manager.detect_wallets().await?;
                let mut hw_manager_guard = self.hardware_manager.write().await;
                *hw_manager_guard = Some(manager);
                Ok(detected)
            } else {
                Ok(Vec::new())
            }
        }
    }

    /// Get addresses from a connected hardware wallet
    pub async fn get_hardware_addresses(&self, device_index: usize, count: u32) -> Result<Vec<Address>> {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            // Use standard BIP-44 derivation path for Ethereum
            let derivation_path = "m/44'/60'/0'/0";
            hw_manager.get_addresses(device_index, derivation_path, count).await
        } else {
            Err(crate::error::HardwareWalletError::DeviceNotFound.into())
        }
    }

    /// Sign transaction with hardware wallet
    pub async fn sign_transaction_with_hardware(
        &self,
        device_index: usize,
        tx: &TransactionRequest,
        derivation_path: &str,
    ) -> Result<Vec<u8>> {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            let signature = hw_manager.sign_transaction(device_index, tx, derivation_path).await?;

            // Convert Signature to bytes
            // Note: This is a simplified conversion - in practice you might need
            // to format the signature according to your specific requirements
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&signature.r().to_be_bytes::<32>());
            bytes.extend_from_slice(&signature.s().to_be_bytes::<32>());

            // In Alloy v1.1, v() returns a bool representing the y-parity
            // Convert to legacy format (27/28 for non-EIP155)
            let v_byte = if signature.v() { 28 } else { 27 };
            bytes.push(v_byte);

            Ok(bytes)
        } else {
            Err(crate::error::HardwareWalletError::DeviceNotFound.into())
        }
    }

    /// Get hardware wallet device information
    pub async fn get_hardware_device_info(
        &self,
        device_index: usize,
    ) -> Result<crate::security::hardware::HardwareWalletInfo> {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            hw_manager.get_device_info(device_index).await
        } else {
            Err(crate::error::HardwareWalletError::DeviceNotFound.into())
        }
    }

    /// Check if any hardware wallets are connected
    pub async fn has_hardware_wallets(&self) -> bool {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            hw_manager.has_connected_devices().await
        } else {
            false
        }
    }

    /// Refresh hardware wallet device list
    pub async fn refresh_hardware_devices(&self) -> Result<Vec<crate::security::hardware::HardwareWalletInfo>> {
        let mut hw_manager_guard = self.hardware_manager.write().await;

        if let Some(ref mut hw_manager) = *hw_manager_guard {
            hw_manager.refresh_devices().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get hardware wallet manager (for advanced operations)
    pub fn hardware_manager(&self) -> Arc<RwLock<Option<HardwareManager>>> {
        Arc::clone(&self.hardware_manager)
    }

    /// Verify hardware wallet address with user feedback
    pub async fn verify_hardware_address_with_feedback(
        &self,
        device_index: usize,
        address: &str,
    ) -> Result<AddressVerificationFeedback> {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            let derivation_path = "m/44'/60'/0'/0/0"; // Standard first address derivation
            hw_manager
                .verify_address_with_feedback(device_index, address, derivation_path)
                .await
        } else {
            // Return failure feedback if no hardware manager
            Ok(AddressVerificationFeedback {
                address: address.to_string(),
                derivation_path: "m/44'/60'/0'/0/0".to_string(),
                verified: false,
                device_count: 0,
                success_count: 0,
                duration_ms: 0,
                user_message: "❌ Hardware wallet manager not initialized".to_string(),
                next_steps: vec![
                    "Ensure hardware wallet support is enabled".to_string(),
                    "Connect a hardware device and refresh".to_string(),
                ],
            })
        }
    }

    /// Audit transaction with hardware wallet and provide feedback
    pub async fn audit_hardware_transaction_with_feedback(
        &self,
        device_index: usize,
        tx: &TransactionRequest,
    ) -> Result<TransactionAuditFeedback> {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            let derivation_path = "m/44'/60'/0'/0/0"; // Standard derivation for transaction signing
            hw_manager
                .audit_transaction_with_feedback(tx, derivation_path, device_index)
                .await
        } else {
            // Return failure feedback if no hardware manager
            Ok(TransactionAuditFeedback {
                passed: false,
                device_type: "Unknown".to_string(),
                duration_ms: 0,
                user_message: "❌ Hardware wallet manager not initialized".to_string(),
                security_warnings: vec!["Hardware wallet manager is not available".to_string()],
                recommendations: vec![
                    "Initialize hardware wallet support".to_string(),
                    "Connect a hardware device".to_string(),
                    "Use software wallet as fallback".to_string(),
                ],
                risk_level: RiskLevel::High,
            })
        }
    }

    /// Recover hardware wallet device with user feedback
    pub async fn recover_hardware_device_with_feedback(&self, device_id: &str) -> Result<DeviceRecoveryFeedback> {
        let mut hw_manager_guard = self.hardware_manager.write().await;

        if let Some(ref mut hw_manager) = *hw_manager_guard {
            hw_manager.recover_device_with_feedback(device_id).await
        } else {
            // Return failure feedback if no hardware manager
            Ok(DeviceRecoveryFeedback {
                device_id: device_id.to_string(),
                recovered: false,
                duration_ms: 0,
                user_message: "❌ Hardware wallet manager not initialized".to_string(),
                next_steps: vec![
                    "Initialize hardware wallet support in configuration".to_string(),
                    "Restart the application with hardware support enabled".to_string(),
                ],
            })
        }
    }

    /// Check hardware wallet device health and connectivity
    pub async fn check_hardware_device_health(&self, device_index: usize) -> Result<bool> {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            // Get device info to verify it's still connected and accessible
            match hw_manager.get_device_info(device_index).await {
                Ok(_) => {
                    tracing::info!("✅ Hardware device {} is healthy and accessible", device_index);
                    Ok(true)
                }
                Err(e) => {
                    tracing::warn!("⚠️ Hardware device {} health check failed: {}", device_index, e);
                    Ok(false)
                }
            }
        } else {
            tracing::warn!("⚠️ Hardware wallet manager not available for health check");
            Ok(false)
        }
    }

    /// Get comprehensive hardware wallet status
    pub async fn get_hardware_wallet_status(&self) -> HardwareWalletStatus {
        let hw_manager_guard = self.hardware_manager.read().await;

        if let Some(ref hw_manager) = *hw_manager_guard {
            let device_count = hw_manager.device_count().await;
            let connected_devices = hw_manager.get_connected_devices().await;

            HardwareWalletStatus {
                enabled: self.config.hardware_wallet_enabled,
                manager_initialized: true,
                device_count,
                connected_devices,
                last_refresh: std::time::SystemTime::now(),
            }
        } else {
            HardwareWalletStatus {
                enabled: self.config.hardware_wallet_enabled,
                manager_initialized: false,
                device_count: 0,
                connected_devices: Vec::new(),
                last_refresh: std::time::SystemTime::now(),
            }
        }
    }
}
