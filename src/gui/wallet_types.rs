//! Wallet Data Types and Structures
//!
//! This module contains all the data structures, enums, and types used by the wallet GUI.

use crate::network::NetworkId;
use iced::Color;

/// Import type for wallet import options
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub enum ImportType {
    #[default]
    SeedPhrase,
    PrivateKey,
}

impl std::fmt::Display for ImportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportType::SeedPhrase => write!(f, "Seed Phrase"),
            ImportType::PrivateKey => write!(f, "Private Key"),
        }
    }
}

/// Account type for authentication requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AccountType {
    /// Account created from or imported via a BIP-39 seed phrase
    SeedBased,
    /// Account imported directly via private key
    PrivateKey,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::SeedBased => write!(f, "Seed-Based"),
            AccountType::PrivateKey => write!(f, "Private Key"),
        }
    }
}

/// Main application tab types
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub enum MainTab {
    #[default]
    Wallet,
    Send,
    History,
    Settings,
}

impl std::fmt::Display for MainTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainTab::Wallet => write!(f, "Wallet"),
            MainTab::Send => write!(f, "Send"),
            MainTab::History => write!(f, "History"),
            MainTab::Settings => write!(f, "Settings"),
        }
    }
}

/// History tab types
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub enum HistoryTab {
    #[default]
    Recent,
    Transactions,
    WalletLogs,
}

impl std::fmt::Display for HistoryTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryTab::Recent => write!(f, "Recent"),
            HistoryTab::Transactions => write!(f, "Transactions"),
            HistoryTab::WalletLogs => write!(f, "Wallet Logs"),
        }
    }
}

/// Log entry for tracking wallet operations and errors
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub timestamp: String,
    pub category: LogCategory,
    pub message: String,
    pub details: Option<String>,
    pub copyable: bool,
    pub error_severity: Option<crate::error::ErrorSeverity>,
    pub error_category: Option<crate::error::ErrorCategory>,
    pub support_code: Option<String>,
    pub recovery_steps: Option<Vec<String>>,
}

impl LogEntry {
    pub fn new(category: LogCategory, message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            category,
            message,
            details: None,
            copyable: false,
            error_severity: None,
            error_category: None,
            support_code: None,
            recovery_steps: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn copyable(mut self) -> Self {
        self.copyable = true;
        self
    }
}

/// Categories for log entries
#[derive(Debug, Clone, PartialEq)]
pub enum LogCategory {
    Transaction,
    Network,
    Security,
    Wallet,
    Error,
    Success,
    Warning,
    Info,
}

/// Gas estimation data
#[derive(Debug, Clone)]
pub struct GasEstimation {
    pub estimated_gas: u64,
    pub gas_price: String,
    pub estimated_cost: String,
    pub total_cost: String,
    pub currency: String,
}

impl GasEstimation {
    pub fn new(estimated_gas: u64, gas_price: String, currency: String) -> Result<Self, String> {
        // Parse gas price to calculate total cost
        let gas_price_f64: f64 = gas_price.parse().map_err(|_| "Invalid gas price format".to_string())?;

        let total_cost_wei = estimated_gas as f64 * gas_price_f64;
        let total_cost_eth = total_cost_wei / 1e18;

        // Format to reasonable precision
        let total_cost = if total_cost_eth < 0.001 {
            format!("{total_cost_eth:.8}")
        } else if total_cost_eth < 0.1 {
            format!("{total_cost_eth:.6}")
        } else {
            format!("{total_cost_eth:.4}")
        };

        // Remove trailing zeros
        let total_cost = total_cost.trim_end_matches('0').trim_end_matches('.').to_string();

        Ok(Self {
            estimated_gas,
            gas_price,
            estimated_cost: total_cost.clone(),
            total_cost,
            currency,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.estimated_gas == 0 {
            return Err("Gas estimate cannot be zero".to_string());
        }

        if self.gas_price.is_empty() {
            return Err("Gas price cannot be empty".to_string());
        }

        // Try to parse gas price to ensure it's valid
        self.gas_price
            .parse::<f64>()
            .map_err(|_| "Gas price must be a valid number".to_string())?;

        Ok(())
    }
}

/// Network configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Network {
    pub id: NetworkId,
    pub name: String,
    pub native_token: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub explorer_url: Option<String>,
}

impl Network {
    pub fn ethereum() -> Self {
        Self {
            id: NetworkId(1),
            name: "Ethereum Mainnet".to_string(),
            native_token: "ETH".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY".to_string(),
            chain_id: 1,
            explorer_url: Some("https://etherscan.io".to_string()),
        }
    }

