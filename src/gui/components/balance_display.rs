//! Balance Display Component
//!
//! This module contains the balance display UI component.

use crate::gui::wallet_messages::Message;
use crate::gui::working_wallet::AppState;
use iced::widget::{Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Color, Element, Length};

pub struct BalanceDisplay;

impl BalanceDisplay {
    pub fn view(state: &AppState) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .push(Self::main_balance(state))
                .push(Space::with_height(Length::Fixed(10.0)))
                .push(Self::account_info(state))
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(Self::action_buttons())
                .spacing(10)
                .padding(20),
        )
        .width(Length::Fill)
        .into()
    }

    fn main_balance(state: &AppState) -> Element<'_, Message> {
        let balance_text = format!(
            "{} {}",
            state.network().balance,
            state
                .network()
                .available_networks
                .iter()
                .find(|n| n.id == state.network().current_network)
                .map(|n| n.symbol.clone())
                .unwrap_or_else(|| "ETH".to_string())
        );
        Column::new()
            .push(Text::new("Balance").size(16).style(Color::from_rgb(0.7, 0.7, 0.7)))
            .push(
                Text::new(balance_text)
                    .size(32)
                    .style(if state.network().balance.contains("0.0000") {
                        Color::from_rgb(1.0, 0.4, 0.4) // Red for zero balance
                    } else {
                        Color::from_rgb(0.2, 0.8, 0.2) // Green for positive balance
                    }),
            )
            .align_items(Alignment::Center)
            .spacing(5)
            .into()
    }

    fn account_info(state: &AppState) -> Element<'_, Message> {
        if let Some(account_id) = &state.wallet().current_account_id {
            if let Some(account) = state.wallet().available_accounts.iter().find(|a| &a.id == account_id) {
                let address_str = format!("{:#x}", account.address);
                let address_display = format!("{}...{}", &address_str[0..6], &address_str[address_str.len() - 4..]);
                Column::new()
                    .push(
                        Text::new("Current Account")
                            .size(14)
                            .style(Color::from_rgb(0.7, 0.7, 0.7)),
                    )
                    .push(Text::new(&account.name).size(18))
                    .push(
                        Row::new()
                            .push(
                                Text::new(address_display)
                                    .size(12)
                                    .style(Color::from_rgb(0.6, 0.6, 0.6)),
                            )
                            .push(Space::with_width(Length::Fixed(10.0)))
                            .push(
                                Button::new(Text::new("📋").size(12))
                                    .on_press(Message::CopyAddress(format!("{:#x}", account.address)))
                                    .padding([2, 6]),
                            )
                            .align_items(Alignment::Center),
                    )
                    .align_items(Alignment::Center)
                    .spacing(5)
                    .into()
            } else {
                Text::new("Account not found")
                    .size(14)
                    .style(Color::from_rgb(1.0, 0.4, 0.4))
                    .into()
            }
        } else {
            Text::new("No account selected")
                .size(14)
                .style(Color::from_rgb(1.0, 0.4, 0.4))
                .into()
        }
    }

    fn action_buttons() -> Element<'static, Message> {
        Row::new()
            .push(
                Button::new(Text::new("🔄 Refresh"))
                    .on_press(Message::RefreshBalance)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(
                Button::new(Text::new("📤 Send"))
                    .on_press(Message::ShowSend)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(
                Button::new(Text::new("📥 Receive"))
                    .on_press(Message::ShowReceive)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            )
            .spacing(10)
            .into()
    }
}
