# Vaughan Wallet

A Prototype Ethereum wallet built with Rust, featuring hardware wallet support and a beautiful native interface.

## Overview

Vaughan is a desktop cryptocurrency wallet designed for security, performance, and ease of use. Built entirely in Rust using the [Alloy](https://github.com/alloy-rs/alloy) library for Ethereum interactions, it provides a fast, reliable, and type-safe foundation for managing your digital assets.

## Key Features

### 🔐 Security First
- **Hardware Wallet Support**: Full integration with Ledger and Trezor devices
- **Secure Memory Management**: Automatic memory locking and zeroization
- **AES-256-CTR Encryption**: MetaMask-compatible keystore encryption with PBKDF2
- **OS Keychain Integration**: Secure key storage using native OS keychains
- **BIP-39/BIP-44 Compliant**: Standard-compliant HD wallet implementation

### ⚡ Performance
- **Native Rust Application**: Fast, efficient, and memory-safe
- **Alloy-Powered**: Built on Alloy, the next-generation Ethereum library
- **Async/Await**: Non-blocking operations for smooth user experience
- **Optimized Builds**: Small binary size with release optimizations

### 🎨 User Experience
- **Native UI**: Built with [Iced](https://github.com/iced-rs/iced) for a responsive desktop experience
- **Multi-Network Support**: Ethereum, PulseChain, BSC, Polygon, and custom networks
- **Transaction History**: Complete transaction tracking and history
- **Custom Tokens**: Easy ERC-20 token management
- **Gas Estimation**: Automatic gas price estimation and optimization

### 🔧 Developer Friendly
- **Type-Safe**: Rust's type system prevents common bugs
- **Well-Documented**: Comprehensive inline documentation
- **Modular Architecture**: Clean separation of concerns
- **Extensive Testing**: Unit and integration tests

## Technology Stack

- **Language**: Rust 2021 Edition
- **Blockchain Library**: [Alloy](https://github.com/alloy-rs/alloy) - Modern, high-performance Ethereum library
- **UI Framework**: [Iced](https://github.com/iced-rs/iced) - Cross-platform GUI library
- **Cryptography**: 
  - `k256` - secp256k1 elliptic curve
  - `aes-gcm` - Authenticated encryption
  - `pbkdf2` - Key derivation
- **Hardware Wallets**: Custom Ledger/Trezor integration
- **Async Runtime**: Tokio

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/vaughan-team/vaughan.git
cd vaughan

# Build in release mode
cargo build --release

# Run the wallet
./target/release/vaughan
```

### Quick Build Script

```bash
# Use the provided build script
./tools/build/build.sh
```

## Feature Flags

Vaughan uses Cargo feature flags to allow customization of the wallet's capabilities. This enables you to build a minimal wallet or include advanced features based on your needs.

### Available Features

#### Core Features

**`minimal`** (included in default)
- Core wallet functionality only
- Account management, transaction signing, balance checking
- No additional dependencies
- **Build Time Impact**: Baseline (~2 minutes)
- **Binary Size**: ~15 MB

**`qr`** (included in default)
- QR code generation for addresses and payment requests
- EIP-681 payment request format support
- Dependencies: `qrcode`, `image`
- **Build Time Impact**: +10 seconds
- **Binary Size**: +2 MB
- **Use Case**: Sharing addresses, mobile wallet integration

**`audio`** (included in default)
- Audio notifications for incoming transactions
- Custom alert sound support
- Dependencies: `rodio`
- **Build Time Impact**: +15 seconds
- **Binary Size**: +3 MB
- **Use Case**: Desktop notifications, accessibility

**`hardware-wallets`** (included in default)
- Ledger and Trezor hardware wallet support
- Uses Alloy native signers (`alloy-signer-ledger`, `alloy-signer-trezor`)
- On-device transaction signing
- Dependencies: `alloy-signer-ledger`, `alloy-signer-trezor`
- **Build Time Impact**: +30 seconds
- **Binary Size**: +5 MB
- **Use Case**: Maximum security for large holdings

**`professional`** (included in default)
- Professional network monitoring features
- Advanced RPC health checking
- Network performance metrics
- **Build Time Impact**: Minimal
- **Binary Size**: +500 KB
- **Use Case**: Power users, developers

**`custom-tokens`** (included in default)
- Custom ERC-20 token management
- Token metadata fetching
- Token price tracking
- **Build Time Impact**: Minimal
- **Binary Size**: +300 KB
- **Use Case**: DeFi users, token traders

#### Advanced Features

**`shamir`** (not in default)
- Shamir's Secret Sharing for seed phrase backup
- Split seed into N shares, require M to recover
- Dependencies: `sharks`
- **Build Time Impact**: +5 seconds
- **Binary Size**: +1 MB
- **Use Case**: Advanced backup strategies, multi-sig-like recovery

**`telemetry`** (not in default)
- OpenTelemetry metrics and tracing
- Performance monitoring
- Error tracking
- Dependencies: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`
- **Build Time Impact**: +45 seconds
- **Binary Size**: +8 MB
- **Use Case**: Development, debugging, production monitoring

**`full`**
- Enables all features: `qr`, `audio`, `hardware-wallets`, `professional`, `custom-tokens`, `shamir`, `telemetry`
- **Build Time Impact**: +2 minutes
- **Binary Size**: ~35 MB
- **Use Case**: Complete feature set for power users

### Feature Combinations

#### Default Build (Recommended)
```bash
cargo build --release
# Includes: minimal, qr, audio, hardware-wallets, professional, custom-tokens
```

**Best for**: Most users, balanced features and build time

#### Minimal Build (Fastest)
```bash
cargo build --release --no-default-features --features minimal
```

**Best for**: Development, testing, embedded systems
**Build Time**: ~2 minutes
**Binary Size**: ~15 MB

#### Full Build (All Features)
```bash
cargo build --release --features full
```

**Best for**: Power users, complete feature set
**Build Time**: ~4 minutes
**Binary Size**: ~35 MB

#### Custom Build Examples

**Wallet with hardware support only:**
```bash
cargo build --release --no-default-features --features "minimal,hardware-wallets"
```

**Wallet with QR and audio:**
```bash
cargo build --release --no-default-features --features "minimal,qr,audio"
```

**Development build with telemetry:**
```bash
cargo build --release --features "default,telemetry"
```

### Testing with Features

#### Test All Features
```bash
cargo test --all-features
```

#### Test Specific Feature
```bash
cargo test --features hardware-wallets
```

#### Test Minimal Configuration
```bash
cargo test --no-default-features --features minimal
```

### Feature Dependencies

Some features have dependencies on others:

- `full` → enables all other features
- `hardware-wallets` → requires platform-specific USB libraries
- `telemetry` → requires network connectivity for metrics export
- `audio` → requires audio output device

### Build Time Comparison

| Configuration | Build Time | Binary Size | Features |
|--------------|------------|-------------|----------|
| Minimal | ~2 min | ~15 MB | Core only |
| Default | ~3 min | ~25 MB | Recommended set |
| Full | ~4 min | ~35 MB | All features |

**Note**: Build times are approximate and depend on your system. Incremental builds are much faster.

### Recommended Configurations

**For End Users:**
```bash
cargo build --release
# Uses default features - best balance
```

**For Developers:**
```bash
cargo build --no-default-features --features "minimal,telemetry"
# Fast builds with debugging capabilities
```

**For Maximum Security:**
```bash
cargo build --release --features "minimal,hardware-wallets,shamir"
# Hardware wallet + advanced backup
```

**For DeFi Users:**
```bash
cargo build --release --features "default,shamir"
# All default features + advanced backup
```

## Usage

### First Launch

1. **Create a New Wallet**: Generate a new seed phrase
2. **Import Existing Wallet**: Use your existing seed phrase or private key
3. **Connect Hardware Wallet**: Plug in your Ledger or Trezor device

### Managing Networks

Vaughan comes pre-configured with popular networks:
- Ethereum Mainnet
- PulseChain Mainnet & Testnet
- Binance Smart Chain
- Polygon

You can also add custom networks through the settings.

### Sending Transactions

1. Select your account
2. Click "Send"
3. Enter recipient address and amount
4. Review gas settings
5. Confirm transaction

### Adding Custom Tokens

1. Go to token management
2. Enter token contract address
3. Token details are automatically fetched
4. Confirm to add

## Configuration

### First Launch

1. **Create a New Wallet**: Generate a new seed phrase
2. **Import Existing Wallet**: Use your existing seed phrase or private key
3. **Connect Hardware Wallet**: Plug in your Ledger or Trezor device

### Managing Networks

Vaughan comes pre-configured with popular networks:
- Ethereum Mainnet
- PulseChain Mainnet & Testnet
- Binance Smart Chain
- Polygon

You can also add custom networks through the settings.

### Sending Transactions

1. Select your account
2. Click "Send"
3. Enter recipient address and amount
4. Review gas settings
5. Confirm transaction

### Adding Custom Tokens

1. Go to token management
2. Enter token contract address
3. Token details are automatically fetched
4. Confirm to add

## Configuration

### API Keys (Optional but Recommended)

Vaughan works out-of-the-box with public APIs, but you can improve performance and reliability by adding your own free API keys.

#### Moralis API Key (For Price Feeds & Transaction History)

Moralis provides both price feeds and transaction history. Get a free API key:

1. **Get a free Moralis API key**:
   - Visit [moralis.io](https://moralis.io/)
   - Sign up for a free account (100k requests/month free)
   - Create a new project
   - Copy your API key

2. **Add to Vaughan**:
   ```bash
   # Copy the template
   cp config/api_config.template.toml config/api_config.toml
   
   # Edit and add your key
   nano config/api_config.toml
   ```

3. **Update the config**:
   ```toml
   [moralis]
   api_key = "your_moralis_api_key_here"
   enable_price_feeds = true
   ```

**What works without Moralis**:
- ✅ Sending/receiving transactions
- ✅ Balance checking
- ✅ Token management
- ⚠️ Price feeds (slower, uses CoinGecko fallback)
- ⚠️ Transaction history (limited)

**What requires Moralis or Block Explorer APIs**:
- Full transaction history
- Real-time price updates
- Token metadata

#### Block Explorer API Keys (Optional - For Transaction History)

For better transaction history on specific chains:

```bash
# Copy the template
cp config/explorer_apis.json.template config/explorer_apis.json

# Edit and add your keys
nano config/explorer_apis.json
```

**Free API Keys Available From**:
- [Etherscan](https://etherscan.io/apis) - Ethereum
- [BSCScan](https://bscscan.com/apis) - Binance Smart Chain
- [PolygonScan](https://polygonscan.com/apis) - Polygon

**Note**: These are optional. Moralis can provide transaction history for multiple chains with a single API key.

#### RPC API Keys (For Better Network Performance)

Create a `.env` file for optional RPC configuration:

```bash
# Alchemy (recommended for Ethereum)
ALCHEMY_API_KEY=your_key_here

# Infura (alternative for Ethereum)
INFURA_API_KEY=your_key_here
```

**Free API Keys Available From**:
- [Alchemy](https://www.alchemy.com/) - 300M compute units/month free
- [Infura](https://infura.io/) - 100k requests/day free
- [Moralis](https://moralis.io/) - 100k requests/month free

### Environment Variables (Optional)

```bash
# Graphics Settings (if you have display issues)
VAUGHAN_SOFTWARE_RENDERING=1  # Force software rendering
VAUGHAN_MINIMAL_MODE=1        # Minimal graphics mode

# Logging
RUST_LOG=info  # Options: error, warn, info, debug, trace
```

See `.env.example` for more details.

## Architecture

### Core Components

```
vaughan/
├── src/
│   ├── blockchain/      # Blockchain interaction (Alloy-based)
│   ├── security/        # Cryptography and key management
│   ├── network/         # Network configuration and RPC
│   ├── gui/            # User interface (Iced)
│   │   ├── state/      # Application state management
│   │   ├── views/      # UI components
│   │   ├── handlers/   # Event handlers
│   │   └── services/   # Business logic
│   └── bin/            # Binary entry points
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
└── docs/               # Documentation
```

### State Management

Vaughan uses a domain-separated state architecture:
- **NetworkState**: Network and RPC management
- **WalletState**: Account and key management
- **TransactionState**: Transaction handling
- **UiState**: User interface state

## Security

### Audit Status

A comprehensive security audit has been completed. See [docs/COMPREHENSIVE_SECURITY_AUDIT.md](docs/COMPREHENSIVE_SECURITY_AUDIT.md) for details.

### Security Features

- ✅ Memory protection with `mlock`/`VirtualLock`
- ✅ Automatic memory zeroization
- ✅ Secure random number generation
- ✅ Hardware wallet support
- ✅ Encrypted local storage
- ✅ OS keychain integration

### Wallet Keystore Format

Vaughan uses a **MetaMask V3-compatible keystore format** for wallet encryption. This ensures interoperability with other Ethereum wallets and follows industry-standard security practices.

#### Technical Specifications

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Version** | 3 | MetaMask V3 keystore standard |
| **Cipher** | AES-256-CTR | Stream cipher for encryption |
| **KDF** | PBKDF2 | Key derivation function |
| **PBKDF2 Iterations** | 262,144 | Same as MetaMask standard |
| **PRF** | HMAC-SHA256 | Pseudorandom function |
| **Key Length** | 32 bytes (256 bits) | Strong encryption |
| **MAC** | SHA-256 | Integrity verification |

#### Keystore File Location

```
~/.vaughan/keystore.json
```

#### Example Keystore Structure

```json
{
  "version": 3,
  "id": "unique-uuid-v4",
  "address": "0x...",
  "crypto": {
    "cipher": "aes-256-ctr",
    "ciphertext": "...",
    "cipherparams": { "iv": "..." },
    "kdf": "pbkdf2",
    "kdfparams": {
      "dklen": 32,
      "salt": "...",
      "c": 262144,
      "prf": "hmac-sha256"
    },
    "mac": "..."
  }
}
```

### Wallet Operations

#### Creating a Wallet

1. A BIP-39 mnemonic (12 words) is generated from cryptographically secure random entropy
2. The private key is derived using BIP-44 path `m/44'/60'/0'/0/0`
3. The private key is encrypted with your password using PBKDF2 + AES-256-CTR
4. The keystore is saved to `~/.vaughan/keystore.json`

#### Unlocking a Wallet

1. The keystore file is loaded
2. Your password is used with PBKDF2 to derive the encryption key
3. MAC verification ensures password correctness and data integrity
4. The private key is decrypted and loaded into memory

#### Exporting Seed Phrase / Private Key

After unlocking, you can export your seed phrase or private key without entering your password again - just like MetaMask.

### Best Practices

- Always verify addresses before sending
- Use hardware wallets for large amounts
- Keep your seed phrase secure and offline
- Test with small amounts first
- Keep the software updated

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

### Building for Development

```bash
# Debug build (faster compilation)
cargo build

# Run directly
cargo run

# Watch for changes (requires cargo-watch)
cargo watch -x run
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting pull requests.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## Roadmap

- [ ] Mobile companion app
- [ ] Multi-signature support
- [ ] DeFi protocol integration
- [ ] NFT management
- [ ] Advanced analytics
- [ ] Plugin system

## License

This project is licensed under the Galactic Druid License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/vaughan-team/vaughan/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vaughan-team/vaughan/discussions)
- **Security**: See [SECURITY.md](SECURITY.md) for reporting vulnerabilities
- **Documentation**: [docs/](docs/)

## Acknowledgments

- Built with [Alloy](https://github.com/alloy-rs/alloy) - The modern Ethereum library for Rust
- UI powered by [Iced](https://github.com/iced-rs/iced)
- Inspired by the Ethereum and Rust communities

---

**⚠️ Disclaimer**: This is cryptocurrency wallet software. Always verify transactions carefully and keep your seed phrase secure. The developers are not responsible for any loss of funds.
