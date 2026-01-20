//! Network configuration management

use crate::network::NetworkConfig;

/// Network configuration manager
pub struct NetworkConfigManager {
    configs: Vec<NetworkConfig>,
}

impl Default for NetworkConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkConfigManager {
    pub fn new() -> Self {
        Self {
            configs: vec![
                NetworkConfig::ethereum_mainnet(),
                NetworkConfig::pulsechain(),
                NetworkConfig::bsc(),
                NetworkConfig::polygon(),
            ],
        }
    }

    pub fn get_configs(&self) -> &[NetworkConfig] {
        &self.configs
    }

    pub fn add_config(&mut self, config: NetworkConfig) {
        self.configs.push(config);
    }
}
