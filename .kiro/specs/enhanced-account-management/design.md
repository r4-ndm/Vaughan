# Enhanced Account Management Design

## Overview

This design document specifies the architecture and implementation approach for enhancing Vaughan's account management system. The design follows industry standards from Alloy libraries and MetaMask while maintaining security, performance, and maintainability.

**Key Design Principles:**
- **Alloy-First**: Use Alloy libraries for all blockchain operations
- **Security by Default**: Proper locking, encryption, and authentication
- **Performance**: Batch processing, caching, and async patterns
- **Observability**: Structured logging with correlation tracking
- **Testability**: Property-based testing for cryptographic correctness

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         GUI Layer                            │
│  (Iced UI Components - account_manager.rs)                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   Account Manager Trait                      │
│  (Unified interface for all account operations)             │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┬──────────────┐
        ▼            ▼            ▼              ▼
┌──────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐
│   Creation   │ │  Import  │ │  Export  │ │  Discovery   │
│   Module     │ │  Module  │ │  Module  │ │   Module     │
└──────┬───────┘ └────┬─────┘ └────┬─────┘ └──────┬───────┘
       │              │            │               │
       └──────────────┴────────────┴───────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    Core Services Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Keystore   │  │   Hardware   │  │    Batch     │     │
│  │   Manager    │  │   Device     │  │  Processor   │     │
│  │              │  │   Manager    │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    Alloy     │  │   Tracing    │  │     LRU      │     │
│  │  Providers   │  │  Structured  │  │    Cache     │     │
│  │   & Signers  │  │   Logging    │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Module Organization

```
src/wallet/account_manager/
├── mod.rs              (200 lines) - AccountManager trait and coordinator
├── creation.rs         (150 lines) - Account creation logic
├── import.rs           (200 lines) - Import from various sources
├── export.rs           (150 lines) - Export with authentication
├── discovery.rs        (100 lines) - HD account discovery
├── validation.rs       (100 lines) - Account validation
├── lifecycle.rs        (150 lines) - Account lifecycle management
└── tests/
    ├── creation_tests.rs
    ├── import_tests.rs
    ├── export_tests.rs
    └── property_tests.rs

src/error/
├── mod.rs              (100 lines) - Error module exports
└── account.rs          (200 lines) - Unified error types with context

src/performance/
├── mod.rs              (50 lines)  - Performance module exports
├── batch.rs            (150 lines) - Batch processor using Alloy
└── cache.rs            (100 lines) - LRU cache implementation

src/security/
├── session.rs          (100 lines) - Session management and auto-lock
└── export_auth.rs      (150 lines) - Export authentication

src/telemetry/
├── mod.rs              (50 lines)  - Telemetry module exports
└── account_events.rs   (150 lines) - Structured logging with tracing
```

## Components and Interfaces

### 1. AccountManager Trait

```rust
use alloy::primitives::Address;
use async_trait::async_trait;
use secrecy::SecretString;

#[async_trait]
pub trait AccountManager: Send + Sync {
    // Lifecycle operations
    async fn create_account(&mut self, config: AccountConfig) 
        -> Result<SecureAccount, AccountError>;
    
    async fn import_account(&mut self, source: ImportSource) 
        -> Result<SecureAccount, AccountError>;
    
    async fn remove_account(&mut self, address: Address) 
        -> Result<(), AccountError>;

    // Query operations
    async fn list_accounts(&self) 
        -> Result<Vec<SecureAccount>, AccountError>;
    
    async fn get_account(&self, address: Address) 
        -> Result<Option<SecureAccount>, AccountError>;
    
    async fn get_current_account(&self) 
        -> Result<Option<SecureAccount>, AccountError>;

    // State operations
    async fn set_current_account(&mut self, address: Address) 
        -> Result<(), AccountError>;
    
    async fn lock(&mut self) -> Result<(), AccountError>;
    
    async fn unlock(&mut self, password: &SecretString) 
        -> Result<(), AccountError>;
    
    fn is_locked(&self) -> bool;

    // Export operations with authentication
    async fn export_seed(
        &self, 
        address: Address, 
        password: &SecretString,
        auth_token: AuthToken
    ) -> Result<SecureExport, AccountError>;
    
    async fn export_private_key(
        &self, 
        address: Address, 
        password: &SecretString,
        auth_token: AuthToken
    ) -> Result<SecureExport, AccountError>;
}
```

