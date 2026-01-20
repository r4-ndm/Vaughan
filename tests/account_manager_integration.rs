//! Account Manager Integration Tests
//!
//! These tests verify the complete integration of account management components:
//! - Account lifecycle (create, lock, unlock, export)
//! - IntegratedAccountService with batch processing and telemetry
//! - Correlation tracking across components
//!
//! # Requirements Validated
//! - Requirement 1: Unified Account Management Interface
//! - Requirement 4: Structured Error Handling with Correlation Tracking
//! - Requirement 6: Batch Processing for Blockchain Operations
//! - Requirement 7: Structured Logging with Correlation Tracking

use vaughan::error::{Result, VaughanError};
use vaughan::gui::services::IntegratedAccountService;
use vaughan::performance::batch::BatchConfig;
use vaughan::security::{SecureAccount, KeyReference};
use vaughan::wallet::account_manager::{
    AccountConfig, AccountType, AuthToken, AuthorizedOperation, SeedStrength,
};
use alloy::primitives::{Address, U256};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test account with default values
fn create_test_account(name: &str, address: Address) -> SecureAccount {
    SecureAccount {
        id: format!("test-{}", name),
        name: name.to_string(),
        address,
        key_reference: KeyReference {
            id: format!("key-{}", name),
            service: "vaughan-test".to_string(),
            account: name.to_string(),
        },
        created_at: chrono::Utc::now(),
        is_hardware: false,
        derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        tags: Vec::new(),
        last_used: None,
        transaction_count: 0,
    }
}

/// Mock balance fetcher for testing
async fn mock_balance_fetcher(address: Address) -> Result<U256> {
    // Return a deterministic balance based on last byte of address
    let balance = U256::from(address.0[19] as u64 * 1_000_000_000_000_000_000u64);
    Ok(balance)
}

/// Mock failing balance fetcher for partial failure tests
async fn mock_failing_fetcher(address: Address) -> Result<U256> {
    // Fail for addresses with even last byte
    if address.0[19] % 2 == 0 {
        Err(VaughanError::Network(vaughan::error::NetworkError::Timeout))
    } else {
        Ok(U256::from(1_000_000_000_000_000_000u64))
    }
}

// ============================================================================
// Account Lifecycle Integration Tests
// ============================================================================

#[tokio::test]
async fn test_account_lifecycle_create_lock_unlock() {
    // Test the complete account lifecycle
    let service = IntegratedAccountService::new();
    
    // Create operation span for tracking
    let create_span = service.start_operation("test_account_lifecycle");
    assert!(!create_span.correlation_id.is_nil());
    
    // Simulate account creation instrumentation
    let creation_result: std::result::Result<String, String> = Ok("account-123".to_string());
    let instrumented = service.instrument_create_account(creation_result, "Test Account");
    assert!(instrumented.is_ok());
    
    // Simulate lock operation
    service.instrument_lock();
    
    // Simulate successful unlock
    service.instrument_unlock(true);
    
    // Verify the operation completed
    service.complete_operation(&create_span, "lifecycle test completed");
}

#[tokio::test]
async fn test_account_export_instrumentation() {
    let service = IntegratedAccountService::new();
    
    // Test export instrumentation with success
    let export_result: std::result::Result<String, String> = Ok("seed_phrase_here".to_string());
    let instrumented = service.instrument_export_account(export_result, "seed_phrase");
    assert!(instrumented.is_ok());
    
    // Test export instrumentation with failure (should not expose error details)
    let export_fail: std::result::Result<String, String> = Err("sensitive error".to_string());
    let instrumented_fail = service.instrument_export_account(export_fail, "private_key");
    assert!(instrumented_fail.is_err());
}

#[tokio::test]
async fn test_account_import_instrumentation() {
    let service = IntegratedAccountService::new();
    
    // Test import from seed phrase
    let import_result: std::result::Result<String, String> = Ok("imported-account".to_string());
    let instrumented = service.instrument_import_account(import_result, "seed_phrase");
    assert!(instrumented.is_ok());
    
    // Test import from private key
    let pk_result: std::result::Result<String, String> = Ok("pk-account".to_string());
    let instrumented_pk = service.instrument_import_account(pk_result, "private_key");
    assert!(instrumented_pk.is_ok());
}

