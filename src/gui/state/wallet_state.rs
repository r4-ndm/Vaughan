//! Wallet and account management state

use crate::gui::wallet_types::ImportType;
use crate::security::SecureAccount;
use crate::security::SeedStrength;
use secrecy::SecretString;
use std::collections::HashSet;

/// Wallet account management and creation/import state
#[derive(Debug, Clone)]
pub struct WalletState {
    // Current account and available accounts
    pub current_account: String,
    pub current_account_id: Option<String>,
    pub available_accounts: Vec<SecureAccount>,
    pub loading_accounts: bool,
    pub account_balance: String,

    // Account operations
    pub show_delete_account: bool,
    pub deleting_account: bool,
    pub address_just_copied: bool,

    // Receive dialog state
    pub receive_dialog: ReceiveDialogState,

    // Wallet creation state
    pub show_create_wallet: bool,
    pub wallet_name: String,
    pub seed_phrase: String,
    pub selected_seed_strength: SeedStrength,
    pub seed_analysis: Option<crate::security::seed::SeedAnalysis>,
    pub generating_seed: bool,
    pub creating_wallet: bool,
    pub master_password: String,
    pub confirm_password: String,

    // Wallet import state
    pub show_import_wallet: bool,
    pub private_key: SecretString,
    pub import_type: ImportType,

    // Export wallet state
    pub show_export_wallet: bool,
    pub exported_seed_phrase: String,
    pub exported_private_key: SecretString,
    pub exporting_data: bool,
    pub selected_export_account_id: Option<String>,
    pub show_account_dropdown: bool,

    pub export_loading: bool,
    pub export_result: String,
    pub export_error_message: Option<String>,

    // Multi-address management
    pub show_address_discovery: bool,
    pub discovered_addresses: Vec<(String, String, bool)>, // (address, derivation_path, has_activity)
    pub selected_addresses_for_import: HashSet<String>,
    pub discovering_addresses: bool,
    pub current_seed_for_discovery: String,

    // Hardware wallet state
    pub show_hardware_wallet: bool,
    pub available_hardware_wallets: Vec<crate::security::hardware::HardwareWalletInfo>,
    pub detecting_hardware_wallets: bool,
    pub hardware_wallet_addresses: Vec<alloy::primitives::Address>,
    pub loading_hardware_addresses: bool,

    // Account creation/import dialogs
    pub show_create_dialog: bool,
    pub show_import_dialog: bool,
    pub create_account_name: String,
    pub import_private_key: String,
    pub import_account_name: String,
    pub creating_account: bool,
    pub importing_account: bool,
}

impl Default for WalletState {
    fn default() -> Self {
        Self {
            current_account: "No account selected".to_string(),
            current_account_id: None,
            available_accounts: Vec::new(),
            loading_accounts: true,
            account_balance: "0.000000 tPLS".to_string(),
            show_delete_account: false,
            deleting_account: false,
            address_just_copied: false,
            show_create_wallet: false,
            wallet_name: String::new(),
            seed_phrase: String::new(),
            selected_seed_strength: SeedStrength::Words12,
            seed_analysis: None,
            generating_seed: false,
            creating_wallet: false,
            master_password: String::new(),
            confirm_password: String::new(),
            show_import_wallet: false,
            private_key: SecretString::new(String::new()),
            import_type: ImportType::PrivateKey,
            show_export_wallet: false,
            exported_seed_phrase: String::new(),
            exported_private_key: SecretString::new(String::new()),
            exporting_data: false,
            selected_export_account_id: None,
            show_account_dropdown: false,
            export_loading: false,
            export_result: String::new(),
            export_error_message: None,
            show_address_discovery: false,
            discovered_addresses: Vec::new(),
            selected_addresses_for_import: HashSet::new(),
            discovering_addresses: false,
            current_seed_for_discovery: String::new(),
            show_hardware_wallet: false,
            available_hardware_wallets: Vec::new(),
            detecting_hardware_wallets: false,
            hardware_wallet_addresses: Vec::new(),
            loading_hardware_addresses: false,
            show_create_dialog: false,
            show_import_dialog: false,
            create_account_name: String::new(),
            import_private_key: String::new(),
            import_account_name: String::new(),
            creating_account: false,
            importing_account: false,
            receive_dialog: ReceiveDialogState::default(),
        }
    }
}

/// Receive dialog state
#[derive(Debug, Clone, Default)]
pub struct ReceiveDialogState {
    /// Dialog visibility
    pub visible: bool,
}
