# Phase 2 Implementation Plan: Module Refactoring

**Feature**: Professional Wallet Codebase Excellence - Phase 2
**Status**: Ready for Execution
**Created**: 2025-01-25
**Priority**: High

## Overview

This document provides a detailed, step-by-step implementation plan for Phase 2 (Module Refactoring) of the professional wallet improvement initiative. Phase 2 focuses on splitting oversized modules into focused, maintainable submodules while preserving all functionality and test coverage.

## Current State Analysis

### Module Sizes (Current)
```
account_manager/mod.rs:        1,596 lines (7.98x over 200-line limit, 3.99x over 400-line coordinator limit)
account_manager/import.rs:       883 lines (4.42x over 200-line limit)
performance/batch.rs:            774 lines (3.87x over 200-line limit)
telemetry/account_events.rs:     726 lines (3.63x over 200-line limit)
account_manager/metadata.rs:     250 lines (1.25x over 200-line limit)
account_manager/types.rs:        230 lines (ALREADY CREATED - 1.15x over limit, acceptable)
```

### Target Module Sizes
- **Coordinator modules**: Maximum 400 lines (orchestration, trait implementations)
- **Logic modules**: Maximum 200 lines (focused functionality)
- **Re-export modules (mod.rs)**: Maximum 50 lines (re-exports only)

### Success Criteria
- ✅ All modules under size limits (400/200 lines)
- ✅ All 399 tests still passing
- ✅ No functionality lost
- ✅ No performance regression
- ✅ Clear separation of concerns

## Prerequisites

### Before Starting Phase 2
1. ✅ Phase 0 complete (security audit)
2. ✅ Phase 1 complete (critical property-based testing)
3. ✅ All 399 tests passing
4. ✅ Git branch: `feature/professional-improvement`
5. ✅ Baseline established

### Required Tools
- Rust toolchain (cargo, rustc)
- Git for version control
- PowerShell for line counting

## Task 2.1: Refactor account_manager/mod.rs (1,596 lines → ~400 lines)

### Overview
Split the massive account_manager/mod.rs into focused submodules. This is the most complex refactoring task.

### Current Structure Analysis
```rust
// Current mod.rs contains:
- Module declarations (7 lines): creation, import, export, metadata, signer_integration, discovery, eip712
- Type definitions (~300 lines): AuthToken, AuthorizedOperation, AccountConfig, AccountType, SeedStrength, ImportSource
- AccountManagerTrait definition (~150 lines): Core trait with all operations
- AccountManagerTrait implementation (~800 lines): Concrete implementation
- Test modules (~300 lines): tests, interface_tests, property_tests, lock_property_tests
```

### Target Structure
```
src/wallet/account_manager/
├── mod.rs                    (~50 lines - re-exports only)
├── types.rs                  (230 lines - ALREADY CREATED ✅)
├── coordinator.rs            (~350 lines - AccountManagerTrait implementation)
├── lifecycle.rs              (~200 lines - CRUD operations: create, list, get, remove)
├── auth.rs                   (~150 lines - lock, unlock, authentication)
├── creation.rs               (existing - no changes)
├── import.rs                 (existing - will be refactored in Task 2.2)
├── export.rs                 (existing - no changes)
├── metadata.rs               (existing - will be refactored in Task 2.5)
├── signer_integration.rs     (existing - no changes)
├── discovery.rs              (existing - no changes)
└── eip712.rs                 (existing - no changes)
```

### Step-by-Step Implementation

#### Step 1: Create coordinator.rs (AccountManagerTrait implementation)
**Estimated Lines**: ~350 lines
**Purpose**: Implement the AccountManagerTrait by delegating to specialized modules

**Actions**:
1. Create `src/wallet/account_manager/coordinator.rs`
2. Move AccountManagerTrait implementation from mod.rs
3. Keep trait definition in mod.rs for now (will move in Step 3)
4. Add necessary imports
5. Ensure all trait methods are implemented

**Code to Extract** (from mod.rs, lines ~500-1300):
- The entire `impl AccountManagerTrait for AccountManager` block
- All helper methods used by the trait implementation
- Internal state management methods

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/coordinator.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run tests
cargo test --all-features account_manager
```

**Expected Results**:
- coordinator.rs: 300-400 lines
- All tests pass
- No compilation errors

**Rollback Procedure**:
```powershell
# If anything fails
git checkout -- src/wallet/account_manager/coordinator.rs
git checkout -- src/wallet/account_manager/mod.rs
```

#### Step 2: Create lifecycle.rs (CRUD operations)
**Estimated Lines**: ~200 lines
**Purpose**: Extract account lifecycle operations (create, list, get, remove)

**Actions**:
1. Create `src/wallet/account_manager/lifecycle.rs`
2. Extract CRUD operation implementations from coordinator.rs
3. Create public functions for each operation
4. Update coordinator.rs to call lifecycle.rs functions

**Code to Extract** (from coordinator.rs after Step 1):
- `create_account()` implementation
- `list_accounts()` implementation
- `get_account()` implementation
- `remove_account()` implementation
- Helper functions for account creation/removal

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/lifecycle.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run lifecycle tests
cargo test --all-features account_manager::lifecycle
```

