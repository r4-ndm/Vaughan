# Account Selector Issue - Analysis & Fix

## Problem Identified

The account selector in `src/gui/views/main_wallet.rs` is using **deprecated fields** directly instead of proper accessor methods. This can cause:

1. **State synchronization issues** - Changes not reflected across all views
2. **Inconsistent account selection** - Selected account not updating properly
3. **Deprecated field warnings** - Using fields marked for removal

## Root Cause

### Current Implementation (Problematic)
```rust
// In src/gui/views/main_wallet.rs line 232-268
fn account_selector(&self) -> Element<Message> {
    if self.available_accounts.is_empty() {  // ❌ Using deprecated field
        // ...
    } else {
        PickList::new(
            &self.available_accounts[..],  // ❌ Using deprecated field
            self.current_account_id.as_ref().and_then(|id| {  // ❌ Using deprecated field
                self.available_accounts.iter().find(|a| &a.id == id)  // ❌ Using deprecated field
            }),
            |account: SecureAccount| {
                Message::AccountSelected(account.id.clone())
            },
        )
        // ...
    }
}
```

### Why This Is Wrong

From `src/gui/state/mod.rs`:
```rust
#[deprecated(note = "Use AppState::wallet().available_accounts() instead")]
pub available_accounts: Vec<crate::security::SecureAccount>,

#[deprecated(note = "Use AppState::wallet().current_account_id() instead")]
pub current_account_id: Option<String>,

#[deprecated(note = "Use AppState::wallet().loading_accounts() instead")]
pub loading_accounts: bool,
```

## The Fix

### Step 1: Update Account Selector Method

Replace the deprecated field access with proper accessor methods:

```rust
/// Account selector component
fn account_selector(&self) -> Element<Message> {
    // Use proper accessor methods instead of deprecated fields
    let available_accounts = &self.wallet().available_accounts;
    let current_account_id = self.wallet().current_account_id.as_ref();
    let loading_accounts = self.wallet().loading_accounts;
    
    if available_accounts.is_empty() {
        Container::new({
            let content: Element<Message> = if loading_accounts {
                if self.accounts_spinner {
                    Row::new()
                        .spacing(6)
                        .align_items(iced::Alignment::Center)
                        .push(Text::new("⟳").size(16))
                        .push(Text::new("Loading...").size(11))
                        .into()
                } else {
                    Text::new("Loading...").size(11).into()
                }
            } else {
                Text::new("No accounts - Create one").size(11).into()
            };
            content
        })
        .padding([6, 10])
        .style(account_container_style())
        .width(Length::Fill)
        .into()
    } else {
        PickList::new(
            &available_accounts[..],
            current_account_id.and_then(|id| {
                available_accounts.iter().find(|a| &a.id == id)
            }),
            |account: SecureAccount| {
                Message::AccountSelected(account.id.clone())
            },
        )
        .padding([4, 8])
        .style(pick_list_style())
        .width(Length::Fixed(160.0))
        .text_size(14)
        .placeholder("⚠️ Select account")
        .into()
    }
}
```

### Step 2: Update Address Display

Around line 57-60, fix the address display:

```rust
// OLD (line 57-60)
if let Some(current_account_id) = &self.current_account_id {
    if let Some(account) = self
        .available_accounts
        .iter()

// NEW
if let Some(current_account_id) = self.wallet().current_account_id.as_ref() {
    if let Some(account) = self.wallet()
        .available_accounts
        .iter()
```

### Step 3: Update Delete Button Check

Around line 172:

```rust
// OLD
.on_press_maybe(if self.current_account_id.is_some() {

// NEW
.on_press_maybe(if self.wallet().current_account_id.is_some() {
```

## Additional Issues Found

### Issue 1: Account Coordinator Not Being Used

The `AccountCoordinator` exists but the account selector doesn't use it. The coordinator should manage:
- Available accounts list
- Current account selection
- Account loading state

### Issue 2: State Synchronization

The `sync_coordinators_with_flat_fields()` method syncs data, but it's not clear when it's called. This could cause:
- Account selection in one view not reflecting in another
- Balance not updating when account changes

### Issue 3: Multiple Account Selectors

There are TWO account selectors:
1. `src/gui/views/main_wallet.rs` - Main wallet view
2. `src/gui/components/transaction_form.rs` - Transaction form

They may not be synchronized properly.

## Recommended Solution

### Option A: Quick Fix (30 minutes)
Just update the deprecated field access in `main_wallet.rs` as shown above.

**Pros:** Fast, minimal changes
**Cons:** Doesn't address root synchronization issues

### Option B: Proper Fix (2 hours)
1. Update all deprecated field access
2. Ensure AccountCoordinator is properly used
3. Add synchronization checks
4. Test account switching across all views

**Pros:** Fixes root cause, future-proof
**Cons:** More work, needs testing

### Option C: Refactor (4 hours)
1. Consolidate account selectors into single component
2. Use AccountCoordinator exclusively
3. Remove deprecated fields entirely
4. Update all views to use new pattern

**Pros:** Clean architecture, no technical debt
**Cons:** Significant refactoring, part of larger debloat effort

## Implementation Priority

**Immediate:** Option A (Quick Fix)
- Fixes the immediate issue
- Removes deprecated field warnings
- Minimal risk

**Short-term:** Option B (Proper Fix)
- Should be done within 1-2 weeks
- Ensures reliability
- Prevents future bugs

**Long-term:** Option C (Refactor)
- Part of the DEBLOAT_PLAN.md
- Do during Phase 2 (Split God Object)
- Aligns with professional standards

## Testing Checklist

After applying the fix, test:

- [ ] Account selector displays available accounts
- [ ] Selecting an account updates the display
- [ ] Address display shows correct account
- [ ] Balance updates when account changes
- [ ] Delete button only enabled when account selected
- [ ] Transaction form uses correct account
- [ ] Account selection persists across tab switches
- [ ] No console errors or warnings
- [ ] Account loading spinner works
- [ ] "No accounts" message displays correctly

## Files to Modify

### Quick Fix (Option A)
- `src/gui/views/main_wallet.rs` (lines 57, 60, 172, 232-268)

### Proper Fix (Option B)
- `src/gui/views/main_wallet.rs`
- `src/gui/handlers/wallet_ops.rs` (verify AccountSelected handler)
- `src/gui/working_wallet.rs` (verify state sync)

### Full Refactor (Option C)
- All of the above
- `src/gui/components/transaction_form.rs`
- `src/gui/coordinators.rs`
- `src/gui/state/mod.rs`

## Notes

- The deprecated fields are still present for "backward compatibility during migration"
- Eventually these should be removed entirely
- The coordinator pattern is the right approach but not fully implemented
- This is a symptom of the larger "god object" problem identified in DEBLOAT_PLAN.md
