//! Create Wallet Dialog Component
//!
//! This module contains the create wallet dialog UI component.
//! Extracted from views/dialogs.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, PickList, Row, Space, Text, TextInput},
    Element, Length,
};

use crate::gui::{theme::styles, working_wallet::AppState, Message};
use crate::security::SeedStrength;

/// Create wallet dialog view
pub fn create_wallet_dialog_view(state: &AppState) -> Element<'_, Message> {
    let content = Column::new()
        .push(
            // Header
            Row::new()
                .push(
                    Button::new(Text::new("← Back"))
                        .on_press(Message::HideCreateWallet)
                        .padding(8)
                        .style(styles::secondary_button()),
                )
                .push(Space::with_width(Length::Fixed(20.0)))
                .push(Text::new("Create New Wallet").size(20))
                .push(Space::with_width(Length::Fill))
                .align_items(iced::Alignment::Center),
        )
        .push(Space::with_height(Length::Fixed(30.0)))
        .push(
            // Wallet Name
            Column::new()
                .push(Text::new("Wallet Name").size(14))
                .push(Space::with_height(Length::Fixed(8.0)))
                .push(
                    TextInput::new("Enter wallet name...", &state.wallet().wallet_name)
                        .on_input(Message::WalletNameChanged)
                        .padding(10)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            // Seed Strength Selection
            Column::new()
                .push(Text::new("Seed Phrase Length").size(14))
                .push(Space::with_height(Length::Fixed(8.0)))
                .push(
                    PickList::new(
                        vec![SeedStrength::Words12, SeedStrength::Words24],
                        Some(state.wallet().selected_seed_strength),
                        Message::SeedStrengthSelected,
                    )
                    .padding([8, 2, 8, 8])
                    .style(styles::dark_grey_pick_list())
                    .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            // Generated Seed Display - Always visible
            Column::new()
                .push(Text::new("Generated Seed Phrase").size(14))
                .push(Space::with_height(Length::Fixed(8.0)))
                .push(
                    Container::new(
                        Text::new(if state.wallet().seed_phrase.is_empty() {
                            "Click 'Generate Seed' to create a new seed phrase"
                        } else {
                            &state.wallet().seed_phrase
                        })
                        .size(12)
                        .style(if state.wallet().seed_phrase.is_empty() {
                            iced::Color::from_rgb(0.6, 0.6, 0.6) // Gray for placeholder
                        } else {
                            iced::Color::from_rgb(0.9, 0.9, 0.9) // Normal text color
                        }),
                    )
                    .padding(15)
                    .style(styles::dark_flat_container())
                    .width(Length::Fill),
                )
                .push(Space::with_height(Length::Fixed(10.0)))
                .push(
                    Text::new(if state.wallet().seed_phrase.is_empty() {
                        "💡 Your seed phrase will appear here after generation"
                    } else {
                        "⚠️ Write down your seed phrase and store it safely!"
                    })
                    .size(12)
                    .style(if state.wallet().seed_phrase.is_empty() {
                        iced::Color::from_rgb(0.6, 0.6, 0.6)
                    } else {
                        iced::Color::from_rgb(1.0, 0.6, 0.2) // Warning orange
                    }),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            // Master Password
            Column::new()
                .push(Text::new("Master Password").size(14))
                .push(Space::with_height(Length::Fixed(8.0)))
                .push(
                    TextInput::new("Enter master password...", &state.wallet().master_password)
                        .on_input(Message::MasterPasswordChanged)
                        .secure(true)
                        .padding(10)
                        .width(Length::Fill),
                )
                .push(Container::new(Space::with_height(Length::Fixed(0.0))))
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            // Confirm Password
            Column::new()
                .push(Text::new("Confirm Password").size(14))
                .push(Space::with_height(Length::Fixed(8.0)))
                .push(
                    TextInput::new("Confirm master password...", &state.wallet().confirm_password)
                        .on_input(Message::ConfirmPasswordChanged)
                        .secure(true)
                        .padding(10)
                        .width(Length::Fill),
                )
                .push(
                    // Password match warning
                    if !state.wallet().confirm_password.is_empty()
                        && !state.wallet().master_password.is_empty()
                        && state.wallet().master_password != state.wallet().confirm_password
                    {
                        Container::new(
                            Text::new("⚠️ Passwords do not match")
                                .size(12)
                                .style(iced::Color::from_rgb(1.0, 0.4, 0.4)), // Red warning
                        )
                        .padding([5, 0, 0, 0])
                    } else {
                        Container::new(Space::with_height(Length::Fixed(0.0)))
                    },
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            // Action buttons
            Row::new()
                .push(
                    Button::new(Text::new("Generate Seed"))
                        .on_press_maybe(if !state.wallet().generating_seed {
                            Some(Message::GenerateNewSeed)
                        } else {
                            None
                        })
                        .padding([10, 16])
                        .style(styles::secondary_button())
                        .width(Length::FillPortion(1)),
                )
                .push(Space::with_width(Length::Fixed(10.0)))
                .push(
                    Button::new(Text::new(if state.wallet().creating_wallet {
                        "Creating..."
                    } else {
                        "Create Wallet"
                    }))
                    .on_press_maybe(
                        if !state.wallet().wallet_name.is_empty()
                            && !state.wallet().seed_phrase.is_empty()
                            && !state.wallet().master_password.is_empty()
                            && state.wallet().master_password == state.wallet().confirm_password
                            && !state.wallet().creating_wallet
                        {
                            Some(Message::CreateWalletFromSeed)
                        } else {
                            None
                        },
                    )
                    .padding([10, 16])
                    .style(styles::primary_button())
                    .width(Length::FillPortion(1)),
                )
                .spacing(10),
        )
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