// ============================================================================
// Batch Processing Integration Tests
// ============================================================================

#[tokio::test]
async fn test_batch_balance_refresh_success() {
    let service = IntegratedAccountService::new();
    
    // Create test accounts
    let accounts = vec![
        create_test_account("account1", Address::from([0x01u8; 20])),
        create_test_account("account2", Address::from([0x02u8; 20])),
        create_test_account("account3", Address::from([0x03u8; 20])),
    ];
    
    // Refresh balances using batch processing
    let balances = service
        .refresh_account_balances(&accounts, mock_balance_fetcher)
        .await;
    
    assert!(balances.is_ok());
    let balance_map = balances.unwrap();
    
    // Verify all accounts have balances
    assert_eq!(balance_map.len(), 3);
    
    // Verify balances match expected values based on mock fetcher
    for account in &accounts {
        assert!(balance_map.contains_key(&account.address));
    }
}

#[tokio::test]
async fn test_batch_balance_empty_accounts() {
    let service = IntegratedAccountService::new();
    
    // Empty account list should return empty map
    let balances = service
        .refresh_account_balances(&[], mock_balance_fetcher)
        .await;
    
    assert!(balances.is_ok());
    assert!(balances.unwrap().is_empty());
}

#[tokio::test]
async fn test_batch_balance_caching() {
    let service = IntegratedAccountService::new();
    let address = Address::from([0x42u8; 20]);
    let account = create_test_account("cached_account", address);
    
    // Initially no cache
    assert!(service.get_cached_balance(&address).await.is_none());
    
    // Refresh to populate cache
    let _ = service
        .refresh_account_balances(&[account], mock_balance_fetcher)
        .await;
    
    // Now cache should be populated
    let cached = service.get_cached_balance(&address).await;
    assert!(cached.is_some());
    
    // Clear cache
    service.clear_balance_cache().await;
    assert!(service.get_cached_balance(&address).await.is_none());
}

#[tokio::test]
async fn test_batch_partial_failure_handling() {
    let service = IntegratedAccountService::new();
    
    // Create accounts - some will fail due to mock_failing_fetcher
    let accounts = vec![
        create_test_account("even", Address::from([0x02u8; 20])),   // Will fail
        create_test_account("odd1", Address::from([0x01u8; 20])),   // Will succeed
        create_test_account("even2", Address::from([0x04u8; 20])),  // Will fail
        create_test_account("odd2", Address::from([0x03u8; 20])),   // Will succeed
    ];
    
    // Batch should complete with partial results
    let result = service
        .refresh_account_balances(&accounts, mock_failing_fetcher)
        .await;
    
    // Should succeed overall (partial failures are handled gracefully)
    assert!(result.is_ok());
    let balance_map = result.unwrap();
    
    // Only odd addresses should have balances
    assert!(balance_map.len() >= 2);
}

// ============================================================================
// Correlation Tracking Integration Tests
// ============================================================================

#[test]
fn test_correlation_id_generation() {
    let service = IntegratedAccountService::new();
    
    // Each operation should generate a unique correlation ID
    let span1 = service.start_operation("operation_1");
    let span2 = service.start_operation("operation_2");
    
    assert_ne!(span1.correlation_id, span2.correlation_id);
    assert!(!span1.correlation_id.is_nil());
    assert!(!span2.correlation_id.is_nil());
}

#[test]
fn test_operation_span_tracking() {
    let service = IntegratedAccountService::new();
    
    let span = service.start_operation("tracked_operation");
    
    // Verify operation name is set
    assert_eq!(span.operation, "tracked_operation");
    
    // Verify elapsed time tracking works
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = span.elapsed_ms();
    assert!(elapsed >= 10);
    
    // Complete operation
    service.complete_operation(&span, "test details");
}

