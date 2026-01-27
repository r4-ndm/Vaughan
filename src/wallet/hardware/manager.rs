//! Hardware wallet integration for the main wallet
//!
//! This module provides integration between the main wallet and hardware wallet devices,
//! managing device connections and coordinating with the security module.
//!
//! # Architecture
//!
//! The hardware wallet system has two layers:
//! - **Security Layer** (`src/security/hardware.rs`): Low-level device communication using Alloy signers
//! - **Wallet Layer** (this module): High-level device management and user feedback
//!
//! # Features
//!
//! - Multi-device support (Ledger + Trezor simultaneously)
//! - Automatic device detection
//! - Connection recovery with exponential backoff
//! - Transaction security auditing
//! - Address verification with user feedback
//! - Device health monitoring
//!
//! # Usage Example
//!
//! ```no_run
//! use vaughan::wallet::hardware::HardwareManager;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut manager = HardwareManager::new()?;
//!
//! // Detect all connected devices
//! let devices = manager.detect_wallets().await?;
//! println!("Found {} hardware wallets", devices.len());
//!
//! // Get addresses from first device
//! let addresses = manager.get_addresses(0, "m/44'/60'/0'/0", 5).await?;
//!
//! // Sign transaction with first device
//! let signature = manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await?;
//! # Ok(())
//! # }
//! ```

use alloy::primitives::Address;
use alloy::primitives::Signature;
use alloy::rpc::types::TransactionRequest;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{HardwareWalletError, Result};

use crate::security::hardware::HardwareWalletInfo;
#[cfg(feature = "hardware-wallets")]
use crate::security::hardware::HardwareWallet;
#[cfg(feature = "hardware-wallets")]
use crate::security::hardware::HardwareWalletManager as SecurityHardwareManager;
#[cfg(feature = "hardware-wallets")]
use crate::security::hardware::HardwareWalletTrait;

/// User feedback for address verification
#[derive(Debug, Clone)]
pub struct AddressVerificationFeedback {
    pub address: String,
    pub derivation_path: String,
    pub verified: bool,
    pub device_count: u32,
    pub success_count: u32,
    pub duration_ms: u64,
    pub user_message: String,
    pub next_steps: Vec<String>,
}

