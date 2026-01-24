# Professional Wallet Improvement - Design

**Feature**: Professional Wallet Codebase Excellence
**Status**: Draft
**Created**: 2025-01-24

## 1. Architecture Overview

This design document outlines the technical approach for transforming Vaughan Wallet into a production-grade professional codebase through security auditing, property-based testing, and modularization.

### 1.1 Design Principles

1. **Security First**: All changes prioritize security over convenience
2. **Test-Driven Refactoring**: Property tests before code changes
3. **Incremental Improvement**: Small, validated steps with rollback capability
4. **Industry Standards**: Follow Alloy patterns, MetaMask where needed
5. **Zero Regression**: Maintain all existing functionality and performance

### 1.2 Technology Stack

**Primary Libraries (Alloy):**
- `alloy-signer` - Transaction signing abstractions
- `alloy-provider` - Network provider interfaces
- `alloy-network` - Network type definitions
- `alloy-primitives` - Core Ethereum types

**Secondary Patterns (MetaMask):**
- Hardware wallet integration (Trezor/Ledger)
- Device communication protocols
- Error handling patterns
- (Used only where Alloy insufficient, with attribution)

**Testing:**
- `proptest` - Property-based testing framework
- Standard Rust test framework for unit tests
- Criterion for performance benchmarks

## 2. Module Architecture

### 2.1 Current State Analysis

**Oversized Modules:**
```
account_manager/mod.rs:        1,777 lines (8.9x over limit)
account_manager/import.rs:       964 lines (4.8x over limit)
performance/batch.rs:            878 lines (4.4x over limit)
telemetry/account_events.rs:    801 lines (4.0x over limit)
account_manager/metadata.rs:     281 lines (1.4x over limit)
```

### 2.2 Target Architecture

**Module Size Limits:**
- Coordinator modules: 400 lines maximum
- Logic modules: 200 lines maximum
- Rationale: Maintainability, testability, cognitive load

**Refactored Structure:**

```
src/wallet/account_manager/
├── mod.rs                    (~50 lines - re-exports only)
├── coordinator.rs            (~300 lines - trait orchestration)
├── types.rs                  (~100 lines - type definitions)
├── lifecycle.rs              (~200 lines - CRUD operations)
├── auth.rs                   (~150 lines - authentication)
├── import/
│   ├── mod.rs               (~50 lines - re-exports)
│   ├── parsers.rs           (~200 lines - format parsing)
│   ├── validators.rs        (~150 lines - validation)
│   └── converters.rs        (~150 lines - conversion)
└── metadata/
    └── (split if needed)

src/performance/
├── mod.rs
└── batch/
    ├── mod.rs               (~50 lines - re-exports)
    ├── config.rs            (~100 lines - configuration)
    ├── processor.rs         (~200 lines - core logic)
    └── retry.rs             (~150 lines - retry/backoff)

src/telemetry/
├── mod.rs
└── account_events/
    ├── mod.rs               (~50 lines - re-exports)
    ├── logger.rs            (~150 lines - logging)
    ├── spans.rs             (~150 lines - span management)
    └── privacy.rs           (~100 lines - privacy filtering)
```

## 3. Security Design

### 3.1 Memory Safety

**Zeroization Strategy:**
- All sensitive types implement `Zeroize` trait
- Drop implementations call `zeroize()` explicitly
- Memory tests verify clearing after use

**Sensitive Data Types:**
```rust
// All must implement Zeroize
- PrivateKey
- Mnemonic
- Password
- EncryptedSeed
- SessionKey
```

### 3.2 Constant-Time Operations

**Requirements:**
- All cryptographic operations must be constant-time
- No branching on secret data
- Use Alloy's constant-time primitives where available

**Critical Operations:**
- Signature generation
- Signature verification
- Key derivation (BIP-32)
- HMAC operations

### 3.3 Hardware Wallet Security

**Trezor Integration (MetaMask Pattern):**
- Device communication via USB HID
- Transaction signing on-device
- No private key exposure
- Proper error handling for device disconnection

**Ledger Integration (MetaMask Pattern):**
- Device communication via USB HID
- BIP-44 derivation path support
- Transaction signing on-device
- Proper error handling for device disconnection

