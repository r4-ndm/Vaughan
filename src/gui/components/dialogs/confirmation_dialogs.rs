//! Confirmation Dialog Components
//!
//! This module contains small confirmation dialog components.

use iced::{
    widget::{Button, Column, Container, Row, Space, Text},
    Alignment, Color, Element, Length,
};

use crate::gui::working_wallet::AppState;
use crate::gui::{theme::styles, Message};
use crate::network::NetworkId;

/// The modal background style for confirmation dialogs
pub struct ModalBackgroundStyle;

impl iced::widget::container::StyleSheet for ModalBackgroundStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            ..Default::default()
        }
    }
}

/// Create the clear logs confirmation dialog
pub fn clear_logs_confirmation_dialog_view(state: &AppState) -> Element<'_, Message> {
    Container::new(
        Container::new(
            Column::new()
                .push(Text::new("Clear All Logs").size(20).style(Color::WHITE))
                .push(Space::with_height(Length::Fixed(20.0)))
                .push(
                    Text::new(format!(
                        "Are you sure you want to clear all {} log entries?",
                        state.log_entries.len()
                    ))
                    .size(14)
                    .style(Color::from_rgb(0.8, 0.8, 0.8)),
                )
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(
                    Text::new("This action cannot be undone.")
                        .size(12)
                        .style(Color::from_rgb(0.9, 0.6, 0.6)),
                )
                .push(Space::with_height(Length::Fixed(30.0)))
                .push(
                    Row::new()
                        .spacing(15)
                        .push(
                            Button::new(Text::new("Cancel"))
                                .on_press(Message::HideClearLogsConfirmation)
                                .padding([10, 20])
                                .style(styles::secondary_button()),
                        )
                        .push(
                            Button::new(Text::new("Clear All Logs"))
                                .on_press(Message::ConfirmClearLogs)
                                .padding([10, 20])
                                .style(styles::danger_button()),
                        ),
                )
                .align_items(iced::Alignment::Center)
                .spacing(5),
        )
        .padding(30)
        .style(styles::dark_flat_container())
        .width(Length::Fixed(400.0)),
    )
    .padding(50)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .style(iced::theme::Container::Custom(Box::new(ModalBackgroundStyle)))
    .into()
}

/// Reset wallet confirmation dialog view
pub fn reset_wallet_confirmation_dialog_view(_state: &AppState) -> Element<'_, Message> {
    Container::new(
        Container::new(
            Column::new()
                .push(Text::new("Reset Wallet").size(20).style(Color::WHITE))
                .push(Space::with_height(Length::Fixed(20.0)))
                .push(
                    Text::new("Are you sure you want to reset your wallet?")
                        .size(14)
                        .style(Color::from_rgb(0.8, 0.8, 0.8)),
                )
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(
                    Text::new("This will permanently delete:")
                        .size(13)
                        .style(Color::from_rgb(0.9, 0.7, 0.7)),
                )
                .push(Space::with_height(Length::Fixed(10.0)))
                .push(
                    Column::new()
                        .spacing(5)
                        .push(
                            Text::new("• All wallet accounts and keys")
                                .size(12)
                                .style(Color::from_rgb(0.9, 0.6, 0.6)),
                        )
                        .push(
                            Text::new("• Wallet configuration")
                                .size(12)
                                .style(Color::from_rgb(0.9, 0.6, 0.6)),
                        )
                        .push(
                            Text::new("• All transaction history")
                                .size(12)
                                .style(Color::from_rgb(0.9, 0.6, 0.6)),
                        ),
                )
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(
                    Text::new("This action cannot be undone!")
                        .size(12)
                        .style(Color::from_rgb(1.0, 0.4, 0.4)),
                )
                .push(Space::with_height(Length::Fixed(30.0)))
                .push(
                    Row::new()
                        .spacing(15)
                        .push(
                            Button::new(Text::new("Cancel"))
                                .on_press(Message::HideResetWalletConfirmation)
                                .padding([10, 20])
                                .style(styles::secondary_button()),
                        )
                        .push(
                            Button::new(Text::new("Reset Wallet"))
                                .on_press(Message::ConfirmResetWallet)
                                .padding([10, 20])
                                .style(styles::danger_button()),
                        ),
                )
                .align_items(iced::Alignment::Center)
                .spacing(5),
        )
        .padding(30)
        .style(styles::dark_flat_container())
        .width(Length::Fixed(450.0)),
    )
    .padding(50)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .style(iced::theme::Container::Custom(Box::new(ModalBackgroundStyle)))
    .into()
}

