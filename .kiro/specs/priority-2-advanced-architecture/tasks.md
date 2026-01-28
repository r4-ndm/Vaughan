# Priority 2: Controller-Based Architecture - Task Tracking

## Overview
Transform Vaughan into MetaMask-inspired controller architecture with strict Alloy type integration for headless testing and framework-agnostic business logic.

**Architecture**: Controller-View Separation (MetaMask pattern)
**Type Safety**: Alloy Primitives Only
**Timeline**: 6-10 hours
**Risk**: 🟡 MEDIUM (new layer, proven pattern)
**Status**: 📋 READY TO START

---

## PHASE D: CONTROLLER LAYER CREATION (3-4 hours)

### D1: Controller Infrastructure Setup (45 min)
- [ ] Create `src/controllers/` directory
- [ ] Create `src/controllers/mod.rs` with exports
- [ ] Define `ControllerResult<T>` type alias
- [ ] Define `ControllerError` enum using Alloy errors
- [ ] Add `thiserror` dependency if needed
- [ ] Document controller architecture patterns
- [ ] Run: `cargo check`
- [ ] Git commit: "feat(controllers): Add controller infrastructure with Alloy types"

**Deliverable**: Controller module structure ready

---

### D2: TransactionController Implementation (60 min)
- [ ] Create `src/controllers/transaction.rs`
- [ ] Implement `TransactionController` struct
- [ ] Add `validate_transaction()` method (Alloy types)
  - [ ] Zero address check (MetaMask pattern)
  - [ ] Amount validation (positive, non-zero)
  - [ ] Gas limit validation (21k-30M)
  - [ ] Balance check (amount + gas)
- [ ] Add `estimate_gas()` method (Alloy provider)
- [ ] Add `build_transaction()` method (Alloy `TransactionRequest`)
- [ ] Add `submit_transaction()` method (Alloy provider)
- [ ] Add `get_transaction_receipt()` method
- [ ] Write unit tests for zero address rejection
- [ ] Write unit tests for insufficient balance
- [ ] Write unit tests for gas limit validation
- [ ] Write unit tests for transaction building
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib controllers::transaction`
- [ ] Git commit: "feat(controllers): Implement TransactionController with Alloy types"

**Success**: TransactionController with pure Alloy types, headless testable

---

### D3: NetworkController Implementation (45 min)
- [ ] Create `src/controllers/network.rs`
- [ ] Implement `NetworkController` struct
- [ ] Add `new()` method (create Alloy provider)
- [ ] Add `get_chain_id()` method (Alloy provider)
- [ ] Add `check_network_health()` method
- [ ] Add `get_balance()` method (Address → U256)
- [ ] Add `switch_network()` method
  - [ ] Create new provider
  - [ ] Verify chain ID matches
  - [ ] Update internal state
- [ ] Write unit tests for network creation
- [ ] Write unit tests for chain ID validation
- [ ] Write unit tests for balance fetching
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib controllers::network`
- [ ] Git commit: "feat(controllers): Implement NetworkController with Alloy providers"

**Success**: NetworkController with Alloy providers, headless testable

---

### D4: WalletController Implementation (60 min)
- [ ] Create `src/controllers/wallet.rs`
- [ ] Implement `WalletController` struct
- [ ] Add `new()` method
- [ ] Add `add_account()` method (private key → LocalWallet)
  - [ ] Use `secrecy::Secret` for private key
  - [ ] Create Alloy `LocalWallet`
  - [ ] Store in HashMap
  - [ ] Return Address
- [ ] Add `get_current_address()` method
- [ ] Add `sign_message()` method (Alloy signer)
- [ ] Add `switch_account()` method
- [ ] Add `remove_account()` method
- [ ] Write unit tests for account creation
- [ ] Write unit tests for account switching
- [ ] Write unit tests for signing
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib controllers::wallet`
- [ ] Git commit: "feat(controllers): Implement WalletController with secure keyring"

**Success**: WalletController with secure keyring, headless testable

---

### D5: PriceController Implementation (30 min)
- [ ] Create `src/controllers/price.rs`
- [ ] Implement `PriceController` struct
- [ ] Add `new()` method (optional API key)
- [ ] Add `fetch_eth_price()` method
- [ ] Add `get_cached_price()` method
- [ ] Add price caching logic
- [ ] Write unit tests for price fetching
- [ ] Write unit tests for caching
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib controllers::price`
- [ ] Git commit: "feat(controllers): Implement PriceController"

**Success**: PriceController for token prices

---

