//! Hardware Wallet User Feedback System
//!
//! This module provides comprehensive user feedback for hardware wallet operations,
//! including progress tracking, status updates, and user guidance.

use crate::error::HardwareWalletError;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Hardware wallet operation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardwareWalletStatus {
    /// Device is disconnected
    Disconnected,
    /// Searching for devices
    Searching,
    /// Device detected but not connected
    Detected,
    /// Connecting to device
    Connecting,
    /// Device connected and ready
    Connected,
    /// Device is locked (PIN required)
    Locked,
    /// Waiting for user confirmation on device
    AwaitingConfirmation,
    /// Operation in progress
    Processing { operation: String },
    /// Operation completed successfully
    Completed,
    /// Error occurred
    Error { error: HardwareWalletError },
}

/// User guidance messages for hardware wallet operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGuidance {
    pub title: String,
    pub message: String,
    pub steps: Vec<String>,
    pub urgency: GuidanceUrgency,
}

/// Urgency level for user guidance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GuidanceUrgency {
    Low,
    Medium,
    High,
    Critical,
}

/// Hardware wallet feedback manager
#[derive(Debug)]
pub struct HardwareWalletFeedback {
    status: HardwareWalletStatus,
    last_update: Instant,
    operation_start: Option<Instant>,
    timeout_duration: Duration,
}

impl Default for HardwareWalletFeedback {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareWalletFeedback {
    /// Create a new feedback manager
    pub fn new() -> Self {
        Self {
            status: HardwareWalletStatus::Disconnected,
            last_update: Instant::now(),
            operation_start: None,
            timeout_duration: Duration::from_secs(30),
        }
    }

    /// Update the status
    pub fn update_status(&mut self, status: HardwareWalletStatus) {
        self.status = status;
        self.last_update = Instant::now();

        // Start timing for operations
        if matches!(
            self.status,
            HardwareWalletStatus::Processing { .. } | HardwareWalletStatus::Connecting
        ) {
            self.operation_start = Some(Instant::now());
        } else if matches!(
            self.status,
            HardwareWalletStatus::Completed | HardwareWalletStatus::Error { .. }
        ) {
            self.operation_start = None;
        }
    }

    /// Get current status
    pub fn status(&self) -> &HardwareWalletStatus {
        &self.status
    }

    /// Check if operation has timed out
    pub fn is_timeout(&self) -> bool {
        if let Some(start) = self.operation_start {
            start.elapsed() > self.timeout_duration
        } else {
            false
        }
    }

