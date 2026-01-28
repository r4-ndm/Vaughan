# Business Logic Extraction - Implementation Tasks

## Overview
Extract business logic from view components into a dedicated service layer for better separation of concerns, testability, and maintainability.

---

## Phase 1: Foundation

- [x] 1. Create service directory structure
  - [x] 1.1 Create `src/gui/services/` directory
  - [x] 1.2 Create `src/gui/services/mod.rs` with service registry exports
  - [x] 1.3 Create `src/gui/services/asset_service.rs` with `AssetServiceTrait` and `AssetService`
  - [x] 1.4 Create `src/gui/services/account_display_service.rs` with `AccountDisplayServiceTrait` and `AccountDisplayService`

- [x] 2. Implement AssetService
  - [x] 2.1 Define `AssetServiceTrait` with `is_asset_available()`, `get_logo_path()`, `get_icon_path()` methods
  - [x] 2.2 Implement `AssetService` struct with `asset_root: PathBuf` field
  - [x] 2.3 Implement trait methods for file system operations
  - [x] 2.4 Add caching with `Arc<RwLock<HashMap<String, bool>>>` for asset availability

- [x] 3. Implement AccountDisplayService
  - [x] 3.1 Define `AccountDisplayInfo` struct with id, name, address, short_address, account_type fields
  - [x] 3.2 Define `AccountDisplayServiceTrait` with `format_account_display()`, `format_address_short()`, `get_account_type_label()` methods
  - [x] 3.3 Implement `AccountDisplayService` struct
  - [x] 3.4 Implement address truncation logic (first 6 + last 4 chars)

- [x] 4. Create ServiceRegistry and integrate with AppState
  - [x] 4.1 Create `ServiceRegistry` struct with `OnceCell` for lazy initialization
  - [x] 4.2 Add accessor methods for each service
  - [x] 4.3 Add `services: ServiceRegistry` field to `AppState`
  - [x] 4.4 Update `AppState::new()` to initialize ServiceRegistry

- [x] 5. Write unit tests for foundation services
  - [x] 5.1 Write tests for `AssetService::is_asset_available()` with existing and missing files
  - [x] 5.2 Write tests for `AccountDisplayService::format_address_short()` with various address lengths
  - [x] 5.3 Write tests for `AccountDisplayService::get_account_type_label()` for each account type

---

## Phase 2: Asset Service Migration

- [x] 6. Migrate logo operations in main_wallet.rs
  - [x] 6.1 Replace `std::path::Path::new("assets/vaughan-logo-513x76-thor.png").exists()` with service call
  - [x] 6.2 Update logo rendering to use `services.asset.get_logo_path()`
  - [x] 6.3 Test logo display with existing asset
  - [x] 6.4 Test logo fallback when asset missing

- [x] 7. Migrate icon operations in main_wallet.rs
  - [x] 7.1 Replace hamburger icon check with `services.asset.get_icon_path("hamburger")`
  - [x] 7.2 Replace clipboard icon check with `services.asset.get_icon_path("clipboard")`
  - [x] 7.3 Update all icon rendering to use service methods
  - [x] 7.4 Remove `std::path` imports from main_wallet.rs

- [x] 8. Write integration tests for asset service
  - [x] 8.1 Test asset loading with real asset files
  - [x] 8.2 Test caching behavior (second call should use cache)
  - [x] 8.3 Test with invalid/corrupted paths

---

## Phase 3: Account Display Migration

- [x] 9. Migrate account selector logic
  - [x] 9.1 Extract account selection logic from main_wallet.rs lines 58-84
  - [x] 9.2 Update account selector to use `services.account.format_account_display()`
  - [x] 9.3 Replace inline address formatting with `services.account.format_address_short()`
  - [x] 9.4 Test account switching with multiple account types

- [x] 10. Migrate address display throughout views
  - [x] 10.1 Find all inline address formatting in views
  - [x] 10.2 Replace with service calls
  - [x] 10.3 Ensure consistent formatting across all views
  - [x] 10.4 Test with various address lengths and edge cases

