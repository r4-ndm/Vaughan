# Professional Wallet Improvement - Tasks

**Feature**: Professional Wallet Codebase Excellence
**Status**: Not Started
**Created**: 2025-01-24

## Task Overview

This task list implements the professional wallet improvement requirements through 5 phases:
- Phase 0: Security Audit (Week 0)
- Phase 1: Critical Property-Based Testing (Week 1)
- Phase 2: Module Refactoring (Week 2-3)
- Phase 3: Comprehensive Property Testing (Week 3-4)
- Phase 4: Warning Cleanup & Documentation (Week 4)

**IMPORTANT**: Each phase includes rollback procedures. If any validation fails, immediately revert changes and document the issue before proceeding.

---

## Pre-Phase 0: Preparation

### [ ] 0.0 Establish Baseline and Branch
**Requirements**: Dependencies  
**Priority**: Critical

Establish performance baseline and create safe working branch.

**Subtasks:**
- [ ] 0.0.1 Verify all 333 tests pass: `cargo test --all-features`
- [ ] 0.0.2 Create feature branch: `git checkout -b feature/professional-improvement`
- [ ] 0.0.3 Run and save performance baseline: `cargo bench > baseline_performance.txt`
  - Verify benches/account_manager_benchmarks.rs exists
  - Verify benches/wallet_benchmarks.rs exists
  - Confirm batch operation benchmarks run
  - Confirm LRU cache benchmarks run
  - Confirm lock/unlock benchmarks run
  - Document any missing benchmarks
- [ ] 0.0.4 Save test baseline: `cargo test --all-features > baseline_tests.txt`
- [ ] 0.0.5 Verify Trezor device availability (if available)
- [ ] 0.0.6 Verify Ledger device availability (if available)
- [ ] 0.0.7 Locate or create design document with 35 properties
- [ ] 0.0.8 Audit Alloy vs MetaMask code attribution
  - Verify hardware wallet implementation uses Alloy signers (alloy-signer-ledger, alloy-signer-trezor)
  - Identify any MetaMask-inspired patterns (keystore encryption, etc.)
  - Check for proper attribution comments in code
  - Document which crypto libraries are MetaMask-compatible
  - Create attribution map document
- [ ] 0.0.9 Check Alloy version compatibility
  - Current version: alloy 1.5
  - Run `cargo audit` for security advisories
  - Check for breaking changes in newer Alloy versions
  - Document upgrade path if needed

**Validation:**
- All tests passing
- Branch created
- Baseline files saved
- Hardware device status documented
- Alloy attribution map created (NEW)
- Alloy version compatibility checked (NEW)

**Rollback:** N/A (preparation phase)

**Note:** Tasks 0.0.8 and 0.0.9 are critical for understanding actual Alloy usage vs MetaMask patterns in the codebase.

---

## Phase 0: Security Audit (Week 0)

### [ ] 0.1 Unsafe Block Audit
**Requirements**: FR-1.1  
**Priority**: Critical

Audit all 12 unsafe blocks in the codebase for memory safety.

**Subtasks:**
- [ ] 0.1.1 Locate all unsafe blocks: `rg "unsafe" --type rust -A 5`
- [ ] 0.1.2 Categorize by file and purpose:
  - Platform-specific (sandbox.rs, memory locking via mlock/VirtualLock)
  - FFI (hardware wallet communication)
  - Performance optimization
  - Other
- [ ] 0.1.3 Document each unsafe block with safety rationale
- [ ] 0.1.4 Verify memory safety guarantees for each block
- [ ] 0.1.5 Add `// SAFETY:` comments following Rust guidelines
- [ ] 0.1.6 Create tracking document: `UNSAFE_CODE_AUDIT.md`

**Validation:**
- All unsafe blocks have `// SAFETY:` comments
- Each safety rationale references specific invariants
- No undefined behavior possible
- Categorization document created (UNSAFE_CODE_AUDIT.md)

**Rollback:** Revert documentation changes if safety rationale cannot be established

