//! Network Selector Component
//!
//! This module contains the network selector UI component.

use crate::gui::wallet_messages::Message;
use crate::gui::working_wallet::AppState;
use iced::widget::{Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Color, Element, Length};

pub struct NetworkSelector;

impl NetworkSelector {
    pub fn view(state: &AppState) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .push(Self::network_dropdown(state))
                .push(Space::with_height(Length::Fixed(10.0)))
                .push(Self::network_info(state))
                .push(Space::with_height(Length::Fixed(10.0)))
                .push(Self::network_actions())
                .spacing(5)
                .padding(15),
        )
        .width(Length::Fill)
        .into()
    }

    fn network_dropdown(state: &AppState) -> Element<'_, Message> {
        Row::new()
            .push(Text::new("Network:").size(14).style(Color::from_rgb(0.7, 0.7, 0.7)))
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(
                // Network picker placeholder - will implement with proper conversion
                Text::new(
                    state
                        .network()
                        .available_networks
                        .iter()
                        .find(|n| n.id == state.network().current_network)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                )
                .size(14),
            )
            .align_items(Alignment::Center)
            .into()
    }

    fn network_info(state: &AppState) -> Element<'_, Message> {
        Column::new()
            .push(
                Row::new()
                    .push(Text::new("Chain ID:").size(12).style(Color::from_rgb(0.6, 0.6, 0.6)))
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Text::new(
                            state
                                .network()
                                .available_networks
                                .iter()
                                .find(|n| n.id == state.network().current_network)
                                .map(|n| n.chain_id.to_string())
                                .unwrap_or_else(|| "Unknown".to_string()),
                        )
                        .size(12),
                    )
                    .align_items(Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Text::new("Currency:").size(12).style(Color::from_rgb(0.6, 0.6, 0.6)))
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Text::new(
                            state
                                .network()
                                .available_networks
                                .iter()
                                .find(|n| n.id == state.network().current_network)
                                .map(|n| n.symbol.clone())
                                .unwrap_or_else(|| "ETH".to_string()),
                        )
                        .size(12),
                    )
                    .align_items(Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Text::new("Type:").size(12).style(Color::from_rgb(0.6, 0.6, 0.6)))
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Text::new(
                            if let Some(network) = state
                                .network()
                                .available_networks
                                .iter()
                                .find(|n| n.id == state.network().current_network)
                            {
                                if network.is_custom {
                                    "Custom"
                                } else {
                                    "Default"
                                }
                            } else {
                                "Unknown"
                            },
                        )
                        .size(12)
                        .style(
                            if let Some(network) = state
                                .network()
                                .available_networks
                                .iter()
                                .find(|n| n.id == state.network().current_network)
                            {
                                if network.is_custom {
                                    Color::from_rgb(0.8, 0.6, 0.2) // Orange for custom
                                } else {
                                    Color::from_rgb(0.2, 0.8, 0.2) // Green for default
                                }
                            } else {
                                Color::from_rgb(0.7, 0.7, 0.7) // Gray for unknown
                            },
                        ),
                    )
                    .align_items(Alignment::Center),
            )
            .spacing(3)
            .into()
    }

    fn network_actions() -> Element<'static, Message> {
        Row::new()
            .push(
                Button::new(Text::new("➕ Add Network").size(12))
                    .on_press(Message::ShowAddNetwork)
                    .padding([6, 12])
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(
                Button::new(Text::new("⚙️ Settings").size(12))
                    .on_press(Message::ShowSettings)
                    .padding([6, 12])
                    .width(Length::FillPortion(1)),
            )
            .spacing(5)
            .into()
    }
}
