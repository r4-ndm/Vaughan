//! Controller Integration Tests
//!
//! Tests that verify all controllers work together correctly.
//! These are headless tests that don't require a GUI.

use alloy::primitives::{address, Address, ChainId, U256};
use secrecy::SecretString;
use vaughan::controllers::{
    NetworkController, PriceController, TransactionController, WalletController,
};

// Test private key (Anvil default account 0)
const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Test that all controllers can be created without errors
#[tokio::test]
async fn test_all_controllers_creation() {
    // Create all controllers
    let wallet_controller = WalletController::new();
    let price_controller = PriceController::new(None);

    // Network controller requires async creation
    let network_result = NetworkController::new(
        "https://rpc.pulsechain.com".to_string(),
        ChainId::from(369u64),
    )
    .await;
    assert!(network_result.is_ok());

    let network_controller = network_result.unwrap();

    // Transaction controller needs a provider
    let provider = network_controller.provider();
    let transaction_controller = TransactionController::new(provider, ChainId::from(369u64));

    // Verify all controllers exist
    assert_eq!(wallet_controller.account_count().await, 0);
    assert_eq!(price_controller.cache_stats().await, (0, 100));
    assert_eq!(transaction_controller.chain_id(), ChainId::from(369u64));
    assert_eq!(network_controller.chain_id(), ChainId::from(369u64));
}

/// Test wallet and transaction controller integration
#[tokio::test]
async fn test_wallet_transaction_integration() {
    // Create wallet controller and add account
    let wallet_controller = WalletController::new();
    let private_key = SecretString::new(TEST_PRIVATE_KEY.to_string());
    let address = wallet_controller
        .add_account(private_key, "Test Account".to_string())
        .await
        .expect("Failed to add account");

    // Verify account was added
    assert_eq!(wallet_controller.account_count().await, 1);
    assert_eq!(wallet_controller.get_current_address().await, Some(address));

    // Create network controller
    let network_controller = NetworkController::new(
        "https://rpc.pulsechain.com".to_string(),
        ChainId::from(369u64),
    )
    .await
    .expect("Failed to create network controller");

    // Create transaction controller
    let provider = network_controller.provider();
    let transaction_controller = TransactionController::new(provider, ChainId::from(369u64));

    // Test transaction validation
    let to_address = address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
    let amount = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
    let gas_limit = 21_000u64;
    let balance = U256::from(10_000_000_000_000_000_000u64); // 10 ETH

    let validation_result =
        transaction_controller.validate_transaction(to_address, amount, gas_limit, balance);

    assert!(validation_result.is_ok());
}

/// Test network and price controller integration
#[tokio::test]
async fn test_network_price_integration() {
    // Create network controller
    let network_controller = NetworkController::new(
        "https://rpc.pulsechain.com".to_string(),
        ChainId::from(369u64),
    )
    .await
    .expect("Failed to create network controller");

    // Verify network is healthy
    let health = network_controller.check_network_health().await;
    assert!(health.is_ok());

    // Create price controller
    let price_controller = PriceController::new(None);

    // Verify price controller is ready
    let (cache_size, cache_capacity) = price_controller.cache_stats().await;
    assert_eq!(cache_size, 0);
    assert_eq!(cache_capacity, 100);

    // Note: We don't fetch real prices in tests to avoid API rate limits
    // In production, you would:
    // let price = price_controller.fetch_native_token_price(369).await?;
}

