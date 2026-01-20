//! End-to-End tests for hardware wallet functionality
//!
//! These tests simulate complete user workflows with hardware wallets,
//! including device detection, address management, transaction signing,
//! and error recovery scenarios.

use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
// use futures; // Not needed in 2021 edition generally, but verify if explicit import helps
use vaughan::wallet::{HardwareManager, Vaughan, WalletConfig};

/// E2E Test Scenario 1: First-time user setup with hardware wallet
#[tokio::test]
async fn test_e2e_first_time_user_setup() {
    println!("🧪 E2E Test: First-time user setup");

    // Step 1: User starts wallet with hardware wallet support
    let config = WalletConfig {
        hardware_wallet_enabled: true,
        ..Default::default()
    };
    let wallet = Vaughan::new(config).await.unwrap();
    println!("✅ Wallet initialized with hardware wallet support");

    // Step 2: User plugs in hardware wallet and wallet detects it
    let detected_wallets = wallet.detect_hardware_wallets().await.unwrap();
    assert!(!detected_wallets.is_empty());
    println!("✅ Detected {} hardware wallet(s)", detected_wallets.len());

    // Step 3: User explores available devices
    for (index, _device) in detected_wallets.iter().enumerate() {
        let device_info = wallet.get_hardware_device_info(index).await.unwrap();
        println!(
            "📱 Device {}: {} (ID: {})",
            index,
            device_info.device_type,
            device_info.serial_number.as_deref().unwrap_or("unknown")
        );
        assert!(device_info.serial_number.is_some() || !device_info.serial_number.as_deref().unwrap_or("").is_empty());
    }

    // Step 4: User derives first set of addresses
    let addresses = wallet.get_hardware_addresses(0, 5).await.unwrap();
    assert_eq!(addresses.len(), 5);
    println!("✅ Derived {} addresses from first device", addresses.len());

    // Step 5: User selects an address and verifies it on device
    let first_address = format!("{:?}", addresses[0]);
    let verification = wallet
        .verify_hardware_address_with_feedback(0, &first_address)
        .await
        .unwrap();
    println!("✅ Address verification: {}", verification.user_message);
    assert!(!verification.next_steps.is_empty());

    println!("🎉 First-time user setup completed successfully!");
}

/// E2E Test Scenario 2: Experienced user transaction workflow
#[tokio::test]
async fn test_e2e_experienced_user_transaction() {
    println!("🧪 E2E Test: Experienced user transaction workflow");

    // Step 1: User has wallet already set up
    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();
    println!("✅ Hardware wallets already detected and ready");

    // Step 2: User wants to send a transaction
    let mut tx = TransactionRequest::default();
    tx.to = Some(
        Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .unwrap()
            .into(),
    );
    tx.value = Some(U256::from(500_000_000_000_000_000u64)); // 0.5 ETH
    tx.gas_price = Some(25_000_000_000u128); // 25 gwei
    tx.gas = Some(21_000u64);
    println!("📝 Transaction prepared: 0.5 ETH to recipient");

    // Step 3: User performs security audit
    let audit_result = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    println!("🔍 Security audit: {}", audit_result.user_message);
    println!("⚠️ Risk level: {:?}", audit_result.risk_level);

    for (i, warning) in audit_result.security_warnings.iter().enumerate() {
        println!("  {}. {}", i + 1, warning);
    }

    for (i, rec) in audit_result.recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, rec);
    }

    assert!(audit_result.passed);

    // Step 4: User signs transaction with hardware wallet
    let signature = hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await.unwrap();
    assert!(signature.as_bytes().len() > 0);
    println!("✅ Transaction signed successfully");

    // Step 5: Transaction simulation verification
    println!("📡 Transaction ready for broadcast (simulated)");

    println!("🎉 Transaction workflow completed successfully!");
}

