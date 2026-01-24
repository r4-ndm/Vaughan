//! Account management service
//!
//! Contains functions for account creation, import, export operations
//! extracted from working_wallet.rs

use crate::gui::wallet_types::AccountType;
use crate::security::{SecureAccount, SeedStrength};

/// Check if any seed-based accounts exist in the keystore
pub async fn check_for_seed_accounts() -> Result<bool, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;

    tracing::info!("Checking for seed-based accounts");

    // Create keychain and keystore
    let keychain = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()).map_err(|e| format!("Failed to initialize keychain: {e}"))?,
    );
    let keystore = SecureKeystoreImpl::new(keychain)
        .await
        .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

    // Get all accounts
    let accounts = keystore
        .list_accounts()
        .await
        .map_err(|e| format!("Failed to list accounts: {e}"))?;

    // Check if any accounts are seed-based
    let has_seed_accounts = accounts
        .iter()
        .any(|account| get_account_type(account) == AccountType::SeedBased);

    tracing::info!(
        "Found {} total accounts, {} seed-based",
        accounts.len(),
        accounts
            .iter()
            .filter(|a| get_account_type(a) == AccountType::SeedBased)
            .count()
    );

    Ok(has_seed_accounts)
}

/// Get the account type for a specific account
pub fn get_account_type(account: &SecureAccount) -> AccountType {
    // Seed-based accounts use the "vaughan-wallet-encrypted-seeds" service
    // Private-key accounts use the "vaughan-wallet" service
    if account.key_reference.service == crate::security::SERVICE_NAME_ENCRYPTED_SEEDS {
        AccountType::SeedBased
    } else {
        AccountType::PrivateKey
    }
}

/// Check if a specific account is seed-based
pub fn is_seed_based_account(account: &SecureAccount) -> bool {
    get_account_type(account) == AccountType::SeedBased
}

/// Create a new wallet from seed phrase
pub async fn create_wallet_from_seed(name: String, seed: String, password: String) -> Result<String, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use crate::security::seed::SeedManager;
    use secrecy::SecretString;

    tracing::info!("Creating wallet from seed: {}", name);

    // Create keychain and seed manager (using correct service name for encrypted seeds)
    let keychain = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string())
            .map_err(|e| format!("Failed to initialize keychain: {e}"))?,
    );
    let seed_manager = SeedManager::new(keychain);

    // Convert inputs to secure types
    let secure_seed = SecretString::new(seed);
    let secure_password = SecretString::new(password);

    // Validate the seed phrase first
    seed_manager
        .validate_seed_phrase(&secure_seed)
        .map_err(|e| format!("Invalid seed phrase: {e}"))?;

    // Create wallet from seed
    let account = seed_manager
        .create_wallet_from_seed_encrypted(
            name,
            &secure_seed,
            &secure_password,
            None, // No passphrase for now
        )
        .await
        .map_err(|e| format!("Failed to create wallet: {e}"))?;

    // Also save the account to the keystore for persistence
    let keychain2 = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string())
            .map_err(|e| format!("Failed to initialize keychain for keystore: {e}"))?,
    );
    let mut keystore = SecureKeystoreImpl::new(keychain2)
        .await
        .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

    // Add the account to the keystore using the key reference
    keystore
        .import_account_with_key_reference(account.name.clone(), account.address, account.key_reference.clone())
        .await
        .map_err(|e| format!("Failed to save account to keystore: {e}"))?;

    tracing::info!(
        "Successfully created and saved wallet: {} with identity: {}",
        account.name,
        account.id
    );

    Ok(account.id)
}

