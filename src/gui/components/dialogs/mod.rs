//! Dialog Components Module
//!
//! This module contains all dialog UI components extracted from the main working_wallet.rs
//! and views/dialogs.rs for better code organization.

// Dialog component modules
pub mod cancel_transaction_dialog;
pub mod confirmation_dialogs;
pub mod create_wallet_dialog;
pub mod custom_token_dialog;
pub mod export_wallet_dialog;
pub mod import_wallet_dialog;
pub mod network_dialog;
pub mod receive_dialog;
pub mod transaction_confirmation;
pub mod unified_password_dialog;

// Re-export dialog functions for easy access
pub use cancel_transaction_dialog::cancel_transaction_dialog_view;
pub use confirmation_dialogs::{
    clear_logs_confirmation_dialog_view, dapps_coming_soon_dialog_view, delete_account_dialog_view,
    delete_network_confirmation_dialog_view, hardware_wallet_dialog_view, reset_wallet_confirmation_dialog_view,
    ModalBackgroundStyle,
};
pub use create_wallet_dialog::create_wallet_dialog_view;
pub use custom_token_dialog::custom_token_screen_view;
pub use export_wallet_dialog::export_wallet_dialog_view;
pub use import_wallet_dialog::import_wallet_dialog_view;
pub use network_dialog::add_network_dialog_view;
pub use receive_dialog::receive_dialog_view;
pub use transaction_confirmation::transaction_confirmation_dialog_view;
pub use unified_password_dialog::password_dialog_view as unified_password_dialog_view;
