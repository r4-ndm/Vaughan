//! Password validation service with rate limiting and security features
//!
//! This module provides password validation for seed-based accounts with:
//! - Rate limiting (3 attempts per minute)
//! - Exponential backoff on failures
//! - Account lockout after 5 failures
//! - Secure password verification via seed decryption

use crate::error::SecurityError;
use crate::gui::state::auth_state::PasswordError;
use crate::security::{KeyReference, SecureSeedStorage};
use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum password attempts per minute
const MAX_ATTEMPTS_PER_MINUTE: u32 = 3;

/// Maximum total failures before account lockout
const MAX_TOTAL_FAILURES: u32 = 5;

/// Lockout duration after max failures (15 minutes)
const LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60);

/// Rate limit window (1 minute)
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Attempt record for rate limiting
#[derive(Debug, Clone)]
struct AttemptRecord {
    pub(crate) timestamp: Instant,
    #[allow(dead_code)] // Used in tests to verify attempt recording (see test_attempt_recording)
    pub(crate) success: bool,
}

/// Account lockout state
#[derive(Debug, Clone)]
struct LockoutState {
    locked_until: Instant,
    total_failures: u32,
}

/// Password validation service with rate limiting
#[derive(Clone)]
pub struct PasswordValidator {
    /// Attempt history per account (for rate limiting)
    attempts: Arc<Mutex<HashMap<String, Vec<AttemptRecord>>>>,
    /// Account lockout state
    lockouts: Arc<Mutex<HashMap<String, LockoutState>>>,
    /// Seed storage for password verification (Arc for cloning, no Mutex needed for async)
    seed_storage: Arc<SecureSeedStorage>,
}