**Expected Results**:
- lifecycle.rs: 150-200 lines
- coordinator.rs reduced by ~200 lines
- All tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/wallet/account_manager/lifecycle.rs
git checkout -- src/wallet/account_manager/coordinator.rs
```

#### Step 3: Create auth.rs (Authentication operations)
**Estimated Lines**: ~150 lines
**Purpose**: Extract authentication and session management operations

**Actions**:
1. Create `src/wallet/account_manager/auth.rs`
2. Extract lock/unlock implementations from coordinator.rs
3. Extract authentication token management
4. Update coordinator.rs to call auth.rs functions

**Code to Extract** (from coordinator.rs):
- `lock()` implementation
- `unlock()` implementation
- `is_locked()` implementation
- `generate_auth_token()` implementation
- `validate_auth_token()` implementation
- Session management helpers

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/auth.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run auth tests
cargo test --all-features account_manager::auth
cargo test --all-features lock_property_tests
```

**Expected Results**:
- auth.rs: 100-150 lines
- coordinator.rs reduced by ~150 lines
- All lock/unlock tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/wallet/account_manager/auth.rs
git checkout -- src/wallet/account_manager/coordinator.rs
```

#### Step 4: Update mod.rs to re-export from submodules
**Estimated Lines**: ~50 lines (target)
**Purpose**: Convert mod.rs to a pure re-export module

**Actions**:
1. Remove all code from mod.rs except:
   - Module declarations
   - Re-exports (pub use)
   - Trait definition (AccountManagerTrait)
2. Add re-exports for coordinator, lifecycle, auth
3. Ensure all public APIs remain accessible

**Final mod.rs Structure**:
```rust
//! Unified Account Manager Interface
//! (documentation)

// Module declarations
pub mod creation;
pub mod import;
pub mod export;
pub mod metadata;
pub mod signer_integration;
pub mod discovery;
pub mod eip712;
pub mod types;
pub mod coordinator;
pub mod lifecycle;
pub mod auth;

// Re-exports
pub use types::*;
pub use coordinator::*;
pub use lifecycle::*;
pub use auth::*;
pub use creation::*;
pub use import::*;
pub use export::*;

// Trait definition (stays in mod.rs as the public interface)
#[async_trait]
pub trait AccountManagerTrait: Send + Sync {
    // ... trait methods
}
```

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/mod.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run ALL account_manager tests
cargo test --all-features account_manager
```

**Expected Results**:
- mod.rs: 40-60 lines (mostly re-exports)
- All 399 tests pass
- No compilation errors
- All public APIs still accessible

**Rollback Procedure**:
```powershell
# Complete rollback of Task 2.1
git checkout -- src/wallet/account_manager/
```

#### Step 5: Final Validation for Task 2.1
**Purpose**: Comprehensive validation of all changes

**Validation Checklist**:
- [ ] mod.rs < 60 lines
- [ ] coordinator.rs < 400 lines
- [ ] lifecycle.rs < 200 lines
- [ ] auth.rs < 200 lines
- [ ] types.rs < 250 lines (already created)
- [ ] All 399 tests pass
- [ ] No compilation warnings
- [ ] No functionality lost
- [ ] All public APIs accessible

**Commands**:
```powershell
# Check all module sizes
Get-Content "Vaughan-main/src/wallet/account_manager/mod.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/coordinator.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/lifecycle.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/auth.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/types.rs" | Measure-Object -Line

# Run full test suite
cargo test --all-features

# Check for warnings
cargo check --all-features
cargo clippy --all-features
```

**Success Criteria**:
- ✅ All modules under size limits
- ✅ All tests passing
- ✅ Zero compilation errors
- ✅ Zero new warnings

**If Validation Fails**:
1. Document the failure
2. Rollback: `git checkout -- src/wallet/account_manager/`
3. Analyze the issue
4. Retry with corrections

---

## Task 2.2: Refactor account_manager/import.rs (883 lines → ~200 lines per module)

### Overview
Split the import.rs module into focused submodules for parsing, validation, and conversion.

### Current Structure Analysis
```rust
// Current import.rs contains:
- Format detection logic (~200 lines)
- Parsing implementations (~300 lines): seed phrases, private keys, keystores
- Validation logic (~200 lines): format validation, security checks
- Conversion logic (~150 lines): converting between formats
- Helper functions (~30 lines)
```

