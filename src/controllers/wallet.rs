//! Wallet Controller - Secure keyring management
//!
//! Follows MetaMask's KeyringController pattern for account management.
//!
//! ## Design Principles
//!
//! 1. **Secure Key Storage**: Uses `secrecy::Secret` for private keys
//! 2. **Alloy Signers**: Pure Alloy `PrivateKeySigner` for signing
//! 3. **Account Management**: HashMap-based keyring
//! 4. **MetaMask Patterns**: Account switching, signing, removal
//!
//! ## MetaMask Inspiration
//!
//! This controller implements patterns from MetaMask's KeyringController:
//! - Secure keyring with multiple accounts
//! - Active account tracking
//! - Message and transaction signing
//! - Account import/export
//! - Private key protection with Secrecy

use super::{ControllerError, ControllerResult};
use alloy::primitives::{Address, Signature};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Account entry in the keyring
///
/// Stores the signer and metadata for each account.
#[derive(Clone)]
struct AccountEntry {
    /// Alloy signer for this account
    signer: PrivateKeySigner,
    /// Account name/label
    name: String,
    /// Account address (cached for quick access)
    address: Address,
}

/// Wallet controller - manages keyring and accounts
///
/// This controller handles account creation, signing, and secure key storage
/// using Alloy signers and Secrecy for sensitive data.
///
/// ## Example
///
/// ```rust,no_run
/// use vaughan::controllers::WalletController;
/// use secrecy::SecretString;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut controller = WalletController::new();
///
/// // Add account from private key
/// let private_key = SecretString::new("0x...".to_string());
/// let address = controller.add_account(private_key, "My Account".to_string()).await?;
///
/// // Sign a message
/// let message = b"Hello, Ethereum!";
/// let signature = controller.sign_message(message).await?;
/// # Ok(())
/// # }
/// ```
pub struct WalletController {
    /// Keyring: Address → AccountEntry
    accounts: Arc<RwLock<HashMap<Address, AccountEntry>>>,
    /// Currently active account address
    active_account: Arc<RwLock<Option<Address>>>,
}

