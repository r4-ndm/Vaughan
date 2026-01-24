//! Keystore Storage Operations
//!
//! This module handles persistent storage of account and network metadata
//! to the filesystem with secure file permissions.

use crate::error::{Result, SecurityError};
use crate::network::{NetworkConfig, NetworkId};
use crate::security::{KeyReference, KeychainInterface, SecureAccount};
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Serializable account metadata for persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAccountMeta {
    pub id: String,
    pub name: String,
    pub address: alloy::primitives::Address,
    pub key_reference: KeyReference,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_hardware: bool,
    pub derivation_path: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub last_used: Option<i64>,
    #[serde(default)]
    pub transaction_count: u64,
}

/// Serializable network metadata for persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredNetworkMeta {
    pub id: NetworkId,
    pub name: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub currency_symbol: String,
    pub block_explorer: String,
    pub is_testnet: bool,
    pub is_custom: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Get the path to the .vaughan directory
pub fn get_vaughan_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    let mut path = home_dir;
    path.push(".vaughan");
    path
}

/// Ensure the .vaughan directory exists with secure permissions
pub fn ensure_vaughan_dir() -> Result<PathBuf> {
    let vaughan_path = get_vaughan_dir();

    if !vaughan_path.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = std::fs::DirBuilder::new();
            builder.mode(0o700);
            builder
                .create(&vaughan_path)
                .map_err(|e| SecurityError::KeystoreError {
                    message: format!("Failed to create .vaughan directory: {e}"),
                })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::create_dir_all(&vaughan_path).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to create .vaughan directory: {e}"),
            })?;
        }
    }

    Ok(vaughan_path)
}

/// Write content to a file with secure permissions (0o600 on Unix)
pub fn write_secure_file(path: &str, content: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to open file: {e}"),
            })?;
        f.write_all(content.as_bytes())
            .map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to write file: {e}"),
            })?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, content).map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to save file: {e}"),
        })?;
    }
    Ok(())
}

/// Load accounts from persistent storage
pub fn load_accounts(
    accounts: &mut HashMap<alloy::primitives::Address, SecureAccount>,
    keychain: &dyn crate::security::KeychainInterface,
) -> Result<()> {
    let mut accounts_path = get_vaughan_dir();
    accounts_path.push("accounts.json");

    let accounts_file = accounts_path.to_string_lossy().to_string();

    if let Ok(content) = std::fs::read_to_string(&accounts_file) {
        if let Ok(stored_accounts) = serde_json::from_str::<Vec<StoredAccountMeta>>(&content) {
            for stored in stored_accounts {
                // Validate that the key reference exists in the keychain
                tracing::info!(
                    "🔍 Checking account '{}' with service '{}', key_id '{}'",
                    stored.name,
                    stored.key_reference.service,
                    stored.key_reference.id
                );
                let key_exists = match stored.key_reference.service.as_str() {
                    crate::security::SERVICE_NAME_ENCRYPTED_SEEDS | "vaughan-wallet-encrypted-seeds" => {
                        // For seed-based accounts, check if the encrypted seed exists
                        let seed_keychain =
                            crate::security::keychain::OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string());
                        match seed_keychain {
                            Ok(kc) => {
                                let result = kc.retrieve(&stored.key_reference).is_ok();
                                tracing::info!("  -> Seed keychain retrieve result for '{}': {}", stored.name, result);
                                result
                            }
                            Err(e) => {
                                tracing::warn!("  -> Failed to create seed keychain: {}", e);
                                false
                            }
                        }
                    }
                    crate::security::SERVICE_NAME_PRIVATE_KEYS | "vaughan-wallet" => {
                        // For private key accounts, check if the key exists
                        let result = keychain.retrieve(&stored.key_reference).is_ok();
                        tracing::info!(
                            "  -> Private key keychain retrieve result for '{}': {}",
                            stored.name,
                            result
                        );
                        result
                    }
                    _ => {
                        tracing::warn!(
                            "Unknown service type for account {}: {}",
                            stored.name,
                            stored.key_reference.service
                        );
                        false
                    }
                };

                if key_exists {
                    let account_name = stored.name.clone();
                    let account_address = stored.address;
                    let account = SecureAccount {
                        id: stored.id,
                        name: stored.name,
                        address: stored.address,
                        key_reference: stored.key_reference,
                        created_at: stored.created_at,
                        is_hardware: stored.is_hardware,
                        derivation_path: stored.derivation_path,
                        tags: stored.tags,
                        last_used: stored.last_used,
                        transaction_count: stored.transaction_count,
                    };
                    accounts.insert(stored.address, account);
                    tracing::info!(
                        "✅ Keystore loaded account: {} ({}) - address key: {:#x}",
                        account_name,
                        account_address,
                        stored.address
                    );
                } else {
                    tracing::warn!(
                        "⚠️ Keystore skipping account {} - key not found in keychain (service: {}, key_id: {})",
                        stored.name,
                        stored.key_reference.service,
                        stored.key_reference.id
                    );
                }
            }
            tracing::info!("Loaded {} valid accounts from persistent storage", accounts.len());
        } else {
            tracing::warn!("Failed to parse accounts.json file");
        }
    } else {
        tracing::info!("No existing accounts.json file found - starting with empty account list");
    }
    Ok(())
}