### Target Structure
```
src/wallet/account_manager/import/
├── mod.rs                    (~50 lines - re-exports and coordination)
├── parsers.rs                (~200 lines - format parsing)
├── validators.rs             (~150 lines - validation logic)
└── converters.rs             (~150 lines - format conversion)
```

### Step-by-Step Implementation

#### Step 1: Create import/ directory and mod.rs
**Actions**:
1. Create directory: `src/wallet/account_manager/import/`
2. Move current `import.rs` to `import/legacy.rs` (temporary backup)
3. Create `import/mod.rs` with re-exports

**Commands**:
```powershell
# Create directory
New-Item -ItemType Directory -Path "Vaughan-main/src/wallet/account_manager/import" -Force

# Backup current import.rs
Copy-Item "Vaughan-main/src/wallet/account_manager/import.rs" "Vaughan-main/src/wallet/account_manager/import/legacy.rs"
```

#### Step 2: Create parsers.rs (Format parsing)
**Estimated Lines**: ~200 lines
**Purpose**: Extract all format parsing logic

**Code to Extract** (from legacy.rs):
- Seed phrase parsing functions
- Private key parsing functions
- Keystore file parsing functions
- Format detection logic
- Parsing error handling

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/import/parsers.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run parser tests
cargo test --all-features import::parsers
```

**Expected Results**:
- parsers.rs: 180-220 lines
- All parsing tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/wallet/account_manager/import/
```

#### Step 3: Create validators.rs (Validation logic)
**Estimated Lines**: ~150 lines
**Purpose**: Extract all validation logic

**Code to Extract** (from legacy.rs):
- Format validation functions
- Security validation (key strength, etc.)
- Input sanitization
- Validation error types
- Validation helper functions

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/import/validators.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run validation tests
cargo test --all-features import::validators
```

**Expected Results**:
- validators.rs: 130-170 lines
- All validation tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/wallet/account_manager/import/
```

#### Step 4: Create converters.rs (Format conversion)
**Estimated Lines**: ~150 lines
**Purpose**: Extract format conversion logic

**Code to Extract** (from legacy.rs):
- Format conversion functions
- Type conversion helpers
- Encoding/decoding utilities
- Conversion error handling

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/import/converters.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run conversion tests
cargo test --all-features import::converters
```

**Expected Results**:
- converters.rs: 130-170 lines
- All conversion tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/wallet/account_manager/import/
```

#### Step 5: Create import/mod.rs (Coordination)
**Estimated Lines**: ~50 lines
**Purpose**: Coordinate the import submodules and provide public API

**Final Structure**:
```rust
//! Account Import Module
//! (documentation)

pub mod parsers;
pub mod validators;
pub mod converters;

// Re-exports
pub use parsers::*;
pub use validators::*;
pub use converters::*;

// Public types (if any remain here)
// Main coordination logic (if needed)
```

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/wallet/account_manager/import/mod.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run ALL import tests
cargo test --all-features import
```

**Expected Results**:
- mod.rs: 40-60 lines
- All import tests pass
- Public API unchanged

#### Step 6: Remove legacy.rs and update parent mod.rs
**Actions**:
1. Delete `import/legacy.rs` (backup no longer needed)
2. Update `account_manager/mod.rs` to reference `import` as a directory module

**Commands**:
```powershell
# Remove backup
Remove-Item "Vaughan-main/src/wallet/account_manager/import/legacy.rs"

# Verify import.rs is gone and import/ directory exists
Test-Path "Vaughan-main/src/wallet/account_manager/import.rs"  # Should be False
Test-Path "Vaughan-main/src/wallet/account_manager/import/"    # Should be True
```

#### Step 7: Final Validation for Task 2.2
**Validation Checklist**:
- [ ] import/mod.rs < 60 lines
- [ ] import/parsers.rs < 220 lines
- [ ] import/validators.rs < 170 lines
- [ ] import/converters.rs < 170 lines
- [ ] All import tests pass
- [ ] No compilation warnings
- [ ] Public API unchanged

**Commands**:
```powershell
# Check all module sizes
Get-Content "Vaughan-main/src/wallet/account_manager/import/mod.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/import/parsers.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/import/validators.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/wallet/account_manager/import/converters.rs" | Measure-Object -Line

# Run full test suite
cargo test --all-features

