//! Network Dialog Component
//!
//! This module contains the add/edit network dialog UI component.
//! Extracted from views/dialogs.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, Row, Space, Text, TextInput},
    Element, Length,
};

use crate::gui::{theme::styles, working_wallet::AppState, Message};

/// Add/Edit Network dialog view
pub fn add_network_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = if state.network().edit_mode {
        "Edit Network"
    } else {
        "Add Custom Network"
    };

    let content = Column::new()
        .push(
            // Header
            Row::new()
                .push(
                    Button::new(Text::new("← Back"))
                        .on_press(Message::HideAddNetwork)
                        .padding(8)
                        .style(styles::secondary_button()),
                )
                .push(Space::with_width(Length::Fixed(20.0)))
                .push(Text::new(title).size(20))
                .push(Space::with_width(Length::Fill))
                .align_items(iced::Alignment::Center),
        )
        .push(Space::with_height(Length::Fixed(30.0)))
        .push(
            Column::new()
                .push(Text::new("Network Name").size(14))
                .push(Space::with_height(Length::Fixed(5.0)))
                .push(
                    TextInput::new("e.g., My Custom Network", &state.network().network_name)
                        .on_input(Message::NetworkNameChanged)
                        .padding(10)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            Column::new()
                .push(Text::new("RPC URL").size(14))
                .push(Space::with_height(Length::Fixed(5.0)))
                .push(
                    TextInput::new("https://...", &state.network().network_rpc_url)
                        .on_input(Message::NetworkRpcUrlChanged)
                        .padding(10)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            Column::new()
                .push(Text::new("Chain ID").size(14))
                .push(Space::with_height(Length::Fixed(5.0)))
                .push(
                    TextInput::new("e.g., 1", &state.network().network_chain_id)
                        .on_input(Message::NetworkChainIdChanged)
                        .padding(10)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            Column::new()
                .push(Text::new("Currency Symbol").size(14))
                .push(Space::with_height(Length::Fixed(5.0)))
                .push(
                    TextInput::new("e.g., ETH", &state.network().network_symbol)
                        .on_input(Message::NetworkSymbolChanged)
                        .padding(10)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            Column::new()
                .push(Text::new("Block Explorer URL (Optional)").size(14))
                .push(Space::with_height(Length::Fixed(5.0)))
                .push(
                    TextInput::new("https://...", &state.network().network_block_explorer)
                        .on_input(Message::NetworkBlockExplorerChanged)
                        .padding(10)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .push(Space::with_height(Length::Fixed(30.0)))
        .push(
            Row::new()
                .push(
                    Button::new(Text::new("Cancel"))
                        .on_press(Message::HideAddNetwork)
                        .padding(10)
                        .width(Length::FillPortion(1))
                        .style(styles::secondary_button()),
                )
                .push(Space::with_width(Length::Fixed(15.0)))
                .push({
                    let can_submit = !state.network().network_name.is_empty()
                        && !state.network().network_rpc_url.is_empty()
                        && !state.network().network_chain_id.is_empty()
                        && !state.network().network_symbol.is_empty();

                    if can_submit {
                        Button::new(Text::new(if state.network().edit_mode {
                            "Update"
                        } else {
                            "Add Network"
                        }))
                        .on_press(Message::AddCustomNetwork)
                        .padding(10)
                        .width(Length::FillPortion(2))
                        .style(styles::primary_button())
                    } else {
                        Button::new(Text::new(if state.network().edit_mode {
                            "Update"
                        } else {
                            "Add Network"
                        }))
                        .padding(10)
                        .width(Length::FillPortion(2))
                        .style(styles::secondary_button())
                    }
                })
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
