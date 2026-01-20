//! Transaction-specific widgets for Iced 0.12 compatibility
//!
//! This module provides specialized widgets for transaction handling,
//! forms, and history display with professional styling.

use iced::{
    alignment::Horizontal,
    widget::{Button, Column, Container, PickList, Row, Rule, Scrollable, Space, Text, TextInput},
    Background, Border, Element, Length,
};

use crate::gui::{
    theme::{styles, text, VaughanColors},
    Message, TransactionDirection, TransactionFormState, TransactionRecord, TransactionStatus,
};

/// Create a professional transaction form with token support
pub fn transaction_form<'a>(
    form_state: &TransactionFormState,
    on_address_change: impl Fn(String) -> Message + 'a,
    on_amount_change: impl Fn(String) -> Message + 'a,
    on_gas_limit_change: impl Fn(String) -> Message + 'a,
    on_gas_price_change: impl Fn(String) -> Message + 'a,
    on_data_change: impl Fn(String) -> Message + 'a,
    on_max_amount: Message,
    on_clear: Message,
    on_send: Message,
    on_token_selected: impl Fn(String) -> Message + 'a,
    on_custom_token_address_change: impl Fn(String) -> Message + 'a,
    on_show_custom_token: Message,
    on_paste_from_clipboard: Message,
) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .push(Text::new("Send Transaction").size(18).style(text::primary()))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                // Token Selection field
                Column::new()
                    .push(
                        Row::new()
                            .push(Text::new("Token").size(12).style(text::secondary()))
                            .push(Space::with_width(Length::Fill))
                            .push(
                                Button::new(Text::new("+ Custom"))
                                    .on_press(on_show_custom_token)
                                    .style(styles::secondary_button())
                                    .padding([4, 8]),
                            ),
                    )
                    .push(Space::with_height(Length::Fixed(5.0)))
                    .push({
                        let mut token_options: Vec<String> = vec!["NATIVE".to_string()]; // Native token option
                        token_options.extend(form_state.available_tokens.iter().map(|token| token.symbol.clone()));

                        PickList::new(
                            token_options,
                            Some(form_state.selected_token_symbol.clone()),
                            on_token_selected,
                        )
                        .placeholder("Native Token")
                        .padding([12, 16])
                        .width(Length::Fill)
                    })
                    .push(if form_state.show_custom_token_input {
                        Column::new()
                            .push(Space::with_height(Length::Fixed(10.0)))
                            .push(
                                Text::new("Custom Token Contract Address")
                                    .size(11)
                                    .style(text::secondary()),
                            )
                            .push(Space::with_height(Length::Fixed(5.0)))
                            .push(
                                Row::new()
                                    .push(
                                        TextInput::new("0x...", &form_state.custom_token_address)
                                            .on_input(on_custom_token_address_change)
                                            .style(styles::primary_text_input())
                                            .padding(12)
                                            .size(14)
                                            .width(Length::Fill),
                                    )
                                    .push(Space::with_width(Length::Fixed(10.0)))
                                    .push(
                                        Button::new(Text::new("📋"))
                                            .on_press(on_paste_from_clipboard)
                                            .style(styles::secondary_button())
                                            .padding([8, 12]),
                                    ),
                            )
                            .into()
                    } else {
                        let empty_element: Element<Message> = Space::with_height(Length::Fixed(0.0)).into();
                        empty_element
                    })
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(15.0)))
            .push(
                // To Address field
                Column::new()
                    .push(Text::new("To Address").size(12).style(text::secondary()))
                    .push(Space::with_height(Length::Fixed(5.0)))
                    .push(
                        TextInput::new("0x...", &form_state.to_address)
                            .on_input(on_address_change)
                            .style(styles::primary_text_input())
                            .padding(12)
                            .size(14)
                            .width(Length::Fill),
                    )
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(15.0)))
            .push(
                // Amount field with Max button
                Column::new()
                    .push(
                        Row::new()
                            .push(Text::new("Amount").size(12).style(text::secondary()))
                            .push(Space::with_width(Length::Fill))
                            .push(
                                Button::new(Text::new("Max").size(10).style(text::primary()))
                                    .on_press(on_max_amount)
                                    .style(styles::secondary_button())
                                    .padding([4, 8]),
                            )
                            .align_items(iced::Alignment::Center),
                    )
                    .push(Space::with_height(Length::Fixed(5.0)))
                    .push(
                        TextInput::new("0.0", &form_state.amount)
                            .on_input(on_amount_change)
                            .style(styles::primary_text_input())
                            .padding(12)
                            .size(14)
                            .width(Length::Fill),
                    )
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(15.0)))
            .push(
                // Gas settings row
                Row::new()
                    .push(
                        Column::new()
                            .push(Text::new("Gas Limit").size(12).style(text::secondary()))
                            .push(Space::with_height(Length::Fixed(5.0)))
                            .push(
                                TextInput::new("21000", &form_state.gas_limit)
                                    .on_input(on_gas_limit_change)
                                    .style(styles::primary_text_input())
                                    .padding(12)
                                    .size(14)
                                    .width(Length::Fill),
                            )
                            .spacing(0)
                            .width(Length::FillPortion(1)),
                    )
                    .push(Space::with_width(Length::Fixed(15.0)))
                    .push(
                        Column::new()
                            .push(Text::new("Gas Price (Gwei)").size(12).style(text::secondary()))
                            .push(Space::with_height(Length::Fixed(5.0)))
                            .push(
                                TextInput::new("20", &form_state.gas_price)
                                    .on_input(on_gas_price_change)
                                    .style(styles::primary_text_input())
                                    .padding(12)
                                    .size(14)
                                    .width(Length::Fill),
                            )
                            .spacing(0)
                            .width(Length::FillPortion(1)),
                    )
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(15.0)))
            .push(
                // Data field (optional)
                Column::new()
                    .push(Text::new("Data (Optional)").size(12).style(text::secondary()))
                    .push(Space::with_height(Length::Fixed(5.0)))
                    .push(
                        TextInput::new("0x...", &form_state.data)
                            .on_input(on_data_change)
                            .style(styles::primary_text_input())
                            .padding(12)
                            .size(14)
                            .width(Length::Fill),
                    )
                    .spacing(0),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                // Validation errors
                {
                    let errors_element: Element<'a, Message> = if !form_state.validation_errors.is_empty() {
                        Column::new()
                            .push(
                                form_state
                                    .validation_errors
                                    .iter()
                                    .fold(Column::new(), |column, error| {
                                        column.push(Text::new(format!("• {error}")).size(12).style(text::error()))
                                    }),
                            )
                            .push(Space::with_height(Length::Fixed(15.0)))
                            .spacing(5)
                            .into()
                    } else {
                        Space::with_height(Length::Fixed(0.0)).into()
                    };
                    errors_element
                },
            )
            .push(
                // Action buttons
                Row::new()
                    .push(
                        Button::new(
                            Text::new("Clear")
                                .size(14)
                                .style(text::secondary())
                                .horizontal_alignment(Horizontal::Center),
                        )
                        .on_press(on_clear)
                        .style(styles::secondary_button())
                        .padding([12, 24])
                        .width(Length::FillPortion(1)),
                    )
                    .push(Space::with_width(Length::Fixed(15.0)))
                    .push(
                        Button::new(
                            Text::new("Send Transaction")
                                .size(14)
                                .style(text::primary())
                                .horizontal_alignment(Horizontal::Center),
                        )
                        .on_press(on_send)
                        .style(if form_state.is_valid {
                            styles::primary_button()
                        } else {
                            styles::disabled_button()
                        })
                        .padding([12, 24])
                        .width(Length::FillPortion(2)),
                    )
                    .spacing(0),
            )
            .spacing(0)
            .padding(20),
    )
    .style(styles::card_container())
    .width(Length::Fill)
    .into()
}