**Note:** Cargo.toml mentions sandbox.rs specifically - likely contains platform-specific memory locking (mlock on Unix, VirtualLock on Windows) which requires unsafe.

---

### [ ] 0.2 Side-Channel Attack Surface Review
**Requirements**: FR-1.5  
**Priority**: Critical

Review code for potential side-channel vulnerabilities.

**Subtasks:**
- [ ] 0.2.1 Identify operations that process secret data
- [ ] 0.2.2 Check for timing-dependent branches on secrets
- [ ] 0.2.3 Review for cache-timing vulnerabilities
- [ ] 0.2.4 Check for power analysis vulnerabilities (hardware wallet communication)
- [ ] 0.2.5 Document side-channel mitigations

**Validation:**
- No secret-dependent branching identified
- Cache-timing risks documented
- Mitigations in place

**Rollback:** N/A (audit only, no code changes)

---

### [ ] 0.3 Constant-Time Cryptography Audit
**Requirements**: FR-1.2  
**Priority**: Critical

Verify all cryptographic operations execute in constant time to prevent timing attacks.

**Subtasks:**
- [ ] 0.2.1 Identify all cryptographic operations (signing, verification, derivation)
- [ ] 0.2.2 Review Alloy library usage for constant-time guarantees
- [ ] 0.2.3 Review MetaMask-inspired code for timing vulnerabilities
- [ ] 0.2.4 Add timing-attack tests for critical operations
- [ ] 0.2.5 Document constant-time guarantees in code

**Validation:**
- All key operations use constant-time implementations
- No branching on secret data
- Timing tests pass for all crypto operations

**Rollback:** N/A (audit only, no code changes)

---

### [ ] 0.4 Memory Zeroization Audit
**Requirements**: FR-1.3  
**Priority**: Critical

Verify all sensitive data is properly zeroized after use.

**Subtasks:**
- [ ] 0.3.1 Audit all types containing sensitive data (keys, mnemonics, passwords)
- [ ] 0.3.2 Verify Zeroize trait implementation on all sensitive types
- [ ] 0.3.3 Check Drop implementations for proper cleanup
- [ ] 0.3.4 Add zeroization tests for all sensitive types
- [ ] 0.3.5 Document zeroization guarantees

**Validation:**
- All sensitive types implement Zeroize
- Drop implementations call zeroize()
- Memory tests verify data cleared

**Rollback:** N/A (audit only, tests added but can be removed if needed)

---

### [ ] 0.5 RNG Quality Audit
**Requirements**: FR-1.4  
**Priority**: Critical

Audit random number generation for cryptographic quality.

**Subtasks:**
- [ ] 0.4.1 Identify all RNG usage in key generation
- [ ] 0.4.2 Verify use of cryptographically secure RNG (OsRng)
- [ ] 0.4.3 Check for proper entropy sources
- [ ] 0.4.4 Review seed generation for BIP-39 mnemonics
- [ ] 0.4.5 Document RNG sources and guarantees

**Validation:**
- All key generation uses OsRng or equivalent
- No weak RNG sources (rand::thread_rng for crypto)
- Entropy sources documented

**Rollback:** N/A (audit only, no code changes)

---

### [ ] 0.6 Hardware Wallet Security Audit
**Requirements**: FR-1.6  
**Priority**: High

Audit Trezor and Ledger integration for security best practices.

**Subtasks:**
- [ ] 0.5.1 Review Trezor integration code (MetaMask patterns)
- [ ] 0.5.2 Review Ledger integration code (MetaMask patterns)
- [ ] 0.5.3 Verify device communication error handling
- [ ] 0.5.4 Check for proper device state management
- [ ] 0.5.5 Verify no private keys exposed in device communication
- [ ] 0.5.6 Document hardware wallet security model

**Validation:**
- Device communication properly error-handled
- No sensitive data leakage in logs
- Thread-safe device state management

**Rollback:** N/A (audit only, no code changes)

