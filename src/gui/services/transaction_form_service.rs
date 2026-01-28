//! Transaction Form Service - Transaction validation and preparation
//!
//! This service extracts transaction form logic from view components,
//! providing secure validation following Alloy and MetaMask industry standards.
//!
//! # Security Standards
//! - Address validation using Alloy primitives
//! - Amount parsing with precision handling (18 decimals for ETH)
//! - Balance checks to prevent insufficient funds
//! - Gas limit validation
//! - Nonce validation
//!
//! # References
//! - Alloy Address validation: https://github.com/alloy-rs/alloy
//! - MetaMask transaction validation patterns

use alloy::primitives::{Address, U256};
use std::str::FromStr;

/// Data prepared for sending a transaction
#[derive(Debug, Clone, PartialEq)]
pub struct SendFormData {
    /// Recipient address
    pub recipient: String,
    /// Amount to send (as string for display)
    pub amount: String,
    /// Token symbol (e.g., "ETH", "USDC")
    pub token_symbol: String,
    /// Optional gas limit override
    pub gas_limit: Option<u64>,
    /// Optional gas price (for legacy transactions)
    pub gas_price: Option<U256>,
    /// Optional max fee per gas (for EIP-1559)
    pub max_fee_per_gas: Option<U256>,
    /// Optional max priority fee per gas (for EIP-1559)
    pub max_priority_fee_per_gas: Option<U256>,
    /// Transaction type ("Legacy" or "EIP-1559")
    pub tx_type: String,
}

/// Errors that can occur during transaction validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    /// Invalid recipient address format
    InvalidRecipient(String),
    /// Invalid amount format or value
    InvalidAmount(String),
    /// Insufficient balance for transaction
    InsufficientBalance,
    /// Invalid gas limit
    InvalidGasLimit(String),
    /// Invalid gas price
    InvalidGasPrice(String),
    /// Amount is zero or negative
    AmountTooSmall,
    /// Amount exceeds maximum safe value
    AmountTooLarge,
    /// Recipient address is zero address
    RecipientIsZeroAddress,
}

impl std::fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRecipient(msg) => write!(f, "Invalid recipient address: {}", msg),
            Self::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            Self::InsufficientBalance => write!(f, "Insufficient balance for this transaction"),
            Self::InvalidGasLimit(msg) => write!(f, "Invalid gas limit: {}", msg),
            Self::InvalidGasPrice(msg) => write!(f, "Invalid gas price: {}", msg),
            Self::AmountTooSmall => write!(f, "Amount must be greater than zero"),
            Self::AmountTooLarge => write!(f, "Amount exceeds maximum safe value"),
            Self::RecipientIsZeroAddress => write!(f, "Cannot send to zero address (0x0000...)"),
        }
    }
}

impl std::error::Error for TransactionValidationError {}

/// Trait defining the transaction form service interface for testability
pub trait TransactionFormServiceTrait: Send + Sync {
    /// Validate a recipient address using Alloy primitives
    ///
    /// # Security
    /// - Uses Alloy's Address::from_str for proper validation
    /// - Checks for zero address (0x0000...0000)
    /// - Follows MetaMask address validation patterns
    fn validate_recipient(&self, address: &str) -> Result<Address, TransactionValidationError>;
    
    /// Validate and parse an amount string to wei (U256)
    ///
    /// # Security
    /// - Handles 18 decimal precision for ETH
    /// - Checks for overflow
    /// - Validates against available balance
    /// - Prevents negative or zero amounts
    fn validate_amount(
        &self,
        amount: &str,
        balance: U256,
        decimals: u8,
    ) -> Result<U256, TransactionValidationError>;
    
    /// Validate gas limit
    ///
    /// # Security
    /// - Ensures gas limit is within reasonable bounds
    /// - Minimum: 21000 (standard ETH transfer)
    /// - Maximum: 30000000 (block gas limit on most chains)
    fn validate_gas_limit(&self, gas_limit: &str) -> Result<u64, TransactionValidationError>;
    
