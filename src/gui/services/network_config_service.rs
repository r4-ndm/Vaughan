//! Network Configuration Service - Network validation and configuration
//!
//! This service extracts network configuration validation logic from view components,
//! providing consistent validation for network settings.

use crate::network::NetworkConfig;

/// Errors that can occur during network validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkValidationError {
    /// RPC URL is invalid or malformed
    InvalidRpcUrl(String),
    /// Chain ID is invalid (zero or negative)
    InvalidChainId,
    /// Explorer URL is invalid or malformed
    InvalidExplorerUrl(String),
    /// Network name already exists
    DuplicateNetwork(String),
    /// Network name is empty or invalid
    InvalidNetworkName(String),
}

impl std::fmt::Display for NetworkValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRpcUrl(msg) => write!(f, "Invalid RPC URL: {}", msg),
            Self::InvalidChainId => write!(f, "Invalid chain ID: must be non-zero"),
            Self::InvalidExplorerUrl(msg) => write!(f, "Invalid explorer URL: {}", msg),
            Self::DuplicateNetwork(name) => write!(f, "Network '{}' already exists", name),
            Self::InvalidNetworkName(msg) => write!(f, "Invalid network name: {}", msg),
        }
    }
}

impl std::error::Error for NetworkValidationError {}

/// Trait defining the network configuration service interface for testability.
pub trait NetworkConfigServiceTrait: Send + Sync {
    /// Validate a network configuration.
    fn validate_network_config(&self, config: &NetworkConfig) -> Result<(), NetworkValidationError>;
    
    /// Check if a network name is unique among existing networks.
    fn is_network_name_unique(&self, name: &str, existing: &[NetworkConfig]) -> bool;
    
    /// Sanitize and validate an RPC URL.
    fn sanitize_rpc_url(&self, url: &str) -> Result<String, NetworkValidationError>;
    
    /// Sanitize and validate an explorer URL.
    fn sanitize_explorer_url(&self, url: &str) -> Result<Option<String>, NetworkValidationError>;
    
    /// Validate a network name.
    fn validate_network_name(&self, name: &str) -> Result<(), NetworkValidationError>;
}

/// Network configuration service implementation.
#[derive(Debug, Default)]
pub struct NetworkConfigService;

impl NetworkConfigService {
    /// Create a new network configuration service.
    pub fn new() -> Self {
        Self
    }
    
    /// Check if a URL is valid (starts with http:// or https://)
    fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}

impl NetworkConfigServiceTrait for NetworkConfigService {
    fn validate_network_config(&self, config: &NetworkConfig) -> Result<(), NetworkValidationError> {
        // Validate network name
        self.validate_network_name(&config.name)?;
        
        // Validate RPC URL
        self.sanitize_rpc_url(&config.rpc_url)?;
        
        // Validate chain ID
        if config.chain_id == 0 {
            return Err(NetworkValidationError::InvalidChainId);
        }
        
        // Validate explorer URL if present
        if !config.block_explorer_url.is_empty() {
            self.sanitize_explorer_url(&config.block_explorer_url)?;
        }
        
        Ok(())
    }
    
    fn is_network_name_unique(&self, name: &str, existing: &[NetworkConfig]) -> bool {
        !existing.iter().any(|n| n.name.eq_ignore_ascii_case(name))
    }
    
    fn sanitize_rpc_url(&self, url: &str) -> Result<String, NetworkValidationError> {
        let trimmed = url.trim();
        
        if trimmed.is_empty() {
            return Err(NetworkValidationError::InvalidRpcUrl(
                "URL cannot be empty".to_string()
            ));
        }
        
        if !Self::is_valid_url(trimmed) {
            return Err(NetworkValidationError::InvalidRpcUrl(
                "URL must start with http:// or https://".to_string()
            ));
        }
        
        // Check for common mistakes
        if trimmed.contains(' ') {
            return Err(NetworkValidationError::InvalidRpcUrl(
                "URL cannot contain spaces".to_string()
            ));
        }
        
        Ok(trimmed.to_string())
    }
    
    fn sanitize_explorer_url(&self, url: &str) -> Result<Option<String>, NetworkValidationError> {
        let trimmed = url.trim();
        
        if trimmed.is_empty() {
            return Ok(None);
        }
        
        if !Self::is_valid_url(trimmed) {
            return Err(NetworkValidationError::InvalidExplorerUrl(
                "URL must start with http:// or https://".to_string()
            ));
        }
        
        // Check for common mistakes
        if trimmed.contains(' ') {
            return Err(NetworkValidationError::InvalidExplorerUrl(
                "URL cannot contain spaces".to_string()
            ));
        }
        
        Ok(Some(trimmed.to_string()))
    }
    