---

### [ ] 0.7 Cryptographic Library Attribution Audit
**Requirements**: FR-5.6, TC-1  
**Priority**: High

Audit all cryptographic libraries for Alloy vs MetaMask attribution.

**Subtasks:**
- [ ] 0.7.1 Audit keystore encryption libraries (aes, ctr, pbkdf2, sha2)
- [ ] 0.7.2 Verify MetaMask compatibility claims in comments
- [ ] 0.7.3 Check if Alloy provides alternatives to these libraries
- [ ] 0.7.4 Add attribution comments where needed
- [ ] 0.7.5 Document why each non-Alloy crypto library is used
- [ ] 0.7.6 Verify eth-keystore usage follows EIP-2335 standard

**Validation:**
- All crypto libraries have attribution
- MetaMask compatibility documented
- Rationale for non-Alloy libraries clear
- EIP-2335 compliance verified

**Rollback:** N/A (audit only, attribution comments added)

**Note:** Alloy provides signing/verification but NOT keystore encryption. Libraries like `aes`, `pbkdf2` are necessary and follow MetaMask's keystore format (EIP-2335 compatible). The `eth-keystore` crate likely handles this.

---

## Phase 1: Critical Property-Based Testing (Week 1)

### [ ] 1.1 Setup Property Testing Infrastructure
**Requirements**: FR-2  
**Priority**: Critical

Create property testing framework and utilities.

**Subtasks:**
- [ ] 1.1.1 Create `tests/properties/` directory structure
- [ ] 1.1.2 Create `tests/properties/mod.rs` with shared utilities
- [ ] 1.1.3 Add proptest dependency configuration (already in Cargo.toml)
- [ ] 1.1.4 Create test data generators for common types
- [ ] 1.1.5 Setup proptest configuration (iterations, timeout)
- [ ] 1.1.6 Configure feature flag test matrix
  - Test with: minimal, default, full features
  - Ensure property tests run in all configurations
  - Document feature-specific test requirements
- [ ] 1.1.7 Setup proptest regression file management
  - Create proptest-regressions/ directory structure
  - Decide: commit regression files or add to .gitignore
  - Document regression file handling policy

**Validation:**
- Property test infrastructure compiles
- Sample property test runs successfully
- Test utilities are reusable
- Tests pass with minimal, default, and full features

**Rollback:** `git checkout -- tests/properties/` if infrastructure broken

**Note:** Vaughan uses complex feature flags (minimal, qr, audio, hardware-wallets, professional, custom-tokens, shamir, telemetry). Property tests must work with all combinations.

---

### [ ] 1.2 Property 3: Lock Memory Clearing
**Requirements**: FR-2.1  
**Priority**: Critical

Implement property test verifying memory is cleared on wallet lock.

**Subtasks:**
- [ ] 1.2.1 Create `tests/properties/security.rs`
- [ ] 1.2.2 Implement memory inspection utilities
- [ ] 1.2.3 Write property: "After lock, no sensitive data in memory"
- [ ] 1.2.4 Configure 10,000 test iterations
- [ ] 1.2.5 Verify test catches memory leaks

**Validation:**
- Property test passes with 10,000 iterations
- Test fails if zeroization removed (negative test)
- Memory inspection reliable

**Rollback:** `git checkout -- tests/properties/security.rs` if test unreliable

**Note:** 10,000 iterations is industry standard for memory safety validation (reference: Rust Secure Code Working Group guidelines)

---

### [ ] 1.3 Property 31: Shamir Secret Sharing Round-Trip
**Requirements**: FR-2.2  
**Priority**: Critical

Implement property test for SSS reconstruction correctness.

**Subtasks:**
- [ ] 1.3.1 Add to `tests/properties/security.rs`
- [ ] 1.3.2 Generate random secrets and share configurations
- [ ] 1.3.3 Write property: "split(secret) then combine(shares) == secret"
- [ ] 1.3.4 Configure 1,000 test iterations
- [ ] 1.3.5 Test various threshold configurations (2-of-3, 3-of-5, etc.)