    /// Get user guidance based on current status
    pub fn get_guidance(&self) -> UserGuidance {
        match &self.status {
            HardwareWalletStatus::Disconnected => UserGuidance {
                title: "Connect Hardware Wallet".to_string(),
                message: "No hardware wallet detected. Please connect your device.".to_string(),
                steps: vec![
                    "Connect your hardware wallet via USB".to_string(),
                    "Unlock your device if it's locked".to_string(),
                    "Make sure the device is in the correct mode".to_string(),
                ],
                urgency: GuidanceUrgency::High,
            },

            HardwareWalletStatus::Searching => UserGuidance {
                title: "Searching for Devices".to_string(),
                message: "Looking for connected hardware wallets...".to_string(),
                steps: vec![
                    "Please wait while we scan for devices".to_string(),
                    "Make sure your device is connected and unlocked".to_string(),
                ],
                urgency: GuidanceUrgency::Low,
            },

            HardwareWalletStatus::Detected => UserGuidance {
                title: "Device Detected".to_string(),
                message: "Hardware wallet found. Establishing connection...".to_string(),
                steps: vec![
                    "Device detected successfully".to_string(),
                    "Attempting to connect...".to_string(),
                ],
                urgency: GuidanceUrgency::Low,
            },

            HardwareWalletStatus::Connecting => UserGuidance {
                title: "Connecting".to_string(),
                message: "Connecting to your hardware wallet...".to_string(),
                steps: vec![
                    "Please wait while we establish connection".to_string(),
                    "Make sure no other applications are using the device".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },

            HardwareWalletStatus::Connected => UserGuidance {
                title: "Connected".to_string(),
                message: "Successfully connected to hardware wallet.".to_string(),
                steps: vec![
                    "Your device is ready for operations".to_string(),
                    "You can now proceed with transactions".to_string(),
                ],
                urgency: GuidanceUrgency::Low,
            },

            HardwareWalletStatus::Locked => UserGuidance {
                title: "Device Locked".to_string(),
                message: "Your hardware wallet is locked. Please unlock it.".to_string(),
                steps: vec![
                    "Enter your PIN on the hardware wallet".to_string(),
                    "Follow the prompts on your device screen".to_string(),
                    "Make sure the device screen is active".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },

            HardwareWalletStatus::AwaitingConfirmation => UserGuidance {
                title: "Confirm on Device".to_string(),
                message: "Please confirm the operation on your hardware wallet.".to_string(),
                steps: vec![
                    "Look at your hardware wallet screen".to_string(),
                    "Review the operation details".to_string(),
                    "Press the confirm button to proceed".to_string(),
                ],
                urgency: GuidanceUrgency::High,
            },

            HardwareWalletStatus::Processing { operation } => UserGuidance {
                title: "Processing".to_string(),
                message: format!("Processing operation: {operation}"),
                steps: vec![
                    "Please wait while the operation completes".to_string(),
                    "Do not disconnect your device".to_string(),
                    "Keep your device screen active".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },

            HardwareWalletStatus::Completed => UserGuidance {
                title: "Operation Complete".to_string(),
                message: "Operation completed successfully!".to_string(),
                steps: vec![
                    "Your operation has been completed".to_string(),
                    "You can safely disconnect if needed".to_string(),
                ],
                urgency: GuidanceUrgency::Low,
            },

            HardwareWalletStatus::Error { error } => self.get_error_guidance(error),
        }
    }

    /// Get specific guidance for errors
    fn get_error_guidance(&self, error: &HardwareWalletError) -> UserGuidance {
        match error {
            HardwareWalletError::DeviceNotFound => UserGuidance {
                title: "Device Not Found".to_string(),
                message: "Hardware wallet not detected.".to_string(),
                steps: vec![
                    "Check USB connection".to_string(),
                    "Try a different USB port or cable".to_string(),
                    "Restart your hardware wallet".to_string(),
                    "Install device drivers if needed".to_string(),
                ],
                urgency: GuidanceUrgency::High,
            },

            HardwareWalletError::DeviceLocked => UserGuidance {
                title: "Device Locked".to_string(),
                message: "Please unlock your hardware wallet.".to_string(),
                steps: vec![
                    "Enter your PIN on the device".to_string(),
                    "Make sure the screen is active".to_string(),
                    "Follow device prompts".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },

            HardwareWalletError::UserCancelled => UserGuidance {
                title: "Operation Cancelled".to_string(),
                message: "You cancelled the operation.".to_string(),
                steps: vec![
                    "You can try the operation again".to_string(),
                    "Make sure to confirm when prompted".to_string(),
                ],
                urgency: GuidanceUrgency::Low,
            },

            HardwareWalletError::AppNotOpen { app } => UserGuidance {
                title: "App Required".to_string(),
                message: format!("Please open the {app} app on your device."),
                steps: vec![
                    format!("Navigate to the {} app", app),
                    "Open the app and wait for it to load".to_string(),
                    "Ensure the app is ready and active".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },

            HardwareWalletError::BlindSigningDisabled => UserGuidance {
                title: "Enable Blind Signing".to_string(),
                message: "Blind signing is required for this operation.".to_string(),
                steps: vec![
                    "Open settings on your hardware wallet".to_string(),
                    "Find 'Blind Signing' or 'Contract Data' option".to_string(),
                    "Enable the setting".to_string(),
                    "Restart the Ethereum app".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },

            HardwareWalletError::CommunicationError => UserGuidance {
                title: "Communication Error".to_string(),
                message: "Unable to communicate with device.".to_string(),
                steps: vec![
                    "Check USB cable connection".to_string(),
                    "Close other applications using the device".to_string(),
                    "Restart the hardware wallet".to_string(),
                    "Try a different USB port".to_string(),
                ],
                urgency: GuidanceUrgency::High,
            },

            _ => UserGuidance {
                title: "Hardware Wallet Error".to_string(),
                message: format!("An error occurred: {error}"),
                steps: vec![
                    "Try the operation again".to_string(),
                    "Check device connection".to_string(),
                    "Restart the application if needed".to_string(),
                    "Contact support if the issue persists".to_string(),
                ],
                urgency: GuidanceUrgency::Medium,
            },
        }
    }

    /// Get progress percentage for operations
    pub fn progress_percentage(&self) -> Option<u8> {
        match &self.status {
            HardwareWalletStatus::Searching => Some(10),
            HardwareWalletStatus::Detected => Some(25),
            HardwareWalletStatus::Connecting => Some(50),
            HardwareWalletStatus::Connected => Some(75),
            HardwareWalletStatus::Processing { .. } => Some(85),
            HardwareWalletStatus::AwaitingConfirmation => Some(90),
            HardwareWalletStatus::Completed => Some(100),
            _ => None,
        }
    }

    /// Check if status indicates an active operation
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            HardwareWalletStatus::Searching
                | HardwareWalletStatus::Connecting
                | HardwareWalletStatus::Processing { .. }
                | HardwareWalletStatus::AwaitingConfirmation
        )
    }

    /// Get time elapsed since operation started
    pub fn operation_duration(&self) -> Option<Duration> {
        self.operation_start.map(|start| start.elapsed())
    }

    /// Set custom timeout duration
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }
}

/// Helper function to create status updates for common operations
pub fn create_operation_status(operation: &str) -> HardwareWalletStatus {
    HardwareWalletStatus::Processing {
        operation: operation.to_string(),
    }
}

/// Helper function to create error status
pub fn create_error_status(error: HardwareWalletError) -> HardwareWalletStatus {
    HardwareWalletStatus::Error { error }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_manager_creation() {
        let feedback = HardwareWalletFeedback::new();
        assert!(matches!(feedback.status(), HardwareWalletStatus::Disconnected));
        assert!(!feedback.is_active());
    }

    #[test]
    fn test_status_updates() {
        let mut feedback = HardwareWalletFeedback::new();

        feedback.update_status(HardwareWalletStatus::Searching);
        assert!(matches!(feedback.status(), HardwareWalletStatus::Searching));
        assert!(feedback.is_active());

        feedback.update_status(HardwareWalletStatus::Connected);
        assert!(matches!(feedback.status(), HardwareWalletStatus::Connected));
        assert!(!feedback.is_active());
    }

    #[test]
    fn test_guidance_generation() {
        let feedback = HardwareWalletFeedback::new();
        let guidance = feedback.get_guidance();

        assert_eq!(guidance.title, "Connect Hardware Wallet");
        assert_eq!(guidance.urgency, GuidanceUrgency::High);
        assert!(!guidance.steps.is_empty());
    }

    #[test]
    fn test_progress_tracking() {
        let mut feedback = HardwareWalletFeedback::new();

        feedback.update_status(HardwareWalletStatus::Searching);
        assert_eq!(feedback.progress_percentage(), Some(10));

        feedback.update_status(HardwareWalletStatus::Completed);
        assert_eq!(feedback.progress_percentage(), Some(100));
    }
}