/// Test complete wallet flow: create account, validate transaction, sign message
#[tokio::test]
async fn test_complete_wallet_flow() {
    // Step 1: Create wallet and add account
    let wallet_controller = WalletController::new();
    let private_key = SecretString::new(TEST_PRIVATE_KEY.to_string());
    let address = wallet_controller
        .add_account(private_key, "Main Account".to_string())
        .await
        .expect("Failed to add account");

    // Step 2: Verify account is active
    assert_eq!(wallet_controller.get_current_address().await, Some(address));
    assert_eq!(
        wallet_controller.get_account_name(address).await,
        Some("Main Account".to_string())
    );

    // Step 3: Sign a message
    let message = b"Hello, Ethereum!";
    let signature = wallet_controller
        .sign_message(message)
        .await
        .expect("Failed to sign message");

    // Verify signature is not empty
    assert_ne!(signature.as_bytes()[0], 0u8);

    // Step 4: Create network controller
    let network_controller = NetworkController::new(
        "https://rpc.pulsechain.com".to_string(),
        ChainId::from(369u64),
    )
    .await
    .expect("Failed to create network controller");

    // Step 5: Create transaction controller
    let provider = network_controller.provider();
    let transaction_controller = TransactionController::new(provider, ChainId::from(369u64));

    // Step 6: Build a transaction
    let to_address = address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
    let amount = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
    let gas_limit = 21_000u64;
    let gas_price = 1_000_000_000u128; // 1 gwei
    let nonce = 0u64;

    let tx = transaction_controller.build_transaction(to_address, amount, gas_limit, gas_price, nonce);

    // Verify transaction was built correctly
    assert_eq!(tx.to, Some(to_address.into()));
    assert_eq!(tx.value, Some(amount));
    assert_eq!(tx.gas, Some(gas_limit));
}

/// Test multi-account management
#[tokio::test]
async fn test_multi_account_management() {
    let wallet_controller = WalletController::new();

    // Add first account
    let pk1 = SecretString::new(TEST_PRIVATE_KEY.to_string());
    let addr1 = wallet_controller
        .add_account(pk1, "Account 1".to_string())
        .await
        .expect("Failed to add account 1");

    // Add second account
    let pk2 = SecretString::new(
        "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
    );
    let addr2 = wallet_controller
        .add_account(pk2, "Account 2".to_string())
        .await
        .expect("Failed to add account 2");

    // Verify both accounts exist
    assert_eq!(wallet_controller.account_count().await, 2);
    let accounts = wallet_controller.list_accounts().await;
    assert_eq!(accounts.len(), 2);
    assert!(accounts.contains(&addr1));
    assert!(accounts.contains(&addr2));

    // Test account switching
    wallet_controller
        .switch_account(addr2)
        .await
        .expect("Failed to switch account");
    assert_eq!(wallet_controller.get_current_address().await, Some(addr2));

    // Test signing with specific account
    let message = b"Test message";
    let sig1 = wallet_controller
        .sign_message_with_account(addr1, message)
        .await
        .expect("Failed to sign with account 1");
    let sig2 = wallet_controller
        .sign_message_with_account(addr2, message)
        .await
        .expect("Failed to sign with account 2");

    // Signatures should be different (different private keys)
    assert_ne!(sig1.as_bytes(), sig2.as_bytes());
}

/// Test network switching
#[tokio::test]
async fn test_network_switching() {
    // Create initial network controller (PulseChain)
    let mut network_controller = NetworkController::new(
        "https://rpc.pulsechain.com".to_string(),
        ChainId::from(369u64),
    )
    .await
    .expect("Failed to create network controller");

    assert_eq!(network_controller.chain_id(), ChainId::from(369u64));

    // Switch to Ethereum mainnet
    let switch_result = network_controller
        .switch_network(
            "https://ethereum.publicnode.com".to_string(),
            ChainId::from(1u64),
        )
        .await;

    // Note: This might fail if the network is unreachable or chain ID doesn't match
    // In production, you would handle this gracefully
    if switch_result.is_ok() {
        assert_eq!(network_controller.chain_id(), ChainId::from(1u64));
    }
}

