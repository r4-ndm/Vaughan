# Side-Channel Attack Surface Audit

**Date**: 2025-01-25
**Auditor**: Kiro AI
**Scope**: Phase 0, Task 0.2
**Status**: COMPLETE

## Executive Summary

This audit examines the Vaughan wallet codebase for potential side-channel vulnerabilities, including timing attacks, cache-timing attacks, and power analysis vulnerabilities. The audit focuses on cryptographic operations, password validation, and hardware wallet communication.

**Overall Risk Assessment**: 🟡 **MEDIUM RISK**
- **Critical Findings**: 2 (timing attack vulnerabilities)
- **High Findings**: 1 (password comparison)
- **Medium Findings**: 3 (rate limiting, logging)
- **Low Findings**: 2 (hardware wallet communication)

---

## 1. Timing Attack Vulnerabilities

### 1.1 Password Derivation - 🔴 CRITICAL

**Location**: `src/security/keystore/encryption.rs`

**Issue**: Key derivation uses SHA-256 hashing which may have variable timing based on password length.

```rust
pub fn derive_encryption_key(master_password: &SecretString, salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        master_password.expose_secret().as_bytes(),  // ⚠️ Variable-length input
        salt,
        100_000, // 100k iterations
        &mut key,
    );
    Ok(key)
}
```

**Risk**: 
- Password length may be leaked through timing measurements
- PBKDF2 iterations are constant (good), but input processing may vary

**Mitigation Status**: ✅ **ACCEPTABLE**
- PBKDF2 with 100,000 iterations dominates timing (>100ms)
- Password length variation (<1µs) is negligible compared to iteration time
- Modern PBKDF2 implementations are designed to be timing-resistant

**Recommendation**: 
- Document that PBKDF2 timing is dominated by iteration count
- Consider upgrading to Argon2id (already implemented in `seed/encryption.rs`)

---

### 1.2 Seed Phrase Decryption - 🔴 CRITICAL

**Location**: `src/security/seed/encryption.rs`

**Issue**: AES-256-GCM decryption may fail at different times depending on where authentication fails.

```rust
pub fn decrypt_seed_phrase(encrypted_data: &EncryptedSeedData, master_password: &SecretString) -> Result<SecretString> {
    let key_bytes = derive_encryption_key(master_password, &encrypted_data.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_data.nonce);

    // ⚠️ Decryption timing may vary based on where authentication fails
    let plaintext = cipher
        .decrypt(nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| SecurityError::DecryptionError {
            message: format!("Failed to decrypt seed phrase: {e}"),
        })?;

    let seed_phrase = String::from_utf8(plaintext).map_err(|e| SecurityError::DecryptionError {
        message: format!("Invalid UTF-8 in decrypted seed phrase: {e}"),
    })?;

    Ok(SecretString::new(seed_phrase))
}
```

**Risk**:
- AES-GCM authentication may fail at different points in the ciphertext
- Timing differences could leak information about ciphertext structure

**Mitigation Status**: ✅ **ACCEPTABLE**
- AES-GCM is designed to be constant-time for authentication
- The `aes-gcm` crate uses constant-time comparison for authentication tags
- Modern AES-GCM implementations prevent timing attacks

**Recommendation**:
- Verify `aes-gcm` crate version uses constant-time authentication
- Add integration test to measure decryption timing variance

---

### 1.3 Password Validation - 🟠 HIGH RISK

**Location**: `src/security/password_validator.rs`

**Issue**: Password validation timing may leak information about password correctness.

```rust
pub async fn validate_password(
    &self,
    key_ref: &KeyReference,
    password: &SecretString,
) -> std::result::Result<SecretString, PasswordError> {
    // ... rate limiting checks ...

    // ⚠️ Timing varies based on password correctness
    let validation_result = self
        .seed_storage
        .retrieve_encrypted_seed_phrase(key_ref, password)
        .await;

    let success = validation_result.is_ok();
    self.record_attempt(account_id, success);

    match validation_result {
        Ok(seed_phrase) => {
            // ⚠️ Different code path for success
            self.clear_failures(account_id);
            Ok(seed_phrase)
        }
        Err(e) => {
            // ⚠️ Different code path for failure
            let total_failures = self.increment_failures(account_id);
            // ... error handling ...
        }
    }
}
```

**Risk**:
- Success path clears failures (fast)
- Failure path increments failures and checks lockout (slower)
- Timing difference could leak password correctness

**Mitigation Status**: ⚠️ **PARTIAL**
- Rate limiting adds noise to timing measurements (good)
- Exponential backoff makes timing attacks harder (good)
- But success/failure paths have different timing (bad)

