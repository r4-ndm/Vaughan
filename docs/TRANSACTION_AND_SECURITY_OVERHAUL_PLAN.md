# Transaction and Security Overhaul Plan

## Executive Summary
This document outlines a comprehensive plan to properly implement transaction signing with password protection for seed-based accounts, along with improvements to the send/receive functionality.

## Current State Analysis

### Problems Identified

#### 1. Seed-Based Account Signing
- **Issue**: Seeds are encrypted with master password, but password is not available during signing
- **Impact**: Cannot sign transactions from seed-based accounts (most accounts)
- **Root Cause**: Architecture assumes passwordless signing, but security requires password

#### 2. Password Management
- **Issue**: No password prompt system in place
- **Impact**: Cannot decrypt seeds when needed
- **Security**: Passwords correctly not stored, but no way to request them

#### 3. Account Type Confusion
- **Issue**: Two account types (seed-based vs private-key) handled inconsistently
- **Impact**: Different code paths, different behaviors, user confusion

#### 4. Session Management
- **Issue**: No concept of "unlocked" session with timeout
- **Impact**: Either always locked (unusable) or always unlocked (insecure)

## Proposed Architecture

### Phase 1: Password Dialog System (Week 1)

#### 1.1 Password Dialog Component
Create a secure password input dialog that:
- Appears when password is needed (signing, exporting, etc.)
- Shows what operation requires the password
- Has timeout (auto-dismiss after 2 minutes)
- Masks password input
- Validates password before proceeding
- Shows error if password is incorrect

**Files to Create:**
- `src/gui/components/dialogs/password_dialog.rs`
- `src/gui/state/security_state.rs` *(unified security state module)*

**New Messages:**
```rust
// In src/gui/wallet_messages.rs, add under // Security & Session comment block:

enum Message {
    // Security & Session Management
    ShowPasswordDialog { reason: PasswordReason },
    HidePasswordDialog,
    PasswordInputChanged(SecretString), // Use SecretString immediately!
    SubmitPassword,
    PasswordValidated(Result<SecretString, PasswordError>), // Specific error type
    
    SessionLocked,
    SessionUnlocked,
    ExtendSession,
    ManualLock,
    
    // Existing messages...
}

enum PasswordReason {
    SignTransaction { tx_details: String },
    ExportPrivateKey { account_name: String },
    ExportSeedPhrase { account_name: String },
    DeleteAccount { account_name: String },
}

pub enum PasswordError {
    IncorrectPassword,
    DecryptionFailed,
    EmptyPassword,
    TooManyAttempts,
    SessionExpired,
}
```

