//! Button styles for Vaughan wallet theme
//!
//! This module contains all button style implementations following Iced 0.12
//! StyleSheet pattern. Button styles are categorized by their semantic purpose.

use iced::widget::button;
use iced::{Background, Border, Color};

use super::colors::VaughanColors;

// ============================================================================
// Primary Button Styles
// ============================================================================

/// Primary button style - main action buttons
pub struct PrimaryButtonStyle;

impl button::StyleSheet for PrimaryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::PRIMARY)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border {
                color: VaughanColors::PRIMARY_HOVER,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::PRIMARY_HOVER)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border {
                color: VaughanColors::PRIMARY_ACTIVE,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::PRIMARY_ACTIVE)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border {
                color: VaughanColors::PRIMARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::SECONDARY)),
            text_color: VaughanColors::TEXT_DISABLED,
            border: Border {
                color: VaughanColors::BORDER_PRIMARY,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

// ============================================================================
// Status Button Styles
// ============================================================================

/// Success button style - for positive actions
pub struct SuccessButtonStyle;

impl button::StyleSheet for SuccessButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::SUCCESS)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::SUCCESS_HOVER)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

/// Warning button style - for caution actions
pub struct WarningButtonStyle;

impl button::StyleSheet for WarningButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::WARNING)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::WARNING_HOVER)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

/// Danger button style - for destructive actions
pub struct DangerButtonStyle;

impl button::StyleSheet for DangerButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::ERROR)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::ERROR_HOVER)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

// ============================================================================
// Secondary and Utility Button Styles
// ============================================================================

/// Secondary button style - for less important actions
pub struct SecondaryButtonStyle;

impl button::StyleSheet for SecondaryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::BACKGROUND_TERTIARY)),
            text_color: VaughanColors::TEXT_SECONDARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::SECONDARY)),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

/// Disabled button style
pub struct DisabledButtonStyle;

impl button::StyleSheet for DisabledButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(VaughanColors::SECONDARY)),
            text_color: VaughanColors::TEXT_DISABLED,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

// ============================================================================
// Dark Theme Button Styles
// ============================================================================

/// Dark grey button style for special dark themes
pub struct DarkGreyButtonStyle;

impl button::StyleSheet for DarkGreyButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            text_color: Color::from_rgb(0.8, 0.8, 0.8),
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.25))),
            text_color: Color::WHITE,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            border: Border {
                color: Color::from_rgb(0.2, 0.2, 0.2),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

/// Transparent button style for image-only buttons
pub struct TransparentButtonStyle;

impl button::StyleSheet for TransparentButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::default(),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
            text_color: VaughanColors::TEXT_PRIMARY,
            border: Border::default(),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

/// Dark slate grey button style for bottom buttons
pub struct DarkSlateGreyButtonStyle;

impl button::StyleSheet for DarkSlateGreyButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.17, 0.2))),
            text_color: Color::from_rgb(0.8, 0.8, 0.8),
            border: Border {
                color: Color::from_rgb(0.25, 0.27, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.22, 0.25))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.3, 0.32, 0.35),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.12, 0.15))),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            border: Border {
                color: Color::from_rgb(0.2, 0.22, 0.25),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

/// Lighter slate grey button style (for Send, Import, Export)
pub struct LighterSlateGreyButtonStyle;

impl button::StyleSheet for LighterSlateGreyButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.25, 0.27, 0.3))),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.32, 0.35))),
            text_color: Color::WHITE,
            border: Border::with_radius(0),
            shadow: iced::Shadow::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}