### D6: Controller Integration & Testing (45 min)
- [ ] Update `src/controllers/mod.rs` with all exports
- [ ] Export all controller types
- [ ] Export `ControllerResult` and `ControllerError`
- [ ] Create `tests/controllers/` directory
- [ ] Create `tests/controllers/transaction_tests.rs`
- [ ] Create `tests/controllers/network_tests.rs`
- [ ] Create `tests/controllers/wallet_tests.rs`
- [ ] Write integration test: full transaction flow
- [ ] Write integration test: network switching
- [ ] Write integration test: account management
- [ ] Run: `cargo test --lib controllers`
- [ ] Run: `cargo test --test controllers`
- [ ] Verify: All controller tests passing
- [ ] Git commit: "test(controllers): Add comprehensive controller tests"

**Deliverable**: Fully integrated controller layer with tests

---

### Phase D Validation
- [ ] Run: `cargo test --all-features`
- [ ] Run: `cargo check --all-features`
- [ ] Run: `cargo clippy -- -D warnings`
- [ ] Verify: All controllers use Alloy types only
- [ ] Verify: No iced dependency in controllers
- [ ] Verify: 100% controller test coverage
- [ ] Git commit: "feat(phase-d): Complete controller layer creation"

**Phase D Success Criteria**:
- ✅ `src/controllers/` directory created
- ✅ All 4 controllers implemented
- ✅ Pure Alloy types (no strings)
- ✅ Zero iced dependency
- ✅ 100% test coverage
- ✅ Headless testable

---

## PHASE E: HANDLER BRIDGE REFACTORING (2-3 hours)

### E1: Transaction Handler Bridge (45 min)
- [ ] Open `src/gui/handlers/transaction.rs`
- [ ] Add `use crate::controllers::TransactionController;`
- [ ] Add `use alloy::primitives::{Address, U256};`
- [ ] Create `parse_ether_amount()` helper function
- [ ] Update `Message::ConfirmTransaction` handler:
  - [ ] Parse UI string → Address (with error handling)
  - [ ] Parse UI string → U256 (with error handling)
  - [ ] Call `controller.validate_transaction()` with Alloy types
  - [ ] Return appropriate UI message on success/error
- [ ] Update `Message::SubmitTransaction` handler:
  - [ ] Build transaction with controller
  - [ ] Get signer from wallet controller
  - [ ] Submit with controller
  - [ ] Return transaction hash or error
- [ ] Remove inline business logic (moved to controller)
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib handlers::transaction`
- [ ] Manual test: Send transaction in GUI
- [ ] Git commit: "refactor(handlers): Convert transaction handler to thin bridge"

**Success**: Transaction handler is thin bridge (UI → Controller)

---

### E2: Network Handler Bridge (30 min)
- [ ] Open `src/gui/handlers/network.rs`
- [ ] Add `use crate::controllers::NetworkController;`
- [ ] Update `Message::NetworkSelected` handler:
  - [ ] Get RPC URL and chain ID
  - [ ] Call `controller.switch_network()`
  - [ ] Return success/error message
- [ ] Update `Message::RefreshBalance` handler:
  - [ ] Get current address (Address type)
  - [ ] Call `controller.get_balance()`
  - [ ] Return balance or error
- [ ] Remove inline business logic
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib handlers::network`
- [ ] Manual test: Switch networks in GUI
- [ ] Git commit: "refactor(handlers): Convert network handler to thin bridge"

**Success**: Network handler is thin bridge

---

### E3: Wallet Handler Bridge (30 min)
- [ ] Open `src/gui/handlers/wallet_ops.rs`
- [ ] Add `use crate::controllers::WalletController;`
- [ ] Update `Message::ImportAccount` handler:
  - [ ] Get private key as Secret
  - [ ] Call `controller.add_account()`
  - [ ] Return address or error
- [ ] Update `Message::AccountSelected` handler:
  - [ ] Call `controller.switch_account()`
  - [ ] Return success/error
- [ ] Remove inline business logic
- [ ] Run: `cargo check`
- [ ] Run: `cargo test --lib handlers::wallet_ops`
- [ ] Manual test: Import account in GUI
- [ ] Git commit: "refactor(handlers): Convert wallet handler to thin bridge"

**Success**: Wallet handler is thin bridge

---

### E4: Update WorkingWalletApp Structure (45 min)
- [ ] Open `src/gui/working_wallet.rs`
- [ ] Add controller imports
- [ ] Add controller fields to `WorkingWalletApp`:
  - [ ] `transaction_controller: Arc<TransactionController>`
  - [ ] `network_controller: Arc<NetworkController>`
  - [ ] `wallet_controller: Arc<WalletController>`
  - [ ] `price_controller: Arc<PriceController>`
