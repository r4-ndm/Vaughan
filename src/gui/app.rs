//! Application-level utilities and helpers for Iced

use iced::{Theme, Color};

/// Application themes
pub enum AppTheme {
    Light,
    Dark,
    Custom(CustomTheme),
}

/// Custom theme definition
pub struct CustomTheme {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background_color: Color,
    pub text_color: Color,
}

impl From<AppTheme> for Theme {
    fn from(app_theme: AppTheme) -> Self {
        match app_theme {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
            AppTheme::Custom(_) => Theme::Dark, // Note: Custom theme implementation pending
        }
    }
}

/// Application constants
pub mod constants {
    pub const WINDOW_MIN_WIDTH: u32 = 600;
    pub const WINDOW_MIN_HEIGHT: u32 = 400;
    pub const DEFAULT_PADDING: u16 = 20;
    pub const DEFAULT_SPACING: u16 = 10;
}