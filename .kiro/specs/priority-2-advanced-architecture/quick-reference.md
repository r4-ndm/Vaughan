# Priority 2: Controller Architecture - Quick Reference Guide

## 🎯 Quick Start

```bash
# 1. Create feature branch
git checkout -b feature/controller-architecture

# 2. Baseline measurements
cargo test --all-features > baseline_tests.txt
cargo clean && time cargo build > baseline_build.txt

# 3. Start with Phase D: Controller Layer Creation
# Follow tasks.md step by step
```

---

## 📊 Current State

```
File: src/gui/working_wallet.rs
├── Total lines:          4,100
├── update() method:      2,902 lines (71% of file!)
├── view() method:        ~400 lines
└── Helper methods:       ~800 lines

Handlers (exist but coupled to iced):
├── transaction.rs        ❌ Coupled to UI framework
├── network.rs            ❌ Coupled to UI framework
├── security.rs           ❌ Coupled to UI framework
├── ui_state.rs           ❌ Coupled to UI framework
├── wallet_ops.rs         ❌ Coupled to UI framework
├── token_ops.rs          ❌ Coupled to UI framework
└── receive.rs            ❌ Coupled to UI framework

Controllers:
└── (none yet)            ❌ Need to create
```

---

## 🎯 Target State

```
File: src/gui/working_wallet.rs
├── Total lines:          <2,000 (51% reduction)
├── update() method:      <500 lines (83% reduction)
├── view() method:        ~400 lines (unchanged)
└── Helper methods:       ~100 lines (moved to controllers)

Controllers (NEW - framework-agnostic):
├── transaction.rs        ✅ Pure Alloy types, headless testable
├── network.rs            ✅ Alloy providers, headless testable
├── wallet.rs             ✅ Secure keyring, headless testable
└── price.rs              ✅ Price fetching, headless testable

Handlers (thin bridges):
├── transaction.rs        ✅ UI strings → Alloy types → Controller
├── network.rs            ✅ UI strings → Alloy types → Controller
├── security.rs           ✅ UI strings → Alloy types → Controller
├── ui_state.rs           ✅ Pure UI state (no controller needed)
├── wallet_ops.rs         ✅ UI strings → Alloy types → Controller
├── token_ops.rs          ✅ UI strings → Alloy types → Controller
└── receive.rs            ✅ Pure UI (no controller needed)
```

---

## ⚡ Quick Commands

### Controller Testing (Headless - No GUI!)
```bash
# Test specific controller
cargo test --lib controllers::transaction
cargo test --lib controllers::network
cargo test --lib controllers::wallet
cargo test --lib controllers::price

# Test all controllers
cargo test --lib controllers

# Test with output
cargo test --lib controllers::transaction -- --nocapture

# Property-based tests
cargo test --test controllers::transaction_properties
```

### Integration Testing (Headless)
```bash
# Test full transaction flow (no GUI required!)
cargo test --test integration::transaction_flow

# Test network switching flow
cargo test --test integration::network_flow

# Test account management flow
cargo test --test integration::wallet_flow
```

### General Testing
```bash
# Quick check
cargo check

# Test specific module
cargo test --lib <module_name>

# Full test suite
cargo test --all-features

# With output
cargo test --lib <module_name> -- --nocapture
```

### Performance
```bash
# Dependency analysis
cargo tree --duplicates
cargo bloat --release --crates

# Build timing
cargo clean
cargo build --timings
# Opens HTML report

# Measure compilation time
cargo clean
time cargo build
```

### Code Quality
```bash
# Clippy
cargo clippy -- -D warnings

# Format
cargo fmt --check
cargo fmt

# Line count
wc -l src/gui/working_wallet.rs
wc -l src/gui/handlers/*.rs
```

### Git
```bash
# Checkpoint
git add .
git commit -m "checkpoint: <description>"

# Rollback last commit
git reset --soft HEAD~1

# Abandon changes
git reset --hard HEAD
```

---

## 📋 Phase Checklist

