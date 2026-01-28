# Priority 2: Quick Reference Guide

## 🎯 Quick Start

```bash
# 1. Create feature branch
git checkout -b feature/priority-2-advanced-architecture

# 2. Baseline measurements
cargo test --all-features > baseline_tests.txt
cargo clean && time cargo build > baseline_build.txt

# 3. Start with Phase D
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

Handlers (already exist):
├── transaction.rs        ✅
├── network.rs            ✅
├── security.rs           ✅
├── ui_state.rs           ✅
├── wallet_ops.rs         ✅
├── token_ops.rs          ✅
└── receive.rs            ✅
```

---

## 🎯 Target State

```
File: src/gui/working_wallet.rs
├── Total lines:          <1,500 (63% reduction)
├── update() method:      <300 lines (90% reduction)
├── view() method:        ~400 lines (unchanged)
└── Helper methods:       ~200 lines (moved to handlers)

Handlers (complete):
├── transaction.rs        ✅ All transaction messages
├── network.rs            ✅ All network messages
├── security.rs           ✅ All security messages
├── ui_state.rs           ✅ All UI state messages
├── wallet_ops.rs         ✅ All wallet ops messages
├── token_ops.rs          ✅ All token ops messages
└── receive.rs            ✅ All receive messages

State Management:
└── manager.rs            ✅ Centralized state updates
```

---

## ⚡ Quick Commands

### Testing
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

### Phase D: Handler Completion (2-3 hours)
- [ ] D1: Analyze coverage (30 min)
- [ ] D2: Transaction handler (30 min)
- [ ] D3: Network handler (20 min)
- [ ] D4: Security handler (30 min)
- [ ] D5: UI state handler (20 min)
- [ ] D6: Wallet ops handler (30 min)
- [ ] D7: Token ops handler (20 min)
- [ ] D8: Clean up update() (30 min)

**Success**: update() <300 lines, working_wallet.rs <1,500 lines

### Phase E: Performance (1-2 hours)
- [ ] E1: Dependency analysis (30 min)
- [ ] E2: Module optimization (45 min)
- [ ] E3: Runtime optimization (45 min)

**Success**: 2-3x faster compilation

### Phase F: State Management (1.5-2 hours)
- [ ] F1: Design (45 min)
- [ ] F2: Implementation (60 min)
- [ ] F3: Integration (45 min)

**Success**: Centralized state management

---

## 🚨 Common Issues & Solutions

### Issue: Compilation errors after extraction
**Solution**: 
```bash
# Check what's missing
cargo check 2>&1 | grep "error"

# Common fixes:
# - Add missing imports
# - Update method signatures
# - Fix visibility (pub/private)
```

### Issue: Tests failing after changes
**Solution**:
```bash
# Run specific test with output
cargo test <test_name> -- --nocapture

# Check what changed
git diff HEAD~1

# Rollback if needed
git reset --soft HEAD~1
```

### Issue: Handler not receiving messages
**Solution**:
```rust
// Check routing in update()
match message {
    Message::YourMessage => {
        // Make sure it routes to correct handler
        handlers::your_handler::handle(self, message)
    }
}

// Check handler implementation
pub fn handle(app: &mut WorkingWalletApp, message: Message) -> Command<Message> {
    match message {
        Message::YourMessage => {
            // Your logic here
        }
        _ => Command::none()
    }
}
```

### Issue: State not updating
**Solution**:
```rust
// Make sure you're mutating state correctly
app.state.your_module_mut().field = value;

// Or using accessor
*app.state.your_field_mut() = value;

// Check state is being returned
// (not just modified in a clone)
```

---

## 📈 Progress Tracking

### Measure Progress
```bash
# File size
wc -l src/gui/working_wallet.rs

# Update method size (approximate)
grep -n "fn update" src/gui/working_wallet.rs
grep -n "fn view" src/gui/working_wallet.rs
# Subtract line numbers

# Handler sizes
wc -l src/gui/handlers/*.rs

# Total handler lines
wc -l src/gui/handlers/*.rs | tail -1
```

