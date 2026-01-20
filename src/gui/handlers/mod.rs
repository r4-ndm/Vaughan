//! Handler modules for WorkingWallet message processing
//!
//! This module contains specialized handlers for different categories of messages,
//! extracted from the main update() method to improve code organization and maintainability.

pub mod network;
pub mod receive;
pub mod security;
pub mod token_ops;
pub mod transaction;
pub mod ui_state;
pub mod wallet_ops;

use crate::gui::working_wallet::WorkingWalletApp;
use crate::gui::Message;
use iced::Command;

/// Common trait for all message handlers
pub trait MessageHandler {
    /// Handle a specific message and return the resulting command
    fn handle_message(&mut self, message: Message) -> Command<Message>;

    /// Check if this handler can process the given message
    fn can_handle(&self, message: &Message) -> bool;
}

/// Handler context providing access to wallet state and utilities
pub struct HandlerContext<'a> {
    pub wallet: &'a mut WorkingWalletApp,
}

impl<'a> HandlerContext<'a> {
    pub fn new(wallet: &'a mut WorkingWalletApp) -> Self {
        Self { wallet }
    }
}
