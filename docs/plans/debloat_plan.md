# Enhanced Debloat Plan for Vaughan Wallet

## Critical Issues Identified

**🚨 Immediate Problems:**
1. **Monster File**: `working_wallet.rs` (5,309 lines) - classic god object
2. **Duplicate State Systems**: Both old AppState and NewAppState coexisting
3. **Cross-Module Dependencies**: View components implementing business logic
4. **Stub Functions**: Unfinished hardware wallet implementations scattered throughout

## Phase 1: Emergency Triage (1-2 days)
**Branch: `debloat/phase1-triage`**

### 1.1 Split the 5,309-line Monster
```
working_wallet.rs →
├── app.rs (Application trait impl)
├── commands/ (command handling)
├── subscriptions/ (event subscriptions)
└── update/ (message routing)
```

### 1.2 State System Unification
- Remove dual AppState definitions
- Consolidate into single state management approach
- Eliminate `pub type AppState = NewAppState` aliasing

### 1.3 Remove Dead Code
- Delete all stub functions (hardware wallet placeholders)
- Remove unused imports and commented code
- Clean up temporary implementations

## Phase 2: Architecture Separation (3-4 days)
**Branch: `debloat/phase2-separation`**

### 2.1 Domain Boundaries
```
src/
├── core/           # Business logic (no GUI deps)
│   ├── wallet/
│   ├── security/
│   ├── network/
│   └── transactions/
├── gui/            # UI layer only
│   ├── views/
│   ├── components/
│   └── state/
└── services/       # Application services
    ├── wallet_service.rs
    ├── network_service.rs
    └── transaction_service.rs
```

### 2.2 Remove Business Logic from Views
- Extract wallet operations from `main_wallet.rs` (841 lines)
- Move dialog logic from `dialogs.rs` (1,383 lines) to services
- Pure view functions only

### 2.3 Service Layer Creation
- Centralize all business operations
- Clear async boundaries
- Proper error handling chains

## Phase 3: State Architecture (2-3 days)
**Branch: `debloat/phase3-state`**

### 3.1 Single State Store Pattern
```rust
pub struct AppState {
    // Core domains
    wallet: WalletState,
    network: NetworkState,
    transaction: TransactionState,
    ui: UiState,

    // No coordinators in state!
}

// Separate coordinator layer
pub struct AppCoordinators {
    wallet: WalletCoordinator,
    network: NetworkCoordinator,
    transaction: TransactionCoordinator,
}
```

### 3.2 Event-Driven Updates
- Commands return state changes, not side effects
- Clear data flow: Event → Service → State → View
- Remove direct state mutations from views

## Phase 4: Security & Performance (2-3 days)
**Branch: `debloat/phase4-security`**

### 4.1 Reduce Surface Area
- Consolidate security modules (currently 15 files)
- Single keystore implementation
- Remove redundant validation layers

### 4.2 Memory Management
- Lazy loading for large state objects
- Clear ownership patterns
- Remove Arc<RwLock<T>> unless truly needed

## Phase 5: Final Optimization (1-2 days)
**Branch: `debloat/phase5-optimize`**

### 5.1 Dependency Cleanup
```bash
# Before: Check actual dependencies
cargo tree --duplicates
cargo machete  # Remove unused deps
```

### 5.2 Module Consolidation
- Merge related small modules
- Remove unnecessary abstraction layers
- Simplify public APIs

## Key Metrics & Validation

### Before/After Targets:
- `working_wallet.rs`: 5,309 → <500 lines
- Total GUI module count: 89 → <50 files
- State modules: 8 separate → 4 consolidated
- Build time improvement: 20-30%

### Automated Validation:
```bash
# After each phase
cargo clippy -- -D warnings
cargo test
cargo check --quiet
env VAUGHAN_SOFTWARE_RENDERING=1 timeout 8s ./target/debug/vaughan
```

### Critical Success Criteria:
1. **No feature regression** - all current functionality works
2. **Clear boundaries** - no business logic in views
3. **Single source of truth** - one state system
4. **Performance improvement** - faster builds and runtime

## Implementation Strategy

**Sequential, not parallel phases** - each phase must be complete before the next
**Feature freeze** during debloat - no new functionality
**Continuous testing** - GUI functionality verified after each major change
**Rollback readiness** - each phase branch can revert cleanly

This plan addresses your architectural debt systematically while maintaining stability. The 5,309-line monster file is the biggest win - splitting it will immediately improve maintainability and development velocity.

## Rollback Points

### Critical Checkpoints:
- After Phase 1: Basic functionality maintained, file structure improved
- After Phase 2: Domain separation complete, service layer established
- After Phase 3: State management unified, event flow clarified
- After Phase 4: Security consolidated, performance optimized
- After Phase 5: Final cleanup complete, all metrics achieved

### Emergency Rollback Strategy:
Each phase creates a tagged checkpoint that can be reverted to if issues arise:
```bash
git tag phase-1-complete
git tag phase-2-complete
# etc.
```

## Success Metrics Dashboard

Track progress with these concrete measurements:

### Code Complexity:
- Lines per file (target: <800 for any single file)
- Cyclomatic complexity (target: <10 per function)
- Module coupling (minimize cross-domain imports)

### Performance:
- Build time (debug/release)
- Binary size
- Memory usage at runtime
- GUI responsiveness

### Maintainability:
- Number of files touching core wallet logic
- Test coverage per domain
- Documentation coverage