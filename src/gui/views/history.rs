//! History View Components
//!
//! This module contains the transaction history and wallet logs view
//! extracted from working_wallet.rs for better code organization.

use iced::{
    widget::{Button, Column, Container, Row, Scrollable, Space, Text},
    Element, Length,
};

use crate::gui::{
    theme::{styles, text},
    wallet_types::HistoryTab,
    working_wallet::AppState,
    Message,
};

impl AppState {
    /// Transaction history and wallet logs view (330 lines extracted from working_wallet.rs)
    pub fn history_view(&self) -> Element<'_, Message> {
        let content = Column::new()
            .push(
                // Header
                Row::new()
                    .push(
                        Button::new(Text::new("← Back"))
                            .on_press(Message::HideHistory)
                            .padding(8)
                            .style(styles::secondary_button()),
                    )
                    .push(Space::with_width(Length::Fixed(20.0)))
                    .push(Text::new("Wallet History").size(20))
                    .push(Space::with_width(Length::Fill))
                    .align_items(iced::Alignment::Center),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                // Tab selection
                Row::new()
                    .push(
                        Button::new(Text::new("Transaction History"))
                            .on_press(Message::HistoryTabSelected(HistoryTab::Transactions))
                            .padding([10, 16])
                            .style(if self.transaction().current_history_tab == HistoryTab::Transactions {
                                styles::primary_button()
                            } else {
                                styles::secondary_button()
                            })
                            .width(Length::FillPortion(1)),
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Button::new(Text::new("Wallet Logs"))
                            .on_press(Message::HistoryTabSelected(HistoryTab::WalletLogs))
                            .padding([10, 16])
                            .style(if self.transaction().current_history_tab == HistoryTab::WalletLogs {
                                styles::primary_button()
                            } else {
                                styles::secondary_button()
                            })
                            .width(Length::FillPortion(1)),
                    )
                    .spacing(10),
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                // Content based on selected tab
                match self.transaction().current_history_tab {
                    HistoryTab::Recent => self.transaction_history_content(),
                    HistoryTab::Transactions => self.transaction_history_content(),
                    HistoryTab::WalletLogs => self.wallet_logs_content(),
                },
            )
            .spacing(5);

