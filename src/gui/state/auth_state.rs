//! Security State Module
//!
//! Unified module for all security-related state including:
//! - Password dialog state
//! - Session management
//! - Key cache handling

use secrecy::SecretString;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::security::key_cache::KeyCache;

/// Unified authentication state consolidating security and session management
#[derive(Debug, Clone, Default)]
pub struct AuthState {
    /// Password dialog state (for account operations)
    pub password_dialog: PasswordDialogState,

    /// Session management state (legacy)
    pub session: SessionState,

    /// Enhanced session management with wallet/account separation
    pub enhanced_session: EnhancedSessionState,

    /// Key cache handle (actual cache stored separately for security)
    pub key_cache_handle: Option<Arc<RwLock<KeyCache>>>,

    /// Password validator service (persists rate limiting state)
    pub password_validator: Option<crate::security::PasswordValidator>,
}

impl AuthState {
    /// Unlock the session
    pub fn unlock(&mut self) {
        self.session.unlock();
        // Also update enhanced session if needed
        self.enhanced_session.wallet_session.is_unlocked = true; // Sync for now
        self.enhanced_session.wallet_session.unlocked_at = Some(Instant::now());
        self.enhanced_session.wallet_session.last_activity = Instant::now();

        if let Some(validator) = &mut self.password_validator {
            validator.reset_attempts();
        }
    }

    /// Lock the session with key zeroization
    pub fn lock(&mut self) {
        self.session.lock();
        self.enhanced_session.wallet_session.lock();

        // Ensure dialog is hidden and inputs zeroized
        self.password_dialog.hide();

        // Clear sensitive data
        if let Some(_key_cache) = &self.key_cache_handle {
            // We can't easily clear the RwLock<KeyCache> here without async or blocking,
            // but we've dropped the session keys.
        }
    }

    /// Record activity to prevent timeout
    pub fn record_activity(&mut self) {
        self.session.update_activity();
        self.enhanced_session.wallet_session.update_activity();
        self.enhanced_session.update_global_activity();
    }

    /// Check if session is timed out
    pub fn is_timed_out(&self) -> bool {
        self.session.is_timed_out() || self.enhanced_session.wallet_session.is_timed_out()
    }

    /// Check if locked out due to too many attempts
    pub fn is_locked_out(&self) -> bool {
        if let Some(validator) = &self.password_validator {
            validator.is_locked_out()
        } else {
            false
        }
    }

    /// Record a failed password attempt
    pub fn record_failed_attempt(&mut self) {
        if let Some(validator) = &mut self.password_validator {
            validator.record_failed_attempt();
        }
        self.password_dialog.attempts += 1;
    }

    /// Clear lockout state
    pub fn clear_lockout(&mut self) {
        if let Some(validator) = &mut self.password_validator {
            validator.reset_attempts();
        }
        self.password_dialog.attempts = 0;
    }
}

/// Unified password dialog state
#[derive(Debug, Clone)]
pub struct PasswordDialogState {
    /// Whether the dialog is visible
    pub visible: bool,

    /// Configuration for the current dialog instance
    pub config: Option<PasswordDialogConfig>,

    /// Current password input (stored as SecretString for security)
    pub input: SecretString,

    /// Current new password input (for change/create flows)
    pub new_password_input: SecretString,

    /// Current confirm password input (for change/create flows)
    pub confirm_password_input: SecretString,

    /// Current error, if any
    pub error: Option<PasswordError>,

    /// Number of failed attempts
    pub attempts: u32,

    /// Whether to remember the session (cache keys)
    pub remember_session: bool,
}

/// Configuration for the password dialog
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordDialogConfig {
    /// Wallet-level authentication
    WalletUnlock,

    /// Account-level authentication
    AccountUnlock { account_id: String, account_name: String },

    /// Unlock wallet for export operations
    WalletExport,

    /// Signing a transaction
    SignTransaction { tx_details: String },

    /// Wallet setup (first time)
    WalletSetup { wallet_name: String },

    /// Change Password
    ChangePassword {
        is_wallet_password: bool, // true for wallet master, false for account
    },

    /// Export Private Key
    ExportPrivateKey { account_name: String },

    /// Export Seed Phrase
    ExportSeedPhrase { account_name: String },

    /// Import Wallet
    ImportWallet,

    /// Delete Account
    DeleteAccount { account_name: String },

    /// Reset Wallet (Factory Reset)
    ResetWallet,

    /// Add Account (various types)
    AddAccount { creation_type: AccountCreationType },

    /// General confirmation for sensitive operations
    ConfirmOperation { operation: WalletOperation },
}

/// Password validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordError {
    /// Password is incorrect
    IncorrectPassword { attempts_remaining: u32 },

    /// Decryption failed (corrupted data or wrong password)
    DecryptionFailed,

    /// Password field is empty
    EmptyPassword,

    /// Too many failed attempts
    TooManyAttempts { retry_after_seconds: u64 },

    /// Session has expired
    SessionExpired,

    /// Account is locked
    AccountLocked { retry_after_seconds: u64 },
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::IncorrectPassword { attempts_remaining } => {
                write!(f, "Incorrect password ({attempts_remaining} attempts remaining)")
            }
            PasswordError::DecryptionFailed => {
                write!(f, "Failed to decrypt - password may be incorrect or data corrupted")
            }
            PasswordError::EmptyPassword => write!(f, "Password cannot be empty"),
            PasswordError::TooManyAttempts { retry_after_seconds } => {
                write!(
                    f,
                    "Too many failed attempts - please wait {retry_after_seconds} seconds"
                )
            }
            PasswordError::SessionExpired => {
                write!(f, "Session has expired - please enter your password again")
            }
            PasswordError::AccountLocked { retry_after_seconds } => {
                write!(
                    f,
                    "Account is locked due to too many failed attempts. Try again in {retry_after_seconds} seconds"
                )
            }
        }
    }
}