impl PasswordValidator {
    /// Create a new password validator
    pub fn new(seed_storage: SecureSeedStorage) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            lockouts: Arc::new(Mutex::new(HashMap::new())),
            seed_storage: Arc::new(seed_storage),
        }
    }

    /// Record a failed attempt for the master wallet password
    pub fn record_failed_attempt(&self) {
        self.record_attempt("wallet_master", false);
        self.increment_failures("wallet_master");
    }

    /// Reset attempts for the master wallet password
    pub fn reset_attempts(&self) {
        self.clear_failures("wallet_master");
    }

    /// Check if the master wallet is locked out
    pub fn is_locked_out(&self) -> bool {
        self.is_locked("wallet_master")
    }

    /// Validate a password by attempting to decrypt the seed phrase
    /// Returns the decrypted seed phrase on success
    pub async fn validate_password(
        &self,
        key_ref: &KeyReference,
        password: &SecretString,
    ) -> std::result::Result<SecretString, PasswordError> {
        let account_id = &key_ref.id;

        // Check if account is locked out
        if let Some(lockout) = self.check_lockout(account_id) {
            let remaining = lockout.locked_until.saturating_duration_since(Instant::now());
            return Err(PasswordError::AccountLocked {
                retry_after_seconds: remaining.as_secs(),
            });
        }

        // Check rate limiting
        self.check_rate_limit(account_id)?;

        // Attempt to decrypt the seed phrase with the password
        tracing::info!("🔓 Attempting to decrypt seed for account: {}", account_id);
        let validation_result = self
            .seed_storage
            .retrieve_encrypted_seed_phrase(key_ref, password)
            .await;

        // Record the attempt
        let success = validation_result.is_ok();
        self.record_attempt(account_id, success);

        // Handle validation result
        match validation_result {
            Ok(seed_phrase) => {
                // Password is correct - clear failure count
                tracing::info!("✅ Password validation successful for account: {}", account_id);
                self.clear_failures(account_id);
                Ok(seed_phrase)
            }
            Err(e) => {
                tracing::warn!(
                    "❌ Password validation failed for account: {} - Error: {:?}",
                    account_id,
                    e
                );
                // Password is incorrect - increment failure count
                let total_failures = self.increment_failures(account_id);

                // Check if we should lock the account
                if total_failures >= MAX_TOTAL_FAILURES {
                    self.lockout_account(account_id);
                    return Err(PasswordError::AccountLocked {
                        retry_after_seconds: LOCKOUT_DURATION.as_secs(),
                    });
                }

                // Map the error to PasswordError
                use crate::error::VaughanError;
                match e {
                    VaughanError::Security(security_error) => match security_error {
                        SecurityError::DecryptionError { .. } => Err(PasswordError::IncorrectPassword {
                            attempts_remaining: MAX_TOTAL_FAILURES.saturating_sub(total_failures),
                        }),
                        SecurityError::KeychainError { message } if message.contains("not found") => {
                            Err(PasswordError::SessionExpired)
                        }
                        _ => Err(PasswordError::DecryptionFailed),
                    },
                    _ => Err(PasswordError::DecryptionFailed),
                }
            }
        }
    }

    /// Check if account is currently locked out
    fn check_lockout(&self, account_id: &str) -> Option<LockoutState> {
        let lockouts = self.lockouts.lock().unwrap();
        if let Some(lockout) = lockouts.get(account_id) {
            if lockout.locked_until > Instant::now() {
                return Some(lockout.clone());
            }
        }
        None
    }

    /// Check rate limiting for an account
    fn check_rate_limit(&self, account_id: &str) -> std::result::Result<(), PasswordError> {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();

        // Get recent attempts within the rate limit window
        let recent_attempts = attempts
            .entry(account_id.to_string())
            .or_default()
            .iter()
            .filter(|a| now.duration_since(a.timestamp) < RATE_LIMIT_WINDOW)
            .count();

        if recent_attempts >= MAX_ATTEMPTS_PER_MINUTE as usize {
            // Calculate backoff time (exponential)
            let backoff_seconds = 2u64.pow((recent_attempts as u32).saturating_sub(MAX_ATTEMPTS_PER_MINUTE));
            return Err(PasswordError::TooManyAttempts {
                retry_after_seconds: backoff_seconds.min(300), // Max 5 minutes
            });
        }

        Ok(())
    }

    /// Record a password attempt
    fn record_attempt(&self, account_id: &str, success: bool) {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();

        let account_attempts = attempts.entry(account_id.to_string()).or_default();

        // Add new attempt
        account_attempts.push(AttemptRecord {
            timestamp: now,
            success,
        });

        // Clean up old attempts (older than rate limit window)
        account_attempts.retain(|a| now.duration_since(a.timestamp) < RATE_LIMIT_WINDOW);
    }

    /// Increment failure count and return total failures
    fn increment_failures(&self, account_id: &str) -> u32 {
        let mut lockouts = self.lockouts.lock().unwrap();
        let lockout = lockouts.entry(account_id.to_string()).or_insert_with(|| LockoutState {
            locked_until: Instant::now(),
            total_failures: 0,
        });

        lockout.total_failures += 1;
        lockout.total_failures
    }

    /// Clear failure count after successful validation
    fn clear_failures(&self, account_id: &str) {
        let mut lockouts = self.lockouts.lock().unwrap();
        lockouts.remove(account_id);
    }

    /// Lock out an account for the lockout duration
    fn lockout_account(&self, account_id: &str) {
        let mut lockouts = self.lockouts.lock().unwrap();
        let lockout = lockouts.entry(account_id.to_string()).or_insert_with(|| LockoutState {
            locked_until: Instant::now(),
            total_failures: MAX_TOTAL_FAILURES,
        });

        lockout.locked_until = Instant::now() + LOCKOUT_DURATION;
        tracing::warn!(
            "Account {} locked out for {} minutes due to too many failed attempts",
            account_id,
            LOCKOUT_DURATION.as_secs() / 60
        );
    }

    /// Get remaining attempts before lockout
    pub fn get_remaining_attempts(&self, account_id: &str) -> u32 {
        let lockouts = self.lockouts.lock().unwrap();
        if let Some(lockout) = lockouts.get(account_id) {
            MAX_TOTAL_FAILURES.saturating_sub(lockout.total_failures)
        } else {
            MAX_TOTAL_FAILURES
        }
    }

    /// Check if account is locked
    pub fn is_locked(&self, account_id: &str) -> bool {
        self.check_lockout(account_id).is_some()
    }

    /// Get time until account is unlocked (in seconds)
    pub fn get_lockout_remaining(&self, account_id: &str) -> Option<u64> {
        self.check_lockout(account_id)
            .map(|lockout| lockout.locked_until.saturating_duration_since(Instant::now()).as_secs())
    }

    /// Clear all rate limiting and lockout data (for testing)
    #[cfg(test)]
    pub fn clear_all(&self) {
        let mut attempts = self.attempts.lock().unwrap();
        let mut lockouts = self.lockouts.lock().unwrap();
        attempts.clear();
        lockouts.clear();
    }
}

