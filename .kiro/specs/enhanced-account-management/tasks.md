# Implementation Plan: Enhanced Account Management

## Overview

This implementation plan breaks down the enhanced account management system into discrete, testable tasks. Each task builds on previous work and includes specific requirements references for traceability.

**Implementation Guidelines:**
- **Alloy-First**: Always use Alloy libraries for blockchain operations
- **Module Size**: Keep each module under 200 lines
- **Correlation Tracking**: Include correlation IDs in all operations and errors
- **Property-Based Testing**: Use `proptest` with minimum 100 iterations
- **Structured Logging**: Use `tracing` crate with appropriate log levels
- **Security**: Use `zeroize` for sensitive data, clear memory on lock
- **Performance**: Batch operations, async patterns, LRU caching
- **No Bloat**: Only implement features that provide measurable value

## Tasks

- [x] 1. Set up unified error handling with correlation tracking
  - Create `src/error/account.rs` with `AccountError`, `ErrorContext`, and correlation ID support
  - Implement `ErrorContext::new()` with automatic UUID generation using `uuid::Uuid::new_v4()`
  - Add builder methods for context enrichment (`with_account`, `with_user_action`, `with_network`)
  - Include timestamp using `chrono::Utc::now()`
  - Implement `Display` and `Debug` traits for user-friendly error messages
  - _Requirements: 4.1, 4.2_

- [x] 1.1 Write property tests for error context
  - **Property 8: Error Context Completeness**
  - **Validates: Requirements 4.1, 4.2**
  - Test that all errors include complete context with correlation IDs

- [x] 2. Implement AccountManager trait and core structure
  - Create `src/wallet/account_manager/mod.rs` with `AccountManager` trait
  - Define all trait methods (create, import, export, list, lock, unlock)
  - Add `AccountConfig`, `ImportSource`, and `SecureExport` types
  - _Requirements: 1.1, 1.2_

- [x] 2.1 Write unit tests for AccountManager interface
  - Test that all operations go through unified interface
  - Test concurrent operation safety with tokio spawn
  - _Requirements: 1.2, 1.4_

- [x] 2.2 Write property test for concurrent operations
  - **Property 2: Concurrent Operation Safety**
  - **Validates: Requirements 1.4**
  - Generate random concurrent operations and verify no data races

- [x] 3. Implement proper locking mechanism
  - Modify `src/wallet/mod.rs` to use conditional compilation for locking
  - Add `#[cfg(not(test))]` for production locking behavior
  - Add `#[cfg(test)]` for test mode auto-unlock
  - Implement memory clearing on lock using `zeroize` for sensitive data
  - Clear `current_account`, signers, and cached keys
  - Add tracing logs for lock/unlock operations with correlation IDs
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 3.1 Write property test for lock memory clearing
  - **Property 3: Lock Memory Clearing**
  - **Validates: Requirements 2.3**
  - Test that sensitive data is cleared after lock

- [x] 3.2 Write property test for unlock restoration
  - **Property 4: Unlock Restoration**
  - **Validates: Requirements 2.4**
  - Test that operations work after unlock with correct credentials

- [x] 4. Implement session management with auto-lock
  - Create `src/security/session.rs` with `SessionManager`
  - Implement activity tracking with `Arc<RwLock<Instant>>`
  - Add configurable timeout with `Duration`
  - Implement auto-lock background task with `tokio::spawn`
  - _Requirements: 2.5_

- [x] 4.1 Write property test for auto-lock timeout
  - **Property 5: Auto-Lock Timeout**
  - **Validates: Requirements 2.5**
  - Test that wallet locks after configured timeout

- [x] 5. Implement hardware device manager
  - Create `src/wallet/hardware/device_manager.rs`
  - Implement `scan_devices()` using `LedgerSigner::list_devices()` and `TrezorSigner::list_devices()`
  - Add device registry with `Arc<RwLock<HashMap<DeviceId, HardwareDevice>>>`
  - Implement `handle_device_disconnect()` with reconnection logic
  - _Requirements: 3.1, 3.2, 3.3_