# Check for warnings
cargo check --all-features
```

**Success Criteria**:
- ✅ All modules under 220 lines
- ✅ All tests passing
- ✅ Zero compilation errors

**If Validation Fails**:
1. Restore from backup: `Copy-Item "Vaughan-main/src/wallet/account_manager/import/legacy.rs" "Vaughan-main/src/wallet/account_manager/import.rs"`
2. Remove directory: `Remove-Item -Recurse "Vaughan-main/src/wallet/account_manager/import/"`
3. Analyze and retry

---

## Task 2.3: Refactor performance/batch.rs (774 lines → ~200 lines per module)

### Overview
Split the batch processing module into configuration, processor, and retry logic.

### Current Structure Analysis
```rust
// Current batch.rs contains:
- Configuration structs (~100 lines)
- Core batch processor (~300 lines)
- Retry/backoff logic (~150 lines)
- Error handling (~100 lines)
- Helper functions (~100 lines)
```

### Target Structure
```
src/performance/batch/
├── mod.rs                    (~50 lines - re-exports and coordination)
├── config.rs                 (~100 lines - configuration structs)
├── processor.rs              (~200 lines - core processing logic)
└── retry.rs                  (~150 lines - retry/backoff logic)
```

### Step-by-Step Implementation

#### Step 1: Create batch/ directory and backup
**Actions**:
1. Create directory: `src/performance/batch/`
2. Move current `batch.rs` to `batch/legacy.rs` (temporary backup)
3. Create `batch/mod.rs` with re-exports

**Commands**:
```powershell
# Create directory
New-Item -ItemType Directory -Path "Vaughan-main/src/performance/batch" -Force

# Backup current batch.rs
Copy-Item "Vaughan-main/src/performance/batch.rs" "Vaughan-main/src/performance/batch/legacy.rs"
```

#### Step 2: Create config.rs (Configuration)
**Estimated Lines**: ~100 lines
**Purpose**: Extract all configuration structs and builders

**Code to Extract** (from legacy.rs):
- BatchConfig struct
- BatchOptions struct
- Configuration builders
- Default implementations
- Configuration validation

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/performance/batch/config.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run config tests
cargo test --all-features batch::config
```

**Expected Results**:
- config.rs: 80-120 lines
- All config tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/performance/batch/
```

#### Step 3: Create processor.rs (Core processing logic)
**Estimated Lines**: ~200 lines
**Purpose**: Extract core batch processing implementation

**Code to Extract** (from legacy.rs):
- BatchProcessor struct
- Main processing loop
- Batch execution logic
- Result aggregation
- Error collection
- Processing state management

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/performance/batch/processor.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run processor tests
cargo test --all-features batch::processor
```

**Expected Results**:
- processor.rs: 180-220 lines
- All processor tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/performance/batch/
```

#### Step 4: Create retry.rs (Retry/backoff logic)
**Estimated Lines**: ~150 lines
**Purpose**: Extract retry and backoff strategies

**Code to Extract** (from legacy.rs):
- Retry strategy implementations
- Backoff algorithms (exponential, linear)
- Retry state management
- Retry error handling
- Timeout logic

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/performance/batch/retry.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run retry tests
cargo test --all-features batch::retry
```

**Expected Results**:
- retry.rs: 130-170 lines
- All retry tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/performance/batch/
```

#### Step 5: Create batch/mod.rs (Coordination)
**Estimated Lines**: ~50 lines
**Purpose**: Coordinate batch submodules and provide public API

**Final Structure**:
```rust
//! Batch Processing Module
//! (documentation)

pub mod config;
pub mod processor;
pub mod retry;

// Re-exports
pub use config::*;
pub use processor::*;
pub use retry::*;

// Public API coordination (if needed)
```

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/performance/batch/mod.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features

# Run ALL batch tests
cargo test --all-features batch
```

**Expected Results**:
- mod.rs: 40-60 lines
- All batch tests pass
- Public API unchanged

#### Step 6: Remove legacy.rs and update parent mod.rs
**Actions**:
1. Delete `batch/legacy.rs` (backup no longer needed)
2. Update `performance/mod.rs` to reference `batch` as a directory module

**Commands**:
```powershell
# Remove backup
Remove-Item "Vaughan-main/src/performance/batch/legacy.rs"

# Verify batch.rs is gone and batch/ directory exists
Test-Path "Vaughan-main/src/performance/batch.rs"  # Should be False
Test-Path "Vaughan-main/src/performance/batch/"    # Should be True
```

#### Step 7: Run Performance Benchmarks
**Purpose**: Verify no performance regression

**Commands**:
```powershell
# Run batch benchmarks
cargo bench --bench account_manager_benchmarks

# Compare with baseline (if available)
# Expected: 244-270% improvement maintained
```

**Expected Results**:
- Batch operations: 244-270% improvement maintained
- No performance regression >5%

#### Step 8: Final Validation for Task 2.3
**Validation Checklist**:
- [ ] batch/mod.rs < 60 lines
- [ ] batch/config.rs < 120 lines
- [ ] batch/processor.rs < 220 lines
- [ ] batch/retry.rs < 170 lines
- [ ] All batch tests pass
- [ ] No performance regression
- [ ] No compilation warnings

