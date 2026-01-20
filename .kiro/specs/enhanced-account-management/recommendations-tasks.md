# Enhanced Account Management - Recommendations Implementation Tasks

## Overview

This task list implements the expert recommendations for enhancing Vaughan's account management system with deeper Alloy integration, MetaMask compatibility improvements, and production-ready features.

**Implementation Guidelines:**
- **Alloy-First**: Always use Alloy libraries for blockchain operations
- **MetaMask Compatibility**: Follow MetaMask patterns where Alloy doesn't provide functionality
- **Module Size**: Keep each module under 200 lines
- **Correlation Tracking**: Include correlation IDs in all operations and errors
- **Property-Based Testing**: Use `proptest` with minimum 100 iterations
- **Structured Logging**: Use `tracing` crate with appropriate log levels
- **Security**: Use `zeroize` for sensitive data, implement rate limiting
- **Performance**: Batch operations, async patterns, caching
- **No Bloat**: Only implement features that provide measurable value

## Phase 1: Alloy Signer Integration (Priority: High)

### Task 1: Implement Alloy Signer Trait Integration

- [x] 1.1 Add Alloy signer trait support to AccountManager
  - Create `src/wallet/account_manager/signer_integration.rs`
  - Implement `get_signer()` method that returns `Box<dyn Signer>`
  - Support `PrivateKeySigner` for software accounts
  - Support `LedgerSigner` for Ledger hardware wallets
  - Support `TrezorSigner` for Trezor hardware wallets
  - Add correlation tracking for all signer operations
  - _Recommendation: 1 - Alloy Signer Integration Enhancement_
  - ✅ Implemented `VaughanSigner` implementing `Signer` and `SignerSync`
  - ✅ Created `SignerManager` with correlation tracking
  - ✅ Added hardware wallet placeholder wrappers

- [ ] 1.2 Create EthereumWallet wrapper
  - Implement `to_ethereum_wallet()` method
  - Support composing signers with Alloy providers
  - Add examples in documentation
  - _Recommendation: 1 - Alloy Signer Integration Enhancement_

- [ ] 1.3 Write unit tests for signer integration
  - Test signer retrieval for each account type
  - Test EthereumWallet composition
  - Test signing operations through Alloy's native methods
  - _Recommendation: 1 - Alloy Signer Integration Enhancement_

- [ ] 1.4 Write integration tests with real Alloy providers
  - Test signing transactions with PrivateKeySigner
  - Test signing with hardware wallet signers (if available)
  - Test provider composition
  - _Recommendation: 1 - Alloy Signer Integration Enhancement_

## Phase 2: MetaMask Keystore Compatibility (Priority: High)

### Task 2: Implement MetaMask V3 Keystore Support

- [ ] 2.1 Add eth-keystore dependency
  - Add `eth-keystore = "0.5"` to Cargo.toml
  - Document that this is MetaMask-inspired (not Alloy)
  - _Recommendation: 2 - MetaMask-Compatible Keystore Format_

- [ ] 2.2 Implement V3 keystore import
  - Create `import_metamask_v3_keystore()` in `src/wallet/account_manager/import.rs`
  - Support standard V3 JSON format (EIP-2335)
  - Validate keystore structure before decryption
  - Add correlation tracking
  - _Recommendation: 2 - MetaMask-Compatible Keystore Format_

- [ ] 2.3 Implement V3 keystore export
  - Create `export_to_v3_keystore()` in `src/wallet/account_manager/export.rs`
  - Use PBKDF2 with 262,144 iterations (MetaMask compatible)
  - Generate standard V3 JSON format
  - Require authentication token
  - _Recommendation: 2 - MetaMask-Compatible Keystore Format_

- [ ] 2.4 Write property tests for V3 keystore round-trip
  - Test import → export → import produces same account
  - Test with various password strengths
  - Test with different PBKDF2 parameters
  - Minimum 100 iterations
  - _Recommendation: 2 - MetaMask-Compatible Keystore Format_

- [ ] 2.5 Write unit tests for V3 keystore validation
  - Test invalid JSON rejection
  - Test corrupted keystore detection
  - Test wrong password handling
  - _Recommendation: 2 - MetaMask-Compatible Keystore Format_