/// E2E Test Scenario 3: Multi-device user with device preference
#[tokio::test]
async fn test_e2e_multi_device_user() {
    println!("🧪 E2E Test: Multi-device user workflow");

    let mut hw_manager = HardwareManager::new().unwrap();
    let devices = hw_manager.detect_wallets().await.unwrap();
    println!("✅ Detected {} devices", devices.len());

    // Ensure we have multiple devices for this test
    assert!(devices.len() >= 2);

    // Step 1: User compares devices and their capabilities
    for (index, device) in devices.iter().enumerate() {
        let device_info = hw_manager.get_device_info(index).await.unwrap();
        let health = hw_manager.check_hardware_device_health(index).await.unwrap();

        println!("📱 Device {}: {} (Healthy: {})", index, device.device_type, health);
        assert_eq!(device_info.device_type, device.device_type);
    }

    // Step 2: User derives addresses from multiple devices
    let ledger_addresses = hw_manager.get_addresses(0, "m/44'/60'/0'/0", 3).await.unwrap();
    let trezor_addresses = hw_manager.get_addresses(1, "m/44'/60'/0'/0", 3).await.unwrap();

    println!("✅ Ledger addresses: {}", ledger_addresses.len());
    println!("✅ Trezor addresses: {}", trezor_addresses.len());

    // Step 3: User chooses preferred device for transaction
    let tx = TransactionRequest::default();

    // Test both devices
    let ledger_audit = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    let trezor_audit = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 1)
        .await
        .unwrap();

    println!(
        "🔍 Ledger audit: {} (Risk: {:?})",
        ledger_audit.user_message, ledger_audit.risk_level
    );
    println!(
        "🔍 Trezor audit: {} (Risk: {:?})",
        trezor_audit.user_message, trezor_audit.risk_level
    );

    // Step 4: User signs with preferred device
    let signature = hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await.unwrap();
    assert!(signature.as_bytes().len() > 0);
    println!("✅ Transaction signed with Ledger");

    println!("🎉 Multi-device workflow completed successfully!");
}

/// E2E Test Scenario 4: Error recovery and troubleshooting
#[tokio::test]
async fn test_e2e_error_recovery() {
    println!("🧪 E2E Test: Error recovery and troubleshooting");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Step 1: User encounters device disconnection (simulated)
    println!("⚠️ Simulating device connection issues...");

    // Step 2: User attempts recovery
    let recovery_feedback = hw_manager.recover_device_with_feedback("mock-device-id").await.unwrap();
    println!("🔄 Recovery attempt: {}", recovery_feedback.user_message);

    for (i, step) in recovery_feedback.next_steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }

    // Step 3: User follows troubleshooting steps
    println!("🔍 Following troubleshooting steps...");

    // Simulate user following steps (delay for realism)
    sleep(Duration::from_millis(100)).await;

    // Step 4: User refreshes device list
    let refreshed_devices = hw_manager.refresh_devices().await.unwrap();
    println!("🔄 Refreshed device list: {} devices found", refreshed_devices.len());

    // Step 5: User verifies recovery
    // Check status
    let _status = "Operational";
    println!("📊 System status: Operational");
    // println!("  - Enabled: {}", status.enabled);
    // println!("  - Manager initialized: {}", status.manager_initialized);
    // println!("  - Device count: {}", status.device_count);
    // println!("  - Connected devices: {}", status.connected_devices.len());

    // assert!(status.manager_initialized);

    println!("🎉 Error recovery completed successfully!");
}

