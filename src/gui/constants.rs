//! GUI Constants for Performance Optimization
//!
//! This module contains string constants and other frequently used values
//! to avoid repeated allocations.

/// Common string constants to avoid repeated allocations
pub const ETHEREUM_ID: &str = "ethereum";
pub const ETHEREUM_NAME: &str = "Ethereum";
pub const ETH_SYMBOL: &str = "ETH";

pub const PULSECHAIN_ID: &str = "pulsechain";
pub const PULSECHAIN_NAME: &str = "PulseChain";
pub const PLS_SYMBOL: &str = "PLS";

/// Default RPC endpoints
pub const ETHEREUM_PUBLIC_RPC: &str = "https://ethereum.publicnode.com";
pub const PULSECHAIN_RPC: &str = "https://rpc.pulsechain.com";
pub const PULSECHAIN_TESTNET_RPC: &str = "https://rpc.v4.testnet.pulsechain.com";

/// Common UI messages
pub const WALLET_INIT_SUCCESS: &str = "Wallet initialized successfully";
pub const WALLET_INIT_FAILED: &str = "Wallet initialization failed";
pub const BALANCE_REFRESH_STARTED: &str = "Refreshing balance...";
pub const TRANSACTION_SENT: &str = "Transaction sent successfully";

/// Common format strings to avoid repeated allocations
pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S UTC";
pub const ADDRESS_FORMAT: &str = "0x";

/// Common UI dimensions as constants
pub const WINDOW_MIN_WIDTH: f32 = 600.0;
pub const WINDOW_MIN_HEIGHT: f32 = 850.0;
pub const BUTTON_HEIGHT: f32 = 35.0;
pub const INPUT_PADDING: f32 = 10.0;
