//! UI state management including dialogs, spinners, and status messages

use crate::gui::StatusMessageColor;
use std::time::Instant;

/// UI-related state including dialogs, spinners, feedback, and themes
#[derive(Debug, Clone)]
pub struct UiState {
    // Status messages and feedback
    pub status_message: String,
    pub status_message_color: StatusMessageColor,
    pub status_message_timer: Option<Instant>,
    pub copy_feedback: Option<String>,
    pub export_copy_feedback: Option<String>,
    pub clipboard_clear_timer_active: bool,
    pub show_retry_options: bool,

    // Polling configuration
    pub poll_interval: u64,
    pub polling_active: bool,

    // General dialogs
    pub show_settings_dialog: bool,
    pub show_dapps_dialog: bool,
    pub show_dapps_coming_soon: bool,
    pub show_clear_logs_confirmation: bool,
    pub clearing_logs: bool,
    pub show_cancel_confirmation: bool,
    pub pending_cancel_tx: Option<crate::gui::state::transaction_state::PendingTransaction>,
    pub show_reset_wallet_confirmation: bool,

    // Spinners for loading states
    pub balance_spinner: bool,
    pub accounts_spinner: bool,
    pub transaction_spinner: bool,

    // Theme state - simplified to deep black only
    pub current_theme: iced::Theme,
    pub current_vaughan_theme: crate::gui::theme::VaughanTheme,

    // Import/account UI state
    pub import_type: String,
    pub show_account_dropdown: bool,

    // Activity tracking
    pub last_activity: std::time::Instant,
}

impl Default for UiState {
    fn default() -> Self {
        // Always use the deep black theme
        let saved_theme = crate::gui::theme::load_theme_preference();

        Self {
            status_message: String::new(),
            status_message_color: StatusMessageColor::Info,
            status_message_timer: None,
            copy_feedback: None,
            export_copy_feedback: None,
            clipboard_clear_timer_active: false,
            show_retry_options: false,
            poll_interval: 10,
            polling_active: false,
            show_settings_dialog: false,
            show_dapps_dialog: false,
            show_dapps_coming_soon: false,
            show_clear_logs_confirmation: false,
            clearing_logs: false,
            show_cancel_confirmation: false,
            show_reset_wallet_confirmation: false,
            pending_cancel_tx: None,
            balance_spinner: false,
            accounts_spinner: false,
            transaction_spinner: false,
            current_theme: saved_theme.clone().into(),
            current_vaughan_theme: saved_theme,
            import_type: "seed".to_string(),
            show_account_dropdown: false,
            last_activity: std::time::Instant::now(),
        }
    }
}