**Validation:**
- Property test passes with 1,000 iterations
- All threshold configurations tested
- Round-trip always recovers original secret

**Rollback:** `git checkout -- tests/properties/security.rs` if test fails

**Note:** 1,000 iterations is standard for cryptographic correctness validation

---

### [ ] 1.4 Property 20: Seed Phrase Import Determinism
**Requirements**: FR-2.3  
**Priority**: Critical

Implement property test for deterministic key derivation.

**Subtasks:**
- [ ] 1.4.1 Create `tests/properties/crypto.rs`
- [ ] 1.4.2 Generate random BIP-39 mnemonics
- [ ] 1.4.3 Write property: "import(mnemonic) twice yields same keys"
- [ ] 1.4.4 Configure 1,000 test iterations
- [ ] 1.4.5 Test with various derivation paths

**Validation:**
- Property test passes with 1,000 iterations
- Same mnemonic always produces same keys
- Derivation paths correctly handled

---

### [ ] 1.5 Property 2: Concurrent Operation Safety
**Requirements**: FR-2.4  
**Priority**: Critical

Implement property test for thread-safe account operations.

**Subtasks:**
- [ ] 1.5.1 Create `tests/properties/interface.rs`
- [ ] 1.5.2 Setup concurrent test harness
- [ ] 1.5.3 Write property: "Concurrent operations maintain consistency"
- [ ] 1.5.4 Configure 1,000 test iterations
- [ ] 1.5.5 Test various operation combinations

**Validation:**
- Property test passes with 1,000 iterations
- No data races detected
- Account state always consistent

---

### [ ] 1.6 Property 1: Unified Interface Consistency
**Requirements**: FR-2.5  
**Priority**: Critical

Implement property test for AccountManager trait consistency.

**Subtasks:**
- [ ] 1.6.1 Add to `tests/properties/interface.rs`
- [ ] 1.6.2 Generate random account operations
- [ ] 1.6.3 Write property: "create then get returns same account"
- [ ] 1.6.4 Configure 1,000 test iterations
- [ ] 1.6.5 Test all CRUD operations

**Validation:**
- Property test passes with 1,000 iterations
- Interface invariants maintained
- All operations consistent

---

## Phase 2: Module Refactoring (Week 2-3)

### [ ] 2.1 Refactor account_manager/mod.rs
**Requirements**: FR-3.1  
**Priority**: High

Split 1,777-line module into focused submodules.

**Subtasks:**
- [ ] 2.1.1 Create `account_manager/coordinator.rs` (~300 lines)
- [ ] 2.1.2 Create `account_manager/types.rs` (~100 lines)
- [ ] 2.1.3 Create `account_manager/lifecycle.rs` (~200 lines)
- [ ] 2.1.4 Create `account_manager/auth.rs` (~150 lines)
- [ ] 2.1.5 Update mod.rs to re-export from submodules
- [ ] 2.1.6 Run all tests to verify no breakage
- [ ] 2.1.7 Verify module sizes with `tokei`

**Validation:**
- All 333 tests still pass
- coordinator.rs < 400 lines
- Other modules < 200 lines
- No functionality lost

**Rollback:** `git checkout -- src/wallet/account_manager/` if tests fail or functionality broken

---

### [ ] 2.2 Refactor account_manager/import.rs
**Requirements**: FR-3.2  
**Priority**: High

Split 964-line import module into focused submodules.

**Subtasks:**
- [ ] 2.2.1 Create `account_manager/import/parsers.rs` (~200 lines)
- [ ] 2.2.2 Create `account_manager/import/validators.rs` (~150 lines)
- [ ] 2.2.3 Create `account_manager/import/converters.rs` (~150 lines)
- [ ] 2.2.4 Update import/mod.rs to coordinate submodules
- [ ] 2.2.5 Run import-related tests
- [ ] 2.2.6 Verify module sizes

