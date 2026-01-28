//! Controller Layer - Framework-Agnostic Business Logic
//!
//! This module implements the controller pattern inspired by MetaMask's architecture,
//! providing pure business logic with strict Alloy type integration.
//!
//! ## Architecture
//!
//! Controllers are framework-agnostic and use only Alloy types:
//! - `Address` for Ethereum addresses
//! - `U256` for amounts and balances
//! - `ChainId` for network identification
//! - `Provider` for blockchain interaction
//!
//! ## Design Principles
//!
//! 1. **Framework Independence**: No iced or GUI dependencies
//! 2. **Type Safety**: Alloy types only, no string-based validation
//! 3. **Headless Testable**: Can test without GUI
//! 4. **Reusable**: Can be used in CLI, API, or mobile applications
//! 5. **Security First**: MetaMask patterns for critical operations
//!
//! ## Controllers
//!
//! - `TransactionController`: Transaction lifecycle management
//! - `NetworkController`: Network and provider management
//! - `WalletController`: Keyring and account management
//! - `PriceController`: Token price fetching and caching

pub mod transaction;
pub mod network;
pub mod wallet;
pub mod price;

// Re-export controller types
pub use transaction::TransactionController;
pub use network::NetworkController;
pub use wallet::WalletController;
pub use price::PriceController;

// Re-export common types
use alloy::primitives::{Address, U256};
use thiserror::Error;

/// Common result type for all controllers
pub type ControllerResult<T> = Result<T, ControllerError>;

/// Controller error types using Alloy error patterns
///
/// These errors are designed to be:
/// - Descriptive for debugging
/// - Convertible to user-friendly messages
/// - Compatible with Alloy error types
#[derive(Debug, Error)]
pub enum ControllerError {
    /// Invalid Ethereum address
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Insufficient balance for transaction
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        /// Required amount (including gas)
        required: U256,
        /// Available balance
        available: U256,
    },

    /// Network/provider error (from Alloy)
    #[error("Network error: {0}")]
    Network(String),

    /// Transaction validation error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Wallet/keyring error
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Price fetching error
    #[error("Price error: {0}")]
    Price(String),

    /// Generic error for unexpected cases
    #[error("Controller error: {0}")]
    Other(String),
}

// Implement From for common error types
impl From<alloy::transports::TransportError> for ControllerError {
    fn from(err: alloy::transports::TransportError) -> Self {
        ControllerError::Network(err.to_string())
    }
}

impl From<alloy::contract::Error> for ControllerError {
    fn from(err: alloy::contract::Error) -> Self {
        ControllerError::Transaction(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_balance_error() {
        let error = ControllerError::InsufficientBalance {
            required: U256::from(1000),
            available: U256::from(500),
        };

        let message = error.to_string();
        assert!(message.contains("Insufficient balance"));
        assert!(message.contains("1000"));
        assert!(message.contains("500"));
    }

    #[test]
    fn test_invalid_address_error() {
        let error = ControllerError::InvalidAddress("0xinvalid".to_string());
        assert!(error.to_string().contains("Invalid address"));
    }
}