/// Save accounts to persistent storage
pub fn save_accounts(accounts: &HashMap<alloy::primitives::Address, SecureAccount>) -> Result<()> {
    let mut vaughan_path = ensure_vaughan_dir()?;
    vaughan_path.push("accounts.json");

    let accounts_file = vaughan_path.to_string_lossy().to_string();
    let stored_accounts: Vec<StoredAccountMeta> = accounts
        .values()
        .map(|account| StoredAccountMeta {
            id: account.id.clone(),
            name: account.name.clone(),
            address: account.address,
            key_reference: account.key_reference.clone(),
            created_at: account.created_at,
            is_hardware: account.is_hardware,
            derivation_path: account.derivation_path.clone(),
            tags: account.tags.clone(),
            last_used: account.last_used,
            transaction_count: account.transaction_count,
        })
        .collect();

    let json_content = serde_json::to_string_pretty(&stored_accounts).map_err(|e| SecurityError::KeystoreError {
        message: format!("Failed to serialize accounts: {e}"),
    })?;

    write_secure_file(&accounts_file, &json_content)?;
    tracing::info!("Saved {} accounts to persistent storage", stored_accounts.len());
    Ok(())
}

/// Load networks from persistent storage
pub fn load_networks(networks: &mut HashMap<NetworkId, NetworkConfig>) -> Result<()> {
    let mut networks_path = get_vaughan_dir();
    networks_path.push("networks.json");

    let networks_file = networks_path.to_string_lossy().to_string();

    if let Ok(content) = std::fs::read_to_string(&networks_file) {
        if let Ok(stored_networks) = serde_json::from_str::<Vec<StoredNetworkMeta>>(&content) {
            for stored in stored_networks {
                let network = NetworkConfig {
                    id: stored.id,
                    name: stored.name,
                    rpc_url: stored.rpc_url,
                    chain_id: stored.chain_id,
                    symbol: stored.currency_symbol,
                    block_explorer_url: stored.block_explorer,
                    is_testnet: stored.is_testnet,
                    is_custom: true, // All stored networks are custom
                };
                networks.insert(stored.id, network);
            }
            tracing::info!("Loaded {} custom networks from persistent storage", networks.len());
        }
    }
    Ok(())
}

/// Save networks to persistent storage
pub fn save_networks(networks: &HashMap<NetworkId, NetworkConfig>) -> Result<()> {
    let mut vaughan_path = ensure_vaughan_dir()?;
    vaughan_path.push("networks.json");

    let networks_file = vaughan_path.to_string_lossy().to_string();
    let stored_networks: Vec<StoredNetworkMeta> = networks
        .values()
        .map(|network| StoredNetworkMeta {
            id: network.id,
            name: network.name.clone(),
            rpc_url: network.rpc_url.clone(),
            chain_id: network.chain_id,
            currency_symbol: network.symbol.clone(),
            block_explorer: network.block_explorer_url.clone(),
            is_testnet: network.is_testnet,
            is_custom: true,
            created_at: chrono::Utc::now(),
        })
        .collect();

    let json_content = serde_json::to_string_pretty(&stored_networks).map_err(|e| SecurityError::KeystoreError {
        message: format!("Failed to serialize networks: {e}"),
    })?;

    write_secure_file(&networks_file, &json_content)?;
    tracing::info!("Saved {} custom networks to persistent storage", stored_networks.len());
    Ok(())
}