impl From<crate::error::SecurityError> for PasswordError {
    fn from(error: crate::error::SecurityError) -> Self {
        use crate::error::SecurityError;

        match error {
            SecurityError::DecryptionError { .. } => PasswordError::DecryptionFailed,
            SecurityError::InvalidSeedPhrase { .. } => PasswordError::IncorrectPassword { attempts_remaining: 0 },
            SecurityError::KeystoreError { message } => {
                // Check if it's a specific error we can map
                if message.contains("incorrect") || message.contains("wrong password") {
                    PasswordError::IncorrectPassword { attempts_remaining: 0 }
                } else if message.contains("locked") {
                    PasswordError::AccountLocked { retry_after_seconds: 0 }
                } else {
                    PasswordError::DecryptionFailed
                }
            }
            SecurityError::KeychainError { .. } => PasswordError::DecryptionFailed,
            _ => PasswordError::DecryptionFailed,
        }
    }
}

/// Session management state
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Whether the session is currently unlocked
    pub is_unlocked: bool,

    /// When the session was unlocked
    pub unlocked_at: Option<Instant>,

    /// Last activity timestamp (for extending timeout)
    pub last_activity: Instant,

    /// Session timeout duration
    pub timeout_duration: Duration,

    /// Whether auto-lock is enabled
    pub auto_lock_enabled: bool,

    /// Whether to lock on window minimize
    pub lock_on_minimize: bool,

    /// Cached password for seed-based account signing (cleared on lock)
    /// This is stored in memory only and cleared when session locks
    pub cached_password: Option<secrecy::SecretString>,

    /// Temporary key for immediate transaction signing when "remember" is disabled
    /// This is cleared immediately after use
    pub temporary_key: Option<secrecy::SecretString>,
}

impl Default for PasswordDialogState {
    fn default() -> Self {
        Self {
            visible: false,
            config: None,
            input: SecretString::new(String::new()),
            new_password_input: SecretString::new(String::new()),
            confirm_password_input: SecretString::new(String::new()),
            error: None,
            attempts: 0,
            remember_session: true, // Default to remembering session
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            // Default to locked for security
            is_unlocked: false,
            unlocked_at: None,
            last_activity: Instant::now(),
            timeout_duration: Duration::from_secs(15 * 60), // 15 minutes default
            auto_lock_enabled: true,
            lock_on_minimize: false,
            cached_password: None,
            temporary_key: None,
        }
    }
}

impl SessionState {
    /// Check if the session has timed out
    pub fn is_timed_out(&self) -> bool {
        if !self.is_unlocked || !self.auto_lock_enabled {
            return false;
        }

        self.last_activity.elapsed() >= self.timeout_duration
    }

    /// Get remaining time before timeout
    pub fn time_until_timeout(&self) -> Option<Duration> {
        if !self.is_unlocked || !self.auto_lock_enabled {
            return None;
        }

        let elapsed = self.last_activity.elapsed();
        if elapsed >= self.timeout_duration {
            Some(Duration::ZERO)
        } else {
            Some(self.timeout_duration - elapsed)
        }
    }

    /// Update last activity (extends session)
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Lock the session
    pub fn lock(&mut self) {
        self.is_unlocked = false;
        self.unlocked_at = None;
        // Clear cached password for security (Requirement 7.2)
        self.cached_password = None;
        self.temporary_key = None;
    }

    /// Unlock the session
    pub fn unlock(&mut self) {
        self.is_unlocked = true;
        self.unlocked_at = Some(Instant::now());
        self.last_activity = Instant::now();
    }
}

impl PasswordDialogState {
    /// Reset the dialog state
    pub fn reset(&mut self) {
        self.visible = false;
        self.config = None;
        self.input = SecretString::new(String::new());
        self.new_password_input = SecretString::new(String::new());
        self.confirm_password_input = SecretString::new(String::new());
        self.error = None;
        // Don't reset attempts - that's handled by rate limiter
    }

    /// Show the dialog with a config
    pub fn show(&mut self, config: PasswordDialogConfig) {
        self.visible = true;
        self.config = Some(config);
        self.input = SecretString::new(String::new());
        self.new_password_input = SecretString::new(String::new());
        self.confirm_password_input = SecretString::new(String::new());
        self.error = None;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.reset();
    }

    /// Set an error
    pub fn set_error(&mut self, error: PasswordError) {
        self.error = Some(error);
        self.attempts += 1;
    }

    /// Clear the error
    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_state_default() {
        let state = AuthState::default();

        assert!(!state.password_dialog.visible);
        assert!(state.password_dialog.config.is_none());
        assert!(state.password_dialog.error.is_none());
        assert_eq!(state.password_dialog.attempts, 0);
        assert!(state.password_dialog.remember_session);

        assert!(!state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_none());
        assert!(state.session.auto_lock_enabled);
        assert!(!state.session.lock_on_minimize);
    }

    #[test]
    fn test_password_dialog_show_hide() {
        let mut state = AuthState::default();

        // Show dialog
        state.password_dialog.show(PasswordDialogConfig::WalletUnlock);
        assert!(state.password_dialog.visible);
        assert_eq!(state.password_dialog.config, Some(PasswordDialogConfig::WalletUnlock));

        // Hide dialog
        state.password_dialog.hide();
        assert!(!state.password_dialog.visible);
        assert!(state.password_dialog.config.is_none());
    }

    #[test]
    fn test_password_dialog_error_handling() {
        let mut state = AuthState::default();

        // Set error
        state
            .password_dialog
            .set_error(PasswordError::IncorrectPassword { attempts_remaining: 3 });
        assert!(state.password_dialog.error.is_some());
        assert_eq!(state.password_dialog.attempts, 1);

        // Clear error
        state.password_dialog.clear_error();
        assert!(state.password_dialog.error.is_none());
    }