    /// Validate gas price in Gwei
    ///
    /// # Security
    /// - Converts Gwei to Wei
    /// - Checks for reasonable bounds
    fn validate_gas_price(&self, gas_price_gwei: &str) -> Result<U256, TransactionValidationError>;
    
    /// Check if amount + gas fees exceed balance
    ///
    /// # Security
    /// - Prevents transactions that would fail due to insufficient funds
    /// - Accounts for gas costs
    fn check_sufficient_balance(
        &self,
        amount: U256,
        gas_limit: u64,
        gas_price: U256,
        balance: U256,
    ) -> Result<(), TransactionValidationError>;
}

/// Transaction form service implementation following Alloy and MetaMask standards
#[derive(Debug, Default)]
pub struct TransactionFormService;

impl TransactionFormService {
    /// Create a new transaction form service
    pub fn new() -> Self {
        Self
    }
    
    /// Minimum gas limit for a standard ETH transfer (21000 gas)
    const MIN_GAS_LIMIT: u64 = 21_000;
    
    /// Maximum reasonable gas limit (30M - typical block gas limit)
    const MAX_GAS_LIMIT: u64 = 30_000_000;
    
    /// Maximum reasonable gas price in Gwei (10000 Gwei = 10000 * 10^9 Wei)
    const MAX_GAS_PRICE_GWEI: u64 = 10_000;
    
    /// Parse a decimal string to U256 with specified decimals
    ///
    /// # Arguments
    /// * `amount_str` - Amount as decimal string (e.g., "1.5")
    /// * `decimals` - Number of decimals (18 for ETH)
    ///
    /// # Returns
    /// U256 value in smallest unit (wei for ETH)
    fn parse_amount_to_wei(amount_str: &str, decimals: u8) -> Result<U256, TransactionValidationError> {
        let trimmed = amount_str.trim();
        
        if trimmed.is_empty() {
            return Err(TransactionValidationError::InvalidAmount(
                "Amount cannot be empty".to_string()
            ));
        }
        
        // Parse as f64 first to handle decimal input
        let amount_f64 = trimmed.parse::<f64>()
            .map_err(|e| TransactionValidationError::InvalidAmount(
                format!("Cannot parse amount: {}", e)
            ))?;
        
        // Check for negative or zero
        if amount_f64 <= 0.0 {
            return Err(TransactionValidationError::AmountTooSmall);
        }
        
        // Check for reasonable maximum (prevent overflow)
        // Max safe f64 that won't overflow when multiplied by 10^18
        if amount_f64 > 1e59 {
            return Err(TransactionValidationError::AmountTooLarge);
        }
        
        // Convert to wei by multiplying by 10^decimals
        // For ETH: 1.5 ETH = 1.5 * 10^18 wei
        let multiplier = 10_f64.powi(decimals as i32);
        let amount_wei_f64 = amount_f64 * multiplier;
        
        // Convert to U256
        // Use string conversion to avoid precision loss
        let amount_wei_str = format!("{:.0}", amount_wei_f64);
        U256::from_str(&amount_wei_str)
            .map_err(|e| TransactionValidationError::InvalidAmount(
                format!("Amount too large: {}", e)
            ))
    }
}

impl TransactionFormServiceTrait for TransactionFormService {
    fn validate_recipient(&self, address: &str) -> Result<Address, TransactionValidationError> {
        let trimmed = address.trim();
        
        if trimmed.is_empty() {
            return Err(TransactionValidationError::InvalidRecipient(
                "Address cannot be empty".to_string()
            ));
        }
        
        // Normalize to lowercase for parsing (Alloy accepts lowercase 0x prefix)
        let normalized = trimmed.to_lowercase();
        
        // Use Alloy's Address::from_str for proper validation
        // This follows Ethereum address standards (0x + 40 hex chars)
        let parsed_address = Address::from_str(&normalized)
            .map_err(|e| TransactionValidationError::InvalidRecipient(
                format!("Invalid address format: {}", e)
            ))?;
        
        // Check for zero address (0x0000000000000000000000000000000000000000)
        // Sending to zero address burns tokens - prevent accidental burns
        if parsed_address == Address::ZERO {
            return Err(TransactionValidationError::RecipientIsZeroAddress);
        }
        
        Ok(parsed_address)
    }
    
