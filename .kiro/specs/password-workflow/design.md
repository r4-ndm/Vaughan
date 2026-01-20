# Design Document: Password Workflow Enhancement

## Overview

This design implements a secure, user-friendly password authentication workflow for the Vaughan wallet. The system ensures that seed-based accounts require password authentication on startup and for transaction signing, while maintaining usability through session management and optional key caching.

The design addresses the current gap where password infrastructure exists but isn't properly integrated into the application lifecycle. It introduces a startup authentication gate, session-aware transaction signing, and proper key cache management.

## Architecture

### High-Level Flow

```
Application Start
    ↓
Check for Seed-Based Accounts
    ↓
    ├─→ No Seed Accounts → Start Unlocked
    │
    └─→ Has Seed Accounts → Show Password Dialog
            ↓
            ├─→ Correct Password → Unlock Session → Load Wallet Data
            │
            └─→ Incorrect/Cancel → Show Locked State View
```

### Transaction Flow

```
User Initiates Transaction
    ↓
Check Account Type
    ↓
    ├─→ Private Key Account → Show Transaction Confirmation
    │
    └─→ Seed-Based Account
            ↓
            Check Session State
            ↓
            ├─→ Unlocked → Show Transaction Confirmation
            │
            └─→ Locked → Show Password Dialog
                    ↓
                    ├─→ Correct Password → Unlock → Show Transaction Confirmation
                    │
                    └─→ Incorrect/Cancel → Cancel Transaction
```

### Session Management Flow

```
Session Unlocked
    ↓
User Activity → Reset Inactivity Timer
    ↓
No Activity for 15 Minutes
    ↓
Auto-Lock Session
    ↓
Clear Cached Keys
    ↓
Show Lock Notification
```

## Components and Interfaces

### 1. Startup Authentication Gate

**Purpose:** Intercept application startup to require authentication before loading sensitive data.

**Location:** `src/gui/working_wallet.rs` - `Application::new()` method

**Interface:**
```rust
// New startup flow in Application::new()
fn new(_flags: ()) -> (Self, Command<Message>) {
    let state = AppState::default();
    
    // Check if we have seed-based accounts
    let check_accounts_cmd = Command::perform(
        check_for_seed_accounts(),
        Message::SeedAccountsChecked
    );
    
    (app, check_accounts_cmd)
}

// New message handler
Message::SeedAccountsChecked(has_seed_accounts) => {
    if has_seed_accounts {
        // Show password dialog for startup unlock
        self.state.security_mut().password_dialog.show(
            PasswordReason::UnlockSession
        );
        Command::none()
    } else {
        // No seed accounts, proceed with normal startup
        self.state.security_mut().session.unlock();
        self.start_normal_initialization()
    }
}
```

**Key Responsibilities:**
- Detect presence of seed-based accounts
- Show password dialog before loading wallet data
- Handle authentication success/failure
- Provide locked state view if authentication fails

### 2. Account Type Detection Service

**Purpose:** Identify whether accounts are seed-based or private-key based.

**Location:** `src/gui/services/account_service.rs`

**Interface:**
```rust
/// Check if any seed-based accounts exist
pub async fn check_for_seed_accounts() -> Result<bool, String> {
    // Load accounts from keystore
    // Check KeyReference type for each account
    // Return true if any are seed-based
}

/// Get account type for a specific account
pub fn get_account_type(account: &SecureAccount) -> AccountType {
    match &account.key_reference.key_type {
        KeyType::Seed { .. } => AccountType::SeedBased,
        KeyType::PrivateKey { .. } => AccountType::PrivateKey,
    }
}

pub enum AccountType {
    SeedBased,
    PrivateKey,
}
```

### 3. Session-Aware Transaction Handler

**Purpose:** Check session state before allowing transaction signing.

**Location:** `src/gui/handlers/transaction.rs`

**Modifications:**
```rust
fn handle_confirm_transaction(&mut self) -> Command<Message> {
    // Get current account
    let account = self.get_current_account();
    
    // Check account type
    match get_account_type(account) {
        AccountType::PrivateKey => {
            // Private key accounts don't need password
            self.proceed_with_transaction_signing()
        }
        AccountType::SeedBased => {
            // Check if session is unlocked
            if self.state.security().session.is_unlocked {
                self.proceed_with_transaction_signing()
            } else {
                // Session locked, show password dialog
                self.state.security_mut().password_dialog.show(
                    PasswordReason::SignTransaction {
                        tx_details: self.get_transaction_summary()
                    }
                );
                Command::none()
            }
        }
    }
}
```