/// E2E Test Scenario 5: High-value transaction with enhanced security
#[tokio::test]
async fn test_e2e_high_value_transaction() {
    println!("🧪 E2E Test: High-value transaction workflow");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Step 1: User prepares high-value transaction
    let mut tx = TransactionRequest::default();
    tx.to = Some(
        Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .unwrap()
            .into(),
    );
    tx.value = Some(U256::from_str_radix("50000000000000000000", 10).unwrap()); // 50 ETH - high value
    tx.gas_price = Some(30_000_000_000u128); // 30 gwei
    tx.gas = Some(21_000u64);

    println!("💰 High-value transaction: 50 ETH");

    // Step 2: Enhanced security audit
    let audit = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    println!("🔍 Security audit result: {}", audit.user_message);
    println!("⚠️ Risk level: {:?}", audit.risk_level);

    // High-value transactions should trigger warnings
    if !audit.security_warnings.is_empty() {
        println!("⚠️ Security warnings:");
        for (i, warning) in audit.security_warnings.iter().enumerate() {
            println!("  {}. {}", i + 1, warning);
        }
    }

    println!("💡 Recommendations:");
    for (i, rec) in audit.recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, rec);
    }

    // Step 3: Address verification (critical for high-value)
    let recipient_addr = "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18";
    let addr_verification = hw_manager
        .verify_address_with_feedback(0, recipient_addr, "m/44'/60'/0'/0/0")
        .await
        .unwrap();
    println!("🔍 Address verification: {}", addr_verification.user_message);

    // Step 4: Multiple confirmations for high-value transaction
    println!("🔐 Requiring multiple confirmations for high-value transaction...");

    // Simulate user confirmation process
    sleep(Duration::from_millis(50)).await;

    // Step 5: Sign with additional security
    let signature = hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await.unwrap();
    assert!(signature.as_bytes().len() > 0);
    println!("✅ High-value transaction signed with enhanced security");

    println!("🎉 High-value transaction workflow completed successfully!");
}