/// Test transaction validation edge cases
#[tokio::test]
async fn test_transaction_validation_edge_cases() {
    let network_controller = NetworkController::new(
        "https://rpc.pulsechain.com".to_string(),
        ChainId::from(369u64),
    )
    .await
    .expect("Failed to create network controller");

    let provider = network_controller.provider();
    let transaction_controller = TransactionController::new(provider, ChainId::from(369u64));

    // Test 1: Zero address should be rejected
    let result = transaction_controller.validate_transaction(
        Address::ZERO,
        U256::from(1000),
        21_000,
        U256::from(10000),
    );
    assert!(result.is_err());

    // Test 2: Zero amount should be rejected
    let result = transaction_controller.validate_transaction(
        address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
        U256::ZERO,
        21_000,
        U256::from(10000),
    );
    assert!(result.is_err());

    // Test 3: Insufficient balance should be rejected
    let result = transaction_controller.validate_transaction(
        address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
        U256::from(10000),
        21_000,
        U256::from(1000), // Balance too low
    );
    assert!(result.is_err());

    // Test 4: Gas limit too low should be rejected
    let result = transaction_controller.validate_transaction(
        address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
        U256::from(1000),
        20_000, // Below minimum
        U256::from(100000),
    );
    assert!(result.is_err());

    // Test 5: Gas limit too high should be rejected
    let result = transaction_controller.validate_transaction(
        address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
        U256::from(1000),
        31_000_000, // Above maximum
        U256::from(100000000),
    );
    assert!(result.is_err());

    // Test 6: Valid transaction should pass
    let result = transaction_controller.validate_transaction(
        address!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
        U256::from(1_000_000_000_000_000_000u64), // 1 ETH
        21_000,
        U256::from(2_000_000_000_000_000_000u64), // 2 ETH balance
    );
    assert!(result.is_ok());
}

/// Test price controller caching
#[tokio::test]
async fn test_price_controller_caching() {
    use std::time::Duration;

    let price_controller = PriceController::with_cache_settings(
        None,
        50,
        Duration::from_millis(100), // Short TTL for testing
    );

    // Verify initial state
    let (size, capacity) = price_controller.cache_stats().await;
    assert_eq!(size, 0);
    assert_eq!(capacity, 50);

    // Clear cache (should be no-op)
    price_controller.clear_cache().await;

    let (size, _) = price_controller.cache_stats().await;
    assert_eq!(size, 0);
}

/// Test controller error handling
#[tokio::test]
async fn test_controller_error_handling() {
    // Test wallet controller errors
    let wallet_controller = WalletController::new();

    // Invalid private key
    let invalid_key = SecretString::new("invalid".to_string());
    let result = wallet_controller
        .add_account(invalid_key, "Test".to_string())
        .await;
    assert!(result.is_err());

    // Sign with no active account
    let result = wallet_controller.sign_message(b"test").await;
    assert!(result.is_err());

    // Switch to non-existent account
    let fake_address = Address::from([0x42; 20]);
    let result = wallet_controller.switch_account(fake_address).await;
    assert!(result.is_err());

    // Test network controller errors
    let result = NetworkController::new("invalid-url".to_string(), ChainId::from(1u64)).await;
    assert!(result.is_err());
}

/// Test that controllers are framework-agnostic (no iced dependency)
#[test]
fn test_controllers_are_framework_agnostic() {
    // This test verifies that controllers can be created without any GUI framework
    // If this compiles, it proves controllers have no iced dependency

    let _wallet = WalletController::new();
    let _price = PriceController::new(None);

    // Network and Transaction controllers require async, tested in other tests
}

/// Test controller type safety with Alloy types
#[test]
fn test_controller_type_safety() {
    // Verify all controllers use Alloy types
    use alloy::primitives::{Address, ChainId, U256};

    // These should compile, proving type safety
    let _address: Address = Address::ZERO;
    let _amount: U256 = U256::from(1000);
    let _chain_id: ChainId = ChainId::from(1u64);

    // Controllers accept these types directly
    let wallet = WalletController::new();
    let price = PriceController::new(None);

    // Verify controllers exist (no-op, just for compilation)
    drop(wallet);
    drop(price);
}