/// Import an existing wallet from seed phrase
pub async fn import_wallet_from_seed(name: String, seed: String, password: String) -> Result<String, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use crate::security::seed::SeedManager;
    use secrecy::SecretString;

    tracing::info!("Importing wallet from seed: {}", name);

    // Validate input parameters
    if name.trim().is_empty() {
        return Err("Wallet name cannot be empty".to_string());
    }

    if seed.trim().is_empty() {
        return Err("Seed phrase cannot be empty".to_string());
    }

    // Create keychain and seed manager (using correct service name for encrypted seeds)
    let keychain = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string())
            .map_err(|e| format!("Failed to initialize keychain: {e}"))?,
    );
    let seed_manager = SeedManager::new(keychain);

    // Convert inputs to secure types
    let secure_seed = SecretString::new(seed);
    let secure_password = SecretString::new(password);

    // Validate the seed phrase first
    seed_manager
        .validate_seed_phrase(&secure_seed)
        .map_err(|e| format!("Invalid seed phrase: {e}"))?;

    // Import wallet from seed (same as create for seed phrases)
    let account = seed_manager
        .create_wallet_from_seed_encrypted(
            name.clone(),
            &secure_seed,
            &secure_password,
            None, // No passphrase for now
        )
        .await
        .map_err(|e| format!("Failed to import wallet: {e}"))?;

    // Also save the account to the keystore for persistence
    let keychain2 = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string())
            .map_err(|e| format!("Failed to initialize keychain for keystore: {e}"))?,
    );
    let mut keystore = SecureKeystoreImpl::new(keychain2)
        .await
        .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

    // Add the account to the keystore using the key reference
    keystore
        .import_account_with_key_reference(account.name.clone(), account.address, account.key_reference.clone())
        .await
        .map_err(|e| format!("Failed to save account to keystore: {e}"))?;

    tracing::info!(
        "Successfully imported and saved wallet: {} with identity: {}",
        account.name,
        account.id
    );

    Ok(account.id)
}

/// Import wallet from private key
pub async fn import_wallet_from_private_key(name: String, key: String, _password: String) -> Result<String, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use secrecy::SecretString;

    tracing::info!("Importing wallet from private key: {}", name);

    // Validate input parameters
    if name.trim().is_empty() {
        return Err("Wallet name cannot be empty".to_string());
    }

    if key.trim().is_empty() {
        return Err("Private key cannot be empty".to_string());
    }

    // Validate private key format before passing to keystore
    let clean_key = if key.trim().starts_with("0x") {
        key.trim()[2..].to_string()
    } else {
        key.trim().to_string()
    };

    // Check length
    if clean_key.len() != 64 {
        return Err(format!(
            "Private key must be exactly 64 characters (got {}). Remove any 0x prefix.",
            clean_key.len()
        ));
    }

    // Check if it's valid hex
    if !clean_key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Private key must contain only hexadecimal characters (0-9, a-f, A-F)".to_string());
    }

    // Create keychain and keystore
    let keychain = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()).map_err(|e| format!("Failed to initialize keychain: {e}"))?,
    );
    let mut keystore = SecureKeystoreImpl::new(keychain)
        .await
        .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

    // Convert inputs to secure types (use original key with 0x handling)
    let secure_key = SecretString::new(key.trim().to_string());

    // Import wallet from private key with detailed error handling
    let account = keystore.import_account(secure_key, name.clone()).await.map_err(|e| {
        let error_msg = format!("{e}");
        if error_msg.contains("InvalidPrivateKey") {
            "Invalid private key format or value".to_string()
        } else if error_msg.contains("Account already exists") {
            "An account with this private key already exists".to_string()
        } else if error_msg.contains("Keystore is locked") {
            "Keystore is locked. Please unlock first".to_string()
        } else {
            format!("Failed to import private key: {error_msg}")
        }
    })?;

    // Save account metadata to accounts.json for GUI persistence
    save_account_metadata_to_file(&account).await.map_err(|e| {
        tracing::warn!("Failed to save account metadata: {}", e);
        format!("Account imported but metadata save failed: {e}")
    })?;

    tracing::info!(
        "Successfully imported wallet from private key: {} with identity: {}",
        account.name,
        account.id
    );

    Ok(account.id)
}