    #[test]
    fn test_session_lock_unlock() {
        let mut state = AuthState::default();

        // Unlock session
        state.session.unlock();
        assert!(state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_some());

        // Lock session
        state.session.lock();
        assert!(!state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_none());
    }

    #[test]
    fn test_session_timeout() {
        let mut state = AuthState::default();

        // Set short timeout for testing
        state.session.timeout_duration = Duration::from_millis(100);
        state.session.unlock();

        // Should not be timed out immediately
        assert!(!state.session.is_timed_out());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should be timed out now
        assert!(state.session.is_timed_out());
    }

    #[test]
    fn test_password_error_display() {
        let error1 = PasswordError::IncorrectPassword { attempts_remaining: 3 };
        assert!(error1.to_string().contains("3 attempts remaining"));

        let error2 = PasswordError::TooManyAttempts {
            retry_after_seconds: 60,
        };
        assert!(error2.to_string().contains("60 seconds"));

        let error3 = PasswordError::AccountLocked {
            retry_after_seconds: 900,
        };
        assert!(error3.to_string().contains("900 seconds"));
    }
}

// ==============================================================================
// Wallet Password State (merged from wallet_password_state.rs)
// ==============================================================================

use serde::{Deserialize, Serialize};

/// Types of sensitive wallet operations that require password confirmation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalletOperation {
    /// Backing up wallet configuration
    BackupWallet,

    /// Restoring wallet from backup
    RestoreWallet,

    /// Changing security settings
    ChangeSecuritySettings,

    /// Viewing wallet seed phrases (all accounts)
    ViewAllSeeds,

    /// Factory reset confirmation
    FactoryReset,

    /// Network security settings changes
    NetworkSecurityChanges,
}

/// Types of account creation that require wallet password
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountCreationType {
    /// Create new account with generated seed
    GenerateNew,

    /// Import account from existing seed phrase
    ImportFromSeed,

    /// Import account from private key
    ImportFromPrivateKey,

    /// Connect hardware wallet account
    HardwareWallet,

    /// Derive additional account from existing seed
    DeriveFromExisting { parent_account_id: String },
}

/// Wallet password specific errors (separate from account password errors)
#[derive(Debug, Clone, PartialEq)]
pub enum WalletPasswordError {
    /// Wallet password is incorrect
    IncorrectPassword { attempts_remaining: u32 },

    /// Wallet decryption failed (corrupted data or wrong password)
    DecryptionFailed,

    /// Wallet password field is empty
    EmptyPassword,

    /// Too many failed attempts for wallet
    TooManyAttempts { retry_after_seconds: u64 },

    /// Wallet session has expired
    SessionExpired,

    /// Wallet is locked due to security policy
    AccountLocked { retry_after_seconds: u64 },

    /// Wallet not found (first time setup required)
    WalletNotFound,

    /// Wallet migration required
    MigrationRequired { legacy_format_version: u32 },

    /// Wallet password confirmation mismatch
    PasswordMismatch,

    /// New password doesn't meet security requirements
    WeakPassword { requirements: Vec<String> },

    /// Invalid input provided
    InvalidInput { message: String },

    /// Wallet creation failed
    CreationFailed { reason: String },
}

impl std::fmt::Display for WalletPasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletPasswordError::IncorrectPassword { attempts_remaining } => {
                write!(f, "Incorrect wallet password ({attempts_remaining} attempts remaining)")
            }
            WalletPasswordError::DecryptionFailed => {
                write!(
                    f,
                    "Failed to decrypt wallet - password may be incorrect or data corrupted"
                )
            }
            WalletPasswordError::EmptyPassword => write!(f, "Wallet password cannot be empty"),
            WalletPasswordError::TooManyAttempts { retry_after_seconds } => {
                write!(
                    f,
                    "Too many failed attempts - please wait {retry_after_seconds} seconds"
                )
            }
            WalletPasswordError::SessionExpired => {
                write!(f, "Wallet session has expired - please enter your password again")
            }
            WalletPasswordError::AccountLocked { retry_after_seconds } => {
                write!(
                    f,
                    "Wallet is locked due to too many failed attempts. Try again in {retry_after_seconds} seconds"
                )
            }
            WalletPasswordError::WalletNotFound => {
                write!(f, "No wallet found - please set up a new wallet first")
            }
            WalletPasswordError::MigrationRequired { legacy_format_version } => {
                write!(
                    f,
                    "Wallet migration required from format version {legacy_format_version} to current version"
                )
            }
            WalletPasswordError::PasswordMismatch => {
                write!(f, "Password confirmation does not match")
            }
            WalletPasswordError::WeakPassword { requirements } => {
                write!(f, "Password is too weak. Requirements: {}", requirements.join(", "))
            }
            WalletPasswordError::InvalidInput { message } => {
                write!(f, "Invalid input: {message}")
            }
            WalletPasswordError::CreationFailed { reason } => {
                write!(f, "Wallet creation failed: {reason}")
            }
        }
    }
}

impl From<WalletPasswordError> for PasswordError {
    fn from(error: WalletPasswordError) -> Self {
        match error {
            WalletPasswordError::IncorrectPassword { attempts_remaining } => {
                PasswordError::IncorrectPassword { attempts_remaining }
            }
            WalletPasswordError::DecryptionFailed => PasswordError::DecryptionFailed,
            WalletPasswordError::EmptyPassword => PasswordError::EmptyPassword,
            WalletPasswordError::TooManyAttempts { retry_after_seconds } => {
                PasswordError::TooManyAttempts { retry_after_seconds }
            }
            WalletPasswordError::SessionExpired => PasswordError::SessionExpired,
            WalletPasswordError::AccountLocked { retry_after_seconds } => {
                PasswordError::AccountLocked { retry_after_seconds }
            }
            // Map legacy specific errors to generic ones or closest match
            WalletPasswordError::WalletNotFound => PasswordError::DecryptionFailed,
            WalletPasswordError::MigrationRequired { .. } => PasswordError::DecryptionFailed,
            WalletPasswordError::PasswordMismatch => PasswordError::DecryptionFailed,
            WalletPasswordError::WeakPassword { .. } => PasswordError::DecryptionFailed,
            WalletPasswordError::InvalidInput { .. } => PasswordError::DecryptionFailed,
            WalletPasswordError::CreationFailed { .. } => PasswordError::DecryptionFailed,
        }
    }
}