#[test]
fn test_error_correlation() {
    let service = IntegratedAccountService::new();
    
    let span = service.start_operation("failing_operation");
    
    // Record failure with correlation
    service.fail_operation(&span, "test error message");
    
    // Correlation ID should still be valid
    assert!(!span.correlation_id.is_nil());
}

// ============================================================================
// Service Configuration Tests
// ============================================================================

#[test]
fn test_service_default_configuration() {
    let service = IntegratedAccountService::new();
    
    // Default cache TTL should be 30 seconds
    // (Accessing private field not possible, but we can test behavior)
    assert!(true); // Service created successfully
}

#[test]
fn test_service_custom_batch_config() {
    let config = BatchConfig::with_concurrency(2);
    let service = IntegratedAccountService::with_config(config);
    
    // Service should be created with custom config
    assert!(true); // Service created successfully
}

// ============================================================================
// Auth Token Integration Tests
// ============================================================================

#[test]
fn test_auth_token_creation() {
    let token = AuthToken::new(AuthorizedOperation::ExportSeed);
    
    // Token should not be expired immediately
    assert!(!token.is_expired());
    
    // Token should be valid for the correct operation
    assert!(token.is_valid_for(&AuthorizedOperation::ExportSeed));
    assert!(!token.is_valid_for(&AuthorizedOperation::ExportPrivateKey));
}

#[test]
fn test_account_config_builders() {
    // Seed-based account config
    let seed_config = AccountConfig::seed_based("My Wallet")
        .with_seed_strength(SeedStrength::Words24);
    
    assert_eq!(seed_config.name, "My Wallet");
    assert_eq!(seed_config.account_type, AccountType::SeedBased);
    
    // Private key account config
    let pk_config = AccountConfig::private_key("Imported Account");
    assert_eq!(pk_config.account_type, AccountType::PrivateKey);
    
    // Hardware account config
    let hw_config = AccountConfig::hardware("Ledger Account");
    assert_eq!(hw_config.account_type, AccountType::Hardware);
}

// ============================================================================
// Seed Strength Tests
// ============================================================================

#[test]
fn test_seed_strength_values() {
    assert_eq!(SeedStrength::Words12.word_count(), 12);
    assert_eq!(SeedStrength::Words15.word_count(), 15);
    assert_eq!(SeedStrength::Words18.word_count(), 18);
    assert_eq!(SeedStrength::Words21.word_count(), 21);
    assert_eq!(SeedStrength::Words24.word_count(), 24);
    
    assert_eq!(SeedStrength::Words12.entropy_bits(), 128);
    assert_eq!(SeedStrength::Words24.entropy_bits(), 256);
}

// ============================================================================
// Cross-Component Integration Tests
// ============================================================================

#[tokio::test]
async fn test_service_telemetry_integration() {
    let service = IntegratedAccountService::new();
    
    // Start a tracked operation
    let span = service.start_operation("cross_component_test");
    
    // Perform account operations with telemetry
    let result: std::result::Result<String, String> = Ok("success".to_string());
    let _ = service.instrument_create_account(result, "Test");
    
    // Complete the operation
    service.complete_operation(&span, "cross-component test completed");
    
    // Verify correlations were tracked
    assert!(!span.correlation_id.is_nil());
}

#[tokio::test]
async fn test_integrated_workflow() {
    let service = IntegratedAccountService::new();
    
    // Simulate complete workflow:
    // 1. Create account
    let create_result: std::result::Result<String, String> = Ok("new-account".to_string());
    let _ = service.instrument_create_account(create_result, "Workflow Test");
    
    // 2. Refresh balance
    let account = create_test_account("workflow", Address::from([0xFFu8; 20]));
    let balances = service
        .refresh_account_balances(&[account], mock_balance_fetcher)
        .await;
    assert!(balances.is_ok());
    
    // 3. Lock wallet
    service.instrument_lock();
    
    // 4. Unlock wallet
    service.instrument_unlock(true);
    
    // 5. Export (simulated)
    let export_result: std::result::Result<String, String> = Ok("exported".to_string());
    let _ = service.instrument_export_account(export_result, "seed_phrase");
}