### 2. Error Context with Correlation Tracking

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub account_id: Option<String>,
    pub network: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
    pub user_action: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            account_id: None,
            network: None,
            timestamp: Utc::now(),
            correlation_id: Uuid::new_v4().to_string(),
            user_action: None,
        }
    }
    
    pub fn with_account(mut self, account_id: String) -> Self {
        self.account_id = Some(account_id);
        self
    }
}
```

### 3. Batch Processor Using Alloy

```rust
use alloy::providers::Provider;
use alloy::primitives::{Address, U256};
use std::collections::HashMap;

pub struct BatchProcessor {
    provider: Arc<dyn Provider>,
    batch_size: usize,
    semaphore: Arc<Semaphore>,
}

impl BatchProcessor {
    pub async fn batch_balance_queries(
        &self,
        addresses: Vec<Address>,
    ) -> Result<HashMap<Address, U256>, BatchError> {
        // Use Alloy's provider to batch RPC calls
        let batch_request = addresses
            .iter()
            .map(|addr| self.provider.get_balance(*addr, BlockNumberOrTag::Latest))
            .collect::<Vec<_>>();
            
        let results = futures::future::join_all(batch_request).await;
        
        // Process results with proper error handling
        let mut balance_map = HashMap::new();
        for (addr, result) in addresses.into_iter().zip(results) {
            match result {
                Ok(balance) => { balance_map.insert(addr, balance); }
                Err(e) => tracing::warn!(
                    correlation_id = %Uuid::new_v4(),
                    address = %addr,
                    error = %e,
                    "Failed to get balance"
                ),
            }
        }
        
        Ok(balance_map)
    }
}
```

### 4. Hardware Device Manager

```rust
use alloy_signer_ledger::LedgerSigner;
use alloy_signer_trezor::TrezorSigner;

pub struct HardwareDeviceManager {
    connected_devices: Arc<RwLock<HashMap<DeviceId, HardwareDevice>>>,
}