/// Create the hardware wallet dialog (stub implementation)
pub fn hardware_wallet_dialog_view(state: &AppState) -> Element<'_, Message> {
    let content = Column::new()
        .spacing(20)
        .align_items(Alignment::Center)
        .push(
            Text::new("🔐 Hardware Wallet Integration")
                .size(24)
                .style(Color::WHITE)
        )
        .push(
            Text::new("Connect and manage your Ledger or Trezor hardware wallet for maximum security")
                .size(14)
                .style(Color::from_rgb(0.8, 0.8, 0.8))
        )
        .push(Space::with_height(Length::Fixed(10.0)))

        // Device scan section
        .push(
            Column::new()
                .spacing(15)
                .push(
                    Text::new("📡 Device Detection")
                        .size(16)
                        .style(Color::from_rgb(0.9, 0.9, 0.9))
                )
                .push(
                    Text::new("Make sure your hardware wallet is connected and unlocked")
                        .size(12)
                        .style(Color::from_rgb(0.7, 0.7, 0.7))
                )
                .push(
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(Text::new("🔍 Scan for Devices").size(12))
                                .on_press(Message::ScanHardwareWallets)
                                .padding([8, 16])
                                .style(styles::secondary_button())
                        )
                        .push(
                            Button::new(Text::new("🔄 Refresh").size(12))
                                .on_press(Message::RefreshHardwareWallets)
                                .padding([8, 16])
                                .style(styles::secondary_button())
                        )
                )
        )

        // Status display
        .push(
            Container::new(
                Column::new()
                    .spacing(10)
                    .push(
                        Text::new("📱 Connected Devices")
                            .size(14)
                            .style(Color::from_rgb(0.9, 0.9, 0.9))
                    )
                    .push(
                        if state.wallet().available_hardware_wallets.is_empty() {
                            Text::new("No hardware wallets detected")
                                .size(12)
                                .style(Color::from_rgb(0.6, 0.6, 0.6))
                        } else {
                            Text::new(format!("✅ {} device(s) connected", state.wallet().available_hardware_wallets.len()))
                                .size(12)
                                .style(Color::from_rgb(0.2, 0.8, 0.2))
                        }
                    )
                    .push(
                        // Show device list
                        state.wallet().available_hardware_wallets.iter().fold(
                            Column::new().spacing(5),
                            |column, device| {
                                column.push(
                                    Container::new(
                                        Row::new()
                                            .spacing(10)
                                            .push(
                                                Text::new(device.device_type.as_str())
                                                    .size(12)
                                                    .style(Color::from_rgb(0.8, 0.8, 0.8))
                                            )
                                            .push(Space::with_width(Length::Fill))
                                            .push(
                                                Button::new(Text::new("Connect").size(10))
                                                    .on_press(Message::ConnectToHardwareWallet(format!("{device}")))
                                                    .padding([4, 8])
                                                    .style(styles::secondary_button())
                                            )
                                    )
                                    .padding(10)
                                    .style(styles::dark_flat_container())
                                    .width(Length::Fill)
                                )
                            }
                        )
                    )
            )
            .padding(15)
            .style(styles::card_container())
            .width(Length::Fill)
        )

        // Instructions
        .push(
            Column::new()
                .spacing(5)
                .push(
                    Text::new("💡 Instructions:")
                        .size(12)
                        .style(Color::from_rgb(0.8, 0.8, 0.8))
                )
                .push(
                    Text::new("1. Connect your Ledger or Trezor via USB")
                        .size(10)
                        .style(Color::from_rgb(0.6, 0.6, 0.6))
                )
                .push(
                    Text::new("2. Unlock the device and open the Ethereum app (Ledger)")
                        .size(10)
                        .style(Color::from_rgb(0.6, 0.6, 0.6))
                )
                .push(
                    Text::new("3. Click 'Scan for Devices' to detect your hardware wallet")
                        .size(10)
                        .style(Color::from_rgb(0.6, 0.6, 0.6))
                )
        )

        // Action buttons
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            Row::new()
                .spacing(15)
                .push(
                    Button::new(Text::new("← Back").size(12))
                        .on_press(Message::HideHardWallet)
                        .padding([8, 16])
                        .style(styles::secondary_button())
                )
                .push(
                    Button::new(Text::new("🔧 Test Connection").size(12))
                        .on_press(Message::TestHardwareWallet)
                        .padding([8, 16])
                        .style(styles::secondary_button())
                )
        );

    let dialog_container = Container::new(content)
        .padding(30)
        .width(Length::Fixed(500.0))
        .style(styles::card_container());

    // Modal overlay with centered dialog
    Container::new(dialog_container)
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(ModalBackgroundStyle)))
        .into()
}

