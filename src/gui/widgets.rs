//! Professional widget system for Vaughan wallet - Iced 0.12 compatible
//!
//! This module provides reusable, themed widgets that maintain consistency
//! across the application while following Iced 0.12 best practices.

use iced::{
    alignment::Horizontal,
    widget::{Button, Checkbox, Column, Container, PickList, ProgressBar, Row, Scrollable, Space, Text, TextInput},
    Background, Border, Color, Element, Length,
};

use crate::gui::{
    theme::{styles, text, VaughanColors},
    Message,
};
use crate::network::NetworkId;

pub mod transaction;
pub use transaction::*;

/// Create a professional card-style container
pub fn card<'a, Message: 'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    Container::new(content)
        .style(styles::card_container())
        .padding(20)
        .width(Length::Fill)
        .into()
}

/// Create a primary action button with professional styling
pub fn primary_button<'a>(label: &'a str, message: Option<Message>) -> Element<'a, Message> {
    let mut button = Button::new(
        Text::new(label)
            .size(14)
            .style(text::primary())
            .horizontal_alignment(Horizontal::Center),
    )
    .style(styles::primary_button())
    .padding([12, 24]);

    if let Some(msg) = message {
        button = button.on_press(msg);
    }

    button.into()
}

/// Create a secondary action button
pub fn secondary_button<'a>(label: &'a str, message: Option<Message>) -> Element<'a, Message> {
    let mut button = Button::new(
        Text::new(label)
            .size(14)
            .style(text::secondary())
            .horizontal_alignment(Horizontal::Center),
    )
    .style(styles::secondary_button())
    .padding([12, 24]);

    if let Some(msg) = message {
        button = button.on_press(msg);
    }

    button.into()
}

/// Create a success action button
pub fn success_button<'a>(label: &'a str, message: Option<Message>) -> Element<'a, Message> {
    let mut button = Button::new(
        Text::new(label)
            .size(14)
            .style(text::primary())
            .horizontal_alignment(Horizontal::Center),
    )
    .style(styles::success_button())
    .padding([12, 24]);

    if let Some(msg) = message {
        button = button.on_press(msg);
    }

    button.into()
}

/// Create a warning action button
pub fn warning_button<'a>(label: &'a str, message: Option<Message>) -> Element<'a, Message> {
    let mut button = Button::new(
        Text::new(label)
            .size(14)
            .style(text::primary())
            .horizontal_alignment(Horizontal::Center),
    )
    .style(styles::warning_button())
    .padding([12, 24]);

    if let Some(msg) = message {
        button = button.on_press(msg);
    }

    button.into()
}

/// Create a professional text input field
pub fn text_input<'a>(
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    TextInput::new(placeholder, value)
        .on_input(on_change)
        .style(styles::primary_text_input())
        .padding(12)
        .size(14)
        .width(Length::Fill)
        .into()
}

/// Create a labeled text input field
pub fn labeled_text_input<'a>(
    label: &'a str,
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    Column::new()
        .push(Text::new(label).size(12).style(text::secondary()))
        .push(Space::with_height(Length::Fixed(5.0)))
        .push(text_input(placeholder, value, on_change))
        .spacing(0)
        .width(Length::Fill)
        .into()
}

/// Create a network selector dropdown
pub fn network_selector<'a>(
    current_network: NetworkId,
    on_change: impl Fn(NetworkId) -> Message + 'a,
) -> Element<'a, Message> {
    let networks = [
        ("Ethereum", NetworkId(1)),
        ("BSC", NetworkId(56)),
        ("Polygon", NetworkId(137)),
        ("PulseChain", NetworkId(369)),
    ];

    let current_name = networks
        .iter()
        .find(|(_, id)| *id == current_network)
        .map(|(name, _)| *name)
        .unwrap_or("Unknown");

    PickList::new(
        networks.iter().map(|(name, _)| *name).collect::<Vec<&str>>(),
        Some(current_name),
        move |selected: &str| {
            let network_id = networks
                .iter()
                .find(|(name, _)| *name == selected)
                .map(|(_, id)| *id)
                .unwrap_or(NetworkId(1));
            on_change(network_id)
        },
    )
    .placeholder("Select Network")
    .width(Length::Fixed(150.0))
    .into()
}

/// Create a connection status indicator
pub fn connection_status<'a>(connected: bool) -> Element<'a, Message> {
    let (icon, text_content, color) = if connected {
        ("🟢", "Connected", VaughanColors::SUCCESS)
    } else {
        ("🔴", "Disconnected", VaughanColors::ERROR)
    };

    Row::new()
        .push(Text::new(icon).size(12))
        .push(Space::with_width(Length::Fixed(5.0)))
        .push(Text::new(text_content).size(12).style(iced::theme::Text::Color(color)))
        .align_items(iced::Alignment::Center)
        .into()
}

/// Create a balance display widget
pub fn balance_display<'a>(symbol: &str, balance: &str, usd_value: &str) -> Element<'a, Message> {
    card(
        Column::new()
            .push(Text::new("Balance").size(14).style(text::secondary()))
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(Text::new(format!("{balance} {symbol}")).size(24).style(text::primary()))
            .push(Space::with_height(Length::Fixed(5.0)))
            .push(Text::new(format!("≈ {usd_value}")).size(14).style(text::muted()))
            .spacing(0)
            .align_items(iced::Alignment::Center)
            .into(),
    )
}