- [x] 5.1 Write property test for device registry consistency
  - **Property 6: Device Registry Consistency**
  - **Validates: Requirements 3.3**
  - Test registry accurately reflects connected devices

- [x] 5.2 Write property test for concurrent hardware operations
  - **Property 7: Concurrent Hardware Operations**
  - **Validates: Requirements 3.4**
  - Test multiple concurrent operations across devices

- [x] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement batch processor using Alloy
  - Create `src/performance/batch.rs` with `BatchProcessor`
  - Implement `batch_balance_queries()` using Alloy provider batch capabilities
  - Use `futures::future::join_all()` for concurrent requests
  - Add semaphore-based concurrency limiting with `Arc<Semaphore>` (default: 10 concurrent)
  - Implement retry logic with exponential backoff (max 3 retries, 1s → 2s → 4s)
  - Add correlation tracking for all batch operations
  - _Requirements: 6.1, 6.2, 6.3, 6.5_

- [x] 7.1 Write property test for batch RPC efficiency
  - **Property 11: Batch RPC Efficiency**
  - **Validates: Requirements 6.1**
  - Test that batch uses fewer RPC calls than individual requests
  - Generate random sets of addresses (10-100 addresses)
  - Count RPC calls for batch vs individual operations
  - Assert batch uses < N calls for N addresses

- [x] 7.2 Write property test for batch concurrency limiting
  - **Property 12: Batch Concurrency Limiting**
  - **Validates: Requirements 6.2**
  - Test that concurrent requests never exceed limit

- [x] 7.3 Write property test for batch partial failure handling
  - **Property 13: Batch Partial Failure Handling**
  - **Validates: Requirements 6.3**
  - Test that partial failures are handled gracefully

- [x] 7.4 Write property test for batch performance
  - **Property 14: Batch Performance Improvement**
  - **Validates: Requirements 6.4**
  - Benchmark and verify 50%+ performance improvement

- [x] 7.5 Write property test for batch retry with backoff
  - **Property 15: Batch Retry with Backoff**
  - **Validates: Requirements 6.5**
  - Test exponential backoff on network errors

- [x] 8. Implement structured logging with tracing
  - Create `src/telemetry/account_events.rs`
  - Add correlation ID generation using `uuid::Uuid::new_v4()`
  - Implement operation logging with `tracing::info_span!` and `.instrument()`
  - Add privacy mode filtering for sensitive data (keys, seeds, passwords)
  - Create log levels: info (success), warn (recoverable), error (failures)
  - Add span context propagation across async boundaries
  - _Requirements: 7.1, 7.3, 7.4, 7.5_

- [x] 8.1 Write property test for operation correlation logging
  - **Property 16: Operation Correlation Logging**
  - **Validates: Requirements 7.1**
  - Test that operations create correlation IDs

- [x] 8.2 Write property test for cross-component correlation
  - **Property 17: Cross-Component Correlation**
  - **Validates: Requirements 7.3**
  - Test correlation ID propagation across components

- [x] 8.3 Write property test for complete operation logging
  - **Property 18: Complete Operation Logging**
  - **Validates: Requirements 7.4**
  - Test that operations log start, completion, and errors

- [x] 8.4 Write property test for privacy mode log sanitization
  - **Property 19: Privacy Mode Log Sanitization**
  - **Validates: Requirements 7.5**
  - Test that logs don't contain sensitive data in privacy mode

- [x] 9. Implement account creation module
  - Create `src/wallet/account_manager/creation.rs` with `AccountCreator`
  - Implement `create_from_seed()` using Alloy's `MnemonicBuilder`
  - Implement `create_from_private_key()` using Alloy's `PrivateKeySigner`
  - Add validation and error handling with correlation tracking
  - _Requirements: 1.1_

- [x] 9.1 Write unit tests for account creation
  - Test seed-based account creation
  - Test private key account creation
  - Test validation and error cases
  - _Requirements: 1.1_

