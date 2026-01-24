# Migration Guide: Moving to the Enhanced Account Manager

This guide assists developers in migrating from the legacy `v0.1` wallet implementation to the new Enhanced Account Management system (`v0.2`).

## Key Changes

| Feature | Legacy (`v0.1`) | Enhanced (`v0.2`) |
|---------|----------------|-------------------|
| **Library** | `ethers-rs` | `alloy-rs` (v0.1.x) |
| **Signer Type** | `LocalWallet` | `Box<dyn Signer>` (Alloy trait) |
| **Keystore** | Custom JSON | MetaMask V3 Standard |
| **Discovery** | Manual | Automated BIP-44 |
| **Provider** | `Provider<Http>` | `Eip1193Provider` / Alloy Provider |

## Code Migration

### 1. Account Creation

**Legacy:**
```rust
let wallet = LocalWallet::new(&mut rng);
// stored manually
```

**New:**
```rust
let config = AccountConfig::seed_based("My Wallet");
let password = SecretString::new("password".to_string());
let account = manager.create_account(config, &password).await?;
// automatically encrypted and stored in secure keystore
```

### 2. Signing Transactions

**Legacy:**
```rust
let tx = TransactionRequest::new().to(to).value(100);
let signature = wallet.sign_transaction(&tx).await?;
```

**New:**
```rust
// Use Alloy's native flow
let signer = manager.signer().get_signer(id).await?;
let tx = TransactionRequest::default().with_to(to).with_value(U256::from(100));
let signature = signer.sign_transaction(&tx).await?;
```

### 3. Provider Integration

**Legacy:**
```rust
let client = SignerMiddleware::new(provider, wallet);
```

**New:**
```rust
// Alloy uses a builder pattern for composition
let provider = ProviderBuilder::new()
    .wallet(EthereumWallet::from(signer))
    .on_http(url);
```

## Data Migration

The new system uses a different storage format (MetaMask V3).
To migrate existing keys:

1.  Export the private key from the legacy system.
2.  Import into the new `AccountManager` using `AccountConfig::private_key()`.
3.  The manager will automatically encrypt it using the new standard.

```rust
// Migration utility snippet
use vaughan::wallet::account_manager::ImportSource;
use secrecy::SecretString;

for old_account in legacy_store.load_all() {
    let pk_string = old_account.decrypt_private_key(password)?;
    
    let source = ImportSource::PrivateKey {
        key: SecretString::new(pk_string),
        name: old_account.name.clone(),
        password: SecretString::new(password.to_string()),
    };
        
    manager.import_account(source).await?;
}
```