**Commands**:
```powershell
# Check all module sizes
Get-Content "Vaughan-main/src/performance/batch/mod.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/performance/batch/config.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/performance/batch/processor.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/performance/batch/retry.rs" | Measure-Object -Line

# Run full test suite
cargo test --all-features

# Run benchmarks
cargo bench --bench account_manager_benchmarks

# Check for warnings
cargo check --all-features
```

**Success Criteria**:
- ✅ All modules under 220 lines
- ✅ All tests passing
- ✅ No performance regression
- ✅ Zero compilation errors

**If Validation Fails**:
1. Restore from backup: `Copy-Item "Vaughan-main/src/performance/batch/legacy.rs" "Vaughan-main/src/performance/batch.rs"`
2. Remove directory: `Remove-Item -Recurse "Vaughan-main/src/performance/batch/"`
3. Analyze and retry

---

## Task 2.4: Refactor telemetry/account_events.rs (726 lines → ~200 lines per module)

### Overview
Split the telemetry module into logger, spans, and privacy filtering.

### Current Structure Analysis
```rust
// Current account_events.rs contains:
- Logging operations (~200 lines)
- Span management (~200 lines)
- Privacy filtering (~150 lines)
- Event types (~100 lines)
- Helper functions (~75 lines)
```

### Target Structure
```
src/telemetry/account_events/
├── mod.rs                    (~50 lines - re-exports and coordination)
├── logger.rs                 (~150 lines - logging operations)
├── spans.rs                  (~150 lines - span management)
└── privacy.rs                (~100 lines - privacy filtering)
```

### Step-by-Step Implementation

#### Step 1: Create account_events/ directory and backup
**Actions**:
1. Create directory: `src/telemetry/account_events/`
2. Move current `account_events.rs` to `account_events/legacy.rs` (temporary backup)
3. Create `account_events/mod.rs` with re-exports

**Commands**:
```powershell
# Create directory
New-Item -ItemType Directory -Path "Vaughan-main/src/telemetry/account_events" -Force

# Backup current account_events.rs
Copy-Item "Vaughan-main/src/telemetry/account_events.rs" "Vaughan-main/src/telemetry/account_events/legacy.rs"
```

#### Step 2: Create logger.rs (Logging operations)
**Estimated Lines**: ~150 lines
**Purpose**: Extract all logging operations

**Code to Extract** (from legacy.rs):
- Log event functions
- Log level management
- Log formatting
- Log output handling
- Structured logging helpers

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/telemetry/account_events/logger.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features --features telemetry

# Run logger tests
cargo test --all-features --features telemetry account_events::logger
```

**Expected Results**:
- logger.rs: 130-170 lines
- All logger tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/telemetry/account_events/
```

#### Step 3: Create spans.rs (Span management)
**Estimated Lines**: ~150 lines
**Purpose**: Extract OpenTelemetry span management

**Code to Extract** (from legacy.rs):
- Span creation functions
- Span context management
- Span attributes
- Span lifecycle management
- Nested span handling

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/telemetry/account_events/spans.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features --features telemetry

# Run span tests
cargo test --all-features --features telemetry account_events::spans
```

**Expected Results**:
- spans.rs: 130-170 lines
- All span tests pass

**Rollback Procedure**:
```powershell
git checkout -- src/telemetry/account_events/
```

#### Step 4: Create privacy.rs (Privacy filtering)
**Estimated Lines**: ~100 lines
**Purpose**: Extract privacy filtering logic

**Code to Extract** (from legacy.rs):
- PII filtering functions
- Sensitive data redaction
- Privacy policy enforcement
- Safe logging helpers
- Privacy validation

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/telemetry/account_events/privacy.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features --features telemetry

# Run privacy tests
cargo test --all-features --features telemetry account_events::privacy
```

**Expected Results**:
- privacy.rs: 80-120 lines
- All privacy tests pass
- No PII leakage in logs

**Rollback Procedure**:
```powershell
git checkout -- src/telemetry/account_events/
```

#### Step 5: Create account_events/mod.rs (Coordination)
**Estimated Lines**: ~50 lines
**Purpose**: Coordinate telemetry submodules and provide public API

**Final Structure**:
```rust
//! Account Events Telemetry Module
//! (documentation)

pub mod logger;
pub mod spans;
pub mod privacy;

// Re-exports
pub use logger::*;
pub use spans::*;
pub use privacy::*;

// Event types (if any remain here)
// Public API coordination
```

**Validation Commands**:
```powershell
# Check line count
Get-Content "Vaughan-main/src/telemetry/account_events/mod.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines

# Verify compilation
cargo check --all-features --features telemetry

# Run ALL telemetry tests
cargo test --all-features --features telemetry account_events
```

