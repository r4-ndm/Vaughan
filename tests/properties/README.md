# Property-Based Testing Infrastructure

## Overview

This directory contains property-based tests for the Vaughan wallet, following industry standards for security-critical financial applications.

## Test Iteration Standards

Based on industry best practices and Rust Secure Code Working Group guidelines:

- **Memory Safety Properties**: 10,000 iterations
  - Memory clearing on lock
  - Zeroization verification
  - Secure memory allocation

- **Cryptographic Properties**: 1,000 iterations
  - Key derivation determinism
  - Signature verification
  - Shamir Secret Sharing round-trip
  - Constant-time operations

- **Interface Consistency**: 1,000 iterations
  - CRUD operation consistency
  - Concurrent operation safety
  - State machine invariants

- **Functional Properties**: 500 iterations
  - Error context completeness
  - Cache correctness
  - Nickname uniqueness

## Test Organization

```
tests/properties/
├── mod.rs          # Shared utilities and generators
├── security.rs     # Memory safety and security properties
├── crypto.rs       # Cryptographic correctness properties
├── interface.rs    # API consistency properties
└── error.rs        # Error handling properties
```

## Proptest Regression Files

### Policy

Regression files are **COMMITTED** to version control for the following reasons:

1. **Reproducibility**: Ensures all developers can reproduce failing test cases
2. **Security**: Captures edge cases that revealed vulnerabilities
3. **Documentation**: Serves as a record of discovered issues
4. **CI/CD**: Allows continuous integration to verify fixes

### Location

Regression files are stored in:
```
proptest-regressions/
├── security/       # Security property regressions
├── crypto/         # Cryptographic property regressions
├── interface/      # Interface property regressions
└── error/          # Error handling property regressions
```

### Management

- **DO** commit regression files when they capture real issues
- **DO** review regression files during code review
- **DO** update regression files when fixing bugs
- **DON'T** delete regression files without understanding the issue
- **DON'T** add regression files to .gitignore

## Feature Flag Testing

Vaughan uses complex feature flags. Property tests must work with all combinations:

- `minimal`: Core wallet functionality only
- `qr`: QR code generation
- `audio`: Audio notifications
- `hardware-wallets`: Ledger and Trezor support (Alloy signers)
- `professional`: Professional network monitoring
- `custom-tokens`: Custom token management
- `shamir`: Shamir's Secret Sharing
- `telemetry`: OpenTelemetry metrics

### Test Matrix

Property tests run with:
1. **Minimal features**: `cargo test --no-default-features --features minimal`
2. **Default features**: `cargo test`
3. **Full features**: `cargo test --all-features`

## Running Property Tests

### All property tests
```bash
cargo test --test properties
```

### Specific property module
```bash
cargo test --test properties::security
cargo test --test properties::crypto
cargo test --test properties::interface
cargo test --test properties::error
```

### With specific feature set
```bash
# Minimal
cargo test --no-default-features --features minimal --test properties

# Full
cargo test --all-features --test properties
```

### Verbose output
```bash
PROPTEST_VERBOSE=1 cargo test --test properties
```

## Writing New Properties

### Template

```rust
use proptest::prelude::*;
use crate::properties::{crypto_config, arb_mnemonic};

proptest! {
    #![proptest_config(crypto_config())]
    
    #[test]
    fn property_name(mnemonic in arb_mnemonic()) {
        // Setup
        let wallet = create_wallet(&mnemonic);
        
        // Property assertion
        prop_assert!(invariant_holds(&wallet));
    }
}
```

### Best Practices

1. **Use appropriate iteration counts** based on property type
2. **Use provided generators** from `mod.rs`
3. **Document the property** being tested
4. **Link to requirements** in comments
5. **Use `prop_assert!`** for assertions
6. **Handle cleanup** in Drop implementations

## Alloy vs MetaMask Attribution

- **Alloy-first**: All blockchain operations use Alloy libraries
- **MetaMask patterns**: Only where Alloy insufficient (documented)
- **Hardware wallets**: Use Alloy native signers (alloy-signer-ledger, alloy-signer-trezor)
- **Keystore**: EIP-2335 standard (MetaMask-compatible)

## References

- [PropTest Book](https://altsysrq.github.io/proptest-book/)
- [Rust Secure Code Working Group](https://github.com/rust-secure-code/wg)
- [Alloy Documentation](https://github.com/alloy-rs/alloy)
- Design Document: `../../.kiro/specs/professional-wallet-improvement/design.md`
- Requirements: `../../.kiro/specs/professional-wallet-improvement/requirements.md`