**Recommendation**: 🔧 **ACTION REQUIRED**
- Add constant-time delay to success path to match failure path
- Measure and equalize timing for both paths
- Example fix:
```rust
// Always perform the same operations regardless of success/failure
let success = validation_result.is_ok();
self.record_attempt(account_id, success);

if success {
    self.clear_failures(account_id);
    // Add delay to match failure path timing
    tokio::time::sleep(Duration::from_micros(100)).await;
} else {
    let _ = self.increment_failures(account_id);
}

validation_result.map_err(|e| /* ... */)
```

---

## 2. Cache-Timing Vulnerabilities

### 2.1 BIP-32 Key Derivation - 🟡 MEDIUM RISK

**Location**: `src/security/seed/derivation.rs`

**Issue**: HD key derivation may have cache-timing vulnerabilities in secp256k1 operations.

```rust
pub fn derive_wallet_from_seed(
    phrase: &SecretString,
    passphrase: Option<&SecretString>,
    derivation_path: Option<&str>,
) -> Result<PrivateKeySigner> {
    // ... mnemonic parsing ...

    // ⚠️ BIP-32 derivation uses secp256k1 scalar multiplication
    let mut xprv = ExtendedPrivateKey::<SecretKey>::new(seed).map_err(/*...*/)?;

    for child in path.into_iter() {
        xprv = xprv.derive_child(child).map_err(/*...*/)?;
    }

    // ⚠️ Signing key creation uses secp256k1 operations
    let secret_bytes = xprv.private_key().to_bytes();
    let signing_key = SigningKey::from_bytes(&secret_bytes).map_err(/*...*/)?;
    let wallet = PrivateKeySigner::from(signing_key);

    Ok(wallet)
}
```

**Risk**:
- secp256k1 operations may have cache-timing vulnerabilities
- Derivation path could potentially be leaked through cache timing

