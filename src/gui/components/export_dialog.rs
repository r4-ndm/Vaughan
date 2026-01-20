//! Export Dialog Component
//!
//! This module contains the export dialog UI component.

use crate::gui::wallet_messages::Message;

use crate::gui::working_wallet::AppState;
use iced::widget::{Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Color, Element, Length};

pub struct ExportDialog;

impl ExportDialog {
    pub fn view(state: &AppState) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .push(Self::header())
                .push(Space::with_height(Length::Fixed(20.0)))
                .push(Self::export_content(state))
                .spacing(10)
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn header() -> Element<'static, Message> {
        Row::new()
            .push(
                Button::new(Text::new("← Back"))
                    .on_press(Message::HideExportWallet)
                    .padding(8),
            )
            .push(Space::with_width(Length::Fixed(20.0)))
            .push(Text::new("Export Wallet").size(20))
            .push(Space::with_width(Length::Fill))
            .align_items(Alignment::Center)
            .into()
    }

    fn export_content(state: &AppState) -> Element<'_, Message> {
        if state.wallet().export_loading {
            Self::exporting_step(state)
        } else if !state.wallet().export_result.is_empty() {
            Self::result_display_step(state)
        } else {
            Self::account_selection_step(state)
        }
    }

    fn account_selection_step(state: &AppState) -> Element<'_, Message> {
        let content = Column::new()
            .push(Text::new("Select Account and Export Type").size(16))
            .push(Space::with_height(Length::Fixed(20.0)));

        // Show error if present
        let content = if let Some(error) = &state.wallet().export_error_message {
            content
                .push(
                    Container::new(Text::new(error).style(Color::from_rgb(1.0, 0.4, 0.4)))
                        .padding(10)
                        .width(Length::Fill),
                )
                .push(Space::with_height(Length::Fixed(10.0)))
        } else {
            content
        };

        content
            .push(
                Column::new()
                    .push(Text::new("Account to Export").size(14))
                    .push(Space::with_height(Length::Fixed(5.0)))
                    .push(
                        // Account selector placeholder
                        Text::new(if let Some(account_id) = &state.wallet().current_account_id {
                            if let Some(account) =
                                state.wallet().available_accounts.iter().find(|a| &a.id == account_id)
                            {
                                let address_str = format!("{:#x}", account.address);
                                format!("{} ({}...)", account.name, &address_str[0..10])
                            } else {
                                "Account not found".to_string()
                            }
                        } else {
                            "No account selected".to_string()
                        })
                        .size(14)
                        .style(if state.wallet().current_account_id.is_some() {
                            Color::from_rgb(0.2, 0.8, 0.2)
                        } else {
                            Color::from_rgb(1.0, 0.4, 0.4)
                        }),
                    )
                    .spacing(5),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Column::new()
                    .push(Text::new("Export Type").size(14))
                    .push(Space::with_height(Length::Fixed(5.0)))
                    .push(
                        Text::new("Choose what you want to export. WARNING: Never share this information!")
                            .size(12)
                            .style(Color::from_rgb(1.0, 0.6, 0.2)),
                    )
                    .spacing(5),
            )
            .push(Space::with_height(Length::Fixed(30.0)))
            .push(Self::export_type_buttons())
            .spacing(10)
            .into()
    }

    fn export_type_buttons() -> Element<'static, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Export Seed Phrase"))
                    .on_press(Message::ExportSeedPhrase)
                    .padding(12)
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(15.0)))
            .push(
                Button::new(Text::new("Export Private Key"))
                    .on_press(Message::ExportPrivateKey)
                    .padding(12)
                    .width(Length::FillPortion(1)),
            )
            .spacing(10)
            .into()
    }

    fn result_display_step(state: &AppState) -> Element<'_, Message> {
        Column::new()
            .push(
                Text::new("Data Exported Successfully")
                    .size(16)
                    .style(Color::from_rgb(0.2, 0.8, 0.2)),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Container::new(
                    Column::new()
                        .push(Text::new(state.wallet().export_result.as_str()).size(12)) // Removed unwrap check since we check is_empty before calling
                        .push(Space::with_height(Length::Fixed(15.0)))
                        .push(
                            Button::new(Text::new("📋 Copy to Clipboard"))
                                .on_press(Message::CopyExportedData(
                                    state.wallet().export_result.clone()
                                ))
                                .padding(10)
                                .width(Length::Fill),
                        )
                        .spacing(5),
                )
                .padding(20)
                .width(Length::Fill),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Container::new(
                    Text::new(
                        "⚠️ This data will be automatically cleared from clipboard after 30 seconds for security.",
                    )
                    .size(11)
                    .style(Color::from_rgb(1.0, 0.6, 0.2)),
                )
                .padding(10)
                .width(Length::Fill),
            )
            .push(Space::with_height(Length::Fixed(30.0)))
            .push(
                Button::new(Text::new("Done"))
                    .on_press(Message::HideExportWallet)
                    .padding(12)
                    .width(Length::Fill),
            )
            .spacing(10)
            .into()
    }

    fn exporting_step(_state: &AppState) -> Element<'_, Message> {
        Column::new()
            .push(Text::new("Exporting...").size(16))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(Text::new("Please wait while we export your data securely.").size(14))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(Text::new("🔄 Processing...").size(12))
            .spacing(10)
            .into()
    }
}
