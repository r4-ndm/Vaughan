//! API Configuration management for external services
//!
//! This module handles secure storage and management of API keys
//! and configuration for external services like Moralis

use crate::error::{ConfigurationError, Result, VaughanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
/// Configuration for external API services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    /// Moralis Web3 API configuration
    pub moralis: Option<MoralisConfig>,

    /// Custom API endpoints (for future extensibility)
    pub custom_apis: HashMap<String, CustomApiConfig>,
}

/// Moralis API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralisConfig {
    /// API key for Moralis
    pub api_key: String,

    /// Base URL (usually default)
    pub base_url: Option<String>,

    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Rate limiting settings
    pub rate_limit: Option<RateLimitConfig>,

    /// Whether to enable for price feeds
    pub enable_price_feeds: bool,
}

/// Custom API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomApiConfig {
    /// Base URL for the API
    pub base_url: String,

    /// API key/token
    pub api_key: Option<String>,

    /// Headers to include with requests
    pub headers: HashMap<String, String>,

    /// Description of what this API is for
    pub description: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,

    /// Burst allowance
    pub burst_size: u32,
}

impl Default for MoralisConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: Some("https://deep-index.moralis.io/api/v2".to_string()),
            timeout_seconds: Some(30),
            rate_limit: Some(RateLimitConfig {
                requests_per_minute: 100, // Free tier default
                burst_size: 10,
            }),
            enable_price_feeds: false,
        }
    }
}

/// API configuration manager
pub struct ApiConfigManager {
    config_path: PathBuf,
    config: ApiConfig,
}

impl ApiConfigManager {
    /// Create new API config manager
    pub fn new<P: AsRef<Path>>(config_dir: P) -> Result<Self> {
        let config_path = config_dir.as_ref().join("api_config.toml");
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            info!("API config file not found, creating default configuration");
            ApiConfig::default()
        };

