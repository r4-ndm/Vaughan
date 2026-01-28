//! Transaction Controller - Pure Alloy-based transaction logic
//!
//! Follows MetaMask's TransactionController pattern for security-critical operations.
//!
//! ## Design Principles
//!
//! 1. **Type Safety**: Uses Alloy primitives (Address, U256, ChainId) exclusively
//! 2. **MetaMask Patterns**: Implements validation rules from MetaMask
//! 3. **Headless Testable**: No GUI dependencies, can test without UI
//! 4. **Security First**: Zero address checks, balance validation, gas limits
//!
//! ## MetaMask Inspiration
//!
//! This controller implements patterns from MetaMask's TransactionController:
//! - Zero address rejection (cannot send to 0x0)
//! - Gas limit bounds (21k minimum, 30M maximum)
//! - Balance validation (amount + gas must not exceed balance)
//! - Nonce management
//! - Transaction status monitoring

use super::{ControllerError, ControllerResult};
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, ChainId, TxHash, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};
use alloy::signers::Signer;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gas limit constants (from Ethereum standards)
pub const MIN_GAS_LIMIT: u64 = 21_000; // Minimum for simple transfer
pub const MAX_GAS_LIMIT: u64 = 30_000_000; // Block gas limit safety

/// Transaction controller - pure business logic, no UI coupling
///
/// Follows MetaMask's TransactionController pattern:
/// - Validates transaction parameters
/// - Estimates gas
/// - Signs transactions
/// - Submits to network
/// - Monitors transaction status
///
/// ## Example
///
/// ```rust,no_run
/// use vaughan::controllers::TransactionController;
/// use alloy::primitives::{Address, U256, ChainId};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let controller = TransactionController::new(provider, ChainId::from(1));
///
/// // Validate with Alloy types
/// controller.validate_transaction(
///     Address::from([0x12; 20]),
///     U256::from(1000),
///     21_000,
///     U256::from(10000),
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct TransactionController<P> {
    provider: Arc<RwLock<P>>,
    chain_id: ChainId,
}

