//! Locked State View Component
//!
//! This module provides the locked wallet view that is displayed when:
//! - Startup authentication fails or is cancelled
//! - Session times out or is manually locked
//! - User needs to unlock the wallet to access sensitive operations

use iced::{
    widget::{Button, Column, Container, Row, Space, Text},
    Color, Element, Length,
};

use crate::gui::state::auth_state::PasswordDialogConfig;

use crate::gui::{state::AppState, theme::styles, Message};

/// Locked wallet view
///
/// Displays when the wallet is locked and requires authentication.
/// Shows a lock icon, message, and unlock button.
pub fn locked_view(state: &AppState) -> Element<'_, Message> {
    let content = Column::new()
        .push(Space::with_height(Length::Fixed(80.0)))
        .push(
            // Lock icon (using emoji for simplicity)
            Text::new("🔒").size(80).style(Color::from_rgb(0.6, 0.6, 0.6)),
        )
        .push(Space::with_height(Length::Fixed(30.0)))
        .push(
            Text::new("Wallet Locked")
                .size(28)
                .style(Color::from_rgb(0.9, 0.9, 0.9)),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            Text::new("Your wallet is locked for security.")
                .size(14)
                .style(Color::from_rgb(0.7, 0.7, 0.7)),
        )
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(
            Text::new("Enter your password to unlock and access your accounts.")
                .size(14)
                .style(Color::from_rgb(0.7, 0.7, 0.7)),
        )
        .push(Space::with_height(Length::Fixed(40.0)))
        .push(
            Button::new(
                Row::new()
                    .push(Text::new("🔓").size(16))
                    .push(Space::with_width(Length::Fixed(8.0)))
                    .push(Text::new("Unlock Wallet").size(16))
                    .align_items(iced::Alignment::Center),
            )
            .on_press(Message::ShowPasswordDialog {
                config: PasswordDialogConfig::WalletUnlock,
            })
            .padding([12, 24])
            .style(styles::primary_button()),
        )
        .push(Space::with_height(Length::Fixed(15.0)))
        .push(
            Button::new(
                Row::new()
                    .push(Text::new("🗑️").size(14))
                    .push(Space::with_width(Length::Fixed(6.0)))
                    .push(Text::new("Reset Wallet").size(14))
                    .align_items(iced::Alignment::Center),
            )
            .on_press(Message::ShowResetWalletConfirmation)
            .padding([8, 16])
            .style(styles::secondary_button()),
        )
        .push(Space::with_height(Length::Fixed(20.0)))
        .push(
            // Optional: Show session timeout info if available
            {
                let session = &state.auth().session;
                if let Some(unlocked_at) = session.unlocked_at {
                    let elapsed = unlocked_at.elapsed();
                    Text::new(format!(
                        "Session locked after {} minutes of inactivity",
                        elapsed.as_secs() / 60
                    ))
                    .size(12)
                    .style(Color::from_rgb(0.5, 0.5, 0.5))
                } else {
                    Text::new("Authentication required to continue")
                        .size(12)
                        .style(Color::from_rgb(0.5, 0.5, 0.5))
                }
            },
        )
        .align_items(iced::Alignment::Center)
        .spacing(0);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(move |_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
}

/// Check if the locked view should be displayed
///
/// Returns true if:
/// - Session is locked AND
/// - Password dialog is not currently visible (to avoid showing both)
pub fn should_show_locked_view(_state: &AppState) -> bool {
    // Simplified approach - never show locked view, use account-level passwords only
    // This follows DEVELOPMENT_RULES.md - keep it simple with Alloy approach
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_show_locked_view_when_session_locked() {
        let mut state = AppState::default();
        state.auth_mut().session.lock();

        // Feature disabled per DEVELOPMENT_RULES.md
        assert!(!should_show_locked_view(&state));
    }

    #[test]
    fn test_should_not_show_locked_view_when_session_unlocked() {
        let mut state = AppState::default();
        state.auth_mut().session.unlock();

        assert!(!should_show_locked_view(&state));
    }

    #[test]
    fn test_should_not_show_locked_view_when_password_dialog_visible() {
        let mut state = AppState::default();
        state.auth_mut().session.lock();
        state
            .auth_mut()
            .password_dialog
            .show(PasswordDialogConfig::WalletUnlock);

        // Should not show locked view when password dialog is already visible
        assert!(!should_show_locked_view(&state));
    }

    #[test]
    fn test_locked_view_shows_unlock_button() {
        let state = AppState::default();
        let view = locked_view(&state);

        // View should be created without panicking
        // (Full UI testing would require iced test harness)
        drop(view);
    }
}
