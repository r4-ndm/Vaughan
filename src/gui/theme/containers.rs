//! Container styles for Vaughan wallet theme
//!
//! This module contains all container style implementations following Iced 0.12
//! StyleSheet pattern. Container styles are categorized by their semantic purpose.

use iced::widget::container;
use iced::{Background, Border, Color};

use super::colors::VaughanColors;

// ============================================================================
// Card and Panel Container Styles
// ============================================================================

/// Card container style - for content sections
pub struct CardContainerStyle;

impl container::StyleSheet for CardContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(VaughanColors::TEXT_PRIMARY),
            background: Some(Background::Color(VaughanColors::BACKGROUND_CARD)),
            border: Border {
                color: VaughanColors::BORDER_PRIMARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
        }
    }
}

/// Accent container style - for status indicators
pub struct AccentContainerStyle;

impl container::StyleSheet for AccentContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(VaughanColors::TEXT_PRIMARY),
            background: Some(Background::Color(VaughanColors::BACKGROUND_TERTIARY)),
            border: Border {
                color: VaughanColors::PRIMARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ============================================================================
// Status Container Styles
// ============================================================================

/// Info container style - for informational content
pub struct InfoContainerStyle;

impl container::StyleSheet for InfoContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(VaughanColors::INFO),
            background: None,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

/// Success container style - for success messages/status
pub struct SuccessContainerStyle;

impl container::StyleSheet for SuccessContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(VaughanColors::SUCCESS),
            background: Some(Background::Color(VaughanColors::with_alpha(
                VaughanColors::SUCCESS,
                0.1,
            ))),
            border: Border {
                color: VaughanColors::SUCCESS,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ============================================================================
// Dark Theme Container Styles
// ============================================================================

/// Dark flat container style (no shadow, square corners)
pub struct DarkFlatContainerStyle;

impl container::StyleSheet for DarkFlatContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(VaughanColors::TEXT_PRIMARY),
            background: Some(Background::Color(VaughanColors::BACKGROUND_SECONDARY)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}
