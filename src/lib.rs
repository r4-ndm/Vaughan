//! # Vaughan - Multi-EVM DeFi Wallet
//!
//! A production-ready, secure, and high-performance cryptocurrency wallet with advanced DeFi capabilities.
//! Built with Rust for memory safety, performance, and security.
//!
//! ## Features
//!
//! - **Multi-EVM Support**: Ethereum, PulseChain, BSC, Polygon, and custom networks
//! - **Token Price Data**: Real-time token pricing from APIs
//! - **Hardware Wallet Support**: Ledger and Trezor integration

pub mod blockchain;
pub mod config;
pub mod error;
pub mod gui;
pub mod network;
pub mod performance;
pub mod security;
pub mod telemetry;
pub mod tokens;
pub mod utils;
pub mod wallet;

// Re-export main types for convenience
pub use network::NetworkManager;
pub use wallet::Vaughan;

// pub use gui::VaughanApp; // Temporarily disabled
pub use error::{Result, VaughanError};

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "Vaughan";

/// Supported networks
pub mod networks {
    use crate::network::NetworkId;

    pub const ETHEREUM_MAINNET: NetworkId = NetworkId(1);
    pub const PULSECHAIN: NetworkId = NetworkId(369);
    pub const PULSECHAIN_TESTNET: NetworkId = NetworkId(943);
    pub const BSC: NetworkId = NetworkId(56);
    pub const POLYGON: NetworkId = NetworkId(137);
}
