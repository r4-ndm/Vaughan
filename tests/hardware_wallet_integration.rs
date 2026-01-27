//! Integration tests for hardware wallet functionality
//!
//! These tests validate the complete hardware wallet integration flow
//! using mock devices to simulate real hardware wallet behavior.

use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;
use std::time::Duration;
use vaughan::wallet::{HardwareManager, Vaughan, WalletConfig};

/// Test helper to create a standard test transaction
fn create_test_transaction() -> TransactionRequest {
    let mut tx = TransactionRequest::default();
    tx.to = Some(
        Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .unwrap()
            .into(),
    );
    tx.value = Some(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH
    tx.gas_price = Some(20_000_000_000u128); // 20 gwei
    tx.gas = Some(21_000u64);
    tx
}

#[tokio::test]
async fn test_full_hardware_wallet_workflow() {
    // Initialize hardware wallet manager
    let mut hw_manager = HardwareManager::new().unwrap();

    // Step 1: Detect devices
    let detected_devices = hw_manager.detect_wallets().await.unwrap();
    assert_eq!(detected_devices.len(), 2); // Ledger + Trezor mock devices

    // Step 2: Get device info
    for (index, device) in detected_devices.iter().enumerate() {
        let device_info = hw_manager.get_device_info(index).await.unwrap();
        assert_eq!(device_info.device_type, device.device_type);
        assert!(!device_info.device_type.is_empty());
    }

    // Step 3: Get addresses from devices
    let ledger_addresses = hw_manager.get_addresses(0, "m/44'/60'/0'/0", 5).await.unwrap();
    assert_eq!(ledger_addresses.len(), 5);

    let trezor_addresses = hw_manager.get_addresses(1, "m/44'/60'/0'/0", 3).await.unwrap();
    assert_eq!(trezor_addresses.len(), 3);

    // Step 4: Test address verification
    let first_address = format!("{:?}", ledger_addresses[0]);
    let verification_feedback = hw_manager
        .verify_address_with_feedback(0, &first_address, "m/44'/60'/0'/0/0")
        .await
        .unwrap();

    assert!(!verification_feedback.user_message.is_empty());
    assert!(!verification_feedback.next_steps.is_empty());

    // Step 5: Test transaction signing
    let tx = create_test_transaction();
    let signature = hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await.unwrap();
    assert!(signature.as_bytes().len() > 0);
}

#[tokio::test]
async fn test_hardware_wallet_security_validation() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    let tx = create_test_transaction();

    // Test successful audit
    let audit_feedback = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    assert!(!audit_feedback.user_message.is_empty());
    assert!(!audit_feedback.recommendations.is_empty());

    // Test high-value transaction audit
    let mut high_value_tx = create_test_transaction();
    high_value_tx.value = Some(U256::from_str_radix("100000000000000000000", 10).unwrap()); // 100 ETH

    let high_value_audit = hw_manager
        .audit_transaction_with_feedback(&high_value_tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    assert!(matches!(
        high_value_audit.risk_level,
        vaughan::wallet::hardware::RiskLevel::High | vaughan::wallet::hardware::RiskLevel::Critical
    ));
}

#[tokio::test]
async fn test_hardware_wallet_error_recovery() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Test device recovery
    let recovery_feedback = hw_manager.recover_device_with_feedback("mock-ledger-id").await.unwrap();
    assert!(!recovery_feedback.user_message.is_empty());
    assert!(!recovery_feedback.next_steps.is_empty());

    // Test invalid device recovery
    let invalid_recovery = hw_manager.recover_device_with_feedback("invalid-device").await.unwrap();
    assert!(!invalid_recovery.recovered);
}

#[tokio::test]
async fn test_concurrent_hardware_operations() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    let tx = create_test_transaction();

    // Test concurrent signing on different devices
    let (ledger_result, trezor_result) = tokio::join!(
        hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0"),
        hw_manager.sign_transaction(1, &tx, "m/44'/60'/0'/0/0")
    );

    assert!(ledger_result.is_ok());
    assert!(trezor_result.is_ok());

    // Test concurrent address derivation
    let (ledger_addresses, trezor_addresses) = tokio::join!(
        hw_manager.get_addresses(0, "m/44'/60'/0'/0", 3),
        hw_manager.get_addresses(1, "m/44'/60'/0'/0", 3)
    );

    assert!(ledger_addresses.is_ok());
    assert!(trezor_addresses.is_ok());
    assert_eq!(ledger_addresses.unwrap().len(), 3);
    assert_eq!(trezor_addresses.unwrap().len(), 3);
}