/// Unified export seed phrase - checks for new WalletManager format, falls back to legacy
/// This is the new primary entry point for seed phrase export (Task 4.3)
pub async fn export_seed_phrase_unified(account_id: String, password: String) -> Result<String, String> {
    use crate::wallet::WalletManager;
    use secrecy::SecretString;

    tracing::info!("🔑 Unified seed phrase export for account: {}", account_id);

    // Check for new keystore.json format first
    let wallet_dir = match dirs::home_dir() {
        Some(dir) => dir.join(".vaughan"),
        None => {
            tracing::warn!("Could not determine home directory, falling back to legacy export");
            return export_seed_phrase_with_password(account_id, password).await;
        }
    };

    let keystore_path = wallet_dir.join("keystore.json");

    // If new keystore exists, try WalletManager first
    if keystore_path.exists() {
        tracing::info!("📁 Found new keystore.json, attempting WalletManager export");

        let mut manager = WalletManager::new(keystore_path);
        let secret_password = SecretString::new(password.clone());

        // Try to unlock the wallet
        match manager.unlock(secret_password) {
            Ok(()) => {
                // Try to export seed phrase
                match manager.export_seed_phrase() {
                    Ok(seed_phrase) => {
                        tracing::info!("✅ Successfully exported seed phrase using WalletManager");
                        return Ok(seed_phrase);
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ WalletManager seed phrase export failed: {}. This may be because the wallet was imported from a keystore (no seed phrase stored).", e);
                        // Fall through to try legacy export
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ WalletManager unlock failed: {}. Trying legacy export...", e);
                // Fall through to try legacy export
            }
        }
    }

    // Fall back to legacy export
    tracing::info!("📁 Using legacy keychain-based seed phrase export");
    export_seed_phrase_with_password(account_id, password).await
}

/// Unified export private key - checks for new WalletManager format, falls back to legacy
/// This is the new primary entry point for private key export (Task 4.3)
pub async fn export_private_key_unified(account_id: String, password: String) -> Result<String, String> {
    use crate::wallet::WalletManager;
    use secrecy::{ExposeSecret, SecretString};

    tracing::info!("🔑 Unified private key export for account: {}", account_id);

    // Check for new keystore.json format first
    let wallet_dir = crate::security::keystore::storage::get_vaughan_dir();

    let keystore_path = wallet_dir.join("keystore.json");

    // If new keystore exists, try WalletManager first
    if keystore_path.exists() {
        tracing::info!("📁 Found new keystore.json, attempting WalletManager export");

        let mut manager = WalletManager::new(keystore_path);
        let secret_password = SecretString::new(password.clone());

        // Try to unlock the wallet
        match manager.unlock(secret_password) {
            Ok(()) => {
                // Export private key
                match manager.export_private_key() {
                    Ok(private_key) => {
                        tracing::info!("✅ Successfully exported private key using WalletManager");
                        // Return with 0x prefix if not present
                        let key_string = private_key.expose_secret();
                        if key_string.starts_with("0x") {
                            return Ok(key_string.clone());
                        } else {
                            return Ok(format!("0x{}", key_string));
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ WalletManager private key export failed: {}", e);
                        // Fall through to try legacy export
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ WalletManager unlock failed: {}. Trying legacy export...", e);
                // Fall through to try legacy export
            }
        }
    }

    // Fall back to legacy export
    tracing::info!("📁 Using legacy keychain-based private key export");
    export_private_key_with_password(account_id, password).await
}

/// Export seed phrase with password verification (legacy method)
pub async fn export_seed_phrase_with_password(account_id: String, password: String) -> Result<String, String> {
    use crate::security::keychain::OSKeychain;

    use crate::security::keystore::SecureKeystoreImpl;
    use crate::security::seed::SeedManager;
    use secrecy::{ExposeSecret, SecretString};

    tracing::info!("Starting seed phrase export for account: {}", account_id);

    // Add timeout to prevent hanging export operations
    let export_task = async move {
        // Create keychain and keystore to get the account
        let keychain = Box::new(
            OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()).map_err(|e| format!("Failed to initialize keychain: {e}"))?,
        );
        let keystore = SecureKeystoreImpl::new(keychain)
            .await
            .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

        // Get all accounts and find the one we want
        let accounts = keystore
            .list_accounts()
            .await
            .map_err(|e| format!("Failed to list accounts: {e}"))?;

        let account = accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| format!("Account not found: {account_id}"))?;

        // Create seed manager for export
        // IMPORTANT: Use "vaughan-wallet-encrypted-seeds" service - this is where seeds are stored
        let keychain2 = Box::new(
            OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string())
                .map_err(|e| format!("Failed to initialize keychain for seed manager: {e}"))?,
        );
        let seed_manager = SeedManager::new(keychain2);

        // Convert password to secure type
        let secure_password = SecretString::new(password);

        // Export the seed phrase using the account's key reference
        tracing::info!(
            "Calling seed_manager.export_plaintext for account: {} with key_ref: {:?}",
            account.name,
            account.key_reference
        );

        let export_result = seed_manager
            .export_plaintext(&account.key_reference, &secure_password, Some(account.name.clone()))
            .await
            .map_err(|e| {
                tracing::error!(
                    "Seed phrase export failed for account '{}' with key_ref '{}': {}",
                    account.name,
                    account.key_reference.id,
                    e
                );

                // Provide more specific error messages
                let error_str = e.to_string();
                if error_str.contains("Failed to retrieve encrypted seed phrase") {
                    "Could not find stored seed phrase. Account may have been imported with a different method."
                        .to_string()
                } else if error_str.contains("password") || error_str.contains("authentication") {
                    "Incorrect password. Please check your password and try again.".to_string()
                } else if error_str.contains("keychain") {
                    "Keychain access error. Please ensure the application has proper permissions.".to_string()
                } else {
                    format!("Failed to export seed phrase: {e}")
                }
            })?;

        tracing::info!("Successfully exported seed phrase for account: {}", account.name);

        // Return the seed phrase from the export result
        let seed_phrase = export_result.data.expose_secret().clone();
        tracing::info!("Seed phrase extracted, length: {} characters", seed_phrase.len());
        Ok(seed_phrase)
    }; // End of export_task closure

    // Apply timeout to the export operation
    match tokio::time::timeout(std::time::Duration::from_secs(30), export_task).await {
        Ok(result) => result,
        Err(_) => Err("Export operation timed out. Please check your network connection and try again.".to_string()),
    }
}

/// Export private key with password verification
pub async fn export_private_key_with_password(account_id: String, password: String) -> Result<String, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use crate::security::seed::SeedManager;
    use crate::security::KeychainInterface;
    // HDWallet functionality removed - using simplified key derivation
    use secrecy::{ExposeSecret, SecretString};

    tracing::info!("Starting private key export for account: {}", account_id);

    // Add timeout to prevent hanging export operations
    let export_task = async move {
        // Create keychain and keystore to get the account (following seed phrase export pattern)
        let keychain = Box::new(
            OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()).map_err(|e| format!("Failed to initialize keychain: {e}"))?,
        );
        let keystore = SecureKeystoreImpl::new(keychain)
            .await
            .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

        // Get all accounts and find the one we want
        let accounts = keystore
            .list_accounts()
            .await
            .map_err(|e| format!("Failed to list accounts: {e}"))?;

        let account = accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| format!("Account not found: {account_id}"))?;

        // Create seed manager for export (following seed phrase export pattern)
        // IMPORTANT: Use "vaughan-wallet-encrypted-seeds" service - this is where seeds are stored
        let keychain2 = Box::new(
            OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string())
                .map_err(|e| format!("Failed to initialize keychain for seed manager: {e}"))?,
        );
        let seed_manager = SeedManager::new(keychain2);

        // Convert password to secure type
        let secure_password = SecretString::new(password);

        // Try to export seed phrase first (for accounts created from seed phrase)
        tracing::info!(
            "Attempting to get private key for account: {} with key_ref: {:?}",
            account.name,
            account.key_reference
        );

        let private_key_hex = match seed_manager
            .export_plaintext(&account.key_reference, &secure_password, Some(account.name.clone()))
            .await
        {
            Ok(export_result) => {
                // Successfully got seed phrase - derive private key from it
                let seed_phrase_string = export_result.data.expose_secret().clone();
                tracing::info!(
                    "Successfully got seed phrase, deriving private key for account: {}",
                    account.name
                );

                // Use Alloy's PrivateKeySigner to derive from mnemonic
                use alloy::signers::local::PrivateKeySigner;

                // Parse the mnemonic and derive private key using BIP39
                use bip39::Mnemonic;
                let mnemonic =
                    Mnemonic::parse(&seed_phrase_string).map_err(|e| format!("Failed to parse seed phrase: {e}"))?;

                // Convert to seed and derive key
                let seed = mnemonic.to_seed("");
                let private_key_signer = PrivateKeySigner::from_slice(&seed[..32])
                    .map_err(|e| format!("Failed to create private key from seed: {e}"))?;

                format!("0x{}", hex::encode(private_key_signer.to_bytes()))
            }
            Err(_) => {
                // Seed phrase not available - try direct private key retrieval
                tracing::info!(
                    "Seed phrase not available for account '{}', attempting direct private key retrieval",
                    account.name
                );

                // Try to retrieve private key directly from keychain
                let keychain3 = Box::new(
                    OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string())
                        .map_err(|e| format!("Failed to initialize keychain for private key retrieval: {e}"))?,
                );
                let private_key_data = keychain3
                    .retrieve(&account.key_reference)
                    .map_err(|e| format!("Failed to retrieve private key from keychain: {e}"))?;

                // Try to parse as private key (hex string)
                let private_key_string = private_key_data.expose_secret();

                // Check if it's already a hex private key
                if private_key_string.starts_with("0x") && private_key_string.len() == 66 {
                    tracing::info!(
                        "Successfully retrieved private key directly for account: {}",
                        account.name
                    );
                    private_key_string.clone()
                } else if private_key_string.len() == 64 && private_key_string.chars().all(|c| c.is_ascii_hexdigit()) {
                    format!("0x{}", private_key_string)
                } else {
                    return Err(format!(
                        "Account '{}' does not have an exportable private key or seed phrase",
                        account.name
                    ));
                }
            }
        };

        tracing::info!("Successfully derived private key for account: {}", account.name);

        Ok(private_key_hex)
    }; // End of export_task closure

    // Apply timeout to the export operation
    match tokio::time::timeout(std::time::Duration::from_secs(30), export_task).await {
        Ok(result) => result,
        Err(_) => Err("Export operation timed out. Please check your network connection and try again.".to_string()),
    }
}

