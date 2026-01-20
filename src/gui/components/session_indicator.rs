//! Session Status Indicator Component
//!
//! Displays the current session lock status and provides manual lock button.

use iced::{
    widget::{Button, Row, Text, Tooltip},
    Color, Element,
};

use crate::gui::{state::AppState, theme::styles, Message};

/// Session status indicator view
pub fn session_indicator_view(state: &AppState) -> Element<'_, Message> {
    let session = &state.auth().session;

    // Build the indicator
    let mut indicator = Row::new().spacing(10).align_items(iced::Alignment::Center);

    // Lock/Unlock icon and status
    if session.is_unlocked {
        // Unlocked - show unlock icon and manual lock button
        indicator = indicator
            .push(Text::new("🔓").size(16).style(Color::from_rgb(0.4, 0.8, 0.4)))
            .push(Text::new("Unlocked").size(12).style(Color::from_rgb(0.7, 0.7, 0.7)));

        // Show countdown if auto-lock is enabled
        if session.auto_lock_enabled {
            if let Some(remaining) = session.time_until_timeout() {
                let minutes = remaining.as_secs() / 60;
                let seconds = remaining.as_secs() % 60;

                let countdown_text = if minutes > 0 {
                    format!("({minutes}:{seconds:02})")
                } else {
                    format!("({seconds}s)")
                };

                indicator = indicator.push(Text::new(countdown_text).size(11).style(Color::from_rgb(0.5, 0.5, 0.5)));
            }
        }

        // Manual lock button
        indicator = indicator.push(Tooltip::new(
            Button::new(Text::new("🔒").size(14))
                .on_press(Message::ManualLock)
                .padding([4, 8])
                .style(styles::secondary_button()),
            "Lock Session",
            iced::widget::tooltip::Position::Bottom,
        ));
    } else {
        // Locked - show lock icon
        indicator = indicator
            .push(Text::new("🔒").size(16).style(Color::from_rgb(0.8, 0.4, 0.4)))
            .push(Text::new("Locked").size(12).style(Color::from_rgb(0.7, 0.7, 0.7)));
    }

    indicator.into()
}

/// Compact session indicator (just icon)
pub fn session_indicator_compact(state: &AppState) -> Element<'_, Message> {
    let session = &state.auth().session;

    let icon = if session.is_unlocked {
        Text::new("🔓").size(16).style(Color::from_rgb(0.4, 0.8, 0.4))
    } else {
        Text::new("🔒").size(16).style(Color::from_rgb(0.8, 0.4, 0.4))
    };

    Tooltip::new(
        icon,
        if session.is_unlocked {
            "Session Unlocked"
        } else {
            "Session Locked"
        },
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_indicator_locked() {
        let state = AppState::default();
        // Default state is locked
        assert!(!state.auth().session.is_unlocked);
    }

    #[test]
    fn test_session_indicator_unlocked() {
        let mut state = AppState::default();
        state.auth_mut().session.unlock();
        assert!(state.auth().session.is_unlocked);
    }
}
