//! Wallet Password Validation and Rate Limiting
//!
//! This module provides wallet master password validation with rate limiting
//! and security features, separate from individual account password validation.

use crate::error::SecurityError;
use crate::security::{WalletConfig, WalletConfigStorage};
use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::gui::state::auth_state::WalletPasswordError;

/// Maximum consecutive failures before wallet lockout
const MAX_WALLET_FAILURES: u32 = 5;

/// Lockout duration after too many failures (15 minutes for wallet)
const WALLET_LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60);

/// Rate limiting window (1 minute)
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Maximum attempts per rate limiting window
const MAX_ATTEMPTS_PER_WINDOW: u32 = 10;

/// Wallet password validation with rate limiting and security
#[derive(Debug, Clone)]
pub struct WalletPasswordValidator {
    /// Wallet configuration storage
    storage: WalletConfigStorage,

    /// Rate limiting: track attempts by wallet ID
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,

    /// Account lockouts: track lockout times by wallet ID
    lockouts: Arc<Mutex<HashMap<String, Instant>>>,

    /// Failed password attempts counter by wallet ID
    failures: Arc<Mutex<HashMap<String, u32>>>,
}

impl WalletPasswordValidator {
    /// Create a new wallet password validator with default storage
    pub fn new() -> crate::error::Result<Self> {
        let storage = WalletConfigStorage::new()?;
        Ok(Self::new_with_storage(storage))
    }

    /// Create a new wallet password validator with custom storage
    pub fn new_with_storage(storage: WalletConfigStorage) -> Self {
        Self {
            storage,
            attempts: Arc::new(Mutex::new(HashMap::new())),
            lockouts: Arc::new(Mutex::new(HashMap::new())),
            failures: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Validate wallet master password
    pub async fn validate_wallet_password(
        &self,
        password: &SecretString,
    ) -> std::result::Result<WalletConfig, WalletPasswordError> {
        // Get wallet ID from storage metadata
        let wallet_metadata = self
            .storage
            .get_wallet_info()
            .map_err(|_| WalletPasswordError::WalletNotFound)?;

        let wallet_id = match wallet_metadata {
            Some(metadata) => metadata.wallet_id,
            None => return Err(WalletPasswordError::WalletNotFound),
        };

        // Check if wallet is currently locked out
        self.check_rate_limit(&wallet_id)?;

        tracing::info!("🔓 Attempting to validate wallet password for wallet: {}", wallet_id);

        // Attempt to load and validate wallet config
        let validation_result = self.storage.load_wallet_config(password).await;

        // Record the attempt
        let success = validation_result.is_ok();
        self.record_attempt(&wallet_id, success);

        // Handle validation result
        match validation_result {
            Ok(Some(wallet_config)) => {
                // Password is correct - clear failure count
                tracing::info!("✅ Wallet password validation successful for wallet: {}", wallet_id);
                self.clear_failures(&wallet_id);
                Ok(wallet_config)
            }
            Ok(None) => {
                tracing::warn!("❌ Wallet not found during password validation");
                Err(WalletPasswordError::WalletNotFound)
            }
            Err(e) => {
                tracing::warn!(
                    "❌ Wallet password validation failed for wallet: {} - Error: {:?}",
                    wallet_id,
                    e
                );

                // Password is incorrect - increment failure count
                let total_failures = self.increment_failures(&wallet_id);

                // Check if we should lock the wallet
                if total_failures >= MAX_WALLET_FAILURES {
                    self.lockout_wallet(&wallet_id);
                    return Err(WalletPasswordError::AccountLocked {
                        retry_after_seconds: WALLET_LOCKOUT_DURATION.as_secs(),
                    });
                }

                // Map the error to WalletPasswordError
                use crate::error::VaughanError;
                match e {
                    VaughanError::Security(security_error) => match security_error {
                        SecurityError::DecryptionError { .. } => Err(WalletPasswordError::IncorrectPassword {
                            attempts_remaining: MAX_WALLET_FAILURES.saturating_sub(total_failures),
                        }),
                        SecurityError::KeystoreError { message } if message.contains("not found") => {
                            Err(WalletPasswordError::WalletNotFound)
                        }
                        _ => Err(WalletPasswordError::DecryptionFailed),
                    },
                    _ => Err(WalletPasswordError::DecryptionFailed),
                }
            }
        }
    }

    /// Create a new wallet with master password
    pub async fn create_wallet(
        &self,
        wallet_name: String,
        password: &SecretString,
    ) -> std::result::Result<WalletConfig, WalletPasswordError> {
        // Validate password strength
        if let Err(requirements) = self.validate_password_strength(password) {
            return Err(WalletPasswordError::WeakPassword { requirements });
        }

        tracing::info!("🔧 Creating new wallet: {}", wallet_name);

        // Create wallet configuration
        let wallet_config = self
            .storage
            .create_wallet_config(wallet_name.clone(), password)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create wallet: {:?}", e);
                WalletPasswordError::DecryptionFailed
            })?;

        tracing::info!("✅ Wallet created successfully: {}", wallet_name);
        Ok(wallet_config)
    }

