# Requirements Document: Comprehensive Debloat

## Introduction

This specification defines a systematic approach to reducing code complexity, binary size, and technical debt in the Vaughan wallet application. This enhanced version adds additional opportunities discovered during codebase analysis, including new large files to decompose (dialogs.rs, keystore.rs), better feature gating strategies, and improved consolidation opportunities. The plan follows industry best practices from MetaMask's codebase organization and Alloy library patterns.

## New Discoveries Summary

### Additional Large Files (>1000 Lines)

| File | Lines | Issue | Priority |
|------|-------|-------|----------|
| **dialogs.rs** | 1,327 | Contains multiple dialog views (custom token, various dialogs) | **HIGH** |
| **keystore.rs** | 1,110 | Account storage, keychain integration | **HIGH** |
| hardware.rs | 1,703 | Future-planned features with dead_code | MEDIUM |
| keychain.rs | 797 | Close to threshold, consider consolidation | LOW |

### Additional Opportunities

1. **Handler Pattern Recognition**: The existing `src/gui/handlers/` directory demonstrates excellent organization that should be extended to working_wallet.rs decomposition
2. **Feature Gating**: Several components (hardware wallet, professional monitoring, custom tokens) are not properly gated with Cargo features
3. **Password Dialog Duplication**: Three separate password dialog files with significant duplication that can be consolidated
4. **State Overlap**: security_state.rs and session_state.rs have overlapping responsibilities

## Glossary

- **Vaughan_Wallet**: The multi-EVM cryptocurrency wallet application being debloated
- **Dead_Code**: Code annotated with `#[allow(dead_code)]` or identified as unreachable
- **Bloat_File**: A source file exceeding 1,000 lines of code requiring decomposition
- **Module_Consolidation**: Merging related files into a single, configurable component
- **Binary_Size**: The size of the compiled release binary in megabytes
- **LOC**: Lines of Code metric for measuring codebase size
- **Test_File**: Files containing test code that should reside in `tests/` directory
- **Dependency_Optimization**: Reducing unused features and consolidating crate versions
- **Feature_Gating**: Using Cargo features to conditionally compile optional components

## Requirements

### Requirement 1: Dead Code Elimination (Enhanced)

**User Story:** As a maintainer, I want all dead code removed or properly documented, so that the codebase is clean and every line serves a purpose.

#### Acceptance Criteria

1. WHEN auditing dead code annotations, THE Vaughan_Wallet codebase SHALL have zero `#[allow(dead_code)]` annotations without documented justification
2. WHEN a dead code annotation exists for serde deserialization fields, THE Vaughan_Wallet codebase SHALL retain the annotation with a comment explaining its necessity
3. WHEN a dead code annotation exists for planned features, THE Vaughan_Wallet codebase SHALL either implement the feature within 3 months, document the exact timeline, or remove the code entirely
4. WHEN a dead code annotation exists for unused handlers, THE Vaughan_Wallet codebase SHALL remove the handler code completely
5. IF dead code is discovered without annotation, THEN THE Vaughan_Wallet codebase SHALL remove it or document why it must remain
6. WHEN auditing dead code in security-critical modules (keystore.rs, seed.rs), THE Vaughan_Wallet codebase SHALL document the security reason for retention or implement a zeroization strategy
7. WHEN dead code annotations exist for future keychain operations, THE Vaughan_Wallet codebase SHALL either remove them or document the specific roadmap milestone for implementation

### Requirement 2: Large File Decomposition - dialogs.rs

**User Story:** As a UI developer, I want the monolithic dialogs.rs file split into logical components, so that I can navigate and maintain dialog code efficiently.

#### Acceptance Criteria

1. WHEN decomposing dialogs.rs, THE Vaughan_Wallet codebase SHALL create a `src/gui/components/dialogs/` directory structure
2. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create custom_token.rs for custom token dialog management
3. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create confirmation.rs for various confirmation dialogs
4. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create receive.rs for receive dialog logic (extracted from dialogs.rs)
5. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create mod.rs with exports and common dialog types
6. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL ensure no individual dialog module exceeds 600 lines
7. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL compile without errors and pass all existing tests

### Requirement 3: Large File Decomposition - keystore.rs

**User Story:** As a security engineer, I want the keystore.rs file split by responsibility, so that account storage, encryption, and keychain operations are isolated and auditable.

#### Acceptance Criteria

1. WHEN decomposing keystore.rs, THE Vaughan_Wallet codebase SHALL create a `src/security/keystore/` directory structure
2. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create mod.rs with public Keystore trait and exports
3. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create storage.rs for file-based persistence operations
4. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create encryption.rs for key encryption/decryption logic
5. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create account.rs for account management operations
6. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create network.rs for custom network storage
7. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL maintain all existing security guarantees
8. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL pass all security-related tests