/// Create a theme switcher toggle
pub fn theme_switcher<'a>(is_dark: bool, on_toggle: Message) -> Element<'a, Message> {
    Row::new()
        .push(Text::new("Dark Mode").size(12).style(text::secondary()))
        .push(Space::with_width(Length::Fixed(10.0)))
        .push(
            Checkbox::new("", is_dark)
                .on_toggle(move |_| on_toggle.clone())
                .size(16),
        )
        .align_items(iced::Alignment::Center)
        .into()
}

/// Create a loading indicator
pub fn loading_indicator<'a>(message: &'a str) -> Element<'a, Message> {
    Column::new()
        .push(ProgressBar::new(0.0..=1.0, 0.5).width(Length::Fixed(200.0)))
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(Text::new(message).size(12).style(text::secondary()))
        .spacing(0)
        .align_items(iced::Alignment::Center)
        .into()
}

/// Create a status message widget
pub fn status_message<'a>(message: &'a str, is_error: bool) -> Element<'a, Message> {
    let style = if is_error { text::error() } else { text::success() };

    Container::new(Text::new(message).size(12).style(style))
        .padding(10)
        .style(move |_theme: &iced::Theme| {
            let color = if is_error {
                VaughanColors::ERROR
            } else {
                VaughanColors::SUCCESS
            };

            iced::widget::container::Appearance {
                text_color: Some(color),
                background: Some(Background::Color(Color::from_rgba(color.r, color.g, color.b, 0.1))),
                border: Border {
                    color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .width(Length::Fill)
        .into()
}

/// Create a section header
pub fn section_header<'a>(title: &'a str) -> Element<'a, Message> {
    Column::new()
        .push(Text::new(title).size(18).style(text::primary()))
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(
            Container::new(Space::with_height(Length::Fixed(1.0)))
                .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                    text_color: None,
                    background: Some(Background::Color(VaughanColors::BORDER_PRIMARY)),
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                })
                .width(Length::Fill),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .spacing(0)
        .into()
}

/// Create a scrollable content area
pub fn scrollable_content<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    Scrollable::new(content).width(Length::Fill).height(Length::Fill).into()
}

/// Create a two-column layout
pub fn two_column_layout<'a>(left: Element<'a, Message>, right: Element<'a, Message>) -> Element<'a, Message> {
    Row::new()
        .push(Container::new(left).width(Length::FillPortion(1)))
        .push(Space::with_width(Length::Fixed(20.0)))
        .push(Container::new(right).width(Length::FillPortion(1)))
        .spacing(0)
        .into()
}

/// Create a centered content container
pub fn centered_content<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

/// Create a professional tooltip-style container
pub fn tooltip_container<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    Container::new(content)
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(Background::Color(VaughanColors::BACKGROUND_SECONDARY)),
            border: Border {
                color: VaughanColors::BORDER_PRIMARY,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        })
        .padding(8)
        .into()
}

/// Create a professional error container
pub fn error_container<'a>(message: &'a str) -> Element<'a, Message> {
    Container::new(Text::new(message).size(12).style(text::error()))
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(
                VaughanColors::ERROR.r,
                VaughanColors::ERROR.g,
                VaughanColors::ERROR.b,
                0.1,
            ))),
            border: Border {
                color: VaughanColors::ERROR,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill)
        .into()
}

/// Create a professional success container
pub fn success_container<'a>(message: &'a str) -> Element<'a, Message> {
    Container::new(Text::new(message).size(12).style(text::success()))
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(
                VaughanColors::SUCCESS.r,
                VaughanColors::SUCCESS.g,
                VaughanColors::SUCCESS.b,
                0.1,
            ))),
            border: Border {
                color: VaughanColors::SUCCESS,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_selector_creation() {
        // Test that network selector can be created without panicking
        let current_network = NetworkId(1);
        let _selector = network_selector(current_network, |_| Message::RefreshBalance);
    }

    #[test]
    fn test_connection_status_display() {
        // Test connection status widget creation
        let _connected = connection_status(true);
        let _disconnected = connection_status(false);
    }

    #[test]
    fn test_button_creation() {
        // Test button widget creation
        let _primary = primary_button("Test", Some(Message::RefreshBalance));
        let _secondary = secondary_button("Test", None);
        let _success = success_button("Test", Some(Message::RefreshBalance));
        let _warning = warning_button("Test", None);
    }

    #[test]
    fn test_text_input_creation() {
        // Test text input widget creation
        let _input = text_input("placeholder", "value", |_| Message::RefreshBalance);
        let _labeled = labeled_text_input("Label", "placeholder", "value", |_| Message::RefreshBalance);
    }

    #[test]
    fn test_status_message_creation() {
        // Test status message widget creation
        let _error = status_message("Error message", true);
        let _success = status_message("Success message", false);
    }

    #[test]
    fn test_balance_display_creation() {
        // Test balance display widget creation
        let _balance = balance_display("ETH", "1.234", "$2,468.00");
    }

    #[test]
    fn test_theme_switcher_creation() {
        // Test theme switcher widget creation
        let _switcher = theme_switcher(true, Message::RefreshBalance);
    }
}
