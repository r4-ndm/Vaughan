# Pre-Tauri Migration Checklist

## Current Status
- ✅ Phase D: Controllers created and tested (100%)
- ⚠️ Phase E: Handler bridges partially complete (60%)
- ✅ Phase F Lite: Headless controller tests (100%)
- ✅ Bug fixes: Transaction flow working
- 🔄 Token balance fix: Ready for testing

---

## Critical: Test Before Migration

### 1. Test Token Balance Fix 🔴 HIGH PRIORITY
**Why**: Verify the fix works before migrating to Tauri

**Test on each network**:
- [ ] Ethereum Mainnet - Check USDC, USDT, WETH balances
- [ ] BSC - Check USDT, BUSD, CAKE balances
- [ ] Polygon - Check USDC, USDT, WETH balances
- [ ] PulseChain Testnet v4 - Verify still works (regression test)

**Expected logs**:
```
✅ Initialized token_balances with X popular tokens for network Y
✅ Received updated token balances: [("USDC", "1.000000"), ...]
```

**If it works**: ✅ Proceed to Tauri  
**If it fails**: ❌ Debug before migrating

---

### 2. Test Transaction Flow 🟡 MEDIUM PRIORITY
**Why**: Ensure all 4 bug fixes are working

**Test scenarios**:
- [ ] Paste address from clipboard
- [ ] Enter amount with different formats (1.5 tPLS, 0.001 ETH)
- [ ] Password dialog appears correctly
- [ ] Password submission works (no infinite loop)
- [ ] Transaction submits successfully

**If it works**: ✅ Proceed to Tauri  
**If it fails**: ❌ Debug before migrating

---

### 3. Manual GUI Smoke Test 🟢 LOW PRIORITY
**Why**: Catch any obvious regressions

**Quick checks** (5 minutes):
- [ ] Wallet opens without errors
- [ ] Can switch networks
- [ ] Can switch accounts
- [ ] Balances display
- [ ] Send form works
- [ ] No console errors

---

## Recommended: Clean Up Before Migration

### 4. Commit Current State 🟡 RECOMMENDED
**Why**: Clean git history for Tauri migration

```bash
# Review changes
git status
git diff

# Commit token balance fix
git add src/gui/working_wallet.rs
git commit -m "fix: correct ERC20 token addresses for multi-network support

- Fixed Ethereum USDC address
- Added BSC token addresses (USDT, BUSD, CAKE)
- Added Polygon token addresses (USDC, USDT, WETH)
- Removed incorrect PulseChain mainnet addresses

Fixes token balance display on Ethereum, BSC, and Polygon.
Addresses verified from official block explorers."

# Commit controller tests
git add tests/controller_properties.rs
git commit -m "test(controllers): Add comprehensive property-based tests

- 9 property tests for controllers
- Tests validation logic, gas limits, overflow prevention
- Tests wallet, network, and price controllers
- All tests passing (20/20 total)"

# Commit documentation
git add docs/
git commit -m "docs: Add Phase F Lite and token balance fix documentation

- Phase F Lite completion report
- Token balance fix analysis and solution
- Pre-Tauri migration checklist"

# Push to GitHub
git push origin main
```

---

### 5. Document Controller Transfer Plan 🟢 OPTIONAL
**Why**: Make Tauri migration smoother

Create `TAURI_MIGRATION_PLAN.md`:
- [ ] List which controllers transfer 100% (all 4)
- [ ] Document E2/E3 implementation in Tauri
- [ ] Note Iced-specific code to remove
- [ ] Plan for state management in Tauri

**Time**: 15 minutes  
**Benefit**: Smoother migration

---

### 6. Clean Up Temporary Files 🟢 OPTIONAL
**Why**: Cleaner codebase for migration

**Remove build artifacts**:
```bash
# Clean build artifacts
cargo clean

# Remove test output files (optional)
rm -f *.txt *.log
# Or on Windows:
del *.txt *.log
```

**Keep**:
- Documentation (docs/)
- Test files (tests/)
- Source code (src/)
- Configuration (Cargo.toml, etc.)

---

### 7. Run Final Validation 🟡 RECOMMENDED
**Why**: Ensure everything compiles and tests pass

```bash
# Check compilation
cargo check --all-features

# Run all tests
cargo test --all-features

# Run controller tests specifically
cargo test --test controllers_integration --test controller_properties

# Build release (optional, takes time)
cargo build --release
```

**Expected**:
- ✅ Compilation succeeds
- ✅ 20/20 controller tests pass
- ✅ All other tests pass
- ⚠️ 4 minor warnings (acceptable)

---

## Optional: Nice-to-Have Before Migration

### 8. Extract Reusable Types 🟢 OPTIONAL
**Why**: Make types available for Tauri

**Consider extracting to separate crate**:
- Controllers (already in `src/controllers/`)
- Network types (already in `src/network/`)
- Security types (already in `src/security/`)

**Benefit**: Can import as dependency in Tauri  
**Time**: 30-60 minutes  
**Priority**: LOW (can do during migration)

---

### 9. Document Known Issues 🟢 OPTIONAL
**Why**: Track what needs fixing in Tauri

Create `KNOWN_ISSUES.md`:
- [ ] E2/E3 blocked by Iced (will fix in Tauri)
- [ ] PulseChain mainnet token addresses missing
- [ ] 4 minor clippy warnings
- [ ] Any other issues found during testing

**Time**: 10 minutes  
**Benefit**: Clear migration goals

---

### 10. Benchmark Performance 🟢 OPTIONAL
**Why**: Compare Iced vs Tauri performance

