# Password Workflow Developer Guide

## Architecture Overview

The Vaughan wallet password workflow is implemented as a secure, session-based authentication system that protects seed-based accounts while maintaining usability. This guide covers the technical implementation for developers.

## Core Components

### 1. Security State Management

**Location**: `src/gui/state/security_state.rs`

```rust
pub struct SecurityState {
    pub password_dialog: PasswordDialogState,
    pub session: SessionState,
    pub key_cache_handle: Option<Arc<RwLock<KeyCache>>>,
    pub password_validator: Option<PasswordValidator>,
}
```

**Key Features:**
- Unified security state container
- Thread-safe key cache handling
- Rate-limited password validation
- Session timeout management

### 2. Account Type Detection

**Location**: `src/gui/services/account_service.rs`

```rust
pub enum AccountType {
    SeedBased,    // Requires password authentication
    PrivateKey,   // Direct access, no password needed
}

pub fn get_account_type(account: &SecureAccount) -> AccountType {
    match &account.key_reference.key_type {
        KeyType::Seed { .. } => AccountType::SeedBased,
        KeyType::PrivateKey { .. } => AccountType::PrivateKey,
    }
}
```

**Usage:**
```rust
// Check if current account needs password
if get_account_type(current_account) == AccountType::SeedBased {
    // Enforce password requirements
}
```

### 3. Session Management

**Location**: `src/gui/state/security_state.rs`

```rust
pub struct SessionState {
    pub is_unlocked: bool,
    pub unlocked_at: Option<Instant>,
    pub last_activity: Instant,
    pub timeout_duration: Duration,  // Default: 15 minutes
    pub auto_lock_enabled: bool,
    pub cached_password: Option<SecretString>,
}
```

**Session Lifecycle:**
1. **Unlock**: `session.unlock()` - Sets unlocked state, caches keys
2. **Activity**: `session.update_activity()` - Resets timeout timer
3. **Timeout Check**: `session.is_expired()` - Checks if timeout reached
4. **Lock**: `session.lock()` - Clears cached keys, sets locked state

## Implementation Patterns

### 1. Startup Authentication Gate

**Location**: `src/gui/working_wallet.rs` - `Application::new()`

```rust
fn new(_flags: ()) -> (Self, Command<Message>) {
    let state = AppState::default();

    // Check for seed-based accounts before showing UI
    let check_accounts_cmd = Command::perform(
        check_for_seed_accounts(),
        Message::SeedAccountsChecked
    );

    (app, check_accounts_cmd)
}
```

**Message Handling:**
```rust
Message::SeedAccountsChecked(has_seed_accounts) => {
    if has_seed_accounts {
        // Show password dialog for startup unlock
        self.state.security_mut().password_dialog.show(
            PasswordReason::UnlockSession
        );
    } else {
        // Skip password for private-key only wallets
        self.state.security_mut().session.unlock();
        self.start_normal_initialization()
    }
}
```

### 2. Transaction Authentication Check

**Location**: `src/gui/handlers/transaction.rs`

```rust
fn handle_confirm_transaction(&mut self) -> Command<Message> {
    let account = self.get_current_account();

    match get_account_type(account) {
        AccountType::PrivateKey => {
            // No password needed
            self.proceed_with_transaction_signing()
        }
        AccountType::SeedBased => {
            if self.state.security().session.is_unlocked {
                // Session active, proceed
                self.proceed_with_transaction_signing()
            } else {
                // Session locked, request password
                self.show_password_dialog(
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

### 3. Password Validation Handler

**Location**: `src/gui/handlers/security.rs`

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

            // Cache seed if remember enabled
            if remember {
                self.state.security_mut().session.cached_password = Some(seed_phrase);
            }

            // Continue with original action
            match reason {
                Some(PasswordReason::UnlockSession) => {
                    self.start_normal_initialization()
                }
                Some(PasswordReason::SignTransaction { .. }) => {
                    self.proceed_to_transaction_confirmation()
                }
                _ => Command::none()
            }
        }
        Err(error) => {
            // Handle password errors
            self.state.security_mut().password_dialog.set_error(error);
            Command::none()
        }
    }
}
```

## Security Implementation

### 1. Secure Key Caching