    /// Check if wallet exists
    pub fn wallet_exists(&self) -> bool {
        self.storage.wallet_exists().unwrap_or(false)
    }

    /// Get remaining attempts before lockout for wallet
    pub fn get_remaining_attempts(&self, wallet_id: &str) -> u32 {
        if let Ok(failures) = self.failures.lock() {
            let current_failures = failures.get(wallet_id).copied().unwrap_or(0);
            MAX_WALLET_FAILURES.saturating_sub(current_failures)
        } else {
            MAX_WALLET_FAILURES
        }
    }

    /// Check if wallet is currently locked
    pub fn is_locked(&self, wallet_id: &str) -> bool {
        if let Ok(lockouts) = self.lockouts.lock() {
            if let Some(&lockout_time) = lockouts.get(wallet_id) {
                lockout_time.elapsed() < WALLET_LOCKOUT_DURATION
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get time remaining in lockout
    pub fn get_lockout_remaining(&self, wallet_id: &str) -> Option<Duration> {
        if let Ok(lockouts) = self.lockouts.lock() {
            if let Some(&lockout_time) = lockouts.get(wallet_id) {
                let elapsed = lockout_time.elapsed();
                if elapsed < WALLET_LOCKOUT_DURATION {
                    Some(WALLET_LOCKOUT_DURATION - elapsed)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Validate password strength for wallet passwords
    fn validate_password_strength(&self, password: &SecretString) -> std::result::Result<(), Vec<String>> {
        use secrecy::ExposeSecret;
        let password_str = password.expose_secret();
        let mut requirements = Vec::new();

        // Minimum length (12 characters for wallet passwords)
        if password_str.len() < 12 {
            requirements.push("At least 12 characters".to_string());
        }

        // Must contain uppercase letter
        if !password_str.chars().any(|c| c.is_uppercase()) {
            requirements.push("At least one uppercase letter".to_string());
        }

        // Must contain lowercase letter
        if !password_str.chars().any(|c| c.is_lowercase()) {
            requirements.push("At least one lowercase letter".to_string());
        }

        // Must contain digit
        if !password_str.chars().any(|c| c.is_numeric()) {
            requirements.push("At least one number".to_string());
        }

        // Must contain special character
        if !password_str
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;':\",./<>?".contains(c))
        {
            requirements.push("At least one special character".to_string());
        }

        // Check for common weak patterns
        let lower_password = password_str.to_lowercase();
        let weak_patterns = [
            "password", "123456", "qwerty", "admin", "wallet", "crypto", "bitcoin", "ethereum",
        ];

        for pattern in &weak_patterns {
            if lower_password.contains(pattern) {
                requirements.push(format!("Must not contain '{pattern}'"));
            }
        }

        if requirements.is_empty() {
            Ok(())
        } else {
            Err(requirements)
        }
    }

    /// Check rate limiting for wallet authentication attempts
    fn check_rate_limit(&self, wallet_id: &str) -> std::result::Result<(), WalletPasswordError> {
        let now = Instant::now();

        // Check if wallet is in lockout period
        if let Ok(lockouts) = self.lockouts.lock() {
            if let Some(&lockout_time) = lockouts.get(wallet_id) {
                if lockout_time.elapsed() < WALLET_LOCKOUT_DURATION {
                    let remaining = WALLET_LOCKOUT_DURATION - lockout_time.elapsed();
                    return Err(WalletPasswordError::AccountLocked {
                        retry_after_seconds: remaining.as_secs(),
                    });
                }
            }
        }

        // Check rate limiting
        if let Ok(mut attempts) = self.attempts.lock() {
            let wallet_attempts = attempts.entry(wallet_id.to_string()).or_default();

            // Remove attempts outside the rate limiting window
            wallet_attempts.retain(|&time| now.duration_since(time) < RATE_LIMIT_WINDOW);

            // Check if we're over the rate limit
            if wallet_attempts.len() >= MAX_ATTEMPTS_PER_WINDOW as usize {
                return Err(WalletPasswordError::TooManyAttempts {
                    retry_after_seconds: 60, // Rate limit window
                });
            }
        }

        Ok(())
    }

    /// Record a wallet authentication attempt
    fn record_attempt(&self, wallet_id: &str, _success: bool) {
        let now = Instant::now();

        if let Ok(mut attempts) = self.attempts.lock() {
            let wallet_attempts = attempts.entry(wallet_id.to_string()).or_default();
            wallet_attempts.push(now);

            // Keep only recent attempts
            wallet_attempts.retain(|&time| now.duration_since(time) < RATE_LIMIT_WINDOW);
        }
    }

    /// Increment failure count for wallet
    fn increment_failures(&self, wallet_id: &str) -> u32 {
        if let Ok(mut failures) = self.failures.lock() {
            let count = failures.entry(wallet_id.to_string()).or_insert(0);
            *count += 1;
            *count
        } else {
            1
        }
    }

    /// Clear failure count for wallet
    fn clear_failures(&self, wallet_id: &str) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.remove(wallet_id);
        }
    }

    /// Lock out wallet after too many failures
    fn lockout_wallet(&self, wallet_id: &str) {
        tracing::warn!("🚫 Locking wallet due to too many failed attempts: {}", wallet_id);

        if let Ok(mut lockouts) = self.lockouts.lock() {
            lockouts.insert(wallet_id.to_string(), Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::OSKeychain;

    #[tokio::test]
    async fn test_wallet_password_validator() {
        let _keychain = Box::new(OSKeychain::new("test-wallet-validator".to_string()).unwrap());
        let _storage = WalletConfigStorage::new().unwrap();
        let validator = WalletPasswordValidator::new().unwrap();

        // Clean up any existing test wallet
        let _ = validator.storage.factory_reset().await;

        let strong_password = SecretString::new("TestVault123!@#".to_string());
        let wallet_name = "Test Wallet".to_string();

        // Create wallet
        let wallet_config = validator
            .create_wallet(wallet_name.clone(), &strong_password)
            .await
            .unwrap();
        assert_eq!(wallet_config.wallet_name, wallet_name);

        // Validate correct password
        let validated_config = validator.validate_wallet_password(&strong_password).await.unwrap();
        assert_eq!(validated_config.wallet_id, wallet_config.wallet_id);

        // Test wrong password
        let wrong_password = SecretString::new("WrongPassword123!".to_string());
        let result = validator.validate_wallet_password(&wrong_password).await;
        assert!(result.is_err());

        // Clean up
        let _ = validator.storage.delete_wallet_config(&strong_password).await;
    }

    #[test]
    fn test_password_strength_validation() {
        let _keychain = Box::new(OSKeychain::new("test-strength".to_string()).unwrap());
        let _storage = WalletConfigStorage::new().unwrap();
        let validator = WalletPasswordValidator::new().expect("Failed to create validator");

        // Test weak passwords
        let weak_passwords = vec![
            "short",             // Too short
            "alllowercase123!",  // No uppercase
            "ALLUPPERCASE123!",  // No lowercase
            "NoNumbers!@#",      // No digits
            "NoSpecialChars123", // No special characters
            "password123!",      // Contains "password"
        ];

        for password in weak_passwords {
            let secret = SecretString::new(password.to_string());
            assert!(validator.validate_password_strength(&secret).is_err());
        }

        // Test strong password
        let strong_password = SecretString::new("StrongVault123!@#".to_string());
        assert!(validator.validate_password_strength(&strong_password).is_ok());
    }
}
