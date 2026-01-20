//! Color palettes for Vaughan wallet theme system
//!
//! This module contains all color definitions used throughout the application.
//! Colors are organized into semantic groups for consistent styling.

use iced::Color;

// ============================================================================
// VaughanColors - Main Professional Color Palette
// ============================================================================

/// Professional color palette for the Vaughan wallet
pub struct VaughanColors;

impl VaughanColors {
    // Primary brand colors - changed to dark grey
    pub const PRIMARY: Color = Color::from_rgb(0.3, 0.3, 0.3);
    pub const PRIMARY_HOVER: Color = Color::from_rgb(0.4, 0.4, 0.4);
    pub const PRIMARY_ACTIVE: Color = Color::from_rgb(0.25, 0.25, 0.25);

    /// Create a color with alpha transparency
    pub fn with_alpha(color: Color, alpha: f32) -> Color {
        Color::from_rgba(color.r, color.g, color.b, alpha)
    }

    /// Create a primary color with alpha
    pub fn primary_with_alpha(alpha: f32) -> Color {
        Self::with_alpha(Self::PRIMARY, alpha)
    }

    /// Create a success color with alpha
    pub fn success_with_alpha(alpha: f32) -> Color {
        Self::with_alpha(Self::SUCCESS, alpha)
    }

    /// Create an error color with alpha
    pub fn error_with_alpha(alpha: f32) -> Color {
        Self::with_alpha(Self::ERROR, alpha)
    }

    // Secondary colors
    pub const SECONDARY: Color = Color::from_rgb(0.5, 0.5, 0.5);
    pub const SECONDARY_HOVER: Color = Color::from_rgb(0.6, 0.6, 0.6);

    // Status colors
    pub const SUCCESS: Color = Color::from_rgb(0.0, 0.8, 0.0);
    pub const SUCCESS_HOVER: Color = Color::from_rgb(0.0, 0.9, 0.0);
    pub const WARNING: Color = Color::from_rgb(1.0, 0.7, 0.0);
    pub const WARNING_HOVER: Color = Color::from_rgb(1.0, 0.8, 0.0);
    pub const ERROR: Color = Color::from_rgb(1.0, 0.4, 0.4);
    pub const ERROR_HOVER: Color = Color::from_rgb(1.0, 0.5, 0.5);
    pub const INFO: Color = Color::from_rgb(0.0, 0.7, 1.0);

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.7, 0.7, 0.7);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.5, 0.5, 0.5);
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.3, 0.3, 0.3);

    // Background colors - Deep black theme
    pub const BACKGROUND_PRIMARY: Color = Color::from_rgb(0.0, 0.0, 0.0); // Pure black
    pub const BACKGROUND_SECONDARY: Color = Color::from_rgb(0.02, 0.02, 0.02); // Almost black
    pub const BACKGROUND_TERTIARY: Color = Color::from_rgb(0.05, 0.05, 0.05); // Very dark
    pub const BACKGROUND_CARD: Color = Color::from_rgb(0.03, 0.03, 0.03); // Dark card

    // Border colors - Deep black theme with subtle borders
    pub const BORDER_PRIMARY: Color = Color::from_rgb(0.15, 0.15, 0.15); // Dark borders
    pub const BORDER_SECONDARY: Color = Color::from_rgb(0.25, 0.25, 0.25); // Slightly lighter
    pub const BORDER_FOCUS: Color = Color::from_rgb(0.2, 0.4, 0.8); // Blue focus
}

// ============================================================================
// BlackThemeColors - Deep Dark Theme Palette
// ============================================================================

/// Black theme color palette for deep dark themes
pub struct BlackThemeColors;

impl BlackThemeColors {
    // Background colors - pure black variations
    pub const BACKGROUND_PRIMARY: Color = Color::from_rgb(0.0, 0.0, 0.0); // Pure black
    pub const BACKGROUND_SECONDARY: Color = Color::from_rgb(0.02, 0.02, 0.02); // Almost black
    pub const BACKGROUND_TERTIARY: Color = Color::from_rgb(0.05, 0.05, 0.05); // Very dark
    pub const BACKGROUND_CARD: Color = Color::from_rgb(0.03, 0.03, 0.03); // Dark card

    // Text colors - high contrast for readability
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.95); // Almost white
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.8, 0.8, 0.8); // Light grey
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.5, 0.5, 0.5); // Medium grey

    // Button colors for black themes
    pub const PRIMARY: Color = Color::from_rgb(0.1, 0.1, 0.1); // Dark primary
    pub const PRIMARY_HOVER: Color = Color::from_rgb(0.15, 0.15, 0.15); // Lighter on hover
    pub const SECONDARY: Color = Color::from_rgb(0.08, 0.08, 0.08); // Dark secondary

    // Special border colors for different black themes
    pub const STEEL_BORDER: Color = Color::from_rgb(0.2, 0.22, 0.25); // Steel-like dark
    pub const VOID_BORDER: Color = Color::from_rgb(0.1, 0.1, 0.1); // Very dark border
    pub const MATTE_BORDER: Color = Color::from_rgb(0.15, 0.15, 0.15); // Subtle matte border
    pub const STEEL_BORDER_FOCUS: Color = Color::from_rgb(0.3, 0.35, 0.4); // Steel focus
    pub const VOID_BORDER_FOCUS: Color = Color::from_rgb(0.2, 0.2, 0.2); // Void focus
    pub const MATTE_BORDER_FOCUS: Color = Color::from_rgb(0.25, 0.25, 0.25); // Matte focus
}
