# Wallet Import and Persistence Fix

## Problem Summary
The wallet import and persistence system had several issues:

1. **Service Name Mismatch**: Accounts imported from seed phrases used `vaughan-wallet-encrypted-seeds` service, but the keystore was looking for `vaughan-wallet` service
2. **Missing Account Persistence**: Imported accounts from seed phrases weren't being saved to the accounts.json file
3. **Account Loading Issues**: The keystore couldn't properly load accounts with different service names

## Root Cause Analysis

### Service Name Inconsistency
- **SeedManager** uses `vaughan-wallet-encrypted-seeds` for storing encrypted seed phrases
- **SecureKeystoreImpl** uses `vaughan-wallet` for private key accounts
- This mismatch caused accounts to not be found when loading

### Missing Persistence Layer
- The `SeedManager.create_wallet_from_seed_encrypted()` method creates accounts but doesn't save metadata
- The import functions were trying to use the wrong keystore service to save accounts
- Account metadata wasn't being written to `~/.vaughan/accounts.json`

## Complete Fix Implementation

### 1. Enhanced Keystore Account Loading (`src/security/keystore.rs`)

**Updated `reload_accounts()` method to handle multiple service types:**
```rust
async fn reload_accounts(&mut self) -> Result<()> {
    // Load account metadata from persistent file
    let accounts_file = "~/.vaughan/accounts.json";
    
    if let Ok(content) = std::fs::read_to_string(&accounts_file) {
        if let Ok(stored_accounts) = serde_json::from_str::<Vec<StoredAccountMeta>>(&content) {
            for stored in stored_accounts {
                // Validate that the key reference exists in the keychain
                let key_exists = match stored.key_reference.service.as_str() {
                    "vaughan-wallet-encrypted-seeds" => {
                        // For seed-based accounts, check encrypted seed keychain
                        let seed_keychain = OSKeychain::new("vaughan-wallet-encrypted-seeds".to_string());
                        match seed_keychain {
                            Ok(keychain) => keychain.retrieve(&stored.key_reference).is_ok(),
                            Err(_) => false,
                        }
                    }
                    "vaughan-wallet" => {
                        // For private key accounts, check main keychain
                        self.keychain.retrieve(&stored.key_reference).is_ok()
                    }
                    _ => {
                        tracing::warn!("Unknown service type: {}", stored.key_reference.service);
                        false
                    }
                };
                
                if key_exists {
                    // Load the account
                    let account = SecureAccount { /* ... */ };
                    self.accounts.insert(stored.address, account);
                    tracing::info!("Loaded account: {} ({})", account_name, account_address);
                } else {
                    tracing::warn!("Skipping account {} - key not found", stored.name);
                }
            }
        }
    }
    Ok(())
}
```

### 2. Fixed Seed Phrase Import Persistence (`src/gui/working_wallet.rs`)

**Updated `import_wallet_from_seed()` to properly save account metadata:**
```rust
async fn import_wallet_from_seed(name: String, seed: String, password: String) -> Result<String, String> {
    // Create wallet using SeedManager (stores encrypted seed)
    let account = seed_manager.create_wallet_from_seed_encrypted(
        name,
        &secure_seed,
        &secure_password,
        None
    ).await?;
    
    // Manually save account metadata to accounts.json
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut vaughan_path = std::path::PathBuf::from(home_dir);
    vaughan_path.push(".vaughan");
    vaughan_path.push("accounts.json");
    
    // Load existing accounts
    let mut existing_accounts = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&accounts_file) {
        if let Ok(accounts) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            existing_accounts = accounts;
        }
    }
    
    // Add new account metadata
    let new_account = serde_json::json!({
        "id": account.id,
        "name": account.name,
        "address": account.address,
        "key_reference": account.key_reference,
        "created_at": account.created_at,
        "is_hardware": account.is_hardware,
        "derivation_path": account.derivation_path
    });
    
    // Check for duplicates and save
    if !account_exists {
        existing_accounts.push(new_account);
        let json_content = serde_json::to_string_pretty(&existing_accounts)?;
        std::fs::write(&accounts_file, json_content)?;
        tracing::info!("Saved account metadata to {}", accounts_file);
    }
    
    Ok(format!("{:?}", account.address))
}
```

### 3. Service-Agnostic Account Management

**The keystore now handles both account types:**
- **Seed-based accounts**: Service `vaughan-wallet-encrypted-seeds`, keys stored as encrypted seed phrases
- **Private key accounts**: Service `vaughan-wallet`, keys stored as raw private keys
- **Validation**: Each account type is validated against its respective keychain before loading

## Account Storage Architecture

### File Structure
```
~/.vaughan/
├── accounts.json          # Account metadata (addresses, names, key references)
└── (OS Keychain)         # Actual cryptographic keys
    ├── vaughan-wallet-encrypted-seeds/  # Encrypted seed phrases
    └── vaughan-wallet/                   # Private keys
```

### Account Metadata Format
```json
[
  {
    "id": "uuid",
    "name": "Account Name",
    "address": "0x...",
    "key_reference": {
      "id": "key-id",
      "service": "vaughan-wallet-encrypted-seeds",  // or "vaughan-wallet"
      "account": "key-account-name"
    },
    "created_at": "2025-08-10T...",
    "is_hardware": false,
    "derivation_path": "m/44'/60'/0'/0/0"
  }
]
```

## Expected Behavior After Fix

### Wallet Import from Seed Phrase:
1. ✅ SeedManager encrypts and stores seed phrase in keychain
2. ✅ Account metadata is saved to accounts.json
3. ✅ Account appears in dropdown immediately
4. ✅ Account persists across wallet restarts
5. ✅ Balance loading works correctly

### Wallet Import from Private Key:
1. ✅ Private key is stored in keychain
2. ✅ Account metadata is saved to accounts.json
3. ✅ Account appears in dropdown immediately
4. ✅ Account persists across wallet restarts
5. ✅ Balance loading works correctly

### Account Loading on Startup:
1. ✅ Reads accounts.json for metadata
2. ✅ Validates each account's key exists in appropriate keychain
3. ✅ Only loads accounts with valid keys
4. ✅ Handles both service types correctly
5. ✅ Provides clear logging for debugging

## Testing the Fix

### To test the import and persistence:
1. Build: `cargo build --release`
2. Run: `cargo run --bin vaughan --release`
3. Import a wallet from seed phrase
4. Verify account appears in dropdown
5. Restart the wallet
6. Verify account still appears and balance loads
7. Import a wallet from private key
8. Verify both accounts work correctly

### Debugging Commands:
```bash
# Check account metadata
cat ~/.vaughan/accounts.json

# Check if keys exist in keychain (Linux)
secret-tool search service vaughan-wallet-encrypted-seeds
secret-tool search service vaughan-wallet
```

The wallet import and persistence system should now work reliably with proper separation of concerns between different account types and robust error handling.