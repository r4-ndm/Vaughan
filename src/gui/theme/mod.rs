//! Professional theme system for Vaughan wallet - Iced 0.12 compatible
//!
//! This module provides consistent theming across the application using Iced's
//! theme system with custom color palettes and styling components.
//!
//! ## Module Structure
//! - `colors` - Color definitions (VaughanColors, BlackThemeColors)
//! - `buttons` - Button style implementations
//! - `containers` - Container style implementations
//! - `text` - Text input style implementations

// Submodules
pub mod buttons;
pub mod colors;
pub mod containers;
pub mod text_input;

// Re-exports from submodules
pub use buttons::*;
pub use colors::{BlackThemeColors, VaughanColors};
pub use containers::*;
pub use text_input::*;

use iced::overlay::menu;
use iced::widget::pick_list;
use iced::{Background, Border, Color, Theme};

// ============================================================================
// VaughanTheme - Main theme enum
// ============================================================================

/// Custom theme for the Vaughan wallet
#[derive(Debug, Clone, PartialEq, Default)]
pub enum VaughanTheme {
    #[default]
    DeepBlack, // Single deep black theme
}

impl From<VaughanTheme> for Theme {
    fn from(_theme: VaughanTheme) -> Self {
        // Create a pure black theme with grey buttons for better contrast
        crate::gui::utils::generate_black_theme(
            Color::from_rgb(0.35, 0.35, 0.35), // Grey buttons
            true,                              // monochrome mode for professional look
        )
    }
}

// ============================================================================
// Styles Module - Style wrapper functions
// ============================================================================

/// Professional styling helpers for Iced 0.12
pub mod styles {
    use super::*;

    /// Primary button style - main action buttons
    pub fn primary_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(PrimaryButtonStyle))
    }

    /// Success button style - for positive actions
    pub fn success_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(SuccessButtonStyle))
    }

    /// Warning button style - for caution actions
    pub fn warning_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(WarningButtonStyle))
    }

    /// Secondary button style - for less important actions
    pub fn secondary_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(SecondaryButtonStyle))
    }

    /// Card container style - for content sections
    pub fn card_container() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(CardContainerStyle))
    }

    /// Primary text input style
    pub fn primary_text_input() -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(PrimaryTextInputStyle))
    }

    /// Disabled button style
    pub fn disabled_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(DisabledButtonStyle))
    }

    /// Danger button style - for destructive actions
    pub fn danger_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(DangerButtonStyle))
    }

    /// Accent container style - for status indicators
    pub fn accent_container() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(AccentContainerStyle))
    }

    /// Info container style - for informational content
    pub fn info_container() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(InfoContainerStyle))
    }

    /// Success container style - for success messages/status
    pub fn success_container() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(SuccessContainerStyle))
    }

    /// Dark flat container style - no shadow, square corners
    pub fn dark_flat_container() -> iced::theme::Container {
        iced::theme::Container::Custom(Box::new(DarkFlatContainerStyle))
    }

    /// Black theme text input with dark grey borders
    pub fn black_grey_text_input() -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(BlackGreyTextInputStyle))
    }

    /// Dark grey button style for special dark themes
    pub fn dark_grey_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(DarkGreyButtonStyle))
    }

    /// Transparent button style for image-only buttons
    pub fn transparent_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(TransparentButtonStyle))
    }

    /// Dark slate grey button style for bottom buttons
    pub fn dark_slate_grey_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(DarkSlateGreyButtonStyle))
    }

    /// Lighter slate grey button style (for Send, Import, Export)
    pub fn lighter_slate_grey_button() -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(LighterSlateGreyButtonStyle))
    }

    /// Dark grey pick list style
    pub fn dark_grey_pick_list() -> iced::theme::PickList {
        iced::theme::PickList::Custom(
            std::rc::Rc::new(DarkGreyPickListStyle),
            std::rc::Rc::new(DarkGreyMenuPickListStyle),
        )
    }
}

// ============================================================================
// Text styling helpers
// ============================================================================

/// Text styling helpers for consistent typography
pub mod text {
    use super::*;