- [x] 11. Write property-based tests for account display
  - [x] 11.1 Write proptest for address formatting (any valid hex address)
  - [x] 11.2 Test that short address always contains "..."
  - [x] 11.3 Test that short address length is always <= original length

---

## Phase 4: Network Configuration Service

- [x] 12. Implement NetworkConfigService
  - [x] 12.1 Create `src/gui/services/network_config_service.rs`
  - [x] 12.2 Define `NetworkValidationError` enum (InvalidRpcUrl, InvalidChainId, InvalidExplorerUrl, DuplicateNetwork)
  - [x] 12.3 Define `NetworkConfigServiceTrait` with validation methods
  - [x] 12.4 Implement `validate_network_config()`, `is_network_name_unique()`, `sanitize_rpc_url()` methods

- [x] 13. Migrate network validation from dialogs
  - [x] 13.1 Extract validation logic from dialogs.rs lines 1149-1216
  - [x] 13.2 Update network configuration dialog to use service
  - [x] 13.3 Test network creation with valid and invalid data
  - [x] 13.4 Verify error messages are user-friendly

- [x] 14. Write tests for network validation
  - [x] 14.1 Test RPC URL validation (http/https required)
  - [x] 14.2 Test chain ID validation (non-zero)
  - [x] 14.3 Test duplicate network detection
  - [x] 14.4 Write property-based tests for URL sanitization

---

## Phase 5: Transaction Form Service (HIGH RISK)

- [x] 15. Implement TransactionFormService
  - [x] 15.1 Create `src/gui/services/transaction_form_service.rs`
  - [x] 15.2 Define `SendFormData` struct
  - [x] 15.3 Define `TransactionValidationError` enum
  - [x] 15.4 Implement `prepare_send_form()`, `validate_recipient()`, `validate_amount()` methods

- [x] 16. Add feature flag for gradual rollout
  - [x] 16.1 Add `use_transaction_service: bool` field to AppState
  - [x] 16.2 Implement parallel rendering (legacy and service-based)
  - [x] 16.3 Add logging for transaction operations
  - [x] 16.4 Create rollback mechanism

- [x] 17. Migrate send form logic
  - [x] 17.1 Extract send form logic from main_wallet.rs lines 154-177
  - [x] 17.2 Update send form to use service when feature flag enabled
  - [x] 17.3 Test both implementations in parallel
  - [x] 17.4 Compare outputs for consistency

- [x] 18. Write comprehensive transaction tests
  - [x] 18.1 Write unit tests for recipient validation
  - [x] 18.2 Write unit tests for amount validation
  - [x] 18.3 Write property-based tests for amount validation (positive amounts, balance checks)
  - [x] 18.4 Test edge cases (zero, max uint, insufficient balance)

---

## Phase 6: Cleanup and Optimization

- [x] 19. Remove legacy code and unused imports
  - [x] 19.1 Remove all `std::path::Path` imports from view files (Phase 2)
  - [x] 19.2 Remove inline validation logic from views (Phase 5 - service-based now)
  - [x] 19.3 Remove commented-out legacy code
  - [x] 19.4 Run clippy and fix all warnings

- [x] 20. Performance optimization
  - [x] 20.1 Verify caching is working in AssetService
  - [x] 20.2 Profile service call overhead
  - [x] 20.3 Benchmark before/after performance
  - [x] 20.4 Optimize any hot paths identified

- [x] 21. Final validation and documentation
  - [x] 21.1 Run full test suite
  - [x] 21.2 Verify >90% test coverage for services
  - [x] 21.3 Update service layer documentation
  - [x] 21.4 Verify zero functional regressions

---

## Acceptance Criteria

- [x] Zero file operations in view layer
- [x] Zero validation logic in view layer (moved to TransactionFormService)
- [x] All services have trait interfaces for mocking
- [x] All services have >90% test coverage (115 total tests)
- [x] Zero functional regressions in UI
- [x] View files reduced by 40-60% in complexity (validation extracted)
