//! Account Manager Component
//!
//! This module contains the account management UI component.

use crate::gui::wallet_messages::Message;
use crate::gui::working_wallet::AppState;
use iced::widget::{Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Color, Element, Length};

pub struct AccountManager;

fn selected_account_style(_theme: &iced::Theme) -> iced::widget::container::Appearance {
    iced::widget::container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.3, 0.1))),
        border: iced::Border {
            width: 2.0,
            color: Color::from_rgb(0.2, 0.8, 0.2),
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn normal_account_style(_theme: &iced::Theme) -> iced::widget::container::Appearance {
    iced::widget::container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
        border: iced::Border {
            width: 1.0,
            color: Color::from_rgb(0.3, 0.3, 0.3),
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

impl AccountManager {
    pub fn view(state: &AppState) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .push(Self::header())
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(Self::account_list(state))
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(Self::account_actions())
                .spacing(10)
                .padding(20),
        )
        .width(Length::Fill)
        .into()
    }

    fn header() -> Element<'static, Message> {
        Row::new()
            .push(Text::new("Account Management").size(18))
            .push(Space::with_width(Length::Fill))
            .align_items(Alignment::Center)
            .into()
    }

    fn account_list(state: &AppState) -> Element<'_, Message> {
        if state.wallet().available_accounts.is_empty() {
            Column::new()
                .push(
                    Text::new("No accounts found")
                        .size(16)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                )
                .push(
                    Text::new("Create or import an account to get started")
                        .size(12)
                        .style(Color::from_rgb(0.5, 0.5, 0.5)),
                )
                .align_items(Alignment::Center)
                .spacing(5)
                .into()
        } else {
            let mut column = Column::new().spacing(10);

            for account in &state.wallet().available_accounts {
                let is_selected = state
                    .wallet()
                    .current_account_id
                    .as_ref()
                    .map(|selected_id| selected_id == &account.id)
                    .unwrap_or(false);

                let account_row = Container::new(
                    Row::new()
                        .push(
                            Column::new()
                                .push(Text::new(&account.name).size(14))
                                .push(
                                    Text::new({
                                        let address_str = format!("{:#x}", account.address);
                                        format!("{}...{}", &address_str[0..6], &address_str[address_str.len() - 4..])
                                    })
                                    .size(12)
                                    .style(Color::from_rgb(0.6, 0.6, 0.6)),
                                )
                                .spacing(3)
                                .width(Length::FillPortion(2)),
                        )
                        .push(Space::with_width(Length::Fixed(10.0)))
                        .push(
                            Text::new("0.0000 ETH") // TODO: Get actual balance
                                .size(14)
                                .style(Color::from_rgb(0.2, 0.8, 0.2))
                                .width(Length::FillPortion(1)),
                        )
                        .push(Space::with_width(Length::Fixed(10.0)))
                        .push(
                            Button::new(Text::new("Select").size(12))
                                .on_press(Message::AccountSelected(account.id.clone()))
                                .padding([4, 8]),
                        )
                        .align_items(Alignment::Center)
                        .spacing(10),
                )
                .padding(15)
                .width(Length::Fill)
                .style(if is_selected {
                    selected_account_style
                } else {
                    normal_account_style
                });

                column = column.push(account_row);
            }

            column.into()
        }
    }

    fn account_actions() -> Element<'static, Message> {
        Row::new()
            .push(
                Button::new(Text::new("➕ Create Account"))
                    .on_press(Message::ShowCreateWallet)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(
                Button::new(Text::new("📥 Import Wallet"))
                    .on_press(Message::ShowImportWallet)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(
                Button::new(Text::new("📤 Export"))
                    .on_press(Message::ShowExportWallet)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            )
            .spacing(10)
            .into()
    }
}