## Phase 3: Hardware Wallet Derivation Standards (Priority: Medium)

### Task 3: Implement Comprehensive Derivation Path Support

- [ ] 3.1 Create derivation standard enum
  - Create `src/wallet/account_manager/derivation.rs`
  - Define `DerivationStandard` enum (BIP-44, Ledger Live, Ledger Legacy, Custom)
  - Implement path generation for each standard
  - Add validation for custom paths
  - _Recommendation: 3 - Hardware Wallet Derivation Path Standards_

- [ ] 3.2 Update hardware device manager
  - Modify `src/wallet/hardware/device_manager.rs`
  - Support all derivation standards
  - Add standard detection based on device type
  - Add correlation tracking
  - _Recommendation: 3 - Hardware Wallet Derivation Path Standards_

- [ ] 3.3 Add derivation path configuration to AccountConfig
  - Update `AccountConfig` to include `DerivationStandard`
  - Provide sensible defaults per device type
  - Document each standard in rustdoc
  - _Recommendation: 3 - Hardware Wallet Derivation Path Standards_

- [ ] 3.4 Write unit tests for derivation paths
  - Test path generation for each standard
  - Test custom path validation
  - Test device-specific defaults
  - _Recommendation: 3 - Hardware Wallet Derivation Path Standards_

- [ ] 3.5 Write integration tests with hardware wallets
  - Test address derivation with each standard (if devices available)
  - Test cross-compatibility (same seed, different standards)
  - _Recommendation: 3 - Hardware Wallet Derivation Path Standards_

## Phase 4: Advanced Batch Operations (Priority: Medium)

### Task 4: Implement Multicall3 Batching

- [ ] 4.1 Add Multicall3 contract integration
  - Create `src/performance/multicall.rs`
  - Define Multicall3 contract ABI using `sol!` macro
  - Implement multicall builder pattern
  - Add support for multiple chains (different Multicall3 addresses)
  - _Recommendation: 4 - Batch Operations with Alloy's Multicall_

- [ ] 4.2 Implement batch balance queries with Multicall3
  - Create `batch_balance_queries_multicall()` method
  - Support both native ETH and ERC20 token balances
  - Handle partial failures gracefully
  - Add correlation tracking
  - _Recommendation: 4 - Batch Operations with Alloy's Multicall_

- [ ] 4.3 Add fallback to parallel HTTP requests
  - Detect when Multicall3 is not available
  - Fall back to existing `futures::join_all` approach
  - Log which method is being used
  - _Recommendation: 4 - Batch Operations with Alloy's Multicall_

- [ ] 4.4 Write unit tests for Multicall3
  - Test multicall builder
  - Test balance query batching
  - Test fallback mechanism
  - _Recommendation: 4 - Batch Operations with Alloy's Multicall_

- [ ] 4.5 Write property tests for batch efficiency
  - Test that Multicall3 uses exactly 1 RPC call for N queries
  - Test that results match individual queries
  - Minimum 100 iterations
  - _Recommendation: 4 - Batch Operations with Alloy's Multicall_

- [ ] 4.6 Write performance benchmarks
  - Benchmark Multicall3 vs parallel HTTP
  - Measure gas savings (on-chain efficiency)
  - Verify >50% performance improvement
  - _Recommendation: 4 - Batch Operations with Alloy's Multicall_

## Phase 5: EIP-1193 Provider Interface (Priority: High)

### Task 5: Implement EIP-1193 Provider

- [ ] 5.1 Create EIP-1193 provider trait
  - Create `src/wallet/provider/eip1193.rs`
  - Define `Eip1193Provider` trait with `request()` method
  - Support all standard Ethereum RPC methods
  - Add correlation tracking for all requests
  - _Recommendation: 5 - EIP-1193 Provider Interface_

- [ ] 5.2 Implement provider for Vaughan wallet
  - Implement `Eip1193Provider` for `Vaughan` struct
  - Handle `eth_requestAccounts` with user approval
  - Handle `eth_accounts`, `eth_chainId`, `eth_sendTransaction`
  - Handle `personal_sign`, `eth_signTypedData_v4`
  - _Recommendation: 5 - EIP-1193 Provider Interface_