**Expected Results**:
- mod.rs: 40-60 lines
- All telemetry tests pass
- Public API unchanged

#### Step 6: Remove legacy.rs and update parent mod.rs
**Actions**:
1. Delete `account_events/legacy.rs` (backup no longer needed)
2. Update `telemetry/mod.rs` to reference `account_events` as a directory module

**Commands**:
```powershell
# Remove backup
Remove-Item "Vaughan-main/src/telemetry/account_events/legacy.rs"

# Verify account_events.rs is gone and account_events/ directory exists
Test-Path "Vaughan-main/src/telemetry/account_events.rs"  # Should be False
Test-Path "Vaughan-main/src/telemetry/account_events/"    # Should be True
```

#### Step 7: Final Validation for Task 2.4
**Validation Checklist**:
- [ ] account_events/mod.rs < 60 lines
- [ ] account_events/logger.rs < 170 lines
- [ ] account_events/spans.rs < 170 lines
- [ ] account_events/privacy.rs < 120 lines
- [ ] All telemetry tests pass
- [ ] No PII leakage in logs
- [ ] No compilation warnings

**Commands**:
```powershell
# Check all module sizes
Get-Content "Vaughan-main/src/telemetry/account_events/mod.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/telemetry/account_events/logger.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/telemetry/account_events/spans.rs" | Measure-Object -Line
Get-Content "Vaughan-main/src/telemetry/account_events/privacy.rs" | Measure-Object -Line

# Run full test suite with telemetry
cargo test --all-features --features telemetry

# Check for warnings
cargo check --all-features --features telemetry
```

**Success Criteria**:
- ✅ All modules under 170 lines
- ✅ All tests passing
- ✅ Privacy guarantees maintained
- ✅ Zero compilation errors

**If Validation Fails**:
1. Restore from backup: `Copy-Item "Vaughan-main/src/telemetry/account_events/legacy.rs" "Vaughan-main/src/telemetry/account_events.rs"`
2. Remove directory: `Remove-Item -Recurse "Vaughan-main/src/telemetry/account_events/"`
3. Analyze and retry

---

## Task 2.5: Analyze and Refactor account_manager/metadata.rs (250 lines)

### Overview
Analyze metadata.rs to determine if refactoring is needed. At 250 lines, it's only 25% over the 200-line limit.

### Current State
- **Current Size**: 250 lines
- **Over Limit By**: 50 lines (25%)
- **Decision**: Analyze structure first, refactor only if beneficial

### Analysis Steps

#### Step 1: Analyze Module Structure
**Purpose**: Understand what the module contains and if it's cohesive

**Commands**:
```powershell
# View the file structure
Get-Content "Vaughan-main/src/wallet/account_manager/metadata.rs" | Select-String "^(pub )?(struct|enum|trait|impl|fn|mod)" | Select-Object -First 30

# Count different types of items
$content = Get-Content "Vaughan-main/src/wallet/account_manager/metadata.rs"
$structs = ($content | Select-String "^(pub )?struct").Count
$enums = ($content | Select-String "^(pub )?enum").Count
$impls = ($content | Select-String "^(pub )?impl").Count
$fns = ($content | Select-String "^(pub )?fn").Count

Write-Host "Structs: $structs"
Write-Host "Enums: $enums"
Write-Host "Impls: $impls"
Write-Host "Functions: $fns"
```

#### Step 2: Decision Criteria
**Refactor if**:
- Module contains multiple distinct responsibilities
- Clear separation points exist (e.g., storage vs. operations)
- Would improve testability significantly
- Contains >3 major struct implementations

**Keep as-is if**:
- Module is cohesive (single responsibility)
- No clear separation points
- Refactoring would add complexity without benefit
- Already well-organized

#### Step 3a: If Refactoring (Conditional)
**Potential Target Structure** (if needed):
```
src/wallet/account_manager/metadata/
├── mod.rs                    (~50 lines - re-exports)
├── types.rs                  (~100 lines - metadata types)
└── operations.rs             (~100 lines - metadata operations)
```

**Follow similar pattern as previous tasks**:
1. Create metadata/ directory
2. Backup current metadata.rs
3. Split into types and operations
4. Create mod.rs with re-exports
5. Validate and test

#### Step 3b: If Keeping As-Is (Likely)
**Justification Document**:
Create a brief note explaining why metadata.rs remains as a single file:
- Cohesive single responsibility
- Only 25% over limit (acceptable for cohesive modules)
- No clear benefit from splitting
- Would add unnecessary complexity

**Validation**:
```powershell
# Verify tests still pass
cargo test --all-features metadata

# Check for warnings
cargo check --all-features
```

### Final Decision for Task 2.5
**Recommendation**: Analyze first, likely keep as-is unless clear separation exists.