/// Create the delete account confirmation dialog
pub fn delete_account_dialog_view(state: &AppState) -> Element<'_, Message> {
    let current_account_name = state
        .wallet()
        .current_account_id
        .as_ref()
        .and_then(|id| {
            state
                .wallet()
                .available_accounts
                .iter()
                .find(|acc| &acc.id == id)
                .map(|acc| acc.name.clone())
        })
        .unwrap_or_else(|| "Unknown Account".to_string());

    let content = Column::new()
        .spacing(20)
        .align_items(Alignment::Center)
        .push(
            Text::new("⚠️ Delete Account")
                .size(24)
                .style(Color::from([0.9, 0.3, 0.3])),
        )
        .push(
            Column::new()
                .spacing(10)
                .push(Text::new("Are you sure you want to delete the account:".to_string()))
                .push(
                    Text::new(current_account_name)
                        .size(18)
                        .style(Color::from([0.2, 0.2, 0.8])),
                )
                .push(Text::new(""))
                .push(Text::new("⚠️ This action cannot be undone!").style(Color::from([0.9, 0.3, 0.3])))
                .push(Text::new("• The account and its private key will be permanently deleted").size(14))
                .push(Text::new("• Make sure you have backed up your seed phrase or private key").size(14))
                .push(Text::new("• Any funds in this account will become inaccessible").size(14)),
        )
        .push(if state.wallet().deleting_account {
            Row::new()
                .spacing(10)
                .align_items(Alignment::Center)
                .push(Text::new("🔄 Deleting account..."))
                .push(Text::new("Please wait...").size(14))
        } else {
            Row::new()
                .spacing(20)
                .push(
                    Button::new(Text::new("Cancel"))
                        .on_press(Message::HideDeleteAccount)
                        .padding([12, 24])
                        .style(styles::secondary_button()),
                )
                .push(
                    Button::new(Text::new("🗑️ Delete Permanently"))
                        .on_press(Message::ConfirmDeleteAccount)
                        .padding([12, 24])
                        .style(styles::danger_button()),
                )
        });

    let dialog_container = Container::new(content)
        .padding(30)
        .width(Length::Fixed(500.0))
        .style(styles::card_container());

    // Modal overlay with centered dialog
    Container::new(dialog_container)
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(ModalBackgroundStyle)))
        .into()
}

