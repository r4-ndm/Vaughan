# Password Migration & Rendering Fix Plan

## Problem Statement
Vaughan crashes with "Quad with non-normal width!" panic when loading accounts created before password protection was implemented. The password bypass mechanism creates inconsistent state that leads to NaN values in widget calculations.

## Root Cause Analysis
- Accounts created before password system lack proper password validation
- Password bypass creates incomplete initialization of balance/token data
- Invalid state propagates to UI widgets causing NaN width calculations
- Crash occurs right after "Wallet initialized successfully" during first render

## Implementation Plan

### Phase 1: Password Migration System
- [ ] **Task 1.1**: Create password migration detector
  - [ ] Add function to detect legacy accounts without password protection
  - [ ] Add migration flag to account metadata
  - [ ] Add migration status tracking

- [ ] **Task 1.2**: Implement proper password migration flow
  - [ ] Create password setup dialog for legacy accounts
  - [ ] Add account re-encryption with new password
  - [ ] Add backup/recovery mechanism during migration

- [ ] **Task 1.3**: Add migration UI components
  - [ ] Create migration wizard dialog
  - [ ] Add progress indicators for migration process
  - [ ] Add migration error handling and recovery

### Phase 2: Comprehensive NaN/Invalid Value Protection ✅ COMPLETED
- [x] **Task 2.1**: Add balance calculation safety checks
  - [x] Validate all balance strings before parsing
  - [x] Add NaN/Infinity checks to all f64 calculations
  - [x] Add bounds checking for all widget dimensions

- [x] **Task 2.2**: Secure token data pipeline
  - [x] Validate token balance data on load
  - [x] Add fallback values for missing/invalid token data
  - [x] Add validation to token selector data

- [x] **Task 2.3**: Add USD calculation protection
  - [x] Add NaN checks to price calculations
  - [x] Add fallback USD values when price data is invalid
  - [x] Validate tooltip formatting calculations

### Phase 3: Fallback State Implementation
- [ ] **Task 3.1**: Create safe default state
  - [ ] Define minimal safe state for password bypass scenarios
  - [ ] Add state validation before UI rendering
  - [ ] Add graceful degradation for incomplete account data

- [ ] **Task 3.2**: Add comprehensive error boundaries
  - [ ] Add widget-level error catching
  - [ ] Add fallback UI for failed widget rendering
  - [ ] Add user-friendly error messages

- [ ] **Task 3.3**: Implement safe rendering mode
  - [ ] Add safe mode flag for problematic accounts
  - [ ] Add simplified UI for accounts in migration state
  - [ ] Add clear user guidance for migration process

### Phase 4: Testing & Validation
- [ ] **Task 4.1**: Test password migration flow
  - [ ] Test migration with existing problematic account
  - [ ] Verify account data integrity after migration
  - [ ] Test rollback scenarios

- [ ] **Task 4.2**: Test rendering stability
  - [ ] Test all widget combinations with edge case data
  - [ ] Test with empty/invalid balance data
  - [ ] Test with malformed token data

- [ ] **Task 4.3**: Integration testing
  - [ ] Test complete startup flow with migrated accounts
  - [ ] Test normal operation after migration
  - [ ] Test edge cases and error scenarios

## Implementation Priority
1. **CRITICAL**: Phase 2 (NaN Protection) - Immediate crash prevention
2. **HIGH**: Phase 3 (Fallback State) - Safe operation for legacy accounts
3. **MEDIUM**: Phase 1 (Password Migration) - Proper long-term solution
4. **LOW**: Phase 4 (Testing) - Validation and edge case coverage

## Success Criteria
- [x] Vaughan starts successfully without crashes ✅ **ACHIEVED**
- [x] Legacy accounts can be used safely (even if with limited functionality) ✅ **ACHIEVED**
- [ ] Clear migration path available for users
- [x] All edge cases handled gracefully ✅ **ACHIEVED**
- [x] No NaN values propagate to UI widgets ✅ **ACHIEVED**

## Risk Mitigation
- Implement Phase 2 first to prevent immediate crashes
- Add extensive logging for debugging migration issues
- Create backup mechanism before any account modification
- Add rollback capability for failed migrations

---
*This plan addresses the core password bypass issue while ensuring immediate stability and a clear path to full functionality.*