**Validation Checklist**:
- [ ] Module structure analyzed
- [ ] Decision documented (refactor or keep)
- [ ] If refactored: all submodules < 200 lines
- [ ] If kept: justification documented
- [ ] All metadata tests pass
- [ ] No compilation warnings

**Commands**:
```powershell
# Final validation
cargo test --all-features metadata
cargo check --all-features
```

---

## Phase 2 Final Validation

### Comprehensive Validation Checklist

After completing all tasks (2.1 through 2.5), perform comprehensive validation:

#### Module Size Validation
```powershell
# Check all refactored modules
$modules = @(
    "Vaughan-main/src/wallet/account_manager/mod.rs",
    "Vaughan-main/src/wallet/account_manager/types.rs",
    "Vaughan-main/src/wallet/account_manager/coordinator.rs",
    "Vaughan-main/src/wallet/account_manager/lifecycle.rs",
    "Vaughan-main/src/wallet/account_manager/auth.rs",
    "Vaughan-main/src/wallet/account_manager/import/mod.rs",
    "Vaughan-main/src/wallet/account_manager/import/parsers.rs",
    "Vaughan-main/src/wallet/account_manager/import/validators.rs",
    "Vaughan-main/src/wallet/account_manager/import/converters.rs",
    "Vaughan-main/src/performance/batch/mod.rs",
    "Vaughan-main/src/performance/batch/config.rs",
    "Vaughan-main/src/performance/batch/processor.rs",
    "Vaughan-main/src/performance/batch/retry.rs",
    "Vaughan-main/src/telemetry/account_events/mod.rs",
    "Vaughan-main/src/telemetry/account_events/logger.rs",
    "Vaughan-main/src/telemetry/account_events/spans.rs",
    "Vaughan-main/src/telemetry/account_events/privacy.rs",
    "Vaughan-main/src/wallet/account_manager/metadata.rs"
)

foreach ($module in $modules) {
    if (Test-Path $module) {
        $lines = (Get-Content $module | Measure-Object -Line).Lines
        $name = Split-Path $module -Leaf
        Write-Host "$name : $lines lines"
    }
}
```

**Expected Results**:
- All mod.rs files: < 60 lines
- All coordinator files: < 400 lines
- All logic files: < 220 lines

#### Test Suite Validation
```powershell
# Run complete test suite
cargo test --all-features

# Expected: All 399+ tests pass
```

#### Compilation Validation
```powershell
# Check for warnings
cargo check --all-features

# Run clippy
cargo clippy --all-features

# Expected: Zero warnings, zero errors
```

#### Performance Validation
```powershell
# Run benchmarks (if available)
cargo bench --bench account_manager_benchmarks
cargo bench --bench wallet_benchmarks

# Expected: No regression >5%
```

#### Feature Flag Validation
```powershell
# Test with minimal features
cargo test --no-default-features --features minimal

# Test with default features
cargo test

# Test with all features
cargo test --all-features

# Expected: All configurations pass
```

### Success Criteria Summary

**Phase 2 is complete when**:
- ✅ All modules under size limits (400/200 lines)
- ✅ All 399+ tests passing
- ✅ Zero compilation errors
- ✅ Zero new warnings
- ✅ No performance regression
- ✅ All feature flag combinations work
- ✅ Public APIs unchanged
- ✅ Documentation updated

### Rollback Strategy

**If Phase 2 validation fails**:
```powershell
# Complete rollback to pre-Phase 2 state
git checkout -- src/wallet/account_manager/
git checkout -- src/performance/batch/
git checkout -- src/telemetry/account_events/

# Verify rollback successful
cargo test --all-features
```

### Documentation Updates

After successful completion, update:
1. `tasks.md` - Mark all Phase 2 tasks as complete [x]
2. Create `PHASE2_COMPLETE.md` - Document completion and metrics
3. Update `README.md` - If module structure changed significantly

---

## Execution Timeline

### Recommended Order
1. **Task 2.1**: account_manager/mod.rs (Most complex, highest priority)
   - Estimated time: 4-6 hours
   - Complexity: High
   - Risk: Medium

2. **Task 2.2**: account_manager/import.rs (Medium complexity)
   - Estimated time: 2-3 hours
   - Complexity: Medium
   - Risk: Low

3. **Task 2.3**: performance/batch.rs (Performance-sensitive)
   - Estimated time: 2-3 hours
   - Complexity: Medium
   - Risk: Medium (performance regression possible)

4. **Task 2.4**: telemetry/account_events.rs (Feature-gated)
   - Estimated time: 2-3 hours
   - Complexity: Low
   - Risk: Low

5. **Task 2.5**: metadata.rs analysis (Quick analysis)
   - Estimated time: 30 minutes - 1 hour
   - Complexity: Low
   - Risk: Very Low