### 3.4 Side-Channel Attack Mitigation

**Timing Attacks:**
- Constant-time comparison for secrets
- No secret-dependent branching
- Constant-time cryptographic operations

**Cache-Timing Attacks:**
- Use cache-resistant implementations where available
- Document cache-timing risks

**Power Analysis:**
- Hardware wallet operations only (device-side protection)
- No software-side power analysis protection needed

## 4. Property-Based Testing Design

### 4.1 Property Test Framework

**Tool**: `proptest` crate
**Configuration:**
- Timeout: 60 seconds per property
- Shrinking: Enabled for counterexample minimization
- Persistence: Save failing cases to regression files

### 4.2 Critical Security Properties

#### Property 1: Unified Interface Consistency
**Description**: AccountManager trait operations maintain consistency
**Iterations**: 1,000
**Test**: `create(account) then get(id) returns same account`

#### Property 2: Concurrent Operation Safety
**Description**: Concurrent account operations maintain consistency
**Iterations**: 1,000
**Test**: Parallel operations don't cause data races or inconsistency

#### Property 3: Lock Memory Clearing
**Description**: Memory is cleared on wallet lock
**Iterations**: 10,000 (industry standard for memory safety)
**Test**: After lock, no sensitive data remains in memory

#### Property 20: Seed Phrase Import Determinism
**Description**: Same mnemonic always produces same keys
**Iterations**: 1,000
**Test**: `import(mnemonic) twice yields identical keys`

#### Property 31: Shamir Secret Sharing Round-Trip
**Description**: SSS split/combine is lossless
**Iterations**: 1,000
**Test**: `combine(split(secret)) == secret` for all threshold configs

### 4.3 Functional Properties

#### Property 8: Error Context Completeness
**Description**: All errors contain actionable context
**Iterations**: 500
**Test**: Every error has context, source, and recovery hint

#### Property 24: LRU Cache Correctness
**Description**: LRU eviction maintains correctness
**Iterations**: 500
**Test**: Cache operations maintain consistency and correct eviction

#### Property 33: Nickname Uniqueness
**Description**: Account nicknames remain unique
**Iterations**: 500
**Test**: No duplicate nicknames, collision handling correct

### 4.4 Remaining Properties (27 total)

**To be defined based on:**
- Existing design documents
- Code analysis
- Security requirements
- Functional requirements

**Minimum iterations**: 100 per property

## 5. Performance Design

### 5.1 Performance Preservation

**Current Benchmarks (must maintain):**
- Batch operations: 244-270% improvement over baseline
- LRU cache: 10,534x speedup over uncached
- Lock operation: 11.8µs average
- Unlock operation: 1.9µs average

### 5.2 Performance Testing Strategy

**Benchmark Suite:**
```bash
cargo bench --bench account_manager_benchmarks
cargo bench --bench wallet_benchmarks
```

**Validation:**
- Run before any changes (baseline)
- Run after each phase
- Immediate rollback if >5% regression

### 5.3 Build Time Optimization

**Target**: Build time increase <10%
**Strategy:**
- Minimize new dependencies
- Use feature flags for optional functionality
- Incremental compilation friendly code structure

## 6. Documentation Design

### 6.1 Rustdoc Standards

**All Public APIs Must Have:**
- Summary line (one sentence)
- Detailed description
- Parameter documentation
- Return value documentation
- Error conditions
- Examples for complex APIs
- Performance characteristics

**Example:**
```rust
/// Creates a new account with the specified parameters.
///
/// This operation derives a new account from the HD wallet seed using
/// BIP-44 derivation. The operation is deterministic and will always
/// produce the same account for the same derivation path.
///
/// # Parameters
/// - `nickname`: Human-readable account name (must be unique)
/// - `derivation_path`: BIP-44 derivation path
///
/// # Returns
/// - `Ok(Account)`: Successfully created account
/// - `Err(AccountError)`: If nickname is duplicate or derivation fails
///
/// # Performance
/// - Time complexity: O(1) for account creation
/// - Space complexity: O(1)
/// - Typical execution time: <1ms
///
/// # Examples
/// ```
/// let account = manager.create_account("Main", "m/44'/60'/0'/0/0")?;
/// ```
pub fn create_account(&self, nickname: &str, derivation_path: &str) 
    -> Result<Account, AccountError>
