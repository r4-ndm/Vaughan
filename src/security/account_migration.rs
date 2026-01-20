//! Account migration utility for password workflow
//!
//! This utility helps migrate accounts created before the password workflow
//! implementation to the new encrypted format.

use crate::error::Result;
use crate::security::keychain::OSKeychain;
use crate::security::keystore::SecureKeystoreImpl;
use crate::security::seed::SeedManager;
use secrecy::SecretString;

/// Migrate an old account to use password encryption
///
/// This function:
/// 1. Retrieves the old account
/// 2. Prompts for the seed phrase (user must provide it)
/// 3. Creates a new encrypted version with a password
/// 4. Replaces the old account
pub async fn migrate_account_to_password_protected(
    account_id: String,
    seed_phrase: String,
    new_password: String,
) -> Result<String> {
    tracing::info!("🔄 Starting account migration for: {}", account_id);

    // Create keychain and keystore
    let keychain = Box::new(OSKeychain::new("vaughan-wallet".to_string())?);
    let mut keystore = SecureKeystoreImpl::new(keychain).await?;

    // Get the old account
    let accounts = keystore.list_accounts().await?;
    let old_account = accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| crate::error::VaughanError::Wallet(
            crate::error::WalletError::AccountNotFound { account_id: account_id.clone() }
        ))?;

    let account_name = old_account.name.clone();
    let account_address = old_account.address;

    tracing::info!("Found account: {} at address: {:?}", account_name, account_address);

    // Create seed manager for new encrypted account
    let seed_keychain = Box::new(OSKeychain::new("vaughan-wallet".to_string())?);
    let seed_manager = SeedManager::new(seed_keychain);

    // Convert to secure types
    let secure_seed = SecretString::new(seed_phrase);
    let secure_password = SecretString::new(new_password);

    // Validate the seed phrase
    seed_manager.validate_seed_phrase(&secure_seed)?;

    // Create new encrypted account
    tracing::info!("Creating new encrypted account...");
    let new_account = seed_manager
        .create_wallet_from_seed_encrypted(
            account_name.clone(),
            &secure_seed,
            &secure_password,
            None,
        )
        .await?;

    // Verify the address matches
    if new_account.address != account_address {
        return Err(crate::error::VaughanError::Wallet(
            crate::error::WalletError::InvalidAddress {
                address: format!("Address mismatch: expected {:?}, got {:?}", account_address, new_account.address)
            }
        ));
    }

    // Remove old account
    tracing::info!("Removing old account...");
    keystore.remove_account(account_address).await?;

    // Add new encrypted account
    tracing::info!("Adding new encrypted account...");
    keystore
        .import_account_with_key_reference(
            new_account.name.clone(),
            new_account.address,
            new_account.key_reference.clone(),
        )
        .await?;

    tracing::info!("✅ Successfully migrated account: {}", account_name);
    Ok(format!("Account '{}' has been migrated to password-protected format", account_name))
}

/// Check if an account needs migration
pub async fn check_if_migration_needed(account_id: String) -> Result<bool> {
    let keychain = Box::new(OSKeychain::new("vaughan-wallet".to_string())?);
    let keystore = SecureKeystoreImpl::new(keychain).await?;

    let accounts = keystore.list_accounts().await?;
    let account = accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| crate::error::VaughanError::Wallet(
            crate::error::WalletError::AccountNotFound { account_id: account_id.clone() }
        ))?;

    // Check if account uses the old format (not encrypted with password)
    // Old accounts won't have the "vaughan-wallet-encrypted-seeds" service
    Ok(account.key_reference.service != "vaughan-wallet-encrypted-seeds")
}

/// List all accounts that need migration
pub async fn list_accounts_needing_migration() -> Result<Vec<String>> {
    let keychain = Box::new(OSKeychain::new("vaughan-wallet".to_string())?);
    let keystore = SecureKeystoreImpl::new(keychain).await?;

    let accounts = keystore.list_accounts().await?;
    
    let mut needs_migration = Vec::new();
    for account in accounts {
        // Check if it's a seed-based account but not using encrypted format
        if account.key_reference.service != "vaughan-wallet-encrypted-seeds" 
            && account.key_reference.service != "vaughan-wallet" {
            // This is likely an old seed account
            needs_migration.push(account.name);
        }
    }

    Ok(needs_migration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_migration_needed() {
        // This test would need a mock account to work properly
        // For now, it's a placeholder
    }
}