impl HardwareDeviceManager {
    pub async fn scan_devices(&self) -> Result<Vec<HardwareDevice>, HardwareWalletError> {
        let mut devices = Vec::new();
        
        // Scan using Alloy signers
        if let Ok(ledger_devices) = LedgerSigner::list_devices().await {
            devices.extend(ledger_devices.into_iter().map(HardwareDevice::Ledger));
        }
        
        if let Ok(trezor_devices) = TrezorSigner::list_devices().await {
            devices.extend(trezor_devices.into_iter().map(HardwareDevice::Trezor));
        }
        
        // Update registry
        let mut connected = self.connected_devices.write().await;
        connected.clear();
        for device in &devices {
            connected.insert(device.id(), device.clone());
        }
        
        Ok(devices)
    }
}
```

## Data Models

### SecureAccount

```rust
use alloy::primitives::Address;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureAccount {
    pub id: String,
    pub address: Address,
    pub name: String,
    pub key_reference: KeyReference,
    pub created_at: DateTime<Utc>,
    pub is_hardware: bool,
    pub derivation_path: Option<String>,
    pub metadata: AccountMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadata {
    pub nickname: Option<String>,
    pub avatar: String,  // Deterministic avatar data URI
    pub tags: Vec<String>,
    pub last_used: Option<DateTime<Utc>>,
    pub transaction_count: u64,
}
```

### AccountConfig

```rust
#[derive(Debug, Clone)]
pub struct AccountConfig {
    pub name: String,
    pub account_type: AccountType,
    pub seed_strength: Option<SeedStrength>,
    pub derivation_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    SeedBased,
    PrivateKey,
    Hardware,
}

#[derive(Debug, Clone, Copy)]
pub enum SeedStrength {
    Words12,  // 128 bits
    Words15,  // 160 bits
    Words18,  // 192 bits
    Words21,  // 224 bits
    Words24,  // 256 bits
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Unified Interface Consistency

*For any* account operation (create, import, export, list, etc.), all operations should go through the AccountManager trait interface without bypassing it.

**Validates: Requirements 1.2**

### Property 2: Concurrent Operation Safety

*For any* set of concurrent account operations, the system should handle them safely without data races, corruption, or inconsistent state.

**Validates: Requirements 1.4**

### Property 3: Lock Memory Clearing

*For any* wallet state, when a lock operation is performed, all sensitive data (private keys, seed phrases, signers) should be cleared from memory and subsequent operations requiring authentication should fail.

**Validates: Requirements 2.3**

### Property 4: Unlock Restoration

*For any* locked wallet, when unlocked with correct credentials, all account operations should become available again and return the same results as before locking.

**Validates: Requirements 2.4**

### Property 5: Auto-Lock Timeout

*For any* configured timeout period, if no activity occurs for that duration, the wallet should automatically lock itself.

**Validates: Requirements 2.5**

### Property 6: Device Registry Consistency

*For any* set of connected hardware devices, the device registry should accurately reflect all available devices and maintain consistency across concurrent access.

**Validates: Requirements 3.3**

### Property 7: Concurrent Hardware Operations

*For any* set of concurrent hardware wallet operations across multiple devices, each operation should complete successfully without interfering with others.

**Validates: Requirements 3.4**

### Property 8: Error Context Completeness

*For any* error that occurs in the system, the error should include a complete ErrorContext with operation name, timestamp, correlation ID, and relevant account information.

**Validates: Requirements 4.1, 4.2**

### Property 9: Correlation Chain Maintenance

*For any* error that propagates through multiple system layers, the correlation ID should remain consistent throughout the error chain.

**Validates: Requirements 4.3**

### Property 10: Error Logging Correlation

*For any* error that is logged, the log entry should include the correlation ID from the error context for debugging.

**Validates: Requirements 4.5**

### Property 11: Batch RPC Efficiency

*For any* set of balance queries for N accounts, batch processing should result in fewer than N individual RPC calls to the provider.

**Validates: Requirements 6.1**

### Property 12: Batch Concurrency Limiting

*For any* batch operation, the number of concurrent requests should never exceed the configured concurrency limit.

**Validates: Requirements 6.2**

### Property 13: Batch Partial Failure Handling

*For any* batch operation where some requests fail, the successful requests should still return valid results and failures should be logged with correlation IDs.

**Validates: Requirements 6.3**

### Property 14: Batch Performance Improvement

*For any* set of repeated balance queries, batch processing should provide at least 50% performance improvement compared to individual sequential requests.

**Validates: Requirements 6.4**

### Property 15: Batch Retry with Backoff

*For any* batch operation that encounters network errors, the system should retry with exponential backoff up to a maximum number of attempts.

**Validates: Requirements 6.5**

### Property 16: Operation Correlation Logging

*For any* account operation, the system should create a correlation ID at the start and include it in all log entries for that operation.

**Validates: Requirements 7.1**

### Property 17: Cross-Component Correlation

*For any* operation that spans multiple components, the correlation ID should be propagated and maintained across all component boundaries.

**Validates: Requirements 7.3**

### Property 18: Complete Operation Logging

*For any* account operation, the system should log operation start, completion (or error), with appropriate log levels (info for success, error for failures).

**Validates: Requirements 7.4**

### Property 19: Privacy Mode Log Sanitization

*For any* log entry when privacy mode is enabled, the entry should not contain private keys, seed phrases, passwords, or other sensitive information.

**Validates: Requirements 7.5**

### Property 20: Seed Phrase Import Determinism

*For any* valid BIP39 seed phrase, importing it multiple times should always produce the same account address for the same derivation path.

**Validates: Requirements 8.2**

### Property 21: Migration Format Validation

*For any* migration request, the system should validate the source format before attempting conversion and reject invalid formats with specific error messages.

**Validates: Requirements 8.3**

### Property 22: Migration Metadata Preservation

*For any* account migration, all available metadata (name, creation date, tags) should be preserved in the migrated account.

**Validates: Requirements 8.4**

### Property 23: Migration Error Specificity

*For any* failed migration, the error message should specifically indicate what went wrong (invalid format, decryption failure, etc.) rather than generic errors.

**Validates: Requirements 8.5**

### Property 24: LRU Cache Correctness

*For any* sequence of account data accesses, the LRU cache should correctly store frequently accessed data and evict least recently used entries when capacity is reached.

**Validates: Requirements 9.1**

### Property 25: Cache Staleness Detection

*For any* cached account data, if the underlying data changes, the cache should detect staleness and refresh automatically.

**Validates: Requirements 9.2**

### Property 26: Cache Performance Improvement

*For any* repeated account data access, cached access should be at least 50% faster than uncached access.

**Validates: Requirements 9.3**

### Property 27: LRU Eviction Under Pressure

*For any* cache at capacity, when new entries are added, the least recently used entries should be evicted first.

**Validates: Requirements 9.4**

### Property 28: Async Non-Blocking Operations

*For any* expensive account operation (key derivation, encryption), the operation should not block the UI thread and should complete asynchronously.

**Validates: Requirements 9.5**

### Property 29: Telemetry Anonymity

*For any* telemetry data collected, the data should contain no private keys, seed phrases, full addresses, or personally identifiable information.

**Validates: Requirements 10.1, 10.4**

### Property 30: Backup Encryption

*For any* account backup, the backup data should be encrypted and decryption should require the correct password.

**Validates: Requirements 11.1**

### Property 31: Shamir Secret Sharing Round-Trip

*For any* secret split using Shamir's Secret Sharing with threshold T and total shares N, any T shares should be able to reconstruct the original secret exactly.

**Validates: Requirements 11.2**

### Property 32: Backup Integrity Verification

*For any* backup restoration, the system should verify backup integrity (checksums, MACs) before applying the restoration.

**Validates: Requirements 11.4**

### Property 33: Nickname Uniqueness

*For any* account nickname, the system should enforce uniqueness across all accounts and reject duplicate nicknames.

**Validates: Requirements 12.1**

### Property 34: Avatar Determinism

*For any* account address, the generated avatar should be deterministic (same address always produces same avatar).

**Validates: Requirements 12.2**

### Property 35: Tag Management Consistency

*For any* account, tags can be added and removed, and querying accounts by tag should return exactly those accounts with that tag.

**Validates: Requirements 12.3**

## Error Handling

All errors use the unified `AccountError` type with structured context:

```rust
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account not found: {address}")]
    AccountNotFound { 
        address: String,
        context: Option<ErrorContext>,
    },
    
    #[error("Import failed: {message}")]
    ImportFailed { 
        message: String,
        context: Option<ErrorContext>,
    },
    
    #[error("Keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    
    #[error("Hardware wallet error: {0}")]
    HardwareWallet(#[from] HardwareWalletError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}
```

**Error Handling Strategy:**
- All errors include correlation IDs for debugging
- Errors are logged with appropriate levels (warn/error)
- User-facing errors provide actionable messages
- Internal errors include technical details for debugging

## Testing Strategy

### Unit Tests
- Test individual components in isolation
- Mock external dependencies (Alloy providers, hardware devices)
- Focus on specific examples and edge cases
- Target: 90% code coverage

### Property-Based Tests
- Use `proptest` for cryptographic operations
- Test universal properties across many generated inputs
- Minimum 100 iterations per property test
- Each test references design document property

**Property Test Examples:**

```rust
use proptest::prelude::*;

proptest! {
    // Property 19: Seed Phrase Import Determinism
    #[test]
    fn test_seed_import_deterministic(
        seed_phrase in "[a-z ]{120,240}",
        derivation_path in "m/44'/60'/0'/0/[0-9]+"
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let addr1 = import_from_seed(&seed_phrase, &derivation_path).await.unwrap();
            let addr2 = import_from_seed(&seed_phrase, &derivation_path).await.unwrap();
            prop_assert_eq!(addr1, addr2);
        });
    }
    
    // Property 29: Telemetry Anonymity
    #[test]
    fn test_telemetry_no_sensitive_data(
        account_data in any::<SecureAccount>()
    ) {
        let telemetry_event = create_telemetry_event(&account_data);
        let serialized = serde_json::to_string(&telemetry_event).unwrap();
        
        // Should not contain private keys or full addresses
        prop_assert!(!serialized.contains("private_key"));
        prop_assert!(!serialized.contains("seed_phrase"));
        prop_assert!(!serialized.contains(&format!("{:#x}", account_data.address)));
    }
    
    // Property 31: Shamir Secret Sharing Round-Trip
    #[test]
    fn test_shamir_round_trip(
        secret in prop::collection::vec(any::<u8>(), 32),
        threshold in 2u8..=5,
        total_shares in 3u8..=10
    ) {
        prop_assume!(threshold <= total_shares);
        
        let shares = split_secret(&secret, threshold, total_shares).unwrap();
        let recovered = recover_secret(&shares[..threshold as usize]).unwrap();
        
        prop_assert_eq!(secret, recovered);
    }
}
```

### Integration Tests
- Test complete workflows end-to-end
- Use real Alloy providers (testnet)
- Test hardware wallet integration (if available)
- Verify correlation tracking across components

### Performance Tests
- Benchmark batch vs individual operations
- Verify cache performance improvements
- Test async operation non-blocking behavior
- Measure memory usage and cleanup

## Security Considerations

1. **Memory Safety**: Use `zeroize` for sensitive data, clear on lock
2. **Encryption**: AES-256-GCM for all encrypted data
3. **Key Derivation**: PBKDF2 with 262,144 iterations (MetaMask compatible)
4. **Hardware Wallets**: Always verify addresses on device
5. **Export Authentication**: Require password + auth token for exports
6. **Rate Limiting**: Limit export operations to prevent brute force
7. **Audit Logging**: Log all sensitive operations with correlation IDs

## Performance Targets

- **Batch Operations**: 50%+ improvement over individual requests
- **Cache Hit Rate**: 80%+ for repeated account access
- **Lock/Unlock**: < 100ms for lock, < 500ms for unlock
- **Account Creation**: < 2s for seed-based, < 500ms for private key
- **UI Responsiveness**: All operations async, no blocking > 16ms

## Dependencies

**Core:**
- `alloy` (1.1) - Blockchain operations, providers, signers
- `tokio` (1.0) - Async runtime with multi-threaded executor
- `secrecy` (0.8) - Secret string handling with zeroize
- `thiserror` (1.0) - Error handling with context
- `async-trait` (0.1) - Async trait support

**Cryptography:**
- `bip39` (2.0) - BIP39 mnemonic support
- `bip32` (0.5) - BIP32 HD wallet derivation
- `aes-gcm` (0.10) - AES-256-GCM authenticated encryption
- `pbkdf2` (0.12) - PBKDF2 key derivation (262,144 iterations)
- `sharks` (0.5) - Shamir's Secret Sharing implementation
- `zeroize` (1.7) - Secure memory clearing

**Testing:**
- `proptest` (1.0) - Property-based testing for crypto operations
- `quickcheck` (1.0) - Alternative property-based testing framework
- `tempfile` (3.0) - Temporary directories for tests
- `tokio-test` (0.4) - Async test utilities

**Performance:**
- `lru` (0.12) - LRU cache with O(1) operations
- `futures` (0.3) - Async utilities and combinators
- `tokio::sync::Semaphore` - Concurrency limiting for batch operations

**Observability:**
- `tracing` (0.1) - Structured logging with spans and events
- `tracing-subscriber` (0.3) - Log formatting and filtering
- `uuid` (1.0) - Correlation ID generation (v4 UUIDs)
- `chrono` (0.4) - Timestamp handling

**Serialization:**
- `serde` (1.0) - Serialization framework
- `serde_json` (1.0) - JSON serialization for metadata

**Optional:**
- `alloy-signer-ledger` (1.1) - Ledger hardware wallet support
- `alloy-signer-trezor` (1.1) - Trezor hardware wallet support

## Implementation Best Practices

### Code Organization
- **Module Size**: Keep modules under 200 lines for maintainability
- **Single Responsibility**: Each module should have one clear purpose
- **Alloy-First**: Always prefer Alloy libraries over custom implementations
- **No Bloat**: Only add features that provide real, measurable value

### Error Handling
- **Always use correlation IDs**: Every error should include a UUID for tracking
- **Structured context**: Include operation name, timestamp, account ID when relevant
- **User-friendly messages**: Errors should be actionable for end users
- **Technical details**: Include stack traces and context for developers

### Testing Strategy
- **Property-based tests**: Use `proptest` for all cryptographic operations
- **100 iterations minimum**: Each property test should run at least 100 times
- **Unit tests for examples**: Test specific edge cases and error conditions
- **Integration tests**: Test complete workflows end-to-end
- **Mock external dependencies**: Use test doubles for Alloy providers and hardware devices

### Performance Guidelines
- **Batch operations**: Always use Alloy's batch capabilities for multiple requests
- **Async patterns**: Never block the UI thread (< 16ms per operation)
- **LRU caching**: Cache expensive operations with automatic eviction
- **Concurrency limiting**: Use semaphores to prevent overwhelming RPC providers
- **Measure everything**: Benchmark before and after optimizations

### Security Checklist
- ✅ Use `zeroize` for all sensitive data in memory
- ✅ Clear memory on lock operations
- ✅ Require authentication for export operations
- ✅ Rate limit sensitive operations
- ✅ Audit log all security-critical operations
- ✅ Verify hardware wallet addresses on device
- ✅ Use AES-256-GCM for all encryption
- ✅ PBKDF2 with 262,144 iterations for key derivation

### Logging Guidelines
- **Use `tracing` crate**: Structured logging with spans and events
- **Include correlation IDs**: Every log entry should have a correlation ID
- **Appropriate log levels**: info for success, warn for recoverable errors, error for failures
- **Privacy mode**: Filter sensitive data (keys, seeds, passwords) from logs
- **Span instrumentation**: Use `.instrument()` for async operations

### Documentation Standards
- **Rustdoc comments**: All public APIs must have documentation
- **Examples**: Include code examples in documentation
- **Error scenarios**: Document all possible error conditions
- **Performance characteristics**: Document time/space complexity
- **Security considerations**: Document security implications