#### 1.2 Password Validation Service
Create service to validate passwords against stored accounts:
- Try to decrypt seed with provided password
- Cache validation result (don't re-decrypt)
- Clear cache on timeout

**Files to Create:**
- `src/security/password_validator.rs`

### Phase 2: Session Management (Week 1-2)

#### 2.1 Unified Security State Module
Create `src/gui/state/security_state.rs` to house all security-related state:

**State Structure:**
```rust
// src/gui/state/security_state.rs

pub struct SecurityState {
    // Password dialog state
    pub password_dialog: PasswordDialogState,
    
    // Session management
    pub session: SessionState,
    
    // Key cache reference (actual cache in secure memory)
    pub key_cache_handle: Option<Arc<RwLock<KeyCache>>>,
}

pub struct PasswordDialogState {
    pub visible: bool,
    pub reason: Option<PasswordReason>,
    pub input: SecretString, // Use SecretString from the start!
    pub error: Option<PasswordError>,
    pub attempts: u32,
}

pub struct SessionState {
    pub is_unlocked: bool,
    pub unlocked_at: Option<Instant>,
    pub last_activity: Instant,
    pub timeout_duration: Duration,
    pub auto_lock_enabled: bool,
    pub lock_on_minimize: bool,
}
```

**Files to Modify:**
- `src/gui/state/mod.rs` - Add `security: SecurityState`
- `src/gui/working_wallet.rs` - Add session timeout subscription

#### 2.2 Derived Key Caching with Secure Memory
When password is validated, derive and cache private keys:
- Decrypt seed with password
- Derive private key for current derivation path
- Store in secure memory (with zeroization)
- Clear on session lock
- Re-derive on session unlock

**Files to Create:**
- `src/security/key_cache.rs`

**Structure:**
```rust
pub struct KeyCache {
    cached_keys: HashMap<Address, SecureMemory>, // Zeroized on drop
    cache_timeout: Duration,
    last_access: HashMap<Address, Instant>,
    memory_lock_available: bool, // Track if mlock succeeded
}

impl KeyCache {
    pub fn new(timeout: Duration) -> Self {
        // Check if memory locking is available
        let memory_lock_available = SecureMemory::test_memory_locking();
        
        if !memory_lock_available {
            tracing::warn!("⚠️ Memory locking unavailable - using shorter timeout");
            // Use shorter timeout (5 min instead of 15) if can't lock memory
        }
        
        Self {
            cached_keys: HashMap::new(),
            cache_timeout: if memory_lock_available { timeout } else { Duration::from_secs(300) },
            last_access: HashMap::new(),
            memory_lock_available,
        }
    }
}
```

**Note:** Handle memory locking failures gracefully (containers, restricted environments)

### Phase 3: Transaction Signing Flow (Week 2)

#### 3.1 Enhanced Transaction Confirmation
Update confirmation dialog to include password prompt:
- Show transaction details
- Show gas estimation
- **NEW**: Request password if session locked
- **NEW**: Show "Remember for 15 minutes" checkbox (only if session is locked)
- Validate password before signing
- Sign transaction with decrypted key
- **NEW**: Show toast on auto-lock ("Wallet locked due to inactivity")

**Flow:**
```
User clicks "Send"
    ↓
Gas Estimation
    ↓
Show Confirmation Dialog
    ↓
Check if session unlocked
    ↓
If locked:
    ├─ Show password input in dialog
    ├─ Show "Remember for 15 minutes" checkbox
    ├─ User enters password
    ├─ Validate password (decrypt seed)
    │   ├─ On error: Show specific error (shake input, show message)
    │   └─ On success: Continue
    ├─ Derive private key
    └─ Cache key in session (if checkbox checked)
    ↓
If unlocked:
    ├─ Checkbox hidden or checked+disabled
    └─ Use cached key
    ↓
Sign transaction
    ↓
Broadcast
    ↓
Success
```

**UX Improvements:**
- Shake password input on `PasswordError::IncorrectPassword`
- Show toast for `PasswordError::DecryptionFailed`
- Show status bar message when session auto-locks
- Clear password input immediately after validation (convert to SecretString)

#### 3.2 Signing Service Refactor
Refactor signing to handle both account types properly:

**Files to Modify:**
- `src/security/keystore.rs` - Update `sign_transaction`

**New Structure:**
```rust
impl SecureKeystoreImpl {
    pub async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        address: &Address,
        password: Option<&SecretString>, // NEW: Optional password
        key_cache: &mut KeyCache,        // NEW: Key cache
    ) -> Result<Vec<u8>> {
        let account = self.accounts.get(address)?;
        
        // Try to get key from cache first
        if let Some(cached_key) = key_cache.get(address) {
            return self.sign_with_key(tx, cached_key);
        }
        
        // Need to derive key
        let key = match account.key_reference.service.as_str() {
            "vaughan-wallet-encrypted-seeds" => {
                // Seed-based: need password
                let password = password.ok_or("Password required")?;
                let key = self.derive_key_from_seed(account, password)?;
                
                // Cache the derived key
                key_cache.insert(address, key.clone());
                key
            }
            "vaughan-wallet" => {
                // Private key: retrieve directly
                self.retrieve_private_key(&account.key_reference)?
            }
            _ => return Err("Unknown account type"),
        };
        
        self.sign_with_key(tx, &key)
    }
    
    fn derive_key_from_seed(
        &self,
        account: &SecureAccount,
        password: &SecretString,
    ) -> Result<SecureMemory> {
        // 1. Retrieve encrypted seed from keychain
        // 2. Decrypt with password
        // 3. Derive private key using derivation path
        // 4. Return as SecureMemory (auto-zeroized)
    }
}
```

### Phase 4: Receive Functionality (Week 2-3)

#### 4.1 Receive Dialog Enhancement
Improve the receive dialog:
- Show QR code for address
- Show full address with copy button
- Show derivation path (for seed-based accounts)
- Show account balance
- Show recent incoming transactions
- **NEW**: Generate new receive address (for HD wallets)

**Files to Create:**
- `src/gui/components/dialogs/receive_dialog.rs`
- `src/gui/services/qr_service.rs` (QR code generation)

#### 4.2 Address Generation
For HD wallets, support generating new addresses:
- Derive next unused address in sequence
- Track used addresses
- Show address index
- Require password to derive new address

### Phase 5: Security Enhancements (Week 3)

#### 5.1 Secure Memory Management
Implement proper secure memory for sensitive data:
- Use `SecureMemory` type for all keys
- Auto-zeroize on drop
- Lock memory pages (prevent swapping)
- Clear clipboard after timeout

**Already exists in:**
- `src/security/memory.rs`

**Need to use it in:**
- Key cache
- Password handling
- Transaction signing

#### 5.2 Audit Logging
Add security audit logging:
- Log all password attempts (success/failure)
- Log all transaction signing attempts
- Log session lock/unlock events
- Log key derivations
- Store in encrypted log file

**Files to Create:**
- `src/security/audit_log.rs`

#### 5.3 Rate Limiting
Implement rate limiting for security operations:
- Max 3 password attempts per minute
- Exponential backoff on failures
- Lock account after 5 failed attempts
- Require waiting period to unlock

### Phase 6: User Experience (Week 3-4)

#### 6.1 Settings Panel
Add security settings:
- Session timeout duration (5/15/30/60 minutes, or never)
- Auto-lock on minimize
- Require password for transactions over X amount
- Clipboard clear timeout
- Show/hide balance

#### 6.2 Account Management
Improve account management:
- Show account type (seed-based vs private-key)
- Show derivation path
- Show when account was last used
- Export options (private key, seed phrase, keystore file)
- Change password for seed-based accounts

#### 6.3 Transaction History
Enhance transaction history:
- Show pending transactions
- Show failed transactions
- Show gas used vs estimated
- Export transaction history
- Filter by date/amount/status

## Implementation Priority

### Critical (Must Have - Week 1)
1. ✅ Password dialog component
2. ✅ Session management basics
3. ✅ Password validation service
4. ✅ Update sign_transaction to accept password

### High Priority (Week 2)
5. ✅ Key caching system
6. ✅ Enhanced transaction confirmation with password
7. ✅ Proper seed decryption and key derivation
8. ✅ Session timeout and auto-lock

### Medium Priority (Week 3)
9. ⚠️ Receive dialog improvements
10. ⚠️ QR code generation
11. ⚠️ Audit logging
12. ⚠️ Rate limiting

### Nice to Have (Week 4)
13. ⚪ Settings panel
14. ⚪ Advanced account management
15. ⚪ Transaction history enhancements
16. ⚪ HD wallet address generation

## Security Considerations

### Password Storage
- ✅ **NEVER** store passwords in plain text
- ✅ **NEVER** store passwords encrypted (key management problem)
- ✅ Only cache derived keys (not passwords) in memory during unlocked session
- ✅ Clear keys from memory on lock
- ✅ Use `SecretString` type for password input (zeroized on drop)
- ✅ Convert raw `String` to `SecretString` immediately in text input handler
- ✅ Drop/clear intermediate strings as soon as possible

### Key Management
- ✅ Derive keys on-demand from seed
- ✅ Cache derived keys in secure memory
- ✅ Clear key cache on session lock
- ✅ Use memory locking to prevent swapping
- ✅ Zeroize keys on drop

### Session Security
- ✅ Default timeout: 15 minutes
- ✅ Extend timeout on activity
- ✅ Auto-lock on minimize (optional)
- ✅ Manual lock button always available
- ✅ Clear all sensitive data on lock

### Transaction Security
- ✅ Always show transaction details before signing
- ✅ Require password for large transactions (optional)
- ✅ Show gas estimation
- ✅ Allow transaction cancellation
- ✅ Log all transaction attempts

## Testing Strategy

### Unit Tests
- Password validation with specific error types
- Key derivation from seed
- Session timeout logic
- Key cache operations
- Secure memory zeroization
- Memory locking fallback behavior

### Integration Tests
- Full transaction flow with password
- Session lock/unlock
- Password retry logic
- Key cache invalidation
- **Property-based testing** for session timeout (simulate time passage)

### Security Tests
- Memory inspection (keys zeroized)
- Password brute force protection
- Session hijacking prevention
- Clipboard security
- Memory locking failure handling

### User Acceptance Tests
- Create account → Send transaction (with password)
- Session timeout → Re-enter password → See toast notification
- Lock wallet → Unlock → Send
- Export private key (with password)
- **Full flow integration test**: Start Tx → Prompt Password → Unlock → Sign → Auto-lock → Start Tx → Prompt Password

## Migration Plan

### Existing Users
For users with existing seed-based accounts:
1. Show migration notice on first launch
2. Explain password requirement
3. Offer to set password for existing accounts
4. Provide export/import option

### Backward Compatibility
- Support both old and new account formats
- Detect account type automatically
- Graceful fallback for missing passwords

## Documentation

### User Documentation
- How to set up password protection
- How session locking works
- How to change password
- How to export/backup accounts
- Security best practices

### Developer Documentation
- Architecture overview
- Password flow diagrams
- Key derivation process
- Session management API
- Security audit log format

## Success Metrics

### Functionality
- ✅ Can send transactions from seed-based accounts
- ✅ Password prompt appears when needed
- ✅ Session management works correctly
- ✅ Keys properly cached and cleared

### Security
- ✅ No passwords stored on disk
- ✅ Keys zeroized after use
- ✅ Session timeout enforced
- ✅ Audit log captures security events

### User Experience
- ✅ Password prompt is clear and helpful
- ✅ Session timeout is reasonable
- ✅ Manual lock is easy to find
- ✅ Transaction flow is smooth

## Timeline

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | Password System | Dialog, validation, session basics |
| 2 | Transaction Signing | Key caching, signing flow, confirmation |
| 3 | Security & Receive | Audit log, rate limiting, receive dialog |
| 4 | Polish & Testing | Settings, UX improvements, testing |

## Next Steps

1. **Review this plan** with team/stakeholders
2. **Create GitHub issues** for each component
3. **Set up development branch** for overhaul
4. **Begin Phase 1** implementation
5. **Regular progress reviews** (daily standups)

## Questions to Resolve

1. **Default session timeout?** (Recommend: 15 minutes)
2. **Password complexity requirements?** (Recommend: 8+ chars, no other requirements)
3. **Max password retry attempts?** (Recommend: 5 attempts, then 5-minute lockout)
4. **Clipboard clear timeout?** (Recommend: 30 seconds)
5. **Require password for all transactions or only large ones?** (Recommend: All transactions)
6. **Support biometric unlock?** (Future consideration)

## Conclusion

This overhaul will transform Vaughan from a partially-functional wallet into a secure, production-ready application. The phased approach allows for incremental progress while maintaining functionality. Security is prioritized throughout, with proper password management, session handling, and key protection.

**Estimated Total Effort:** 3-4 weeks for full implementation
**Risk Level:** Medium (significant refactoring, but well-defined scope)
**Impact:** High (enables core functionality, improves security)

---

**Document Version:** 1.0  
**Date:** November 22, 2025  
**Author:** Kiro AI Assistant  
**Status:** Proposed - Awaiting Approval


---

## Gemini Review Feedback Integration

### Changes Made Based on Review

#### 1. ✅ Unified Security State Module
**Suggestion:** Create `security_state.rs` instead of just `password_state.rs`  
**Rationale:** Keeps all security-related state (password, session, key cache) in one cohesive module  
**Implementation:** Created `SecurityState` struct containing `PasswordDialogState`, `SessionState`, and key cache handle

#### 2. ✅ Specific Error Types
**Suggestion:** Define `PasswordError` enum instead of generic `String`  
**Rationale:** Allows UI to react intelligently (shake input vs show toast)  
**Implementation:** Added `PasswordError` enum with variants: `IncorrectPassword`, `DecryptionFailed`, `EmptyPassword`, `TooManyAttempts`, `SessionExpired`

#### 3. ✅ Message Organization
**Suggestion:** Group security messages under `// Security & Session` comment block  
**Rationale:** Logical organization in `wallet_messages.rs`  
**Implementation:** Added comment block grouping in message enum structure

#### 4. ✅ Secure Memory Handling
**Suggestion:** Handle memory locking failures gracefully  
**Rationale:** Containers/restricted environments may not support `mlock`  
**Implementation:** Added `memory_lock_available` flag to `KeyCache`, shorter timeout if locking fails, warning logs

#### 5. ✅ SecretString Conversion
**Suggestion:** Convert to `SecretString` immediately in text input handler  
**Rationale:** Minimize time sensitive data exists as plain `String`  
**Implementation:** Changed `PasswordInputChanged(String)` to `PasswordInputChanged(SecretString)`

#### 6. ✅ UX Enhancements
**Suggestions:**
- Hide "Remember" checkbox if session already unlocked
- Show toast when session auto-locks
- Shake input on incorrect password

**Implementation:** Added to Phase 3 flow with specific UI behaviors for each error type

#### 7. ✅ Enhanced Testing Strategy
**Suggestion:** Emphasize property-based testing and full flow integration test  
**Implementation:** Added specific test cases including time simulation and complete transaction flow

### Review Conclusion
**Status:** ✅ Approved with refinements incorporated  
**Reviewer:** Gemini 3  
**Date:** November 22, 2025  
**Recommendation:** Ready for implementation

---

**Document Version:** 1.1 (Updated with Gemini feedback)  
**Last Updated:** November 22, 2025