### 4. Enhanced Password Validation Handler

**Purpose:** Handle password validation with proper session unlocking and key caching.

**Location:** `src/gui/handlers/security.rs`

**Enhancements:**
```rust
fn handle_password_validated(
    &mut self,
    result: Result<SecretString, PasswordError>,
) -> Command<Message> {
    match result {
        Ok(seed_phrase) => {
            let remember = self.state.security().password_dialog.remember_session;
            let reason = self.state.security().password_dialog.reason.clone();
            
            // Unlock session
            self.state.security_mut().session.unlock();
            
            // Cache seed if remember is enabled
            if remember {
                self.state.security_mut().session.cached_password = Some(seed_phrase);
            }
            
            // Hide password dialog
            self.state.security_mut().password_dialog.hide();
            
            // Continue with the action that triggered password request
            match reason {
                Some(PasswordReason::UnlockSession) => {
                    // Startup unlock - proceed with initialization
                    self.start_normal_initialization()
                }
                Some(PasswordReason::SignTransaction { .. }) => {
                    // Transaction signing - continue to confirmation
                    self.dispatch_message(Message::ShowTransactionConfirmation)
                }
                _ => Command::none()
            }
        }
        Err(error) => {
            // Handle error (already implemented)
            self.state.security_mut().password_dialog.set_error(error);
            Command::none()
        }
    }
}
```

### 5. Session Timeout Monitor

**Purpose:** Automatically lock session after inactivity period.

**Location:** `src/gui/working_wallet.rs` - `subscription()` method

**Interface:**
```rust
fn subscription(&self) -> Subscription<Message> {
    let mut subscriptions = vec![
        // Existing subscriptions...
        
        // Session timeout check every 10 seconds
        iced::time::every(Duration::from_secs(10))
            .map(|_| Message::SessionTimeoutCheck),
    ];
    
    Subscription::batch(subscriptions)
}
```

### 6. Locked State View

**Purpose:** Display a locked wallet view when session is locked.

**Location:** `src/gui/views/locked_view.rs` (new file)

**Interface:**
```rust
pub fn locked_view(state: &AppState) -> Element<Message> {
    Container::new(
        Column::new()
            .push(Icon::lock())
            .push(Text::new("Wallet Locked"))
            .push(Text::new("Enter your password to unlock"))
            .push(
                Button::new("Unlock")
                    .on_press(Message::ShowPasswordDialog {
                        reason: PasswordReason::UnlockSession
                    })
            )
    )
    .center_x()
    .center_y()
    .into()
}
```

### 7. Session Indicator Component

**Purpose:** Display current session state in the UI.

**Location:** `src/gui/components/session_indicator.rs` (already exists)

**Enhancements:**
```rust
pub fn session_indicator(state: &AppState) -> Element<Message> {
    let session = &state.security().session;
    
    if session.is_unlocked {
        let time_remaining = session.time_until_timeout();
        Row::new()
            .push(Icon::unlock())
            .push(Text::new(format!("Unlocked ({})", format_time(time_remaining))))
            .push(
                Button::new("Lock")
                    .on_press(Message::ManualLock)
            )
    } else {
        Row::new()
            .push(Icon::lock())
            .push(Text::new("Locked"))
            .push(
                Button::new("Unlock")
                    .on_press(Message::ShowPasswordDialog {
                        reason: PasswordReason::UnlockSession
                    })
            )
    }
}
```

## Data Models

### SecurityState (existing, with enhancements)

```rust
pub struct SecurityState {
    pub password_dialog: PasswordDialogState,
    pub session: SessionState,
    pub key_cache_handle: Option<Arc<RwLock<KeyCache>>>,
}

pub struct SessionState {
    pub is_unlocked: bool,
    pub unlocked_at: Option<Instant>,
    pub last_activity: Instant,
    pub timeout_duration: Duration,  // Default: 15 minutes
    pub auto_lock_enabled: bool,
    pub lock_on_minimize: bool,
    pub cached_password: Option<SecretString>,  // Cleared on lock
}
```