#[tokio::test]
async fn test_hardware_wallet_device_health_monitoring() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Test device health check
    let ledger_health = hw_manager.check_hardware_device_health(0).await.unwrap();
    assert!(ledger_health == "Operational");

    let trezor_health = hw_manager.check_hardware_device_health(1).await.unwrap();
    assert!(trezor_health == "Operational");

    // Test invalid device health check
    let invalid_health = hw_manager.check_hardware_device_health(999).await.unwrap();
    assert!(invalid_health.is_empty() || invalid_health != "Operational");

    // Test hardware wallet status
    let status = hw_manager
        .check_hardware_device_health(0)
        .await
        .expect("Failed to check health");
    assert_eq!(status, "Operational");
}

#[tokio::test]
async fn test_full_wallet_integration() {
    // Test complete wallet integration with hardware wallets enabled
    let config = WalletConfig {
        hardware_wallet_enabled: true,
        ..Default::default()
    };

    let wallet = Vaughan::new(config).await.unwrap();

    // Test hardware wallet detection through main wallet
    let detected_wallets = wallet.detect_hardware_wallets().await.unwrap();
    assert_eq!(detected_wallets.len(), 2);

    // Test hardware wallet status
    let has_wallets = wallet.has_hardware_wallets().await;
    assert!(has_wallets);

    // Test getting addresses
    let addresses = wallet.get_hardware_addresses(0, 5).await.unwrap();
    assert_eq!(addresses.len(), 5);

    // Test device info
    let device_info = wallet.get_hardware_device_info(0).await.unwrap();
    assert!(!device_info.device_type.is_empty());

    // Test refresh
    let refreshed = wallet.refresh_hardware_devices().await.unwrap();
    assert_eq!(refreshed.len(), 2);
}

#[tokio::test]
async fn test_hardware_wallet_stress_operations() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Test rapid address derivation requests
    // Test rapid address derivation requests
    let mut address_tasks = Vec::new();
    for i in 0..10 {
        let path = format!("m/44'/60'/0'/0");
        let task = hw_manager.get_addresses(i % 2, &path, 1).await;
        address_tasks.push(task);
    }

    let results = address_tasks;
    for result in results {
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    // Test rapid signing requests
    let tx = create_test_transaction();
    let mut signing_results = Vec::new();

    for i in 0..5 {
        let path = format!("m/44'/60'/0'/0/{}", i);
        let task = hw_manager.sign_transaction(i % 2, &tx, &path).await;
        signing_results.push(task);
    }
    for result in signing_results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_hardware_wallet_timeout_handling() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Test operation with timeout simulation
    let tx = create_test_transaction();

    // This should complete quickly with mock implementations
    let start = tokio::time::Instant::now();
    let result = hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration < Duration::from_secs(1)); // Should be fast with mocks
}

#[tokio::test]
async fn test_multiple_derivation_paths() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Test various valid derivation paths
    let paths = vec![
        "m/44'/60'/0'/0/0",
        "m/44'/60'/0'/0/1",
        "m/44'/60'/0'/0/10",
        "m/44'/60'/1'/0/0",
        "m/44'/60'/2'/0/5",
    ];

    for path in paths {
        // Extract the account and address index for get_addresses call
        let path_parts: Vec<&str> = path.split('/').collect();
        let base_path = format!(
            "{}/{}/{}/{}",
            path_parts[0], path_parts[1], path_parts[2], path_parts[3]
        );

        let addresses = hw_manager.get_addresses(0, &base_path, 1).await;
        assert!(addresses.is_ok(), "Failed for path: {}", path);

        let signature = hw_manager.sign_transaction(0, &create_test_transaction(), path).await;
        assert!(signature.is_ok(), "Signing failed for path: {}", path);
    }
}

#[tokio::test]
async fn test_hardware_wallet_feedback_systems() {
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Test comprehensive feedback system
    let test_address = "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18";

    // Test address verification feedback
    let addr_feedback = hw_manager
        .verify_address_with_feedback(0, test_address, "m/44'/60'/0'/0/0")
        .await
        .unwrap();
    assert_eq!(addr_feedback.address, test_address);
    assert_eq!(addr_feedback.derivation_path, "m/44'/60'/0'/0/0");
    assert!(addr_feedback.duration_ms > 0);

    // Test transaction audit feedback
    let tx = create_test_transaction();
    let audit_feedback = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    assert!(!audit_feedback.device_type.is_empty());
    assert!(audit_feedback.duration_ms > 0);
    assert!(!audit_feedback.recommendations.is_empty());

    // Test device recovery feedback
    let recovery_feedback = hw_manager.recover_device_with_feedback("test-device").await.unwrap();
    assert_eq!(recovery_feedback.device_id, "test-device");
    // assert!(recovery_feedback.duration_ms >= 0);
    assert!(!recovery_feedback.next_steps.is_empty());
}