- [ ] 5.3 Add permission management
  - Create `src/wallet/provider/permissions.rs`
  - Implement permission request/approval flow
  - Store approved dApp origins
  - Add permission revocation
  - _Recommendation: 5 - EIP-1193 Provider Interface_

- [ ] 5.4 Implement EIP-1193 events
  - Add event emitter for `accountsChanged`
  - Add event emitter for `chainChanged`
  - Add event emitter for `connect` / `disconnect`
  - Use `tokio::sync::broadcast` for event distribution
  - _Recommendation: 5 - EIP-1193 Provider Interface_

- [ ] 5.5 Write unit tests for EIP-1193 provider
  - Test each RPC method
  - Test permission flows
  - Test event emission
  - _Recommendation: 5 - EIP-1193 Provider Interface_

- [ ] 5.6 Write integration tests with mock dApp
  - Test complete dApp connection flow
  - Test transaction signing from dApp
  - Test account switching
  - _Recommendation: 5 - EIP-1193 Provider Interface_

## Phase 6: Transaction Simulation (Priority: High)

### Task 6: Implement Transaction Simulation

- [ ] 6.1 Create transaction simulator
  - Create `src/wallet/transaction/simulator.rs`
  - Implement `simulate_transaction()` using Alloy's `call()`
  - Decode revert reasons from failed simulations
  - Add gas estimation from simulation results
  - Add correlation tracking
  - _Recommendation: 6 - Transaction Simulation Before Signing_

- [ ] 6.2 Add simulation result types
  - Define `SimulationResult` struct
  - Include success/failure status
  - Include gas used estimate
  - Include return data and decoded revert reasons
  - Include state changes (optional, for advanced users)
  - _Recommendation: 6 - Transaction Simulation Before Signing_

- [ ] 6.3 Integrate simulation into signing flow
  - Modify `sign_transaction()` to simulate first
  - Warn user if simulation fails
  - Allow user to proceed anyway (with confirmation)
  - Log simulation results with correlation ID
  - _Recommendation: 6 - Transaction Simulation Before Signing_

- [ ] 6.4 Write unit tests for simulator
  - Test successful simulation
  - Test failed simulation with revert reason
  - Test gas estimation accuracy
  - _Recommendation: 6 - Transaction Simulation Before Signing_

- [ ] 6.5 Write integration tests with Anvil
  - Deploy test contracts to Anvil
  - Test simulation of successful transactions
  - Test simulation of reverting transactions
  - Test gas estimation accuracy
  - _Recommendation: 6 - Transaction Simulation Before Signing_

## Phase 7: EIP-712 Typed Data Signing (Priority: Medium)

### Task 7: Implement EIP-712 Support

- [ ] 7.1 Add EIP-712 signing to AccountManager
  - Create `src/wallet/account_manager/eip712.rs`
  - Implement `sign_typed_data()` method
  - Use Alloy's `Eip712Domain` and `SolStruct`
  - Support all EIP-712 types
  - Add correlation tracking
  - _Recommendation: 7 - EIP-712 Typed Data Signing_

- [ ] 7.2 Add EIP-712 domain builder
  - Create helper for constructing `Eip712Domain`
  - Support all domain fields (name, version, chainId, verifyingContract, salt)
  - Validate domain parameters
  - _Recommendation: 7 - EIP-712 Typed Data Signing_

- [ ] 7.3 Add common EIP-712 message types
  - Define common types (Permit, Vote, etc.) using `sol!` macro
  - Provide examples in documentation
  - _Recommendation: 7 - EIP-712 Typed Data Signing_

- [ ] 7.4 Write unit tests for EIP-712 signing
  - Test signing with various message types
  - Test domain validation
  - Test signature verification
  - _Recommendation: 7 - EIP-712 Typed Data Signing_

- [ ] 7.5 Write property tests for EIP-712
  - Test that signatures are deterministic
  - Test that signatures verify correctly
  - Minimum 100 iterations
  - _Recommendation: 7 - EIP-712 Typed Data Signing_

## Phase 8: HD Wallet Account Discovery (Priority: Medium)