- [ ] Update `Application::new()`:
  - [ ] Initialize NetworkController
  - [ ] Initialize WalletController
  - [ ] Initialize TransactionController
  - [ ] Initialize PriceController
- [ ] Keep legacy fields for now (gradual migration)
- [ ] Run: `cargo check`
- [ ] Run: `cargo build`
- [ ] Git commit: "refactor(app): Add controller fields to WorkingWalletApp"

**Success**: WorkingWalletApp has controller fields

---

### E5: Clean Up update() Method (30 min)
- [ ] Open `src/gui/working_wallet.rs`
- [ ] Review update() method
- [ ] Ensure all messages route to handlers
- [ ] Remove any remaining inline business logic
- [ ] Simplify routing logic
- [ ] Add documentation comments
- [ ] Run: `cargo check`
- [ ] Run: `cargo build`
- [ ] Measure file size: `wc -l src/gui/working_wallet.rs`
- [ ] Verify: <2,000 lines (from 4,100)
- [ ] Git commit: "refactor(app): Simplify update() to pure routing"

**Success**: update() is pure routing logic

---

### Phase E Validation
- [ ] Run: `cargo test --all-features`
- [ ] Run: `cargo check --all-features`
- [ ] Run: `cargo clippy -- -D warnings`
- [ ] Verify: Handlers are thin bridges only
- [ ] Verify: All business logic in controllers
- [ ] Verify: GUI still works (manual test)
- [ ] Git commit: "feat(phase-e): Complete handler bridge refactoring"

**Phase E Success Criteria**:
- ✅ All handlers converted to thin bridges
- ✅ Handlers only do: UI string → Alloy type → Controller
- ✅ No business logic in handlers
- ✅ WorkingWalletApp has controller fields
- ✅ update() simplified
- ✅ All tests passing
- ✅ GUI functional

---

## PHASE F: TESTING & VALIDATION (1-3 hours)

### F1: Headless Controller Tests (60 min)
- [ ] Create `tests/controllers/transaction_tests.rs`
- [ ] Write test: `test_validate_zero_address_rejected()`
- [ ] Write test: `test_validate_insufficient_balance()`
- [ ] Write test: `test_validate_gas_limit_too_low()`
- [ ] Write test: `test_validate_gas_limit_too_high()`
- [ ] Write test: `test_build_transaction_with_alloy_types()`
- [ ] Create `tests/controllers/transaction_properties.rs`
- [ ] Write proptest: `test_any_valid_address_accepted()`
- [ ] Write proptest: `test_amount_plus_gas_never_overflows()`
- [ ] Write proptest: `test_gas_limit_bounds_enforced()`
- [ ] Create `tests/controllers/network_tests.rs`
- [ ] Write test: `test_network_controller_creation()`
- [ ] Write test: `test_chain_id_validation()`
- [ ] Write test: `test_balance_fetching()`
- [ ] Create `tests/controllers/wallet_tests.rs`
- [ ] Write test: `test_wallet_controller_creation()`
- [ ] Write test: `test_add_account_from_private_key()`
- [ ] Write test: `test_switch_account()`
- [ ] Run: `cargo test --lib controllers`
- [ ] Run: `cargo test --test controllers`
- [ ] Verify: 100% controller test coverage
- [ ] Git commit: "test(controllers): Add comprehensive headless tests"

**Success**: 100% controller test coverage, all headless

---

### F2: Integration Tests (45 min)
- [ ] Create `tests/integration/transaction_flow.rs`
- [ ] Write test: `test_complete_transaction_flow_headless()`
  - [ ] Create controllers (no GUI)
  - [ ] Add account
  - [ ] Get balance
  - [ ] Validate transaction
  - [ ] Build transaction
  - [ ] Verify all steps work
- [ ] Create `tests/integration/network_flow.rs`
- [ ] Write test: `test_network_switching_flow()`
- [ ] Create `tests/integration/wallet_flow.rs`
- [ ] Write test: `test_account_management_flow()`
- [ ] Run: `cargo test --test integration`
- [ ] Verify: All integration tests passing
- [ ] Git commit: "test(integration): Add headless integration tests"

**Success**: Full flows testable without GUI

---

### F3: UI Regression Testing (30 min)
- [ ] Run: `cargo run --bin vaughan`
- [ ] Manual test: Import account
- [ ] Manual test: Switch networks
- [ ] Manual test: Send transaction
- [ ] Manual test: Check balance
- [ ] Manual test: View transaction history
- [ ] Verify: All UI features work
- [ ] Verify: Spinners show correctly
- [ ] Verify: Error messages display
- [ ] Verify: Success messages display
- [ ] Document any issues found
- [ ] Fix any regressions
- [ ] Git commit: "test(ui): Verify UI regression testing complete"