### Phase D: Controller Layer Creation (3-4 hours)
- [ ] D1: Controller infrastructure (45 min)
- [ ] D2: TransactionController (60 min)
- [ ] D3: NetworkController (45 min)
- [ ] D4: WalletController (60 min)
- [ ] D5: PriceController (30 min)
- [ ] D6: Integration & testing (45 min)

**Success**: Controllers with pure Alloy types, 100% headless testable

### Phase E: Handler Bridge Refactoring (2-3 hours)
- [ ] E1: Transaction handler bridge (45 min)
- [ ] E2: Network handler bridge (30 min)
- [ ] E3: Wallet handler bridge (30 min)
- [ ] E4: Update WorkingWalletApp (45 min)
- [ ] E5: Clean up update() (30 min)

**Success**: Handlers are thin bridges (UI → Controller)

### Phase F: Testing & Validation (1-3 hours)
- [ ] F1: Headless controller tests (60 min)
- [ ] F2: Integration tests (45 min)
- [ ] F3: UI regression testing (30 min)
- [ ] F4: Performance validation (30 min)
- [ ] F5: Documentation (30 min)

**Success**: 100% coverage, zero regressions, headless testing works

---

## 🚨 Common Issues & Solutions

### Issue: Compilation errors in controllers
**Solution**: 
```bash
# Check what's missing
cargo check 2>&1 | grep "error"

# Common fixes:
# - Add Alloy imports: use alloy::primitives::{Address, U256, ChainId};
# - Add provider imports: use alloy::providers::Provider;
# - Check type conversions (String → Address, etc.)
```

### Issue: Tests failing after controller creation
**Solution**:
```bash
# Run specific test with output
cargo test <test_name> -- --nocapture

# Check what changed
git diff HEAD~1

# Rollback if needed
git reset --soft HEAD~1
```

### Issue: Handler not calling controller
**Solution**:
```rust
// Check handler has controller field
pub fn handle(app: &mut WorkingWalletApp, message: Message) -> Command<Message> {
    let controller = app.transaction_controller.clone();
    
    // Convert UI string → Alloy type
    let address = Address::from_str(&app.state.send_to_address())?;
    
    // Call controller with Alloy types
    Command::perform(
        async move { controller.validate_transaction(address, ...).await },
        |result| Message::ValidationResult(result)
    )
}
```

### Issue: Alloy type conversion errors
**Solution**:
```rust
// String → Address
let address = Address::from_str("0x...")
    .map_err(|e| format!("Invalid address: {}", e))?;

// String → U256 (ETH amount with 18 decimals)
let amount = parse_ether_amount("1.5")?; // Helper function

// u64 → ChainId
let chain_id = ChainId::from(943u64);
```

---

## 📈 Progress Tracking

### Measure Progress
```bash
# Controller lines (new)
wc -l src/controllers/*.rs

# Handler sizes (should stay small - thin bridges)
wc -l src/gui/handlers/*.rs

# Working wallet size
wc -l src/gui/working_wallet.rs

# Test coverage
cargo test --lib controllers
cargo test --test integration
```

### Expected Progress
```
After D1 (infrastructure):  Controllers: ~100 lines (setup)
After D2 (transaction):     Controllers: ~600 lines
After D3 (network):         Controllers: ~1,000 lines
After D4 (wallet):          Controllers: ~1,600 lines
After D5 (price):           Controllers: ~1,800 lines
After D6 (integration):     Controllers: ~2,000 lines ✅

After E1 (tx bridge):       Handlers: thin bridges
After E2 (net bridge):      Handlers: thin bridges
After E3 (wallet bridge):   Handlers: thin bridges
After E4 (app update):      WorkingWalletApp: has controllers
After E5 (cleanup):         update(): <500 lines ✅

After F1 (tests):           Test coverage: 100% ✅
After F2 (integration):     Integration tests: passing ✅
After F3 (UI):              GUI: working ✅
After F4 (perf):            Performance: maintained ✅
After F5 (docs):            Documentation: complete ✅
```

---

