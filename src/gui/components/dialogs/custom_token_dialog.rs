//! Custom Token Management Dialog Component
//!
//! This module contains the full custom token screen UI component for managing custom tokens.
//! Extracted from views/dialogs.rs for better code organization.

#![cfg_attr(not(feature = "custom-tokens"), allow(dead_code))]

use iced::{
    alignment::Horizontal,
    widget::{Button, Column, Container, Row, Scrollable, Space, Text, TextInput},
    Element, Length,
};

use crate::gui::{theme::styles, working_wallet::AppState, Message};

/// Create the custom token management screen view
#[cfg(feature = "custom-tokens")]
pub fn custom_token_screen_view(state: &AppState) -> Element<'_, Message> {
    let back_button = Button::new(
        Row::new()
            .push(Text::new("‹").size(18))
            .push(Space::with_width(Length::Fixed(5.0)))
            .push(Text::new("Back").size(14))
            .spacing(0)
            .align_items(iced::Alignment::Center),
    )
    .on_press(Message::HideCustomTokenScreen)
    .style(styles::secondary_button())
    .padding([8, 12]);

    let validation_message = if let Some(error) = &state.custom_token_validation_error {
        Container::new(Text::new(error).size(12).style(iced::Color::from_rgb(0.9, 0.3, 0.3)))
            .padding([8, 12])
            .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.9, 0.3, 0.3, 0.1))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.9, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill)
    } else {
        Container::new(Space::with_height(Length::Fixed(0.0)))
    };

    let address_input_section = Container::new(
        Column::new()
            .push(
                Row::new()
                    .push(Text::new("Contract Address").size(14).style(iced::Color::WHITE))
                    .push(Space::with_width(Length::Fill))
                    .push(
                        Text::new("* Required")
                            .size(10)
                            .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .push(Space::with_height(Length::Fixed(8.0)))
            .push(
                Row::new()
                    .push(
                        TextInput::new(
                            "Enter token contract address (0x...)...",
                            &state.custom_token_address_input,
                        )
                        .on_input(Message::CustomTokenAddressChanged)
                        .padding(12)
                        .size(14)
                        .width(Length::FillPortion(3)),
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Button::new(Text::new("Paste").size(12))
                            .on_press(Message::PasteTokenAddress)
                            .padding([12, 16])
                            .style(styles::secondary_button())
                            .width(Length::FillPortion(1)),
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Button::new(
                            Text::new(if state.fetching_token_info {
                                "Fetching..."
                            } else {
                                "Auto-Fetch"
                            })
                            .size(12),
                        )
                        .on_press_maybe(
                            if !state.custom_token_address_input.is_empty() && !state.fetching_token_info {
                                Some(Message::AutoFetchTokenInfo)
                            } else {
                                None
                            },
                        )
                        .padding([12, 16])
                        .style(styles::primary_button())
                        .width(Length::FillPortion(1)),
                    )
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(8.0)))
            .push(
                Text::new("Automatically fetch token information from the blockchain")
                    .size(10)
                    .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            )
            .spacing(0),
    )
    .padding(16)
    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
        border: iced::Border {
            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    let manual_input_section = Container::new(
        Column::new()
            .push(Text::new("Token Information").size(16).style(iced::Color::WHITE))
            .push(Space::with_height(Length::Fixed(8.0)))
            .push(
                Text::new("Manually enter token details or use Auto-Fetch to retrieve from blockchain")
                    .size(12)
                    .style(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            )
            .push(Space::with_height(Length::Fixed(16.0)))
            .push(
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                Text::new("Token Name")
                                    .size(12)
                                    .style(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                            )
                            .push(Space::with_height(Length::Fixed(6.0)))
                            .push(
                                TextInput::new("e.g., USD Coin", &state.custom_token_name_input)
                                    .on_input(Message::CustomTokenNameChanged)
                                    .padding(10)
                                    .size(12),
                            )
                            .width(Length::FillPortion(1)),
                    )
                    .push(Space::with_width(Length::Fixed(16.0)))
                    .push(
                        Column::new()
                            .push(
                                Text::new("Symbol/Ticker")
                                    .size(12)
                                    .style(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                            )
                            .push(Space::with_height(Length::Fixed(6.0)))
                            .push(
                                TextInput::new("e.g., USDC", &state.custom_token_symbol_input)
                                    .on_input(Message::CustomTokenSymbolChanged)
                                    .padding(10)
                                    .size(12),
                            )
                            .width(Length::FillPortion(1)),
                    )
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(16.0)))
            .push(
                Column::new()
                    .push(
                        Text::new("Decimals")
                            .size(12)
                            .style(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                    )
                    .push(Space::with_height(Length::Fixed(6.0)))
                    .push(
                        TextInput::new("18", &state.custom_token_decimals_input)
                            .on_input(Message::CustomTokenDecimalsChanged)
                            .padding(10)
                            .size(12)
                            .width(Length::Fixed(120.0)),
                    )
                    .push(Space::with_height(Length::Fixed(4.0)))
                    .push(
                        Text::new("Most ERC-20 tokens use 18 decimals")
                            .size(10)
                            .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                    )
                    .spacing(0),
            )
            .spacing(0),
    )
    .padding(16)
    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
        border: iced::Border {
            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    let existing_tokens_section = if !state.custom_tokens.is_empty() {
        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(Text::new("Your Custom Tokens").size(16).style(iced::Color::WHITE))
                        .push(Space::with_width(Length::Fill))
                        .push(
                            Text::new(format!("{} tokens", state.custom_tokens.len()))
                                .size(12)
                                .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                        )
                        .align_items(iced::Alignment::Center),
                )
                .push(Space::with_height(Length::Fixed(12.0)))
                .push(
                    Scrollable::new(
                        state
                            .custom_tokens
                            .iter()
                            .fold(Column::new().spacing(8), |column, token| {
                                let token_name = token.name.clone();
                                let token_symbol = token.symbol.clone();
                                let token_decimals = token.decimals;
                                let token_address = token.address.clone();
                                let short_address = format!(
                                    "{}...{}",
                                    &token_address[0..8],
                                    &token_address[token_address.len() - 6..]
                                );

                                column.push(
                                    Container::new(
                                        Row::new()
                                            .push(
                                                Column::new()
                                                    .push(Text::new(token_name).size(14).style(iced::Color::WHITE))
                                                    .push(
                                                        Row::new()
                                                            .push(
                                                                Text::new(token_symbol)
                                                                    .size(12)
                                                                    .style(iced::Color::from_rgb(0.4, 0.8, 0.4)),
                                                            )
                                                            .push(
                                                                Text::new(" • ")
                                                                    .size(12)
                                                                    .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                                            )
                                                            .push(
                                                                Text::new(format!("{token_decimals} decimals"))
                                                                    .size(12)
                                                                    .style(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                                            )
                                                            .push(
                                                                Text::new(" • ")
                                                                    .size(12)
                                                                    .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                                            )
                                                            .push(
                                                                Text::new(short_address)
                                                                    .size(12)
                                                                    .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                                                            )
                                                            .spacing(0)
                                                            .align_items(iced::Alignment::Center),
                                                    )
                                                    .spacing(4),
                                            )
                                            .push(Space::with_width(Length::Fill))
                                            .push(
                                                Button::new(
                                                    Text::new("Remove")
                                                        .size(11)
                                                        .horizontal_alignment(Horizontal::Center),
                                                )
                                                .on_press(Message::RemoveCustomToken(token_address))
                                                .padding([6, 12])
                                                .style(styles::danger_button()),
                                            )
                                            .align_items(iced::Alignment::Center)
                                            .spacing(0),
                                    )
                                    .padding(12)
                                    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                                            0.0, 0.0, 0.0, 0.3,
                                        ))),
                                        border: iced::Border {
                                            color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                                            width: 1.0,
                                            radius: 4.0.into(),
                                        },
                                        ..Default::default()
                                    })
                                    .width(Length::Fill),
                                )
                            }),
                    )
                    .height(Length::Fixed(200.0))
                    .width(Length::Fill),
                )
                .spacing(0),
        )
        .padding(16)
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
            border: iced::Border {
                color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
    } else {
        Container::new(
            Column::new()
                .push(
                    Text::new("No Custom Tokens Added Yet")
                        .size(14)
                        .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                )
                .push(Space::with_height(Length::Fixed(8.0)))
                .push(
                    Text::new("Add your first custom token using the contract address above")
                        .size(12)
                        .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                )
                .align_items(iced::Alignment::Center)
                .spacing(0),
        )
        .padding(16)
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
            border: iced::Border {
                color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
    };

    let action_buttons = Row::new()
        .push(
            Button::new(Text::new("Cancel").size(14).horizontal_alignment(Horizontal::Center))
                .on_press(Message::HideCustomTokenScreen)
                .style(styles::secondary_button())
                .padding([12, 24])
                .width(Length::FillPortion(1)),
        )
        .push(Space::with_width(Length::Fixed(16.0)))
        .push(
            Button::new(Text::new("Add Token").size(14).horizontal_alignment(Horizontal::Center))
                .on_press_maybe(
                    if !state.custom_token_address_input.is_empty()
                        && !state.custom_token_symbol_input.is_empty()
                        && !state.custom_token_decimals_input.is_empty()
                        && !state.fetching_token_info
                    {
                        Some(Message::CreateCustomTokenManually)
                    } else {
                        None
                    },
                )
                .style(styles::primary_button())
                .padding([12, 24])
                .width(Length::FillPortion(1)),
        )
        .spacing(0);

    Container::new(
        Column::new()
            .push(
                Container::new(
                    Row::new()
                        .push(back_button)
                        .push(Space::with_width(Length::Fill))
                        .push(
                            Text::new("Add Custom Token")
                                .size(20)
                                .style(iced::Color::WHITE),
                        )
                        .push(Space::with_width(Length::Fill))
                        .push(Space::with_width(Length::Fixed(100.0))) // Balance the back button
                        .align_items(iced::Alignment::Center),
                )
                .padding([20, 20, 10, 20])
                .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.7),
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }),
            )
            .push(
                Container::new(Scrollable::new(
                    Column::new()
                        .push(validation_message)
                        .push(Space::with_height(Length::Fixed(20.0)))
                        .push(address_input_section)
                        .push(Space::with_height(Length::Fixed(20.0)))
                        .push(manual_input_section)
                        .push(Space::with_height(Length::Fixed(20.0)))
                        .push(existing_tokens_section)
                        .push(Space::with_height(Length::Fixed(30.0)))
                        .push(action_buttons)
                        .spacing(0),
                ))
                .padding([0, 20, 20, 20]),
            )
            .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.1))),
        ..Default::default()
     })
     .into()
}

/// Placeholder when custom-tokens feature is disabled
#[cfg(not(feature = "custom-tokens"))]
pub fn custom_token_screen_view(_state: &AppState) -> Element<'_, Message> {
    Container::new(
        Text::new("Custom tokens feature is disabled")
            .size(14)
    )
    .padding([20, 20])
    .width(Length::Fill)
    .into()
}