- [x] 10. Implement account import module
  - Create `src/wallet/account_manager/import.rs` with `AccountImporter`
  - Implement `import_from_seed()` with BIP39 validation
  - Implement `import_from_private_key()` with hex validation
  - Implement `import_from_metamask()` for MetaMask keystore format
  - Add format detection and validation
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 10.1 Write property test for seed import determinism
  - **Property 20: Seed Phrase Import Determinism**
  - **Validates: Requirements 8.2**
  - Test that same seed always produces same address
  - Generate random valid BIP39 seed phrases (12/15/18/21/24 words)
  - Import same seed multiple times with same derivation path
  - Assert all imports produce identical addresses

- [x] 10.2 Write property test for migration format validation
  - **Property 21: Migration Format Validation**
  - **Validates: Requirements 8.3**
  - Test that invalid formats are rejected with specific errors

- [x] 10.3 Write property test for migration metadata preservation
  - **Property 22: Migration Metadata Preservation**
  - **Validates: Requirements 8.4**
  - Test that metadata is preserved during migration

- [x] 10.4 Write property test for migration error specificity
  - **Property 23: Migration Error Specificity**
  - **Validates: Requirements 8.5**
  - Test that migration errors are specific and actionable

- [x] 11. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 12. Implement LRU cache for performance
  - Create `src/performance/cache.rs` with `SecureCache<K, V>`
  - Implement LRU eviction using `lru` crate (default capacity: 100 entries)
  - Add async-safe access with `Arc<RwLock<LruCache>>`
  - Implement cache invalidation and refresh logic with TTL (default: 5 minutes)
  - Add cache hit/miss metrics for monitoring
  - Ensure O(1) get/put operations
  - _Requirements: 9.1, 9.2, 9.4_

- [x] 12.1 Write property test for LRU cache correctness
  - **Property 24: LRU Cache Correctness**
  - **Validates: Requirements 9.1**
  - Test that cache stores and evicts correctly

- [x] 12.2 Write property test for cache staleness detection
  - **Property 25: Cache Staleness Detection**
  - **Validates: Requirements 9.2**
  - Test that stale data is detected and refreshed

- [x] 12.3 Write property test for cache performance
  - **Property 26: Cache Performance Improvement**
  - **Validates: Requirements 9.3**
  - Benchmark and verify 50%+ performance improvement

- [x] 12.4 Write property test for LRU eviction
  - **Property 27: LRU Eviction Under Pressure**
  - **Validates: Requirements 9.4**
  - Test that LRU entries are evicted first

- [ ] 13. Implement export authentication
  - Create `src/security/export_auth.rs` with `ExportAuthenticator`
  - Implement password verification with rate limiting
  - Add auth token generation with time-based expiration
  - Implement audit logging for export operations
  - _Requirements: 2.2_

- [ ] 13.1 Write unit tests for export authentication
  - Test password verification
  - Test rate limiting
  - Test token expiration
  - _Requirements: 2.2_

- [ ] 14. Implement account export module
  - Create `src/wallet/account_manager/export.rs` with `AccountExporter`
  - Implement `export_seed()` with authentication requirement
  - Implement `export_private_key()` with authentication requirement
  - Add correlation tracking for all export operations
  - _Requirements: 1.1_

- [ ] 14.1 Write unit tests for account export
  - Test seed phrase export with authentication
  - Test private key export with authentication
  - Test export failures without authentication
  - _Requirements: 1.1_

- [ ] 15. Implement telemetry system
  - Create `src/telemetry/account_events.rs` with `AccountTelemetry`
  - Implement event recording with privacy mode support
  - Add opt-out mechanism
  - Implement data anonymization (no PII, no full addresses)
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 15.1 Write property test for telemetry anonymity
  - **Property 29: Telemetry Anonymity**
  - **Validates: Requirements 10.1, 10.4**
  - Test that telemetry contains no sensitive data

- [ ] 16. Implement backup and recovery system
  - Create `src/wallet/backup/mod.rs` with `BackupManager`
  - Implement encrypted backup using AES-256-GCM with user password
  - Implement Shamir's Secret Sharing using `sharks` crate (configurable threshold)
  - Add backup integrity verification (SHA-256 checksums, HMAC)
  - Add support for multiple backup destinations (local file, optional cloud)
  - Include backup versioning with timestamps
  - Add correlation tracking for all backup operations
  - _Requirements: 11.1, 11.2, 11.3, 11.4_