**Validation:**
- All import tests pass
- All modules < 200 lines
- Import functionality preserved

---

### [ ] 2.3 Refactor performance/batch.rs
**Requirements**: FR-3.3  
**Priority**: High

Split 878-line batch processor into focused submodules.

**Subtasks:**
- [ ] 2.3.1 Create `performance/batch/config.rs` (~100 lines)
- [ ] 2.3.2 Create `performance/batch/processor.rs` (~200 lines)
- [ ] 2.3.3 Create `performance/batch/retry.rs` (~150 lines)
- [ ] 2.3.4 Update batch/mod.rs to coordinate submodules
- [ ] 2.3.5 Run performance benchmarks
- [ ] 2.3.6 Verify no performance regression

**Validation:**
- Performance benchmarks maintained
- All modules < 200 lines
- Batch operations functional

---

### [ ] 2.4 Refactor telemetry/account_events.rs
**Requirements**: FR-3.4  
**Priority**: Medium

Split 801-line telemetry module into focused submodules.

**Subtasks:**
- [ ] 2.4.1 Create `telemetry/account_events/logger.rs` (~150 lines)
- [ ] 2.4.2 Create `telemetry/account_events/spans.rs` (~150 lines)
- [ ] 2.4.3 Create `telemetry/account_events/privacy.rs` (~100 lines)
- [ ] 2.4.4 Update account_events/mod.rs to coordinate
- [ ] 2.4.5 Run telemetry tests
- [ ] 2.4.6 Verify privacy filtering still works

**Validation:**
- Telemetry tests pass
- All modules < 200 lines
- Privacy guarantees maintained

---

### [ ] 2.5 Refactor account_manager/metadata.rs
**Requirements**: FR-3.5  
**Priority**: Low

Split 281-line metadata module if needed.

**Subtasks:**
- [ ] 2.5.1 Analyze metadata.rs structure
- [ ] 2.5.2 Determine if split needed (currently 281 lines)
- [ ] 2.5.3 If needed, create submodules
- [ ] 2.5.4 Run metadata tests
- [ ] 2.5.5 Verify module size

**Validation:**
- Module < 200 lines (or justified exception)
- Metadata tests pass
- Functionality preserved

---

## Phase 3: Comprehensive Property Testing (Week 3-4)

### [ ] 3.1 Property 8: Error Context Completeness
**Requirements**: FR-2.6  
**Priority**: Medium

Implement property test for error context information.

**Subtasks:**
- [ ] 3.1.1 Create `tests/properties/error.rs`
- [ ] 3.1.2 Generate random error scenarios
- [ ] 3.1.3 Write property: "All errors contain context"
- [ ] 3.1.4 Configure 500 test iterations
- [ ] 3.1.5 Verify error messages are actionable

**Validation:**
- Property test passes with 500 iterations
- All errors have context
- Error messages are helpful

---

### [ ] 3.2 Property 24: LRU Cache Correctness
**Requirements**: FR-2.7  
**Priority**: Medium

Implement property test for LRU cache behavior.

**Subtasks:**
- [ ] 3.2.1 Add to `tests/properties/crypto.rs`
- [ ] 3.2.2 Generate random cache operations
- [ ] 3.2.3 Write property: "LRU eviction maintains correctness"
- [ ] 3.2.4 Configure 500 test iterations
- [ ] 3.2.5 Test cache hit/miss behavior

**Validation:**
- Property test passes with 500 iterations
- LRU eviction correct
- Cache consistency maintained

---

### [ ] 3.3 Property 33: Nickname Uniqueness
**Requirements**: FR-2.8  
**Priority**: Medium

Implement property test for account nickname uniqueness.

**Subtasks:**
- [ ] 3.3.1 Add to `tests/properties/crypto.rs`
- [ ] 3.3.2 Generate random nickname operations
- [ ] 3.3.3 Write property: "Nicknames remain unique"
- [ ] 3.3.4 Configure 500 test iterations
- [ ] 3.3.5 Test nickname collision handling