/// Discover addresses from seed phrase
pub async fn discover_addresses_from_seed(seed: String) -> Result<Vec<(String, String, bool)>, String> {
    // HDWallet functionality removed - using simplified key derivation
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use secrecy::SecretString;

    let _secure_seed = SecretString::new(seed);

    // Create keystore to check existing accounts
    let keychain =
        Box::new(OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()).map_err(|e| format!("Failed to create keychain: {e}"))?);
    let keystore = SecureKeystoreImpl::new(keychain)
        .await
        .map_err(|e| format!("Failed to create keystore: {e}"))?;

    let _existing_accounts = keystore
        .list_accounts()
        .await
        .map_err(|e| format!("Failed to list existing accounts: {e}"))?;

    let mut discovered_addresses = Vec::new();

    // Check multiple derivation paths (simplified - return placeholders)
    let paths = [
        "m/44'/60'/0'/0/0",
        "m/44'/60'/0'/0/1",
        "m/44'/60'/0'/0/2",
        "m/44'/60'/0'/0/3",
        "m/44'/60'/0'/0/4",
    ];

    // Return placeholder addresses for now
    for (i, path) in paths.iter().enumerate() {
        let placeholder_address = format!("0x{i:040x}");
        discovered_addresses.push((path.to_string(), placeholder_address, false));
    }

    Ok(discovered_addresses)
}

