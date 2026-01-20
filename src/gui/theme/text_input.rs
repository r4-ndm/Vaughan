//! Text input and text styling for Vaughan wallet theme
//!
//! This module contains text input styles and text color helpers
//! following Iced 0.12 StyleSheet pattern.

use iced::widget::text_input;
use iced::{Background, Border, Color};

use super::colors::VaughanColors;

// ============================================================================
// Primary Text Input
// ============================================================================

/// Primary text input style
pub struct PrimaryTextInputStyle;

impl text_input::StyleSheet for PrimaryTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(VaughanColors::BACKGROUND_SECONDARY),
            border: Border {
                color: VaughanColors::BORDER_PRIMARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_SECONDARY,
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(VaughanColors::BACKGROUND_SECONDARY),
            border: Border {
                color: VaughanColors::BORDER_FOCUS,
                width: 2.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_PRIMARY,
        }
    }

    fn hovered(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(VaughanColors::BACKGROUND_SECONDARY),
            border: Border {
                color: VaughanColors::BORDER_SECONDARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_SECONDARY,
        }
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(VaughanColors::BACKGROUND_PRIMARY),
            border: Border {
                color: VaughanColors::BORDER_PRIMARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_DISABLED,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::TEXT_MUTED
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::TEXT_PRIMARY
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::TEXT_DISABLED
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::PRIMARY
    }
}

// ============================================================================
// Black Theme Text Input Styles
// ============================================================================

/// Black theme text input with dark grey borders
pub struct BlackGreyTextInputStyle;

impl text_input::StyleSheet for BlackGreyTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::BLACK),
            border: Border {
                color: Color::from_rgb(0.1, 0.1, 0.1),
                width: 1.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_SECONDARY,
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::BLACK),
            border: Border {
                color: Color::from_rgb(0.2, 0.2, 0.2),
                width: 2.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_PRIMARY,
        }
    }

    fn hovered(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::BLACK),
            border: Border {
                color: Color::from_rgb(0.15, 0.15, 0.15),
                width: 1.5,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_SECONDARY,
        }
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::BLACK),
            border: Border {
                color: Color::from_rgb(0.1, 0.1, 0.1),
                width: 1.0,
                radius: 0.0.into(),
            },
            icon_color: VaughanColors::TEXT_DISABLED,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::TEXT_MUTED
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::TEXT_PRIMARY
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::TEXT_DISABLED
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        VaughanColors::PRIMARY
    }
}
