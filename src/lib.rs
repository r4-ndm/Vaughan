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

/// Supported blockchain networks
///
/// Pre-defined network IDs for commonly used EVM-compatible chains.
pub mod networks {
    use crate::network::NetworkId;

    /// Ethereum Mainnet (Chain ID: 1)
    pub const ETHEREUM_MAINNET: NetworkId = NetworkId(1);
    
    /// PulseChain Mainnet (Chain ID: 369)
    pub const PULSECHAIN: NetworkId = NetworkId(369);
    
    /// PulseChain Testnet v4 (Chain ID: 943)
    pub const PULSECHAIN_TESTNET: NetworkId = NetworkId(943);
    
    /// Binance Smart Chain (Chain ID: 56)
    pub const BSC: NetworkId = NetworkId(56);
    
    /// Polygon (Matic) Mainnet (Chain ID: 137)
    pub const POLYGON: NetworkId = NetworkId(137);
}