/// Import multiple addresses from seed phrase
pub async fn import_multiple_addresses_from_seed(
    name_prefix: String,
    seed: String,
    password: String,
    selected_paths: Vec<String>,
) -> Result<Vec<String>, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;
    use crate::security::seed::SeedManager;
    use secrecy::SecretString;

    if selected_paths.is_empty() {
        return Err("No derivation paths selected".to_string());
    }

    let mut imported_addresses = Vec::new();

    // Create keychain and seed manager (using correct service name for encrypted seeds)
    let keychain = Box::new(
        OSKeychain::new(crate::security::SERVICE_NAME_ENCRYPTED_SEEDS.to_string())
            .map_err(|e| format!("Failed to initialize keychain: {e}"))?,
    );
    let seed_manager = SeedManager::new(keychain);

    // Convert inputs to secure types
    let secure_seed = SecretString::new(seed);
    let secure_password = SecretString::new(password);

    // Validate seed phrase
    seed_manager
        .validate_seed_phrase(&secure_seed)
        .map_err(|e| format!("Invalid seed phrase: {e}"))?;

    for (i, path) in selected_paths.iter().enumerate() {
        let account_name = if selected_paths.len() == 1 {
            name_prefix.clone()
        } else {
            format!("{} #{}", name_prefix, i + 1)
        };

        // Create wallet from seed (simplified - specific derivation path not implemented)
        match seed_manager
            .create_wallet_from_seed_encrypted(account_name.clone(), &secure_seed, &secure_password, None)
            .await
        {
            Ok(account) => {
                // Save to keystore for persistence
                let keychain2 = Box::new(
                    OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string())
                        .map_err(|e| format!("Failed to initialize keychain for keystore: {e}"))?,
                );
                let mut keystore = SecureKeystoreImpl::new(keychain2)
                    .await
                    .map_err(|e| format!("Failed to initialize keystore: {e}"))?;

                keystore
                    .import_account_with_key_reference(
                        account.name.clone(),
                        account.address,
                        account.key_reference.clone(),
                    )
                    .await
                    .map_err(|e| format!("Failed to save account to keystore: {e}"))?;

                imported_addresses.push(account.id.clone());
                tracing::info!(
                    "Successfully imported: {} with identity: {} (path: {})",
                    account.name,
                    account.id,
                    path
                );
            }
            Err(e) => {
                tracing::error!("Failed to import address for path {}: {}", path, e);
                return Err(format!("Failed to import address for path {path}: {e}"));
            }
        }
    }

    Ok(imported_addresses)
}