### New Message Types

```rust
// Add to Message enum
pub enum Message {
    // ... existing messages ...
    
    // Startup authentication
    SeedAccountsChecked(bool),
    StartupAuthenticationRequired,
    StartupAuthenticationComplete,
    
    // Session state changes
    SessionStateChanged(SessionState),
    
    // Activity tracking
    UserInteraction,  // Fired on any user action
}
```

### AccountType Enum (new)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    SeedBased,
    PrivateKey,
}
```

## Error Handling

### Password Validation Errors

All password errors are handled through the existing `PasswordError` enum:

```rust
pub enum PasswordError {
    IncorrectPassword { attempts_remaining: u32 },
    DecryptionFailed,
    EmptyPassword,
    TooManyAttempts { retry_after_seconds: u64 },
    SessionExpired,
    AccountLocked { retry_after_seconds: u64 },
}
```

### Startup Authentication Failures

- **No Password Entered:** Display locked state view with retry option
- **Incorrect Password:** Show error in password dialog, allow retry
- **Too Many Attempts:** Temporarily lock with countdown timer
- **Keystore Access Error:** Display error message with option to retry or exit

### Transaction Signing Failures

- **Session Expired During Transaction:** Show password dialog with transaction context
- **Password Incorrect:** Display error, allow retry without losing transaction data
- **User Cancels Password:** Cancel transaction, return to send dialog

## Testing Strategy

### Unit Tests

1. **Account Type Detection**
   - Test identification of seed-based accounts
   - Test identification of private-key accounts
   - Test mixed account scenarios

2. **Session State Management**
   - Test session unlock/lock transitions
   - Test timeout calculation
   - Test activity tracking
   - Test cached password clearing

3. **Password Validation**
   - Test correct password handling
   - Test incorrect password handling
   - Test empty password rejection
   - Test attempt tracking

### Integration Tests

1. **Startup Flow**
   - Test startup with seed accounts (should show password)
   - Test startup without seed accounts (should skip password)
   - Test startup authentication success
   - Test startup authentication failure

2. **Transaction Flow**
   - Test transaction with unlocked session
   - Test transaction with locked session
   - Test transaction with private-key account
   - Test transaction with seed-based account

3. **Session Management**
   - Test auto-lock after timeout
   - Test activity extension
   - Test manual lock
   - Test key cache clearing

### Property-Based Tests

Property-based tests will be defined after prework analysis in the next section.



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Startup Password Prompt for Seed Accounts

*For any* wallet state containing at least one seed-based account, launching the application should display the password dialog with reason "UnlockSession" before loading sensitive data.

**Validates: Requirements 1.1, 9.1, 9.2**

### Property 2: Correct Password Unlocks Session

*For any* seed-based account and its correct password, entering the password should result in session state transitioning to unlocked with cached seed stored in memory.

**Validates: Requirements 1.2**

### Property 3: Incorrect Password Shows Error

*For any* seed-based account and an incorrect password, password validation should display an error message and increment the attempt counter without unlocking the session.

**Validates: Requirements 1.3, 6.2**

### Property 4: Unlocked Session Allows Transactions

*For any* seed-based account with an unlocked session, initiating a transaction should proceed directly to transaction confirmation without displaying a password dialog.

**Validates: Requirements 2.1**

### Property 5: Locked Session Requires Password for Transactions

*For any* seed-based account with a locked session, initiating a transaction should display the password dialog with reason "SignTransaction" before proceeding to transaction confirmation.

**Validates: Requirements 2.2, 10.2**

### Property 6: Correct Password Enables Transaction Signing

*For any* transaction from a seed-based account, providing the correct password should result in the transaction being signed and broadcast using the decrypted seed.

**Validates: Requirements 2.3, 10.3, 10.4**

### Property 7: Incorrect Password Prevents Transaction

*For any* transaction from a seed-based account, providing an incorrect password should display an error and prevent the transaction from being signed or broadcast.

**Validates: Requirements 2.4**

### Property 8: Session Timeout Triggers Auto-Lock

*For any* unlocked session, when the inactivity period exceeds the timeout duration (15 minutes), the session should automatically lock and clear all cached keys from memory.

**Validates: Requirements 3.1, 3.3, 7.2**

### Property 9: User Activity Extends Session

*For any* unlocked session, performing a wallet action should reset the inactivity timer, extending the session timeout period.

**Validates: Requirements 3.2**

### Property 10: Manual Lock Clears Keys Immediately

*For any* unlocked session, manually locking the session should immediately clear all cached keys from memory and set session state to locked.

**Validates: Requirements 3.5, 7.2**

### Property 11: Remember Option Caches Seed

*For any* password authentication with "Remember for 15 minutes" enabled, successful authentication should store the decrypted seed in a SecretString in memory for the cache duration.

**Validates: Requirements 4.2, 7.1**

### Property 12: Cached Keys Allow Passwordless Transactions

*For any* seed-based account with cached keys, initiating and signing a transaction should not require password re-entry during the cache period.

**Validates: Requirements 4.3**

### Property 13: Cache Expiration Requires Re-Authentication

*For any* cached seed, when the cache period (15 minutes) expires, the cached seed should be cleared and the next transaction should require password re-authentication.

**Validates: Requirements 4.4, 7.5**

### Property 14: Account Type Detection

*For any* set of accounts loaded at startup, the wallet should correctly identify each account as either seed-based or private-key based according to its KeyReference type.

**Validates: Requirements 5.1**

### Property 15: Seed Accounts Enforce Password

*For any* seed-based account, attempting to sign a transaction should enforce password authentication when the session is locked.

**Validates: Requirements 5.2**

### Property 16: Account Type Switch Updates Authentication

*For any* account selection change between seed-based and private-key accounts, the authentication requirements should adjust accordingly (password required for seed, not required for private-key).

**Validates: Requirements 5.5**

### Property 17: Password Attempt Tracking

*For any* sequence of incorrect password attempts, the wallet should track the attempt count and display the remaining attempts in error messages.

**Validates: Requirements 6.2**

### Property 18: Attempt Limit Triggers Lockout

*For any* account, when password attempts exceed the maximum limit, the wallet should temporarily lock the account and display a lockout message with retry timer.

**Validates: Requirements 6.3**

### Property 19: Successful Authentication Clears Errors

*For any* password dialog with previous errors, successfully entering the correct password should clear all error messages and proceed with authentication.

**Validates: Requirements 6.5**

### Property 20: Cached Keys Never Persist to Disk

*For any* transaction signed using cached keys, the decrypted seed should be accessed from memory only without writing to disk or logs.

**Validates: Requirements 7.4**

### Property 21: UI Reflects Session State

*For any* session state change (lock/unlock), all UI components should immediately update to reflect the new state.

**Validates: Requirements 8.4**

### Property 22: Startup Authentication Before Data Load

*For any* application startup with seed-based accounts, the password dialog should be displayed and authentication completed before loading account balances or sensitive data.

**Validates: Requirements 9.2, 9.3**

### Property 23: Transaction Flow Session Check

*For any* transaction initiation from a seed-based account, the wallet should check session state before proceeding with gas estimation or transaction confirmation.

**Validates: Requirements 10.1**

## Implementation Notes

### Security Considerations

1. **Key Storage:** Cached seeds must be stored in `SecretString` to prevent accidental logging or memory dumps
2. **Secure Erasure:** On session lock, use `zeroize` or similar to securely erase cached keys
3. **No Disk Persistence:** Cached keys must never be written to disk, only kept in memory
4. **Timeout Enforcement:** Session timeout must be enforced even if user activity continues (absolute timeout)

### Performance Considerations

1. **Startup Performance:** Account type detection should be fast to avoid delaying startup
2. **Password Validation:** Use async validation to avoid blocking UI
3. **Session Checks:** Session state checks should be lightweight (simple boolean check)

### User Experience Considerations

1. **Clear Messaging:** Password dialogs should clearly explain why password is needed
2. **Progress Indication:** Show loading states during password validation
3. **Error Recovery:** Allow easy retry after password errors
4. **Session Visibility:** Display session status and time remaining prominently

### Migration Strategy

1. **Backward Compatibility:** Existing accounts should work without changes
2. **Gradual Rollout:** Can be enabled per-account or globally
3. **User Education:** Provide clear documentation on new password workflow
4. **Testing:** Extensive testing with existing wallets before release
