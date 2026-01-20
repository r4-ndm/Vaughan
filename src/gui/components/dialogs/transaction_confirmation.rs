//! Transaction Confirmation Dialog
//!
//! Shows gas estimation and transaction details before final confirmation.
//! Includes password input when session is locked.

use iced::{
    widget::{Button, Checkbox, Column, Container, Row, Space, Text, TextInput},
    Color, Element, Length,
};

use crate::gui::state::AppState;
use crate::gui::{theme::styles, Message};

/// Transaction confirmation dialog view
pub fn transaction_confirmation_dialog_view(state: &AppState) -> Element<'_, Message> {
    let gas_estimation = state.transaction().gas_estimation.as_ref();
    let session_locked = !state.auth().session.is_unlocked;
    let password_error = state.auth().password_dialog.error.as_ref();

    let content = if let Some(estimation) = gas_estimation {
        let mut column = Column::new()
            .push(Text::new("Confirm Transaction").size(20).style(Color::WHITE))
            .push(Space::with_height(Length::Fixed(20.0)));

        // Transaction details
        column = column
            .push(
                Row::new()
                    .push(Text::new("To:").size(14).style(Color::from_rgb(0.7, 0.7, 0.7)))
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Text::new(&state.transaction().send_to_address)
                            .size(14)
                            .style(Color::WHITE),
                    )
                    .spacing(5),
            )
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(
                Row::new()
                    .push(Text::new("Amount:").size(14).style(Color::from_rgb(0.7, 0.7, 0.7)))
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Text::new(format!(
                            "{} {}",
                            state.transaction().send_amount,
                            state.transaction().send_selected_token
                        ))
                        .size(14)
                        .style(Color::WHITE),
                    )
                    .spacing(5),
            )
            .push(Space::with_height(Length::Fixed(20.0)));

        // Password section (only if session is locked)
        if session_locked {
            column = column
                .push(
                    Container::new(
                        Column::new()
                            .push(
                                Text::new("🔒 Session Locked")
                                    .size(14)
                                    .style(Color::from_rgb(1.0, 0.8, 0.2)),
                            )
                            .push(Space::with_height(Length::Fixed(10.0)))
                            .push(
                                Text::new("Enter your password to sign this transaction")
                                    .size(12)
                                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
                            )
                            .push(Space::with_height(Length::Fixed(15.0)))
                            .push({
                                use secrecy::ExposeSecret;
                                TextInput::new("Enter password", state.auth().password_dialog.input.expose_secret())
                                    .on_input(|input| Message::PasswordInputChanged(secrecy::SecretString::new(input)))
                                    .on_submit(Message::ConfirmTransaction)
                                    .secure(true)
                                    .padding(12)
                                    .width(Length::Fill)
                            })
                            .push(Space::with_height(Length::Fixed(10.0)))
                            .push(
                                Checkbox::new("Remember for 15 minutes", state.auth().password_dialog.remember_session)
                                    .on_toggle(Message::PasswordRememberChanged)
                                    .size(16)
                                    .text_size(12),
                            ),
                    )
                    .padding(15)
                    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.8, 0.2, 0.1))),
                        border: iced::Border {
                            color: Color::from_rgba(1.0, 0.8, 0.2, 0.3),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill),
                )
                .push(Space::with_height(Length::Fixed(15.0)));

            // Show password error if present
            if let Some(error) = password_error {
                column = column
                    .push(
                        Container::new(
                            Text::new(format!("{error}"))
                                .size(13)
                                .style(Color::from_rgb(1.0, 0.3, 0.3)),
                        )
                        .padding(10)
                        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.3, 0.3, 0.1))),
                            border: iced::Border {
                                color: Color::from_rgb(1.0, 0.3, 0.3),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        })
                        .width(Length::Fill),
                    )
                    .push(Space::with_height(Length::Fixed(15.0)));
            }
        }

        // Gas estimation section
        column = column.push(
            Container::new(
                Column::new()
                    .push(
                        Text::new("Gas Estimation")
                            .size(16)
                            .style(Color::from_rgb(0.9, 0.9, 0.9)),
                    )
                    .push(Space::with_height(Length::Fixed(10.0)))
                    .push(
                        Row::new()
                            .push(Text::new("Gas Limit:").size(13).style(Color::from_rgb(0.7, 0.7, 0.7)))
                            .push(Space::with_width(Length::Fill))
                            .push(
                                Text::new(format!("{} units", estimation.estimated_gas))
                                    .size(13)
                                    .style(Color::WHITE),
                            ),
                    )
                    .push(Space::with_height(Length::Fixed(8.0)))
                    .push(
                        Row::new()
                            .push(Text::new("Gas Price:").size(13).style(Color::from_rgb(0.7, 0.7, 0.7)))
                            .push(Space::with_width(Length::Fill))
                            .push(Text::new(&estimation.gas_price).size(13).style(Color::WHITE)),
                    )
                    .push(Space::with_height(Length::Fixed(8.0)))
                    .push(
                        Row::new()
                            .push(Text::new("Gas Cost:").size(13).style(Color::from_rgb(0.7, 0.7, 0.7)))
                            .push(Space::with_width(Length::Fill))
                            .push(Text::new(&estimation.estimated_cost).size(13).style(Color::WHITE)),
                    )
                    .push(Space::with_height(Length::Fixed(12.0)))
                    .push(
                        Container::new(
                            Row::new()
                                .push(Text::new("Total Cost:").size(14).style(Color::from_rgb(1.0, 0.8, 0.2)))
                                .push(Space::with_width(Length::Fill))
                                .push(
                                    Text::new(&estimation.total_cost)
                                        .size(14)
                                        .style(Color::from_rgb(1.0, 0.8, 0.2)),
                                ),
                        )
                        .padding(10)
                        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.8, 0.2, 0.1))),
                            border: iced::Border {
                                color: Color::from_rgba(1.0, 0.8, 0.2, 0.3),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                    )
                    .spacing(5),
            )
            .padding(15)
            .style(styles::dark_flat_container())
            .width(Length::Fill),
        );

        // Buttons
        column = column
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Row::new()
                    .push(
                        Button::new(Text::new("Cancel").size(14))
                            .on_press(Message::HideTransactionConfirmation)
                            .padding([10, 20])
                            .style(styles::secondary_button())
                            .width(Length::Fill),
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Button::new(
                            Text::new(if state.transaction().sending_transaction {
                                "Sending..."
                            } else if session_locked {
                                "Unlock & Send"
                            } else {
                                "Confirm & Send"
                            })
                            .size(14),
                        )
                        .on_press_maybe(if !state.transaction().sending_transaction {
                            Some(Message::ConfirmTransaction)
                        } else {
                            None
                        })
                        .padding([10, 20])
                        .style(styles::primary_button())
                        .width(Length::Fill),
                    )
                    .spacing(0),
            )
            .align_items(iced::Alignment::Center)
            .spacing(5);

        column
    } else {
        // Estimating gas
        Column::new()
            .push(Text::new("Estimating Gas...").size(20).style(Color::WHITE))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(Text::new("⟳").size(32).style(Color::from_rgb(0.5, 0.8, 1.0)))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Text::new("Please wait while we calculate gas costs")
                    .size(14)
                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
            )
            .align_items(iced::Alignment::Center)
            .spacing(5)
    };

    // Modal overlay
    Container::new(
        Container::new(content)
            .padding(30)
            .style(styles::dark_flat_container())
            .width(Length::Fixed(500.0)),
    )
    .padding(50)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
        ..Default::default()
    })
    .into()
}