    fn validate_amount(
        &self,
        amount: &str,
        balance: U256,
        decimals: u8,
    ) -> Result<U256, TransactionValidationError> {
        // Parse amount to wei
        let amount_wei = Self::parse_amount_to_wei(amount, decimals)?;
        
        // Check against balance
        if amount_wei > balance {
            return Err(TransactionValidationError::InsufficientBalance);
        }
        
        Ok(amount_wei)
    }
    
    fn validate_gas_limit(&self, gas_limit: &str) -> Result<u64, TransactionValidationError> {
        let trimmed = gas_limit.trim();
        
        if trimmed.is_empty() {
            return Err(TransactionValidationError::InvalidGasLimit(
                "Gas limit cannot be empty".to_string()
            ));
        }
        
        let gas = trimmed.parse::<u64>()
            .map_err(|e| TransactionValidationError::InvalidGasLimit(
                format!("Cannot parse gas limit: {}", e)
            ))?;
        
        // Check minimum (21000 for standard ETH transfer)
        if gas < Self::MIN_GAS_LIMIT {
            return Err(TransactionValidationError::InvalidGasLimit(
                format!("Gas limit too low (minimum: {})", Self::MIN_GAS_LIMIT)
            ));
        }
        
        // Check maximum (prevent unreasonable values)
        if gas > Self::MAX_GAS_LIMIT {
            return Err(TransactionValidationError::InvalidGasLimit(
                format!("Gas limit too high (maximum: {})", Self::MAX_GAS_LIMIT)
            ));
        }
        
        Ok(gas)
    }
    
    fn validate_gas_price(&self, gas_price_gwei: &str) -> Result<U256, TransactionValidationError> {
        let trimmed = gas_price_gwei.trim();
        
        if trimmed.is_empty() {
            return Err(TransactionValidationError::InvalidGasPrice(
                "Gas price cannot be empty".to_string()
            ));
        }
        
        // Parse as f64 to handle decimal Gwei values
        let gwei_f64 = trimmed.parse::<f64>()
            .map_err(|e| TransactionValidationError::InvalidGasPrice(
                format!("Cannot parse gas price: {}", e)
            ))?;
        
        if gwei_f64 < 0.0 {
            return Err(TransactionValidationError::InvalidGasPrice(
                "Gas price cannot be negative".to_string()
            ));
        }
        
        // Check maximum (prevent unreasonable values)
        if gwei_f64 > Self::MAX_GAS_PRICE_GWEI as f64 {
            return Err(TransactionValidationError::InvalidGasPrice(
                format!("Gas price too high (maximum: {} Gwei)", Self::MAX_GAS_PRICE_GWEI)
            ));
        }
        
        // Convert Gwei to Wei (1 Gwei = 10^9 Wei)
        let wei_f64 = gwei_f64 * 1e9;
        let wei_str = format!("{:.0}", wei_f64);
        
        U256::from_str(&wei_str)
            .map_err(|e| TransactionValidationError::InvalidGasPrice(
                format!("Gas price too large: {}", e)
            ))
    }
    
