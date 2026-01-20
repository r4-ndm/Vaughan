//! Simple Hardware Wallet Integration Verification
//!
//! This module provides basic verification that hardware wallet integration compiles
//! and basic functionality works as expected.

#[cfg(test)]
mod integration_tests {
    use vaughan::security::{HardwareWalletManager, HardwareWalletTrait};

    #[tokio::test]
    async fn test_hardware_wallet_compilation() {
        // Test that hardware wallet managers can be created
        let manager = HardwareWalletManager::new();
        assert_eq!(manager.get_connected_wallets().len(), 0);
        println!("✅ Hardware wallet manager compiles and creates successfully");
    }

    #[tokio::test]
    async fn test_feedback_system() {
        // Test that feedback system works
        let mut feedback = vaughan::security::hardware_feedback::HardwareWalletFeedback::new();
        assert!(matches!(
            feedback.status(),
            vaughan::security::hardware_feedback::HardwareWalletStatus::Disconnected
        ));

        feedback.update_status(vaughan::security::hardware_feedback::HardwareWalletStatus::Searching);
        assert_eq!(feedback.progress_percentage(), Some(10));
        println!("✅ Hardware wallet feedback system works correctly");
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test that error handling works
        let error = vaughan::error::VaughanError::HardwareWallet(vaughan::error::HardwareWalletError::DeviceNotFound);

        let context = error.context();
        assert!(!context.user_message.is_empty());
        assert!(!context.recovery_steps.is_empty());
        assert!(error.is_recoverable());

        let actions = error.recovery_actions();
        assert!(!actions.is_empty());
        println!("✅ Hardware wallet error handling works correctly");
    }

    #[tokio::test]
    async fn test_hardware_wallets_creation() {
        // Test that hardware wallets can be created
        let ledger = vaughan::security::hardware::LedgerWallet::new();
        let trezor = vaughan::security::hardware::TrezorWallet::new();

        assert!(!ledger.is_connected());
        assert!(!trezor.is_connected());
        println!("✅ Hardware wallet creation works correctly");
    }

    #[tokio::test]
    async fn test_main_wallet_hardware_support() {
        // Test that main wallet can be created with hardware support
        let config = vaughan::wallet::WalletConfig::default();
        let result = vaughan::wallet::Vaughan::new(config).await;

        match result {
            Ok(wallet) => {
                // Test basic hardware wallet operations
                let has_hardware = wallet.has_hardware_wallets().await;
                println!("✅ Main wallet with hardware support created successfully");
                println!("   Hardware wallets available: {}", has_hardware);
            }
            Err(e) => {
                panic!("Failed to create main wallet with hardware support: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_gui_operations_compile() {
        // Test that GUI operations compile correctly

        // vaughan::gui::utils::detect_hardware_wallets returns Vec<String>
        let detection_result = vaughan::gui::utils::detect_hardware_wallets().await;
        println!(
            "✅ GUI hardware detection successful: {} devices",
            detection_result.len()
        );

        println!("✅ GUI operations compile and run correctly");
    }
}

/// Manual verification function that can be called to test the integration
pub async fn verify_hardware_wallet_integration() -> vaughan::error::Result<()> {
    use vaughan::security::HardwareWalletManager;

    println!("🔍 Verifying Hardware Wallet Integration...");

    // Test 1: Manager creation
    println!("1. Testing hardware wallet manager creation...");
    let manager = HardwareWalletManager::new();
    println!("   ✅ Hardware wallet manager created successfully");

    // Test 2: Feedback system
    println!("2. Testing feedback system...");
    let mut feedback = vaughan::security::hardware_feedback::HardwareWalletFeedback::new();
    feedback.update_status(vaughan::security::hardware_feedback::HardwareWalletStatus::Connected);
    let guidance = feedback.get_guidance();
    println!("   ✅ Feedback system working: {}", guidance.title);

    // Test 3: Error handling
    println!("3. Testing error handling...");
    let error = vaughan::error::VaughanError::HardwareWallet(vaughan::error::HardwareWalletError::DeviceNotFound);
    let context = error.context();
    println!("   ✅ Error context: {}", context.user_message);

    // Test 4: Main wallet integration
    println!("4. Testing main wallet integration...");
    let config = vaughan::wallet::WalletConfig::default();
    let wallet = vaughan::wallet::Vaughan::new(config).await?;
    println!("   ✅ Main wallet with hardware support initialized");

    // Test 5: Hardware wallet creation
    println!("5. Testing hardware wallet creation...");
    let _ledger = vaughan::security::hardware::LedgerWallet::new();
    let _trezor = vaughan::security::hardware::TrezorWallet::new();
    println!("   ✅ Hardware wallets created successfully");

    println!("\n🎉 Hardware Wallet Integration Verification Complete!");
    println!("   All core components compile and initialize correctly.");
    println!("   Ready for hardware wallet testing with physical devices.");

    Ok(())
}