impl WalletController {
    /// Create new wallet controller
    ///
    /// Initializes an empty keyring with no accounts.
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            active_account: Arc::new(RwLock::new(None)),
        }
    }

    /// Add account from private key (Alloy signer)
    ///
    /// Imports a private key and creates an Alloy signer for it.
    /// Uses `secrecy::Secret` to protect the private key in memory.
    ///
    /// # Arguments
    ///
    /// * `private_key` - Private key as SecretString (hex format with or without 0x prefix)
    /// * `name` - Account name/label
    ///
    /// # Returns
    ///
    /// * `Ok(Address)` - Address of the imported account
    /// * `Err(ControllerError)` - Invalid private key or parsing error
    ///
    /// # Security
    ///
    /// - Private key is wrapped in `secrecy::Secret` to prevent accidental exposure
    /// - Key is only exposed during signer creation
    /// - Signer stores key securely in Alloy's internal format
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vaughan::controllers::WalletController;
    /// # use secrecy::SecretString;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut controller = WalletController::new();
    /// let private_key = SecretString::new("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string());
    /// let address = controller.add_account(private_key, "Account 1".to_string()).await?;
    /// println!("Imported account: {}", address);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_account(&self, private_key: SecretString, name: String) -> ControllerResult<Address> {
        // Parse private key (with or without 0x prefix)
        let key_str = private_key.expose_secret();
        let clean_key = key_str.strip_prefix("0x").unwrap_or(key_str);

        // Create Alloy signer from private key
        let signer = PrivateKeySigner::from_str(clean_key).map_err(|e| {
            ControllerError::Wallet(format!("Invalid private key: {}", e))
        })?;

        let address = signer.address();

        // Create account entry
        let entry = AccountEntry {
            signer,
            name,
            address,
        };

        // Add to keyring
        let mut accounts = self.accounts.write().await;
        accounts.insert(address, entry);

        // Set as active if no active account
        let mut active = self.active_account.write().await;
        if active.is_none() {
            *active = Some(address);
        }

        Ok(address)
    }

    /// Get current active account address
    ///
    /// Returns the address of the currently active account, or None if no accounts exist.
    ///
    /// # Returns
    ///
    /// * `Some(Address)` - Active account address
    /// * `None` - No active account
    pub async fn get_current_address(&self) -> Option<Address> {
        *self.active_account.read().await
    }

    /// Get account name by address
    ///
    /// # Arguments
    ///
    /// * `address` - Account address
    ///
    /// # Returns
    ///
    /// * `Some(String)` - Account name
    /// * `None` - Account not found
    pub async fn get_account_name(&self, address: Address) -> Option<String> {
        let accounts = self.accounts.read().await;
        accounts.get(&address).map(|entry| entry.name.clone())
    }

    /// List all account addresses
    ///
    /// Returns a vector of all account addresses in the keyring.
    ///
    /// # Returns
    ///
    /// Vector of account addresses
    pub async fn list_accounts(&self) -> Vec<Address> {
        let accounts = self.accounts.read().await;
        accounts.keys().copied().collect()
    }

    /// Sign a message with the active account (Alloy signer)
    ///
    /// Signs a message using the currently active account's Alloy signer.
    /// Follows EIP-191 personal message signing.
    ///
    /// # Arguments
    ///
    /// * `message` - Message bytes to sign
    ///
    /// # Returns
    ///
    /// * `Ok(Signature)` - Alloy signature
    /// * `Err(ControllerError)` - No active account or signing error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vaughan::controllers::WalletController;
    /// # async fn example(controller: &WalletController) -> Result<(), Box<dyn std::error::Error>> {
    /// let message = b"Hello, Ethereum!";
    /// let signature = controller.sign_message(message).await?;
    /// println!("Signature: {:?}", signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_message(&self, message: &[u8]) -> ControllerResult<Signature> {
        // Get active account
        let active_addr = self
            .active_account
            .read()
            .await
            .ok_or_else(|| ControllerError::Wallet("No active account".to_string()))?;

        // Get signer
        let accounts = self.accounts.read().await;
        let entry = accounts.get(&active_addr).ok_or_else(|| {
            ControllerError::Wallet(format!("Active account not found: {}", active_addr))
        })?;

        // Sign message with Alloy signer
        let signature = entry
            .signer
            .sign_message(message)
            .await
            .map_err(|e| ControllerError::Wallet(format!("Failed to sign message: {}", e)))?;

        Ok(signature)
    }

    /// Sign a message with a specific account
    ///
    /// Signs a message using a specific account's signer, regardless of which account is active.
    ///
    /// # Arguments
    ///
    /// * `address` - Account address to sign with
    /// * `message` - Message bytes to sign
    ///
    /// # Returns
    ///
    /// * `Ok(Signature)` - Alloy signature
    /// * `Err(ControllerError)` - Account not found or signing error
    pub async fn sign_message_with_account(
        &self,
        address: Address,
        message: &[u8],
    ) -> ControllerResult<Signature> {
        let accounts = self.accounts.read().await;
        let entry = accounts
            .get(&address)
            .ok_or_else(|| ControllerError::Wallet(format!("Account not found: {}", address)))?;

        let signature = entry
            .signer
            .sign_message(message)
            .await
            .map_err(|e| ControllerError::Wallet(format!("Failed to sign message: {}", e)))?;

        Ok(signature)
    }

    /// Switch to a different account
    ///
    /// Changes the active account to the specified address.
    /// Follows MetaMask's account switching pattern.
    ///
    /// # Arguments
    ///
    /// * `address` - Address of account to switch to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully switched accounts
    /// * `Err(ControllerError)` - Account not found
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vaughan::controllers::WalletController;
    /// # use alloy::primitives::Address;
    /// # async fn example(controller: &mut WalletController, address: Address) -> Result<(), Box<dyn std::error::Error>> {
    /// controller.switch_account(address).await?;
    /// println!("Switched to account: {}", address);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn switch_account(&self, address: Address) -> ControllerResult<()> {
        // Verify account exists
        let accounts = self.accounts.read().await;
        if !accounts.contains_key(&address) {
            return Err(ControllerError::Wallet(format!(
                "Account not found: {}",
                address
            )));
        }
        drop(accounts); // Release read lock

        // Update active account
        let mut active = self.active_account.write().await;
        *active = Some(address);

        Ok(())
    }

    /// Remove an account from the keyring
    ///
    /// Removes an account and its signer from the keyring.
    /// If the removed account was active, switches to another account if available.
    ///
    /// # Arguments
    ///
    /// * `address` - Address of account to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully removed account
    /// * `Err(ControllerError)` - Account not found
    ///
    /// # Security
    ///
    /// - Signer is dropped, which should zeroize the private key (Alloy internal)
    /// - Account cannot be recovered after removal
    pub async fn remove_account(&self, address: Address) -> ControllerResult<()> {
        let mut accounts = self.accounts.write().await;

        // Remove account
        accounts
            .remove(&address)
            .ok_or_else(|| ControllerError::Wallet(format!("Account not found: {}", address)))?;

        // Update active account if needed
        let mut active = self.active_account.write().await;
        if *active == Some(address) {
            // Switch to first available account, or None if no accounts left
            *active = accounts.keys().next().copied();
        }

        Ok(())
    }

    /// Get account count
    ///
    /// Returns the number of accounts in the keyring.
    pub async fn account_count(&self) -> usize {
        self.accounts.read().await.len()
    }

    /// Check if an account exists
    ///
    /// # Arguments
    ///
    /// * `address` - Address to check
    ///
    /// # Returns
    ///
    /// `true` if account exists, `false` otherwise
    pub async fn has_account(&self, address: Address) -> bool {
        self.accounts.read().await.contains_key(&address)
    }

    /// Get signer for active account (for advanced operations)
    ///
    /// Returns a clone of the active account's signer for operations
    /// not covered by the controller API.
    ///
    /// # Returns
    ///
    /// * `Ok(PrivateKeySigner)` - Cloned signer
    /// * `Err(ControllerError)` - No active account
    ///
    /// # Security Note
    ///
    /// This exposes the signer for advanced operations. Use with caution.
    pub async fn get_active_signer(&self) -> ControllerResult<PrivateKeySigner> {
        let active_addr = self
            .active_account
            .read()
            .await
            .ok_or_else(|| ControllerError::Wallet("No active account".to_string()))?;

        let accounts = self.accounts.read().await;
        let entry = accounts.get(&active_addr).ok_or_else(|| {
            ControllerError::Wallet(format!("Active account not found: {}", active_addr))
        })?;

        Ok(entry.signer.clone())
    }

    /// Get signer for specific account (for advanced operations)
    ///
    /// Returns a clone of a specific account's signer.
    ///
    /// # Arguments
    ///
    /// * `address` - Account address
    ///
    /// # Returns
    ///
    /// * `Ok(PrivateKeySigner)` - Cloned signer
    /// * `Err(ControllerError)` - Account not found
    pub async fn get_signer(&self, address: Address) -> ControllerResult<PrivateKeySigner> {
        let accounts = self.accounts.read().await;
        let entry = accounts
            .get(&address)
            .ok_or_else(|| ControllerError::Wallet(format!("Account not found: {}", address)))?;

        Ok(entry.signer.clone())
    }
}