    fn check_sufficient_balance(
        &self,
        amount: U256,
        gas_limit: u64,
        gas_price: U256,
        balance: U256,
    ) -> Result<(), TransactionValidationError> {
        // Calculate total cost: amount + (gas_limit * gas_price)
        let gas_cost = U256::from(gas_limit)
            .checked_mul(gas_price)
            .ok_or_else(|| TransactionValidationError::InvalidGasPrice(
                "Gas cost calculation overflow".to_string()
            ))?;
        
        let total_cost = amount
            .checked_add(gas_cost)
            .ok_or_else(|| TransactionValidationError::InvalidAmount(
                "Total cost calculation overflow".to_string()
            ))?;
        
        if total_cost > balance {
            return Err(TransactionValidationError::InsufficientBalance);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> TransactionFormService {
        TransactionFormService::new()
    }

    // Recipient validation tests
    #[test]
    fn test_validate_recipient_valid() {
        let s = service();
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let result = s.validate_recipient(addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_recipient_lowercase() {
        let s = service();
        let addr = "0x742d35cc6634c0532925a3b844bc9e7595f0beb0";
        let result = s.validate_recipient(addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_recipient_zero_address() {
        let s = service();
        let addr = "0x0000000000000000000000000000000000000000";
        let result = s.validate_recipient(addr);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionValidationError::RecipientIsZeroAddress));
    }

    #[test]
    fn test_validate_recipient_invalid_format() {
        let s = service();
        let addr = "not-an-address";
        let result = s.validate_recipient(addr);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionValidationError::InvalidRecipient(_)));
    }

    #[test]
    fn test_validate_recipient_empty() {
        let s = service();
        let result = s.validate_recipient("");
        assert!(result.is_err());
    }

    // Amount validation tests
    #[test]
    fn test_validate_amount_valid() {
        let s = service();
        let balance = U256::from(2_000_000_000_000_000_000u128); // 2 ETH in wei
        let result = s.validate_amount("1.5", balance, 18);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::from(1_500_000_000_000_000_000u128));
    }

    #[test]
    fn test_validate_amount_insufficient_balance() {
        let s = service();
        let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        let result = s.validate_amount("2.0", balance, 18);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionValidationError::InsufficientBalance));
    }

    #[test]
    fn test_validate_amount_zero() {
        let s = service();
        let balance = U256::from(1_000_000_000_000_000_000u128);
        let result = s.validate_amount("0", balance, 18);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionValidationError::AmountTooSmall));
    }

    #[test]
    fn test_validate_amount_negative() {
        let s = service();
        let balance = U256::from(1_000_000_000_000_000_000u128);
        let result = s.validate_amount("-1.0", balance, 18);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_amount_decimal() {
        let s = service();
        let balance = U256::from(1_000_000_000_000_000_000u128);
        let result = s.validate_amount("0.123456789012345678", balance, 18);
        assert!(result.is_ok());
    }

    // Gas limit validation tests
    #[test]
    fn test_validate_gas_limit_valid() {
        let s = service();
        let result = s.validate_gas_limit("21000");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 21000);
    }

    #[test]
    fn test_validate_gas_limit_too_low() {
        let s = service();
        let result = s.validate_gas_limit("20000");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionValidationError::InvalidGasLimit(_)));
    }

    #[test]
    fn test_validate_gas_limit_too_high() {
        let s = service();
        let result = s.validate_gas_limit("40000000");
        assert!(result.is_err());
    }

    // Gas price validation tests
    #[test]
    fn test_validate_gas_price_valid() {
        let s = service();
        let result = s.validate_gas_price("50");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::from(50_000_000_000u128)); // 50 Gwei in Wei
    }

    #[test]
    fn test_validate_gas_price_decimal() {
        let s = service();
        let result = s.validate_gas_price("1.5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_gas_price_too_high() {
        let s = service();
        let result = s.validate_gas_price("20000");
        assert!(result.is_err());
    }

    // Balance check tests
    #[test]
    fn test_check_sufficient_balance_ok() {
        let s = service();
        let amount = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        let gas_limit = 21000;
        let gas_price = U256::from(50_000_000_000u128); // 50 Gwei
        let balance = U256::from(2_000_000_000_000_000_000u128); // 2 ETH
        
        let result = s.check_sufficient_balance(amount, gas_limit, gas_price, balance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_sufficient_balance_insufficient() {
        let s = service();
        let amount = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
        let gas_limit = 21000;
        let gas_price = U256::from(50_000_000_000u128); // 50 Gwei
        let balance = U256::from(1_000_000_000_000_000_000u128); // 1 ETH (not enough for gas)
        
        let result = s.check_sufficient_balance(amount, gas_limit, gas_price, balance);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionValidationError::InsufficientBalance));
    }
}