// ==============================================================================
// Enhanced Session State (merged from session_state.rs)
// ==============================================================================

use std::collections::HashMap;

/// Wallet session state (separate from individual account sessions)
#[derive(Debug, Clone)]
pub struct WalletSessionState {
    /// Whether wallet is currently unlocked
    pub is_unlocked: bool,

    /// When wallet was unlocked
    pub unlocked_at: Option<Instant>,

    /// Last wallet activity
    pub last_activity: Instant,

    /// Wallet session timeout duration
    pub timeout_duration: Duration,

    /// Whether wallet auto-lock is enabled
    pub auto_lock_enabled: bool,

    /// Cached wallet configuration (only while unlocked)
    pub cached_wallet_config: Option<crate::security::WalletConfig>,

    /// Cached master password (only while unlocked and remember_session is enabled)
    pub cached_master_password: Option<secrecy::SecretString>,

    /// Whether remember wallet session is enabled
    pub remember_session: bool,
}

impl Default for WalletSessionState {
    fn default() -> Self {
        Self {
            is_unlocked: false,
            unlocked_at: None,
            last_activity: Instant::now(),
            timeout_duration: Duration::from_secs(30 * 60), // 30 minutes for wallet
            auto_lock_enabled: true,
            cached_wallet_config: None,
            cached_master_password: None,
            remember_session: false,
        }
    }
}

impl WalletSessionState {
    /// Unlock the wallet session
    pub fn unlock(
        &mut self,
        wallet_config: crate::security::WalletConfig,
        master_password: secrecy::SecretString,
        remember: bool,
    ) {
        self.is_unlocked = true;
        self.unlocked_at = Some(Instant::now());
        self.last_activity = Instant::now();
        self.remember_session = remember;

        if remember {
            // Cache wallet config and master password for session
            self.cached_wallet_config = Some(wallet_config);
            self.cached_master_password = Some(master_password);
        }
    }

    /// Lock the wallet session
    pub fn lock(&mut self) {
        self.is_unlocked = false;
        self.unlocked_at = None;
        self.cached_wallet_config = None;
        self.cached_master_password = None; // Clear cached password for security
        self.remember_session = false;
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        if self.is_unlocked {
            self.last_activity = Instant::now();
        }
    }

    /// Check if wallet session has timed out
    pub fn is_timed_out(&self) -> bool {
        if !self.is_unlocked || !self.auto_lock_enabled {
            return false;
        }

        self.last_activity.elapsed() > self.timeout_duration
    }

    /// Get time remaining before timeout
    pub fn time_until_timeout(&self) -> Option<Duration> {
        if !self.is_unlocked || !self.auto_lock_enabled {
            return None;
        }

        let elapsed = self.last_activity.elapsed();
        if elapsed >= self.timeout_duration {
            None
        } else {
            Some(self.timeout_duration - elapsed)
        }
    }
}

/// Enhanced session management that separates wallet and account sessions
#[derive(Debug, Clone, Default)]
pub struct EnhancedSessionState {
    /// Wallet-level session (controls access to wallet metadata)
    pub wallet_session: WalletSessionState,

    /// Account-specific sessions (controls access to each account's keys)
    pub account_sessions: HashMap<String, AccountSessionState>,

    /// Global session settings
    pub global_settings: GlobalSessionSettings,

    /// Session coordinator for managing cross-session behavior
    pub coordinator: SessionCoordinator,
}

/// Individual account session state
#[derive(Debug, Clone)]
pub struct AccountSessionState {
    /// Account ID this session belongs to
    pub account_id: String,

    /// Whether this account's session is unlocked
    pub is_unlocked: bool,

    /// When this account was unlocked
    pub unlocked_at: Option<Instant>,

    /// Last activity for this account
    pub last_activity: Instant,

    /// Session timeout duration for this account
    pub timeout_duration: Duration,

    /// Whether auto-lock is enabled for this account
    pub auto_lock_enabled: bool,

    /// Cached password for this account (cleared on lock)
    pub cached_password: Option<SecretString>,

    /// Account-specific session settings
    pub account_settings: AccountSessionSettings,

    /// Last operation performed with this account
    pub last_operation: Option<AccountOperation>,

    /// Account session priority (for resource management)
    pub priority: SessionPriority,
}

/// Account-specific session settings
#[derive(Debug, Clone)]
pub struct AccountSessionSettings {
    /// Custom timeout for this account (overrides default)
    pub custom_timeout_minutes: Option<u32>,

    /// Whether this account requires password for transactions
    pub require_password_for_transactions: bool,

    /// Whether this account should auto-lock when wallet locks
    pub lock_with_wallet: bool,

    /// Whether this account can extend wallet session
    pub can_extend_wallet_session: bool,

    /// Session priority level
    pub priority_level: SessionPriority,
}

impl Default for AccountSessionSettings {
    fn default() -> Self {
        Self {
            custom_timeout_minutes: None,
            require_password_for_transactions: true,
            lock_with_wallet: true,
            can_extend_wallet_session: true,
            priority_level: SessionPriority::Normal,
        }
    }
}