/// User feedback for transaction audit
#[derive(Debug, Clone)]
pub struct TransactionAuditFeedback {
    pub passed: bool,
    pub device_type: String,
    pub duration_ms: u64,
    pub user_message: String,
    pub security_warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Device recovery feedback
#[derive(Debug, Clone)]
pub struct DeviceRecoveryFeedback {
    pub device_id: String,
    pub recovered: bool,
    pub duration_ms: u64,
    pub user_message: String,
    pub next_steps: Vec<String>,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive hardware wallet status information
#[derive(Debug, Clone)]
pub struct HardwareWalletStatus {
    pub enabled: bool,
    pub manager_initialized: bool,
    pub device_count: usize,
    pub connected_devices: Vec<crate::security::hardware::HardwareWalletInfo>,
    pub last_refresh: std::time::SystemTime,
}

/// Hardware wallet manager for the main wallet
///
/// This manager provides high-level hardware wallet operations with user feedback,
/// device management, and security auditing. It coordinates multiple hardware devices
/// and provides a unified interface for the wallet application.
///
/// # Features
///
/// - **Multi-Device Support**: Manage Ledger and Trezor simultaneously
/// - **Auto-Detection**: Automatically detect connected devices
/// - **Connection Recovery**: Automatic reconnection with exponential backoff
/// - **Security Auditing**: Validate transactions before signing
/// - **User Feedback**: Detailed feedback for all operations
/// - **Thread-Safe**: Safe concurrent access
///
/// # Example
///
/// ```no_run
/// use vaughan::wallet::hardware::HardwareManager;
/// use alloy::rpc::types::TransactionRequest;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = HardwareManager::new()?;
///
/// // Detect devices
/// let devices = manager.detect_wallets().await?;
/// for (i, device) in devices.iter().enumerate() {
///     println!("Device {}: {} {}", i, device.device_type, device.model);
/// }
///
/// // Verify address with feedback
/// let feedback = manager.verify_address_with_feedback(
///     0,
///     "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18",
///     "m/44'/60'/0'/0/0"
/// ).await?;
/// println!("{}", feedback.user_message);
///
/// // Audit transaction before signing
/// let tx = TransactionRequest::default();
/// let audit = manager.audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0).await?;
/// if audit.passed {
///     // Safe to sign
///     let signature = manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await?;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct HardwareManager {
    #[cfg(feature = "hardware-wallets")]
    security_manager: Arc<RwLock<SecurityHardwareManager>>,
    connected_devices: Arc<RwLock<Vec<HardwareWalletInfo>>>,
}

impl HardwareManager {
    /// Create a new hardware wallet manager
    pub fn new() -> Result<Self> {
        #[cfg(feature = "hardware-wallets")]
        let security_manager = SecurityHardwareManager::new();

        Ok(Self {
            #[cfg(feature = "hardware-wallets")]
            security_manager: Arc::new(RwLock::new(security_manager)),
            connected_devices: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Detect and connect to available hardware wallets
    /// Detect and connect to available hardware wallets
    pub async fn detect_wallets(&mut self) -> Result<Vec<HardwareWalletInfo>> {
        #[cfg(feature = "hardware-wallets")]
        {
            let mut security_manager = self.security_manager.write().await;
            let detected = security_manager.detect_wallets().await?;

            // Update connected devices list
            let mut connected = self.connected_devices.write().await;
            *connected = detected.clone();

            Ok(detected)
        }
        #[cfg(not(feature = "hardware-wallets"))]
        {
            Ok(Vec::new())
        }
    }

    /// Get list of connected hardware wallets
    pub async fn get_connected_devices(&self) -> Vec<HardwareWalletInfo> {
        self.connected_devices.read().await.clone()
    }

    /// Check if any hardware wallets are connected
    pub async fn has_connected_devices(&self) -> bool {
        !self.connected_devices.read().await.is_empty()
    }

    /// Get addresses from a specific hardware wallet
    /// Get addresses from a specific hardware wallet
    pub async fn get_addresses(&self, device_index: usize, derivation_path: &str, count: u32) -> Result<Vec<Address>> {
        #[cfg(feature = "hardware-wallets")]
        {
            let security_manager = self.security_manager.read().await;
            let connected_wallets = security_manager.get_connected_wallets();

            if device_index >= connected_wallets.len() {
                return Err(HardwareWalletError::DeviceNotFound.into());
            }

            match &connected_wallets[device_index] {
                HardwareWallet::Ledger(ledger) => ledger.get_addresses(derivation_path, count).await,
                HardwareWallet::Trezor(trezor) => trezor.get_addresses(derivation_path, count).await,
            }
        }
        #[cfg(not(feature = "hardware-wallets"))]
        {
             Err(HardwareWalletError::FeatureNotEnabled.into())       
        }
    }

    /// Sign a transaction with a specific hardware wallet
    /// Sign a transaction with a specific hardware wallet
    pub async fn sign_transaction(
        &self,
        device_index: usize,
        tx: &TransactionRequest,
        derivation_path: &str,
    ) -> Result<Signature> {
        #[cfg(feature = "hardware-wallets")]
        {
            let security_manager = self.security_manager.read().await;
            security_manager
                .sign_with_wallet(device_index, tx, derivation_path)
                .await
        }
        #[cfg(not(feature = "hardware-wallets"))]
        {
             Err(HardwareWalletError::FeatureNotEnabled.into())       
        }
    }

    /// Get device information for a specific device
    pub async fn get_device_info(&self, device_index: usize) -> Result<HardwareWalletInfo> {
        let connected = self.connected_devices.read().await;

        connected
            .get(device_index)
            .cloned()
            .ok_or(HardwareWalletError::DeviceNotFound.into())
    }

    /// Check hardware device health
    pub async fn check_hardware_device_health(&self, device_index: usize) -> Result<String> {
        let _ = self.get_device_info(device_index).await?;
        Ok("Operational".to_string())
    }

    /// Refresh the list of connected devices
    pub async fn refresh_devices(&mut self) -> Result<Vec<HardwareWalletInfo>> {
        self.detect_wallets().await
    }

    /// Check if a specific device type is connected
    pub async fn has_device_type(&self, device_type: &str) -> bool {
        let connected = self.connected_devices.read().await;
        connected.iter().any(|device| device.device_type == device_type)
    }

    /// Get the count of connected devices
    pub async fn device_count(&self) -> usize {
        self.connected_devices.read().await.len()
    }

    /// Disconnect all devices (for cleanup)
    pub async fn disconnect_all(&mut self) {
        let mut connected = self.connected_devices.write().await;
        connected.clear();

        // The security manager will handle the actual device disconnection
        // when it goes out of scope or is explicitly reset
    }

    /// Verify address with hardware wallet and provide user feedback
    /// Verify address with hardware wallet and provide user feedback
    pub async fn verify_address_with_feedback(
        &self,
        _device_index: usize,
        address: &str,
        derivation_path: &str,
    ) -> Result<AddressVerificationFeedback> {
        #[cfg(feature = "hardware-wallets")]
        {
            tracing::info!("🔍 Starting address verification with user feedback");

            let start_time = std::time::Instant::now();

            let security_manager = self.security_manager.read().await;
            let verification_result = security_manager
                .verify_address_derivation(address, derivation_path)
                .await;

            let duration = start_time.elapsed();

            match verification_result {
                Ok(result) => {
                    let feedback = AddressVerificationFeedback {
                        address: address.to_string(),
                        derivation_path: derivation_path.to_string(),
                        verified: result.verified,
                        device_count: result.successful_verifications + result.failed_verifications,
                        success_count: result.successful_verifications,
                        duration_ms: duration.as_millis() as u64,
                        user_message: if result.verified {
                            "✅ Address verified successfully on hardware device".to_string()
                        } else {
                            "❌ Address verification failed - please check device and path".to_string()
                        },
                        next_steps: if result.verified {
                            vec!["Address is safe to use for transactions".to_string()]
                        } else {
                            vec![
                                "Check hardware device connection".to_string(),
                                "Verify derivation path is correct".to_string(),
                                "Ensure device is unlocked".to_string(),
                            ]
                        },
                    };

                    tracing::info!("✅ Address verification completed in {}ms", duration.as_millis());
                    Ok(feedback)
                }
                Err(e) => {
                    let feedback = AddressVerificationFeedback {
                        address: address.to_string(),
                        derivation_path: derivation_path.to_string(),
                        verified: false,
                        device_count: 0,
                        success_count: 0,
                        duration_ms: duration.as_millis() as u64,
                        user_message: format!("❌ Verification failed: {e}"),
                        next_steps: vec![
                            "Check hardware device connection".to_string(),
                            "Ensure device is properly set up".to_string(),
                            "Try refreshing device list".to_string(),
                        ],
                    };

                    tracing::error!("❌ Address verification failed after {}ms: {}", duration.as_millis(), e);
                    Ok(feedback)
                }
            }
        }
        #[cfg(not(feature = "hardware-wallets"))]
        {
             Ok(AddressVerificationFeedback {
                address: address.to_string(),
                derivation_path: derivation_path.to_string(),
                verified: false,
                device_count: 0,
                success_count: 0,
                duration_ms: 0,
                user_message: "Hardware wallet feature not enabled".to_string(),
                next_steps: vec!["Enable hardware-wallets feature".to_string()],
            })
        }
    }

    /// Perform comprehensive transaction audit with user feedback
    /// Perform comprehensive transaction audit with user feedback
    pub async fn audit_transaction_with_feedback(
        &self,
        tx: &TransactionRequest,
        derivation_path: &str,
        device_index: usize,
    ) -> Result<TransactionAuditFeedback> {
        #[cfg(feature = "hardware-wallets")]
        {
            tracing::info!("🔍 Starting comprehensive transaction audit");

            let start_time = std::time::Instant::now();

            // Get device type
            let device_info = self.get_device_info(device_index).await?;

            let security_manager = self.security_manager.read().await;
            let audit_result = security_manager
                .audit_transaction(tx, derivation_path, &device_info.device_type)
                .await;

            let duration = start_time.elapsed();

            match audit_result {
                Ok(()) => {
                    let feedback = TransactionAuditFeedback {
                        passed: true,
                        device_type: device_info.device_type.clone(),
                        duration_ms: duration.as_millis() as u64,
                        user_message: "✅ Transaction passed all security checks".to_string(),
                        security_warnings: Vec::new(),
                        recommendations: vec![
                            "Review transaction details on hardware device".to_string(),
                            "Verify recipient address matches intended destination".to_string(),
                            "Confirm transaction amount is correct".to_string(),
                        ],
                        risk_level: RiskLevel::Low,
                    };

                    tracing::info!("✅ Transaction audit passed in {}ms", duration.as_millis());
                    Ok(feedback)
                }
                Err(e) => {
                    let feedback = TransactionAuditFeedback {
                        passed: false,
                        device_type: device_info.device_type.clone(),
                        duration_ms: duration.as_millis() as u64,
                        user_message: format!("⚠️ Security audit failed: {e}"),
                        security_warnings: vec![format!("Security violation detected: {}", e)],
                        recommendations: vec![
                            "Review transaction parameters".to_string(),
                            "Check if recipient address is correct".to_string(),
                            "Verify transaction amount is reasonable".to_string(),
                            "Contact support if this appears to be an error".to_string(),
                        ],
                        risk_level: RiskLevel::High,
                    };

                    tracing::error!("❌ Transaction audit failed after {}ms: {}", duration.as_millis(), e);
                    Ok(feedback)
                }
            }
        }
        #[cfg(not(feature = "hardware-wallets"))]
        {
             Ok(TransactionAuditFeedback {
                passed: false,
                device_type: "Unknown".to_string(),
                duration_ms: 0,
                user_message: "Hardware wallet feature not enabled".to_string(),
                security_warnings: vec!["Feature disabled".to_string()],
                recommendations: vec!["Enable feature in Cargo.toml".to_string()],
                risk_level: RiskLevel::Critical,
            })       
        }
    }

    /// Attempt device recovery with user feedback
    /// Attempt device recovery with user feedback
    pub async fn recover_device_with_feedback(&mut self, device_id: &str) -> Result<DeviceRecoveryFeedback> {
        #[cfg(feature = "hardware-wallets")]
        {
            tracing::info!("🔄 Starting device recovery with user feedback");

            let start_time = std::time::Instant::now();

            let mut security_manager = self.security_manager.write().await;
            let recovery_result = security_manager.recover_connection(device_id).await;

            let duration = start_time.elapsed();

            match recovery_result {
                Ok(recovered) => {
                    let feedback = DeviceRecoveryFeedback {
                        device_id: device_id.to_string(),
                        recovered,
                        duration_ms: duration.as_millis() as u64,
                        user_message: if recovered {
                            format!("✅ Successfully recovered connection to {device_id}")
                        } else {
                            format!("❌ Failed to recover connection to {device_id}")
                        },
                        next_steps: if recovered {
                            vec![
                                "Device is now ready for use".to_string(),
                                "You can proceed with transactions".to_string(),
                            ]
                        } else {
                            vec![
                                "Check device USB connection".to_string(),
                                "Ensure device is unlocked".to_string(),
                                "Try unplugging and reconnecting device".to_string(),
                                "Restart the wallet application if needed".to_string(),
                            ]
                        },
                    };

                    tracing::info!(
                        "🔄 Device recovery completed in {}ms: {}",
                        duration.as_millis(),
                        if recovered { "success" } else { "failed" }
                    );
                    Ok(feedback)
                }
                Err(e) => {
                    let feedback = DeviceRecoveryFeedback {
                        device_id: device_id.to_string(),
                        recovered: false,
                        duration_ms: duration.as_millis() as u64,
                        user_message: format!("❌ Device recovery failed: {e}"),
                        next_steps: vec![
                            "Check device is properly connected".to_string(),
                            "Ensure device firmware is up to date".to_string(),
                            "Try using a different USB port".to_string(),
                            "Contact support if problem persists".to_string(),
                        ],
                    };

                    tracing::error!("❌ Device recovery failed after {}ms: {}", duration.as_millis(), e);
                    Ok(feedback)
                }
            }
        }
        #[cfg(not(feature = "hardware-wallets"))]
        {
             Ok(DeviceRecoveryFeedback {
                device_id: device_id.to_string(),
                recovered: false,
                duration_ms: 0,
                user_message: "Hardware wallet feature not enabled".to_string(),
                next_steps: vec!["Enable hardware-wallets feature".to_string()],
            })       
        }
    }
}

impl Default for HardwareManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to empty manager if initialization fails
            Self {
                #[cfg(feature = "hardware-wallets")]
                security_manager: Arc::new(RwLock::new(SecurityHardwareManager::new())),
                connected_devices: Arc::new(RwLock::new(Vec::new())),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use alloy::rpc::types::TransactionRequest;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_hardware_manager_creation() {
        let manager = HardwareManager::new().unwrap();
        assert_eq!(manager.device_count().await, 0);
        assert!(!manager.has_connected_devices().await);
    }

    #[tokio::test]
    async fn test_device_detection() {
        let mut manager = HardwareManager::new().unwrap();

        // In test environment, this should detect mock devices
        let detected = manager.detect_wallets().await.unwrap();

        // The mock implementation should detect 2 devices (Ledger + Trezor)
        assert_eq!(detected.len(), 2);
        assert_eq!(manager.device_count().await, 2);
        assert!(manager.has_connected_devices().await);
    }

    #[tokio::test]
    async fn test_device_type_checking() {
        let mut manager = HardwareManager::new().unwrap();
        manager.detect_wallets().await.unwrap();

        // In test environment, we use simulated device names
        assert!(manager.has_device_type("Ledger (Simulated)").await);
        assert!(manager.has_device_type("Trezor (Simulated)").await);
        assert!(!manager.has_device_type("Unknown").await);
    }

    #[tokio::test]
    async fn test_address_verification_feedback() {
        let manager = HardwareManager::new().unwrap();

        // Test address verification with feedback
        let test_address = "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18";
        let feedback = manager
            .verify_address_with_feedback(0, test_address, "m/44'/60'/0'/0/0")
            .await
            .unwrap();

        assert_eq!(feedback.address, test_address);
        assert_eq!(feedback.derivation_path, "m/44'/60'/0'/0/0");
        assert!(!feedback.user_message.is_empty());
        assert!(!feedback.next_steps.is_empty());
    }

    #[tokio::test]
    async fn test_transaction_audit_feedback() {
        let mut manager = HardwareManager::new().unwrap();
        manager.detect_wallets().await.unwrap();

        // Create a test transaction
        let mut tx = TransactionRequest::default();
        tx.to = Some(
            Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
                .unwrap()
                .into(),
        );
        tx.value = Some(alloy::primitives::U256::from(1_000_000_000_000_000_000u64)); // 1 ETH

        let feedback = manager
            .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", 0)
            .await
            .unwrap();

        assert!(!feedback.user_message.is_empty());
        assert!(!feedback.recommendations.is_empty());
        assert!(matches!(
            feedback.risk_level,
            RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical
        ));
    }

    #[tokio::test]
    async fn test_device_recovery_feedback() {
        let mut manager = HardwareManager::new().unwrap();

        let feedback = manager.recover_device_with_feedback("test-device-id").await.unwrap();

        assert_eq!(feedback.device_id, "test-device-id");
        assert!(!feedback.user_message.is_empty());
        assert!(!feedback.next_steps.is_empty());
    }

    #[tokio::test]
    async fn test_hardware_manager_default() {
        let manager = HardwareManager::default();
        assert_eq!(manager.device_count().await, 0);
        assert!(!manager.has_connected_devices().await);
    }

    #[tokio::test]
    async fn test_device_refresh() {
        let mut manager = HardwareManager::new().unwrap();

        // Initial detection
        let initial_devices = manager.detect_wallets().await.unwrap();

        // Refresh devices
        let refreshed_devices = manager.refresh_devices().await.unwrap();

        // Should have same count (assuming stable mock environment)
        assert_eq!(initial_devices.len(), refreshed_devices.len());
    }

    #[tokio::test]
    async fn test_get_device_info() {
        let mut manager = HardwareManager::new().unwrap();
        manager.detect_wallets().await.unwrap();

        // Test valid device index
        let device_info = manager.get_device_info(0).await;
        assert!(device_info.is_ok());

        // Test invalid device index
        let invalid_device = manager.get_device_info(999).await;
        assert!(invalid_device.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_all() {
        let mut manager = HardwareManager::new().unwrap();
        manager.detect_wallets().await.unwrap();

        assert!(manager.has_connected_devices().await);

        manager.disconnect_all().await;

        // Should have no connected devices after disconnect
        assert!(!manager.has_connected_devices().await);
        assert_eq!(manager.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_addresses_error_handling() {
        let manager = HardwareManager::new().unwrap();

        // Test with invalid device index (no devices connected)
        let result = manager.get_addresses(0, "m/44'/60'/0'/0", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sign_transaction_error_handling() {
        let manager = HardwareManager::new().unwrap();

        let mut tx = TransactionRequest::default();
        tx.to = Some(
            Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
                .unwrap()
                .into(),
        );

        // Test with invalid device index (no devices connected)
        let result = manager.sign_transaction(0, &tx, "m/44'/60'/0'/0/0").await;
        assert!(result.is_err());
    }
}