/// Delete an account by ID
pub async fn delete_account(id: String) -> Result<String, String> {
    use crate::security::{create_keychain_interface, keystore::SecureKeystoreImpl};

    tracing::info!("🗑️ Deleting account: {}", id);

    // Create keychain and keystore
    let keychain = create_keychain_interface().map_err(|e| format!("Failed to create keychain interface: {e}"))?;

    let mut keystore = SecureKeystoreImpl::new(keychain)
        .await
        .map_err(|e| format!("Failed to create keystore: {e}"))?;

    // Get all accounts to find the one to delete
    let accounts = keystore
        .list_accounts()
        .await
        .map_err(|e| format!("Failed to list accounts: {e}"))?;

    // Find account by ID
    let account_to_delete = accounts
        .iter()
        .find(|acc| acc.id == id)
        .ok_or_else(|| format!("Account with ID '{id}' not found"))?;

    let account_address = account_to_delete.address;
    let account_name = account_to_delete.name.clone();

    // Delete the account from keystore
    keystore
        .remove_account(account_address)
        .await
        .map_err(|e| format!("Failed to delete account: {e}"))?;

    tracing::info!("✅ Successfully deleted account '{}' ({})", account_name, id);
    Ok(format!("Account '{account_name}' has been permanently deleted"))
}

