# Hardware Wallet Integration Guide

Vaughan supports hardware wallets (Ledger, Trezor) via the `AccountManager`. This integration respects standard derivation paths (BIP-44) to ensure compatibility with other wallets like MetaMask.

## Supported Devices

-   **Ledger**: Nano S, Nano X, Stax (via `alloy-signer-ledger` pattern)
-   **Trezor**: Model One, Model T (via `alloy-signer-trezor` pattern)

## Derivation Paths

We enforce standard paths to ensure users find their funds expectedly.

### Standards

| Standard | Path Pattern | Purpose |
|----------|-------------|---------|
| **BIP-44 (MetaMask)** | `m/44'/60'/0'/0/x` | Standard Ethereum accounts (Ledger Live/MetaMask) |
| **BIP-44 (Legacy)** | `m/44'/60'/0'/x` | Older integrations |
| **Ledger Live** | `m/44'/60'/x'/0/0` | Ledger Live specific pattern |

### Configuration

When creating a hardware account, specify the derivation standard:

```rust
use vaughan::wallet::hardware::DerivationStandard;
use vaughan::wallet::account_manager::AccountConfig;

// Create account configuration
let config = AccountConfig::hardware("My Ledger")
    .with_standard(DerivationStandard::Bip44) // Default
    // Note: To target a specific index, use explicit path:
    .with_derivation_path("m/44'/60'/0'/0/1"); // Index 1
```

## Setup & Troubleshooting

### Ledger
1.  Connect device via USB.
2.  Unlock device with PIN.
3.  Open "Ethereum" app on device.
4.  **Important**: Enable "Blind Signing" if interacting with smart contracts.

### Trezor
1.  Connect device via USB.
2.  Install Trezor Bridge (if required by transport).
3.  Unlock via PIN/Passphrase.

### Common Issues

-   **"Device not found"**: Ensure no other wallet (Ledger Live, MetaMask web) is claiming the USB connection.
-   **"Locked"**: The device must be unlocked before the `AccountManager` requests a signer.
-   **"Invalid Path"**: Ensure you are using the correct derivation standard. If you migrated from an old wallet, try `DerivationStandard::Bip44Legacy`.