- [ ] 16.1 Write property test for backup encryption
  - **Property 30: Backup Encryption**
  - **Validates: Requirements 11.1**
  - Test that backups are encrypted and require password

- [ ] 16.2 Write property test for Shamir round-trip
  - **Property 31: Shamir Secret Sharing Round-Trip**
  - **Validates: Requirements 11.2**
  - Test that threshold shares reconstruct original secret
  - Generate random secrets (32 bytes for private keys)
  - Generate random threshold (2-5) and total shares (3-10)
  - Split secret into shares, reconstruct from threshold subset
  - Assert reconstructed secret equals original

- [ ] 16.3 Write property test for backup integrity
  - **Property 32: Backup Integrity Verification**
  - **Validates: Requirements 11.4**
  - Test that corrupted backups are detected and rejected

- [ ] 17. Implement account metadata management
  - Create `src/wallet/account_manager/metadata.rs`
  - Implement nickname validation (uniqueness, character restrictions)
  - Implement deterministic avatar generation based on address
  - Implement tag management (add, remove, query by tag)
  - Add activity tracking (last used, transaction count)
  - _Requirements: 12.1, 12.2, 12.3, 12.4_

- [ ] 17.1 Write property test for nickname uniqueness
  - **Property 33: Nickname Uniqueness**
  - **Validates: Requirements 12.1**
  - Test that duplicate nicknames are rejected

- [ ] 17.2 Write property test for avatar determinism
  - **Property 34: Avatar Determinism**
  - **Validates: Requirements 12.2**
  - Test that same address always produces same avatar

- [ ] 17.3 Write property test for tag management
  - **Property 35: Tag Management Consistency**
  - **Validates: Requirements 12.3**
  - Test that tag queries return correct accounts

- [ ] 18. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 19. Integration and wiring
  - Wire AccountManager implementation to GUI components
  - Update `src/gui/components/account_manager.rs` to use new AccountManager trait
  - Connect batch processor to balance queries
  - Connect telemetry to all account operations
  - Add correlation tracking to all error paths
  - _Requirements: All_

- [ ] 19.1 Write integration tests
  - Test complete account lifecycle (create, lock, unlock, export)
  - Test hardware wallet integration (if devices available)
  - Test batch operations with real Alloy provider (testnet)
  - Test correlation tracking across components
  - _Requirements: All_

- [ ] 20. Performance optimization and benchmarking
  - Run performance benchmarks for batch operations
  - Run performance benchmarks for cache hit rates
  - Verify async operations don't block UI (< 16ms)
  - Optimize any bottlenecks found
  - _Requirements: 6.4, 9.3, 9.5_

- [ ] 20.1 Write performance tests
  - Benchmark batch vs individual operations
  - Benchmark cached vs uncached access
  - Measure lock/unlock times
  - Measure account creation times
  - _Requirements: 6.4, 9.3_

- [ ] 21. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- All tasks are required for comprehensive implementation
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- Integration tests validate end-to-end workflows
- Performance tests validate optimization targets
- All tests should use correlation IDs for debugging
- Hardware wallet tests may be skipped if devices unavailable

## Success Criteria

**Code Quality:**
- All modules under 200 lines
- 90%+ test coverage including property-based tests
- Zero compiler warnings
- All rustdoc comments complete

**Security:**
- Zero security bypasses in production code
- All exports require authentication
- All sensitive data uses `zeroize`
- Correlation tracking on all operations

**Performance:**
- Batch operations: 50%+ improvement over individual requests
- Cache hit rate: 80%+ for repeated operations
- Lock/unlock: < 100ms for lock, < 500ms for unlock
- UI responsiveness: No blocking > 16ms

**Testing:**
- All property tests run 100+ iterations
- All tests pass consistently
- Integration tests cover complete workflows
- Performance benchmarks meet targets

**Observability:**
- All operations have correlation IDs
- Structured logging with `tracing`
- Privacy mode filters sensitive data
- Clear error messages for users

