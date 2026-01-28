//! Main Wallet View Components
//!
//! This module contains the main wallet interface view components extracted
//! from working_wallet.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, Image, PickList, Row, Space, Text, TextInput},
    Element, Length,
};

use crate::gui::{
    safe_calculations::*, theme::styles, utils::create_full_address_display, working_wallet::AppState, GasSpeed,
    Message, StatusMessageColor,
    services::AssetServiceTrait,
};

impl AppState {
    /// Main wallet interface view
    pub fn main_wallet_view(&self) -> Element<'_, Message> {
        let mut content = Column::new()
            .push(
                // Header with logo and settings
                Row::new()
                    .push({
                        // Logo display - using AssetService
                        let logo_element: Element<Message> =
                            if let Some(logo_path) = self.services().asset().get_logo_path() {
                                Image::new(iced::widget::image::Handle::from_path(logo_path))
                                    .width(Length::Fixed(safe_dimension(513.0)))  // Original size
                                    .height(Length::Fixed(safe_dimension(76.0)))  // Original size
                                    .into()
                            } else {
                                Text::new("VAUGHAN")
                                    .size(24)
                                    .style(iced::Color::from_rgb(0.0, 0.5, 1.0))
                                    .into()
                            };
                        logo_element
                    })
                    .push(Space::with_width(Length::Fill))
                    .align_items(iced::Alignment::Center),
            )
            .push(Space::with_height(Length::Fixed(safe_dimension(20.0))))
            .push(
                // Address display centered above network and account selectors
                {
                    let address_element: Element<Message> = if let Some(current_account_id) = self.current_account_id()
                    {
                        if let Some(account) = self.available_accounts().iter().find(|a| &a.id == current_account_id) {
                            // Use AccountDisplayService for consistent address formatting
                            let address_str = format!("{:?}", account.address);
                            Container::new(create_full_address_display(
                                address_str,
                                self.address_just_copied(),
                            ))
                            .width(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .into()
                        } else {
                            Space::with_height(Length::Fixed(safe_dimension(0.0))).into()
                        }
                    } else {
                        Space::with_height(Length::Fixed(safe_dimension(0.0))).into()
                    };
                    address_element
                },
            )
            .push(Space::with_height(Length::Fixed(safe_dimension(15.0))))
            .push(
                // Network selection only (account management moved to send form)
                Row::new()
                    .push(
                        PickList::new(
                            &self.available_networks()[..],
                            self.available_networks()
                                .iter()
                                .find(|n| &n.id == self.current_network())
                                .cloned(),
                            |config| Message::NetworkSelected(config.id),
                        )
                        .padding([8, 2, 8, 8])
                        .style(styles::dark_grey_pick_list())
                        .width(Length::Fill),
                    )
                    .push(Space::with_width(Length::Fixed(safe_dimension(8.0))))
                    .push(
                        Button::new(
                            Image::new(
                                self.services()
                                    .asset()
                                    .get_icon_path("hamburger")
                                    .unwrap_or_else(|| "assets/hamburger-128.png".into())
                            )
                            .width(Length::Fixed(safe_dimension(28.0)))
                            .height(Length::Fixed(safe_dimension(28.0))),
                        )
                        .on_press(Message::ShowAddNetwork)
                        .padding(0)
                        .style(styles::transparent_button()),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .push(Space::with_height(Length::Fixed(safe_dimension(15.0))))
            .push(self.send_form_view())
            .push(Space::with_height(Length::Fixed(safe_dimension(15.0))))
            .push(self.action_buttons_view())
            .push(Space::with_height(Length::Fixed(safe_dimension(10.0))))
            .push(self.wallet_management_buttons_view());

        // Add price information panel if visible
        if self.show_price_info() {
            content = content
                .push(Space::with_height(Length::Fixed(safe_dimension(15.0))))
                .push(self.price_info_panel());
        }

        // Add status message if present (only for Success or Error - no grey boxes)
        if !self.status_message().is_empty() {
            match *self.status_message_color() {
                StatusMessageColor::Success | StatusMessageColor::Error | StatusMessageColor::Warning => {
                    content = content
                        .push(Space::with_height(Length::Fixed(safe_dimension(10.0))))
                        .push(self.status_message_view());
                }
                _ => {} // Don't display Default or Info messages as grey boxes
            }
        }

        // Add retro green loading bar for pending transactions
        if self.transaction().pending_transactions.iter().any(|tx| tx.cancellable) {
            content = content.push(self.pending_transactions_view());
        }

        Container::new(content).padding(safe_dimension(20.0)).into()
    }

    /// Send form view component
    fn send_form_view(&self) -> Element<'_, Message> {
        Container::new(
            Column::new()
                .push(self.account_balance_row())
                .push(Space::with_height(Length::Fixed(safe_dimension(12.0))))
                .push(self.address_input_row())
                .push(Space::with_height(Length::Fixed(safe_dimension(30.0))))
                .push(self.token_amount_row())
                .push(Space::with_height(Length::Fixed(safe_dimension(10.0))))
                .push(self.gas_settings_row())
                .push(Space::with_height(Length::Fixed(safe_dimension(10.0))))
                .push(self.tx_type_nonce_row())
                .push(self.max_priority_fee_section())
                .push(Space::with_height(Length::Fixed(safe_dimension(10.0))))
                .push(self.gas_speed_buttons())
                .push(Space::with_height(Length::Fixed(safe_dimension(10.0))))
                .push(self.send_button())
                .spacing(5),
        )
        .padding(safe_dimension(20.0))
        .style(styles::dark_flat_container())
        .width(Length::Fill)
        .into()
    }

    /// Account and balance row - unified layout
    fn account_balance_row(&self) -> Element<'_, Message> {
        // Get the balance for the selected ticker using safe calculations
        // Extract just the symbol from the selected ticker (handle both "SYMBOL" and "SYMBOL (ADDRESS)" formats)
        let selected_symbol = if self.balance_selected_ticker().contains('(') {
            self.balance_selected_ticker()
                .split('(')
                .next()
                .unwrap_or(self.balance_selected_ticker())
                .trim()
                .to_string()
        } else {
            self.balance_selected_ticker().clone()
        };

        let current_balance = self
            .token_balances()
            .iter()
            .find(|token| token.symbol == selected_symbol)
            .map(|token| safe_balance(&token.balance))
            .unwrap_or_else(|| "0.0000".to_string());

        Column::new()
            .push(Space::with_height(Length::Fixed(safe_dimension(5.0))))
            .push(
                Row::new()
                    // Left side: Account selector and delete button
                    .push(self.account_selector())
                    .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
                    .push(
                        Button::new(
                            Image::new(
                                self.services()
                                    .asset()
                                    .get_icon_path("hamburger")
                                    .unwrap_or_else(|| "assets/hamburger-128.png".into())
                            )
                            .width(Length::Fixed(safe_dimension(26.0)))
                            .height(Length::Fixed(safe_dimension(26.0)))
                        )
                        .on_press_maybe(if self.current_account_id().is_some() {
                            Some(Message::ShowDeleteAccount)
                        } else {
                            None
                        })
                        .padding(0)
                        .style(styles::transparent_button()),
                    )

                    // Flexible space to push balance to the right
                    .push(Space::with_width(Length::Fill))

                    // Right side: Balance value (right-aligned for left expansion)
                    .push(
                        Container::new(
                            if self.balance_spinner() {
                                Row::new()
                                    .spacing(6)
                                    .align_items(iced::Alignment::Center)
                                    .push(Text::new("⟲").size(16).style(iced::Color::from_rgb(0.2, 0.8, 0.2)))
                                    .push(Text::new(current_balance.clone()).size(18))
                            } else {
                                Row::new()
                                    .push(Text::new(current_balance.clone()).size(18))
                            }
                        )
                        .padding([6, 10])
                        .style(styles::dark_flat_container())
                    )
                    .push(Space::with_width(Length::Fixed(safe_dimension(8.0))))

                    // Token selector and add button
                    .push(
                        PickList::new(
                            &self.balance_available_tickers()[..],
                            Some(self.balance_selected_ticker().clone()),
                            Message::BalanceTickerSelected,
                        )
                        .width(Length::Fixed(safe_dimension(80.0)))
                        .padding([4, 8])
                        .style(styles::dark_grey_pick_list()),
                    )
                    .push(Space::with_width(Length::Fixed(safe_dimension(8.0))))
                    .push(
                        Button::new(
                            Image::new(
                                self.services()
                                    .asset()
                                    .get_icon_path("hamburger")
                                    .unwrap_or_else(|| "assets/hamburger-128.png".into())
                            )
                            .width(Length::Fixed(safe_dimension(26.0)))
                            .height(Length::Fixed(safe_dimension(26.0)))
                        )
                        .on_press(Message::ShowBalanceAddToken)
                        .padding(0)
                        .style(styles::transparent_button()),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .into()
    }

    /// Account selector with safe calculations
    fn account_selector(&self) -> Element<'_, Message> {
        if self.available_accounts().is_empty() {
            if self.loading_accounts() {
                Container::new(
                    Text::new("⟲ Loading accounts...")
                        .size(12)
                        .style(iced::Color::from_rgb(0.6, 0.4, 1.0)),
                )
                .width(Length::Fill)
                .padding(safe_dimension(10.0))
                .into()
            } else {
                // Show text placeholder when no accounts (avoids PickList crash)
                Container::new(
                    Text::new("No accounts")
                        .size(14)
                        .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                )
                .width(Length::Fixed(safe_dimension(200.0)))
                .height(Length::Fixed(safe_dimension(30.0)))
                .padding([6, 12])
                .style(iced::theme::Container::Box)
                .into()
            }
        } else {
            // Find the currently selected account
            let selected_account = if let Some(current_id) = self.current_account_id() {
                self.available_accounts()
                    .iter()
                    .find(|a| &a.id == current_id)
                    .cloned()
            } else {
                None
            };

            PickList::new(
                &self.available_accounts()[..],
                selected_account,
                |account| Message::AccountSelected(account.id),
            )
            .width(Length::Fixed(safe_dimension(200.0)))
            .style(styles::dark_grey_pick_list())
            .into()
        }
    }

    /// Address input row
    fn address_input_row(&self) -> Element<'_, Message> {
        let send_to_address = &self.send_to_address();

        Row::new()
            .push(
                Text::new("To Address :")
                    .size(13)
                    .width(Length::Fixed(safe_dimension(85.0)))
                    .vertical_alignment(iced::alignment::Vertical::Center),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(8.0))))
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Button::new(
                                    Image::new(
                                        self.services()
                                            .asset()
                                            .get_icon_path("clipboard")
                                            .unwrap_or_else(|| "assets/clipboard-128.png".into())
                                    )
                                    .width(Length::Fixed(safe_dimension(24.0)))
                                    .height(Length::Fixed(safe_dimension(24.0))),
                                )
                                .on_press(Message::SendPasteAddressFromClipboard)
                                .padding(0)
                                .style(iced::theme::Button::Text),
                            )
                            .align_items(iced::Alignment::Center),
                    )
                    .spacing(4),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(4.0))))
            .push(
                TextInput::new("Recipient address (0x...)", send_to_address)
                    .on_input(Message::SendToAddressChanged)
                    .padding(10)
                    .width(Length::Fill)
                    .style(styles::black_grey_text_input()),
            )
            .align_items(iced::Alignment::Center)
            .into()
    }

    /// Token and amount selection row
    fn token_amount_row(&self) -> Element<'_, Message> {
        Row::new()
            .push(
                Container::new(
                    Text::new("Send :")
                        .size(13)
                        .vertical_alignment(iced::alignment::Vertical::Center)
                )
                .width(Length::Fixed(safe_dimension(85.0)))
                .align_y(iced::alignment::Vertical::Center)
            )

            .push(Space::with_width(Length::Fixed(safe_dimension(12.0))))
            .push(Space::with_width(Length::FillPortion(1))) // Spacer to push input to right
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Button::new(
                                    Image::new(
                                        self.services()
                                            .asset()
                                            .get_icon_path("clipboard")
                                            .unwrap_or_else(|| "assets/clipboard-128.png".into())
                                    )
                                    .width(Length::Fixed(safe_dimension(24.0)))
                                    .height(Length::Fixed(safe_dimension(24.0)))
                                )
                                .on_press(Message::SendPasteAmountFromClipboard)
                                .padding(0)
                                .style(iced::theme::Button::Text),
                            )
                            .align_items(iced::Alignment::Center),
                    )
                    .spacing(4)
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(4.0))))
            .push(
                TextInput::new("0.0", self.send_amount())
                    .on_input(Message::SendAmountChanged)
                    .padding(10)
                    .width(Length::FillPortion(4))
                    .style(styles::black_grey_text_input()),
            )
            .align_items(iced::Alignment::Center)
            .into()
    }

    /// Gas settings row
    fn gas_settings_row(&self) -> Element<'_, Message> {
        Row::new()
            .push(
                Column::new()
                    .push(Text::new("Gas Limit").size(12))
                    .push(Space::with_height(Length::Fixed(safe_dimension(5.0))))
                    .push(
                        TextInput::new("21000", self.send_gas_limit())
                            .on_input(Message::SendGasLimitChanged)
                            .padding([8, 8])
                            .width(Length::Fill)
                            .style(styles::black_grey_text_input()),
                    )
                    .spacing(3)
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Column::new()
                    .push(
                        Text::new(if *self.send_tx_type() == "EIP-1559" {
                            "Max Fee (Gwei)"
                        } else {
                            "Gas Price (Gwei)"
                        })
                        .size(12),
                    )
                    .push(Space::with_height(Length::Fixed(safe_dimension(5.0))))
                    .push(if *self.send_tx_type() == "EIP-1559" {
                        TextInput::new("30", self.send_max_fee_gwei())
                            .on_input(Message::SendMaxFeeChanged)
                            .padding([8, 8])
                            .width(Length::Fill)
                            .style(styles::black_grey_text_input())
                    } else {
                        TextInput::new("20", self.send_gas_price())
                            .on_input(Message::SendGasPriceChanged)
                            .padding([8, 8])
                            .width(Length::Fill)
                            .style(styles::black_grey_text_input())
                    })
                    .spacing(3)
                    .width(Length::FillPortion(1)),
            )
            .spacing(8)
            .into()
    }

    /// Transaction type, Max Priority Fee, and Nonce row
    fn tx_type_nonce_row(&self) -> Element<'_, Message> {
        let tx_type_col = Column::new()
            .push(Text::new("Tx Type").size(12))
            .push(Space::with_height(Length::Fixed(safe_dimension(5.0))))
            .push({
                let options = vec!["Auto".to_string(), "Legacy".to_string(), "EIP-1559".to_string()];
                PickList::new(options, Some(self.send_tx_type().clone()), Message::SendTxTypeChanged)
                    .padding([8, 8])
                    .width(Length::Fill)
                    .style(styles::dark_grey_pick_list())
            })
            .spacing(3);

        let nonce_col = Column::new()
            .push(Text::new("Nonce (optional)").size(12))
            .push(Space::with_height(Length::Fixed(safe_dimension(5.0))))
            .push(
                TextInput::new("Auto", self.send_nonce_override())
                    .on_input(Message::SendNonceOverrideChanged)
                    .padding([8, 8])
                    .width(Length::Fill)
                    .style(styles::black_grey_text_input()),
            )
            .spacing(3);

        if *self.send_tx_type() == "EIP-1559" {
            let left_group = Row::new()
                .push(tx_type_col.width(Length::FillPortion(3))) // 60% of left half
                .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
                .push(
                    Column::new()
                        .push(Text::new("Max Priority").size(12))
                        .push(Space::with_height(Length::Fixed(safe_dimension(5.0))))
                        .push(
                            TextInput::new("2", self.send_max_priority_fee_gwei())
                                .on_input(Message::SendMaxPriorityFeeChanged)
                                .padding([8, 8])
                                .width(Length::Fill)
                                .style(styles::black_grey_text_input()),
                        )
                        .spacing(3)
                        .width(Length::FillPortion(2)), // 40% of left half
                )
                .width(Length::FillPortion(1)); // 50% of total width

            Row::new()
                .push(left_group)
                .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
                .push(nonce_col.width(Length::FillPortion(1))) // 50% of total width
                .spacing(8)
                .into()
        } else {
            Row::new()
                .push(tx_type_col.width(Length::FillPortion(1))) // 50% width
                .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
                .push(nonce_col.width(Length::FillPortion(1))) // 50% width
                .spacing(8)
                .into()
        }
    }

    /// Max priority fee section for EIP-1559 (now integrated into tx_type_nonce_row)
    fn max_priority_fee_section(&self) -> Element<'_, Message> {
        // This section is now integrated into tx_type_nonce_row for better layout
        Column::new().into()
    }

    /// Gas speed selection buttons
    fn gas_speed_buttons(&self) -> Element<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Standard").size(11))
                    .on_press(Message::GasSpeedSelected(GasSpeed::Standard))
                    .padding([5, 8])
                    .style(if *self.gas_speed() == GasSpeed::Standard {
                        styles::primary_button()
                    } else {
                        styles::dark_slate_grey_button()
                    })
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Fast").size(11))
                    .on_press(Message::GasSpeedSelected(GasSpeed::Fast))
                    .padding([5, 8])
                    .style(if *self.gas_speed() == GasSpeed::Fast {
                        styles::primary_button()
                    } else {
                        styles::dark_slate_grey_button()
                    })
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Cancel TX").size(11))
                    .on_press(Message::CancelLastTransaction)
                    .padding([5, 8])
                    .style(styles::dark_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .spacing(0)
            .into()
    }

    /// Send button
    fn send_button(&self) -> Element<'_, Message> {
        Button::new(
            Text::new(if self.sending_transaction() {
                "Sending..."
            } else {
                "Send"
            })
            .size(14),
        )
        .on_press_maybe(
            if !self.send_to_address().is_empty()
                && !self.send_amount().is_empty()
                && self.send_from_account_id().is_some()
                && !self.sending_transaction()
            {
                Some(Message::SubmitTransaction)
            } else {
                None
            },
        )
        .padding([10, 16])
        .style(styles::lighter_slate_grey_button())
        .width(Length::Fill)
        .into()
    }

    /// Action buttons view (refresh, receive, history, etc.)
    fn action_buttons_view(&self) -> Element<'_, Message> {
        Row::new()
            .push(
                Button::new(
                    Text::new(if self.is_loading_state() {
                        "Refreshing..."
                    } else {
                        "Refresh"
                    })
                    .size(12),
                )
                .on_press_maybe(if !self.is_loading_state() {
                    Some(Message::RefreshBalance)
                } else {
                    None
                })
                .padding([8, 10])
                .style(styles::dark_slate_grey_button())
                .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Receive").size(12))
                    .on_press(Message::ShowReceive)
                    .padding([8, 10])
                    .style(styles::dark_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("History").size(12))
                    .on_press(Message::ShowHistory)
                    .padding([8, 10])
                    .style(styles::dark_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Dapps").size(12))
                    .on_press(Message::ShowDappsComingSoon)
                    .padding([8, 10])
                    .style(styles::dark_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .spacing(0)
            .into()
    }

    /// Wallet management buttons (create, import, export, hardware)
    fn wallet_management_buttons_view(&self) -> Element<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Create").size(12))
                    .on_press(Message::ShowCreateWallet)
                    .padding([8, 10])
                    .style(styles::dark_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Import").size(12))
                    .on_press(Message::ShowImportWallet)
                    .padding([8, 10])
                    .style(styles::lighter_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Export").size(12))
                    .on_press(Message::ShowExportWallet)
                    .padding([8, 10])
                    .style(styles::lighter_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
            .push(
                Button::new(Text::new("Hardware").size(12))
                    .on_press(Message::ShowHardWallet)
                    .padding([8, 10])
                    .style(styles::dark_slate_grey_button())
                    .width(Length::FillPortion(1)),
            )
            .spacing(0)
            .into()
    }

    /// Price info panel
    fn price_info_panel(&self) -> Element<'_, Message> {
        let mut content = Column::new()
            .push(
                Row::new()
                    .push(
                        Text::new("💰 Price Information")
                            .size(16)
                            .style(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                    )
                    .push(Space::with_width(Length::Fill))
                    .push(
                        Button::new(Text::new("🔄").size(14))
                            .on_press(Message::RefreshEthPrice)
                            .padding([6, 8])
                            .style(styles::secondary_button())
                            .width(Length::Fixed(safe_dimension(35.0))),
                    )
                    .push(Space::with_width(Length::Fixed(safe_dimension(10.0))))
                    .push(
                        Button::new(Text::new("✖").size(14))
                            .on_press(Message::HidePriceInfo)
                            .padding([6, 8])
                            .style(iced::theme::Button::Destructive)
                            .width(Length::Fixed(safe_dimension(35.0))),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .push(Space::with_height(Length::Fixed(safe_dimension(12.0))))
            .spacing(5);

        // ETH Price Section
        let eth_price_text = if self.eth_price_loading() {
            "Loading ETH price...".to_string()
        } else {
            format!("ETH: ${:.2}", self.current_eth_price().unwrap_or(0.0))
        };

        // Balance section
        let eth_balance = self
            .token_balances()
            .iter()
            .find(|token| token.symbol == "ETH")
            .map(|token| safe_balance(&token.balance))
            .unwrap_or_else(|| "0.0000".to_string());

        let balance_usd_value = if let Some(price) = self.current_eth_price() {
            let balance_f64 = eth_balance.parse::<f64>().unwrap_or(0.0);
            format!("{:.2}", safe_usd_value(balance_f64, price))
        } else {
            "0.00".to_string()
        };

        let price_section = Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(Text::new(eth_price_text.clone()).size(14))
                        .push(Space::with_width(Length::Fill))
                        .push(Text::new("Gas: ⟳").size(12).style(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                )
                .push(Space::with_height(Length::Fixed(safe_dimension(8.0))))
                .push(
                    Row::new()
                        .push(Text::new(format!("Balance: {eth_balance} ETH")).size(13))
                        .push(Space::with_width(Length::Fill))
                        .push(
                            Text::new(format!("≈ ${balance_usd_value}"))
                                .size(13)
                                .style(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                        ),
                )
                .spacing(4),
        )
        .padding(safe_dimension(16.0))
        .style(styles::dark_flat_container())
        .width(Length::Fill);

        content = content.push(price_section);

        Container::new(content).padding(0).width(Length::Fill).into()
    }

    /// Status message view
    fn status_message_view(&self) -> Element<'_, Message> {
        let message_color = match *self.status_message_color() {
            StatusMessageColor::Default => iced::Color::from_rgb(0.8, 0.8, 0.8),
            StatusMessageColor::Success => iced::Color::from_rgb(0.0, 0.7, 0.0),
            StatusMessageColor::Error => iced::Color::from_rgb(0.8, 0.0, 0.0),
            StatusMessageColor::Warning => iced::Color::from_rgb(0.8, 0.6, 0.0),
            StatusMessageColor::Info => iced::Color::from_rgb(0.0, 0.5, 0.8),
        };

        Container::new(Text::new(self.status_message()).size(14).style(message_color))
            .padding(safe_dimension(12.0))
            .style(styles::dark_flat_container())
            .width(Length::Fill)
            .into()
    }

    /// Pending transactions view with retro green loading bar
    fn pending_transactions_view(&self) -> Element<'_, Message> {
        let cancellable_count = self
            .transaction()
            .pending_transactions
            .iter()
            .filter(|tx| tx.cancellable)
            .count();

        // Retro horizontal loading bar with blocks moving left to right
        let frame = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 150) as usize;

        let bar_length = 20;
        let mut loading_bar = String::new();
        let filled_blocks = (frame % (bar_length + 4)) as i32 - 2;

        for i in 0..bar_length {
            if i as i32 >= filled_blocks - 2 && i as i32 <= filled_blocks {
                loading_bar.push('█'); // Filled block
            } else {
                loading_bar.push('▒'); // Empty/background block
            }
        }

        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Text::new(format!("{cancellable_count} transaction(s) pending"))
                                .size(14)
                                .style(iced::Color::from_rgb(0.0, 0.8, 0.0)),
                        )
                        .align_items(iced::Alignment::Center),
                )
                .push(Space::with_height(Length::Fixed(safe_dimension(8.0))))
                .push(
                    Text::new(loading_bar)
                        .size(12)
                        .style(iced::Color::from_rgb(0.0, 0.8, 0.0))
                        .font(iced::Font::MONOSPACE),
                )
                .align_items(iced::Alignment::Center),
        )
        .padding(safe_dimension(12.0))
        .style(styles::dark_flat_container())
        .width(Length::Fill)
        .into()
    }
}