```bash
# Run benchmarks (if they exist)
cargo bench

# Or just note current performance
# - Startup time
# - Transaction time
# - Network switch time
```

**Time**: 15 minutes  
**Benefit**: Performance comparison data

---

## What NOT to Do

### ❌ Don't Refactor Major Code
**Why**: Save it for Tauri migration
- Don't restructure handlers
- Don't change state management
- Don't modify core logic

**Exception**: Bug fixes are OK

### ❌ Don't Add New Features
**Why**: Focus on migration readiness
- Don't add new networks
- Don't add new token support
- Don't add new UI features

**Exception**: Critical fixes only

### ❌ Don't Optimize Prematurely
**Why**: Tauri will be different
- Don't optimize Iced-specific code
- Don't optimize GUI rendering
- Don't optimize state updates

**Exception**: Controller optimization is OK (transfers to Tauri)

---

## Priority Ranking

### 🔴 MUST DO (Before Tauri)
1. ✅ Test token balance fix on multiple networks
2. ✅ Test transaction flow end-to-end
3. ✅ Commit current state to git

**Time**: 30-45 minutes  
**Blocker**: Yes - don't migrate without testing

### 🟡 SHOULD DO (Recommended)
4. ✅ Run final validation (cargo test)
5. ✅ Manual GUI smoke test
6. ✅ Push to GitHub

**Time**: 15-30 minutes  
**Blocker**: No - but highly recommended

### 🟢 NICE TO HAVE (Optional)
7. Document controller transfer plan
8. Clean up temporary files
9. Extract reusable types
10. Document known issues
11. Benchmark performance

**Time**: 60-90 minutes  
**Blocker**: No - can do during migration

---

## Estimated Time

### Minimum (Must Do Only)
- Test token balances: 15 minutes
- Test transactions: 10 minutes
- Commit and push: 10 minutes
- **Total: 35 minutes**

### Recommended (Must + Should)
- Above: 35 minutes
- Final validation: 10 minutes
- GUI smoke test: 5 minutes
- **Total: 50 minutes**

### Complete (Everything)
- Above: 50 minutes
- Optional tasks: 60 minutes
- **Total: 110 minutes (~2 hours)**

---

## Decision Matrix

### If You Have 30 Minutes
✅ Do: Test token balances, test transactions, commit  
⏭️ Skip: Everything else

### If You Have 1 Hour
✅ Do: Above + final validation + GUI test + push  
⏭️ Skip: Optional tasks

### If You Have 2 Hours
✅ Do: Everything on this checklist  
🎯 Result: Perfectly clean state for Tauri

---

## Ready for Tauri When...

### Minimum Requirements ✅
- [X] Controllers created and tested (Phase D)
- [X] Controllers are framework-agnostic
- [X] 20/20 controller tests passing
- [ ] Token balance fix tested and working
- [ ] Transaction flow tested and working
- [ ] Current state committed to git

### Ideal State ✅✅
- All minimum requirements above
- [ ] Final validation passing
- [ ] GUI smoke test complete
- [ ] Changes pushed to GitHub
- [ ] Documentation complete
- [ ] Known issues documented

---

## What Transfers to Tauri

### ✅ Transfers 100%
- All 4 controllers (Transaction, Network, Wallet, Price)
- All controller tests (20 tests)
- Network types and logic
- Security types and logic
- Utility functions
- Error types

### ⚠️ Needs Adaptation
- GUI code (Iced → Tauri)
- State management (Iced → Tauri)
- Message passing (Iced → Tauri commands)
- Event handling (Iced → Tauri events)

### ❌ Won't Transfer
- Iced-specific code
- `working_wallet.rs` (will be rewritten)
- Iced message types
- Iced command types

---

## Tauri Migration Benefits

### What You'll Gain
1. ✅ E2/E3 will work (controller initialization)
2. ✅ Better state management
3. ✅ Smaller binary size
4. ✅ Better performance
5. ✅ Web technologies for UI
6. ✅ Easier debugging
7. ✅ Better cross-platform support

### What You'll Keep
1. ✅ All controllers (100% transfer)
2. ✅ All business logic
3. ✅ All tests
4. ✅ All security code
5. ✅ All network code

---

## Final Recommendation

### Absolute Minimum (30 min)
1. Test token balance fix
2. Test transaction flow
3. Commit changes

**Then**: Start Tauri migration

### Recommended (1 hour)
1. Above
2. Run final validation
3. GUI smoke test
4. Push to GitHub

**Then**: Start Tauri migration with confidence

### Ideal (2 hours)
1. Above
2. Document migration plan
3. Clean up files
4. Document known issues

**Then**: Start Tauri migration perfectly prepared

---

## Questions to Answer

Before starting Tauri migration, answer:

1. ✅ Do token balances work on all networks?
2. ✅ Do transactions submit successfully?
3. ✅ Are all controller tests passing?
4. ✅ Is current state committed to git?
5. ❓ Do you have a Tauri project structure ready?
6. ❓ Do you know which UI framework to use in Tauri?
7. ❓ Do you have a state management plan for Tauri?

**If you answered ✅ to 1-4**: You're ready to start!  
**If you answered ❓ to 5-7**: Plan these during migration

---

## Next Steps

1. **Now**: Complete this checklist (30-120 min)
2. **Then**: Start Tauri migration
3. **During migration**: 
   - Copy controllers to Tauri project
   - Implement E2/E3 (controller initialization)
   - Build new UI with Tauri
   - Run full Phase F in Tauri

**Good luck with the Tauri migration!** 🚀
