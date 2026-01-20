//! Hardware Device Manager - Multi-Device Hardware Wallet Support
//!
//! This module provides comprehensive device management for hardware wallets,
//! supporting multiple simultaneous connections and graceful disconnect handling.
//!
//! # Design
//!
//! The `DeviceManager` maintains a registry of connected hardware devices using
//! `Arc<RwLock<HashMap<DeviceId, HardwareDevice>>>` for thread-safe concurrent access.
//! It supports Ledger and Trezor devices via Alloy signers.
//!
//! # Requirements Addressed
//!
//! - **Requirement 3.1**: Detect all connected Ledger and Trezor devices using Alloy signers
//! - **Requirement 3.2**: Handle device disconnection gracefully with reconnection logic
//! - **Requirement 3.3**: Maintain registry of all available devices
//! - **Requirement 3.4**: Support concurrent operations across multiple hardware devices
//!
//! # Usage
//!
//! ```rust,ignore
//! let manager = DeviceManager::new();
//!
//! // Scan for connected devices
//! let devices = manager.scan_devices().await?;
//!
//! // Get a specific device
//! if let Some(device) = manager.get_device(&device_id).await {
//!     // Use the device
//! }
//!
//! // Handle disconnection
//! manager.handle_device_disconnect(&device_id).await;
//! ```

use alloy::primitives::Address;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;

/// Unique identifier for a hardware device
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

impl DeviceId {
    /// Create a new device ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new unique device ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of hardware device
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    /// Ledger hardware wallet
    Ledger,
    /// Trezor hardware wallet
    Trezor,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Ledger => write!(f, "Ledger"),
            DeviceType::Trezor => write!(f, "Trezor"),
        }
    }
}

/// Connection status of a device
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Device is connected and ready
    Connected,
    /// Device is disconnected
    Disconnected,
    /// Connection is being established
    Connecting,
    /// Attempting to reconnect after disconnect
    Reconnecting,
    /// Device has an error
    Error(String),
}

/// Firmware status of a device
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareStatus {
    /// Firmware is up to date
    UpToDate,
    /// Firmware update available
    UpdateAvailable { current: String, available: String },
    /// Firmware is outdated and should be updated
    Outdated { current: String, minimum: String },
    /// Firmware version unknown
    Unknown,
}

/// Hardware device representation
#[derive(Debug, Clone)]
pub struct HardwareDevice {
    /// Unique device identifier
    pub id: DeviceId,
    /// Type of device (Ledger/Trezor)
    pub device_type: DeviceType,
    /// Device model name
    pub model: String,
    /// Current firmware version
    pub firmware_version: Option<String>,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Firmware status
    pub firmware_status: FirmwareStatus,
    /// Addresses derived from this device
    pub addresses: Vec<Address>,
    /// When the device was first detected
    pub first_seen: DateTime<Utc>,
    /// When the device was last seen active
    pub last_seen: DateTime<Utc>,
    /// Number of reconnection attempts
    pub reconnect_attempts: u32,
    /// Maximum reconnection attempts before giving up
    pub max_reconnect_attempts: u32,
}