### Task 8: Implement BIP-44 Account Discovery

- [ ] 8.1 Create account discovery module
  - Create `src/wallet/account_manager/discovery.rs`
  - Implement `discover_accounts()` method
  - Use gap limit of 20 (MetaMask standard)
  - Support custom gap limits
  - Add correlation tracking
  - _Recommendation: 8 - Account Discovery for HD Wallets_

- [ ] 8.2 Implement activity checking
  - Create `check_account_activity()` method
  - Check for non-zero balance
  - Check for transaction history (nonce > 0)
  - Use batch operations for efficiency
  - _Recommendation: 8 - Account Discovery for HD Wallets_

- [ ] 8.3 Add discovered account metadata
  - Define `DiscoveredAccount` struct
  - Include address, derivation path, index
  - Include activity summary (balance, tx count)
  - _Recommendation: 8 - Account Discovery for HD Wallets_

- [ ] 8.4 Write unit tests for discovery
  - Test discovery with mock accounts
  - Test gap limit enforcement
  - Test activity detection
  - _Recommendation: 8 - Account Discovery for HD Wallets_

- [ ] 8.5 Write integration tests with Anvil
  - Create accounts with activity on Anvil
  - Test discovery finds all active accounts
  - Test discovery stops at gap limit
  - _Recommendation: 8 - Account Discovery for HD Wallets_

- [ ] 8.6 Write property tests for discovery
  - Test that discovery is deterministic
  - Test that all active accounts are found
  - Minimum 100 iterations
  - _Recommendation: 8 - Account Discovery for HD Wallets_

## Phase 9: EIP-1559 Fee Estimation (Priority: High)

### Task 9: Implement EIP-1559 Fee Estimation

- [ ] 9.1 Create fee estimator module
  - Create `src/wallet/transaction/fees.rs`
  - Implement `estimate_fees_eip1559()` method
  - Use Alloy's `estimate_eip1559_fees()`
  - Add correlation tracking
  - _Recommendation: 9 - Gas Estimation with EIP-1559 Support_

- [ ] 9.2 Add fee priority levels
  - Define `FeePriority` enum (Slow, Medium, Fast, Custom)
  - Implement priority multipliers (MetaMask-style)
  - Calculate fees for each priority level
  - _Recommendation: 9 - Gas Estimation with EIP-1559 Support_

- [ ] 9.3 Add fee history analysis
  - Implement `analyze_fee_history()` method
  - Use `eth_feeHistory` RPC call
  - Calculate percentile-based recommendations
  - _Recommendation: 9 - Gas Estimation with EIP-1559 Support_

- [ ] 9.4 Add legacy gas price support
  - Detect when EIP-1559 is not supported
  - Fall back to `eth_gasPrice`
  - Log which method is being used
  - _Recommendation: 9 - Gas Estimation with EIP-1559 Support_

- [ ] 9.5 Write unit tests for fee estimation
  - Test EIP-1559 fee calculation
  - Test priority level multipliers
  - Test legacy fallback
  - _Recommendation: 9 - Gas Estimation with EIP-1559 Support_

- [ ] 9.6 Write integration tests with Anvil
  - Test fee estimation on EIP-1559 chain
  - Test fee estimation on legacy chain
  - Verify fees are reasonable
  - _Recommendation: 9 - Gas Estimation with EIP-1559 Support_

## Phase 10: Rate Limiting for Security (Priority: High)

### Task 10: Implement Rate Limiting

- [ ] 10.1 Create rate limiter module
  - Create `src/security/rate_limiter.rs`
  - Implement token bucket algorithm
  - Support per-operation rate limits
  - Add correlation tracking
  - _Recommendation: 10 - Security: Rate Limiting for Sensitive Operations_

- [ ] 10.2 Add rate limiting to export operations
  - Apply rate limiter to `export_seed()`
  - Apply rate limiter to `export_private_key()`
  - Configure limits (e.g., 3 exports per hour)
  - Return clear error with retry time
  - _Recommendation: 10 - Security: Rate Limiting for Sensitive Operations_

