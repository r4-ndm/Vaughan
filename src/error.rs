//! Error handling for Vaughan wallet
//!
//! This module provides a comprehensive error handling system with user-friendly messages,
//! security-safe logging, and error recovery mechanisms.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod account;
pub mod logging;
pub mod recovery;
pub mod reporting;

pub use account::{AccountError, AccountResult, ErrorContext as AccountErrorContext};
pub use logging::{init_error_logger, log_error, ErrorLogger};
pub use recovery::{retry_network_operation, retry_operation, ErrorRecoveryManager};
pub use reporting::{init_error_reporter, record_error, ErrorReporter, ErrorStats};

/// Main result type for Vaughan operations
pub type Result<T> = std::result::Result<T, VaughanError>;

/// Main error type for Vaughan wallet operations
#[derive(Error, Debug, Clone)]
pub enum VaughanError {
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Contract error: {0}")]
    Contract(#[from] ContractError),

    #[error("GUI error: {0}")]
    Gui(#[from] GuiError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("IO error: {message}")]
    Io { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),

    #[error("Hardware wallet error: {0}")]
    HardwareWallet(#[from] HardwareWalletError),

    #[error("Token error: {0}")]
    Token(#[from] TokenError),

    #[error("Foundry integration error: {0}")]
    Foundry(#[from] FoundryError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

// Manual implementations for non-Clone error types
impl From<std::io::Error> for VaughanError {
    fn from(error: std::io::Error) -> Self {
        VaughanError::Io {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for VaughanError {
    fn from(error: serde_json::Error) -> Self {
        VaughanError::Serialization {
            message: error.to_string(),
        }
    }
}

impl From<url::ParseError> for VaughanError {
    fn from(error: url::ParseError) -> Self {
        VaughanError::Network(NetworkError::RpcError {
            message: format!("URL parsing error: {error}"),
        })
    }
}

impl From<String> for VaughanError {
    fn from(error: String) -> Self {
        VaughanError::ValidationError(error)
    }
}

impl From<std::num::ParseIntError> for VaughanError {
    fn from(error: std::num::ParseIntError) -> Self {
        VaughanError::ValidationError(format!("Parse error: {error}"))
    }
}

impl From<alloy::primitives::AddressError> for VaughanError {
    fn from(error: alloy::primitives::AddressError) -> Self {
        VaughanError::Configuration(ConfigurationError::ParseError {
            message: error.to_string(),
        })
    }
}

impl From<alloy::hex::FromHexError> for VaughanError {
    fn from(error: alloy::hex::FromHexError) -> Self {
        VaughanError::Configuration(ConfigurationError::ParseError {
            message: format!("Hex parsing error: {error}"),
        })
    }
}

impl From<alloy::primitives::ruint::ParseError> for VaughanError {
    fn from(error: alloy::primitives::ruint::ParseError) -> Self {
        VaughanError::Configuration(ConfigurationError::ParseError {
            message: format!("U256 parsing error: {error}"),
        })
    }
}

impl From<anyhow::Error> for VaughanError {
    fn from(error: anyhow::Error) -> Self {
        VaughanError::ValidationError(error.to_string())
    }
}

#[derive(Error, Debug, Clone)]
pub enum WalletError {
    #[error("Account not found: {address}")]
    AccountNotFound { address: String },

    #[error("Invalid private key format")]
    InvalidPrivateKey,

    #[error("Wallet is locked")]
    WalletLocked,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Wallet error: {message}")]
    WalletError { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Network not supported: {network_id}")]
    UnsupportedNetwork { network_id: u64 },

    #[error("RPC connection failed: {url}")]
    RpcConnectionFailed { url: String },

    #[error("RPC error: {message}")]
    RpcError { message: String },

    #[error("Invalid network configuration")]
    InvalidConfiguration,

    #[error("Chain ID mismatch: expected {expected}, got {actual}")]
    ChainIdMismatch { expected: u64, actual: u64 },

    #[error("Network timeout")]
    Timeout,

    #[error("Network error: {message}")]
    NetworkError { message: String },
}

#[derive(Error, Debug, Clone)]
pub enum ContractError {
    #[error("Contract call failed: {reason}")]
    CallFailed { reason: String },

    #[error("Invalid contract address: {address}")]
    InvalidAddress { address: String },

    #[error("ABI parsing error: {error}")]
    AbiError { error: String },
}

#[derive(Error, Debug, Clone)]
pub enum GuiError {
    #[error("Widget creation failed: {widget}")]
    WidgetCreationFailed { widget: String },

    #[error("Layout error: {message}")]
    LayoutError { message: String },

    #[error("Event handling error: {event}")]
    EventHandlingError { event: String },

    #[error("Window error: {message}")]
    WindowError { message: String },
}

#[derive(Error, Debug, Clone)]
pub enum SecurityError {
    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Hardware wallet not connected")]
    HardwareWalletNotConnected,

    #[error("Transaction confirmation required")]
    ConfirmationRequired,

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Keystore error: {message}")]
    KeystoreError { message: String },

    #[error("Invalid seed phrase: {reason}")]
    InvalidSeedPhrase { reason: String },

    #[error("Key derivation error: {message}")]
    KeyDerivationError { message: String },

    #[error("Keychain error: {message}")]
    KeychainError { message: String },

    #[error("Encryption error: {message}")]
    EncryptionError { message: String },

    #[error("Decryption error: {message}")]
    DecryptionError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    #[error("Rate limit exceeded for {operation}. Try again in {wait_time_seconds} seconds.")]
    RateLimitExceeded {
        operation: String,
        wait_time_seconds: u64,
    },

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Authentication token expired")]
    TokenExpired,

    #[error("Integrity check failed: {message}")]
    IntegrityCheckFailed { message: String },
}

#[derive(Error, Debug, Clone)]
pub enum FoundryError {
    #[error("Forge command failed: {command} (exit code: {exit_code}): {stderr}")]
    ForgeCommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("Build failed for contract {contract}: {reason}")]
    BuildFailed { contract: String, reason: String },

    #[error("Deployment failed: {reason}")]
    DeploymentFailed { reason: String },

    #[error("Test execution failed: {reason}")]
    TestExecutionFailed { reason: String },

    #[error("Contract verification failed: {reason}")]
    VerificationFailed { reason: String },

    #[error("Invalid Foundry project structure")]
    InvalidProjectStructure,

    #[error("Missing build artifacts for contract: {contract}")]
    MissingArtifacts { contract: String },

    #[error("Contract compilation failed: {error}")]
    CompilationFailed { error: String },

    #[error("ABI generation failed: {contract}")]
    AbiGenerationFailed { contract: String },

    #[error("Sandbox execution failed: {reason}")]
    SandboxFailed { reason: String },

    #[error("Script execution timeout")]
    ExecutionTimeout,
}

#[derive(Error, Debug, Clone)]
pub enum ConfigurationError {
    #[error("Invalid configuration file: {path}")]
    InvalidFile { path: String },

    #[error("Missing required configuration: {key}")]
    MissingKey { key: String },

    #[error("Configuration parsing error: {message}")]
    ParseError { message: String },

    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },
}

#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardwareWalletError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Invalid address format")]
    InvalidAddress,

    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Operation cancelled by user")]
    UserCancelled,

    #[error("Firmware version not supported: {version}")]
    UnsupportedFirmware { version: String },

    #[error("Device locked or requires PIN")]
    DeviceLocked,

    #[error("Transaction rejected by device")]
    TransactionRejected,

    #[error("Communication error with device")]
    CommunicationError,

    #[error("Transaction signing failed")]
    SigningFailed,

    #[error("Invalid derivation path: {path}")]
    InvalidDerivationPath { path: String },

    #[error("Device not connected")]
    DeviceNotConnected,

    #[error("Operation timeout: {operation}")]
    OperationTimeout { operation: String },

    #[error("User confirmation required on device")]
    ConfirmationRequired,

    #[error("Device app not open: {app}")]
    AppNotOpen { app: String },

    #[error("Insufficient device permissions")]
    InsufficientPermissions,

    #[error("Device memory full")]
    DeviceMemoryFull,

    #[error("Address verification failed")]
    AddressVerificationFailed,

    #[error("Feature not enabled on device")]
    FeatureNotEnabled,

    #[error("Multiple devices detected")]
    MultipleDevicesDetected,

    #[error("Device initialization failed: {reason}")]
    InitializationFailed { reason: String },

    #[error("Blind signing not enabled")]
    BlindSigningDisabled,

    #[error("Contract data too large for device")]
    ContractDataTooLarge,

    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
}

/// Error context with recovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub user_message: String,
    pub recovery_steps: Vec<String>,
    pub support_code: String,
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub timestamp: u64,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error categories for better organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    Network,
    Security,
    UserInput,
    System,
    External,
}

/// Recovery action suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    Retry,
    RetryWithDelay { seconds: u64 },
    SwitchNetwork,
    CheckConnection,
    UpdateConfiguration,
    ContactSupport,
    RestartApplication,
}

impl VaughanError {
    /// Get comprehensive error context with recovery information
    pub fn context(&self) -> ErrorContext {
        let support_code = Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match self {
            VaughanError::Security(SecurityError::InvalidPrivateKey) => ErrorContext {
                user_message: "The private key format is invalid. Please check and try again.".to_string(),
                recovery_steps: vec![
                    "Verify the private key is in the correct format (64 hex characters)".to_string(),
                    "Check for any extra spaces or characters".to_string(),
                    "Try importing from a different source".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::High,
                category: ErrorCategory::Security,
                timestamp,
            },

            VaughanError::Network(NetworkError::RpcConnectionFailed { url }) => ErrorContext {
                user_message: "Unable to connect to the network. Please check your internet connection.".to_string(),
                recovery_steps: vec![
                    "Check your internet connection".to_string(),
                    "Try switching to a different network".to_string(),
                    format!("Verify the RPC endpoint is accessible: {}", url),
                    "Wait a moment and try again".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::Network,
                timestamp,
            },

            VaughanError::Wallet(WalletError::InsufficientBalance) => ErrorContext {
                user_message: "Insufficient balance for this transaction.".to_string(),
                recovery_steps: vec![
                    "Check your account balance".to_string(),
                    "Reduce the transaction amount".to_string(),
                    "Add funds to your account".to_string(),
                    "Account for gas fees in your calculation".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::DeviceNotFound) => ErrorContext {
                user_message: "Hardware wallet not detected. Please connect your device.".to_string(),
                recovery_steps: vec![
                    "Connect your hardware wallet via USB".to_string(),
                    "Unlock your device if it's locked".to_string(),
                    "Install the latest device drivers".to_string(),
                    "Try a different USB port or cable".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::High,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::ConnectionFailed { reason }) => ErrorContext {
                user_message: format!("Failed to connect to hardware wallet: {reason}"),
                recovery_steps: vec![
                    "Check USB connection".to_string(),
                    "Restart your hardware wallet".to_string(),
                    "Close other applications using the device".to_string(),
                    "Update device firmware if available".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::High,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::UserCancelled) => ErrorContext {
                user_message: "Operation was cancelled. You can try again when ready.".to_string(),
                recovery_steps: vec![
                    "Try the operation again".to_string(),
                    "Make sure you confirm the action on your device".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Low,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::DeviceLocked) => ErrorContext {
                user_message: "Hardware wallet is locked. Please unlock your device.".to_string(),
                recovery_steps: vec![
                    "Enter your PIN on the hardware wallet".to_string(),
                    "Make sure the device screen is active".to_string(),
                    "Try disconnecting and reconnecting if needed".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::TransactionRejected) => ErrorContext {
                user_message: "Transaction was rejected on the hardware wallet.".to_string(),
                recovery_steps: vec![
                    "Review transaction details carefully".to_string(),
                    "Confirm the transaction on your device".to_string(),
                    "Check if you have sufficient balance".to_string(),
                    "Try the transaction again".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::ConfirmationRequired) => ErrorContext {
                user_message: "Please confirm the operation on your hardware wallet.".to_string(),
                recovery_steps: vec![
                    "Look at your hardware wallet screen".to_string(),
                    "Press the confirm button on your device".to_string(),
                    "Follow the prompts on the device display".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Low,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::AppNotOpen { app }) => ErrorContext {
                user_message: format!("Please open the {app} app on your hardware wallet."),
                recovery_steps: vec![
                    format!("Navigate to the {} app on your device", app),
                    "Open the app and ensure it's ready".to_string(),
                    "Make sure your device firmware supports this app".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::OperationTimeout { operation }) => ErrorContext {
                user_message: format!("Operation timed out: {operation}. Please try again."),
                recovery_steps: vec![
                    "Ensure your hardware wallet is responsive".to_string(),
                    "Check the USB connection".to_string(),
                    "Try the operation again".to_string(),
                    "Restart the hardware wallet if needed".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::CommunicationError) => ErrorContext {
                user_message: "Communication error with hardware wallet.".to_string(),
                recovery_steps: vec![
                    "Check USB cable and connection".to_string(),
                    "Close other applications using the device".to_string(),
                    "Restart the hardware wallet".to_string(),
                    "Try a different USB port".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::High,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::SigningFailed) => ErrorContext {
                user_message: "Failed to sign transaction with hardware wallet.".to_string(),
                recovery_steps: vec![
                    "Verify transaction details on your device".to_string(),
                    "Ensure sufficient balance for transaction".to_string(),
                    "Check that the device app is up to date".to_string(),
                    "Try the signing operation again".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::High,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::BlindSigningDisabled) => ErrorContext {
                user_message:
                    "Blind signing is disabled. Enable it in your device settings for smart contract interactions."
                        .to_string(),
                recovery_steps: vec![
                    "Open settings on your hardware wallet".to_string(),
                    "Enable 'Blind Signing' or 'Contract Data'".to_string(),
                    "Restart the Ethereum app on your device".to_string(),
                    "Try the transaction again".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                timestamp,
            },

            VaughanError::HardwareWallet(HardwareWalletError::InvalidDerivationPath { path: _ }) => ErrorContext {
                user_message: "Invalid derivation path for hardware wallet.".to_string(),
                recovery_steps: vec![
                    "Check if you're using the correct account".to_string(),
                    "Verify the derivation path format".to_string(),
                    "Try importing the account again".to_string(),
                    "Contact support if the issue persists".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::Configuration(ConfigurationError::InvalidFile { path }) => ErrorContext {
                user_message: "Configuration file is invalid or corrupted.".to_string(),
                recovery_steps: vec![
                    format!("Check the configuration file at: {}", path),
                    "Restore from backup if available".to_string(),
                    "Reset to default configuration".to_string(),
                    "Contact support if the issue persists".to_string(),
                ],
                support_code,
                severity: ErrorSeverity::High,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::Io { message } => ErrorContext {
                user_message: "A file system error occurred.".to_string(),
                recovery_steps: vec![
                    "Check file permissions".to_string(),
                    "Ensure sufficient disk space".to_string(),
                    "Try the operation again".to_string(),
                    format!("Technical details: {}", message),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::System,
                timestamp,
            },

            VaughanError::Serialization { message } => ErrorContext {
                user_message: "Data format error occurred.".to_string(),
                recovery_steps: vec![
                    "Check data format and try again".to_string(),
                    "Reset to default settings if applicable".to_string(),
                    format!("Technical details: {}", message),
                ],
                support_code,
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::System,
                timestamp,
            },

            _ => ErrorContext {
                user_message: self.to_string(),
                recovery_steps: vec!["Try the operation again".to_string()],
                support_code,
                severity: ErrorSeverity::Low,
                category: ErrorCategory::System,
                timestamp,
            },
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        self.context().user_message
    }

    /// Check if this error should be logged (excludes sensitive information)
    pub fn should_log(&self) -> bool {
        !matches!(
            self,
            VaughanError::Security(SecurityError::InvalidPrivateKey)
                | VaughanError::Security(SecurityError::KeystoreError { .. })
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        self.context().severity
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        self.context().category
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            VaughanError::Network(_) => true,
            VaughanError::Configuration(_) => true,
            VaughanError::HardwareWallet(HardwareWalletError::DeviceNotFound) => true,
            VaughanError::HardwareWallet(HardwareWalletError::ConnectionFailed { .. }) => true,
            VaughanError::HardwareWallet(HardwareWalletError::UserCancelled) => true,
            VaughanError::HardwareWallet(HardwareWalletError::DeviceLocked) => true,
            VaughanError::HardwareWallet(HardwareWalletError::TransactionRejected) => true,
            VaughanError::HardwareWallet(HardwareWalletError::CommunicationError) => true,
            VaughanError::HardwareWallet(HardwareWalletError::OperationTimeout { .. }) => true,
            VaughanError::HardwareWallet(HardwareWalletError::ConfirmationRequired) => true,
            VaughanError::HardwareWallet(HardwareWalletError::AppNotOpen { .. }) => true,
            VaughanError::HardwareWallet(HardwareWalletError::DeviceNotConnected) => true,
            VaughanError::HardwareWallet(HardwareWalletError::BlindSigningDisabled) => true,
            VaughanError::Wallet(WalletError::InsufficientBalance) => true,
            VaughanError::Security(SecurityError::ConfirmationRequired) => true,
            _ => false,
        }
    }

    /// Get suggested recovery actions
    pub fn recovery_actions(&self) -> Vec<RecoveryAction> {
        match self {
            VaughanError::Network(NetworkError::RpcConnectionFailed { .. }) => vec![
                RecoveryAction::CheckConnection,
                RecoveryAction::RetryWithDelay { seconds: 5 },
                RecoveryAction::SwitchNetwork,
            ],
            VaughanError::Network(NetworkError::Timeout) => vec![
                RecoveryAction::RetryWithDelay { seconds: 3 },
                RecoveryAction::CheckConnection,
            ],
            VaughanError::HardwareWallet(HardwareWalletError::DeviceNotFound) => vec![
                RecoveryAction::Retry,
                RecoveryAction::CheckConnection,
                RecoveryAction::RestartApplication,
            ],
            VaughanError::HardwareWallet(HardwareWalletError::ConnectionFailed { .. }) => vec![
                RecoveryAction::Retry,
                RecoveryAction::CheckConnection,
                RecoveryAction::RestartApplication,
            ],
            VaughanError::HardwareWallet(HardwareWalletError::UserCancelled) => vec![RecoveryAction::Retry],
            VaughanError::HardwareWallet(HardwareWalletError::DeviceLocked) => {
                vec![RecoveryAction::Retry, RecoveryAction::CheckConnection]
            }
            VaughanError::HardwareWallet(HardwareWalletError::ConfirmationRequired) => vec![RecoveryAction::Retry],
            VaughanError::HardwareWallet(HardwareWalletError::AppNotOpen { .. }) => {
                vec![RecoveryAction::Retry, RecoveryAction::CheckConnection]
            }
            VaughanError::HardwareWallet(HardwareWalletError::OperationTimeout { .. }) => vec![
                RecoveryAction::RetryWithDelay { seconds: 3 },
                RecoveryAction::CheckConnection,
            ],
            VaughanError::HardwareWallet(_) => vec![
                RecoveryAction::Retry,
                RecoveryAction::CheckConnection,
                RecoveryAction::RestartApplication,
            ],
            VaughanError::Configuration(_) => vec![
                RecoveryAction::UpdateConfiguration,
                RecoveryAction::RestartApplication,
                RecoveryAction::ContactSupport,
            ],
            _ => vec![RecoveryAction::Retry],
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum TokenError {
    #[error("Token not found: {0}")]
    NotFound(alloy::primitives::Address),

    #[error("Failed to discover token metadata for: {0}")]
    MetadataDiscoveryFailed(alloy::primitives::Address),

    #[error("Failed to query token balance: {0}")]
    BalanceQueryFailed(String),

    #[error("Invalid token address: {0}")]
    InvalidAddress(String),

    #[error("Token transfer failed: {reason}")]
    TransferFailed { reason: String },

    #[error("Insufficient token balance")]
    InsufficientBalance,

    #[error("Token is blacklisted: {symbol}")]
    Blacklisted { symbol: String },
}
