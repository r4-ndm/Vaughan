//! Network and balance management state

use crate::network::{NetworkConfig, NetworkId};
use std::time::{Duration, Instant};

/// Network-related state including current network, balance, and polling
#[derive(Debug, Clone)]
pub struct NetworkState {
    // Current network and available networks
    pub current_network: NetworkId,
    pub available_networks: Vec<NetworkConfig>,
    pub loading_networks: bool,

    // Balance management
    pub balance: String,
    pub last_balance: Option<String>,

    // Network polling state
    pub polling_active: bool,
    pub poll_interval: Duration,

    // Network editing state
    pub show_add_network: bool,
    pub network_name: String,
    pub network_rpc_url: String,
    pub network_chain_id: String,
    pub network_symbol: String,
    pub network_block_explorer: String,
    pub adding_network: bool,
    pub edit_mode: bool,
    pub selected_network_for_edit: Option<String>,
    pub editing_network: bool,
    pub show_delete_network_confirmation: bool,
    pub show_http_warning_dialog: bool,

    // Price information
    pub show_price_info: bool,
    pub eth_price: Option<f64>,
    pub eth_price_change_24h: Option<String>,
    pub fetching_price: bool,
    pub price_last_updated: Option<Instant>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            current_network: NetworkId(943), // Default to PulseChain Testnet v4
            available_networks: Vec::new(),
            loading_networks: true,
            balance: "0.000000 tPLS".to_string(),
            last_balance: None,
            polling_active: false,
            poll_interval: Duration::from_secs(10),
            show_add_network: false,
            network_name: String::new(),
            network_rpc_url: String::new(),
            network_chain_id: String::new(),
            network_symbol: String::new(),
            network_block_explorer: String::new(),
            adding_network: false,
            edit_mode: false,
            selected_network_for_edit: None,
            editing_network: false,
            show_delete_network_confirmation: false,
            show_http_warning_dialog: false,
            show_price_info: false,
            eth_price: None,
            eth_price_change_24h: None,
            fetching_price: false,
            price_last_updated: None,
        }
    }
}

impl NetworkState {
    /// Get the RPC URL for the current network
    pub fn get_current_rpc_url(&self) -> String {
        self.available_networks
            .iter()
            .find(|n| n.id == self.current_network)
            .map(|n| n.rpc_url.clone())
            .unwrap_or_else(|| "https://ethereum.publicnode.com".to_string())
    }
}