**Total Estimated Time**: 11-16 hours

### Daily Breakdown (Recommended)
**Day 1**: Task 2.1 (account_manager/mod.rs)
- Morning: Steps 1-2 (coordinator.rs, lifecycle.rs)
- Afternoon: Steps 3-5 (auth.rs, mod.rs update, validation)

**Day 2**: Tasks 2.2 and 2.3
- Morning: Task 2.2 (import.rs refactoring)
- Afternoon: Task 2.3 (batch.rs refactoring)

**Day 3**: Tasks 2.4, 2.5, and Final Validation
- Morning: Task 2.4 (telemetry refactoring)
- Midday: Task 2.5 (metadata analysis)
- Afternoon: Phase 2 final validation

## Risk Mitigation

### High-Risk Areas
1. **AccountManagerTrait implementation** (Task 2.1)
   - Risk: Breaking trait implementation
   - Mitigation: Extensive testing after each step
   - Rollback: Immediate if tests fail

2. **Performance regression** (Task 2.3)
   - Risk: Batch processing slowdown
   - Mitigation: Run benchmarks after changes
   - Rollback: If >5% regression detected

3. **Public API changes** (All tasks)
   - Risk: Breaking downstream code
   - Mitigation: Preserve all public exports
   - Rollback: If compilation fails elsewhere

### Low-Risk Areas
1. **Telemetry refactoring** (Task 2.4)
   - Feature-gated, limited impact
   - Easy to test in isolation

2. **Metadata analysis** (Task 2.5)
   - Small module, may not need refactoring
   - Low complexity

## Common Pitfalls to Avoid

### Import Management
- ❌ **Don't**: Forget to update re-exports in mod.rs
- ✅ **Do**: Update all re-exports immediately after creating submodules

### Test Coverage
- ❌ **Don't**: Assume tests will pass without running them
- ✅ **Do**: Run tests after each step, not just at the end

### Module Visibility
- ❌ **Don't**: Change pub/pub(crate) visibility accidentally
- ✅ **Do**: Preserve all existing visibility modifiers

### Feature Flags
- ❌ **Don't**: Forget to test with different feature combinations
- ✅ **Do**: Test minimal, default, and all-features configurations

### Documentation
- ❌ **Don't**: Remove module-level documentation
- ✅ **Do**: Preserve and update all documentation comments

## Troubleshooting Guide

### Problem: Tests fail after refactoring
**Solution**:
1. Check import paths in test modules
2. Verify all re-exports are correct
3. Check for missing pub modifiers
4. Rollback and retry with smaller steps

### Problem: Compilation errors in other modules
**Solution**:
1. Check if public API changed
2. Update import paths in dependent modules
3. Verify re-exports are complete
4. Check for circular dependencies

### Problem: Module still too large after split
**Solution**:
1. Identify largest remaining sections
2. Consider additional submodules
3. Extract helper functions to separate files
4. Document justification if keeping >200 lines

### Problem: Performance regression detected
**Solution**:
1. Profile to identify bottleneck
2. Check for unnecessary allocations
3. Verify inlining hints preserved
4. Consider reverting specific changes
5. Document performance trade-offs

## Post-Phase 2 Actions

### Immediate Actions
1. Update tasks.md with completion status
2. Create PHASE2_COMPLETE.md document
3. Commit changes with descriptive message
4. Tag commit: `phase2-complete`

### Documentation Updates
1. Update module documentation
2. Update architecture diagrams (if any)
3. Update developer guide
4. Update CHANGELOG.md

### Preparation for Phase 3
1. Review Phase 3 requirements
2. Identify properties to implement
3. Plan property test implementation
4. Estimate Phase 3 timeline

---

## Appendix: Quick Reference Commands

### Line Counting
```powershell
Get-Content "path/to/file.rs" | Measure-Object -Line | Select-Object -ExpandProperty Lines
```

### Test Running
```powershell
# All tests
cargo test --all-features

# Specific module
cargo test --all-features module_name

# With output
cargo test --all-features -- --nocapture
```

### Compilation Checking
```powershell
# Check compilation
cargo check --all-features

# Check with clippy
cargo clippy --all-features

# Check specific features
cargo check --no-default-features --features minimal
```

### Benchmarking
```powershell
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench account_manager_benchmarks
```

### Git Operations
```powershell
# Create backup branch
git branch phase2-backup

# Commit progress
git add .
git commit -m "Phase 2: Task X.Y complete"

# Rollback
git checkout -- path/to/file.rs
```

---

**End of Phase 2 Implementation Plan**

This plan provides a complete, step-by-step guide for executing Phase 2 of the professional wallet improvement initiative. Follow each step carefully, validate frequently, and don't hesitate to rollback if issues arise.