impl Default for WalletController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test private key (Anvil default account 0)
    const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const TEST_PRIVATE_KEY_2: &str = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    #[tokio::test]
    async fn test_wallet_controller_creation() {
        let controller = WalletController::new();
        assert_eq!(controller.account_count().await, 0);
        assert!(controller.get_current_address().await.is_none());
    }

    #[tokio::test]
    async fn test_add_account_from_private_key() {
        let controller = WalletController::new();

        let private_key = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let result = controller
            .add_account(private_key, "Test Account".to_string())
            .await;

        assert!(result.is_ok());
        let address = result.unwrap();

        // Verify account was added
        assert_eq!(controller.account_count().await, 1);
        assert!(controller.has_account(address).await);

        // Verify it's set as active
        assert_eq!(controller.get_current_address().await, Some(address));

        // Verify name
        assert_eq!(
            controller.get_account_name(address).await,
            Some("Test Account".to_string())
        );
    }

    #[tokio::test]
    async fn test_add_account_with_0x_prefix() {
        let controller = WalletController::new();

        let private_key = SecretString::new(format!("0x{}", TEST_PRIVATE_KEY));
        let result = controller
            .add_account(private_key, "Test Account".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_invalid_private_key() {
        let controller = WalletController::new();

        let private_key = SecretString::new("invalid_key".to_string());
        let result = controller
            .add_account(private_key, "Test Account".to_string())
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::Wallet(msg) => {
                assert!(msg.contains("Invalid private key"));
            }
            _ => panic!("Expected Wallet error"),
        }
    }

    #[tokio::test]
    async fn test_switch_account() {
        let controller = WalletController::new();

        // Add two accounts
        let pk1 = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr1 = controller
            .add_account(pk1, "Account 1".to_string())
            .await
            .unwrap();

        let pk2 = SecretString::new(TEST_PRIVATE_KEY_2.to_string());
        let addr2 = controller
            .add_account(pk2, "Account 2".to_string())
            .await
            .unwrap();

        // First account should be active
        assert_eq!(controller.get_current_address().await, Some(addr1));

        // Switch to second account
        let result = controller.switch_account(addr2).await;
        assert!(result.is_ok());
        assert_eq!(controller.get_current_address().await, Some(addr2));

        // Switch back to first account
        let result = controller.switch_account(addr1).await;
        assert!(result.is_ok());
        assert_eq!(controller.get_current_address().await, Some(addr1));
    }

    #[tokio::test]
    async fn test_switch_to_nonexistent_account() {
        let controller = WalletController::new();

        let pk = SecretString::new(TEST_PRIVATE_KEY.to_string());
        controller
            .add_account(pk, "Account 1".to_string())
            .await
            .unwrap();

        // Try to switch to non-existent account
        let fake_address = Address::from([0x42; 20]);
        let result = controller.switch_account(fake_address).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::Wallet(msg) => {
                assert!(msg.contains("Account not found"));
            }
            _ => panic!("Expected Wallet error"),
        }
    }

    #[tokio::test]
    async fn test_sign_message() {
        let controller = WalletController::new();

        let pk = SecretString::new(TEST_PRIVATE_KEY.to_string());
        controller
            .add_account(pk, "Test Account".to_string())
            .await
            .unwrap();

        let message = b"Hello, Ethereum!";
        let result = controller.sign_message(message).await;

        assert!(result.is_ok());
        let signature = result.unwrap();

        // Verify signature is not empty (check first byte is non-zero)
        assert_ne!(signature.as_bytes()[0], 0u8);
    }

    #[tokio::test]
    async fn test_sign_message_no_active_account() {
        let controller = WalletController::new();

        let message = b"Hello, Ethereum!";
        let result = controller.sign_message(message).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::Wallet(msg) => {
                assert!(msg.contains("No active account"));
            }
            _ => panic!("Expected Wallet error"),
        }
    }

    #[tokio::test]
    async fn test_sign_message_with_specific_account() {
        let controller = WalletController::new();

        // Add two accounts
        let pk1 = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr1 = controller
            .add_account(pk1, "Account 1".to_string())
            .await
            .unwrap();

        let pk2 = SecretString::new(TEST_PRIVATE_KEY_2.to_string());
        let addr2 = controller
            .add_account(pk2, "Account 2".to_string())
            .await
            .unwrap();

        // Sign with specific account (not active)
        let message = b"Test message";
        let result = controller.sign_message_with_account(addr2, message).await;

        assert!(result.is_ok());

        // Active account should still be addr1
        assert_eq!(controller.get_current_address().await, Some(addr1));
    }

    #[tokio::test]
    async fn test_remove_account() {
        let controller = WalletController::new();

        // Add two accounts
        let pk1 = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr1 = controller
            .add_account(pk1, "Account 1".to_string())
            .await
            .unwrap();

        let pk2 = SecretString::new(TEST_PRIVATE_KEY_2.to_string());
        let addr2 = controller
            .add_account(pk2, "Account 2".to_string())
            .await
            .unwrap();

        assert_eq!(controller.account_count().await, 2);

        // Remove first account (active)
        let result = controller.remove_account(addr1).await;
        assert!(result.is_ok());

        // Verify account was removed
        assert_eq!(controller.account_count().await, 1);
        assert!(!controller.has_account(addr1).await);

        // Active account should switch to addr2
        assert_eq!(controller.get_current_address().await, Some(addr2));
    }

    #[tokio::test]
    async fn test_remove_last_account() {
        let controller = WalletController::new();

        let pk = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr = controller
            .add_account(pk, "Test Account".to_string())
            .await
            .unwrap();

        // Remove the only account
        let result = controller.remove_account(addr).await;
        assert!(result.is_ok());

        // No accounts left
        assert_eq!(controller.account_count().await, 0);
        assert!(controller.get_current_address().await.is_none());
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let controller = WalletController::new();

        // Add two accounts
        let pk1 = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr1 = controller
            .add_account(pk1, "Account 1".to_string())
            .await
            .unwrap();

        let pk2 = SecretString::new(TEST_PRIVATE_KEY_2.to_string());
        let addr2 = controller
            .add_account(pk2, "Account 2".to_string())
            .await
            .unwrap();

        let accounts = controller.list_accounts().await;
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains(&addr1));
        assert!(accounts.contains(&addr2));
    }

    #[tokio::test]
    async fn test_get_active_signer() {
        let controller = WalletController::new();

        let pk = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr = controller
            .add_account(pk, "Test Account".to_string())
            .await
            .unwrap();

        let result = controller.get_active_signer().await;
        assert!(result.is_ok());

        let signer = result.unwrap();
        assert_eq!(signer.address(), addr);
    }

    #[tokio::test]
    async fn test_get_signer_for_specific_account() {
        let controller = WalletController::new();

        let pk = SecretString::new(TEST_PRIVATE_KEY.to_string());
        let addr = controller
            .add_account(pk, "Test Account".to_string())
            .await
            .unwrap();

        let result = controller.get_signer(addr).await;
        assert!(result.is_ok());

        let signer = result.unwrap();
        assert_eq!(signer.address(), addr);
    }
}