- [ ] 10.3 Add rate limiting to authentication
  - Apply rate limiter to password verification
  - Apply rate limiter to unlock operations
  - Configure limits (e.g., 5 attempts per minute)
  - Implement exponential backoff
  - _Recommendation: 10 - Security: Rate Limiting for Sensitive Operations_

- [ ] 10.4 Add rate limit persistence
  - Store rate limit state across restarts
  - Use secure storage for rate limit data
  - Clear rate limits on successful operations
  - _Recommendation: 10 - Security: Rate Limiting for Sensitive Operations_

- [ ] 10.5 Write unit tests for rate limiter
  - Test token bucket refill
  - Test rate limit enforcement
  - Test retry time calculation
  - _Recommendation: 10 - Security: Rate Limiting for Sensitive Operations_

- [ ] 10.6 Write property tests for rate limiter
  - Test that rate limits are never exceeded
  - Test that tokens refill correctly
  - Minimum 100 iterations
  - _Recommendation: 10 - Security: Rate Limiting for Sensitive Operations_

## Phase 11: OpenTelemetry Observability (Priority: Low)

### Task 11: Implement OpenTelemetry Integration

- [ ] 11.1 Add OpenTelemetry dependencies
  - Add `opentelemetry = "0.21"` to Cargo.toml
  - Add `opentelemetry-otlp = "0.14"` to Cargo.toml
  - Add `tracing-opentelemetry = "0.22"` to Cargo.toml
  - Make OpenTelemetry optional feature
  - _Recommendation: 11 - Observability: OpenTelemetry Integration_

- [ ] 11.2 Create telemetry initialization
  - Create `src/telemetry/opentelemetry.rs`
  - Implement `init_telemetry()` function
  - Configure OTLP exporter
  - Support environment variable configuration
  - _Recommendation: 11 - Observability: OpenTelemetry Integration_

- [ ] 11.3 Add span instrumentation
  - Add `#[tracing::instrument]` to key methods
  - Include correlation IDs in spans
  - Add custom span attributes
  - _Recommendation: 11 - Observability: OpenTelemetry Integration_

- [ ] 11.4 Add metrics collection
  - Define key metrics (operation count, duration, errors)
  - Implement metric exporters
  - Add metric dashboards documentation
  - _Recommendation: 11 - Observability: OpenTelemetry Integration_

- [ ] 11.5 Write integration tests for telemetry
  - Test span creation and propagation
  - Test metric collection
  - Test OTLP export (with mock collector)
  - _Recommendation: 11 - Observability: OpenTelemetry Integration_

## Phase 12: Anvil Integration Testing (Priority: Medium)

### Task 12: Implement Anvil-Based Integration Tests

- [ ] 12.1 Add Anvil test utilities
  - Create `tests/utils/anvil.rs`
  - Implement Anvil node spawning
  - Implement test account funding
  - Implement contract deployment helpers
  - _Recommendation: 12 - Testing: Anvil Integration for Integration Tests_

- [ ] 12.2 Migrate existing integration tests to Anvil
  - Update `tests/account_manager_integration.rs`
  - Replace mocks with real Anvil interactions
  - Test with real blockchain state
  - _Recommendation: 12 - Testing: Anvil Integration for Integration Tests_

- [ ] 12.3 Add end-to-end transaction tests
  - Test complete transaction flow (create, sign, send, confirm)
  - Test transaction simulation
  - Test gas estimation accuracy
  - Test EIP-1559 transactions
  - _Recommendation: 12 - Testing: Anvil Integration for Integration Tests_

- [ ] 12.4 Add contract interaction tests
  - Deploy ERC20 token contract
  - Test token transfers
  - Test EIP-712 permit signatures
  - Test Multicall3 batching
  - _Recommendation: 12 - Testing: Anvil Integration for Integration Tests_

- [ ] 12.5 Add hardware wallet simulation tests
  - Mock hardware wallet responses
  - Test derivation path standards
  - Test signing with hardware wallets
  - _Recommendation: 12 - Testing: Anvil Integration for Integration Tests_

## Phase 13: Documentation and Examples (Priority: Medium)

### Task 13: Create Comprehensive Documentation

- [ ] 13.1 Update AccountManager documentation
  - Document all new methods with examples
  - Add usage examples for each feature
  - Document Alloy integration patterns
  - Document MetaMask compatibility
  - _All Recommendations_