    pub fn pulsechain() -> Self {
        Self {
            id: NetworkId(369),
            name: "PulseChain".to_string(),
            native_token: "PLS".to_string(),
            rpc_url: "https://rpc.pulsechain.com".to_string(),
            chain_id: 369,
            explorer_url: Some("https://scan.pulsechain.com".to_string()),
        }
    }

    pub fn pulsechain_testnet() -> Self {
        Self {
            id: NetworkId(943),
            name: "PulseChain Testnet".to_string(),
            native_token: "tPLS".to_string(),
            rpc_url: "https://rpc.v4.testnet.pulsechain.com".to_string(),
            chain_id: 943,
            explorer_url: Some("https://scan.v4.testnet.pulsechain.com".to_string()),
        }
    }

    pub fn bsc() -> Self {
        Self {
            id: NetworkId(56),
            name: "Binance Smart Chain".to_string(),
            native_token: "BNB".to_string(),
            rpc_url: "https://bsc-dataseed1.binance.org".to_string(),
            chain_id: 56,
            explorer_url: Some("https://bscscan.com".to_string()),
        }
    }

    pub fn polygon() -> Self {
        Self {
            id: NetworkId(137),
            name: "Polygon".to_string(),
            native_token: "MATIC".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            chain_id: 137,
            explorer_url: Some("https://polygonscan.com".to_string()),
        }
    }

    pub fn arbitrum() -> Self {
        Self {
            id: NetworkId(42161),
            name: "Arbitrum One".to_string(),
            native_token: "ETH".to_string(),
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            chain_id: 42161,
            explorer_url: Some("https://arbiscan.io".to_string()),
        }
    }

    pub fn optimism() -> Self {
        Self {
            id: NetworkId(10),
            name: "Optimism".to_string(),
            native_token: "ETH".to_string(),
            rpc_url: "https://mainnet.optimism.io".to_string(),
            chain_id: 10,
            explorer_url: Some("https://optimistic.etherscan.io".to_string()),
        }
    }

    pub fn default_networks() -> Vec<Self> {
        vec![
            Self::ethereum(),
            Self::pulsechain(),
            Self::pulsechain_testnet(),
            Self::bsc(),
            Self::polygon(),
            Self::arbitrum(),
            Self::optimism(),
        ]
    }

    pub fn custom(name: String, chain_id: u64, native_token: String, rpc_url: String) -> Self {
        Self {
            id: NetworkId(chain_id),
            name,
            native_token,
            rpc_url,
            chain_id,
            explorer_url: None,
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Token information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>, // User's balance for this token
}

/// Transaction data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub timestamp: String,
    pub status: TransactionStatus,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl TransactionStatus {
    pub fn text(&self) -> &str {
        match self {
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Confirmed => "Confirmed",
            TransactionStatus::Failed => "Failed",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TransactionStatus::Pending => Color::from_rgb(1.0, 0.8, 0.2), // Yellow
            TransactionStatus::Confirmed => Color::from_rgb(0.2, 0.8, 0.2), // Green
            TransactionStatus::Failed => Color::from_rgb(1.0, 0.4, 0.4),  // Red
        }
    }
}

/// Gas speed options
#[derive(Debug, Clone, PartialEq)]
pub enum GasSpeed {
    Slow,
    Standard,
    Fast,
}

impl GasSpeed {
    pub fn multiplier(&self) -> f64 {
        match self {
            GasSpeed::Slow => 0.8,
            GasSpeed::Standard => 1.0,
            GasSpeed::Fast => 1.2,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            GasSpeed::Slow => "Slow (cheaper)",
            GasSpeed::Standard => "Standard",
            GasSpeed::Fast => "Fast (more expensive)",
        }
    }
}

impl std::fmt::Display for GasSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            GasSpeed::Slow => "Slow",
            GasSpeed::Standard => "Standard",
            GasSpeed::Fast => "Fast",
        };
        write!(f, "{name}")
    }
}

/// Status message colors
#[derive(Debug, Clone, PartialEq)]
pub enum StatusMessageColor {
    Default,
    Success,
    Error,
    Warning,
    Info,
}

/// Graphics capabilities for rendering
#[derive(Debug, Clone)]
pub struct GraphicsCapabilities {
    pub supports_transparency: bool,
    pub max_texture_size: u32,
    pub vendor: String,
    pub renderer: String,
}

/// Cancel TX button states for smart UI behavior
#[derive(Debug, Clone, PartialEq)]
pub enum CancelButtonState {
    /// No pending transactions available for cancellation
    NoPending,
    /// Transaction(s) can be cancelled - shows count
    Cancellable(usize),
    /// Transaction already confirmed/too late to cancel
    TooLate,
    /// Cancellation is currently in progress
    InProgress,
}