/// Create the delete network confirmation dialog
pub fn delete_network_confirmation_dialog_view(state: &AppState) -> Element<'_, Message> {
    // Determine selected network name and id for message
    let (_name, id_display) = if let Some(network_id_str) = &state.network().selected_network_for_edit {
        if let Ok(chain_id) = network_id_str.parse::<u64>() {
            let nid = NetworkId(chain_id);
            if let Some(net) = state.network().available_networks.iter().find(|n| n.id == nid) {
                (net.name.clone(), format!("{} (Chain ID: {})", net.name, chain_id))
            } else {
                ("Selected Network".to_string(), format!("Chain ID: {chain_id}"))
            }
        } else {
            ("Selected Network".to_string(), network_id_str.clone())
        }
    } else {
        ("Selected Network".to_string(), "Unknown".to_string())
    };

    Container::new(
        Container::new(
            Column::new()
                .push(Text::new("Delete Custom Network").size(20).style(Color::WHITE))
                .push(Space::with_height(Length::Fixed(20.0)))
                .push(
                    Text::new(format!("Are you sure you want to delete {id_display}?"))
                        .size(14)
                        .style(Color::from_rgb(0.8, 0.8, 0.8)),
                )
                .push(Space::with_height(Length::Fixed(15.0)))
                .push(
                    Text::new("This action will remove the network from your list. You can re-add it later.")
                        .size(12)
                        .style(Color::from_rgb(0.9, 0.6, 0.6)),
                )
                .push(Space::with_height(Length::Fixed(30.0)))
                .push(
                    Row::new()
                        .spacing(15)
                        .push(
                            Button::new(Text::new("Cancel"))
                                .on_press(Message::HideDeleteNetworkConfirm)
                                .padding([10, 20])
                                .style(styles::secondary_button()),
                        )
                        .push(
                            Button::new(Text::new("Delete"))
                                .on_press(Message::ConfirmDeleteNetwork)
                                .padding([10, 20])
                                .style(styles::danger_button()),
                        ),
                )
                .align_items(Alignment::Center)
                .spacing(5),
        )
        .padding(30)
        .style(styles::dark_flat_container())
        .width(Length::Fixed(420.0)),
    )
    .padding(50)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .style(iced::theme::Container::Custom(Box::new(ModalBackgroundStyle)))
    .into()
}
/// Create the Dapps coming soon dialog
pub fn dapps_coming_soon_dialog_view(_state: &AppState) -> Element<'_, Message> {
    let content = Column::new()
        .spacing(20)
        .align_items(Alignment::Center)
        .push(Text::new("🚀 Dapps").size(24).style(Color::WHITE))
        .push(
            Text::new("Decentralized Applications Support")
                .size(16)
                .style(Color::from_rgb(0.9, 0.9, 0.9)),
        )
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(
            Text::new("🔮 Coming Soon!")
                .size(20)
                .style(Color::from_rgb(0.3, 0.8, 1.0)),
        )
        .push(
            Text::new("We're working on integrating popular DeFi protocols and dApps directly into Vaughan.")
                .size(14)
                .style(Color::from_rgb(0.8, 0.8, 0.8)),
        )
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(
            Column::new()
                .spacing(8)
                .push(
                    Text::new("📋 Planned Features:")
                        .size(14)
                        .style(Color::from_rgb(0.9, 0.9, 0.9)),
                )
                .push(
                    Text::new("• Uniswap & DEX Integration")
                        .size(12)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                )
                .push(
                    Text::new("• DeFi Lending & Borrowing")
                        .size(12)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                )
                .push(
                    Text::new("• NFT Marketplace Access")
                        .size(12)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                )
                .push(
                    Text::new("• Cross-Chain Bridge Support")
                        .size(12)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                ),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            Button::new(Text::new("← Back").size(12))
                .on_press(Message::HideDappsComingSoon)
                .padding([8, 16])
                .style(styles::secondary_button()),
        );

    let dialog_container = Container::new(content)
        .padding(30)
        .width(Length::Fixed(500.0))
        .style(styles::card_container());

    // Modal overlay with centered dialog
    Container::new(dialog_container)
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(ModalBackgroundStyle)))
        .into()
}