**Validation:**
- Property test passes with 500 iterations
- No duplicate nicknames
- Collision handling correct

---

### [ ] 3.4 Implement Remaining 27 Properties
**Requirements**: FR-2.9  
**Priority**: Medium

Implement remaining design properties with 100+ iterations each.

**Subtasks:**
- [ ] 3.4.1 Locate design document with all 35 properties (check .kiro/specs/ or docs/)
- [ ] 3.4.2 Identify remaining 27 properties not yet implemented
- [ ] 3.4.3 Prioritize by security/functional importance
- [ ] 3.4.4 Implement properties in batches of 5
- [ ] 3.4.5 Configure 100+ iterations for each
- [ ] 3.4.6 Document property coverage

**Validation:**
- All 35 properties have tests
- Each property has 100+ iterations
- Property coverage documented

**Rollback:** Remove failing property tests if they cannot be fixed

**Note:** If design document with 35 properties doesn't exist, create it first by extracting properties from existing code and requirements

---

## Phase 4: Warning Cleanup & Documentation (Week 4)

### [ ] 4.1 Automated Warning Fixes
**Requirements**: FR-4.1, FR-4.2  
**Priority**: Medium

Use cargo fix to automatically resolve warnings.

**Subtasks:**
- [ ] 4.1.1 Run `cargo fix --lib --allow-dirty`
- [ ] 4.1.2 Run `cargo fix --tests --allow-dirty`
- [ ] 4.1.3 Run `cargo clippy --fix --allow-dirty`
- [ ] 4.1.4 Review automated changes
- [ ] 4.1.5 Run full test suite

**Validation:**
- Automated fixes applied
- All tests still pass
- No functionality broken

**Rollback:** `git checkout -- .` if automated fixes break functionality, then apply fixes manually

---

### [ ] 4.2 Manual Warning Cleanup
**Requirements**: FR-4.1, FR-4.2, FR-4.3  
**Priority**: Medium

Manually resolve remaining warnings.

**Subtasks:**
- [ ] 4.2.1 Remove remaining unused imports
- [ ] 4.2.2 Prefix unused variables with underscore
- [ ] 4.2.3 Remove dead code instances
- [ ] 4.2.4 Verify no functionality lost
- [ ] 4.2.5 Run `cargo check` to verify zero warnings

**Validation:**
- `cargo check` produces zero warnings
- All tests pass
- No dead code remains

---

### [ ] 4.3 Document Unsafe Blocks
**Requirements**: FR-4.4  
**Priority**: High

Add safety documentation to all unsafe blocks.

**Subtasks:**
- [ ] 4.3.1 Review unsafe block audit from Phase 0
- [ ] 4.3.2 Add `// SAFETY:` comments to each block
- [ ] 4.3.3 Document invariants and guarantees
- [ ] 4.3.4 Reference relevant safety proofs
- [ ] 4.3.5 Review with security mindset

**Validation:**
- All unsafe blocks documented
- Safety rationale clear
- Invariants explicit

---

### [ ] 4.4 Clippy Compliance
**Requirements**: FR-4.6  
**Priority**: Medium

Achieve zero clippy warnings.

**Subtasks:**
- [ ] 4.4.1 Run `cargo clippy -- -D warnings`
- [ ] 4.4.2 Fix all clippy warnings
- [ ] 4.4.3 Review clippy suggestions for improvements
- [ ] 4.4.4 Add clippy configuration if needed
- [ ] 4.4.5 Verify zero warnings

**Validation:**
- `cargo clippy -- -D warnings` passes
- Code follows Rust idioms
- No clippy warnings remain

---

### [ ] 4.5 Public API Documentation
**Requirements**: FR-5.1, FR-5.2  
**Priority**: Medium

Document all public APIs with rustdoc.

**Subtasks:**
- [ ] 4.5.1 Run `cargo doc --no-deps --open`
- [ ] 4.5.2 Identify undocumented public items
- [ ] 4.5.3 Add rustdoc comments to all public functions
- [ ] 4.5.4 Add examples to complex APIs
- [ ] 4.5.5 Verify documentation builds without warnings

