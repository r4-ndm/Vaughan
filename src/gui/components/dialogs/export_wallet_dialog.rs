//! Export Wallet Dialog Component
//!
//! This module contains the export wallet dialog UI component.
//! Extracted from views/dialogs.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, Row, Space, Text},
    Element, Length,
};

use crate::gui::{
    theme::{styles, text},
    working_wallet::AppState,
    Message,
};

/// Export wallet dialog view
pub fn export_wallet_dialog_view(state: &AppState) -> Element<'_, Message> {
    let content = Column::new()
        .push(
            // Header
            Row::new()
                .push(
                    Button::new(Text::new("← Back"))
                        .on_press(Message::HideExportWallet)
                        .padding(8)
                        .style(styles::secondary_button()),
                )
                .push(Space::with_width(Length::Fixed(20.0)))
                .push(Text::new("Export Wallet").size(20))
                .push(Space::with_width(Length::Fill))
                .align_items(iced::Alignment::Center),
        )
        .push(Space::with_height(Length::Fixed(30.0)))
        .push(if state.exporting_data {
            // Show loading state while exporting
            Column::new()
                .push(Text::new("Exporting wallet data...").size(16))
                .push(Space::with_height(Length::Fixed(10.0)))
                .align_items(iced::Alignment::Center)
        } else if state.exported_seed_phrase.as_ref().is_some_and(|s| !s.is_empty())
            || state.exported_private_key.as_ref().is_some_and(|s| !s.is_empty())
        {
            // Show exported data based on what was exported
            let mut column = Column::new()
                .push(
                    Text::new("⚠️ IMPORTANT: Keep this information secure!")
                        .size(16)
                        .style(iced::Color::from_rgb(1.0, 0.6, 0.0)),
                )
                .push(Space::with_height(Length::Fixed(20.0)));

            // Show seed phrase if available
            if state.exported_seed_phrase.as_ref().is_some_and(|s| !s.is_empty()) {
                column = column.push(
                    Column::new()
                        .push(Text::new("Seed Phrase:").size(14))
                        .push(Space::with_height(Length::Fixed(8.0)))
                        .push(
                            Row::new()
                                .push(
                                    Container::new(
                                        Text::new(state.exported_seed_phrase.as_deref().unwrap_or(""))
                                            .size(14)
                                            .style(text::primary()),
                                    )
                                    .padding(15)
                                    .style(styles::info_container())
                                    .width(Length::Fill),
                                )
                                .push(Space::with_width(Length::Fixed(10.0)))
                                .push(
                                    Button::new(Text::new("📋 Copy"))
                                        .on_press(Message::CopyExportedData(
                                            state.exported_seed_phrase.clone().unwrap_or_default(),
                                        ))
                                        .padding([8, 12])
                                        .style(styles::secondary_button()),
                                )
                                .align_items(iced::Alignment::Center),
                        )
                        .spacing(5),
                );
            }

            // Show private key if available
            if let Some(ref key) = state.exported_private_key {
                if !key.is_empty() {
                    if state.exported_seed_phrase.as_ref().is_some_and(|s| !s.is_empty()) {
                        column = column.push(Space::with_height(Length::Fixed(20.0)));
                    }
                    column = column.push(
                        Column::new()
                            .push(Text::new("Private Key:").size(14))
                            .push(Space::with_height(Length::Fixed(8.0)))
                            .push(
                                Row::new()
                                    .push(
                                        Container::new(Text::new(key).size(12).style(text::primary()))
                                            .padding(15)
                                            .style(styles::info_container())
                                            .width(Length::Fill),
                                    )
                                    .push(Space::with_width(Length::Fixed(10.0)))
                                    .push(
                                        Button::new(Text::new("📋 Copy"))
                                            .on_press(Message::CopyExportedData(key.clone()))
                                            .padding([8, 12])
                                            .style(styles::secondary_button()),
                                    )
                                    .align_items(iced::Alignment::Center),
                            )
                            .spacing(5),
                    );
                }
            }

            // Add back button
            column = column.push(Space::with_height(Length::Fixed(30.0))).push(
                Row::new()
                    .push(Space::with_width(Length::Fill))
                    .push(
                        Button::new(Text::new("← Back to Export Options"))
                            .on_press(Message::CancelInlineExport)
                            .padding([10, 20])
                            .style(styles::secondary_button()),
                    )
                    .push(Space::with_width(Length::Fill)),
            );

            column.spacing(5)
        } else {
            // Show account selection and export buttons
            Column::new()
                .push(Text::new("Select account to export:").size(16))
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(
                    // Account selection dropdown
                    if state.wallet().available_accounts.is_empty() {
                        Container::new(
                            Text::new("No accounts available")
                                .size(14)
                                .style(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                        )
                        .padding(10)
                        .style(styles::info_container())
                    } else {
                        let selected_text = if let Some(ref account_id) = state.selected_export_account_id {
                            // Find the account name from the ID
                            state
                                .wallet()
                                .available_accounts
                                .iter()
                                .find(|acc| &acc.id == account_id)
                                .map(|acc| {
                                    let addr_str = format!("{:?}", acc.address);
                                    format!("{} ({}...)", acc.name, &addr_str[2..10])
                                })
                                .unwrap_or_else(|| "Select Account".to_string())
                        } else {
                            "Select Account".to_string()
                        };

                        Container::new(
                            Column::new()
                                .push(
                                    Button::new(
                                        Row::new()
                                            .push(Text::new(selected_text).size(14))
                                            .push(Space::with_width(Length::Fill))
                                            .push(Text::new("▼").size(12))
                                            .align_items(iced::Alignment::Center),
                                    )
                                    .on_press(Message::ToggleAccountDropdown)
                                    .padding(10)
                                    .width(Length::Fill)
                                    .style(styles::secondary_button()),
                                )
                                .push(if state.ui().show_account_dropdown {
                                    Column::new().push(Space::with_height(Length::Fixed(5.0))).push({
                                        let mut account_list = Column::new().spacing(2);
                                        for account in &state.wallet().available_accounts {
                                            account_list = account_list.push(
                                                Button::new(
                                                    Text::new({
                                                        let addr_str = format!("{:?}", account.address);
                                                        format!("{} ({}...)", account.name, &addr_str[2..10])
                                                    })
                                                    .size(13),
                                                )
                                                .on_press(Message::SelectExportAccount(account.id.clone()))
                                                .padding(8)
                                                .width(Length::Fill)
                                                .style(
                                                    if Some(&account.id) == state.selected_export_account_id.as_ref() {
                                                        styles::primary_button()
                                                    } else {
                                                        styles::secondary_button()
                                                    },
                                                ),
                                            );
                                        }
                                        Container::new(account_list).padding(5).style(styles::card_container())
                                    })
                                } else {
                                    Column::new()
                                }),
                        )
                        .width(Length::Fill)
                    },
                )
                .push(Space::with_height(Length::Fixed(20.0)))
                .push(
                    Text::new("⚠️ Warning: Anyone with this information can access your funds!")
                        .size(14)
                        .style(iced::Color::from_rgb(1.0, 0.4, 0.4)),
                )
                .push(Space::with_height(Length::Fixed(20.0)))
                .push({
                    if state.selected_export_account_id.is_some() {
                        Row::new()
                            .push(
                                Button::new(Text::new("Export Seed Phrase"))
                                    .on_press(Message::ExportSeedPhrase)
                                    .padding([12, 24])
                                    .style(styles::danger_button())
                                    .width(Length::FillPortion(1)),
                            )
                            .push(Space::with_width(Length::Fixed(10.0)))
                            .push(
                                Button::new(Text::new("Export Private Key"))
                                    .on_press(Message::ExportPrivateKey)
                                    .padding([12, 24])
                                    .style(styles::danger_button())
                                    .width(Length::FillPortion(1)),
                            )
                            .spacing(5)
                    } else {
                        Row::new().push(
                            Container::new(
                                Text::new("Please select an account first")
                                    .size(14)
                                    .style(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                            )
                            .center_x()
                            .width(Length::Fill),
                        )
                    }
                })
                .spacing(5)
        })
        .spacing(5);

    Container::new(
        Container::new(content)
            .padding(30)
            .style(styles::dark_flat_container())
            .width(Length::Fixed(600.0))
            .height(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