impl std::fmt::Debug for PasswordValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordValidator")
            .field("attempts", &"[REDACTED]")
            .field("lockouts", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require a working keychain and seed storage
    // These are unit tests for the rate limiting and lockout logic

    #[test]
    fn test_initial_state() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        assert_eq!(validator.get_remaining_attempts("test_account"), MAX_TOTAL_FAILURES);
        assert!(!validator.is_locked("test_account"));
        assert_eq!(validator.get_lockout_remaining("test_account"), None);
    }

    #[test]
    fn test_failure_increment() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account_id = "test_account";

        // Increment failures
        for i in 1..=3 {
            validator.increment_failures(account_id);
            assert_eq!(validator.get_remaining_attempts(account_id), MAX_TOTAL_FAILURES - i);
        }
    }

    #[test]
    fn test_lockout_after_max_failures() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account_id = "test_account";

        // Increment to max failures
        for _ in 0..MAX_TOTAL_FAILURES {
            validator.increment_failures(account_id);
        }

        // Lock the account
        validator.lockout_account(account_id);

        // Verify account is locked
        assert!(validator.is_locked(account_id));
        assert_eq!(validator.get_remaining_attempts(account_id), 0);

        // Verify lockout duration
        let remaining = validator.get_lockout_remaining(account_id);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > 0);
    }

    #[test]
    fn test_clear_failures_on_success() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account_id = "test_account";

        // Add some failures
        validator.increment_failures(account_id);
        validator.increment_failures(account_id);
        assert_eq!(validator.get_remaining_attempts(account_id), MAX_TOTAL_FAILURES - 2);

        // Clear failures (simulating successful validation)
        validator.clear_failures(account_id);

        // Verify failures are cleared
        assert_eq!(validator.get_remaining_attempts(account_id), MAX_TOTAL_FAILURES);
    }

    #[test]
    fn test_attempt_recording() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account_id = "test_account";

        // Record some attempts
        validator.record_attempt(account_id, false);
        validator.record_attempt(account_id, false);
        validator.record_attempt(account_id, true);

        // Verify attempts are recorded (internal state)
        let attempts = validator.attempts.lock().unwrap();
        let account_attempts = attempts.get(account_id).unwrap();
        assert_eq!(account_attempts.len(), 3);
        assert!(!account_attempts[0].success);
        assert!(!account_attempts[1].success);
        assert!(account_attempts[2].success);
    }

    #[test]
    fn test_rate_limit_check() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account_id = "test_account";

        // Record attempts up to the limit
        for _ in 0..MAX_ATTEMPTS_PER_MINUTE {
            validator.record_attempt(account_id, false);
        }

        // Next attempt should be rate limited
        let result = validator.check_rate_limit(account_id);
        assert!(result.is_err());

        if let Err(PasswordError::TooManyAttempts { retry_after_seconds }) = result {
            assert!(retry_after_seconds > 0);
        } else {
            panic!("Expected TooManyAttempts error");
        }
    }

    #[test]
    fn test_clear_all() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account_id = "test_account";

        // Add some state
        validator.increment_failures(account_id);
        validator.record_attempt(account_id, false);

        // Clear all
        validator.clear_all();

        // Verify everything is cleared
        assert_eq!(validator.get_remaining_attempts(account_id), MAX_TOTAL_FAILURES);
        assert!(!validator.is_locked(account_id));

        let attempts = validator.attempts.lock().unwrap();
        assert!(attempts.is_empty());
    }

    #[test]
    fn test_multiple_accounts() {
        let keychain = crate::security::keychain::OSKeychain::new("test".to_string()).unwrap();
        let seed_storage = SecureSeedStorage::new(Box::new(keychain));
        let validator = PasswordValidator::new(seed_storage);

        let account1 = "account1";
        let account2 = "account2";

        // Add failures to account1
        validator.increment_failures(account1);
        validator.increment_failures(account1);

        // Add failures to account2
        validator.increment_failures(account2);

        // Verify independent tracking
        assert_eq!(validator.get_remaining_attempts(account1), MAX_TOTAL_FAILURES - 2);
        assert_eq!(validator.get_remaining_attempts(account2), MAX_TOTAL_FAILURES - 1);

        // Clear account1
        validator.clear_failures(account1);

        // Verify account1 is cleared but account2 is not
        assert_eq!(validator.get_remaining_attempts(account1), MAX_TOTAL_FAILURES);
        assert_eq!(validator.get_remaining_attempts(account2), MAX_TOTAL_FAILURES - 1);
    }
}
