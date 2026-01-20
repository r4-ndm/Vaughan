# Send Button Fix - Critical Bug Resolution

## Issue Summary
The send button in Vaughan wallet was permanently disabled, preventing users from sending transactions even when all required fields were filled.

## Root Cause
The send button's enable condition required `send_from_account_id` to be set:

```rust
// src/gui/views/main_wallet.rs:675-679
.on_press_maybe(
    if !self.transaction().send_to_address.is_empty()
        && !self.transaction().send_amount.is_empty()
        && self.transaction().send_from_account_id.is_some()  // ← This was never set!
        && !self.transaction().sending_transaction
    {
        Some(Message::SubmitTransaction)
    } else {
        None
    },
)
```

However, when users selected an account from the dropdown, only `current_account_id` was updated, not `send_from_account_id`. This meant the send button remained disabled indefinitely.

## Technical Details

### The Problem
1. User selects account from dropdown → triggers `Message::AccountSelected`
2. Handler updates `current_account_id` but NOT `send_from_account_id`
3. Send button checks for `send_from_account_id.is_some()` → returns false
4. Button remains disabled

### The Solution
Updated all account selection paths to synchronize both fields:

#### 1. Account Selection Handler
**File:** `src/gui/handlers/wallet_ops.rs`

```rust
fn handle_account_selected(&mut self, account_id: String) -> Command<Message> {
    self.state.wallet_mut().current_account_id = Some(account_id.clone());
    // FIX: Also set the send_from_account_id so the send button becomes enabled
    self.state.transaction_mut().send_from_account_id = Some(account_id.clone());
    self.state.last_activity = Instant::now();
    
    tracing::info!("Account selected: {} (also set as send_from account)", account_id);
    
    self.dispatch_message(Message::RefreshBalance)
}
```

#### 2. Coordinated Account Change
**File:** `src/gui/state/mod.rs`

```rust
pub fn change_account_coordinated(&mut self, account_id: String) -> Vec<iced::Command<crate::gui::Message>> {
    let commands = self.account_coordinator.change_account(account_id.clone());
    self.wallet.current_account_id = Some(account_id.clone());
    // FIX: Also set send_from_account_id so send button works
    self.transaction.send_from_account_id = Some(account_id);
    
    self.loading_coordinator.set_account_loading(true);
    
    tracing::info!("🔄 AppState: Coordinated account change initiated");
    commands
}
```

#### 3. State Management Commands
**File:** `src/gui/state_management/commands.rs`

```rust
Command::SelectAccount { account_id } => {
    let _ = self.store.update_wallet_state(|wallet| {
        wallet.current_account_id = Some(account_id.clone());
    });
    // FIX: Also update send_from_account_id so send button works
    let _ = self.store.update_transaction_state(|transaction| {
        transaction.send_from_account_id = Some(account_id.clone());
    });
    
    self.emit_event(StateEvent::AccountSelected { account_id: account_id.clone() });
    Effect::RefreshAccountData { account_id }
}
```

#### 4. Account Creation/Import
**File:** `src/gui/handlers/wallet_ops.rs`

Updated both `handle_account_created` and `handle_account_imported` to set `send_from_account_id` when a new account is created or imported.

## Testing Verification

### Before Fix
- ❌ Send button always disabled
- ❌ Cannot send transactions
- ❌ No visual feedback on why button is disabled

### After Fix
- ✅ Send button enables when account selected
- ✅ Send button enables when address and amount filled
- ✅ Transaction submission works correctly
- ✅ Account switching properly updates send form

## Impact
- **Severity:** Critical - Core functionality completely broken
- **User Impact:** High - Users unable to send any transactions
- **Fix Complexity:** Low - Simple state synchronization
- **Risk:** Low - Straightforward fix with no side effects

## Files Modified
1. `src/gui/handlers/wallet_ops.rs` - Account selection handler
2. `src/gui/state/mod.rs` - Coordinated state management
3. `src/gui/state_management/commands.rs` - Command processor

## Related Code
- Send button logic: `src/gui/views/main_wallet.rs:670-688`
- Transaction state: `src/gui/state/transaction_state.rs:87`
- Transaction handler: `src/gui/handlers/transaction.rs:21-23`

## Prevention
To prevent similar issues in the future:
1. Consider unifying `current_account_id` and `send_from_account_id` into a single source of truth
2. Add validation tests that check button enable states
3. Add logging when button conditions are not met for debugging

## Date
November 22, 2025

## Status
✅ Fixed and verified