        Ok(Self { config_path, config })
    }

    /// Load configuration from file
    fn load_config(path: &Path) -> Result<ApiConfig> {
        let content = fs::read_to_string(path).map_err(|e| {
            VaughanError::Configuration(ConfigurationError::ParseError {
                message: format!("Failed to read API config: {e}"),
            })
        })?;

        toml::from_str(&content).map_err(|e| {
            VaughanError::Configuration(ConfigurationError::ParseError {
                message: format!("Failed to parse API config: {e}"),
            })
        })
    }

    /// Save configuration to file
    pub fn save_config(&self) -> Result<()> {
        let toml_content = toml::to_string_pretty(&self.config).map_err(|e| {
            VaughanError::Configuration(ConfigurationError::ParseError {
                message: format!("Failed to serialize API config: {e}"),
            })
        })?;

        // Create directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                VaughanError::Configuration(ConfigurationError::ParseError {
                    message: format!("Failed to create config directory: {e}"),
                })
            })?;
        }

        fs::write(&self.config_path, toml_content).map_err(|e| {
            VaughanError::Configuration(ConfigurationError::ParseError {
                message: format!("Failed to write API config: {e}"),
            })
        })?;

        info!("API configuration saved to: {:?}", self.config_path);
        Ok(())
    }

    /// Set Moralis API key
    pub fn set_moralis_api_key(&mut self, api_key: String) -> Result<()> {
        if api_key.trim().is_empty() {
            return Err(VaughanError::Configuration(ConfigurationError::ValidationFailed {
                reason: "Moralis API key cannot be empty".to_string(),
            }));
        }

        // Initialize Moralis config if it doesn't exist
        if self.config.moralis.is_none() {
            self.config.moralis = Some(MoralisConfig::default());
        }

        if let Some(moralis_config) = &mut self.config.moralis {
            moralis_config.api_key = api_key;
        }

        self.save_config()?;
        info!("Moralis API key updated successfully");
        Ok(())
    }

    /// Get Moralis API key
    pub fn get_moralis_api_key(&self) -> Option<String> {
        self.config.moralis.as_ref().map(|m| m.api_key.clone())
    }

    /// Get full Moralis configuration
    pub fn get_moralis_config(&self) -> Option<&MoralisConfig> {
        self.config.moralis.as_ref()
    }

    /// Add custom API configuration
    pub fn add_custom_api(&mut self, name: String, config: CustomApiConfig) -> Result<()> {
        self.config.custom_apis.insert(name.clone(), config);
        self.save_config()?;
        info!("Added custom API configuration: {}", name);
        Ok(())
    }

    /// Remove custom API configuration
    pub fn remove_custom_api(&mut self, name: &str) -> Result<bool> {
        let removed = self.config.custom_apis.remove(name).is_some();
        if removed {
            self.save_config()?;
            info!("Removed custom API configuration: {}", name);
        }
        Ok(removed)
    }

    /// Check if Moralis is enabled for DEX validation
    pub fn is_moralis_enabled_for_dex(&self) -> bool {
        self.config
            .moralis
            .as_ref()
            .map(|m| !m.api_key.is_empty())
            .unwrap_or(false)
    }

    /// Get all configured APIs summary
    pub fn get_api_summary(&self) -> ApiSummary {
        ApiSummary {
            moralis_configured: self
                .config
                .moralis
                .as_ref()
                .map(|m| !m.api_key.is_empty())
                .unwrap_or(false),
            moralis_dex_enabled: self.is_moralis_enabled_for_dex(),
            moralis_price_enabled: false, // Price API not configured
            custom_apis: self.config.custom_apis.keys().cloned().collect(),
        }
    }

    /// Validate all API configurations
    pub async fn validate_all_apis(&self) -> ValidationReport {
        let mut report = ValidationReport {
            moralis: None,
            custom_apis: HashMap::new(),
        };

        // Validate Moralis if configured
        if let Some(moralis_config) = &self.config.moralis {
            if !moralis_config.api_key.is_empty() {
                report.moralis = Some(self.validate_moralis_api(moralis_config).await);
            }
        }

        report
    }

    /// Validate Moralis API key
    async fn validate_moralis_api(&self, config: &MoralisConfig) -> ApiValidationResult {
        // Simple API key validation - just check if it's not empty and has reasonable length
        if config.api_key.is_empty() {
            ApiValidationResult {
                success: false,
                message: "Moralis API key is empty".to_string(),
                response_time_ms: None,
            }
        } else if config.api_key.len() < 20 {
            ApiValidationResult {
                success: false,
                message: "Moralis API key appears to be too short".to_string(),
                response_time_ms: None,
            }
        } else {
            // For now, assume the key is valid if it has reasonable length
            // Note: Actual API connection test requires MoralisClient integration
            ApiValidationResult {
                success: true,
                message: "Moralis API key format appears valid (connection not tested)".to_string(),
                response_time_ms: None,
            }
        }
    }

    /// Create a template configuration file
    pub fn create_template_config(&self) -> Result<()> {
        let template_path = self.config_path.with_extension("template.toml");

        let template_config = ApiConfig {
            moralis: Some(MoralisConfig {
                api_key: "YOUR_MORALIS_API_KEY_HERE".to_string(),
                ..MoralisConfig::default()
            }),
            custom_apis: [(
                "example_api".to_string(),
                CustomApiConfig {
                    base_url: "https://api.example.com".to_string(),
                    api_key: Some("YOUR_API_KEY_HERE".to_string()),
                    headers: [("User-Agent".to_string(), "Vaughan-Wallet/1.0".to_string())]
                        .into_iter()
                        .collect(),
                    description: "Example custom API configuration".to_string(),
                },
            )]
            .into_iter()
            .collect(),
        };

        let template_content = toml::to_string_pretty(&template_config).map_err(|e| {
            VaughanError::Configuration(ConfigurationError::ParseError {
                message: format!("Failed to serialize template: {e}"),
            })
        })?;

        fs::write(&template_path, template_content).map_err(|e| {
            VaughanError::Configuration(ConfigurationError::ParseError {
                message: format!("Failed to write template: {e}"),
            })
        })?;

        info!("API configuration template created at: {:?}", template_path);
        Ok(())
    }
}

/// Summary of configured APIs
#[derive(Debug, Clone)]
pub struct ApiSummary {
    pub moralis_configured: bool,
    pub moralis_dex_enabled: bool,
    pub moralis_price_enabled: bool,
    pub custom_apis: Vec<String>,
}

/// Validation report for all APIs
#[derive(Debug)]
pub struct ValidationReport {
    pub moralis: Option<ApiValidationResult>,
    pub custom_apis: HashMap<String, ApiValidationResult>,
}

/// Individual API validation result
#[derive(Debug)]
pub struct ApiValidationResult {
    pub success: bool,
    pub message: String,
    pub response_time_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_api_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ApiConfigManager::new(temp_dir.path()).unwrap();

        assert!(manager.get_moralis_api_key().is_none());
    }

    #[test]
    fn test_moralis_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ApiConfigManager::new(temp_dir.path()).unwrap();

        // Set API key
        manager.set_moralis_api_key("test_api_key".to_string()).unwrap();

        assert_eq!(manager.get_moralis_api_key().unwrap(), "test_api_key");

        // Test persistence
        let manager2 = ApiConfigManager::new(temp_dir.path()).unwrap();
        assert_eq!(manager2.get_moralis_api_key().unwrap(), "test_api_key");
    }
}
