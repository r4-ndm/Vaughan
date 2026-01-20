/// Enhanced transaction error handling system
/// Provides detailed, user-friendly error messages with actionable suggestions
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionError {
    // Wallet/Account related errors
    WalletNotInitialized,
    WalletLocked,
    NoAccountSelected,
    AccountUnlockFailed(String),

    // Input validation errors
    EmptyRecipient,
    EmptyAmount,
    InvalidAddress { address: String, reason: String },
    InvalidAmount { amount: String, reason: String },
    NegativeAmount,

    // Gas related errors
    InvalidGasPrice { gas_price: String, reason: String },
    InvalidGasLimit { gas_limit: String, reason: String },
    GasPriceTooLow { current: f64, minimum: f64 },
    GasPriceTooHigh { current: f64, maximum: f64 },
    GasLimitTooLow { current: u64, minimum: u64 },
    GasLimitTooHigh { current: u64, maximum: u64 },

    // Balance related errors
    InsufficientBalance { have: f64, need: f64, token: String },
    InsufficientGasFunds { have: f64, gas_cost: f64 },
    BalanceCheckFailed(String),

    // Conversion errors
    AmountConversionFailed(String),

    // Transaction signing errors
    SigningFailed(String),

    // Network related errors
    NetworkConnectionFailed(String),
    RpcCallFailed(String),
    TransactionBroadcastFailed(String),
    NonceError(String),

    // Token related errors
    TokenNotSupported(String),
    TokenContractError(String),

    // Generic errors for fallback
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct TransactionErrorInfo {
    pub title: String,
    pub message: String,
    pub suggestion: String,
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub retry_recommended: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Low,      // Minor issues, user can easily fix
    Medium,   // Moderate issues, may need some action
    High,     // Serious issues, requires attention
    Critical, // Critical issues, transaction cannot proceed
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    UserInput, // User needs to correct input
    Account,   // Account/wallet related issues
    Balance,   // Balance/funds related issues
    Network,   // Network connectivity issues
    Gas,       // Gas price/limit issues
    System,    // System/internal errors
}

impl TransactionError {
    pub fn to_error_info(&self) -> TransactionErrorInfo {
        match self {
            TransactionError::WalletNotInitialized => TransactionErrorInfo {
                title: "Wallet Not Ready".to_string(),
                message: "Your wallet is still initializing. Please wait a moment.".to_string(),
                suggestion: "Wait for wallet initialization to complete, then try again.".to_string(),
                severity: ErrorSeverity::High,
                category: ErrorCategory::System,
                retry_recommended: true,
            },

            TransactionError::WalletLocked => TransactionErrorInfo {
                title: "Wallet Locked".to_string(),
                message: "Your wallet is locked and cannot send transactions.".to_string(),
                suggestion: "Select and unlock an account from the account dropdown menu.".to_string(),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Account,
                retry_recommended: false,
            },

            TransactionError::NoAccountSelected => TransactionErrorInfo {
                title: "No Account Selected".to_string(),
                message: "You need to select an account before sending transactions.".to_string(),
                suggestion: "Choose an account from the dropdown menu and unlock it with your password.".to_string(),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Account,
                retry_recommended: false,
            },

            TransactionError::EmptyRecipient => TransactionErrorInfo {
                title: "Missing Recipient Address".to_string(),
                message: "You must specify who to send the transaction to.".to_string(),
                suggestion: "Enter a valid Ethereum address in the 'To' field (0x followed by 40 characters).".to_string(),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                retry_recommended: false,
            },

            TransactionError::EmptyAmount => TransactionErrorInfo {
                title: "Missing Amount".to_string(),
                message: "You must specify how much to send.".to_string(),
                suggestion: "Enter the amount you want to send in the 'Amount' field.".to_string(),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                retry_recommended: false,
            },

            TransactionError::InvalidAddress { address, reason } => TransactionErrorInfo {
                title: "Invalid Recipient Address".to_string(),
                message: format!("The address '{address}' is not valid."),
                suggestion: format!("Enter a valid Ethereum address (0x followed by 40 hex characters). Error: {reason}"),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                retry_recommended: false,
            },

            TransactionError::InvalidAmount { amount, reason } => TransactionErrorInfo {
                title: "Invalid Amount".to_string(),
                message: format!("The amount '{amount}' is not valid."),
                suggestion: format!("Enter a valid number (e.g., 0.1, 1.5, 100). Error: {reason}"),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                retry_recommended: false,
            },

            TransactionError::NegativeAmount => TransactionErrorInfo {
                title: "Invalid Amount".to_string(),
                message: "The amount must be greater than zero.".to_string(),
                suggestion: "Enter a positive number greater than 0.".to_string(),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::UserInput,
                retry_recommended: false,
            },

            TransactionError::InsufficientBalance { have, need, token } => TransactionErrorInfo {
                title: "Insufficient Balance".to_string(),
                message: format!("You don't have enough {token} to complete this transaction."),
                suggestion: format!("You have {have:.6} {token} but need {need:.6} {token} (including gas fees). Add more funds or reduce the amount."),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Balance,
                retry_recommended: false,
            },

            TransactionError::InsufficientGasFunds { have, gas_cost } => TransactionErrorInfo {
                title: "Insufficient Gas Funds".to_string(),
                message: "You don't have enough funds to pay for gas fees.".to_string(),
                suggestion: format!("You have {have:.6} ETH but need {gas_cost:.6} ETH for gas. Add more ETH or reduce gas price/limit."),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Gas,
                retry_recommended: false,
            },

            TransactionError::InvalidGasPrice { gas_price, reason } => TransactionErrorInfo {
                title: "Invalid Gas Price".to_string(),
                message: format!("The gas price '{gas_price}' is not valid."),
                suggestion: format!("Enter a valid gas price in Gwei (e.g., 20, 50). Error: {reason}"),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::Gas,
                retry_recommended: false,
            },

            TransactionError::GasPriceTooLow { current, minimum } => TransactionErrorInfo {
                title: "Gas Price Too Low".to_string(),
                message: format!("Your gas price of {current:.1} Gwei is too low."),
                suggestion: format!("Increase gas price to at least {minimum:.1} Gwei for faster transaction processing."),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::Gas,
                retry_recommended: true,
            },

            TransactionError::GasPriceTooHigh { current, maximum } => TransactionErrorInfo {
                title: "Gas Price Very High".to_string(),
                message: format!("Your gas price of {current:.1} Gwei is unusually high."),
                suggestion: format!("Consider reducing gas price to around {maximum:.1} Gwei to save on fees."),
                severity: ErrorSeverity::Low,
                category: ErrorCategory::Gas,
                retry_recommended: true,
            },

            TransactionError::SigningFailed(reason) => TransactionErrorInfo {
                title: "Transaction Signing Failed".to_string(),
                message: "Failed to sign the transaction with your private key.".to_string(),
                suggestion: format!("This might be due to wallet issues. Try unlocking your account again. Error: {reason}"),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Account,
                retry_recommended: true,
            },

            TransactionError::NetworkConnectionFailed(reason) => TransactionErrorInfo {
                title: "Network Connection Failed".to_string(),
                message: "Cannot connect to the blockchain network.".to_string(),
                suggestion: format!("Check your internet connection and try again. Error: {reason}"),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Network,
                retry_recommended: true,
            },

            TransactionError::RpcCallFailed(reason) => TransactionErrorInfo {
                title: "Network Request Failed".to_string(),
                message: "Failed to communicate with the blockchain network.".to_string(),
                suggestion: format!("The network might be experiencing issues. Try again in a few minutes. Error: {reason}"),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Network,
                retry_recommended: true,
            },

            TransactionError::TransactionBroadcastFailed(reason) => TransactionErrorInfo {
                title: "Transaction Broadcast Failed".to_string(),
                message: "The transaction was signed but failed to broadcast to the network.".to_string(),
                suggestion: format!("Try sending the transaction again. Error: {reason}"),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Network,
                retry_recommended: true,
            },

            TransactionError::AccountUnlockFailed(reason) => {
                // Check if this is a missing keystore issue
                if reason.contains("encrypted seed data is missing") {
                    TransactionErrorInfo {
                        title: "Account Data Missing".to_string(),
                        message: "Your account's encrypted seed data is not found in the keychain.".to_string(),
                        suggestion: "This usually happens after system updates or keychain resets. You'll need to re-import your wallet using your seed phrase or private key. Go to 'Import Wallet' to restore your account.".to_string(),
                        severity: ErrorSeverity::Critical,
                        category: ErrorCategory::Account,
                        retry_recommended: false,
                    }
                } else {
                    TransactionErrorInfo {
                        title: "Account Unlock Failed".to_string(),
                        message: "Failed to unlock your account for transaction signing.".to_string(),
                        suggestion: format!("Check your password and try again. Error: {reason}"),
                        severity: ErrorSeverity::High,
                        category: ErrorCategory::Account,
                        retry_recommended: true,
                    }
                }
            },

            TransactionError::BalanceCheckFailed(reason) => TransactionErrorInfo {
                title: "Balance Check Failed".to_string(),
                message: "Unable to verify your current balance.".to_string(),
                suggestion: format!("Network connection issues. Try again in a moment. Error: {reason}"),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::Network,
                retry_recommended: true,
            },

            _ => TransactionErrorInfo {
                title: "Transaction Failed".to_string(),
                message: format!("An unexpected error occurred: {self:?}"),
                suggestion: "Please try again or contact support if the issue persists.".to_string(),
                severity: ErrorSeverity::Medium,
                category: ErrorCategory::System,
                retry_recommended: true,
            },
        }
    }

    /// Parse error string and classify it into appropriate TransactionError
    pub fn from_error_string(error: &str) -> TransactionError {
        let error_lower = error.to_lowercase();

        // Wallet/Account errors
        if error_lower.contains("wallet not initialized") {
            return TransactionError::WalletNotInitialized;
        }
        if error_lower.contains("wallet is locked") || error_lower.contains("no current account") {
            return TransactionError::WalletLocked;
        }

        // Balance errors - check for insufficient funds in any context (including RPC errors)
        if error_lower.contains("insufficient balance") || error_lower.contains("insufficient funds") {
            // Try to extract balance amounts
            if let Some(have_need) = extract_balance_amounts(error) {
                return TransactionError::InsufficientBalance {
                    have: have_need.0,
                    need: have_need.1,
                    token: "tPLS".to_string(), // Use tPLS for testnet
                };
            }
            return TransactionError::InsufficientBalance {
                have: 0.0,
                need: 0.0,
                token: "tPLS".to_string(),
            };
        }

        // Address validation errors
        if error_lower.contains("invalid") && error_lower.contains("address") {
            return TransactionError::InvalidAddress {
                address: extract_address_from_error(error).unwrap_or_default(),
                reason: error.to_string(),
            };
        }

        // Amount validation errors
        if error_lower.contains("invalid amount") || error_lower.contains("parse") && error_lower.contains("float") {
            return TransactionError::InvalidAmount {
                amount: extract_amount_from_error(error).unwrap_or_default(),
                reason: error.to_string(),
            };
        }

        if error_lower.contains("amount") && error_lower.contains("empty") {
            return TransactionError::EmptyAmount;
        }

        if error_lower.contains("amount must be greater than") || error_lower.contains("negative") {
            return TransactionError::NegativeAmount;
        }

        // Gas errors
        if error_lower.contains("gas price") && error_lower.contains("invalid") {
            return TransactionError::InvalidGasPrice {
                gas_price: extract_gas_price_from_error(error).unwrap_or_default(),
                reason: error.to_string(),
            };
        }

        // Signing errors - check for specific keystore issues
        if error_lower.contains("signing failed") || error_lower.contains("sign") && error_lower.contains("fail") {
            // Check for missing keystore files
            if error_lower.contains("no such file or directory") || error_lower.contains("failed to read key file") {
                return TransactionError::AccountUnlockFailed("Your account's encrypted seed data is missing from the keychain. You may need to re-import your wallet.".to_string());
            }
            return TransactionError::SigningFailed(error.to_string());
        }

        // Network errors
        if error_lower.contains("connection") || error_lower.contains("network") || error_lower.contains("rpc") {
            if error_lower.contains("connection") {
                return TransactionError::NetworkConnectionFailed(error.to_string());
            } else {
                return TransactionError::RpcCallFailed(error.to_string());
            }
        }

        // Fallback to unknown error
        TransactionError::Unknown(error.to_string())
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = self.to_error_info();
        write!(f, "{}: {}", info.title, info.message)
    }
}

// Helper functions to extract specific information from error strings
fn extract_balance_amounts(error: &str) -> Option<(f64, f64)> {
    // Try to extract "Have: X, Need: Y" pattern
    if let (Some(have_start), Some(need_start)) = (error.find("Have: "), error.find("Need: ")) {
        let have_part = &error[have_start + 6..];
        let need_part = &error[need_start + 6..];

        let have = have_part.split_whitespace().next()?.parse().ok()?;
        let need = need_part.split_whitespace().next()?.parse().ok()?;

        Some((have, need))
    } else {
        None
    }
}

fn extract_address_from_error(error: &str) -> Option<String> {
    // Look for patterns like 'address' or "address"
    if let Some(start) = error.find("address '") {
        let start = start + 9;
        if let Some(end) = error[start..].find("'") {
            return Some(error[start..start + end].to_string());
        }
    }

    if let Some(start) = error.find("address \"") {
        let start = start + 9;
        if let Some(end) = error[start..].find("\"") {
            return Some(error[start..start + end].to_string());
        }
    }

    None
}

fn extract_amount_from_error(error: &str) -> Option<String> {
    // Look for patterns like 'amount' or "amount"
    if let Some(start) = error.find("amount '") {
        let start = start + 8;
        if let Some(end) = error[start..].find("'") {
            return Some(error[start..start + end].to_string());
        }
    }

    None
}

fn extract_gas_price_from_error(error: &str) -> Option<String> {
    // Look for patterns like 'gas_price' or "gas_price"
    if let Some(start) = error.find("gas price '") {
        let start = start + 11;
        if let Some(end) = error[start..].find("'") {
            return Some(error[start..start + end].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let error = TransactionError::from_error_string("Wallet not initialized");
        assert_eq!(error, TransactionError::WalletNotInitialized);

        let error = TransactionError::from_error_string("Invalid address '0xInvalid': invalid format");
        match error {
            TransactionError::InvalidAddress { address, .. } => {
                assert_eq!(address, "0xInvalid");
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_error_info_generation() {
        let error = TransactionError::WalletLocked;
        let info = error.to_error_info();

        assert_eq!(info.title, "Wallet Locked");
        assert_eq!(info.severity, ErrorSeverity::High);
        assert_eq!(info.category, ErrorCategory::Account);
        assert!(!info.retry_recommended);
    }
}
