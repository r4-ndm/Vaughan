//! Receive Dialog Component
//!
//! Shows QR code and address for receiving payments.

use iced::{
    widget::{Button, Column, Container, Row, Space, Text, Tooltip},
    Color, Element, Length,
};

#[cfg(feature = "qr")]
use iced::widget::Image;

use crate::gui::{state::AppState, theme::styles, Message};

#[cfg(feature = "qr")]
use crate::gui::services::qr_service;

/// Receive dialog view
pub fn receive_dialog_view(state: &AppState) -> Element<'_, Message> {
    let receive_state = &state.wallet().receive_dialog;

    if !receive_state.visible {
        return Space::new(Length::Fill, Length::Fill).into();
    }

    // Get current account
    let current_account = if let Some(account_id) = &state.wallet().current_account_id {
        state.wallet().available_accounts.iter().find(|a| &a.id == account_id)
    } else {
        None
    };

    let content = if let Some(account) = current_account {
        let address_str = format!("{:?}", account.address);

        // Generate QR code (only when qr feature is enabled)
        #[cfg(feature = "qr")]
        let qr_section = {
            match qr_service::generate_address_qr_code(&address_str) {
                Ok(handle) => {
                    Container::new(
                        Image::new(handle)
                            .width(Length::Fixed(300.0))
                            .height(Length::Fixed(300.0))
                    )
                    .padding(20)
                    .style(styles::info_container())
                }
                Err(_) => {
                    Container::new(
                        Text::new("Failed to generate QR code")
                            .size(16)
                            .style(Color::from_rgb(1.0, 0.3, 0.3))
                    )
                    .padding(20)
                }
            }
        };

        #[cfg(not(feature = "qr"))]
        let qr_section = {
            Container::new(
                Text::new("QR code feature is disabled")
                    .size(14)
                    .style(Color::from_rgb(0.5, 0.5, 0.5))
            )
            .padding(20)
            .style(styles::info_container())
        };

        Column::new()
            .push(Text::new("Receive Payments").size(24).style(Color::WHITE))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Text::new(format!("Account: {}", account.name))
                    .size(16)
                    .style(Color::from_rgb(0.8, 0.8, 0.8))
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            // QR Code section
            .push(qr_section)
            .push(Space::with_height(Length::Fixed(20.0)))
            // Address display
            .push(
                Column::new()
                    .push(
                        Text::new("Your Address:")
                            .size(14)
                            .style(Color::from_rgb(0.7, 0.7, 0.7))
                    )
                    .push(Space::with_height(Length::Fixed(8.0)))
                    .push(
                        Container::new(
                            Row::new()
                                .push(
                                    Container::new(
                                        Text::new(address_str.clone())
                                            .size(12)
                                            .style(Color::WHITE)
                                    )
                                    .width(Length::Fill)
                                    .padding(12)
                                    .style(styles::dark_flat_container())
                                )
                                .push(Space::with_width(Length::Fixed(10.0)))
                                .push(
                                    Tooltip::new(
                                        Button::new(Text::new("📋").size(16))
                                            .on_press(Message::CopyToClipboard(address_str.clone()))
                                            .padding([8, 12])
                                            .style(styles::secondary_button()),
                                        "Copy to clipboard",
                                        iced::widget::tooltip::Position::Top,
                                    )
                                )
                                .align_items(iced::Alignment::Center)
                        )
                        .padding(15)
                        .style(styles::dark_flat_container())
                        .width(Length::Fill)
                    )
                    .spacing(5)
            )
            .push(Space::with_height(Length::Fixed(30.0)))
            // Buttons
            .push(
                Row::new()
                    .push(
                        Button::new(Text::new("Close").size(14))
                            .on_press(Message::HideReceiveDialog)
                            .padding([10, 20])
                            .style(styles::primary_button())
                            .width(Length::Fill)
                    )
                    .spacing(0)
            )
            .align_items(iced::Alignment::Center)
            .spacing(5)
    } else {
        Column::new()
            .push(
                Text::new("No Account Selected")
                    .size(20)
                    .style(Color::from_rgb(1.0, 0.3, 0.3)),
            )
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(
                Text::new("Please select an account to receive payments")
                    .size(14)
                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Button::new(Text::new("Close").size(14))
                    .on_press(Message::HideReceiveDialog)
                    .padding([10, 20])
                    .style(styles::primary_button()),
            )
            .align_items(iced::Alignment::Center)
            .spacing(5)
    };

    // Modal overlay
    Container::new(
        Container::new(content)
            .padding(30)
            .style(styles::dark_flat_container())
            .width(Length::Fixed(500.0))
            .height(Length::Shrink),
    )
    .padding(40)
    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