/// Types of operations that can be performed with an account
#[derive(Debug, Clone, PartialEq)]
pub enum AccountOperation {
    /// View account balance
    ViewBalance,
    /// Sign a transaction
    SignTransaction,
    /// Export private key/seed
    ExportSecrets,
    /// View transaction history
    ViewHistory,
    /// Add new account
    CreateAccount,
    /// Remove account
    DeleteAccount,
}

/// Session priority levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionPriority {
    /// Low priority (longer timeout, first to be cleaned up)
    Low,
    /// Normal priority
    Normal,
    /// High priority (shorter timeout, longer retention)
    High,
    /// Critical priority (active trading/signing account)
    Critical,
}

/// Global session settings that affect all sessions
#[derive(Debug, Clone)]
pub struct GlobalSessionSettings {
    /// Master auto-lock setting (affects all sessions)
    pub master_auto_lock_enabled: bool,

    /// Global session timeout multiplier
    pub global_timeout_multiplier: f32,

    /// Maximum number of concurrent account sessions
    pub max_concurrent_account_sessions: usize,

    /// Whether to cascade lock (lock all accounts when wallet locks)
    pub cascade_lock_enabled: bool,

    /// Whether to cascade unlock (unlock accounts when wallet unlocks)
    pub cascade_unlock_enabled: bool,

    /// Idle detection settings
    pub idle_detection: IdleDetectionSettings,
}

impl Default for GlobalSessionSettings {
    fn default() -> Self {
        Self {
            master_auto_lock_enabled: true,
            global_timeout_multiplier: 1.0,
            max_concurrent_account_sessions: 10,
            cascade_lock_enabled: true,
            cascade_unlock_enabled: false, // Require explicit account unlocks
            idle_detection: IdleDetectionSettings::default(),
        }
    }
}

/// Idle detection settings for automatic session management
#[derive(Debug, Clone)]
pub struct IdleDetectionSettings {
    /// Whether idle detection is enabled
    pub enabled: bool,

    /// Idle timeout duration
    pub idle_timeout: Duration,

    /// Whether to lock on system idle
    pub lock_on_system_idle: bool,

    /// Whether to lock on application unfocus
    pub lock_on_unfocus: bool,

    /// Grace period before locking after idle detection
    pub grace_period: Duration,
}

impl Default for IdleDetectionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_timeout: Duration::from_secs(30 * 60), // 30 minutes
            lock_on_system_idle: true,
            lock_on_unfocus: false,
            grace_period: Duration::from_secs(60), // 1 minute grace period
        }
    }
}

/// Session coordinator for managing interactions between sessions
#[derive(Debug, Clone)]
pub struct SessionCoordinator {
    /// Last global activity timestamp
    pub last_global_activity: Instant,

    /// Active session tracking
    pub active_sessions: Vec<String>,

    /// Session cleanup queue
    pub cleanup_queue: Vec<(String, Instant)>,

    /// Session statistics
    pub statistics: SessionStatistics,
}

impl Default for SessionCoordinator {
    fn default() -> Self {
        Self {
            last_global_activity: Instant::now(),
            active_sessions: Vec::new(),
            cleanup_queue: Vec::new(),
            statistics: SessionStatistics::default(),
        }
    }
}

/// Session usage statistics
#[derive(Debug, Default, Clone)]
pub struct SessionStatistics {
    /// Total session unlock count
    pub total_unlocks: u64,

    /// Total session lock count
    pub total_locks: u64,

    /// Total session timeout count
    pub total_timeouts: u64,

    /// Average session duration
    pub average_session_duration: Option<Duration>,

    /// Most active account ID
    pub most_active_account: Option<String>,

    /// Session security events
    pub security_events: u32,
}

/// Session validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum SessionValidationError {
    /// Wallet is locked
    WalletLocked,
    /// Wallet session has expired
    WalletSessionExpired,
    /// Account not found
    AccountNotFound,
    /// Account is locked
    AccountLocked,
    /// Account session has expired
    AccountSessionExpired,
    /// Too many active sessions
    TooManyActiveSessions,
    /// System is locked down
    SystemLocked,
    /// Global idle timeout exceeded
    GlobalIdleTimeout,
}

impl std::fmt::Display for SessionValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionValidationError::WalletLocked => write!(f, "Wallet is locked"),
            SessionValidationError::WalletSessionExpired => write!(f, "Wallet session has expired"),
            SessionValidationError::AccountNotFound => write!(f, "Account not found"),
            SessionValidationError::AccountLocked => write!(f, "Account is locked"),
            SessionValidationError::AccountSessionExpired => write!(f, "Account session has expired"),
            SessionValidationError::TooManyActiveSessions => write!(f, "Too many active sessions"),
            SessionValidationError::SystemLocked => write!(f, "System is locked"),
            SessionValidationError::GlobalIdleTimeout => write!(f, "Global idle timeout exceeded"),
        }
    }
}

impl std::error::Error for SessionValidationError {}

/// Session health status
#[derive(Debug, Clone, PartialEq)]
pub enum SessionHealth {
    /// Session is active and healthy
    Active,
    /// Session is locked
    Locked,
    /// Session has expired
    Expired,
    /// Session is in degraded state
    Degraded,
}

/// Comprehensive session health report
#[derive(Debug, Clone)]
pub struct SessionHealthReport {
    /// Wallet session status
    pub wallet_status: SessionHealth,
    /// Individual account session statuses
    pub account_statuses: HashMap<String, SessionHealth>,
    /// Global session health
    pub global_health: SessionHealth,
    /// Number of active sessions
    pub active_sessions_count: usize,
    /// Total number of sessions
    pub total_sessions_count: usize,
    /// Last global activity timestamp
    pub last_global_activity: Instant,
}