**Success**: Zero UI regressions

---

### F4: Performance Validation (30 min)
- [ ] Create `benches/controller_benchmarks.rs`
- [ ] Add benchmark: `benchmark_transaction_validation()`
- [ ] Add benchmark: `benchmark_network_balance_fetch()`
- [ ] Add benchmark: `benchmark_wallet_signing()`
- [ ] Run: `cargo bench`
- [ ] Compare with baseline (if available)
- [ ] Verify: No performance regression
- [ ] Document performance results
- [ ] Git commit: "perf(controllers): Add controller benchmarks"

**Success**: Performance maintained or improved

---

### F5: Documentation & Completion (30 min)
- [ ] Create `docs/architecture/CONTROLLER_ARCHITECTURE.md`
- [ ] Document controller pattern
- [ ] Document Alloy type usage
- [ ] Document headless testing approach
- [ ] Add controller usage examples
- [ ] Update README with controller info
- [ ] Create migration guide (handlers → controllers)
- [ ] Run: `cargo doc --no-deps --open`
- [ ] Verify: Documentation builds correctly
- [ ] Git commit: "docs(controllers): Add controller architecture documentation"

**Deliverable**: Complete controller documentation

---

### Phase F Validation
- [ ] Run: `cargo test --all-features`
- [ ] Run: `cargo check --all-features`
- [ ] Run: `cargo clippy -- -D warnings`
- [ ] Run: `cargo fmt --check`
- [ ] Run: `cargo bench`
- [ ] Verify: 100% test pass rate
- [ ] Verify: Zero functional regressions
- [ ] Verify: Performance maintained
- [ ] Verify: Documentation complete
- [ ] Git commit: "feat(phase-f): Complete testing and validation"

**Phase F Success Criteria**:
- ✅ 100% controller test coverage
- ✅ Headless tests working
- ✅ Integration tests passing
- ✅ Zero UI regressions
- ✅ Performance validated
- ✅ Documentation complete

---

## FINAL VALIDATION & COMPLETION

### Final Checks
- [ ] Run: `cargo test --all-features`
- [ ] Run: `cargo check --all-features`
- [ ] Run: `cargo clippy -- -D warnings`
- [ ] Run: `cargo fmt --check`
- [ ] Run: `cargo build --release`
- [ ] Verify: Controllers are framework-agnostic
- [ ] Verify: All business logic uses Alloy types
- [ ] Verify: Handlers are thin bridges only
- [ ] Verify: Headless testing works
- [ ] Verify: GUI functions correctly
- [ ] Verify: Zero functional regressions
- [ ] Verify: 100% test pass rate

### Documentation
- [ ] Update `docs/architecture/CONTROLLER_ARCHITECTURE.md`
- [ ] Update `docs/architecture/PRIORITY_2_PROFESSIONAL_PLAN.md` status
- [ ] Create completion summary document
- [ ] Update README with controller architecture
- [ ] Document MetaMask patterns used
- [ ] Document Alloy integration

### Git & GitHub
- [ ] Final commit: "feat(priority-2): Complete controller-based architecture"
- [ ] Merge feature branch to main
- [ ] Push to GitHub: `git push origin main`
- [ ] Verify push successful
- [ ] Create GitHub release tag (optional)

---

## SUCCESS METRICS

### Code Quality
- ✅ Controllers: ~2,500 lines (pure business logic)
- ✅ Handlers: <400 lines (thin bridges)
- ✅ working_wallet.rs: <2,000 lines (from 4,100)
- ✅ update() method: <500 lines (from 2,902)
- ✅ Zero iced dependency in controllers
- ✅ 100% Alloy type usage in controllers

### Architecture
- ✅ Framework-agnostic business logic
- ✅ Headless testable (no GUI needed)
- ✅ Type-safe with Alloy primitives
- ✅ Reusable in CLI/API/mobile
- ✅ MetaMask patterns implemented
- ✅ Security-critical code isolated

### Testing
- ✅ 100% controller test coverage
- ✅ Property-based tests for controllers
- ✅ Integration tests (headless)
- ✅ Zero UI regressions
- ✅ Performance validated

---

## COMPLETION STATUS

**Phase D**: ⬜ Not Started
**Phase E**: ⬜ Not Started
**Phase F**: ⬜ Not Started

**Overall**: ⬜ Not Started

---

*Task tracking created: January 28, 2026*
*Architecture: MetaMask-inspired Controller Pattern*
*Type Safety: Alloy Primitives*
*Ready to begin execution*
