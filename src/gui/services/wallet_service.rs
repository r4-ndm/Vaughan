//! Wallet initialization and management service
//!
//! Contains functions for wallet initialization, account loading, and core wallet operations
//! extracted from working_wallet.rs

use crate::network::NetworkId;
use crate::security::SecureAccount;
use std::sync::Arc;

/// Initialize a new wallet instance with default configuration
pub async fn initialize_wallet() -> Result<Arc<tokio::sync::RwLock<crate::wallet::Vaughan>>, String> {
    use crate::wallet::{Vaughan, WalletConfig};

    let config = WalletConfig {
        default_network: NetworkId(943), // PulseChain Testnet v4 - consistent with imported accounts
        ..Default::default()
    };

    match Vaughan::new(config).await {
        Ok(wallet) => {
            tracing::info!("✅ Wallet initialized successfully");
            Ok(Arc::new(tokio::sync::RwLock::new(wallet)))
        }
        Err(e) => {
            tracing::error!("❌ Failed to initialize wallet: {}", e);
            Err(format!("Failed to initialize wallet: {e}"))
        }
    }
}

/// Load all available accounts from persistent storage
pub async fn load_available_accounts() -> Result<Vec<SecureAccount>, String> {
    use crate::security::{keychain::OSKeychain, KeyReference, KeychainInterface};

    tracing::info!("Loading available accounts from persistent storage...");

    // Load account metadata from persistent file using standardized path
    let mut accounts_path = crate::security::keystore::storage::get_vaughan_dir();
    accounts_path.push("accounts.json");

    let mut all_accounts = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&accounts_path) {
        if let Ok(stored_accounts) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            tracing::info!(
                "Found {} stored accounts in {}",
                stored_accounts.len(),
                accounts_path.display()
            );

            for stored in stored_accounts {
                // Extract account metadata
                let id = stored["id"].as_str().unwrap_or("").to_string();
                let name = stored["name"].as_str().unwrap_or("Unknown").to_string();
                let address_str = stored["address"].as_str().unwrap_or("");
                let service = stored["key_reference"]["service"].as_str().unwrap_or("");
                let created_at_str = stored["created_at"].as_str().unwrap_or("");
                let derivation_path = stored["derivation_path"]
                    .as_str()
                    .unwrap_or("m/44'/60'/0'/0/0")
                    .to_string();

                // Parse address
                if let Ok(address) = address_str.parse::<alloy::primitives::Address>() {
                    // Validate that the key exists in the appropriate keychain
                    let key_exists = match service {
                        "vaughan-wallet-encrypted-seeds" | crate::security::SERVICE_NAME_ENCRYPTED_SEEDS => {
                            // Check encrypted seed keychain
                            if let Ok(seed_keychain) = OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string()) {
                                let key_ref = KeyReference {
                                    id: stored["key_reference"]["id"].as_str().unwrap_or("").to_string(),
                                    service: service.to_string(),
                                    account: stored["key_reference"]["account"].as_str().unwrap_or("").to_string(),
                                };
                                seed_keychain.retrieve(&key_ref).is_ok()
                            } else {
                                false
                            }
                        }
                        "vaughan-wallet" | crate::security::SERVICE_NAME_PRIVATE_KEYS => {
                            // Check private key keychain
                            if let Ok(keychain) = OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()) {
                                let key_ref = KeyReference {
                                    id: stored["key_reference"]["id"].as_str().unwrap_or("").to_string(),
                                    service: service.to_string(),
                                    account: stored["key_reference"]["account"].as_str().unwrap_or("").to_string(),
                                };
                                keychain.retrieve(&key_ref).is_ok()
                            } else {
                                false
                            }
                        }
                        _ => {
                            tracing::warn!("Unknown service type: {}", service);
                            false
                        }
                    };

                    if key_exists {
                        // Parse created_at timestamp
                        let created_at = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at_str) {
                            dt.with_timezone(&chrono::Utc)
                        } else {
                            chrono::Utc::now()
                        };

                        // Create SecureAccount
                        let account = crate::security::SecureAccount {
                            id,
                            name: name.clone(),
                            address,
                            key_reference: KeyReference {
                                id: stored["key_reference"]["id"].as_str().unwrap_or("").to_string(),
                                service: service.to_string(),
                                account: stored["key_reference"]["account"].as_str().unwrap_or("").to_string(),
                            },
                            created_at,
                            is_hardware: false,
                            derivation_path: Some(derivation_path),
                            tags: Vec::new(),
                            last_used: None,
                            transaction_count: 0,
                        };

                        all_accounts.push(account);
                        tracing::info!("✅ Loaded account: {} ({})", name, address);
                    } else {
                        tracing::warn!("⚠️ Account key not found in keychain, skipping: {} ({})", name, address);
                    }
                } else {
                    tracing::warn!("⚠️ Invalid address format, skipping account: {}", name);
                }
            }
        }
    } else {
        tracing::info!("No stored accounts found ({})", accounts_path.display());
    }

    tracing::info!("Loaded {} valid accounts", all_accounts.len());
    Ok(all_accounts)
}