/// Summary of individual account session
#[derive(Debug, Clone)]
pub struct AccountSessionSummary {
    /// Account ID
    pub account_id: String,
    /// Whether account is unlocked
    pub is_unlocked: bool,
    /// Whether account session is timed out
    pub is_timed_out: bool,
    /// How long the session has been active
    pub session_duration: Option<Duration>,
    /// Time remaining before timeout
    pub time_until_timeout: Option<Duration>,
    /// Last operation performed
    pub last_operation: Option<AccountOperation>,
    /// Session priority level
    pub priority: SessionPriority,
    /// Whether auto-lock is enabled
    pub auto_lock_enabled: bool,
}

/// Session statistics for a specific account
#[derive(Debug, Clone)]
pub struct AccountSessionStatistics {
    /// Account ID
    pub account_id: String,
    /// Total duration of current session
    pub total_duration: Duration,
    /// Last operation performed
    pub last_operation: Option<AccountOperation>,
    /// Number of operations performed in this session
    pub operations_count: u32,
    /// Current priority level
    pub priority_level: SessionPriority,
    /// Whether the account is currently active
    pub is_currently_active: bool,
}

/// Reasons for locking a session
#[derive(Debug, Clone, PartialEq)]
pub enum LockReason {
    /// User manually locked
    Manual,
    /// Session timed out
    Timeout,
    /// Wallet was locked (cascade)
    WalletLocked,
    /// System idle detected
    SystemIdle,
    /// Application unfocused
    AppUnfocused,
    /// Emergency lock triggered
    Emergency,
    /// Security policy violation
    SecurityViolation,
}

/// Summary of current session state
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub wallet_unlocked: bool,
    pub total_account_sessions: usize,
    pub unlocked_account_sessions: usize,
    pub is_any_session_active: bool,
    pub global_idle_time: Duration,
}

impl EnhancedSessionState {
    /// Create a new enhanced session state
    pub fn new() -> Self {
        Self::default()
    }

    /// Unlock an account session
    pub fn unlock_account_session(
        &mut self,
        account_id: String,
        password: SecretString,
        remember: bool,
        settings: Option<AccountSessionSettings>,
    ) {
        let mut session = AccountSessionState {
            account_id: account_id.clone(),
            is_unlocked: true,
            unlocked_at: Some(Instant::now()),
            last_activity: Instant::now(),
            timeout_duration: Duration::from_secs(15 * 60), // 15 minutes default
            auto_lock_enabled: true,
            cached_password: if remember { Some(password) } else { None },
            account_settings: settings.unwrap_or_default(),
            last_operation: None,
            priority: SessionPriority::Normal,
        };

        // Apply global timeout multiplier
        session.timeout_duration = Duration::from_secs(
            (session.timeout_duration.as_secs() as f32 * self.global_settings.global_timeout_multiplier) as u64,
        );

        self.account_sessions.insert(account_id.clone(), session);
        self.coordinator.active_sessions.push(account_id.clone());
        self.coordinator.statistics.total_unlocks += 1;

        tracing::info!("🔓 Account session unlocked: {}", account_id);

        // Update global activity
        self.update_global_activity();
    }

    /// Lock an account session
    pub fn lock_account_session(&mut self, account_id: &str, reason: LockReason) {
        if let Some(mut session) = self.account_sessions.remove(account_id) {
            // Clear sensitive data
            session.cached_password = None;
            session.is_unlocked = false;

            tracing::info!("🔒 Account session locked: {} (reason: {:?})", account_id, reason);

            // Update statistics
            self.coordinator.statistics.total_locks += 1;
            if reason == LockReason::Timeout {
                self.coordinator.statistics.total_timeouts += 1;
            }

            // Remove from active sessions
            self.coordinator.active_sessions.retain(|id| id != account_id);
        }
    }

    /// Lock all account sessions
    pub fn lock_all_account_sessions(&mut self, reason: LockReason) {
        let account_ids: Vec<String> = self.account_sessions.keys().cloned().collect();
        for account_id in account_ids {
            self.lock_account_session(&account_id, reason.clone());
        }
        tracing::info!("🔒 All account sessions locked (reason: {:?})", reason);
    }

    /// Check if an account session is unlocked
    pub fn is_account_unlocked(&self, account_id: &str) -> bool {
        self.account_sessions
            .get(account_id)
            .map(|session| session.is_unlocked && !session.is_timed_out())
            .unwrap_or(false)
    }

    /// Check if wallet session is ready for use
    pub fn is_wallet_ready(&self) -> bool {
        self.wallet_session.is_unlocked && !self.wallet_session.is_timed_out()
    }

    /// Update activity for an account session
    pub fn update_account_activity(&mut self, account_id: &str, operation: Option<AccountOperation>) {
        let can_extend = if let Some(session) = self.account_sessions.get_mut(account_id) {
            session.last_activity = Instant::now();
            session.last_operation = operation;
            session.account_settings.can_extend_wallet_session
        } else {
            false
        };

        // Update global activity
        self.update_global_activity();

        // Extend wallet session if allowed
        if can_extend {
            self.wallet_session.update_activity();
        }
    }

    /// Update global activity timestamp
    pub fn update_global_activity(&mut self) {
        self.coordinator.last_global_activity = Instant::now();
    }

    /// Perform session cleanup (remove expired sessions)
    pub fn cleanup_expired_sessions(&mut self) {
        let expired_accounts: Vec<String> = self
            .account_sessions
            .iter()
            .filter(|(_, session)| session.is_timed_out())
            .map(|(id, _)| id.clone())
            .collect();

        for account_id in expired_accounts {
            self.lock_account_session(&account_id, LockReason::Timeout);
        }

        // Clean up wallet session if expired
        if self.wallet_session.is_timed_out() {
            self.wallet_session.lock();
            if self.global_settings.cascade_lock_enabled {
                self.lock_all_account_sessions(LockReason::WalletLocked);
            }
        }
    }