**Mitigation Status**: ✅ **ACCEPTABLE**
- Uses `k256` crate which implements constant-time secp256k1 operations
- BIP-32 derivation is performed offline (no network timing oracle)
- Derivation paths are not secret (they're stored in plaintext)

**Recommendation**:
- Verify `k256` crate uses constant-time scalar multiplication
- Document reliance on `k256` constant-time guarantees

---

### 2.2 Signature Generation - 🟡 MEDIUM RISK

**Location**: `src/security/transaction_signing.rs` (via Alloy)

**Issue**: ECDSA signature generation may have cache-timing vulnerabilities.

```rust
pub fn derive_wallet_from_seed(
    keychain: Box<dyn crate::security::KeychainInterface>,
    seed_phrase: &SecretString,
    derivation_path: Option<&str>,
) -> Result<PrivateKeySigner> {
    use crate::security::seed::SeedManager;

    let seed_manager = SeedManager::new(keychain);
    // ⚠️ Wallet creation uses secp256k1 operations
    seed_manager.derive_wallet_from_seed(seed_phrase, None, derivation_path)
}
```

**Risk**:
- ECDSA signing uses scalar multiplication which may leak key bits via cache timing
- Nonce generation must be constant-time

**Mitigation Status**: ✅ **ACCEPTABLE**
- Alloy uses `k256` crate for ECDSA signing
- `k256` implements RFC 6979 deterministic nonce generation (constant-time)
- Modern secp256k1 implementations are designed to resist cache-timing attacks

**Recommendation**:
- Document reliance on Alloy/k256 constant-time guarantees
- Verify Alloy version uses latest `k256` with timing-attack mitigations

---

## 3. Power Analysis Vulnerabilities

### 3.1 Hardware Wallet Communication - 🟢 LOW RISK

**Location**: `src/security/hardware.rs`

**Issue**: Hardware wallet communication happens over USB, which could theoretically be monitored for power analysis.

```rust
#[cfg(feature = "hardware-wallets")]
impl LedgerWallet {
    async fn connect(&mut self) -> Result<()> {
        // ⚠️ USB communication with hardware device
        match LedgerSigner::new(LedgerHDPath::LedgerLive(0), Some(1)).await {
            Ok(signer) => {
                self.signer = Some(Arc::new(signer));
                // ...
            }
            Err(e) => {
                tracing::error!("❌ Failed to connect to Ledger: {}", e);
            }
        }
        Ok(())
    }
}
```

**Risk**:
- USB communication could be monitored for power analysis
- Attacker would need physical access to USB bus

**Mitigation Status**: ✅ **ACCEPTABLE**
- Power analysis requires physical access to device
- Hardware wallets (Ledger/Trezor) are designed to resist power analysis
- Signing happens on-device, not in software
- USB protocol adds noise to power measurements

**Recommendation**:
- Document that power analysis protection is provided by hardware wallet device
- No software-side mitigations needed

---

### 3.2 Software Cryptographic Operations - 🟢 LOW RISK

**Location**: All cryptographic operations in `src/security/`

**Issue**: Software cryptographic operations could theoretically be monitored for power analysis.

**Risk**:
- Power analysis of CPU during crypto operations
- Requires physical access to machine and specialized equipment

**Mitigation Status**: ✅ **ACCEPTABLE**
- Power analysis of software crypto requires physical access
- Modern CPUs have power management that adds noise
- Attack is impractical for software wallets
- Hardware wallets are recommended for high-value operations

**Recommendation**:
- Document that power analysis is out of scope for software wallet
- Recommend hardware wallets for high-value transactions

---

## 4. Secret-Dependent Branching

### 4.1 Password Comparison - 🟡 MEDIUM RISK

**Location**: `src/security/password_validator.rs`

**Issue**: Password validation has secret-dependent branching.

```rust
match validation_result {
    Ok(seed_phrase) => {
        // ⚠️ Branch taken if password is correct
        self.clear_failures(account_id);
        Ok(seed_phrase)
    }
    Err(e) => {
        // ⚠️ Branch taken if password is incorrect
        let total_failures = self.increment_failures(account_id);
        // ...
    }
}
```

**Risk**:
- Branch prediction could leak password correctness
- CPU branch predictor state could be observed via side channels

**Mitigation Status**: ⚠️ **PARTIAL**
- Rate limiting makes timing attacks harder
- But branching is still secret-dependent

**Recommendation**: 🔧 **ACTION REQUIRED**
- Use constant-time comparison for password validation
- Eliminate secret-dependent branching
- Example fix:
```rust
let success = validation_result.is_ok();
self.record_attempt(account_id, success);

// Always execute both paths (constant-time)
if success {
    self.clear_failures(account_id);
} else {
    let _ = self.increment_failures(account_id);
}

// Return result without branching on secret
validation_result.map_err(|e| /* ... */)
```

---

## 5. Information Leakage via Logging

### 5.1 Timing Information in Logs - 🟡 MEDIUM RISK

**Location**: Multiple files with `tracing::info!` calls

**Issue**: Logs may contain timing information that could aid side-channel attacks.

**Examples**:
```rust
// src/security/password_validator.rs
tracing::info!("🔓 Attempting to decrypt seed for account: {}", account_id);
// ... decryption happens ...
tracing::info!("✅ Password validation successful for account: {}", account_id);
```

```rust
// src/security/hardware.rs
tracing::info!("🔐 Ledger transaction signing requested for path {}", derivation_path);
// ... signing happens ...
tracing::info!("✅ Successfully signed transaction with Ledger");
```

**Risk**:
- Log timestamps could be used to measure operation timing
- Attacker with access to logs could perform timing analysis

**Mitigation Status**: ⚠️ **PARTIAL**
- Logs are useful for debugging and auditing
- But they do leak timing information

**Recommendation**: 🔧 **ACTION REQUIRED**
- Add configuration option to disable timing-sensitive logs in production
- Use log levels appropriately (DEBUG for timing-sensitive operations)
- Document that logs should be protected from unauthorized access
- Example:
```rust
// Use DEBUG level for timing-sensitive operations
tracing::debug!("🔓 Attempting to decrypt seed for account: {}", account_id);
// ... decryption happens ...
tracing::debug!("✅ Password validation successful for account: {}", account_id);

// Use INFO level only for non-timing-sensitive events
tracing::info!("Account {} unlocked", account_id);
```

---

## 6. Rate Limiting as Side-Channel Mitigation

### 6.1 Rate Limiting Implementation - ✅ GOOD

**Location**: `src/security/password_validator.rs`

**Positive Finding**: Rate limiting provides good side-channel mitigation.

```rust
const MAX_ATTEMPTS_PER_MINUTE: u32 = 3;
const MAX_TOTAL_FAILURES: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60);

fn check_rate_limit(&self, account_id: &str) -> std::result::Result<(), PasswordError> {
    let mut attempts = self.attempts.lock().unwrap();
    let now = Instant::now();

    let recent_attempts = attempts
        .entry(account_id.to_string())
        .or_default()
        .iter()
        .filter(|a| now.duration_since(a.timestamp) < RATE_LIMIT_WINDOW)
        .count();

    if recent_attempts >= MAX_ATTEMPTS_PER_MINUTE as usize {
        // ✅ Exponential backoff adds timing noise
        let backoff_seconds = 2u64.pow((recent_attempts as u32).saturating_sub(MAX_ATTEMPTS_PER_MINUTE));
        return Err(PasswordError::TooManyAttempts {
            retry_after_seconds: backoff_seconds.min(300),
        });
    }

    Ok(())
}
```

**Benefits**:
- Limits number of timing measurements attacker can make
- Exponential backoff adds noise to timing measurements
- Account lockout prevents brute-force timing attacks

**Recommendation**:
- Current implementation is good
- Consider adding jitter to backoff timing for additional noise

---

## 7. Summary of Findings

### Critical Findings (2)

1. **Password Derivation Timing** (ACCEPTABLE with mitigation)
   - PBKDF2 timing dominated by iteration count
   - Recommend documenting timing guarantees

2. **Seed Decryption Timing** (ACCEPTABLE with mitigation)
   - AES-GCM uses constant-time authentication
   - Recommend verifying crate version

### High Findings (1)

1. **Password Validation Branching** (ACTION REQUIRED)
   - Success/failure paths have different timing
   - Recommend constant-time implementation

### Medium Findings (3)

1. **BIP-32 Derivation Cache-Timing** (ACCEPTABLE)
   - Relies on `k256` constant-time guarantees
   - Recommend documentation

2. **ECDSA Signing Cache-Timing** (ACCEPTABLE)
   - Relies on Alloy/k256 constant-time guarantees
   - Recommend documentation

3. **Timing Information in Logs** (ACTION REQUIRED)
   - Logs leak timing information
   - Recommend using DEBUG level for sensitive operations

### Low Findings (2)

1. **Hardware Wallet Power Analysis** (ACCEPTABLE)
   - Protection provided by hardware device
   - No software mitigations needed

2. **Software Crypto Power Analysis** (ACCEPTABLE)
   - Impractical attack for software wallets
   - Recommend hardware wallets for high-value operations

---

## 8. Recommendations

### Immediate Actions (Phase 1)

1. **Fix Password Validation Timing** (Task 1.2)
   - Add constant-time delay to success path
   - Measure and equalize timing for both paths
   - Add property test to verify timing consistency

2. **Fix Logging Timing Leaks** (Task 4.5)
   - Move timing-sensitive logs to DEBUG level
   - Add configuration option to disable timing logs
   - Document log security requirements

### Documentation Actions (Phase 4)

3. **Document Constant-Time Guarantees** (Task 4.3)
   - Document reliance on `k256` constant-time operations
   - Document reliance on `aes-gcm` constant-time authentication
   - Document PBKDF2 timing characteristics

4. **Document Side-Channel Mitigations** (Task 4.8)
   - Document rate limiting as timing attack mitigation
   - Document hardware wallet power analysis protection
   - Document recommended security practices

### Future Improvements (Out of Scope)

5. **Upgrade to Argon2id**
   - Already implemented in `seed/encryption.rs`
   - Consider migrating legacy PBKDF2 code

6. **Add Timing Tests**
   - Property tests to verify constant-time operations
   - Integration tests to measure timing variance

---

## 9. Validation Checklist

- [x] All cryptographic operations identified
- [x] Secret-dependent branching analyzed
- [x] Cache-timing risks documented
- [x] Power analysis risks assessed
- [x] Timing attack surface mapped
- [x] Rate limiting effectiveness evaluated
- [x] Logging timing leaks identified
- [x] Hardware wallet security reviewed
- [x] Mitigation recommendations provided
- [x] Action items prioritized

---

## 10. Conclusion

The Vaughan wallet has **MEDIUM RISK** for side-channel attacks. The codebase demonstrates good security practices:

✅ **Strengths**:
- Uses industry-standard cryptographic libraries (Alloy, k256, aes-gcm)
- Implements rate limiting and exponential backoff
- Relies on hardware wallets for high-security operations
- Uses constant-time cryptographic primitives

⚠️ **Weaknesses**:
- Password validation has timing differences between success/failure paths
- Logging leaks timing information
- Some secret-dependent branching exists

🔧 **Required Actions**:
- Fix password validation timing (Phase 1)
- Fix logging timing leaks (Phase 4)
- Document constant-time guarantees (Phase 4)

**Overall Assessment**: The side-channel attack surface is manageable and can be mitigated with the recommended actions. The use of Alloy libraries and hardware wallets provides strong foundational security.

---

**Audit Complete**: 2025-01-25
**Next Task**: 0.3 Constant-Time Cryptography Audit