**Validation:**
- All public APIs documented
- Examples compile and run
- `cargo doc` produces no warnings

---

### [ ] 4.6 Performance Documentation
**Requirements**: FR-5.3  
**Priority**: Low

Document performance characteristics of APIs.

**Subtasks:**
- [ ] 4.6.1 Identify performance-critical APIs
- [ ] 4.6.2 Add time complexity documentation
- [ ] 4.6.3 Add space complexity documentation
- [ ] 4.6.4 Document caching behavior
- [ ] 4.6.5 Document batch operation benefits

**Validation:**
- Performance characteristics documented
- Complexity analysis accurate
- Optimization guidance provided

---

### [ ] 4.7 Error Documentation
**Requirements**: FR-5.4  
**Priority**: Medium

Document all error conditions and types.

**Subtasks:**
- [ ] 4.7.1 Audit all error types
- [ ] 4.7.2 Document when each error occurs
- [ ] 4.7.3 Document error recovery strategies
- [ ] 4.7.4 Add error examples to documentation
- [ ] 4.7.5 Document error context information

**Validation:**
- All error types documented
- Error conditions clear
- Recovery strategies provided

---

### [ ] 4.8 Hardware Wallet Documentation
**Requirements**: FR-5.5  
**Priority**: Medium

Document hardware wallet integration patterns.

**Subtasks:**
- [ ] 4.8.1 Document Trezor integration (MetaMask patterns)
- [ ] 4.8.2 Document Ledger integration (MetaMask patterns)
- [ ] 4.8.3 Document device communication protocol
- [ ] 4.8.4 Document error handling strategies
- [ ] 4.8.5 Add hardware wallet usage examples

**Validation:**
- Hardware wallet integration documented
- MetaMask pattern attribution clear
- Usage examples provided

---

### [ ] 4.9 Code Attribution Documentation
**Requirements**: FR-5.6  
**Priority**: Low

Document Alloy vs MetaMask code attribution.

**Subtasks:**
- [ ] 4.9.1 Audit codebase for MetaMask-inspired code
- [ ] 4.9.2 Add attribution comments where needed
- [ ] 4.9.3 Document why MetaMask pattern used (Alloy insufficient)
- [ ] 4.9.4 Create attribution reference document
- [ ] 4.9.5 Verify all attributions present

**Validation:**
- All MetaMask code attributed
- Rationale documented
- Attribution reference complete

**Rollback:** N/A (documentation only)

**Note:** Based on code audit, hardware wallets use Alloy signers (alloy-signer-ledger, alloy-signer-trezor). MetaMask patterns are primarily in keystore encryption (eth-keystore, aes, pbkdf2) which follow EIP-2335 standard.

---

### [ ] 4.10 Document Feature Flag System
**Requirements**: FR-5.1  
**Priority**: Medium

Document the feature flag system for users and developers.

**Subtasks:**
- [ ] 4.10.1 Document each feature flag purpose:
  - minimal: Core wallet functionality only
  - qr: QR code generation for addresses
  - audio: Audio notifications for incoming transactions
  - hardware-wallets: Ledger and Trezor support via Alloy signers
  - professional: Professional network monitoring features
  - custom-tokens: Custom token management features
  - shamir: Shamir's Secret Sharing (sharks crate)
  - telemetry: OpenTelemetry metrics (optional)
- [ ] 4.10.2 Document feature dependencies and conflicts
- [ ] 4.10.3 Document recommended feature combinations
- [ ] 4.10.4 Add feature flag examples to README
- [ ] 4.10.5 Document build time impact of features
- [ ] 4.10.6 Document testing requirements per feature

**Validation:**
- Feature flags documented in README
- Examples provided for common use cases
- Build time impact measured and documented
- Testing matrix documented

**Rollback:** N/A (documentation only)