### Expected Progress
```
After D2 (transaction):   update() ~2,700 lines
After D3 (network):       update() ~2,500 lines
After D4 (security):      update() ~2,100 lines
After D5 (ui_state):      update() ~1,800 lines
After D6 (wallet_ops):    update() ~1,200 lines
After D7 (token_ops):     update() ~600 lines
After D8 (cleanup):       update() <300 lines ✅
```

---

## 🎯 Success Indicators

### You're on track if:
- ✅ Each handler completion reduces update() by 200-500 lines
- ✅ Tests pass after each step
- ✅ Compilation is clean (no errors)
- ✅ Git commits are small and focused
- ✅ You can explain what each handler does

### Warning signs:
- ⚠️ Tests failing after changes
- ⚠️ Compilation errors accumulating
- ⚠️ update() not getting smaller
- ⚠️ Handlers becoming too large (>1,000 lines)
- ⚠️ Circular dependencies appearing

---

## 💡 Pro Tips

### 1. Work incrementally
```bash
# After each substep:
cargo check
cargo test --lib <module>
git add .
git commit -m "step: <description>"
```

### 2. Use grep to find messages
```bash
# Find all Message:: variants
rg "Message::" src/gui/working_wallet.rs | grep "=>"

# Find specific message handling
rg "Message::YourMessage" src/gui/
```

### 3. Test as you go
```bash
# Don't wait until the end
# Test after each handler completion
cargo test --lib <handler_name>
```

### 4. Keep handlers focused
```
Good handler size: 200-800 lines
Warning: >1,000 lines (consider splitting)
Red flag: >1,500 lines (definitely split)
```

### 5. Document as you extract
```rust
// Add clear documentation to handlers
/// Handles all transaction-related messages
/// 
/// Messages handled:
/// - EstimateGas: Estimates gas for transaction
/// - ConfirmTransaction: Shows confirmation dialog
/// - SubmitTransaction: Submits transaction to network
pub fn handle(app: &mut WorkingWalletApp, message: Message) -> Command<Message> {
    // ...
}
```

---

## 🔗 Related Files

### Key Files to Know
```
src/gui/working_wallet.rs       - Main file to reduce
src/gui/handlers/mod.rs         - Handler registry
src/gui/handlers/*.rs           - Individual handlers
src/gui/state/mod.rs            - State management
src/gui/wallet_messages.rs      - Message definitions
```

### Documentation
```
.kiro/specs/priority-2-advanced-architecture/
├── plan.md                     - Detailed execution plan
├── tasks.md                    - Task tracking
└── quick-reference.md          - This file
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

**Extract message handling:**
```rust
// Before (in update()):
Message::Something => {
    // 50 lines of logic
}

// After (in handler):
pub fn handle(app: &mut WorkingWalletApp, message: Message) -> Command<Message> {
    match message {
        Message::Something => {
            // 50 lines of logic (moved here)
        }
        _ => Command::none()
    }
}

// In update():
Message::Something => handlers::your_handler::handle(self, message),
```

**Access state:**
```rust
// Read state
let value = app.state.your_field();

// Mutate state
app.state.your_field_mut().update(value);

// Or direct access
app.state.network_mut().current_network = network_id;
```

---

## ✅ Final Checklist

Before considering Phase D complete:
- [ ] update() method <300 lines
- [ ] working_wallet.rs <1,500 lines
- [ ] All tests passing
- [ ] Zero compilation warnings
- [ ] All handlers documented
- [ ] Git history clean and organized

Before considering Phase E complete:
- [ ] Compilation 20-30% faster
- [ ] Zero duplicate dependencies
- [ ] Feature flags optimized
- [ ] All tests passing

Before considering Phase F complete:
- [ ] StateManager implemented
- [ ] All handlers use StateManager
- [ ] State validation working
- [ ] All tests passing

---

**Remember**: Slow and steady wins the race. Test after each step, commit frequently, and don't hesitate to rollback if something goes wrong.

**You've got this!** 🚀