**Location**: `src/security/key_cache.rs`

```rust
pub struct KeyCache {
    cached_seeds: HashMap<String, SecretString>,
    cache_expiry: HashMap<String, Instant>,
    cache_duration: Duration,
}

impl KeyCache {
    pub fn store_seed(&mut self, account_id: &str, seed: SecretString) {
        self.cached_seeds.insert(account_id.to_string(), seed);
        self.cache_expiry.insert(
            account_id.to_string(),
            Instant::now() + self.cache_duration
        );
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache_expiry.retain(|account_id, expiry| {
            if *expiry <= now {
                self.cached_seeds.remove(account_id);
                false
            } else {
                true
            }
        });
    }
}
```

### 2. Session Timeout Monitoring

**Location**: `src/gui/working_wallet.rs` - `subscription()` method

```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch(vec![
        // Other subscriptions...

        // Session timeout check every 10 seconds
        iced::time::every(Duration::from_secs(10))
            .map(|_| Message::SessionTimeoutCheck),
    ])
}
```

**Timeout Handler:**
```rust
Message::SessionTimeoutCheck => {
    if self.state.security().session.is_expired() {
        // Auto-lock session
        self.state.security_mut().session.lock();

        // Clear cached keys
        if let Some(cache) = &self.state.security().key_cache_handle {
            cache.write().await.clear_all();
        }

        Command::none()
    } else {
        Command::none()
    }
}
```

### 3. Password Validation with Rate Limiting

**Location**: `src/security/password_validator.rs`

```rust
pub struct PasswordValidator {
    attempt_count: u32,
    last_attempt: Option<Instant>,
    lockout_until: Option<Instant>,
    max_attempts: u32,
    lockout_duration: Duration,
}

impl PasswordValidator {
    pub async fn validate_password(
        &mut self,
        password: &SecretString,
        account_id: &str
    ) -> Result<SecretString, PasswordError> {
        // Check if currently locked out
        if let Some(lockout_until) = self.lockout_until {
            if Instant::now() < lockout_until {
                return Err(PasswordError::AccountLocked {
                    retry_after_seconds: (lockout_until - Instant::now()).as_secs()
                });
            }
        }

        // Attempt password validation
        match self.decrypt_seed(password, account_id).await {
            Ok(seed) => {
                // Success - reset attempts
                self.reset_attempts();
                Ok(seed)
            }
            Err(_) => {
                // Failed - increment attempts
                self.attempt_count += 1;

                if self.attempt_count >= self.max_attempts {
                    // Lock out temporarily
                    self.lockout_until = Some(Instant::now() + self.lockout_duration);
                    Err(PasswordError::TooManyAttempts {
                        retry_after_seconds: self.lockout_duration.as_secs()
                    })
                } else {
                    Err(PasswordError::IncorrectPassword {
                        attempts_remaining: self.max_attempts - self.attempt_count
                    })
                }
            }
        }
    }
}
```

## UI Integration

### 1. Password Dialog Component

**Location**: `src/gui/components/dialogs/password_dialog.rs`

```rust
pub fn password_dialog_view(state: &AppState) -> Element<Message> {
    let password_state = &state.security().password_dialog;

    if !password_state.visible {
        return Space::with_height(Length::Fixed(0.0)).into();
    }

    let reason_text = match &password_state.reason {
        Some(PasswordReason::SignTransaction { tx_details }) => {
            format!("Sign transaction: {}", tx_details)
        }
        Some(PasswordReason::UnlockSession) => {
            "Enter password to unlock wallet".to_string()
        }
        // ... other reasons
    };

    Container::new(
        Column::new()
            .push(Text::new("🔐 Password Required"))
            .push(Text::new(reason_text))
            .push(
                TextInput::new("Enter password", &password_state.input)
                    .on_input(Message::PasswordInput)
                    .password()
                    .on_submit(Message::PasswordSubmitted)
            )
            .push(
                Checkbox::new(
                    "Remember for 15 minutes",
                    password_state.remember_session
                )
                .on_toggle(Message::RememberSessionToggled)
            )
            .push(
                Row::new()
                    .push(
                        Button::new("Cancel")
                            .on_press(Message::PasswordCancelled)
                    )
                    .push(
                        Button::new("Unlock")
                            .on_press(Message::PasswordSubmitted)
                    )
            )
    )
}
```