**Note:** Current default features: minimal, qr, audio, hardware-wallets, professional, custom-tokens. Full feature set includes all features.

---

## Final Validation

### [ ] 5.1 Comprehensive Test Suite
**Requirements**: NFR-3  
**Priority**: Critical

Verify all tests pass after all changes.

**Subtasks:**
- [ ] 5.1.1 Run `cargo test --all-features`
- [ ] 5.1.2 Verify all 333 original tests pass
- [ ] 5.1.3 Verify all new property tests pass
- [ ] 5.1.4 Run integration tests
- [ ] 5.1.5 Run performance benchmarks

**Validation:**
- All tests pass
- No test regressions
- New tests integrated

---

### [ ] 5.2 Performance Validation
**Requirements**: NFR-1  
**Priority**: Critical

Verify no performance regressions.

**Subtasks:**
- [ ] 5.2.1 Run batch operation benchmarks
- [ ] 5.2.2 Run LRU cache benchmarks
- [ ] 5.2.3 Run lock/unlock benchmarks
- [ ] 5.2.4 Compare with baseline metrics
- [ ] 5.2.5 Document any changes

**Validation:**
- Batch operations: 244-270% improvement maintained
- LRU cache: 10,534x speedup maintained
- Lock/unlock: 11.8µs / 1.9µs maintained

---

### [ ] 5.3 Security Validation
**Requirements**: NFR-2  
**Priority**: Critical

Verify no security regressions.

**Subtasks:**
- [ ] 5.3.1 Run all security property tests
- [ ] 5.3.2 Verify memory zeroization tests pass
- [ ] 5.3.3 Verify constant-time tests pass
- [ ] 5.3.4 Review unsafe block documentation
- [ ] 5.3.5 Manual security review

**Validation:**
- All security tests pass
- No new vulnerabilities introduced
- Security guarantees maintained

---

### [ ] 5.4 Code Quality Validation
**Requirements**: NFR-4  
**Priority**: High

Verify code quality standards met.

**Subtasks:**
- [ ] 5.4.1 Run `cargo check` - verify zero warnings
- [ ] 5.4.2 Run `cargo clippy -- -D warnings` - verify passes
- [ ] 5.4.3 Run `tokei` - verify module sizes
- [ ] 5.4.4 Run `cargo doc` - verify complete documentation
- [ ] 5.4.5 Review code for idiomatic Rust

**Validation:**
- Zero compiler warnings
- Zero clippy warnings
- All modules under size limits
- Complete documentation

---

### [ ] 5.5 Hardware Wallet Integration Testing
**Requirements**: FR-1.6  
**Priority**: High

Test hardware wallet integration with real devices.

**Subtasks:**
- [ ] 5.5.1 Test Trezor device connection
- [ ] 5.5.2 Test Trezor transaction signing
- [ ] 5.5.3 Test Ledger device connection
- [ ] 5.5.4 Test Ledger transaction signing
- [ ] 5.5.5 Test error handling with devices

**Validation:**
- Trezor integration functional
- Ledger integration functional
- Error handling robust

---

## Success Metrics

**Code Quality:**
- ✅ All modules < 400 lines (coordinators) or < 200 lines (logic)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Complete rustdoc coverage
- ✅ All feature flag combinations tested

**Security:**
- ✅ All unsafe blocks documented and categorized
- ✅ All crypto operations constant-time verified
- ✅ All sensitive data zeroization verified
- ✅ All 35 property tests passing
- ✅ All cryptographic libraries attributed (Alloy vs MetaMask)

**Performance:**
- ✅ No regression from baseline benchmarks
- ✅ All 333+ tests passing
- ✅ Build time not increased by >10%
- ✅ Benchmarks verified to exist and run

**Documentation:**
- ✅ All public APIs documented
- ✅ All error conditions documented
- ✅ Hardware wallet integration documented (Alloy signers)
- ✅ Alloy vs MetaMask attribution complete
- ✅ Feature flag system documented