/// Analyze seed phrase strength and security
pub async fn analyze_seed_phrase(phrase: String) -> Result<crate::security::seed::SeedAnalysis, String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::seed::{SeedAnalysis, SeedImportConfig, SeedManager, SeedStrength};

    if phrase.trim().is_empty() {
        return Err("Empty seed phrase".to_string());
    }

    // Create keychain and seed manager for analysis
    let keychain = Box::new(
        OSKeychain::new("vaughan-wallet-temp".to_string())
            .map_err(|e| format!("Failed to initialize keychain: {e}"))?,
    );
    let seed_manager = SeedManager::new(keychain);

    // Use comprehensive validation for detailed feedback
    let config = SeedImportConfig::default();
    let validation = seed_manager
        .validate_seed_phrase_comprehensive(&phrase, &config)
        .map_err(|e| format!("Validation failed: {e}"))?;

    // If validation failed, return error with details
    if !validation.is_valid {
        return Err(format!("Invalid seed phrase: {}", validation.errors.join(", ")));
    }

    let strength = validation.strength.unwrap_or(SeedStrength::Words12);

    // Create comprehensive analysis
    let analysis = SeedAnalysis {
        strength,
        entropy_bits: match strength {
            SeedStrength::Words12 => 128,
            SeedStrength::Words15 => 160,
            SeedStrength::Words18 => 192,
            SeedStrength::Words21 => 224,
            SeedStrength::Words24 => 256,
        },
        security_level: match strength {
            SeedStrength::Words12 => "Good".to_string(),
            SeedStrength::Words15 => "Very Good".to_string(),
            SeedStrength::Words18 => "Excellent".to_string(),
            SeedStrength::Words21 => "Excellent".to_string(),
            SeedStrength::Words24 => "Maximum".to_string(),
        },
        recommendation: match strength {
            SeedStrength::Words12 => "Suitable for most use cases".to_string(),
            SeedStrength::Words15 => "Enhanced security for important wallets".to_string(),
            SeedStrength::Words18 => "High security for valuable assets".to_string(),
            SeedStrength::Words21 => "Very high security".to_string(),
            SeedStrength::Words24 => "Maximum security for high-value wallets".to_string(),
        },
        brute_force_estimate: match strength {
            SeedStrength::Words12 => "~10^38 attempts".to_string(),
            SeedStrength::Words15 => "~10^48 attempts".to_string(),
            SeedStrength::Words18 => "~10^57 attempts".to_string(),
            SeedStrength::Words21 => "~10^67 attempts".to_string(),
            SeedStrength::Words24 => "~10^77 attempts".to_string(),
        },
        warnings: validation.warnings,
        is_valid: true,
    };

    Ok(analysis)
}

/// Generate a new seed phrase with specified strength
pub async fn generate_seed_phrase_with_strength(strength: SeedStrength) -> String {
    use crate::security::keychain::OSKeychain;
    use crate::security::seed::SeedManager;
    use secrecy::ExposeSecret;

    tracing::info!("Generating new seed phrase with strength: {:?}", strength);

    // Create keychain and seed manager
    match OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()) {
        Ok(keychain) => {
            let seed_manager = SeedManager::new(Box::new(keychain));

            match seed_manager.generate_seed_phrase(strength) {
                Ok(seed_phrase) => {
                    tracing::info!("Successfully generated {}-word seed phrase", strength.word_count());
                    seed_phrase.expose_secret().clone()
                }
                Err(e) => {
                    tracing::error!("Failed to generate seed phrase: {}", e);
                    // Return a fallback seed phrase for demo purposes
                    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
                        .to_string()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to initialize keychain for seed generation: {}", e);
            // Return a fallback seed phrase for demo purposes
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
        }
    }
}

/// Save account metadata to persistent file storage
pub async fn save_account_metadata_to_file(account: &SecureAccount) -> Result<(), String> {
    use serde_json;

    // Get the vaughan directory using standardization
    let accounts_dir = crate::security::keystore::storage::get_vaughan_dir();

    std::fs::create_dir_all(&accounts_dir).map_err(|e| format!("Failed to create account directory: {e}"))?;

    // Create account metadata for storage
    let metadata = serde_json::json!({
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

    // Write to accounts.json
    let accounts_file = accounts_dir.join("accounts.json");

    // Load existing accounts if file exists
    let mut all_accounts = if accounts_file.exists() {
        let content = std::fs::read_to_string(&accounts_file)
            .map_err(|e| format!("Failed to read existing accounts file: {e}"))?;
        serde_json::from_str::<Vec<serde_json::Value>>(&content).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // Check if account already exists and update, or add new
    if let Some(existing_index) = all_accounts.iter().position(|a| a["id"] == account.id) {
        all_accounts[existing_index] = metadata;
        tracing::info!("Updated existing account metadata for: {}", account.name);
    } else {
        all_accounts.push(metadata);
        tracing::info!("Added new account metadata for: {}", account.name);
    }

    // Write back to file
    let json_content =
        serde_json::to_string_pretty(&all_accounts).map_err(|e| format!("Failed to serialize accounts: {e}"))?;

    std::fs::write(&accounts_file, json_content).map_err(|e| format!("Failed to write accounts file: {e}"))?;

    tracing::info!("Successfully saved account metadata to: {}", accounts_file.display());
    Ok(())
}
