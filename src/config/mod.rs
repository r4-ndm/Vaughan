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

/// Network configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub id: String,
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub native_currency: String,
    pub explorer_url: String,
    pub color: String,
    pub is_testnet: bool,
    pub is_custom: bool,
}

/// Networks configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworksConfig {
    pub networks: Vec<NetworkConfig>,
    pub default_network: String,
    pub version: String,
}

/// Custom token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub added_timestamp: f64,
    pub verified: bool,
    pub tags: Vec<String>,
}

/// Custom tokens configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTokensConfig {
    pub version: String,
    pub last_updated: String,
    pub tokens: std::collections::HashMap<String, Vec<CustomToken>>,
}

/// User settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: String,
    pub language: String,
    pub show_test_networks: bool,
    pub auto_refresh_interval: u32,
    pub hide_small_balances: bool,
    pub default_gas_mode: String,
    pub custom_alert_sound_path: Option<String>,
}

/// Security settings structure
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