### Requirement 4: Large File Decomposition - working_wallet.rs (Enhanced)

**User Story:** As a developer, I want the monolithic working_wallet.rs split into logical modules following the handler pattern, so that I can navigate and maintain the GUI code efficiently.

#### Acceptance Criteria

1. WHEN decomposing working_wallet.rs, THE Vaughan_Wallet codebase SHALL create a `src/gui/wallet/` directory structure
2. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL move existing handlers to `wallet/handlers/` following the established pattern
3. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create mod.rs containing the main WalletState struct and Message enum
4. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create update.rs containing message routing to handlers
5. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create view.rs containing main view composition
6. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create components/send.rs containing send transaction logic and UI
7. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create components/receive.rs containing receive and QR logic
8. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create components/tokens.rs containing token management logic
9. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL ensure no individual module exceeds 1,000 lines
10. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL compile without errors and pass all existing tests

### Requirement 5: Large File Decomposition - seed.rs (Enhanced)

**User Story:** As a security engineer, I want the seed.rs file split by responsibility, so that security-critical code is isolated and auditable.

#### Acceptance Criteria

1. WHEN decomposing seed.rs, THE Vaughan_Wallet codebase SHALL create a `src/security/seed/` directory structure
2. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create mod.rs containing the public API and re-exports
3. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create encryption.rs containing seed encryption and decryption logic
4. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create derivation.rs containing key derivation logic
5. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create backup.rs containing backup and restore functionality
6. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create validation.rs containing seed phrase validation
7. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create zeroization.rs for memory zeroization utilities
8. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL maintain all existing security guarantees
9. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL pass all security-related tests
10. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL ensure all seed data is properly zeroized after use

### Requirement 6: Large File Decomposition - theme.rs (Enhanced)

**User Story:** As a UI developer, I want theme definitions organized by component type, so that I can easily find and modify styles.

#### Acceptance Criteria

1. WHEN decomposing theme.rs, THE Vaughan_Wallet codebase SHALL create a `src/gui/theme/` directory structure
2. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create colors.rs containing all color definitions
3. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create buttons.rs containing button style definitions
4. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create containers.rs containing container style definitions
5. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL create text.rs containing text style definitions
6. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL identify and document unused style definitions
7. WHEN splitting the file, THE Vaughan_Wallet codebase SHALL remove unused style definitions
8. WHEN decomposition is complete, THE Vaughan_Wallet GUI SHALL render identically to before
9. WHEN decomposition is complete, THE Vaughan_Wallet codebase SHALL reduce theme code by at least 29%

### Requirement 7: Password Dialog Consolidation (Enhanced)

**User Story:** As a developer, I want a single configurable password dialog component, so that authentication UI is consistent and maintainable.

#### Acceptance Criteria

1. WHEN consolidating password dialogs, THE Vaughan_Wallet codebase SHALL merge password_dialog.rs, master_password_dialog.rs, and wallet_password_dialog.rs into a single file
2. WHEN creating the unified dialog, THE Vaughan_Wallet codebase SHALL use a configuration enum (PasswordDialogConfig) with all dialog purposes: UnlockSession, SignTransaction, ExportKey, CreateWallet, ImportWallet, ChangePassword, MasterPassword
3. WHEN creating the unified dialog, THE Vaughan_Wallet codebase SHALL preserve all existing functionality from the three original dialogs
4. WHEN creating the unified dialog, THE Vaughan_Wallet codebase SHALL add show/hide methods with config parameter
5. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL reduce total password dialog code by at least 33%
6. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL pass all password-related tests
7. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL ensure password fields are properly zeroized when dialog is hidden

### Requirement 8: State File Consolidation (Enhanced)

**User Story:** As a developer, I want related state management consolidated, so that state transitions are predictable and centralized.

#### Acceptance Criteria

1. WHEN consolidating state files, THE Vaughan_Wallet codebase SHALL merge security_state.rs and session_state.rs into auth_state.rs
2. WHEN consolidating state files, THE Vaughan_Wallet codebase SHALL combine session management, password dialogs, key cache, and attempt tracking in the unified AuthState
3. WHEN consolidating state files, THE Vaughan_Wallet codebase SHALL add methods for unlock, lock, record_activity, is_timed_out, is_locked_out, record_failed_attempt, and clear_lockout
4. WHEN consolidating state files, THE Vaughan_Wallet codebase SHALL ensure cached keys are cleared with proper zeroization on lock
5. WHEN consolidating state files, THE Vaughan_Wallet codebase SHALL remove duplicate ui_state.rs if it exists in both handlers/ and state/
6. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL keep network_state.rs and transaction_state.rs separate
7. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL have no duplicate state definitions
8. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL pass all state management tests
9. WHEN consolidation is complete, THE Vaughan_Wallet codebase SHALL reduce state code by at least 50%

