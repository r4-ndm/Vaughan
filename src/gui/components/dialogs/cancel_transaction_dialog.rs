//! Cancel Transaction Dialog Component
//!
//! This module contains the cancel transaction confirmation dialog UI component.
//! Extracted from views/dialogs.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, Row, Text},
    Element, Length,
};

use crate::gui::{state::transaction_state::TransactionType, theme::styles, working_wallet::AppState, Message};

/// Cancel transaction confirmation dialog
pub fn cancel_transaction_dialog_view(state: &AppState) -> Element<'_, Message> {
    let pending_tx = state.ui().pending_cancel_tx.as_ref().unwrap();

    // Format transaction hash for display
    let tx_hash_display = if pending_tx.tx_hash.len() > 20 {
        format!(
            "{}...{}",
            &pending_tx.tx_hash[0..10],
            &pending_tx.tx_hash[pending_tx.tx_hash.len() - 10..]
        )
    } else {
        pending_tx.tx_hash.clone()
    };

    let dialog_content = Column::new()
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .push(
            Text::new("🚫 Cancel Transaction")
                .size(22)
                .style(iced::Color::from_rgb(1.0, 0.6, 0.2))
        )
        .push(
            Text::new("Are you sure you want to cancel this transaction?")
                .size(16)
                .style(iced::Color::from_rgb(0.9, 0.9, 0.9))
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .push(
            Container::new(
                Column::new()
                    .spacing(10)
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(Text::new("Transaction:").size(14))
                            .push(Text::new(tx_hash_display.clone()).size(14).style(iced::Color::from_rgb(0.7, 0.7, 0.7)))
                    )
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(Text::new("Network:").size(14))
                            .push(Text::new(format!("{:?}", pending_tx.network)).size(14).style(iced::Color::from_rgb(0.7, 0.7, 0.7)))
                    )
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(Text::new("Type:").size(14))
                            .push(Text::new(match pending_tx.tx_type {
                                TransactionType::Legacy => "Legacy",
                                TransactionType::Eip1559 => "EIP-1559",
                            }).size(14).style(iced::Color::from_rgb(0.7, 0.7, 0.7)))
                    )
            )
            .padding(15)
            .width(Length::Fill)
        )
        .push(
            Text::new("⚠️ This action will send a replacement transaction with higher gas fees to cancel the original transaction.")
                .size(12)
                .style(iced::Color::from_rgb(1.0, 0.8, 0.4))
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .push(
            Row::new()
                .spacing(15)
                .align_items(iced::Alignment::Center)
                .push(
                    Button::new(
                        Text::new("Cancel")
                            .size(16)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    )
                    .on_press(Message::HideCancelConfirmation)
                    .style(styles::secondary_button())
                    .padding(12)
                    .width(Length::Fixed(120.0))
                )
                .push(
                    Button::new(
                        Text::new("🚫 Confirm Cancel")
                            .size(16)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    )
                    .on_press(Message::ConfirmCancelTransaction)
                    .style(styles::danger_button())
                    .padding(12)
                    .width(Length::Fixed(160.0))
                )
        );

    Container::new(Container::new(dialog_content).padding(30).max_width(500))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