```

### 6.2 Code Attribution

**Alloy Code:**
- No attribution needed (primary library)
- Document which Alloy types/traits used

**MetaMask-Inspired Code:**
- Add comment: `// Pattern inspired by MetaMask: <reason>`
- Document why Alloy insufficient
- Reference MetaMask source if applicable

**Example:**
```rust
// Pattern inspired by MetaMask: Alloy doesn't provide Ledger integration
// Reference: https://github.com/MetaMask/eth-ledger-bridge-keyring
pub struct LedgerDevice {
    // ...
}
```

## 7. Error Handling Design

### 7.1 Error Types

**Hierarchy:**
```rust
pub enum WalletError {
    Account(AccountError),
    Crypto(CryptoError),
    Hardware(HardwareError),
    Network(NetworkError),
    // ...
}
```

**Each Error Must Provide:**
- Context (what operation failed)
- Source (underlying cause)
- Recovery hint (what user can do)

### 7.2 Error Context

**Example:**
```rust
AccountError::DuplicateNickname {
    nickname: String,
    existing_account_id: AccountId,
    hint: "Choose a different nickname or rename the existing account",
}
```

## 8. Testing Strategy

### 8.1 Test Pyramid

```
        /\
       /  \  Property Tests (35 properties, 100-10,000 iterations)
      /____\
     /      \  Integration Tests (17 tests)
    /________\
   /          \  Unit Tests (305 tests)
  /____________\
```

### 8.2 Test Coverage Goals

- Unit test coverage: 90%+
- Integration test coverage: All critical paths
- Property test coverage: All security properties
- Performance test coverage: All critical operations

### 8.3 Test Execution

**Pre-commit:**
```bash
cargo test --all-features
cargo clippy -- -D warnings
```

**Pre-merge:**
```bash
cargo test --all-features --release
cargo bench
```

**Pre-release:**
```bash
cargo test --all-features --release
cargo bench
cargo audit
Manual hardware wallet testing
```

## 9. Rollback Strategy

### 9.1 Phase-Level Rollback

**Each phase has rollback procedure:**
- Git commit before phase start
- Git tag for phase completion
- Documented rollback steps in tasks

**Rollback Triggers:**
- Any test failure
- Performance regression >5%
- Security regression
- Build time increase >10%

### 9.2 Emergency Rollback

**Procedure:**
```bash
# Identify last known good commit
git log --oneline

# Rollback to last good state
git reset --hard <commit-hash>

# Document issue
echo "Rollback reason: <description>" >> rollback_log.txt
```

## 10. Success Criteria

### 10.1 Code Quality

- ✅ All modules < 400 lines (coordinators) or < 200 lines (logic)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Complete rustdoc coverage
- ✅ All public APIs documented

### 10.2 Security

- ✅ All unsafe blocks documented
- ✅ All crypto operations constant-time verified
- ✅ All sensitive data zeroization verified
- ✅ All 35 property tests passing
- ✅ No security regressions

### 10.3 Performance

- ✅ No regression from baseline benchmarks
- ✅ All 333+ tests passing
- ✅ Build time increase <10%

### 10.4 Documentation

- ✅ All public APIs documented
- ✅ All error conditions documented
- ✅ Hardware wallet integration documented
- ✅ Code attribution complete

## 11. Future Considerations

### 11.1 Out of Scope (This Initiative)

- New feature development
- UI/UX changes
- Network protocol changes
- Additional hardware wallet support
- Migration to different crypto libraries

### 11.2 Future Improvements

- Formal verification of critical properties
- Fuzzing integration
- Additional hardware wallet support (YubiKey, etc.)
- Performance optimizations beyond current levels
- Additional property tests beyond 35

## 12. References

- [Alloy Documentation](https://github.com/alloy-rs/alloy)
- [MetaMask Repository](https://github.com/MetaMask)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Secure Code Working Group](https://github.com/rust-secure-code/wg)
- BIP-32: HD Wallet Derivation
- BIP-39: Mnemonic Generation
- BIP-44: Multi-Account Hierarchy
- EIP-712: Typed Data Signing
- [PropTest Book](https://altsysrq.github.io/proptest-book/)
