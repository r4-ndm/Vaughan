# Code Quality Improvement Plan

**Goal**: Polish the codebase before public release
**Status**: Ready to execute
**Estimated Time**: 2-3 hours

---

## Task 1: Auto-Fix Minor Issues with Clippy

**Priority**: High (Quick wins)
**Estimated Time**: 15 minutes

### Subtasks

- [x] **1.1** Run `cargo clippy --fix --allow-dirty` ✅
  - Fixed 18 issues in test_rpc.rs
  - Fixed benchmark imports
  - Fixes applied successfully

- [x] **1.2** Run `cargo fmt` to format code ✅
  - Code formatted consistently
  - Whitespace issues fixed

- [x] **1.3** Verify build still works ✅
  - Release build successful (3m 51s)
  - No regressions

- [x] **1.4** Review changes ✅
  - Benchmark imports updated
  - Minor fixes applied
  - All changes safe

**Expected Outcome**: ✅ Clean clippy output, consistent formatting - COMPLETE

---

## Task 2: Review and Remove Dead Code

**Priority**: Medium (Code cleanliness)
**Estimated Time**: 1-1.5 hours

### 2.1: Token Management Dead Code

- [x] **2.1.1** Review `token` field usage ✅
  - File: Token-related structs
  - Action: Remove if truly unused, or add `#[allow(dead_code)]` if planned

- [ ] **2.1.2** Review `prices` field
  - File: Pricing module
  - Action: Remove or implement usage

- [ ] **2.1.3** Review `tokens` field
  - File: Token management
  - Action: Remove or implement usage

- [ ] **2.1.4** Review token metadata fields (`id`, `symbol`, `name`)
  - File: Token structs
  - Action: Remove or implement usage

- [ ] **2.1.5** Review `usd_price_formatted` field
  - File: Pricing display
  - Action: Remove or implement usage

### 2.2: Monitoring Dead Code

- [ ] **2.2.1** Review `monitoring_tasks` field
  - File: Monitoring module
  - Action: Remove if unused, or implement monitoring

- [ ] **2.2.2** Review `default_ttl` field
  - File: Cache/monitoring
  - Action: Remove or implement TTL logic

### 2.3: Network Dead Code

- [ ] **2.3.1** Review `network_manager` field
  - File: Network module
  - Action: Remove or implement network management

### 2.4: Hardware Wallet Dead Code

- [ ] **2.4.1** Review `max_signing_attempts` field
  - File: `src/security/hardware.rs`
  - Action: Remove or implement retry logic

- [ ] **2.4.2** Review unused methods
  - `parse_derivation_path`
  - `validate_transaction_request`
  - `convert_to_legacy_tx`
  - Action: Remove or add `#[allow(dead_code)]` if planned

### 2.5: Keychain Dead Code

- [ ] **2.5.1** Review `service_name` fields (appears twice)
  - File: `src/security/keychain.rs`
  - Action: Remove duplicate or unused instances

### 2.6: Verification

- [ ] **2.6.1** Run `cargo build` after each removal
  - Ensure no compilation errors

- [ ] **2.6.2** Run `cargo test` if tests exist
  - Ensure functionality preserved

- [ ] **2.6.3** Check for new dead code warnings
  - Run `cargo build 2>&1 | grep "never used"`

**Expected Outcome**: Cleaner codebase, no unused fields

---

## Task 3: Split Large Files

**Priority**: Medium (Maintainability)
**Estimated Time**: 1-1.5 hours

### 3.1: Split working_wallet.rs (4,013 lines)

**Current**: Single massive file
**Target**: Modular structure

- [ ] **3.1.1** Create module structure
  ```
  src/gui/working_wallet/
  ├── mod.rs          (main app struct, ~200 lines)
  ├── update.rs       (update logic, ~1500 lines)
  ├── view.rs         (view logic, ~1500 lines)
  ├── subscriptions.rs (subscriptions, ~300 lines)
  └── commands.rs     (command helpers, ~500 lines)
  ```

