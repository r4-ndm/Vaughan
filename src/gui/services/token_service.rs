//! Token Service - Custom token persistence
//!
//! Handles saving and loading custom tokens using the config system.

use crate::config::{ConfigManager, CustomToken, CustomTokensConfig};
use crate::gui::wallet_types::TokenInfo;
use std::collections::HashMap;

const TOKENS_CONFIG_FILE: &str = "custom_tokens.json";

/// Save custom tokens to config file
pub async fn save_custom_tokens(tokens: &[TokenInfo]) -> Result<(), String> {
    let config_manager = ConfigManager::new();

    // Convert GUI tokens to config tokens
    let mut tokens_by_network: HashMap<String, Vec<CustomToken>> = HashMap::new();

    for token in tokens {
        let custom_token = CustomToken {
            address: token.address.clone(),
            name: token.name.clone(),
            symbol: token.symbol.clone(),
            decimals: token.decimals,
            added_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
            verified: false,
            tags: vec![],
        };

        // Use "main" as default network for now
        tokens_by_network
            .entry("main".to_string())
            .or_default()
            .push(custom_token);
    }

    let config = CustomTokensConfig {
        version: "1.0".to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
        tokens: tokens_by_network,
    };

    config_manager
        .save_config(TOKENS_CONFIG_FILE, &config)
        .map_err(|e| format!("Failed to save tokens: {}", e))
}

/// Load custom tokens from config file
pub async fn load_custom_tokens() -> Result<Vec<TokenInfo>, String> {
    let config_manager = ConfigManager::new();

    if !config_manager.config_exists(TOKENS_CONFIG_FILE) {
        return Ok(vec![]);
    }

    let config: CustomTokensConfig = config_manager
        .load_config(TOKENS_CONFIG_FILE)
        .map_err(|e| format!("Failed to load tokens: {}", e))?;

    // Convert config tokens to GUI tokens
    let mut gui_tokens = Vec::new();

    for (_network, tokens) in config.tokens {
        for token in tokens {
            let gui_token = TokenInfo {
                address: token.address,
                name: token.name,
                symbol: token.symbol,
                decimals: token.decimals,
                balance: None,
            };
            gui_tokens.push(gui_token);
        }
    }

    Ok(gui_tokens)
}