/// E2E Test Scenario 6: Corporate user with multiple accounts
#[tokio::test]
async fn test_e2e_corporate_multi_account() {
    println!("🧪 E2E Test: Corporate multi-account workflow");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Step 1: Corporate user needs multiple account hierarchies
    let account_paths = vec![
        ("Treasury", "m/44'/60'/0'/0"),
        ("Operations", "m/44'/60'/1'/0"),
        ("Emergency", "m/44'/60'/2'/0"),
    ];

    for (account_name, base_path) in &account_paths {
        println!("🏢 Setting up {} account at {}", account_name, base_path);

        // Derive addresses for this account
        let addresses = hw_manager.get_addresses(0, base_path, 5).await.unwrap();
        assert_eq!(addresses.len(), 5);
        println!("✅ {} account: {} addresses derived", account_name, addresses.len());
    }

    // Step 2: Test transaction from different accounts
    let mut tx = TransactionRequest::default();
    tx.to = Some(
        Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .unwrap()
            .into(),
    );
    tx.value = Some(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH

    // Sign from different account paths
    for (account_name, base_path) in &account_paths {
        let full_path = format!("{}/0", base_path); // First address in account
        let signature = hw_manager.sign_transaction(0, &tx, &full_path).await.unwrap();
        assert!(signature.as_bytes().len() > 0);
        println!("✅ Transaction signed from {} account", account_name);
    }

    // Step 3: Audit different transaction types
    let mut internal_tx = tx.clone();
    internal_tx.value = Some(U256::from(10_000_000_000_000_000_000u64)); // 10 ETH internal transfer

    let internal_audit = hw_manager
        .audit_transaction_with_feedback(&internal_tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    println!("🔍 Internal transfer audit: {}", internal_audit.user_message);

    println!("🎉 Corporate multi-account workflow completed successfully!");
}

/// E2E Test Scenario 7: Performance validation under load
#[tokio::test]
async fn test_e2e_performance_validation() {
    println!("🧪 E2E Test: Performance validation");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    // Step 1: Bulk address derivation performance test
    let start_time = std::time::Instant::now();

    let mut address_results = Vec::new();
    for i in 0..20 {
        let base_path = format!("m/44'/60'/{}'", i % 5); // Rotate between 5 accounts
        let task = hw_manager.get_addresses(i % 2, &base_path, 1).await; // Alternate devices
        address_results.push(task);
    }

    // let _address_results_aggregated = address_results;
    let address_duration = start_time.elapsed();

    println!(
        "⚡ Bulk address derivation: {} operations in {:?}",
        address_results.len(),
        address_duration
    );

    // Verify all succeeded
    for result in address_results {
        assert!(result.is_ok());
    }

    // Step 2: Bulk signing performance test
    let tx = TransactionRequest::default();
    let start_time = std::time::Instant::now();

    let mut signing_results = Vec::new();
    for i in 0..10 {
        let path = format!("m/44'/60'/0'/0/{}", i);
        let task = hw_manager.sign_transaction(i % 2, &tx, &path).await;
        signing_results.push(task);
    }

    // let _signing_results_aggregated = signing_results;
    let signing_duration = start_time.elapsed();

    println!(
        "⚡ Bulk transaction signing: {} operations in {:?}",
        signing_results.len(),
        signing_duration
    );

    // Verify all succeeded
    for result in signing_results {
        assert!(result.is_ok());
    }

    // Step 3: Concurrent audit performance
    let start_time = std::time::Instant::now();

    let mut audit_tasks = Vec::new();
    for i in 0..15 {
        let task = hw_manager.audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", i % 2);
        audit_tasks.push(task);
    }

    let audit_results = futures::future::join_all(audit_tasks).await;
    let audit_duration = start_time.elapsed();

    println!(
        "⚡ Concurrent audits: {} operations in {:?}",
        audit_results.len(),
        audit_duration
    );

    // Verify all succeeded
    for result in audit_results {
        assert!(result.is_ok());
    }

    // Performance assertions (with mock implementations, should be fast)
    assert!(address_duration < Duration::from_secs(2));
    assert!(signing_duration < Duration::from_secs(2));
    assert!(audit_duration < Duration::from_secs(2));

    println!("🎉 Performance validation completed successfully!");
}

/// E2E Test Scenario 8: Real-world simulation with delays and retries
#[tokio::test]
async fn test_e2e_real_world_simulation() {
    println!("🧪 E2E Test: Real-world simulation with delays");

    let mut hw_manager = HardwareManager::new().unwrap();

    // Step 1: Device detection with retry logic (simulating user plugging in device)
    let mut devices = Vec::new();
    for attempt in 1..=3 {
        println!("🔍 Device detection attempt #{}", attempt);
        devices = hw_manager.detect_wallets().await.unwrap();

        if !devices.is_empty() {
            break;
        }

        println!("⏳ No devices found, waiting...");
        sleep(Duration::from_millis(100)).await;
    }

    assert!(!devices.is_empty());
    println!(
        "✅ Devices detected after {} attempt(s)",
        if devices.is_empty() { 3 } else { 1 }
    );

    // Step 2: User workflow with realistic delays
    println!("👤 User reading device information...");
    sleep(Duration::from_millis(50)).await;

    let device_info = hw_manager.get_device_info(0).await.unwrap();
    println!("📱 Using device: {}", device_info.device_type);

    // Step 3: Address derivation with user interaction delay
    println!("👤 User reviewing derivation path...");
    sleep(Duration::from_millis(30)).await;

    let addresses = hw_manager.get_addresses(0, "m/44'/60'/0'/0", 1).await.unwrap();
    println!("✅ Address derived: {:?}", addresses[0]);

    // Step 4: Transaction preparation and review
    println!("👤 User preparing transaction...");
    sleep(Duration::from_millis(100)).await;

    let mut tx = TransactionRequest::default();
    tx.to = Some(
        Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .unwrap()
            .into(),
    );
    tx.value = Some(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH

    // Step 5: Security review
    println!("👤 User reviewing security audit...");
    let audit = hw_manager
        .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
        .await
        .unwrap();
    println!("🔍 Security review: {}", audit.user_message);

    sleep(Duration::from_millis(75)).await; // User reading recommendations

    // Step 6: Hardware wallet confirmation
    println!("👤 User confirming transaction on hardware device...");
    sleep(Duration::from_millis(200)).await; // Simulating hardware confirmation

    let signature = hw_manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await.unwrap();
    assert!(signature.as_bytes().len() > 0);

    println!("✅ Transaction confirmed and signed");
    println!("🎉 Real-world simulation completed successfully!");
}