    fn validate_network_name(&self, name: &str) -> Result<(), NetworkValidationError> {
        let trimmed = name.trim();
        
        if trimmed.is_empty() {
            return Err(NetworkValidationError::InvalidNetworkName(
                "Name cannot be empty".to_string()
            ));
        }
        
        if trimmed.len() > 50 {
            return Err(NetworkValidationError::InvalidNetworkName(
                "Name is too long (max 50 characters)".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> NetworkConfigService {
        NetworkConfigService::new()
    }

    fn create_test_config(name: &str, rpc_url: &str, chain_id: u64) -> NetworkConfig {
        use crate::network::NetworkId;
        NetworkConfig {
            id: NetworkId(chain_id),
            name: name.to_string(),
            rpc_url: rpc_url.to_string(),
            chain_id,
            symbol: "ETH".to_string(),
            block_explorer_url: String::new(),
            is_testnet: false,
            is_custom: true,
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let s = service();
        let config = create_test_config("Test Network", "https://rpc.example.com", 1);
        assert!(s.validate_network_config(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_rpc_url() {
        let s = service();
        let config = create_test_config("Test Network", "invalid-url", 1);
        let result = s.validate_network_config(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidRpcUrl(_)));
    }

    #[test]
    fn test_validate_zero_chain_id() {
        let s = service();
        let config = create_test_config("Test Network", "https://rpc.example.com", 0);
        let result = s.validate_network_config(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidChainId));
    }

    #[test]
    fn test_validate_empty_network_name() {
        let s = service();
        let config = create_test_config("", "https://rpc.example.com", 1);
        let result = s.validate_network_config(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidNetworkName(_)));
    }

    #[test]
    fn test_is_network_name_unique() {
        let s = service();
        let existing = vec![
            create_test_config("Ethereum", "https://eth.example.com", 1),
            create_test_config("Polygon", "https://polygon.example.com", 137),
        ];
        
        assert!(!s.is_network_name_unique("Ethereum", &existing));
        assert!(!s.is_network_name_unique("ethereum", &existing)); // Case insensitive
        assert!(s.is_network_name_unique("BSC", &existing));
    }

    #[test]
    fn test_sanitize_rpc_url_valid() {
        let s = service();
        assert_eq!(
            s.sanitize_rpc_url("https://rpc.example.com").unwrap(),
            "https://rpc.example.com"
        );
        assert_eq!(
            s.sanitize_rpc_url("http://localhost:8545").unwrap(),
            "http://localhost:8545"
        );
    }

    #[test]
    fn test_sanitize_rpc_url_trims_whitespace() {
        let s = service();
        assert_eq!(
            s.sanitize_rpc_url("  https://rpc.example.com  ").unwrap(),
            "https://rpc.example.com"
        );
    }

    #[test]
    fn test_sanitize_rpc_url_empty() {
        let s = service();
        let result = s.sanitize_rpc_url("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidRpcUrl(_)));
    }

    #[test]
    fn test_sanitize_rpc_url_no_protocol() {
        let s = service();
        let result = s.sanitize_rpc_url("rpc.example.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidRpcUrl(_)));
    }

    #[test]
    fn test_sanitize_rpc_url_with_spaces() {
        let s = service();
        let result = s.sanitize_rpc_url("https://rpc example.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidRpcUrl(_)));
    }

    #[test]
    fn test_sanitize_explorer_url_valid() {
        let s = service();
        assert_eq!(
            s.sanitize_explorer_url("https://explorer.example.com").unwrap(),
            Some("https://explorer.example.com".to_string())
        );
    }

    #[test]
    fn test_sanitize_explorer_url_empty() {
        let s = service();
        assert_eq!(s.sanitize_explorer_url("").unwrap(), None);
        assert_eq!(s.sanitize_explorer_url("   ").unwrap(), None);
    }

    #[test]
    fn test_sanitize_explorer_url_invalid() {
        let s = service();
        let result = s.sanitize_explorer_url("invalid-url");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidExplorerUrl(_)));
    }

    #[test]
    fn test_validate_network_name_valid() {
        let s = service();
        assert!(s.validate_network_name("Ethereum").is_ok());
        assert!(s.validate_network_name("My Custom Network").is_ok());
    }

    #[test]
    fn test_validate_network_name_empty() {
        let s = service();
        let result = s.validate_network_name("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidNetworkName(_)));
    }

    #[test]
    fn test_validate_network_name_too_long() {
        let s = service();
        let long_name = "a".repeat(51);
        let result = s.validate_network_name(&long_name);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkValidationError::InvalidNetworkName(_)));
    }

    #[test]
    fn test_error_display() {
        let err = NetworkValidationError::InvalidRpcUrl("test error".to_string());
        assert_eq!(err.to_string(), "Invalid RPC URL: test error");
        
        let err = NetworkValidationError::InvalidChainId;
        assert_eq!(err.to_string(), "Invalid chain ID: must be non-zero");
        
        let err = NetworkValidationError::DuplicateNetwork("Ethereum".to_string());
        assert_eq!(err.to_string(), "Network 'Ethereum' already exists");
    }
}