    /// Primary text color
    pub fn primary() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::TEXT_PRIMARY)
    }

    /// Secondary text color
    pub fn secondary() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::TEXT_SECONDARY)
    }

    /// Muted text color
    pub fn muted() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::TEXT_MUTED)
    }

    /// Success text color
    pub fn success() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::SUCCESS)
    }

    /// Warning text color
    pub fn warning() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::WARNING)
    }

    /// Error text color
    pub fn error() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::ERROR)
    }

    /// Danger text color (alias for error)
    pub fn danger() -> iced::theme::Text {
        iced::theme::Text::Color(VaughanColors::ERROR)
    }
}

// ============================================================================
// Network utilities
// ============================================================================

/// Network-specific utilities
pub mod network {
    use super::*;

    /// Get network-specific color
    pub fn color(network_id: u64) -> Color {
        match network_id {
            1 => Color::from_rgb(0.4, 0.6, 1.0),   // Ethereum - Blue
            56 => Color::from_rgb(1.0, 0.8, 0.0),  // BSC - Yellow
            137 => Color::from_rgb(0.5, 0.3, 1.0), // Polygon - Purple
            369 => Color::from_rgb(1.0, 0.4, 0.8), // PulseChain - Pink
            _ => Color::from_rgb(0.5, 0.5, 0.5),   // Unknown - Gray
        }
    }

    /// Get connection status color
    pub fn status_color(connected: bool) -> Color {
        if connected {
            VaughanColors::SUCCESS
        } else {
            VaughanColors::ERROR
        }
    }
}

// ============================================================================
// Theme utilities
// ============================================================================

/// Get the single deep black theme
pub fn get_theme() -> VaughanTheme {
    VaughanTheme::DeepBlack
}

// ============================================================================
// Theme persistence
// ============================================================================

/// Save theme preference to file
pub fn save_theme_preference(theme: &VaughanTheme) -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;

    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vaughan");
    path.push("vaughan_theme.json");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let theme_name = get_theme_name(theme);
    let preference = serde_json::json!({
        "theme_name": theme_name,
        "saved_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    std::fs::write(&path, serde_json::to_string_pretty(&preference)?)?;
    tracing::debug!("Theme saved: {:?}", theme);
    Ok(())
}

/// Load theme preference from disk - always returns DeepBlack
pub fn load_theme_preference() -> VaughanTheme {
    VaughanTheme::DeepBlack
}

/// Get theme name for logging
pub fn get_theme_name(_theme: &VaughanTheme) -> &'static str {
    "Deep Black"
}

// ============================================================================
// Pick List Styles
// ============================================================================

/// Dark grey pick list style
pub struct DarkGreyPickListStyle;

impl pick_list::StyleSheet for DarkGreyPickListStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: VaughanColors::TEXT_PRIMARY,
            placeholder_color: VaughanColors::TEXT_SECONDARY,
            handle_color: VaughanColors::TEXT_PRIMARY,
            background: Background::Color(VaughanColors::BACKGROUND_SECONDARY),
            border: Border {
                color: Color::from_rgb(0.25, 0.25, 0.25),
                width: 1.0,
                radius: 0.0.into(),
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: VaughanColors::TEXT_PRIMARY,
            placeholder_color: VaughanColors::TEXT_SECONDARY,
            handle_color: VaughanColors::TEXT_PRIMARY,
            background: Background::Color(VaughanColors::BACKGROUND_TERTIARY),
            border: Border {
                color: VaughanColors::BORDER_SECONDARY,
                width: 1.0,
                radius: 0.0.into(),
            },
        }
    }
}

/// Dark grey menu style for pick list
pub struct DarkGreyMenuPickListStyle;

impl menu::StyleSheet for DarkGreyMenuPickListStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> menu::Appearance {
        menu::Appearance {
            text_color: VaughanColors::TEXT_PRIMARY,
            background: Background::Color(VaughanColors::BACKGROUND_SECONDARY),
            border: Border {
                color: VaughanColors::BORDER_PRIMARY,
                width: 1.0,
                radius: 4.0.into(),
            },
            selected_text_color: VaughanColors::TEXT_PRIMARY,
            selected_background: Background::Color(VaughanColors::PRIMARY),
        }
    }
}