### Requirement 9: Test File Relocation (Enhanced)

**User Story:** As a developer, I want all test files in the tests/ directory, so that source and test code are properly separated.

#### Acceptance Criteria

1. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move password_integration_tests.rs from src/gui/state/ to tests/
2. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move security_state_tests.rs from src/gui/state/ to tests/
3. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move session_property_tests.rs from src/gui/state/ to tests/
4. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move integration_test.rs from src/ to tests/
5. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move professional_test.rs from src/network/ to tests/
6. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move hardware_tests.rs from src/security/ to tests/
7. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL move multicall_test.rs from src/tokens/ to tests/
8. WHEN relocating test files, THE Vaughan_Wallet codebase SHALL update all imports from `crate::` to `vaughan::` paths
9. WHEN relocation is complete, THE Vaughan_Wallet codebase SHALL have zero test files in src/ directory
10. WHEN relocation is complete, THE Vaughan_Wallet codebase SHALL pass all relocated tests

### Requirement 10: Dependency Optimization (Enhanced)

**User Story:** As a release engineer, I want minimal dependencies with optimized features, so that the binary is small and compiles quickly.

#### Acceptance Criteria

1. WHEN optimizing dependencies, THE Vaughan_Wallet Cargo.toml SHALL replace alloy "full" feature with only required features: provider-http, signer-local, rlp, consensus, contract, eip2930, eip1559
2. WHEN optimizing dependencies, THE Vaughan_Wallet Cargo.toml SHALL remove "advanced" feature from iced if unused
3. WHEN optimizing dependencies, THE Vaughan_Wallet Cargo.toml SHALL add missing feature flags: professional, custom-tokens, shamir
4. WHEN optimizing dependencies, THE Vaughan_Wallet codebase SHALL consolidate duplicate dependency versions using patch section if needed
5. WHEN optimizing dependencies, THE Vaughan_Wallet codebase SHALL compile successfully with reduced features
6. WHEN optimization is complete, THE Vaughan_Wallet release binary SHALL be smaller than before optimization
7. WHEN optimization is complete, THE Vaughan_Wallet codebase SHALL verify minimal build works with --no-default-features --features minimal

### Requirement 11: Compiler Warning Elimination (Enhanced)

**User Story:** As a developer, I want zero compiler warnings, so that the codebase meets professional quality standards.

#### Acceptance Criteria

1. WHEN eliminating warnings, THE Vaughan_Wallet codebase SHALL fix all 9 existing compiler warnings
2. WHEN eliminating warnings, THE Vaughan_Wallet codebase SHALL not introduce new warnings
3. WHEN eliminating warnings, THE Vaughan_Wallet codebase SHALL compile with `cargo check --all-features` producing zero warnings
4. IF a warning cannot be fixed, THEN THE Vaughan_Wallet codebase SHALL document the reason and use targeted allow attribute

### Requirement 12: Code Quality Improvements (Enhanced)

**User Story:** As a maintainer, I want consistent code organization, so that the codebase follows Rust best practices.

#### Acceptance Criteria