- [ ] 13.2 Create migration guide
  - Document migration from old API to new API
  - Provide code examples for common patterns
  - Document breaking changes
  - _All Recommendations_

- [ ] 13.3 Create dApp integration guide
  - Document EIP-1193 provider usage
  - Provide example dApp integration
  - Document permission management
  - _Recommendation: 5 - EIP-1193 Provider Interface_

- [ ] 13.4 Create hardware wallet guide
  - Document derivation path standards
  - Provide setup instructions for each device
  - Document troubleshooting steps
  - _Recommendation: 3 - Hardware Wallet Derivation Path Standards_

- [ ] 13.5 Create security best practices guide
  - Document rate limiting configuration
  - Document secure key management
  - Document audit logging
  - _Recommendation: 10 - Security: Rate Limiting_

## Phase 14: Performance Optimization (Priority: Low)

### Task 14: Optimize Performance

- [ ] 14.1 Profile critical paths
  - Profile account creation
  - Profile transaction signing
  - Profile batch operations
  - Identify bottlenecks
  - _All Recommendations_

- [ ] 14.2 Optimize hot paths
  - Optimize signer retrieval
  - Optimize cache access patterns
  - Optimize async task spawning
  - _All Recommendations_

- [ ] 14.3 Add performance benchmarks
  - Benchmark all new features
  - Compare with baseline performance
  - Verify performance targets met
  - _All Recommendations_

- [ ] 14.4 Optimize memory usage
  - Profile memory allocations
  - Reduce unnecessary clones
  - Optimize cache sizes
  - _All Recommendations_

## Phase 15: Final Integration and Testing (Priority: High)

### Task 15: Final Integration

- [ ] 15.1 Integration checkpoint
  - Ensure all modules integrate correctly
  - Test complete workflows end-to-end
  - Verify no regressions in existing functionality
  - _All Recommendations_

- [ ] 15.2 Run full test suite
  - Run all unit tests
  - Run all property tests
  - Run all integration tests
  - Run all performance tests
  - Verify 100% pass rate
  - _All Recommendations_

- [ ] 15.3 Security audit
  - Review all sensitive operations
  - Verify rate limiting is effective
  - Verify memory clearing on lock
  - Verify no sensitive data in logs
  - _All Recommendations_

- [ ] 15.4 Performance validation
  - Verify all performance targets met
  - Verify no performance regressions
  - Verify cache hit rates
  - Verify batch operation efficiency
  - _All Recommendations_

- [ ] 15.5 Documentation review
  - Verify all public APIs documented
  - Verify all examples work
  - Verify migration guide is complete
  - _All Recommendations_

## Success Criteria

**Code Quality:**
- All modules under 200 lines
- 90%+ test coverage including property-based tests
- Zero compiler warnings
- All rustdoc comments complete

**Security:**
- Rate limiting on all sensitive operations
- Transaction simulation prevents failed transactions
- All sensitive data uses `zeroize`
- Correlation tracking on all operations

**Performance:**
- Multicall3 batching: 1 RPC call for N queries
- Cache hit rate: 80%+ for repeated operations
- Transaction simulation: < 500ms
- Account discovery: < 5s for 20 accounts

**Testing:**
- All property tests run 100+ iterations
- All tests pass consistently
- Integration tests use Anvil for realistic testing
- Performance benchmarks meet targets

**Compatibility:**
- Full Alloy signer integration
- MetaMask V3 keystore import/export
- EIP-1193 provider interface
- EIP-712 typed data signing
- EIP-1559 fee estimation

**Observability:**
- All operations have correlation IDs
- Structured logging with `tracing`
- OpenTelemetry integration (optional)
- Clear error messages for users

## Notes

- All tasks build on the existing enhanced account management implementation
- Tasks are prioritized: High (core functionality), Medium (important features), Low (nice-to-have)
- Each task references the specific recommendation it implements
- Hardware wallet tests may be skipped if devices unavailable
- OpenTelemetry is optional and can be enabled via feature flag
- All new code follows Alloy-first principle, falling back to MetaMask patterns only where necessary
