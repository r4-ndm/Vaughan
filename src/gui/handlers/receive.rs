//! Receive dialog handlers
//!
//! Handles receive dialog messages including address display and clipboard operations.

use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::{LogCategory, Message};
use iced::Command;

impl WorkingWalletApp {
    /// Handle receive-related messages
    pub fn handle_receive_message(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ShowReceiveDialog => self.handle_show_receive_dialog(),
            Message::HideReceiveDialog => self.handle_hide_receive_dialog(),
            Message::CopyToClipboard(text) => self.handle_copy_to_clipboard(text),
            _ => Command::none(),
        }
    }

    /// Show receive dialog
    fn handle_show_receive_dialog(&mut self) -> Command<Message> {
        let wallet_state = self.state.wallet_mut();
        wallet_state.receive_dialog.visible = true;

        tracing::info!("Receive dialog shown");
        self.add_log_entry(LogCategory::Wallet, "Receive dialog opened".to_string(), None);

        Command::none()
    }

    /// Hide receive dialog
    fn handle_hide_receive_dialog(&mut self) -> Command<Message> {
        let wallet_state = self.state.wallet_mut();
        wallet_state.receive_dialog.visible = false;

        tracing::info!("Receive dialog hidden");
        Command::none()
    }

    /// Copy text to clipboard
    fn handle_copy_to_clipboard(&mut self, text: String) -> Command<Message> {
        tracing::info!("Copying to clipboard: {}", text);

        // Use iced clipboard functionality
        let command = iced::clipboard::write(text.clone());

        self.add_log_entry(
            LogCategory::Wallet,
            "Copied to clipboard".to_string(),
            Some(format!(
                "Copied: {}...{}",
                &text[..std::cmp::min(6, text.len())],
                if text.len() > 4 { &text[text.len() - 4..] } else { "" }
            )),
        );

        command
    }
}