1. WHEN improving code quality, THE Vaughan_Wallet codebase SHALL organize imports in standard order (std, external, local)
2. WHEN improving code quality, THE Vaughan_Wallet codebase SHALL extract common utilities into dedicated modules
3. WHEN improving code quality, THE Vaughan_Wallet codebase SHALL ensure all public items have documentation comments
4. WHEN improving code quality, THE Vaughan_Wallet codebase SHALL pass `cargo clippy --all-features` with zero warnings
5. WHEN improving code quality, THE Vaughan_Wallet codebase SHALL pass `cargo fmt --check` with no formatting issues
6. WHEN improving code quality, THE Vaughan_Wallet codebase SHALL credit MetaMask for any code patterns inspired by their codebase (when Alloy doesn't suffice)

### Requirement 13: Feature Gating

**User Story:** As a user, I want to build the wallet with only the features I need, so that the binary is minimal for my use case.

#### Acceptance Criteria

1. WHEN adding feature gates, THE Vaughan_Wallet Cargo.toml SHALL add a "professional" feature for network monitoring
2. WHEN adding feature gates, THE Vaughan_Wallet Cargo.toml SHALL add a "custom-tokens" feature for custom token management
3. WHEN adding feature gates, THE Vaughan_Wallet Cargo.toml SHALL add a "shamir" feature for Shamir's Secret Sharing
4. WHEN adding feature gates, THE Vaughan_Wallet codebase SHALL gate hardware.rs with the "hardware-wallets" feature using #[cfg(feature)]
5. WHEN adding feature gates, THE Vaughan_Wallet codebase SHALL gate professional.rs with the "professional" feature using #[cfg(feature)]
6. WHEN adding feature gates, THE Vaughan_Wallet codebase SHALL gate custom token dialogs with the "custom-tokens" feature
7. WHEN feature gating is complete, THE Vaughan_Wallet codebase SHALL compile successfully with all combinations of features
8. WHEN feature gating is complete, THE Vaughan_Wallet codebase SHALL verify minimal build works and produces a significantly smaller binary

### Requirement 14: Binary Size Reduction (Enhanced)

**User Story:** As a user, I want a smaller binary download, so that installation is faster and uses less disk space.

#### Acceptance Criteria

1. WHEN all debloat phases are complete, THE Vaughan_Wallet release binary SHALL be less than 14MB (enhanced from 15MB)
2. WHEN measuring binary size, THE Vaughan_Wallet build process SHALL use `cargo build --release` with LTO enabled
3. WHEN building with minimal features, THE Vaughan_Wallet release binary SHALL be less than 10MB
4. WHEN binary size exceeds target, THE Vaughan_Wallet codebase SHALL identify and remove additional bloat sources
5. WHEN binary size is measured, THE Vaughan_Wallet documentation SHALL record before and after sizes for both full and minimal builds

### Requirement 15: Feature Preservation (Enhanced)

**User Story:** As a user, I want all wallet features to work after debloating, so that functionality is not sacrificed for size.

#### Acceptance Criteria

1. WHEN debloating is complete, THE Vaughan_Wallet SHALL support wallet creation and import
2. WHEN debloating is complete, THE Vaughan_Wallet SHALL support transaction signing (Legacy and EIP-1559)
3. WHEN debloating is complete, THE Vaughan_Wallet SHALL support multi-network connectivity
4. WHEN debloating is complete, THE Vaughan_Wallet SHALL support hardware wallet integration (when feature enabled)
5. WHEN debloating is complete, THE Vaughan_Wallet SHALL support QR code generation (when feature enabled)
6. WHEN debloating is complete, THE Vaughan_Wallet SHALL support audio notifications (when feature enabled)
7. WHEN debloating is complete, THE Vaughan_Wallet SHALL pass all existing integration tests
8. WHEN debloating is complete, THE Vaughan_Wallet SHALL support all password dialog scenarios (unlock, sign, export, create, import, change, master)

### Requirement 16: Metrics Tracking (Enhanced)

**User Story:** As a project manager, I want measurable progress metrics, so that I can track debloat effectiveness.

#### Acceptance Criteria

1. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record file count before and after each phase
2. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record LOC count before and after each phase
3. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record binary size before and after each phase (for both full and minimal builds)
4. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record warning count before and after each phase
5. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record dead code annotation count before and after each phase
6. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record module sizes after decomposition
7. WHEN tracking metrics, THE Vaughan_Wallet documentation SHALL record code reduction percentages

### Requirement 17: Security Verification

**User Story:** As a security engineer, I want to verify that debloating hasn't introduced security vulnerabilities, so that the wallet remains secure.

#### Acceptance Criteria

1. WHEN decomposing security-critical modules, THE Vaughan_Wallet codebase SHALL ensure all seed data is properly zeroized
2. WHEN consolidating password dialogs, THE Vaughan_Wallet codebase SHALL ensure password fields are zeroized when dialog is hidden
3. WHEN consolidating auth state, THE Vaughan_Wallet codebase SHALL ensure cached keys are zeroized on lock
4. WHEN refactoring seed.rs, THE Vaughan_Wallet codebase SHALL never log seed phrases or private keys
5. WHEN refactoring keystore.rs, THE Vaughan_Wallet codebase SHALL use OS keychain for persistent storage
6. WHEN decomposing modules, THE Vaughan_Wallet codebase SHALL pass all existing security tests
7. WHEN debloating is complete, THE Vaughan_Wallet SHALL maintain all security guarantees (encryption, key derivation, zeroization)

---

## Success Metrics (Enhanced)

| Metric | Before | After (Full) | After (Minimal) | Improvement |
|--------|--------|---------------|-----------------|-------------|
| Files | 117 | <75 | <60 | ~36% / ~49% |
| Lines | 49,741 | <38,000 | <32,000 | ~24% / ~36% |
| Binary | 21MB | <14MB | <10MB | ~33% / ~52% |
| Warnings | 9 | 0 | 0 | 100% |
| Dead code | 22 | 0 | 0 | 100% |
| Test files in src/ | 7 | 0 | 0 | 100% |
| Password dialogs | 3 → 1 | 1 | 1 | 66% reduction |
| Auth state files | 2 → 1 | 1 | 1 | 50% reduction |
| Modules >1000 lines | 6 | 0 | 0 | 100% |