- [ ] **3.1.2** Extract update logic to `update.rs`
  - Move all `Message` handling
  - Keep imports minimal

- [ ] **3.1.3** Extract view logic to `view.rs`
  - Move all UI rendering code
  - Keep widget creation separate

- [ ] **3.1.4** Extract subscriptions to `subscriptions.rs`
  - Move subscription logic
  - Keep event handling clean

- [ ] **3.1.5** Extract commands to `commands.rs`
  - Move command creation helpers
  - Reduce duplication

- [ ] **3.1.6** Update `mod.rs` to re-export
  - Keep public API unchanged
  - Use `pub use` for exports

- [ ] **3.1.7** Test compilation
  - Run `cargo build`
  - Fix any import issues

### 3.2: Split seed.rs (2,887 lines) - Optional

**Current**: Single large file
**Target**: Logical modules

- [ ] **3.2.1** Create module structure (if time permits)
  ```
  src/security/seed/
  ├── mod.rs          (main exports)
  ├── generation.rs   (seed generation)
  ├── validation.rs   (seed validation)
  ├── analysis.rs     (seed analysis)
  └── recovery.rs     (seed recovery)
  ```

- [ ] **3.2.2** Split functionality
  - Move related functions to appropriate modules

- [ ] **3.2.3** Test compilation
  - Ensure no regressions

**Note**: This is optional - seed.rs is acceptable as-is for security-critical code

### 3.3: Verification

- [ ] **3.3.1** Run full build
  - `cargo build --release`

- [ ] **3.3.2** Check file sizes
  - Ensure no file > 1500 lines

- [ ] **3.3.3** Test application
  - Run wallet and test basic functions

**Expected Outcome**: More maintainable file structure

---

## Task 4: Final Verification

**Priority**: High (Quality assurance)
**Estimated Time**: 15 minutes

- [ ] **4.1** Run complete build
  - `cargo build --release`
  - Ensure clean compilation

- [ ] **4.2** Run clippy check
  - `cargo clippy --all-targets`
  - Verify minimal warnings

- [ ] **4.3** Check warning count
  - Count remaining warnings
  - Document any that remain

- [ ] **4.4** Test basic functionality
  - Launch wallet
  - Test account creation
  - Test transaction sending

- [ ] **4.5** Update CODE_AUDIT_REPORT.md
  - Mark completed tasks
  - Document final state

**Expected Outcome**: Production-ready codebase

---

## Success Criteria

### Must Have (Before Public Release)
- ✅ Zero compilation errors
- ✅ Clippy auto-fixes applied
- ✅ Code formatted consistently
- ✅ Dead code reviewed and addressed

### Should Have (Quality Improvements)
- ✅ working_wallet.rs split into modules
- ✅ Unused fields removed
- ✅ File sizes under 1500 lines

### Nice to Have (Future Improvements)
- ⏳ seed.rs split (optional)
- ⏳ Additional integration tests
- ⏳ Performance benchmarks

---

## Execution Order

1. **Task 1** - Auto-fix (15 min) ← Start here
2. **Task 2** - Dead code (1-1.5 hours)
3. **Task 3** - Split files (1-1.5 hours)
4. **Task 4** - Verification (15 min)

**Total Time**: 2-3 hours

---

## Rollback Plan

If any task causes issues:

1. **Check git status**: `git status`
2. **Review changes**: `git diff`
3. **Rollback if needed**: `git checkout -- <file>`
4. **Test build**: `cargo build`

---

## Notes

- Each task is independent and can be done separately
- Task 1 is quick and safe - do it first
- Task 2 requires careful review - take your time
- Task 3 is the most complex - test thoroughly
- Commit after each major task

---

## Completion Checklist

- [ ] All tasks completed
- [ ] Code compiles cleanly
- [ ] Clippy warnings minimal
- [ ] Application tested
- [ ] Changes committed
- [ ] Ready for public release

---

*Plan created: November 19, 2025*
*Status: Ready to execute*
*Priority: Medium (improves quality, not blocking release)*