### 2. Session Indicator Component

**Location**: `src/gui/components/session_indicator.rs`

```rust
pub fn session_indicator(state: &AppState) -> Element<Message> {
    let session = &state.security().session;

    if session.is_unlocked {
        let time_remaining = session.time_until_timeout();
        Row::new()
            .push(Text::new("🔓").color(Color::from_rgb(0.0, 0.8, 0.0)))
            .push(Text::new(format!("{}m", time_remaining.as_secs() / 60)))
            .push(
                Button::new("Lock")
                    .on_press(Message::ManualLock)
                    .style(Button::Secondary)
            )
    } else {
        Row::new()
            .push(Text::new("🔒").color(Color::from_rgb(0.8, 0.0, 0.0)))
            .push(Text::new("Locked"))
            .push(
                Button::new("Unlock")
                    .on_press(Message::ShowPasswordDialog {
                        reason: PasswordReason::UnlockSession
                    })
                    .style(Button::Primary)
            )
    }
    .into()
}
```

### 3. Locked State View

**Location**: `src/gui/views/locked_view.rs`

```rust
pub fn locked_view(state: &AppState) -> Element<Message> {
    Container::new(
        Column::new()
            .push(Text::new("🔒").size(48).color(Color::from_rgb(0.8, 0.0, 0.0)))
            .push(Text::new("Wallet Locked").size(24))
            .push(Text::new("Your wallet is locked for security."))
            .push(Text::new("Enter your password to unlock and access your accounts."))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Button::new("Unlock Wallet")
                    .on_press(Message::ShowPasswordDialog {
                        reason: PasswordReason::UnlockSession
                    })
                    .style(Button::Primary)
            )
            .spacing(10)
            .align_items(Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
```

## Message Flow

### Startup Flow
```
Application::new()
    ↓
Command::perform(check_for_seed_accounts)
    ↓
Message::SeedAccountsChecked(true)
    ↓
Show PasswordDialog(UnlockSession)
    ↓
Message::PasswordValidated(Ok(seed))
    ↓
session.unlock() + start_normal_initialization()
```

### Transaction Flow
```
Message::ConfirmTransaction
    ↓
check_account_type() + check_session_state()
    ↓
Show PasswordDialog(SignTransaction) [if locked]
    ↓
Message::PasswordValidated(Ok(seed))
    ↓
Message::ProceedToTransactionConfirmation
```

## Testing Strategy

### Unit Tests
- Account type detection accuracy
- Session state transitions
- Password validation logic
- Key cache expiration
- Rate limiting functionality

### Integration Tests
- Startup authentication flow
- Transaction signing flow
- Session timeout behavior
- Error recovery scenarios
- UI state consistency

### Property-Based Tests
All properties from `requirements.md` should be verified with proptest:
- Password unlocks session
- Session timeout triggers lock
- Account type determines auth requirements
- Key caching follows security rules

## Security Considerations

### Key Handling
1. **Memory Only**: Cached keys never written to disk
2. **Secure Erasure**: Use `zeroize` crate for key clearing
3. **Limited Lifetime**: Automatic expiration after timeout
4. **Thread Safety**: Arc<RwLock<>> for safe concurrent access

### Session Management
1. **Absolute Timeout**: Maximum session duration regardless of activity
2. **Inactivity Timeout**: Configurable idle time before lock
3. **Manual Override**: User can lock immediately
4. **Visual Feedback**: Clear session state indicators

### Error Handling
1. **Rate Limiting**: Prevent brute force attacks
2. **Temporary Lockouts**: Progressive delay after failures
3. **Clear Messages**: User-friendly error descriptions
4. **Attempt Tracking**: Persistent failure counting

### Audit Trail
1. **No Sensitive Logging**: Passwords/seeds never logged
2. **Security Events**: Login/logout tracking
3. **Error Logging**: Failed attempts (without passwords)
4. **Performance Metrics**: Session duration statistics

This architecture provides a secure, usable password workflow that protects seed-based accounts while maintaining excellent user experience for daily wallet operations.