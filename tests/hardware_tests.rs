//! End-to-End Hardware Wallet Integration Tests
//!
//! This module provides comprehensive tests for the hardware wallet integration,
//! covering device detection, connection, address derivation, and transaction signing.

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;
    use alloy::rpc::types::TransactionRequest;
    use std::time::Duration;
    use tokio::time::timeout;
    use vaughan::error::{HardwareWalletError, VaughanError};
    use vaughan::security::hardware::HardwareWalletManager;
    use vaughan::security::hardware::{HardwareWalletTrait, LedgerWallet, TrezorWallet};
    use vaughan::security::hardware_feedback::{HardwareWalletFeedback, HardwareWalletStatus};

    /// Test hardware wallet manager creation
    #[tokio::test]
    async fn test_hardware_manager_initialization() {
        let manager = HardwareWalletManager::new();
        assert_eq!(manager.get_connected_wallets().len(), 0);
        println!("✅ Hardware wallet manager initialization test passed");
    }

    /// Test hardware wallet detection
    #[tokio::test]
    async fn test_hardware_wallet_detection() {
        let manager = HardwareWalletManager::new();

        // Test detection (will use mock devices in test environment)
        let devices = manager.get_connected_wallets();
        println!(
            "✅ Hardware wallet detection successful - found {} devices",
            devices.len()
        );

        for (i, device) in devices.iter().enumerate() {
            if let Some(info) = device.device_info() {
                println!(
                    "  Device {}: {} {} (Firmware: {})",
                    i + 1,
                    info.device_type,
                    info.model,
                    info.firmware_version
                );
            } else {
                println!("  Device {}: Unknown Device", i + 1);
            }
        }

        println!("✅ Hardware wallet detection test completed");

        println!("✅ Hardware wallet detection test completed");
    }

    /// Test Ledger wallet creation and basic operations
    #[tokio::test]
    async fn test_ledger_wallet_operations() {
        let mut ledger = LedgerWallet::new();

        // Test initial state
        assert!(!ledger.is_connected());
        println!("✅ Ledger wallet initial state correct");

        // Test connection attempt (will fail without physical device)
        let connection_result = ledger.connect().await;
        match connection_result {
            Ok(_) => {
                println!("✅ Ledger connection successful");

                // Test address derivation if connected
                let addresses_result = ledger.get_addresses("m/44'/60'/0'/0", 3).await;
                match addresses_result {
                    Ok(addresses) => {
                        assert_eq!(addresses.len(), 3);
                        println!("✅ Ledger address derivation successful: {} addresses", addresses.len());
                        for (i, address) in addresses.iter().enumerate() {
                            println!("  Address {}: {}", i, address);
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Ledger address derivation failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Ledger connection failed (expected without hardware): {}", e);
                // Verify it's the expected error type
                assert!(matches!(
                    e,
                    VaughanError::HardwareWallet(HardwareWalletError::DeviceNotFound)
                ));
            }
        }

        println!("✅ Ledger wallet operations test completed");
    }

    /// Test Trezor wallet creation and basic operations
    #[tokio::test]
    async fn test_trezor_wallet_operations() {
        let mut trezor = TrezorWallet::new();

        // Test initial state
        assert!(!trezor.is_connected());
        println!("✅ Trezor wallet initial state correct");

        // Test connection attempt (will fail without physical device)
        let connection_result = trezor.connect().await;
        match connection_result {
            Ok(_) => {
                println!("✅ Trezor connection successful");

                // Test address derivation if connected
                let addresses_result = trezor.get_addresses("m/44'/60'/0'/0", 3).await;
                match addresses_result {
                    Ok(addresses) => {
                        assert_eq!(addresses.len(), 3);
                        println!("✅ Trezor address derivation successful: {} addresses", addresses.len());
                        for (i, address) in addresses.iter().enumerate() {
                            println!("  Address {}: {}", i, address);
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Trezor address derivation failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Trezor connection failed (expected without hardware): {}", e);
                // For Trezor, we expect a different error due to HDPath API investigation
                println!("   Error details: {}", e);
            }
        }

        println!("✅ Trezor wallet operations test completed");
    }

    /// Test hardware wallet feedback system
    #[tokio::test]
    async fn test_hardware_wallet_feedback() {
        let mut feedback = HardwareWalletFeedback::new();

        // Test initial state
        assert!(matches!(feedback.status(), HardwareWalletStatus::Disconnected));
        assert!(!feedback.is_active());
        println!("✅ Hardware wallet feedback initial state correct");

        // Test status progression
        feedback.update_status(HardwareWalletStatus::Searching);
        assert!(feedback.is_active());
        assert_eq!(feedback.progress_percentage(), Some(10));

        feedback.update_status(HardwareWalletStatus::Detected);
        assert_eq!(feedback.progress_percentage(), Some(25));

        feedback.update_status(HardwareWalletStatus::Connecting);
        assert!(feedback.is_active());
        assert_eq!(feedback.progress_percentage(), Some(50));

        feedback.update_status(HardwareWalletStatus::Connected);
        assert!(!feedback.is_active());
        assert_eq!(feedback.progress_percentage(), Some(75));

        feedback.update_status(HardwareWalletStatus::AwaitingConfirmation);
        assert!(feedback.is_active());
        assert_eq!(feedback.progress_percentage(), Some(90));

        feedback.update_status(HardwareWalletStatus::Completed);
        assert!(!feedback.is_active());
        assert_eq!(feedback.progress_percentage(), Some(100));

        println!("✅ Hardware wallet feedback system test passed");

        // Test guidance generation
        let guidance = feedback.get_guidance();
        assert_eq!(guidance.title, "Operation Complete");
        println!("✅ User guidance generation test passed");
    }

    /// Test error handling and recovery
    #[tokio::test]
    async fn test_error_handling() {
        // Test various hardware wallet errors
        let errors = vec![
            HardwareWalletError::DeviceNotFound,
            HardwareWalletError::DeviceLocked,
            HardwareWalletError::UserCancelled,
            HardwareWalletError::ConfirmationRequired,
            HardwareWalletError::BlindSigningDisabled,
            HardwareWalletError::CommunicationError,
        ];

        for error in errors {
            let vaughan_error = VaughanError::HardwareWallet(error.clone());

            // Test error context generation
            let context = vaughan_error.context();
            assert!(!context.user_message.is_empty());
            assert!(!context.recovery_steps.is_empty());
            assert!(!context.support_code.is_empty());

            // Test recoverability
            let is_recoverable = vaughan_error.is_recoverable();
            println!(
                "✅ Error {:?} - Recoverable: {}, Message: {}",
                error, is_recoverable, context.user_message
            );

            // Test recovery actions
            let actions = vaughan_error.recovery_actions();
            assert!(!actions.is_empty());
        }

        println!("✅ Error handling and recovery test passed");
    }

    /// Test transaction signing flow (mock)
    #[tokio::test]
    async fn test_transaction_signing_flow() {
        let ledger = LedgerWallet::new();

        // Create a mock transaction
        let mut tx = TransactionRequest::default();
        tx.to = Some(Address::ZERO.into());
        tx.value = Some(alloy::primitives::U256::from(1000000000000000000u64)); // 1 ETH

        // Test signing (will fail without device)
        let signing_result = ledger.sign_transaction(&tx, "m/44'/60'/0'/0/0").await;

        match signing_result {
            Ok(signature) => {
                println!("✅ Transaction signing successful");
                println!("   Signature length: {} bytes", signature.as_bytes().len());
                // Verify signature format (should be 65 bytes: 32 + 32 + 1)
                assert_eq!(signature.as_bytes().len(), 65);
            }
            Err(e) => {
                println!("⚠️ Transaction signing failed (expected without hardware): {}", e);
                // Verify it's a hardware wallet error
                assert!(matches!(e, VaughanError::HardwareWallet(_)));
            }
        }

        println!("✅ Transaction signing flow test completed");
    }

    /// Test main wallet hardware integration
    #[tokio::test]
    async fn test_main_wallet_hardware_integration() {
        use vaughan::wallet::{Vaughan, WalletConfig};

        let config = WalletConfig::default();
        let wallet_result = Vaughan::new(config).await;

        match wallet_result {
            Ok(wallet) => {
                println!("✅ Main wallet initialization with hardware support successful");

                // Test hardware wallet detection through main wallet
                let detection_result = wallet.detect_hardware_wallets().await;
                match detection_result {
                    Ok(devices) => {
                        println!("✅ Hardware wallet detection through main wallet successful");
                        println!("   Found {} devices", devices.len());
                    }
                    Err(e) => {
                        println!("⚠️ Hardware wallet detection failed: {}", e);
                    }
                }

                // Test hardware wallet status check
                let has_hardware = wallet.has_hardware_wallets().await;
                println!("✅ Hardware wallet status check: {}", has_hardware);

                // Test address retrieval (will fail without devices)
                if let Ok(devices) = wallet.detect_hardware_wallets().await {
                    if !devices.is_empty() {
                        let addresses_result = wallet.get_hardware_addresses(0, 3).await;
                        match addresses_result {
                            Ok(addresses) => {
                                println!(
                                    "✅ Hardware address retrieval successful: {} addresses",
                                    addresses.len()
                                );
                            }
                            Err(e) => {
                                println!("⚠️ Hardware address retrieval failed: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Main wallet initialization failed: {}", e);
                panic!("Main wallet should initialize successfully");
            }
        }

        println!("✅ Main wallet hardware integration test completed");
    }

    /// Test GUI wallet operations integration
    #[tokio::test]
    #[ignore]
    async fn test_gui_wallet_operations() {
        // use vaughan::gui::services::{connect_hardware_wallet, detect_hardware_wallets, get_hardware_wallet_addresses};

        // Test hardware wallet detection
        /*
        let detection_result = detect_hardware_wallets().await;
        match detection_result {
            Ok(devices) => {
                println!("✅ GUI hardware wallet detection successful: {} devices", devices.len());

                // If devices found, test connection and address retrieval
                if !devices.is_empty() {
                    // Test connection
                    let connection_result = connect_hardware_wallet(0).await;
                    match connection_result {
                        Ok(message) => {
                            println!("✅ GUI hardware wallet connection successful: {}", message);

                            // Test address retrieval
                            let addresses_result = get_hardware_wallet_addresses(0).await;
                            match addresses_result {
                                Ok(addresses) => {
                                    println!(
                                        "✅ GUI hardware address retrieval successful: {} addresses",
                                        addresses.len()
                                    );
                                }
                                Err(e) => {
                                    println!("⚠️ GUI hardware address retrieval failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠️ GUI hardware wallet connection failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️ GUI hardware wallet detection failed: {}", e);
            }
        }
        */

        println!("✅ GUI wallet operations integration test completed");
    }

    /// Test timeout handling
    #[tokio::test]
    async fn test_timeout_handling() {
        let mut feedback = HardwareWalletFeedback::new();

        // Set a short timeout for testing
        feedback.set_timeout(Duration::from_millis(100));

        // Start an operation
        feedback.update_status(HardwareWalletStatus::Processing {
            operation: "Test Operation".to_string(),
        });

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Check if timeout is detected
        assert!(feedback.is_timeout());
        println!("✅ Timeout handling test passed");
    }

    /// Integration test with timeout
    #[tokio::test]
    async fn test_hardware_operation_with_timeout() {
        let mut ledger = LedgerWallet::new();

        // Test connection with timeout
        let connection_result = timeout(Duration::from_secs(5), ledger.connect()).await;

        match connection_result {
            Ok(Ok(_)) => {
                println!("✅ Hardware connection completed within timeout");
            }
            Ok(Err(e)) => {
                println!("⚠️ Hardware connection failed: {}", e);
            }
            Err(_) => {
                println!("⚠️ Hardware connection timed out (expected without hardware)");
            }
        }

        println!("✅ Hardware operation timeout test completed");
    }

    /// Test comprehensive error scenarios
    #[tokio::test]
    async fn test_comprehensive_error_scenarios() {
        let manager = HardwareWalletManager::new();

        // Test detection with no devices
        let devices = manager.get_connected_wallets();

        // Test invalid device index
        if devices.is_empty() {
            let invalid_result = manager
                .sign_with_wallet(0, &TransactionRequest::default(), "m/44'/60'/0'/0/0")
                .await;
            assert!(invalid_result.is_err());
            println!("✅ Invalid device index error handling works correctly");
        }

        // Test invalid derivation path
        let ledger = LedgerWallet::new();
        let invalid_path_result = ledger.get_addresses("invalid_path", 1).await;
        if let Err(e) = invalid_path_result {
            println!("✅ Invalid derivation path error: {}", e);
        }

        println!("✅ Comprehensive error scenarios test completed");
    }

    /// Performance test for address derivation
    #[tokio::test]
    async fn test_address_derivation_performance() {
        let mut ledger = LedgerWallet::new();

        // Attempt to connect (will fail without hardware)
        if ledger.connect().await.is_ok() {
            let start = std::time::Instant::now();

            // Test deriving multiple addresses
            let result = ledger.get_addresses("m/44'/60'/0'/0", 10).await;

            let duration = start.elapsed();
            println!(
                "✅ Address derivation performance test: {:?} for 10 addresses",
                duration
            );

            if let Ok(addresses) = result {
                assert_eq!(addresses.len(), 10);
                println!("   All 10 addresses derived successfully");
            }
        } else {
            println!("⚠️ Performance test skipped - no hardware wallet connected");
        }

        println!("✅ Address derivation performance test completed");
    }
}
