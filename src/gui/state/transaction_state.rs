//! Transaction and gas management state

use crate::gui::wallet_types::{GasEstimation, GasSpeed};
use crate::gui::{HistoryTab, Transaction};
use crate::network::NetworkId;
use alloy::primitives::{Address, U256};
use std::time::Instant;

/// Transaction type for gas management (Legacy or EIP-1559)
#[derive(Debug, Clone)]
pub enum TransactionType {
    Legacy,
    Eip1559,
}

/// Cancellation progress states for detailed user feedback
#[derive(Debug, Clone, PartialEq)]
pub enum CancellationProgress {
    None,
    ValidatingTransaction,
    CalculatingGasSettings,
    CheckingBalance,
    SigningReplacement,
    BroadcastingReplacement,
    WaitingConfirmation,
}

/// Pending transaction information for cancellation tracking
#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub tx_hash: String,
    pub nonce: u64,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    // Gas fields
    pub tx_type: TransactionType,
    pub gas_limit: u64,
    // Legacy
    pub gas_price: Option<U256>,
    // EIP-1559
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    pub timestamp: Instant,
    pub network: NetworkId,
    pub cancellable: bool,
}

/// Transaction-related state including history, sending, and gas estimation
#[derive(Debug, Clone)]
pub struct TransactionState {
    // Transaction history
    pub show_history: bool,
    pub show_transaction_history: bool,
    pub current_history_tab: HistoryTab,
    pub transaction_history: Vec<Transaction>,
    pub transaction_fetch_error: bool,
    pub loading_transactions: bool,

    // Send transaction dialog
    pub send_to_address: String,
    pub send_amount: String,
    pub send_gas_limit: String,
    pub send_gas_price: String,
    pub send_selected_token: String,
    pub send_custom_token_address: String,
    pub send_show_custom_token_input: bool,
    pub send_available_tokens: Vec<String>,
    pub sending_transaction: bool,

    // Advanced send options
    pub send_tx_type: String,               // "Legacy" or "EIP-1559"
    pub send_max_fee_gwei: String,          // for EIP-1559
    pub send_max_priority_fee_gwei: String, // for EIP-1559
    pub send_nonce_override: String,        // optional
    pub gas_speed: GasSpeed,                // selected gas speed
    pub send_show_advanced: bool,

    // Gas estimation and confirmation
    pub estimating_gas: bool,
    pub gas_estimation: Option<GasEstimation>,
    pub show_transaction_confirmation: bool,

    // Send from account selection
    pub send_from_account_id: Option<String>, // ID of the account to send from

    // Pending transaction tracking for cancellation
    pub pending_transactions: Vec<PendingTransaction>,
    pub last_used_nonce: Option<u64>,
    pub cancellation_in_progress: bool,
    pub cancellation_progress: CancellationProgress,
}

impl Default for TransactionState {
    fn default() -> Self {
        Self {
            show_history: false,
            show_transaction_history: false,
            current_history_tab: HistoryTab::Transactions,
            transaction_history: Vec::new(),
            transaction_fetch_error: false,
            loading_transactions: false,
            send_to_address: String::new(),
            send_amount: String::new(),
            send_gas_limit: String::new(),
            send_gas_price: String::new(),
            send_selected_token: "NATIVE (ETH)".to_string(),
            send_custom_token_address: String::new(),
            send_show_custom_token_input: false,
            send_available_tokens: Vec::new(),
            sending_transaction: false,
            send_tx_type: "Legacy".to_string(),
            send_max_fee_gwei: String::new(),
            send_max_priority_fee_gwei: String::new(),
            send_nonce_override: String::new(),
            gas_speed: GasSpeed::Standard,
            send_show_advanced: false,
            estimating_gas: false,
            gas_estimation: None,
            show_transaction_confirmation: false,
            send_from_account_id: None,
            pending_transactions: Vec::new(),
            last_used_nonce: None,
            cancellation_in_progress: false,
            cancellation_progress: CancellationProgress::None,
        }
    }
}