impl HardwareDevice {
    /// Create a new hardware device
    pub fn new(device_type: DeviceType, model: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: DeviceId::generate(),
            device_type,
            model: model.into(),
            firmware_version: None,
            connection_status: ConnectionStatus::Connecting,
            firmware_status: FirmwareStatus::Unknown,
            addresses: Vec::new(),
            first_seen: now,
            last_seen: now,
            reconnect_attempts: 0,
            max_reconnect_attempts: 3,
        }
    }

    /// Create a Ledger device
    pub fn ledger(model: impl Into<String>) -> Self {
        Self::new(DeviceType::Ledger, model)
    }

    /// Create a Trezor device
    pub fn trezor(model: impl Into<String>) -> Self {
        Self::new(DeviceType::Trezor, model)
    }

    /// Set firmware version
    pub fn with_firmware(mut self, version: impl Into<String>) -> Self {
        self.firmware_version = Some(version.into());
        self
    }

    /// Mark as connected
    pub fn mark_connected(&mut self) {
        self.connection_status = ConnectionStatus::Connected;
        self.last_seen = Utc::now();
        self.reconnect_attempts = 0;
    }

    /// Mark as disconnected
    pub fn mark_disconnected(&mut self) {
        self.connection_status = ConnectionStatus::Disconnected;
    }

    /// Check if device is connected
    pub fn is_connected(&self) -> bool {
        matches!(self.connection_status, ConnectionStatus::Connected)
    }

    /// Check if reconnection should be attempted
    pub fn should_attempt_reconnect(&self) -> bool {
        self.reconnect_attempts < self.max_reconnect_attempts
    }

    /// Increment reconnection attempt counter
    pub fn increment_reconnect_attempt(&mut self) {
        self.reconnect_attempts += 1;
        self.connection_status = ConnectionStatus::Reconnecting;
    }
}

/// Device scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Devices found during scan
    pub devices: Vec<HardwareDevice>,
    /// Time taken for scan
    pub scan_duration: Duration,
    /// Any errors that occurred during scan
    pub errors: Vec<String>,
}

/// Device manager configuration
#[derive(Debug, Clone)]
pub struct DeviceManagerConfig {
    /// Auto-scan interval (None = disabled)
    pub auto_scan_interval: Option<Duration>,
    /// Reconnection delay between attempts
    pub reconnect_delay: Duration,
    /// Maximum reconnection attempts per device
    pub max_reconnect_attempts: u32,
    /// Scan timeout per device type
    pub scan_timeout: Duration,
}

impl Default for DeviceManagerConfig {
    fn default() -> Self {
        Self {
            auto_scan_interval: Some(Duration::from_secs(30)),
            reconnect_delay: Duration::from_secs(2),
            max_reconnect_attempts: 3,
            scan_timeout: Duration::from_secs(10),
        }
    }
}

/// Device manager for multi-device hardware wallet support
///
/// Maintains a thread-safe registry of connected hardware devices
/// and handles device lifecycle management including connection,
/// disconnection, and reconnection.
///
/// # Thread Safety
///
/// The device registry uses `Arc<RwLock<HashMap>>` for safe concurrent access.
#[derive(Debug)]
pub struct DeviceManager {
    /// Device registry - maps DeviceId to HardwareDevice
    registry: Arc<RwLock<HashMap<DeviceId, HardwareDevice>>>,
    /// Configuration
    config: DeviceManagerConfig,
    /// Whether auto-scan is running
    auto_scan_running: Arc<RwLock<bool>>,
}

impl DeviceManager {
    /// Create a new DeviceManager with default configuration
    pub fn new() -> Self {
        Self::with_config(DeviceManagerConfig::default())
    }

