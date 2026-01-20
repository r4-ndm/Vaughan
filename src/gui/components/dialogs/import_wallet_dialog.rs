//! Import Wallet Dialog Component
//!
//! This module contains the import wallet dialog UI component.
//! Extracted from views/dialogs.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, Row, Space, Text, TextInput},
    Element, Length,
};

use crate::gui::{theme::styles, wallet_types::ImportType, working_wallet::AppState, Message};

/// Import wallet dialog view
pub fn import_wallet_dialog_view(state: &AppState) -> Element<'_, Message> {
    let import_type = &state.wallet().import_type;

    let mut content = Column::new()
        .push(
            // Header
            Row::new()
                .push(
                    Button::new(Text::new("← Back"))
                        .on_press(Message::HideImportWallet)
                        .padding(8)
                        .style(styles::secondary_button()),
                )
                .push(Space::with_width(Length::Fixed(20.0)))
                .push(Text::new("Import Existing Wallet").size(20))
                .push(Space::with_width(Length::Fill))
                .align_items(iced::Alignment::Center),
        )
        .push(Space::with_height(Length::Fixed(30.0)))
        .push(
            // Import type selector
            Row::new()
                .push(
                    Button::new(Text::new("Seed Phrase").size(14))
                        .on_press(Message::ImportTypeSelected(ImportType::SeedPhrase))
                        .padding([10, 20])
                        .style(if matches!(import_type, ImportType::SeedPhrase) {
                            styles::primary_button()
                        } else {
                            styles::secondary_button()
                        })
                        .width(Length::Fill),
                )
                .push(Space::with_width(Length::Fixed(10.0)))
                .push(
                    Button::new(Text::new("Private Key").size(14))
                        .on_press(Message::ImportTypeSelected(ImportType::PrivateKey))
                        .padding([10, 20])
                        .style(if matches!(import_type, ImportType::PrivateKey) {
                            styles::primary_button()
                        } else {
                            styles::secondary_button()
                        })
                        .width(Length::Fill),
                )
                .spacing(0),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
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
        .push(Space::with_height(Length::Fixed(20.0)));

    // Add import-type-specific fields
    content = match import_type {
        ImportType::SeedPhrase => {
            content.push(
                // Seed Phrase Input
                Column::new()
                    .push(Text::new("Seed Phrase (12 or 24 words)").size(14))
                    .push(Space::with_height(Length::Fixed(8.0)))
                    .push(
                        TextInput::new("Enter your seed phrase...", &state.wallet().seed_phrase)
                            .on_input(Message::SeedPhraseChanged)
                            .padding(10)
                            .width(Length::Fill),
                    )
                    .spacing(5),
            )
        }
        ImportType::PrivateKey => {
            use secrecy::ExposeSecret;
            content.push(
                // Private Key Input
                Column::new()
                    .push(Text::new("Private Key (hex format)").size(14))
                    .push(Space::with_height(Length::Fixed(8.0)))
                    .push(
                        TextInput::new(
                            "Enter your private key (0x...)...",
                            state.wallet().private_key.expose_secret(),
                        )
                        .on_input(Message::PrivateKeyChanged)
                        .padding(10)
                        .width(Length::Fill)
                        .secure(true),
                    )
                    .spacing(5),
            )
        }
    };

    content = content
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
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
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
                    // Password validation warning
                    if !state.wallet().confirm_password.is_empty()
                        && !state.wallet().master_password.is_empty()
                        && state.wallet().master_password != state.wallet().confirm_password
                    {
                        Container::new(
                            Text::new("⚠️ Passwords do not match")
                                .size(12)
                                .style(iced::Color::from_rgb(1.0, 0.4, 0.4)),
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
            // Import button
            Button::new(Text::new(if state.wallet().creating_wallet {
                "Importing..."
            } else {
                "Import Wallet"
            }))
            .on_press_maybe({
                let name_valid = !state.wallet().wallet_name.is_empty();
                let password_valid = !state.wallet().master_password.is_empty()
                    && state.wallet().master_password == state.wallet().confirm_password;
                let not_busy = !state.wallet().creating_wallet;

                let input_valid = match import_type {
                    ImportType::SeedPhrase => !state.wallet().seed_phrase.is_empty(),
                    ImportType::PrivateKey => {
                        use secrecy::ExposeSecret;
                        !state.wallet().private_key.expose_secret().is_empty()
                    }
                };

                if name_valid && input_valid && password_valid && not_busy {
                    Some(match import_type {
                        ImportType::SeedPhrase => Message::CreateWalletFromSeed,
                        ImportType::PrivateKey => Message::ImportWalletFromPrivateKey,
                    })
                } else {
                    None
                }
            })
            .padding([12, 24])
            .style(styles::primary_button())
            .width(Length::Fill),
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