## 🎯 Success Indicators

### You're on track if:
- ✅ Controllers use only Alloy types (Address, U256, ChainId)
- ✅ Controllers have zero iced dependency
- ✅ Controllers are headless testable (no GUI required)
- ✅ Handlers are thin bridges (UI → Alloy → Controller)
- ✅ Tests pass after each step
- ✅ Compilation is clean (no errors)
- ✅ Git commits are small and focused

### Warning signs:
- ⚠️ Controllers importing iced types
- ⚠️ Business logic still in handlers
- ⚠️ Tests requiring GUI to run
- ⚠️ Controllers becoming too large (>1,000 lines each)
- ⚠️ Circular dependencies appearing
- ⚠️ String-based validation in controllers

---

## 💡 Pro Tips

### 1. Work incrementally
```bash
# After each substep:
cargo check
cargo test --lib controllers::<module>
git add .
git commit -m "step: <description>"
```

### 2. Test controllers headlessly
```bash
# No GUI required! Test business logic directly
cargo test --lib controllers::transaction

# Property-based tests
cargo test --test controllers::transaction_properties

# Integration tests (full flows, no GUI)
cargo test --test integration::transaction_flow
```

### 3. Use Alloy types everywhere in controllers
```rust
// ✅ GOOD: Pure Alloy types
pub fn validate_transaction(
    &self,
    to: Address,        // Alloy type
    amount: U256,       // Alloy type
    gas_limit: u64,
    balance: U256,      // Alloy type
) -> ControllerResult<()> {
    // Business logic with type safety
}

// ❌ BAD: String-based (runtime errors)
pub fn validate_transaction(
    &self,
    to: &str,           // Runtime parsing
    amount: &str,       // Runtime parsing
    gas_limit: u64,
    balance: &str,      // Runtime parsing
) -> ControllerResult<()> {
    // Parsing errors at runtime
}
```

### 4. Keep handlers thin
```rust
// Handler's ONLY job: UI strings → Alloy types → Controller
pub fn handle(app: &mut WorkingWalletApp, message: Message) -> Command<Message> {
    match message {
        Message::ConfirmTransaction => {
            // 1. Parse UI strings → Alloy types
            let to = Address::from_str(&app.state.send_to_address())?;
            let amount = parse_ether_amount(&app.state.send_amount())?;
            
            // 2. Call controller
            let controller = app.transaction_controller.clone();
            Command::perform(
                async move { controller.validate_transaction(to, amount, ...).await },
                |result| Message::ValidationResult(result)
            )
        }
    }
}
```

### 5. Document controller patterns
```rust
/// Transaction controller - pure business logic, no UI coupling
/// 
/// Follows MetaMask's TransactionController pattern:
/// - Validates transaction parameters
/// - Estimates gas
/// - Signs transactions
/// - Submits to network
/// - Monitors transaction status
/// 
/// Uses Alloy types exclusively for compile-time safety.
pub struct TransactionController {
    provider: Arc<RwLock<Provider>>,
    chain_id: ChainId,
}
```

---

## 🔗 Related Files

### Key Files to Know
```
src/controllers/                - NEW: Framework-agnostic business logic
├── mod.rs                      - Controller exports
├── transaction.rs              - Transaction controller (Alloy types)
├── network.rs                  - Network controller (Alloy providers)
├── wallet.rs                   - Wallet controller (keyring)
├── price.rs                    - Price controller
└── errors.rs                   - Controller errors

src/gui/working_wallet.rs       - Main file (will add controller fields)
src/gui/handlers/mod.rs         - Handler registry
src/gui/handlers/*.rs           - Handlers (will become thin bridges)
src/gui/state/mod.rs            - State management
src/gui/wallet_messages.rs      - Message definitions
```

### Documentation
```
.kiro/specs/priority-2-advanced-architecture/
├── plan.md                     - Detailed execution plan
├── tasks.md                    - Task tracking
├── quick-reference.md          - This file
└── README.md                   - Overview

.kiro/specs/improved_architecture_plan.md  - Gemini 3's original suggestion
```