impl<P> TransactionController<P>
where
    P: Provider + Clone + 'static,
{
    /// Create new transaction controller
    ///
    /// # Arguments
    ///
    /// * `provider` - Alloy provider for blockchain interaction
    /// * `chain_id` - Network chain ID
    pub fn new(provider: Arc<RwLock<P>>, chain_id: ChainId) -> Self {
        Self { provider, chain_id }
    }

    /// Get current chain ID
    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    /// Validate transaction parameters (Alloy types only)
    ///
    /// Implements MetaMask validation rules:
    /// - Zero address check (cannot send to 0x0)
    /// - Amount validation (positive, non-zero)
    /// - Gas limit validation (21k-30M)
    /// - Balance check (amount + gas cost)
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient address (Alloy Address)
    /// * `amount` - Transfer amount in wei (Alloy U256)
    /// * `gas_limit` - Gas limit for transaction
    /// * `sender_balance` - Current sender balance (Alloy U256)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation passes
    /// * `Err(ControllerError)` with specific validation failure
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vaughan::controllers::TransactionController;
    /// # use alloy::primitives::{Address, U256};
    /// # fn example(controller: &TransactionController<impl alloy::providers::Provider>) {
    /// let result = controller.validate_transaction(
    ///     Address::from([0x12; 20]),
    ///     U256::from(1_000_000_000_000_000_000u64), // 1 ETH
    ///     21_000,
    ///     U256::from(2_000_000_000_000_000_000u64), // 2 ETH balance
    /// );
    /// assert!(result.is_ok());
    /// # }
    /// ```
    pub fn validate_transaction(
        &self,
        to: Address,
        amount: U256,
        gas_limit: u64,
        sender_balance: U256,
    ) -> ControllerResult<()> {
        // Zero address check (MetaMask pattern)
        // Sending to 0x0 is almost always a mistake and can result in lost funds
        if to == Address::ZERO {
            return Err(ControllerError::InvalidAddress(
                "Cannot send to zero address (0x0)".to_string(),
            ));
        }

        // Amount validation
        // Zero-value transactions are technically valid but often indicate errors
        if amount == U256::ZERO {
            return Err(ControllerError::Transaction(
                "Amount must be greater than zero".to_string(),
            ));
        }

        // Gas limit validation (Ethereum standards)
        // Minimum: 21,000 gas for simple transfer
        // Maximum: 30,000,000 gas (typical block gas limit)
        if gas_limit < MIN_GAS_LIMIT {
            return Err(ControllerError::Transaction(format!(
                "Gas limit too low: minimum {} gas required for transfer",
                MIN_GAS_LIMIT
            )));
        }

        if gas_limit > MAX_GAS_LIMIT {
            return Err(ControllerError::Transaction(format!(
                "Gas limit too high: maximum {} gas (block limit)",
                MAX_GAS_LIMIT
            )));
        }

        // Balance check (amount + estimated gas cost)
        // Use conservative gas price estimate (1 gwei) for validation
        // Actual gas price will be determined at submission time
        let gas_price_estimate = U256::from(1_000_000_000u64); // 1 gwei
        let gas_cost = U256::from(gas_limit)
            .checked_mul(gas_price_estimate)
            .ok_or_else(|| {
                ControllerError::Transaction("Gas cost calculation overflow".to_string())
            })?;

        let total_cost = amount.checked_add(gas_cost).ok_or_else(|| {
            ControllerError::Transaction("Total cost calculation overflow".to_string())
        })?;

        if total_cost > sender_balance {
            return Err(ControllerError::InsufficientBalance {
                required: total_cost,
                available: sender_balance,
            });
        }

        Ok(())
    }

    /// Estimate gas for transaction (Alloy provider)
    ///
    /// Uses Alloy's gas estimation to determine required gas.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient address
    /// * `amount` - Transfer amount in wei
    /// * `from` - Sender address
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Estimated gas limit
    /// * `Err(ControllerError)` - Network or estimation error
    pub async fn estimate_gas(
        &self,
        to: Address,
        amount: U256,
        from: Address,
    ) -> ControllerResult<u64> {
        let provider = self.provider.read().await;

        let tx = TransactionRequest::default()
            .to(to)
            .value(amount)
            .from(from);

        let gas_estimate = provider
            .estimate_gas(tx)
            .await
            .map_err(|e| ControllerError::Network(format!("Gas estimation failed: {}", e)))?;

        Ok(gas_estimate)
    }

    /// Build transaction request (Alloy types)
    ///
    /// Creates a properly formatted TransactionRequest with all parameters.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient address
    /// * `amount` - Transfer amount in wei
    /// * `gas_limit` - Gas limit
    /// * `gas_price` - Gas price in wei
    /// * `nonce` - Transaction nonce
    ///
    /// # Returns
    ///
    /// Alloy TransactionRequest ready for signing
    pub fn build_transaction(
        &self,
        to: Address,
        amount: U256,
        gas_limit: u64,
        gas_price: u128,
        nonce: u64,
    ) -> TransactionRequest {
        TransactionRequest::default()
            .with_to(to)
            .with_value(amount)
            .with_gas_limit(gas_limit)
            .with_gas_price(gas_price)
            .with_nonce(nonce)
            .with_chain_id(self.chain_id.into())
    }

    /// Get transaction receipt (Alloy provider)
    ///
    /// Retrieves transaction receipt for status monitoring.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - Transaction hash
    ///
    /// # Returns
    ///
    /// * `Ok(Some(receipt))` - Transaction receipt if found
    /// * `Ok(None)` - Transaction not yet mined
    /// * `Err(ControllerError)` - Network error
    pub async fn get_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> ControllerResult<Option<TransactionReceipt>> {
        let provider = self.provider.read().await;

        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| ControllerError::Network(format!("Failed to get receipt: {}", e)))?;

        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    // Mock provider for testing
    #[derive(Clone)]
    struct MockProvider;

    impl Provider for MockProvider {
        fn root(&self) -> &alloy::providers::RootProvider {
            unimplemented!("Mock provider for unit tests")
        }
    }

    fn create_test_controller() -> TransactionController<MockProvider> {
        TransactionController::new(
            Arc::new(RwLock::new(MockProvider)),
            ChainId::from(1u64),
        )
    }

    #[test]
    fn test_validate_zero_address_rejected() {
        let controller = create_test_controller();

        let result = controller.validate_transaction(
            Address::ZERO,
            U256::from(1000),
            21_000,
            U256::from(10000),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::InvalidAddress(msg) => {
                assert!(msg.contains("zero address"));
            }
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_validate_zero_amount_rejected() {
        let controller = create_test_controller();

        let result = controller.validate_transaction(
            address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
            U256::ZERO,
            21_000,
            U256::from(10000),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::Transaction(msg) => {
                assert!(msg.contains("greater than zero"));
            }
            _ => panic!("Expected Transaction error"),
        }
    }

    #[test]
    fn test_validate_insufficient_balance() {
        let controller = create_test_controller();

        let result = controller.validate_transaction(
            address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
            U256::from(10000),
            21_000,
            U256::from(1000), // Balance too low
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::InsufficientBalance { required, available } => {
                assert!(required > available);
            }
            _ => panic!("Expected InsufficientBalance error"),
        }
    }

    #[test]
    fn test_validate_gas_limit_too_low() {
        let controller = create_test_controller();

        let result = controller.validate_transaction(
            address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
            U256::from(1000),
            20_000, // Below minimum
            U256::from(100000),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::Transaction(msg) => {
                assert!(msg.contains("too low"));
            }
            _ => panic!("Expected Transaction error"),
        }
    }

    #[test]
    fn test_validate_gas_limit_too_high() {
        let controller = create_test_controller();

        let result = controller.validate_transaction(
            address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
            U256::from(1000),
            31_000_000, // Above maximum
            U256::from(100000000),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ControllerError::Transaction(msg) => {
                assert!(msg.contains("too high"));
            }
            _ => panic!("Expected Transaction error"),
        }
    }

    #[test]
    fn test_validate_valid_transaction() {
        let controller = create_test_controller();

        let result = controller.validate_transaction(
            address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
            U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            21_000,
            U256::from(2_000_000_000_000_000_000u64), // 2 ETH balance
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_build_transaction() {
        let controller = create_test_controller();

        let to = address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
        let amount = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let gas_limit = 21_000u64;
        let gas_price = 1_000_000_000u128; // 1 gwei
        let nonce = 0u64;

        let tx = controller.build_transaction(to, amount, gas_limit, gas_price, nonce);

        assert_eq!(tx.to, Some(to.into()));
        assert_eq!(tx.value, Some(amount));
        assert_eq!(tx.gas, Some(gas_limit));
        assert_eq!(tx.gas_price, Some(gas_price));
        assert_eq!(tx.nonce, Some(nonce));
        assert_eq!(tx.chain_id, Some(1));
    }
}
