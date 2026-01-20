//! Secure Keystore Implementation
//!
//! This module provides secure storage and management of private keys with
//! automatic memory zeroization and OS keychain integration.
//!
//! ## Module Structure
//! - `storage` - Persistent account/network storage
//! - `encryption` - AES-256-GCM encryption utilities

pub mod encryption;
pub mod storage;

use crate::error::{Result, SecurityError};
use crate::network::{NetworkConfig, NetworkId};
use crate::security::{EncryptionType, KeyReference, KeychainInterface, SecureAccount, SecureExport};
use alloy::{
    network::TxSigner,
    primitives::{Address, TxKind},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use k256::ecdsa::SigningKey;
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;
use uuid::Uuid;

// Re-export storage types for convenience
pub use storage::{StoredAccountMeta, StoredNetworkMeta};

/// Secure keystore implementation
#[derive(Debug)]
pub struct SecureKeystoreImpl {
    accounts: HashMap<Address, SecureAccount>,
    custom_networks: HashMap<NetworkId, NetworkConfig>,
    keychain: Box<dyn KeychainInterface>,
    is_locked: bool,
    #[allow(dead_code)] // Stored for future keychain operations
    service_name: String,
}

impl SecureKeystoreImpl {
    /// Create a new secure keystore
    pub async fn new(keychain: Box<dyn KeychainInterface>) -> Result<Self> {
        let mut keystore = Self {
            accounts: HashMap::new(),
            custom_networks: HashMap::new(),
            keychain,
            is_locked: false,
            service_name: "vaughan-wallet".to_string(),
        };

        // Load existing accounts and networks from persistent storage
        keystore.reload_accounts().await?;
        keystore.reload_networks().await?;

        Ok(keystore)
    }

    /// Create a new account with generated private key
    pub async fn create_account(&mut self, name: String) -> Result<SecureAccount> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked".to_string(),
            }
            .into());
        }

        // Generate a new private key
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let wallet = PrivateKeySigner::from(signing_key.clone());
        let address = wallet.address();

        // Create key reference
        let account_id = Uuid::new_v4().to_string();
        let key_ref = KeyReference {
            id: account_id.clone(),
            service: "vaughan-wallet".to_string(),
            account: format!("{address}"),
        };

        // Store private key in keychain
        let private_key_hex = hex::encode(signing_key.to_bytes());
        let secret_key = SecretString::new(private_key_hex);

        self.keychain.store(&key_ref, secret_key)?;

        // Create secure account
        let account = SecureAccount {
            id: account_id,
            name,
            address,
            key_reference: key_ref,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: None,
        };

        self.accounts.insert(address, account.clone());

        // Save accounts to persistent storage
        self.save_accounts().await?;

        Ok(account)
    }

    /// Import an account with an existing key reference (for encrypted seed phrases)
    pub async fn import_account_with_key_reference(
        &mut self,
        name: String,
        address: Address,
        key_reference: KeyReference,
    ) -> Result<SecureAccount> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked".to_string(),
            }
            .into());
        }

        // Check if account already exists
        if self.accounts.contains_key(&address) {
            return Err(SecurityError::KeystoreError {
                message: "Account already exists".to_string(),
            }
            .into());
        }

        // Create secure account using the provided key reference
        let account = SecureAccount {
            id: key_reference.id.clone(),
            name,
            address,
            key_reference,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        };

        self.accounts.insert(address, account.clone());

        // Save accounts to persistent storage
        self.save_accounts().await?;

        Ok(account)
    }

    /// Import an account from private key
    pub async fn import_account(&mut self, private_key: SecretString, name: String) -> Result<SecureAccount> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked".to_string(),
            }
            .into());
        }

        // Validate and create wallet from private key
        let private_key_str = private_key.expose_secret();

        // Remove 0x prefix if present
        let clean_key = if private_key_str.starts_with("0x") {
            &private_key_str[2..]
        } else {
            private_key_str
        };

        // Validate hex format and length
        if clean_key.len() != 64 {
            return Err(SecurityError::InvalidPrivateKey.into());
        }

        let key_bytes = hex::decode(clean_key).map_err(|_| SecurityError::InvalidPrivateKey)?;

        if key_bytes.len() != 32 {
            return Err(SecurityError::InvalidPrivateKey.into());
        }

        let signing_key = SigningKey::from_bytes(
            key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| SecurityError::InvalidPrivateKey)?,
        )
        .map_err(|_| SecurityError::InvalidPrivateKey)?;

        let wallet = PrivateKeySigner::from(signing_key);
        let address = wallet.address();

        // Check if account already exists
        if self.accounts.contains_key(&address) {
            return Err(SecurityError::KeystoreError {
                message: "Account already exists".to_string(),
            }
            .into());
        }

        // Create key reference
        let account_id = Uuid::new_v4().to_string();
        let key_ref = KeyReference {
            id: account_id.clone(),
            service: "vaughan-wallet".to_string(),
            account: format!("{address}"),
        };

        // Store private key in keychain
        self.keychain.store(&key_ref, private_key)?;

        // Create secure account
        let account = SecureAccount {
            id: account_id,
            name,
            address,
            key_reference: key_ref,
            created_at: chrono::Utc::now(),
            is_hardware: false,
            derivation_path: None,
        };

        self.accounts.insert(address, account.clone());

        // Save accounts to persistent storage
        self.save_accounts().await?;

        Ok(account)
    }

    /// Export an account with password encryption
    pub async fn export_account(&self, address: Address, password: SecretString) -> Result<SecureExport> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked".to_string(),
            }
            .into());
        }

        let account = self
            .accounts
            .get(&address)
            .ok_or(SecurityError::InvalidAddress(address.to_string()))?;

        // Retrieve private key from keychain
        let private_key = self.keychain.retrieve(&account.key_reference)?;

        // Encrypt private key with password
        let encrypted_data = encryption::encrypt_with_password(
            private_key.expose_secret().as_bytes(),
            password.expose_secret().as_bytes(),
        )?;

        Ok(SecureExport {
            encrypted_data,
            encryption_type: EncryptionType::Aes256Gcm,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Sign a transaction and return the signed transaction bytes
    ///
    /// For seed-based accounts, requires a password to decrypt the seed.
    /// The key cache can be used to avoid repeated password prompts.
    pub async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        address: &Address,
        password: Option<&SecretString>,
        key_cache: Option<&mut crate::security::KeyCache>,
    ) -> Result<Vec<u8>> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked".to_string(),
            }
            .into());
        }

        tracing::info!("🔍 Attempting to sign transaction for address: {}", address);
        tracing::info!(
            "📋 Available accounts in keystore: {:?}",
            self.accounts.keys().collect::<Vec<_>>()
        );
        tracing::info!("🔓 Keystore locked status: {}", self.is_locked);

        let account = self.accounts.get(address).ok_or_else(|| {
            let available_addresses: Vec<String> = self.accounts.keys().map(|addr| addr.to_string()).collect();
            let error_msg = if available_addresses.is_empty() {
                "No accounts found in keystore - keystore might be locked or empty".to_string()
            } else {
                format!(
                    "Account not found in keystore. Requested: {}, Available accounts: [{}]",
                    address,
                    available_addresses.join(", ")
                )
            };
            tracing::error!("❌ Account lookup failed: {}", error_msg);
            SecurityError::InvalidAddress(error_msg)
        })?;

        // Check if this is a seed-based account or private-key account
        let is_seed_based = account.key_reference.service == "vaughan-wallet-encrypted-seeds";

        let key_bytes = if is_seed_based {
            tracing::info!("🌱 Seed-based account detected");

            // Check key cache first
            if let Some(cache) = key_cache {
                if let Some(cached_key) = cache.get(address) {
                    tracing::info!("🔑 Using cached key for address: {}", address);
                    cached_key
                } else {
                    // Not in cache - need to derive from seed
                    tracing::info!("🔓 Key not in cache, deriving from seed");

                    // Require password for seed-based accounts
                    let password = password.ok_or_else(|| SecurityError::KeystoreError {
                        message: "Password required for seed-based account".to_string(),
                    })?;

                    // Decrypt seed with password
                    let seed_storage = crate::security::SecureSeedStorage::new(self.keychain.clone_box());
                    let seed_phrase =
                        crate::security::decrypt_seed_with_password(&seed_storage, &account.key_reference, password)
                            .await?;

                    // Derive private key from seed
                    let derivation_path = account.derivation_path.as_deref();
                    let mut secure_key = crate::security::derive_key_from_seed(
                        self.keychain.clone_box(),
                        &seed_phrase,
                        derivation_path,
                    )?;

                    // Get key bytes
                    let key_bytes = secure_key.as_mut_slice().to_vec();

                    // Cache the key for future use
                    cache.insert(*address, key_bytes.clone())?;
                    tracing::info!("🔑 Derived and cached key for address: {}", address);

                    key_bytes
                }
            } else {
                // No cache provided - derive without caching
                tracing::info!("🔓 No cache provided, deriving key without caching");

                // Require password
                let password = password.ok_or_else(|| SecurityError::KeystoreError {
                    message: "Password required for seed-based account".to_string(),
                })?;

                // Decrypt seed with password
                let seed_storage = crate::security::SecureSeedStorage::new(self.keychain.clone_box());
                let seed_phrase =
                    crate::security::decrypt_seed_with_password(&seed_storage, &account.key_reference, password)
                        .await?;

                // Derive private key from seed
                let derivation_path = account.derivation_path.as_deref();
                let mut secure_key =
                    crate::security::derive_key_from_seed(self.keychain.clone_box(), &seed_phrase, derivation_path)?;

                // Get key bytes (will be zeroized when secure_key drops)
                secure_key.as_mut_slice().to_vec()
            }
        } else {
            // For private-key accounts, retrieve directly from keychain
            tracing::info!("🔑 Private-key account detected, retrieving from keychain");

            let private_key = self.keychain.retrieve(&account.key_reference)?;
            let private_key_str = private_key.expose_secret();
            let clean_key = if private_key_str.starts_with("0x") {
                &private_key_str[2..]
            } else {
                private_key_str
            };

            hex::decode(clean_key).map_err(|_| SecurityError::InvalidPrivateKey)?
        };

        let signing_key = SigningKey::from_bytes(
            key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| SecurityError::InvalidPrivateKey)?,
        )
        .map_err(|_| SecurityError::InvalidPrivateKey)?;

        // Build Alloy signer from the private key and use its built-in transaction signing
        let signer = PrivateKeySigner::from(signing_key);

        // Sanity-check: the signer's derived address must match the requested account address
        let signer_addr = signer.address();
        if &signer_addr != address {
            tracing::error!(
                "❌ Signer address ({}) does not match requested address ({})",
                signer_addr,
                address
            );
            return Err(SecurityError::InvalidAddress(format!(
                "Signer address {signer_addr} does not match requested address {address}"
            ))
            .into());
        }

        tracing::info!(
            "🧾 Signing tx via Alloy signer: to={:?}, value={:?}, nonce={:?}, chain_id={:?}",
            tx.to,
            tx.value,
            tx.nonce,
            tx.chain_id
        );

        // Convert the generic request into a concrete transaction for signing
        use alloy::consensus::{TxEip1559, TxLegacy};
        use alloy::primitives::Bytes;

        let chain_id = tx.chain_id.unwrap_or(1u64);
        let nonce = tx.nonce.unwrap_or(0u64);
        let gas_limit = tx.gas.unwrap_or(21_000u64);
        let value = tx.value.unwrap_or_default();
        let input_data: Bytes = tx.input.input.clone().unwrap_or_default();

        let to_kind: TxKind = match &tx.to {
            Some(TxKind::Call(addr)) => TxKind::Call(*addr),
            _ => TxKind::Create,
        };

        // Sign and encode transaction based on type using Alloy's consensus types
        use alloy::consensus::TxEnvelope;
        use alloy::rlp::Encodable;

        let raw_bytes: Vec<u8> =
            if let (Some(max_fee), Some(max_prio)) = (tx.max_fee_per_gas, tx.max_priority_fee_per_gas) {
                // EIP-1559 transaction
                let mut eip1559_tx = TxEip1559 {
                    chain_id,
                    nonce,
                    max_priority_fee_per_gas: max_prio,
                    max_fee_per_gas: max_fee,
                    gas_limit,
                    to: to_kind,
                    value,
                    input: input_data,
                    access_list: Default::default(),
                };

                // Use TxSigner to sign and get the signature
                let signature =
                    signer
                        .sign_transaction(&mut eip1559_tx)
                        .await
                        .map_err(|e| SecurityError::KeystoreError {
                            message: format!("Failed to sign EIP-1559 tx: {e}"),
                        })?;

                // Build the signed transaction
                use alloy::consensus::Signed;
                let signed_tx = Signed::new_unchecked(eip1559_tx, signature, Default::default());

                // Build the signed envelope
                let envelope = TxEnvelope::from(signed_tx);

                // Encode to bytes
                let mut buf = Vec::new();
                envelope.encode(&mut buf);
                buf
            } else {
                // Legacy transaction
                let gas_price = tx.gas_price.unwrap_or(20_000_000_000u128);
                let mut legacy_tx = TxLegacy {
                    chain_id: Some(chain_id),
                    nonce,
                    gas_price,
                    gas_limit,
                    to: to_kind,
                    value,
                    input: input_data,
                };

                // Use TxSigner to sign and get the signature
                let signature =
                    signer
                        .sign_transaction(&mut legacy_tx)
                        .await
                        .map_err(|e| SecurityError::KeystoreError {
                            message: format!("Failed to sign legacy tx: {e}"),
                        })?;

                // Build the signed transaction
                use alloy::consensus::Signed;
                let signed_tx = Signed::new_unchecked(legacy_tx, signature, Default::default());

                // Build the signed envelope
                let envelope = TxEnvelope::from(signed_tx);

                // Encode to bytes
                let mut buf = Vec::new();
                envelope.encode(&mut buf);
                buf
            };

        // Basic diagnostics
        tracing::info!("✅ Transaction signed with Alloy signer for address: {}", address);
        tracing::info!("📦 Encoded transaction length: {} bytes", raw_bytes.len());
        tracing::info!(
            "📄 Encoded transaction (hex, prefix 0x, truncated): 0x{}...",
            hex::encode(&raw_bytes).chars().take(96).collect::<String>()
        );

        Ok(raw_bytes)
    }

    /// Get an account
    pub async fn get_account(&self, address: Address) -> Result<SecureAccount> {
        self.accounts.get(&address).cloned().ok_or_else(|| {
            use crate::error::{VaughanError, WalletError};
            VaughanError::from(WalletError::AccountNotFound {
                address: address.to_string(),
            })
        })
    }

    /// List all accounts
    pub async fn list_accounts(&self) -> Result<Vec<SecureAccount>> {
        Ok(self.accounts.values().cloned().collect())
    }

    /// Remove an account
    pub async fn remove_account(&mut self, address: Address) -> Result<()> {
        let account = self
            .accounts
            .remove(&address)
            .ok_or(SecurityError::InvalidAddress(address.to_string()))?;

        // Remove from keychain
        self.keychain.delete(&account.key_reference)?;

        // Save accounts to persistent storage to persist the deletion
        self.save_accounts().await?;

        Ok(())
    }

    /// Lock the keystore
    pub async fn lock(&mut self) -> Result<()> {
        self.is_locked = true;
        // Clear sensitive data from memory
        self.accounts.clear();
        tracing::info!("🔒 Keystore locked");
        Ok(())
    }

    /// Unlock the keystore
    pub async fn unlock(&mut self) -> Result<()> {
        self.is_locked = false;
        // Reload accounts and networks from keychain metadata
        self.reload_accounts().await?;
        self.reload_networks().await?;
        tracing::info!("🔓 Keystore unlocked");
        Ok(())
    }

    /// Check if keystore is locked
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Ensure keystore is unlocked and accounts are loaded
    pub async fn ensure_unlocked(&mut self) -> Result<()> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked. Please unlock it first.".to_string(),
            }
            .into());
        }

        if self.accounts.is_empty() {
            tracing::warn!("⚠️ Keystore is unlocked but no accounts loaded, reloading...");
            self.reload_accounts().await?;
            tracing::info!("✅ Reloaded {} accounts", self.accounts.len());
        } else {
            tracing::info!("✅ Keystore is properly unlocked with {} accounts", self.accounts.len());
        }
        Ok(())
    }

    /// Add a custom network
    pub async fn add_custom_network(&mut self, config: NetworkConfig) -> Result<()> {
        // Check if network already exists
        if self.custom_networks.contains_key(&config.id) {
            return Err(SecurityError::KeystoreError {
                message: "Network already exists".to_string(),
            }
            .into());
        }

        // Add the network to custom networks
        self.custom_networks.insert(config.id, config);

        // Save networks to persistent storage
        self.save_networks().await?;

        Ok(())
    }

    /// Update a custom network
    pub async fn update_custom_network(&mut self, config: NetworkConfig) -> Result<()> {
        // Check if network exists
        if !self.custom_networks.contains_key(&config.id) {
            return Err(SecurityError::KeystoreError {
                message: "Network not found".to_string(),
            }
            .into());
        }

        // Update the network
        self.custom_networks.insert(config.id, config);

        // Save networks to persistent storage
        self.save_networks().await?;

        Ok(())
    }

    /// Remove a custom network
    pub async fn remove_custom_network(&mut self, network_id: NetworkId) -> Result<()> {
        let _removed = self
            .custom_networks
            .remove(&network_id)
            .ok_or(SecurityError::KeystoreError {
                message: "Network not found".to_string(),
            })?;

        // Save networks to persistent storage
        self.save_networks().await?;

        Ok(())
    }

    /// Get all custom networks
    pub fn get_custom_networks(&self) -> &HashMap<NetworkId, NetworkConfig> {
        &self.custom_networks
    }

    /// Get a specific custom network
    pub fn get_custom_network(&self, network_id: NetworkId) -> Option<&NetworkConfig> {
        self.custom_networks.get(&network_id)
    }

    /// Retrieve private key from keychain (for advanced operations like forge deployment)
    /// ⚠️ USE WITH CAUTION - exposes raw private key
    pub fn retrieve(&self, key_ref: &KeyReference) -> Result<SecretString> {
        if self.is_locked {
            return Err(SecurityError::KeystoreError {
                message: "Keystore is locked".to_string(),
            }
            .into());
        }

        self.keychain.retrieve(key_ref)
    }

    /// Decrypt data with password (public interface to encryption module)
    pub fn decrypt_with_password(&self, encrypted_data: &[u8], password: &[u8]) -> Result<Vec<u8>> {
        encryption::decrypt_with_password(encrypted_data, password)
    }

    /// Reload accounts from persistent storage
    async fn reload_accounts(&mut self) -> Result<()> {
        storage::load_accounts(&mut self.accounts, self.keychain.as_ref())
    }

    /// Save accounts to persistent storage
    async fn save_accounts(&self) -> Result<()> {
        storage::save_accounts(&self.accounts)
    }

    /// Reload networks from persistent storage
    async fn reload_networks(&mut self) -> Result<()> {
        storage::load_networks(&mut self.custom_networks)
    }

    /// Save networks to persistent storage
    async fn save_networks(&self) -> Result<()> {
        storage::save_networks(&self.custom_networks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::MockKeychain;
    use anyhow::Context;

    #[tokio::test]
    async fn test_create_account() -> Result<()> {
        let keychain = Box::new(MockKeychain::new());
        let mut keystore = SecureKeystoreImpl::new(keychain)
            .await
            .context("Failed to process keystore")?;

        let account = keystore.create_account("Test Account".to_string()).await.unwrap();

        assert_eq!(account.name, "Test Account");
        assert!(!account.address.is_zero());
        Ok(())
    }

    #[tokio::test]
    async fn test_import_account() -> Result<()> {
        let keychain = Box::new(MockKeychain::new());
        let mut keystore = SecureKeystoreImpl::new(keychain)
            .await
            .context("Failed to process keystore")?;

        let private_key =
            SecretString::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());
        let account = keystore
            .import_account(private_key, "Imported Account".to_string())
            .await
            .context("Operation failed")?;

        assert_eq!(account.name, "Imported Account");
        assert!(!account.address.is_zero());
        Ok(())
    }

    #[tokio::test]
    async fn test_keystore_locking() -> Result<()> {
        let keychain = Box::new(MockKeychain::new());
        let mut keystore = SecureKeystoreImpl::new(keychain)
            .await
            .context("Failed to process keystore")?;

        assert!(!keystore.is_locked());

        keystore.lock().await.context("Operation failed")?;
        assert!(keystore.is_locked());

        keystore.unlock().await.context("Operation failed")?;
        assert!(!keystore.is_locked());
        Ok(())
    }
}