---

## 📞 Need Help?

### Debugging Steps
1. Read the error message carefully
2. Check git diff to see what changed
3. Run cargo check for detailed errors
4. Test the specific module: `cargo test --lib <module>`
5. Rollback if stuck: `git reset --soft HEAD~1`

### Common Patterns

**Create controller:**
```rust
// src/controllers/transaction.rs
use alloy::primitives::{Address, U256, ChainId};
use alloy::providers::Provider;

pub struct TransactionController {
    provider: Arc<RwLock<Provider>>,
    chain_id: ChainId,
}

impl TransactionController {
    pub fn new(provider: Arc<RwLock<Provider>>, chain_id: ChainId) -> Self {
        Self { provider, chain_id }
    }
    
    // Pure Alloy types - no strings!
    pub fn validate_transaction(
        &self,
        to: Address,
        amount: U256,
        gas_limit: u64,
        balance: U256,
    ) -> ControllerResult<()> {
        // Business logic here
        Ok(())
    }
}
```

**Convert handler to bridge:**
```rust
// src/gui/handlers/transaction.rs
use crate::controllers::TransactionController;
use alloy::primitives::{Address, U256};

pub fn handle(app: &mut WorkingWalletApp, message: Message) -> Command<Message> {
    match message {
        Message::ConfirmTransaction => {
            // Parse UI strings → Alloy types
            let to = match Address::from_str(&app.state.send_to_address()) {
                Ok(addr) => addr,
                Err(e) => return error_command(format!("Invalid address: {}", e)),
            };
            
            let amount = match parse_ether_amount(&app.state.send_amount()) {
                Ok(amt) => amt,
                Err(e) => return error_command(format!("Invalid amount: {}", e)),
            };
            
            // Call controller with Alloy types
            let controller = app.transaction_controller.clone();
            Command::perform(
                async move {
                    controller.validate_transaction(to, amount, 21_000, balance).await
                },
                |result| match result {
                    Ok(_) => Message::ShowTransactionConfirmation,
                    Err(e) => Message::SetStatusMessage(e.to_string(), StatusMessageColor::Error),
                }
            )
        }
        _ => Command::none()
    }
}
```

**Test controller headlessly:**
```rust
// tests/controllers/transaction_tests.rs
#[tokio::test]
async fn test_validate_zero_address_rejected() {
    let controller = create_test_controller().await;
    
    let result = controller.validate_transaction(
        Address::ZERO,  // Should reject
        U256::from(1000),
        21_000,
        U256::from(10000),
    );
    
    assert!(result.is_err());
}
```

---

## ✅ Final Checklist

Before considering Phase D complete:
- [ ] `src/controllers/` directory created
- [ ] TransactionController implemented with Alloy types
- [ ] NetworkController implemented with Alloy providers
- [ ] WalletController implemented with secure keyring
- [ ] PriceController implemented
- [ ] All controllers have unit tests
- [ ] 100% controller test coverage
- [ ] Zero iced dependency in controllers
- [ ] All tests passing
- [ ] Zero compilation warnings

Before considering Phase E complete:
- [ ] Transaction handler converted to thin bridge
- [ ] Network handler converted to thin bridge
- [ ] Wallet handler converted to thin bridge
- [ ] WorkingWalletApp has controller fields
- [ ] update() method simplified to routing
- [ ] All handlers call controllers (not business logic)
- [ ] String → Alloy type conversion in handlers
- [ ] All tests passing

Before considering Phase F complete:
- [ ] Headless controller tests (100% coverage)
- [ ] Property-based tests for controllers
- [ ] Integration tests (full flows, no GUI)
- [ ] UI regression tests (manual)
- [ ] Performance benchmarks (no regression)
- [ ] Documentation complete
- [ ] All tests passing

---

**Remember**: Controllers are framework-agnostic, use only Alloy types, and are headless testable. Handlers are thin bridges that convert UI strings to Alloy types and call controllers.

**You've got this!** 🚀