    /// Get session status summary
    pub fn get_session_summary(&self) -> SessionSummary {
        let active_accounts = self.account_sessions.len();
        let unlocked_accounts = self
            .account_sessions
            .values()
            .filter(|session| session.is_unlocked)
            .count();

        SessionSummary {
            wallet_unlocked: self.wallet_session.is_unlocked,
            total_account_sessions: active_accounts,
            unlocked_account_sessions: unlocked_accounts,
            is_any_session_active: self.wallet_session.is_unlocked || unlocked_accounts > 0,
            global_idle_time: self.coordinator.last_global_activity.elapsed(),
        }
    }

    /// Force lock all sessions (emergency/security measure)
    pub fn emergency_lock(&mut self) {
        tracing::warn!("🚨 Emergency lock triggered - locking all sessions");

        self.wallet_session.lock();
        self.lock_all_account_sessions(LockReason::Emergency);

        // Clear all cached data
        self.coordinator.active_sessions.clear();
        self.coordinator.statistics.security_events += 1;
    }

    /// Set global session settings
    pub fn update_global_settings(&mut self, settings: GlobalSessionSettings) {
        self.global_settings = settings;

        // Apply timeout multiplier to existing sessions
        for session in self.account_sessions.values_mut() {
            let base_timeout = Duration::from_secs(15 * 60); // Base 15 minutes
            session.timeout_duration = Duration::from_secs(
                (base_timeout.as_secs() as f32 * self.global_settings.global_timeout_multiplier) as u64,
            );
        }

        // Apply multiplier to wallet session too
        let wallet_base_timeout = Duration::from_secs(30 * 60); // Base 30 minutes
        self.wallet_session.timeout_duration = Duration::from_secs(
            (wallet_base_timeout.as_secs() as f32 * self.global_settings.global_timeout_multiplier) as u64,
        );
    }

    // Wallet session validation functions

    /// Validate wallet session is active and not expired
    pub fn validate_wallet_session(&self) -> Result<(), SessionValidationError> {
        if !self.wallet_session.is_unlocked {
            return Err(SessionValidationError::WalletLocked);
        }

        if self.wallet_session.is_timed_out() {
            return Err(SessionValidationError::WalletSessionExpired);
        }

        Ok(())
    }

    /// Validate account session is active and not expired
    pub fn validate_account_session(&self, account_id: &str) -> Result<(), SessionValidationError> {
        let session = self
            .account_sessions
            .get(account_id)
            .ok_or(SessionValidationError::AccountNotFound)?;

        if !session.is_unlocked {
            return Err(SessionValidationError::AccountLocked);
        }

        if session.is_timed_out() {
            return Err(SessionValidationError::AccountSessionExpired);
        }

        Ok(())
    }

    /// Validate that either wallet or specific account can perform an operation
    pub fn validate_operation_permission(
        &self,
        account_id: &str,
        operation: &AccountOperation,
    ) -> Result<(), SessionValidationError> {
        // Always need wallet session for metadata operations
        self.validate_wallet_session()?;

        // Some operations require account session
        match operation {
            AccountOperation::ViewBalance | AccountOperation::ViewHistory => {
                // Read-only operations can work with just wallet session
                Ok(())
            }
            AccountOperation::SignTransaction | AccountOperation::ExportSecrets => {
                // Sensitive operations require account session
                self.validate_account_session(account_id)
            }
            AccountOperation::CreateAccount | AccountOperation::DeleteAccount => {
                // Account management operations only need wallet session
                Ok(())
            }
        }
    }

    /// Check if global session state allows new operations
    pub fn validate_global_session_health(&self) -> Result<(), SessionValidationError> {
        // Check if we're within session limits
        if self.account_sessions.len() > self.global_settings.max_concurrent_account_sessions {
            return Err(SessionValidationError::TooManyActiveSessions);
        }

        // Check if master auto-lock is forcing a lockdown
        if !self.global_settings.master_auto_lock_enabled && self.coordinator.active_sessions.is_empty() {
            return Err(SessionValidationError::SystemLocked);
        }

        // Check global idle timeout
        if self.global_settings.idle_detection.enabled {
            let idle_elapsed = self.coordinator.last_global_activity.elapsed();
            if idle_elapsed >= self.global_settings.idle_detection.idle_timeout {
                return Err(SessionValidationError::GlobalIdleTimeout);
            }
        }

        Ok(())
    }

    /// Get session health report
    pub fn get_session_health_report(&self) -> SessionHealthReport {
        let wallet_status = if !self.wallet_session.is_unlocked {
            SessionHealth::Locked
        } else if self.wallet_session.is_timed_out() {
            SessionHealth::Expired
        } else {
            SessionHealth::Active
        };

        let mut account_statuses = HashMap::new();
        for (account_id, session) in &self.account_sessions {
            let status = if !session.is_unlocked {
                SessionHealth::Locked
            } else if session.is_timed_out() {
                SessionHealth::Expired
            } else {
                SessionHealth::Active
            };
            account_statuses.insert(account_id.clone(), status);
        }

        let global_health = if self.validate_global_session_health().is_ok() {
            SessionHealth::Active
        } else {
            SessionHealth::Degraded
        };

        SessionHealthReport {
            wallet_status,
            account_statuses,
            global_health,
            active_sessions_count: self.coordinator.active_sessions.len(),
            total_sessions_count: self.account_sessions.len(),
            last_global_activity: self.coordinator.last_global_activity,
        }
    }

    // Advanced account session tracking functions

    /// Get summary for a specific account session
    pub fn get_account_session_summary(&self, account_id: &str) -> Option<AccountSessionSummary> {
        self.account_sessions
            .get(account_id)
            .map(|session| session.get_summary())
    }

    /// Get summaries for all account sessions
    pub fn get_all_account_summaries(&self) -> Vec<AccountSessionSummary> {
        self.account_sessions
            .values()
            .map(|session| session.get_summary())
            .collect()
    }

