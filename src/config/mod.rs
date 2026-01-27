//! Configuration Management Module
//!
//! This module handles loading and managing configuration files
//! that have been migrated from the Python version of Vaughan.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// Configuration submodules
pub mod api_config;

/// Main configuration manager
#[derive(Debug)]
pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        let config_dir = Self::get_config_dir();
        Self { config_dir }
    }

    /// Get the configuration directory path
    pub fn get_config_dir() -> PathBuf {
        // Use current working directory + config
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("config")
    }

    /// Load a configuration file
    pub fn load_config<T>(&self, filename: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config_path = self.config_dir.join(filename);

        let contents = fs::read_to_string(&config_path).map_err(|e| {
            crate::error::VaughanError::Configuration(crate::error::ConfigurationError::ParseError {
                message: format!("Failed to read config file {filename}: {e}"),
            })
        })?;

        serde_json::from_str(&contents).map_err(|e| {
            crate::error::VaughanError::Configuration(crate::error::ConfigurationError::ParseError {
                message: format!("Failed to parse config file {filename}: {e}"),
            })
        })
    }

    /// Save a configuration file
    pub fn save_config<T>(&self, filename: &str, config: &T) -> Result<()>
    where
        T: Serialize,
    {
        // Ensure config directory exists
        fs::create_dir_all(&self.config_dir).map_err(|e| {
            crate::error::VaughanError::Configuration(crate::error::ConfigurationError::ParseError {
                message: format!("Failed to create config directory: {e}"),
            })
        })?;

        let config_path = self.config_dir.join(filename);
        let contents = serde_json::to_string_pretty(config).map_err(|e| {
            crate::error::VaughanError::Configuration(crate::error::ConfigurationError::ParseError {
                message: format!("Failed to serialize config: {e}"),
            })
        })?;

        fs::write(&config_path, contents).map_err(|e| {
            crate::error::VaughanError::Configuration(crate::error::ConfigurationError::ParseError {
                message: format!("Failed to write config file {filename}: {e}"),
            })
        })?;

        tracing::info!("Saved configuration: {}", filename);
        Ok(())
    }

    /// Check if a configuration file exists
    pub fn config_exists(&self, filename: &str) -> bool {
        self.config_dir.join(filename).exists()
    }

    /// List all configuration files
    pub fn list_configs(&self) -> Vec<String> {
        let mut configs = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.config_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".json") {
                                configs.push(filename.to_string());
                            }
                        }
                    }
                }
            }
        }

        configs.sort();
        configs
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Network configuration for EVM-compatible chains
///
/// Defines connection parameters and metadata for blockchain networks
/// like Ethereum, PulseChain, BSC, Polygon, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Unique identifier for the network
    pub id: String,
    /// Human-readable network name (e.g., "Ethereum Mainnet")
    pub name: String,
    /// EIP-155 chain ID
    pub chain_id: u64,
    /// RPC endpoint URL for network communication
    pub rpc_url: String,
    /// Native currency symbol (e.g., "ETH", "PLS")
    pub native_currency: String,
    /// Block explorer URL (e.g., Etherscan)
    pub explorer_url: String,
    /// UI color theme for the network
    pub color: String,
    /// Whether this is a testnet
    pub is_testnet: bool,
    /// Whether this is a user-added custom network
    pub is_custom: bool,
}

/// Networks configuration file structure
///
/// Contains all configured networks and the default network selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworksConfig {
    /// List of all configured networks
    pub networks: Vec<NetworkConfig>,
    /// ID of the default network to use
    pub default_network: String,
    /// Configuration file version
    pub version: String,
}

/// Custom ERC-20 token configuration
///
/// Represents a user-added custom token with metadata and verification status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToken {
    /// Token contract address
    pub address: String,
    /// Token name (e.g., "Wrapped Ether")
    pub name: String,
    /// Token symbol (e.g., "WETH")
    pub symbol: String,
    /// Number of decimal places
    pub decimals: u8,
    /// Unix timestamp when token was added
    pub added_timestamp: f64,
    /// Whether the token contract has been verified
    pub verified: bool,
    /// User-defined tags for organization
    pub tags: Vec<String>,
}

/// Custom tokens configuration file structure
///
/// Stores all user-added custom tokens organized by network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTokensConfig {
    /// Configuration file version
    pub version: String,
    /// Last update timestamp
    pub last_updated: String,
    /// Tokens mapped by network ID
    pub tokens: std::collections::HashMap<String, Vec<CustomToken>>,
}

/// User interface preferences
///
/// Configures the wallet's appearance and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    /// UI theme ("light" or "dark")
    pub theme: String,
    /// Interface language code (e.g., "en", "es")
    pub language: String,
    /// Whether to show testnet networks in the UI
    pub show_test_networks: bool,
    /// Auto-refresh interval in seconds
    pub auto_refresh_interval: u32,
    /// Hide balances below a threshold
    pub hide_small_balances: bool,
    /// Default gas fee mode ("slow", "normal", "fast")
    pub default_gas_mode: String,
    /// Path to custom alert sound file
    pub custom_alert_sound_path: Option<String>,
}

/// Security settings
///
/// Configures wallet security features and authentication requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub auto_lock_minutes: u32,
    pub require_password_for_transactions: bool,
    pub show_seed_phrase_warning: bool,
}

/// Migration note structure (if present)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationNote {
    pub seed_phrase_found: bool,
    pub derivation_path: String,
    pub migration_date: String,
}

/// User settings configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsConfig {
    pub version: String,
    pub current_network: String,
    pub ui_preferences: UiPreferences,
    pub security: SecuritySettings,
    pub last_updated: String,
    pub migration_note: Option<MigrationNote>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let config_manager = ConfigManager::new();
        assert!(config_manager.config_dir.exists() || config_manager.config_dir.to_str().unwrap().contains("config"));
    }

    #[test]
    fn test_get_config_dir() {
        let config_dir = ConfigManager::get_config_dir();
        assert!(config_dir.to_str().unwrap().ends_with("config"));
    }
}