/// Create a transaction confirmation dialog
pub fn transaction_confirmation<'a>(
    to_address: &'a str,
    amount: &'a str,
    gas_estimate: &'a str,
    total_cost: &'a str,
    on_confirm: Message,
    on_cancel: Message,
) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .push(Text::new("Confirm Transaction").size(20).style(text::primary()))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Container::new(
                    Column::new()
                        .push(
                            Row::new()
                                .push(
                                    Text::new("To:")
                                        .size(12)
                                        .style(text::secondary())
                                        .width(Length::Fixed(80.0)),
                                )
                                .push(Text::new(to_address).size(12).style(text::primary()))
                                .spacing(10),
                        )
                        .push(Space::with_height(Length::Fixed(10.0)))
                        .push(
                            Row::new()
                                .push(
                                    Text::new("Amount:")
                                        .size(12)
                                        .style(text::secondary())
                                        .width(Length::Fixed(80.0)),
                                )
                                .push(Text::new(amount).size(12).style(text::primary()))
                                .spacing(10),
                        )
                        .push(Space::with_height(Length::Fixed(10.0)))
                        .push(
                            Row::new()
                                .push(
                                    Text::new("Gas:")
                                        .size(12)
                                        .style(text::secondary())
                                        .width(Length::Fixed(80.0)),
                                )
                                .push(Text::new(gas_estimate).size(12).style(text::primary()))
                                .spacing(10),
                        )
                        .push(Space::with_height(Length::Fixed(10.0)))
                        .push(Rule::horizontal(1))
                        .push(Space::with_height(Length::Fixed(10.0)))
                        .push(
                            Row::new()
                                .push(
                                    Text::new("Total:")
                                        .size(14)
                                        .style(text::primary())
                                        .width(Length::Fixed(80.0)),
                                )
                                .push(Text::new(total_cost).size(14).style(text::primary()))
                                .spacing(10),
                        )
                        .spacing(0),
                )
                .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
                    background: Some(Background::Color(VaughanColors::BACKGROUND_SECONDARY)),
                    border: Border::with_radius(6),
                    ..Default::default()
                })
                .padding(15)
                .width(Length::Fill),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Row::new()
                    .push(
                        Button::new(
                            Text::new("Cancel")
                                .size(14)
                                .style(text::secondary())
                                .horizontal_alignment(Horizontal::Center),
                        )
                        .on_press(on_cancel)
                        .style(styles::secondary_button())
                        .padding([12, 24])
                        .width(Length::FillPortion(1)),
                    )
                    .push(Space::with_width(Length::Fixed(15.0)))
                    .push(
                        Button::new(
                            Text::new("Confirm")
                                .size(14)
                                .style(text::primary())
                                .horizontal_alignment(Horizontal::Center),
                        )
                        .on_press(on_confirm)
                        .style(styles::primary_button())
                        .padding([12, 24])
                        .width(Length::FillPortion(1)),
                    )
                    .spacing(0),
            )
            .spacing(0)
            .padding(30),
    )
    .style(styles::card_container())
    .width(Length::Fixed(400.0))
    .into()
}

