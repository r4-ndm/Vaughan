# Account Management Guide

The `AccountManager` is the core component of the Vaughan wallet, providing a unified interface for managing Seed-based, Private Key, and Hardware wallet accounts. It leverages the [Alloy](https://github.com/alloy-rs/alloy) library for all blockchain interactions and cryptographic operations.

## Key Features

- **Unified Interface**: Treat all account types uniformly via `SecureAccount`.
- **Alloy Integration**: Native support for Alloy `Signer` and `Provider` traits.
- **Hardware Support**: Integrated support for Ledger and Trezor devices.
- **Security**: Rate limiting, secure storage, and zeroization of sensitive data.
- **Discovery**: BIP-44 automated account discovery.

## Usage Examples

### 1. Creating a Seed-Based Wallet

```rust
use vaughan::wallet::account_manager::{AccountManager, AccountConfig, SeedStrength};

async fn create_wallet() -> Result<()> {
    let manager = AccountManager::new(keystore, secure_memory);
    
    let config = AccountConfig::seed_based("Main Wallet")
        .with_seed_strength(SeedStrength::Words24);

    let password = SecretString::new("secure-password".to_string());
        
    let account = manager.create_account(config, &password).await?;
    println!("Created account: {}", account.address);
    Ok(())
}
```

### 2. Importing a Private Key

```rust
use vaughan::wallet::account_manager::{AccountManager, ImportSource};
use secrecy::SecretString;

async fn import_key() -> Result<()> {
    let manager = AccountManager::new(keystore, secure_memory);
    
    let source = ImportSource::PrivateKey {
        key: SecretString::new("0x...".to_string()),
        name: "Imported Logic".to_string(),
        password: SecretString::new("secure-password".to_string()),
    };
        
    let account = manager.import_account(source).await?;
    Ok(())
}
```

### 3. Using with Alloy Providers

The `AccountManager` provides seamless integration with Alloy. You can retrieve a signer or a full provider for any account.

```rust
use alloy::providers::ProviderBuilder;
use vaughan::wallet::account_manager::SignerManager;

async fn send_transaction(manager: &AccountManager, account_id: &str) -> Result<()> {
    // 1. Get the signer (requires unlocking the wallet first)
    manager.unlock("secure-password").await?;
    let signer = manager.signer().get_signer(account_id).await?;
    
    // 2. Create an Alloy provider with the signer
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(signer.into())
        .on_http("https://eth-mainnet.g.alchemy.com/v2/...".parse()?);
        
    // 3. Send a transaction
    let tx = TransactionRequest::default()
        .with_to(receiver)
        .with_value(U256::from(100));
        
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;
    println!("Tx Hash: {}", receipt.transaction_hash);
    Ok(())
}
```

### 4. Account Discovery

Automatically find active accounts from a seed phrase.

```rust
async fn discover(manager: &AccountManager) -> Result<()> {
    // Discover up to 20 empty accounts (gap limit)
    let accounts = manager.discover_accounts(20).await?;
    
    for acc in accounts {
        println!("Found active account: {} (Balance: {})", acc.address, acc.balance);
        // Option to add it to the manager
        manager.add_discovered_account(acc).await?;
    }
    Ok(())
}
```

## Batch Operations

The `IntegratedAccountService` allows for efficient batch processing of read-only operations using Multicall3 where supported.

```rust
use vaughan::gui::services::IntegratedAccountService;

async fn refresh_all(service: &IntegratedAccountService, accounts: &[SecureAccount]) {
    // Automatically uses Multicall3 if available on the chain
    let balances = service.refresh_account_balances(accounts, fetcher).await?;
}
```