    /// Get accounts by priority level
    pub fn get_accounts_by_priority(&self, priority: SessionPriority) -> Vec<String> {
        self.account_sessions
            .iter()
            .filter(|(_, session)| session.priority == priority)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get most recently active account
    pub fn get_most_recent_account(&self) -> Option<String> {
        self.account_sessions
            .iter()
            .filter(|(_, session)| session.is_unlocked)
            .max_by_key(|(_, session)| session.last_activity)
            .map(|(id, _)| id.clone())
    }

    /// Get accounts that will expire soon
    pub fn get_expiring_soon_accounts(&self, threshold: Duration) -> Vec<String> {
        self.account_sessions
            .iter()
            .filter_map(|(id, session)| {
                if let Some(time_left) = session.time_until_timeout() {
                    if time_left <= threshold && time_left > Duration::ZERO {
                        Some(id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Update session priority for an account
    pub fn update_account_priority(&mut self, account_id: &str, priority: SessionPriority) {
        if let Some(session) = self.account_sessions.get_mut(account_id) {
            session.priority = priority;
            tracing::info!("🔧 Account {} priority updated to {:?}", account_id, priority);
        }
    }

    /// Set custom timeout for a specific account
    pub fn set_account_custom_timeout(&mut self, account_id: &str, timeout_minutes: u32) {
        if let Some(session) = self.account_sessions.get_mut(account_id) {
            session.account_settings.custom_timeout_minutes = Some(timeout_minutes);
            session.timeout_duration = Duration::from_secs((timeout_minutes as u64) * 60);
            tracing::info!("⏰ Account {} timeout set to {} minutes", account_id, timeout_minutes);
        }
    }

    /// Get session statistics for an account
    pub fn get_account_statistics(&self, account_id: &str) -> Option<AccountSessionStatistics> {
        self.account_sessions.get(account_id).map(|session| {
            AccountSessionStatistics {
                account_id: account_id.to_string(),
                total_duration: session.session_duration().unwrap_or_default(),
                last_operation: session.last_operation.clone(),
                operations_count: 1, // Would track this in a more complete implementation
                priority_level: session.priority,
                is_currently_active: session.is_unlocked && !session.is_timed_out(),
            }
        })
    }

    /// Track operation performed on an account
    pub fn track_account_operation(&mut self, account_id: &str, operation: AccountOperation) {
        if let Some(session) = self.account_sessions.get_mut(account_id) {
            session.last_operation = Some(operation.clone());
            session.last_activity = Instant::now();

            // Update statistics
            if let Some(most_active) = &self.coordinator.statistics.most_active_account {
                if most_active != account_id {
                    // Could implement more sophisticated tracking here
                }
            } else {
                self.coordinator.statistics.most_active_account = Some(account_id.to_string());
            }

            tracing::debug!("📊 Tracked operation {:?} for account {}", operation, account_id);
        }
    }

    /// Auto-manage session priorities based on activity
    pub fn auto_manage_priorities(&mut self) {
        let now = Instant::now();

        for (account_id, session) in self.account_sessions.iter_mut() {
            let activity_age = now.duration_since(session.last_activity);

            // Demote priority for inactive accounts
            if activity_age > Duration::from_secs(30 * 60) {
                // 30 minutes
                if session.priority == SessionPriority::High {
                    session.priority = SessionPriority::Normal;
                    tracing::debug!("📉 Demoted {} priority due to inactivity", account_id);
                } else if session.priority == SessionPriority::Normal && activity_age > Duration::from_secs(60 * 60) {
                    session.priority = SessionPriority::Low;
                    tracing::debug!("📉 Demoted {} priority to Low due to long inactivity", account_id);
                }
            }
        }
    }
}

impl AccountSessionState {
    /// Check if this account session has timed out
    pub fn is_timed_out(&self) -> bool {
        if !self.is_unlocked || !self.auto_lock_enabled {
            return false;
        }
        self.last_activity.elapsed() >= self.timeout_duration
    }

    /// Get time remaining before timeout
    pub fn time_until_timeout(&self) -> Option<Duration> {
        if !self.is_unlocked || !self.auto_lock_enabled {
            return None;
        }

        let elapsed = self.last_activity.elapsed();
        if elapsed >= self.timeout_duration {
            Some(Duration::ZERO)
        } else {
            Some(self.timeout_duration - elapsed)
        }
    }

    /// Update account activity
    pub fn update_activity(&mut self) {
        if self.is_unlocked {
            self.last_activity = Instant::now();
        }
    }

    /// Get session duration (how long it's been unlocked)
    pub fn session_duration(&self) -> Option<Duration> {
        self.unlocked_at.map(|unlocked| unlocked.elapsed())
    }

    /// Check if account needs password for operation
    pub fn requires_password_for_operation(&self, operation: &AccountOperation) -> bool {
        if !self.account_settings.require_password_for_transactions {
            return false;
        }

        matches!(
            operation,
            AccountOperation::SignTransaction | AccountOperation::ExportSecrets
        )
    }

    /// Update account settings
    pub fn update_settings(&mut self, settings: AccountSessionSettings) {
        self.account_settings = settings;

        // Apply custom timeout if specified
        if let Some(custom_timeout_minutes) = self.account_settings.custom_timeout_minutes {
            self.timeout_duration = Duration::from_secs((custom_timeout_minutes as u64) * 60);
        }
    }

    /// Get account session summary
    pub fn get_summary(&self) -> AccountSessionSummary {
        AccountSessionSummary {
            account_id: self.account_id.clone(),
            is_unlocked: self.is_unlocked,
            is_timed_out: self.is_timed_out(),
            session_duration: self.session_duration(),
            time_until_timeout: self.time_until_timeout(),
            last_operation: self.last_operation.clone(),
            priority: self.priority,
            auto_lock_enabled: self.auto_lock_enabled,
        }
    }
}