/// Create a transaction history item
pub fn transaction_history_item<'a>(transaction: &TransactionRecord) -> Element<'a, Message> {
    let (direction_icon, direction_color) = match transaction.direction {
        TransactionDirection::Sent => ("↗", VaughanColors::WARNING),
        TransactionDirection::Received => ("↙", VaughanColors::SUCCESS),
    };

    let status_color = match transaction.status {
        TransactionStatus::Confirmed => VaughanColors::SUCCESS,
        TransactionStatus::Pending => VaughanColors::WARNING,
        TransactionStatus::Failed => VaughanColors::ERROR,
    };

    Container::new(
        Row::new()
            .push(
                Text::new(direction_icon)
                    .size(16)
                    .style(iced::theme::Text::Color(direction_color)),
            )
            .push(Space::with_width(Length::Fixed(15.0)))
            .push(
                Column::new()
                    .push(
                        Text::new(format!(
                            "{} {} {}",
                            match transaction.direction {
                                TransactionDirection::Sent => "Sent",
                                TransactionDirection::Received => "Received",
                            },
                            transaction.amount,
                            crate::gui::get_network_currency(transaction.network)
                        ))
                        .size(14)
                        .style(text::primary()),
                    )
                    .push(
                        Text::new(format!(
                            "{} • {}",
                            format!(
                                "{}...{}",
                                &transaction.address[..6],
                                &transaction.address[transaction.address.len() - 4..]
                            ),
                            format_timestamp(transaction.timestamp)
                        ))
                        .size(11)
                        .style(text::muted()),
                    )
                    .spacing(2)
                    .width(Length::Fill),
            )
            .push(Space::with_width(Length::Fixed(15.0)))
            .push(
                Text::new(format!("{:?}", transaction.status))
                    .size(11)
                    .style(iced::theme::Text::Color(status_color)),
            )
            .align_items(iced::Alignment::Center)
            .padding([12, 15])
            .spacing(0),
    )
    .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
        background: Some(Background::Color(VaughanColors::BACKGROUND_SECONDARY)),
        border: Border::with_radius(6),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

/// Create a transaction history list
pub fn transaction_history<'a>(transactions: &[TransactionRecord]) -> Element<'a, Message> {
    if transactions.is_empty() {
        Container::new(
            Column::new()
                .push(Text::new("No transactions yet").size(14).style(text::muted()))
                .push(Space::with_height(Length::Fixed(5.0)))
                .push(
                    Text::new("Your transaction history will appear here")
                        .size(12)
                        .style(text::muted()),
                )
                .spacing(0)
                .align_items(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .center_x()
        .padding(40)
        .into()
    } else {
        Scrollable::new(transactions.iter().fold(Column::new().spacing(8), |column, tx| {
            column.push(transaction_history_item(tx))
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// Format timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    // Simple timestamp formatting - in production, use proper date formatting
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Validate numeric input for amounts
pub fn validate_amount(amount: &str) -> bool {
    if amount.is_empty() {
        return false;
    }

    amount.parse::<f64>().is_ok() && amount.parse::<f64>().unwrap_or(0.0) > 0.0
}

/// Validate gas limit input
pub fn validate_gas_limit(gas_limit: &str) -> bool {
    if gas_limit.is_empty() {
        return false;
    }

    if let Ok(limit) = gas_limit.parse::<u64>() {
        (21000..=10_000_000).contains(&limit)
    } else {
        false
    }
}

/// Validate gas price input (in Gwei)
pub fn validate_gas_price(gas_price: &str) -> bool {
    if gas_price.is_empty() {
        return false;
    }

    if let Ok(price) = gas_price.parse::<f64>() {
        price > 0.0 && price <= 1000.0 // Reasonable gas price range
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::NetworkId;

    #[test]
    fn test_format_timestamp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert_eq!(format_timestamp(now), "Just now");
        assert_eq!(format_timestamp(now - 120), "2m ago");
        assert_eq!(format_timestamp(now - 7200), "2h ago");
        assert_eq!(format_timestamp(now - 172800), "2d ago");
    }

    #[test]
    fn test_transaction_confirmation_creation() {
        // Test transaction confirmation dialog creation
        let _confirmation = transaction_confirmation(
            "0x1234567890123456789012345678901234567890",
            "1.5 ETH",
            "0.002 ETH",
            "1.502 ETH",
            Message::RefreshBalance,
            Message::RefreshBalance,
        );
    }

    #[test]
    fn test_transaction_history_empty() {
        // Test empty transaction history
        let transactions = vec![];
        let _history = transaction_history(&transactions);
    }

    #[test]
    fn test_transaction_history_with_data() {
        // Test transaction history with sample data
        let transactions = vec![TransactionRecord {
            hash: "0x123".to_string(),
            address: "0x1234567890123456789012345678901234567890".to_string(),
            amount: "1.5".to_string(),
            direction: TransactionDirection::Sent,
            status: TransactionStatus::Confirmed,
            timestamp: 1234567890,
            network: NetworkId(1),
        }];
        let _history = transaction_history(&transactions);
    }

    #[test]
    fn test_validate_amount() {
        // Test amount validation
        assert!(validate_amount("1.5"));
        assert!(validate_amount("0.001"));
        assert!(!validate_amount("0"));
        assert!(!validate_amount("-1.5"));
        assert!(!validate_amount(""));
        assert!(!validate_amount("abc"));
    }

    #[test]
    fn test_validate_gas_limit() {
        // Test gas limit validation
        assert!(validate_gas_limit("21000"));
        assert!(validate_gas_limit("100000"));
        assert!(!validate_gas_limit("20000")); // Too low
        assert!(!validate_gas_limit("20000000")); // Too high
        assert!(!validate_gas_limit(""));
        assert!(!validate_gas_limit("abc"));
    }

    #[test]
    fn test_validate_gas_price() {
        // Test gas price validation
        assert!(validate_gas_price("20"));
        assert!(validate_gas_price("1.5"));
        assert!(!validate_gas_price("0"));
        assert!(!validate_gas_price("2000")); // Too high
        assert!(!validate_gas_price(""));
        assert!(!validate_gas_price("abc"));
    }
}