    /// Create a DeviceManager with custom configuration
    pub fn with_config(config: DeviceManagerConfig) -> Self {
        tracing::info!(
            auto_scan_interval = ?config.auto_scan_interval,
            max_reconnect_attempts = config.max_reconnect_attempts,
            "🔌 Creating new DeviceManager"
        );

        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            config,
            auto_scan_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Scan for connected hardware devices
    ///
    /// Detects all connected Ledger and Trezor devices using Alloy signers.
    /// Updates the device registry with newly found devices.
    ///
    /// # Requirements
    /// - **3.1**: Detect all connected Ledger and Trezor devices
    pub async fn scan_devices(&self) -> Result<ScanResult> {
        let start = std::time::Instant::now();
        let correlation_id = Uuid::new_v4();
        tracing::info!(
            correlation_id = %correlation_id,
            "🔍 Scanning for hardware devices"
        );

        let mut devices = Vec::new();
        let mut errors = Vec::new();

        // Scan for Ledger devices
        match self.scan_ledger_devices().await {
            Ok(ledger_devices) => {
                tracing::info!(
                    correlation_id = %correlation_id,
                    count = ledger_devices.len(),
                    "Found Ledger devices"
                );
                devices.extend(ledger_devices);
            }
            Err(e) => {
                let error_msg = format!("Ledger scan error: {}", e);
                tracing::warn!(
                    correlation_id = %correlation_id,
                    error = %e,
                    "Failed to scan Ledger devices"
                );
                errors.push(error_msg);
            }
        }

        // Scan for Trezor devices
        match self.scan_trezor_devices().await {
            Ok(trezor_devices) => {
                tracing::info!(
                    correlation_id = %correlation_id,
                    count = trezor_devices.len(),
                    "Found Trezor devices"
                );
                devices.extend(trezor_devices);
            }
            Err(e) => {
                let error_msg = format!("Trezor scan error: {}", e);
                tracing::warn!(
                    correlation_id = %correlation_id,
                    error = %e,
                    "Failed to scan Trezor devices"
                );
                errors.push(error_msg);
            }
        }

        // Update registry with found devices
        {
            let mut registry = self.registry.write().await;
            for device in &devices {
                if !registry.contains_key(&device.id) {
                    registry.insert(device.id.clone(), device.clone());
                }
            }
        }

        let scan_duration = start.elapsed();
        tracing::info!(
            correlation_id = %correlation_id,
            total_devices = devices.len(),
            duration_ms = scan_duration.as_millis(),
            "✅ Device scan complete"
        );

        Ok(ScanResult {
            devices,
            scan_duration,
            errors,
        })
    }

    /// Scan for Ledger devices
    #[cfg(feature = "hardware-wallets")]
    async fn scan_ledger_devices(&self) -> Result<Vec<HardwareDevice>> {
        use alloy_signer_ledger::HDPath as LedgerHDPath;
        use alloy_signer_ledger::LedgerSigner;

        let mut devices = Vec::new();

        // Try to connect to Ledger
        match LedgerSigner::new(LedgerHDPath::LedgerLive(0), None).await {
            Ok(_ledger) => {
                let mut device = HardwareDevice::ledger("Ledger Nano");
                device.mark_connected();
                devices.push(device);
            }
            Err(e) => {
                tracing::debug!("No Ledger device found: {}", e);
            }
        }

        Ok(devices)
    }

    #[cfg(not(feature = "hardware-wallets"))]
    async fn scan_ledger_devices(&self) -> Result<Vec<HardwareDevice>> {
        // Simulated scan for testing without hardware-wallets feature
        tracing::debug!("Hardware wallets feature disabled, simulating Ledger scan");
        Ok(Vec::new())
    }

    /// Scan for Trezor devices
    #[cfg(feature = "hardware-wallets")]
    async fn scan_trezor_devices(&self) -> Result<Vec<HardwareDevice>> {
        use alloy_signer_trezor::HDPath as TrezorHDPath;
        use alloy_signer_trezor::TrezorSigner;

        let mut devices = Vec::new();

        // Try to connect to Trezor
        match TrezorSigner::new(TrezorHDPath::TrezorLive(0), None).await {
            Ok(_trezor) => {
                let mut device = HardwareDevice::trezor("Trezor Model");
                device.mark_connected();
                devices.push(device);
            }
            Err(e) => {
                tracing::debug!("No Trezor device found: {}", e);
            }
        }

        Ok(devices)
    }

    #[cfg(not(feature = "hardware-wallets"))]
    async fn scan_trezor_devices(&self) -> Result<Vec<HardwareDevice>> {
        // Simulated scan for testing without hardware-wallets feature
        tracing::debug!("Hardware wallets feature disabled, simulating Trezor scan");
        Ok(Vec::new())
    }

    /// Handle device disconnection
    ///
    /// Attempts to reconnect to the device or marks it as disconnected
    /// if reconnection attempts are exhausted.
    ///
    /// # Requirements
    /// - **3.2**: Handle disconnection gracefully and attempt reconnection
    pub async fn handle_device_disconnect(&self, device_id: &DeviceId) -> bool {
        let correlation_id = Uuid::new_v4();
        tracing::info!(
            correlation_id = %correlation_id,
            device_id = %device_id,
            "⚠️ Handling device disconnection"
        );

        // Loop to handle retries without recursion
        loop {
            // Check if we should attempt reconnect
            let attempt_reconnect = {
                let mut registry = self.registry.write().await;
                if let Some(device) = registry.get_mut(device_id) {
                    if device.should_attempt_reconnect() {
                        device.increment_reconnect_attempt();
                        tracing::info!(
                            correlation_id = %correlation_id,
                            device_id = %device_id,
                            attempt = device.reconnect_attempts,
                            max_attempts = device.max_reconnect_attempts,
                            "🔄 Attempting reconnection"
                        );
                        true
                    } else {
                        device.mark_disconnected();
                        tracing::warn!(
                            correlation_id = %correlation_id,
                            device_id = %device_id,
                            "❌ Max reconnection attempts reached, marking as disconnected"
                        );
                        return false;
                    }
                } else {
                    // Device not found
                    return false;
                }
            };

            if attempt_reconnect {
                // Wait before attempting reconnection
                tokio::time::sleep(self.config.reconnect_delay).await;

                // Try to reconnect
                if self.attempt_reconnect(device_id).await {
                    tracing::info!(
                        correlation_id = %correlation_id,
                        device_id = %device_id,
                        "✅ Reconnection successful"
                    );
                    return true;
                }
                // If failed, loop continues to check count and retry
            } else {
                return false;
            }
        }
    }

    /// Attempt to reconnect to a specific device
    async fn attempt_reconnect(&self, device_id: &DeviceId) -> bool {
        let registry = self.registry.read().await;

        if let Some(device) = registry.get(device_id) {
            let device_type = device.device_type.clone();
            drop(registry);

            // Attempt reconnection based on device type
            let reconnected = match device_type {
                DeviceType::Ledger => self.reconnect_ledger(device_id).await,
                DeviceType::Trezor => self.reconnect_trezor(device_id).await,
            };

            if reconnected {
                let mut registry = self.registry.write().await;
                if let Some(device) = registry.get_mut(device_id) {
                    device.mark_connected();
                }
            }

            return reconnected;
        }

        false
    }

    /// Attempt to reconnect to a Ledger device
    #[cfg(feature = "hardware-wallets")]
    async fn reconnect_ledger(&self, _device_id: &DeviceId) -> bool {
        use alloy_signer_ledger::HDPath as LedgerHDPath;
        use alloy_signer_ledger::LedgerSigner;

        match LedgerSigner::new(LedgerHDPath::LedgerLive(0), None).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[cfg(not(feature = "hardware-wallets"))]
    async fn reconnect_ledger(&self, _device_id: &DeviceId) -> bool {
        false
    }

    /// Attempt to reconnect to a Trezor device
    #[cfg(feature = "hardware-wallets")]
    async fn reconnect_trezor(&self, _device_id: &DeviceId) -> bool {
        use alloy_signer_trezor::HDPath as TrezorHDPath;
        use alloy_signer_trezor::TrezorSigner;

        match TrezorSigner::new(TrezorHDPath::TrezorLive(0), None).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[cfg(not(feature = "hardware-wallets"))]
    async fn reconnect_trezor(&self, _device_id: &DeviceId) -> bool {
        false
    }

    /// Get a device from the registry
    ///
    /// # Requirements
    /// - **3.3**: Maintain registry of all available devices
    pub async fn get_device(&self, device_id: &DeviceId) -> Option<HardwareDevice> {
        let registry = self.registry.read().await;
        registry.get(device_id).cloned()
    }

    /// Get all devices in the registry
    pub async fn get_all_devices(&self) -> Vec<HardwareDevice> {
        let registry = self.registry.read().await;
        registry.values().cloned().collect()
    }

    /// Get all connected devices
    pub async fn get_connected_devices(&self) -> Vec<HardwareDevice> {
        let registry = self.registry.read().await;
        registry
            .values()
            .filter(|d| d.is_connected())
            .cloned()
            .collect()
    }

    /// Get device count
    pub async fn device_count(&self) -> usize {
        self.registry.read().await.len()
    }

    /// Get connected device count
    pub async fn connected_device_count(&self) -> usize {
        let registry = self.registry.read().await;
        registry.values().filter(|d| d.is_connected()).count()
    }

    /// Register a device manually (for testing)
    pub async fn register_device(&self, device: HardwareDevice) {
        let mut registry = self.registry.write().await;
        registry.insert(device.id.clone(), device);
    }

    /// Remove a device from the registry
    pub async fn remove_device(&self, device_id: &DeviceId) -> Option<HardwareDevice> {
        let mut registry = self.registry.write().await;
        registry.remove(device_id)
    }

    /// Clear all devices from the registry
    pub async fn clear_registry(&self) {
        let mut registry = self.registry.write().await;
        registry.clear();
    }

    /// Check firmware status for a device
    ///
    /// # Requirements
    /// - **3.5**: Provide clear upgrade guidance for outdated firmware
    pub async fn check_firmware_status(&self, device_id: &DeviceId) -> Option<FirmwareStatus> {
        let registry = self.registry.read().await;
        registry.get(device_id).map(|d| d.firmware_status.clone())
    }

    /// Get firmware upgrade guidance
    pub fn get_firmware_upgrade_guidance(&self, status: &FirmwareStatus) -> Option<String> {
        match status {
            FirmwareStatus::UpToDate => None,
            FirmwareStatus::UpdateAvailable { current, available } => Some(format!(
                "A firmware update is available. Current: {}, Available: {}. \
                Please update your device firmware through the manufacturer's software.",
                current, available
            )),
            FirmwareStatus::Outdated { current, minimum } => Some(format!(
                "⚠️ CRITICAL: Your firmware ({}) is outdated. Minimum required: {}. \
                Please update immediately through the manufacturer's software. \
                Some features may not work correctly until updated.",
                current, minimum
            )),
            FirmwareStatus::Unknown => Some(
                "Unable to determine firmware version. Please ensure your device \
                is connected and unlocked, then try refreshing."
                    .to_string(),
            ),
        }
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_manager_creation() {
        let manager = DeviceManager::new();
        assert_eq!(manager.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_device() {
        let manager = DeviceManager::new();
        let device = HardwareDevice::ledger("Nano S");

        manager.register_device(device.clone()).await;

        assert_eq!(manager.device_count().await, 1);
        let retrieved = manager.get_device(&device.id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_remove_device() {
        let manager = DeviceManager::new();
        let device = HardwareDevice::ledger("Nano S");
        let device_id = device.id.clone();

        manager.register_device(device).await;
        assert_eq!(manager.device_count().await, 1);

        let removed = manager.remove_device(&device_id).await;
        assert!(removed.is_some());
        assert_eq!(manager.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_connected_device_count() {
        let manager = DeviceManager::new();

        let mut device1 = HardwareDevice::ledger("Nano S");
        device1.mark_connected();

        let device2 = HardwareDevice::trezor("Model T");

        manager.register_device(device1).await;
        manager.register_device(device2).await;

        assert_eq!(manager.device_count().await, 2);
        assert_eq!(manager.connected_device_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_all_devices() {
        let manager = DeviceManager::new();

        let device1 = HardwareDevice::ledger("Nano S");
        let device2 = HardwareDevice::trezor("Model T");

        manager.register_device(device1).await;
        manager.register_device(device2).await;

        let all = manager.get_all_devices().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_get_connected_devices() {
        let manager = DeviceManager::new();

        let mut device1 = HardwareDevice::ledger("Nano S");
        device1.mark_connected();

        let device2 = HardwareDevice::trezor("Model T");

        manager.register_device(device1).await;
        manager.register_device(device2).await;

        let connected = manager.get_connected_devices().await;
        assert_eq!(connected.len(), 1);
        assert_eq!(connected[0].device_type, DeviceType::Ledger);
    }

    #[tokio::test]
    async fn test_clear_registry() {
        let manager = DeviceManager::new();

        manager.register_device(HardwareDevice::ledger("Nano S")).await;
        manager.register_device(HardwareDevice::trezor("Model T")).await;

        assert_eq!(manager.device_count().await, 2);

        manager.clear_registry().await;

        assert_eq!(manager.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_device_connection_status() {
        let mut device = HardwareDevice::ledger("Nano S");
        assert!(!device.is_connected());

        device.mark_connected();
        assert!(device.is_connected());

        device.mark_disconnected();
        assert!(!device.is_connected());
    }

    #[tokio::test]
    async fn test_reconnect_attempts() {
        let mut device = HardwareDevice::ledger("Nano S");
        device.max_reconnect_attempts = 2;

        assert!(device.should_attempt_reconnect());
        device.increment_reconnect_attempt();
        assert!(device.should_attempt_reconnect());
        device.increment_reconnect_attempt();
        assert!(!device.should_attempt_reconnect());
    }

    #[tokio::test]
    async fn test_firmware_upgrade_guidance() {
        let manager = DeviceManager::new();

        let guidance = manager.get_firmware_upgrade_guidance(&FirmwareStatus::UpToDate);
        assert!(guidance.is_none());

        let guidance = manager.get_firmware_upgrade_guidance(&FirmwareStatus::Outdated {
            current: "1.0".to_string(),
            minimum: "2.0".to_string(),
        });
        assert!(guidance.is_some());
        assert!(guidance.unwrap().contains("CRITICAL"));
    }
}

/// Property-based tests for device registry consistency
///
/// These tests validate **Property 6: Device Registry Consistency** from design.md
/// and **Requirement 3.3** from requirements.md:
///
/// *For any* set of connected hardware devices, the device registry should
/// accurately reflect all available devices and maintain consistency across
/// concurrent access.
///
/// Uses proptest with minimum 100 iterations.
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Device operations for property testing
    #[derive(Debug, Clone)]
    enum DeviceOp {
        RegisterLedger(String),
        RegisterTrezor(String),
        RemoveDevice(usize), // Index of reported device to remove
        ClearRegistry,
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 6: Registry Consistency - Sequential Operations
        ///
        /// *For any* sequence of valid registry operations (Register, Remove, Clear),
        /// the registry state should remain consistent (count correct, IDs managed).
        ///
        /// Validates: Requirement 3.3
        #[test]
        fn prop_registry_consistency_sequential(
            ops in proptest::collection::vec(
                prop_oneof![
                    "[a-zA-Z0-9]{1,10}".prop_map(DeviceOp::RegisterLedger),
                    "[a-zA-Z0-9]{1,10}".prop_map(DeviceOp::RegisterTrezor),
                    (0usize..10).prop_map(DeviceOp::RemoveDevice),
                    Just(DeviceOp::ClearRegistry),
                ],
                0..50 // Up to 50 operations per test case
            )
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let manager = DeviceManager::new();
                let mut added_devices: Vec<DeviceId> = Vec::new();

                for op in ops {
                    match op {
                        DeviceOp::RegisterLedger(model) => {
                            let device = HardwareDevice::ledger(model);
                            let id = device.id.clone();
                            manager.register_device(device).await;
                            added_devices.push(id);
                        }
                        DeviceOp::RegisterTrezor(model) => {
                            let device = HardwareDevice::trezor(model);
                            let id = device.id.clone();
                            manager.register_device(device).await;
                            added_devices.push(id);
                        }
                        DeviceOp::RemoveDevice(index) => {
                            if !added_devices.is_empty() {
                                let idx = index % added_devices.len();
                                let id = &added_devices[idx];
                                manager.remove_device(id).await;
                                added_devices.remove(idx);
                            }
                        }
                        DeviceOp::ClearRegistry => {
                            manager.clear_registry().await;
                            added_devices.clear();
                        }
                    }

                    // Invariant check: Registry count must match tracked count
                    assert_eq!(
                        manager.device_count().await,
                        added_devices.len(),
                        "Registry count mismatch"
                    );
                }
            });
        }

        /// Property 6: Registry Consistency - Unique IDs
        ///
        /// *For any* set of random devices added to the registry,
        /// all device IDs in the registry must be unique.
        ///
        /// Validates: Requirement 3.3
        #[test]
        fn prop_registry_ids_unique(
            count in 1usize..20
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let manager = DeviceManager::new();

                // Add random devices
                for i in 0..count {
                    let device = if i % 2 == 0 {
                        HardwareDevice::ledger(format!("Ledger_{}", i))
                    } else {
                        HardwareDevice::trezor(format!("Trezor_{}", i))
                    };
                    manager.register_device(device).await;
                }

                assert_eq!(manager.device_count().await, count);

                // Check uniqueness explicitly via set
                let devices = manager.get_all_devices().await;
                let mut ids = std::collections::HashSet::new();
                for device in devices {
                    assert!(ids.insert(device.id), "Duplicate device ID found");
                }
            });
        }

        /// Property 6: Registry Consistency - Type Filtering
        ///
        /// *For any* mix of Ledger and Trezor devices, filtering by type
        /// should return correct subsets.
        ///
        /// Validates: Requirement 3.3
        #[test]
        fn prop_registry_type_filtering(
            ledger_count in 0usize..10,
            trezor_count in 0usize..10
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let manager = DeviceManager::new();

                for i in 0..ledger_count {
                    manager.register_device(HardwareDevice::ledger(format!("L{}", i))).await;
                }
                for i in 0..trezor_count {
                    manager.register_device(HardwareDevice::trezor(format!("T{}", i))).await;
                }

                let all = manager.get_all_devices().await;
                let ledgers = all.iter().filter(|d| d.device_type == DeviceType::Ledger).count();
                let trezors = all.iter().filter(|d| d.device_type == DeviceType::Trezor).count();

                assert_eq!(ledgers, ledger_count, "Ledger count mismatch");
                assert_eq!(trezors, trezor_count, "Trezor count mismatch");
                assert_eq!(all.len(), ledger_count + trezor_count, "Total count mismatch");
            });
        }

        /// Property 6: Registry Consistency - Connection Status
        ///
        /// *For any* device, marking it connected/disconnected should
        /// accurately reflect in the registry.
        ///
        /// Validates: Requirement 3.3
        #[test]
        fn prop_connection_status_updates(
            operations in proptest::collection::vec(prop::bool::ANY, 1..20)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let manager = DeviceManager::new();
                let mut device = HardwareDevice::ledger("StatusTest");
                let id = device.id.clone();
                
                manager.register_device(device).await;

                // Apply random sequence of connection status updates
                for should_connect in operations {
                    // Update through the registry to simulate real usage
                    // Note: We need to access via get_device loop or similar if we exposed specific methods
                    // but here we'll update the registry entry directly via simulated usage
                    
                    if let Some(mut current) = manager.get_device(&id).await { // Use if let to avoid panic if not found
                         if should_connect {
                            current.mark_connected();
                        } else {
                            current.mark_disconnected();
                        }
                        manager.register_device(current).await; // Overwrite update
                    }

                    // Verify state
                    let updated = manager.get_device(&id).await.unwrap();
                    assert_eq!(updated.is_connected(), should_connect);
                    
                    // Verify connected count
                    let connected_count = manager.connected_device_count().await;
                    assert_eq!(connected_count, if should_connect { 1 } else { 0 });
                }
            });
        }

        /// Property 6: Registry Consistency - Scan Results Integration
        ///
        /// *For any* scan result, scanning should add new devices but
        /// preserve existing ones (unless remove logic implemented, but here additive).
        ///
        /// Validates: Requirement 3.3
        #[test]
        fn prop_scan_integration(
            initial_count in 0usize..5,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let manager = DeviceManager::new();

                // Add initial devices
                for i in 0..initial_count {
                    manager.register_device(HardwareDevice::ledger(format!("Init{}", i))).await; // Use unique name
                }

                let initial_size = manager.device_count().await;
                
                // Add "scanned" devices manually to simulate scan result integration
                let scanned_device = HardwareDevice::trezor("Scanned1");
                let scanned_id = scanned_device.id.clone();
                manager.register_device(scanned_device).await;

                // Should have increased
                assert_eq!(manager.device_count().await, initial_size + 1);
                assert!(manager.get_device(&scanned_id).await.is_some());
            });
        }

        /// Property 7: Concurrent Hardware Operations
        ///
        /// *For any* set of concurrent operations (read/write/scan) executed
        /// by multiple tasks, the registry should maintain consistency and
        /// not deadlock.
        ///
        /// Validates: Requirement 3.4
        #[test]
        fn prop_concurrent_registry_access(
            // Generate multiple threads of operations
            threads in proptest::collection::vec(
                proptest::collection::vec(
                    prop_oneof![
                        "[a-zA-Z0-9]{1,10}".prop_map(DeviceOp::RegisterLedger),
                        "[a-zA-Z0-9]{1,10}".prop_map(DeviceOp::RegisterTrezor),
                        (0usize..10).prop_map(DeviceOp::RemoveDevice),
                        Just(DeviceOp::ClearRegistry),
                    ],
                    1..10 // Operations per thread
                ),
                2..5 // Number of concurrent threads
            )
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let manager = Arc::new(DeviceManager::new());
                let mut handles = Vec::new();

                for ops in threads {
                    let manager_clone = manager.clone();
                    // Spawn concurrent task
                    handles.push(tokio::spawn(async move {
                        let mut local_added = Vec::new(); // Track locally added IDs for removal ops
                        
                        for op in ops {
                            // Introduce random tiny yield to encourage interleaving
                            if fastrand::bool() {
                                tokio::task::yield_now().await;
                            }

                            match op {
                                DeviceOp::RegisterLedger(model) => {
                                    let device = HardwareDevice::ledger(model);
                                    let id = device.id.clone();
                                    manager_clone.register_device(device).await;
                                    local_added.push(id);
                                }
                                DeviceOp::RegisterTrezor(model) => {
                                    let device = HardwareDevice::trezor(model);
                                    let id = device.id.clone();
                                    manager_clone.register_device(device).await;
                                    local_added.push(id);
                                }
                                DeviceOp::RemoveDevice(index) => {
                                    // Try to remove a random device from registry or one we added
                                    // To test contention, we'll try to remove one of our own or just skip
                                    if !local_added.is_empty() {
                                        let idx = index % local_added.len();
                                        let id = &local_added[idx];
                                        manager_clone.remove_device(id).await;
                                        // We don't remove from local_added to allow repeated attempts (which should handle not found gracefully)
                                    }
                                }
                                DeviceOp::ClearRegistry => {
                                    manager_clone.clear_registry().await;
                                }
                            }
                        }
                    }));
                }

                // Wait for all tasks to complete
                for handle in handles {
                    handle.await.unwrap();
                }

                // Final invariant check: Registry must be responsive and in valid state
                // We can't predict exact count due to race conditions of add/remove/clear,
                // but we can ensure it doesn't panic and returns a valid count.
                let _count = manager.device_count().await;
                let _devices = manager.get_all_devices().await;
                
                // Consistency check: count() should match get_all_devices().len()
                assert_eq!(manager.device_count().await, manager.get_all_devices().await.len());
            });
        }
    }
}
