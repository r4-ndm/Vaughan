# Phase E: Deprecated Field Removal - Execution Plan

## Objective
Remove all deprecated fields from AppState to achieve 100% clean codebase with zero deprecation warnings.

## Current Status
- **Compilation**: ✅ Passing
- **AppState Warnings**: 139 (all in Default implementation)
- **Real Code**: ✅ 100% migrated to domain accessors

## Strategy

### Step 1: Identify Deprecated Fields
All deprecated fields are in `src/gui/state/mod.rs` AppState struct.

### Step 2: Remove Deprecated Fields
Remove the deprecated field declarations from AppState struct.

### Step 3: Update Default Implementation
Update the Default impl to only initialize non-deprecated fields.

### Step 4: Validation
- Ensure compilation succeeds
- Verify zero deprecation warnings
- Test basic functionality

## Execution

### Phase E.1: Remove Deprecated Fields from AppState Struct
Remove all fields marked with `#[deprecated]` attribute.

### Phase E.2: Update Default Implementation
Update `impl Default for AppState` to remove deprecated field initializations.

### Phase E.3: Final Validation
- Run cargo build
- Check for warnings
- Verify functionality

## Risk Assessment
- **Risk Level**: 🟢 LOW
- **Reason**: All code already migrated to use accessors
- **Rollback**: Git revert if needed

## Expected Outcome
- Zero deprecation warnings
- Clean, professional codebase
- 100% domain separation achieved