        Container::new(content)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Transaction history content component
    fn transaction_history_content(&self) -> Element<'_, Message> {
        if self.transaction().loading_transactions {
            // Show loading spinner
            Container::new(
                Column::new()
                    .push(Text::new("🔄 Loading transactions...").size(16))
                    .push(Space::with_height(Length::Fixed(10.0)))
                    .push(Text::new("Fetching transaction history from blockchain").size(12))
                    .align_items(iced::Alignment::Center),
            )
            .padding(40)
            .style(styles::dark_flat_container())
            .width(Length::Fill)
            .into()
        } else if self.transaction().transaction_history.is_empty() {
            // Different messages based on whether fetch failed or history is genuinely empty
            let (title, subtitle) = if self.transaction().transaction_fetch_error {
                (
                    "Data unavailable",
                    "Unable to fetch transaction history from block explorer",
                )
            } else {
                ("No transactions yet", "This address has no transaction history")
            };

            Container::new(
                Column::new()
                    .push(Text::new(title).size(16))
                    .push(Space::with_height(Length::Fixed(10.0)))
                    .push(Text::new(subtitle).size(12))
                    .push(Space::with_height(Length::Fixed(20.0)))
                    .push(
                        Button::new(Text::new("🔄 Refresh").size(14))
                            .on_press(Message::ShowTransactionHistory)
                            .padding([8, 16])
                            .style(styles::primary_button()),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .padding(40)
            .style(styles::dark_flat_container())
            .width(Length::Fill)
            .into()
        } else {
            self.transaction_list_content()
        }
    }

    /// Transaction list content component
    fn transaction_list_content(&self) -> Element<'_, Message> {
        let mut tx_column = Column::new()
            .push(
                Row::new()
                    .push(Text::new(format!("{} Transactions", self.transaction().transaction_history.len())).size(14))
                    .push(Space::with_width(Length::Fill))
                    .push(
                        Button::new(Text::new("🔄 Refresh").size(12))
                            .on_press(Message::ShowTransactionHistory)
                            .padding([6, 12])
                            .style(styles::secondary_button()),
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Button::new(Text::new("🗑 Clear").size(12))
                            .on_press(Message::ClearTransactionHistory)
                            .padding([6, 12])
                            .style(styles::danger_button()),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .push(Space::with_height(Length::Fixed(15.0)))
            .spacing(8);

        // Get current account address for comparison
        let current_address = if let Some(account_id) = &self.wallet().current_account_id {
            if let Some(account) = self.wallet().available_accounts.iter().find(|a| &a.id == account_id) {
                format!("{:#x}", account.address).to_lowercase() // Use proper hex formatting without quotes
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Create transaction entries
        for tx in &self.transaction().transaction_history {
            let is_incoming = tx.to.to_lowercase() == current_address;
            let direction_icon = if is_incoming { "📥" } else { "📤" };
            let direction_text = if is_incoming { "Received" } else { "Sent" };
            let direction_color = if is_incoming {
                iced::Color::from_rgb(0.2, 0.8, 0.2) // Green for incoming
            } else {
                iced::Color::from_rgb(0.9, 0.5, 0.2) // Orange for outgoing
            };

            let tx_container = Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(Text::new(direction_icon).size(20))
                            .push(Space::with_width(Length::Fixed(10.0)))
                            .push(
                                Column::new()
                                    .push(
                                        Row::new()
                                            .push(Text::new(direction_text).size(14).style(direction_color))
                                            .push(Space::with_width(Length::Fixed(10.0)))
                                            .push(
                                                Text::new("Amount:")
                                                    .size(12)
                                                    .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                            )
                                            .push(Space::with_width(Length::Fixed(5.0)))
                                            .push(
                                                Text::new(&tx.amount)
                                                    .size(14)
                                                    .style(iced::Color::from_rgb(0.9, 0.9, 0.1)),
                                            )
                                            .push(Space::with_width(Length::Fill))
                                            .push(
                                                Text::new(&tx.timestamp)
                                                    .size(11)
                                                    .style(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                                            )
                                            .align_items(iced::Alignment::Center),
                                    )
                                    .push(Space::with_height(Length::Fixed(5.0)))
                                    .push(
                                        Row::new()
                                            .push(
                                                Text::new(if is_incoming { "From: " } else { "To: " })
                                                    .size(11)
                                                    .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                            )
                                            .push(
                                                Button::new(
                                                    Text::new(if is_incoming {
                                                        format!(
                                                            "{}...{}",
                                                            &tx.from[0..6],
                                                            &tx.from[tx.from.len() - 4..]
                                                        )
                                                    } else {
                                                        format!("{}...{}", &tx.to[0..6], &tx.to[tx.to.len() - 4..])
                                                    })
                                                    .size(11)
                                                    .style(iced::Color::from_rgb(0.7, 0.7, 0.9)),
                                                )
                                                .on_press(Message::CopyTransactionAddress(if is_incoming {
                                                    tx.from.clone()
                                                } else {
                                                    tx.to.clone()
                                                }))
                                                .padding(0)
                                                .style(iced::theme::Button::Text),
                                            )
                                            .push(Space::with_width(Length::Fixed(20.0)))
                                            .push(
                                                Text::new("Tx Hash: ")
                                                    .size(11)
                                                    .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                            )
                                            .push(
                                                Button::new(
                                                    Text::new({
                                                        // Format hash to show first 8 and last 4 characters for better readability
                                                        if tx.hash.len() > 12 {
                                                            format!(
                                                                "{}...{}",
                                                                &tx.hash[0..8],
                                                                &tx.hash[tx.hash.len() - 4..]
                                                            )
                                                        } else {
                                                            tx.hash.clone()
                                                        }
                                                    })
                                                    .size(11)
                                                    .style(iced::Color::from_rgb(0.7, 0.7, 0.9)),
                                                )
                                                .on_press(Message::CopyTransactionHash(tx.hash.clone()))
                                                .padding(0)
                                                .style(iced::theme::Button::Text),
                                            )
                                            .push(Space::with_width(Length::Fill))
                                            .push(Text::new(tx.status.text()).size(11).style(tx.status.color()))
                                            .align_items(iced::Alignment::Center),
                                    )
                                    .width(Length::Fill),
                            )
                            .align_items(iced::Alignment::Center),
                    )
                    .padding(12)
                    .spacing(5),
            )
            .padding(2)
            .style(styles::dark_flat_container())
            .width(Length::Fill);

            tx_column = tx_column.push(tx_container);
        }

        let scrollable = Scrollable::new(tx_column)
            .height(Length::Fixed(450.0))
            .width(Length::Fill);

        Container::new(scrollable).width(Length::Fill).into()
    }

    /// Wallet logs content component
    fn wallet_logs_content(&self) -> Element<'_, Message> {
        let mut logs_column = Column::new();

        // Simple action buttons
        let mut action_row = Row::new().spacing(10);

        let clear_button = Button::new(Text::new("Clear Logs"))
            .on_press(Message::ShowClearLogsConfirmation)
            .padding([8, 12])
            .style(styles::danger_button());

        action_row = action_row.push(clear_button);

        logs_column = logs_column
            .push(Text::new("Wallet Logs").size(16))
            .push(Space::with_height(Length::Fixed(5.0)))
            .push(action_row)
            .push(Space::with_height(Length::Fixed(15.0)));

        // Display all log entries without filtering
        let entries_to_show = &self.log_entries;

        if entries_to_show.is_empty() {
            logs_column = logs_column.push(
                Container::new(
                    Column::new()
                        .push(Text::new("No logs found").size(16))
                        .push(Space::with_height(Length::Fixed(5.0)))
                        .push(Text::new("No wallet logs available").size(12))
                        .align_items(iced::Alignment::Center),
                )
                .padding(20)
                .style(styles::dark_flat_container())
                .width(Length::Fill),
            );
        } else {
            logs_column = logs_column.push(Text::new(format!("{} log entries", entries_to_show.len())).size(12));

            // Create scrollable list of log entries
            let mut log_entries_column = Column::new().spacing(5);

            for (index, entry) in entries_to_show.iter().enumerate() {
                let mut entry_row = Row::new()
                    .spacing(10)
                    .align_items(iced::Alignment::Center)
                    .push(
                        Text::new(&entry.timestamp)
                            .size(11)
                            .style(text::muted())
                            .width(Length::Fixed(130.0)),
                    )
                    .push(Text::new(&entry.message).size(12).width(Length::Fill));

                // Add copy button
                let copy_button = Button::new(Text::new("Copy").size(10))
                    .on_press(Message::CopyLogEntry(index))
                    .padding([4, 8])
                    .style(styles::secondary_button());
                entry_row = entry_row.push(copy_button);

                let entry_container = Container::new(entry_row)
                    .padding(8)
                    .style(styles::dark_flat_container())
                    .width(Length::Fill);

                log_entries_column = log_entries_column.push(entry_container);
            }

            let scrollable_logs = Scrollable::new(log_entries_column).height(Length::Fixed(400.0));

            logs_column = logs_column.push(scrollable_logs);
        }

        // Show feedback message if any
        if let Some(feedback) = &self.ui().copy_feedback {
            logs_column = logs_column.push(Space::with_height(Length::Fixed(10.0))).push(
                Container::new(Text::new(feedback).size(12).style(iced::Color::from_rgb(0.1, 0.7, 0.1)))
                    .padding(5)
                    .style(styles::dark_flat_container()),
            );
        }

        Container::new(logs_column)
            .padding(10)
            .style(styles::dark_flat_container())
            .width(Length::Fill)
            .into()
    }
}
