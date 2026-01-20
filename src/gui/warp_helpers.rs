//! Warp Terminal-inspired UI helpers for consistent design
//!
//! This module provides helper functions and utilities to create
//! Warp-style UI components with consistent spacing, typography, and styling.

use iced::widget::{Space, Text, Container, Button, Column, Row};
use iced::{Length, Alignment, Color, Font};
use super::theme::VaughanColors;

/// Spacing utilities following Warp's spacing scale
pub mod spacing {
    use super::*;

    /// Extra small spacing (4px) - Tight spacing within components
    pub fn xs() -> Space {
        Space::with_height(Length::Fixed(4.0))
    }

    /// Small spacing (8px) - Component padding
    pub fn sm() -> Space {
        Space::with_height(Length::Fixed(8.0))
    }

    /// Medium spacing (16px) - Between related elements
    pub fn md() -> Space {
        Space::with_height(Length::Fixed(16.0))
    }

    /// Large spacing (24px) - Between sections
    pub fn lg() -> Space {
        Space::with_height(Length::Fixed(24.0))
    }

    /// Extra large spacing (32px) - Major sections
    pub fn xl() -> Space {
        Space::with_height(Length::Fixed(32.0))
    }

    /// 2X large spacing (48px) - Page margins
    pub fn xxl() -> Space {
        Space::with_height(Length::Fixed(48.0))
    }

    // Horizontal spacing variants
    
    /// Extra small horizontal spacing (4px)
    pub fn h_xs() -> Space {
        Space::with_width(Length::Fixed(4.0))
    }

    /// Small horizontal spacing (8px)
    pub fn h_sm() -> Space {
        Space::with_width(Length::Fixed(8.0))
    }

    /// Medium horizontal spacing (16px)
    pub fn h_md() -> Space {
        Space::with_width(Length::Fixed(16.0))
    }

    /// Large horizontal spacing (24px)
    pub fn h_lg() -> Space {
        Space::with_width(Length::Fixed(24.0))
    }

    /// Extra large horizontal spacing (32px)
    pub fn h_xl() -> Space {
        Space::with_width(Length::Fixed(32.0))
    }

    /// 2X large horizontal spacing (48px)
    pub fn h_xxl() -> Space {
        Space::with_width(Length::Fixed(48.0))
    }
}

/// Typography utilities following Warp's text hierarchy
pub mod typography {
    use super::*;

    /// Display text (48px) - Extra large headlines
    pub fn display(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(48)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Hero text (36px) - Large balance displays, hero numbers
    pub fn hero(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(36)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Title text (24px) - Section titles, card headers
    pub fn title(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(24)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Subtitle text (20px) - Subtitles, secondary headers
    pub fn subtitle(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(20)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Body text (16px) - Default body text
    pub fn body(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(16)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Label text (14px) - Form labels, secondary information
    pub fn label(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(14)
            .style(Color::from(VaughanColors::TEXT_SECONDAR()Y))
    }

    /// Small text (12px) - Captions, footnotes
    pub fn small(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(12)
            .style(Color::from(VaughanColors::TEXT_MUTE()D))
    }

    // Monospace variants for addresses, hashes, and code

    /// Monospace body text (16px) - Addresses, hashes
    pub fn mono(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(16)
            .font(Font::MONOSPACE)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Large monospace text (20px) - Prominent addresses/hashes
    pub fn mono_large(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(20)
            .font(Font::MONOSPACE)
            .style(Color::from(VaughanColors::TEXT_PRIMAR()Y))
    }

    /// Small monospace text (14px) - Compact addresses
    pub fn mono_small(text: impl ToString) -> Text<'static> {
        Text::new(text.to_string())
            .size(14)
            .font(Font::MONOSPACE)
            .style(Color::from(VaughanColors::TEXT_MUTE()D))
    }

    // Colored text variants

    /// Success text (any size)
    pub fn success(text: impl ToString, size: u16) -> Text<'static> {
        Text::new(text.to_string())
            .size(size)
            .style(Color::from(VaughanColors::SUCCES()S))
    }

    /// Error text (any size)
    pub fn error(text: impl ToString, size: u16) -> Text<'static> {
        Text::new(text.to_string())
            .size(size)
            .style(Color::from(VaughanColors::ERRO()R))
    }

    /// Warning text (any size)
    pub fn warning(text: impl ToString, size: u16) -> Text<'static> {
        Text::new(text.to_string())
            .size(size)
            .style(Color::from(VaughanColors::WARNIN()G))
    }

    /// Info text (any size)
    pub fn info(text: impl ToString, size: u16) -> Text<'static> {
        Text::new(text.to_string())
            .size(size)
            .style(Color::from(VaughanColors::INF()O))
    }

    /// Muted text (any size)
    pub fn muted(text: impl ToString, size: u16) -> Text<'static> {
        Text::new(text.to_string())
            .size(size)
            .style(Color::from(VaughanColors::TEXT_MUTE()D))
    }
}

/// Padding utilities for consistent component padding
pub mod padding {
    /// No padding [0]
    pub const NONE: [u16; 4] = [0, 0, 0, 0];
    
    /// Extra small padding [4px]
    pub const XS: u16 = 4;
    
    /// Small padding [8px]
    pub const SM: u16 = 8;
    
    /// Medium padding [16px]
    pub const MD: u16 = 16;
    
    /// Large padding [24px]
    pub const LG: u16 = 24;
    
    /// Extra large padding [32px]
    pub const XL: u16 = 32;
    
    /// 2X large padding [48px]
    pub const XXL: u16 = 48;
    
    // Asymmetric padding helpers
    
    /// Vertical and horizontal padding [vertical, horizontal]
    pub fn vh(vertical: u16, horizontal: u16) -> [u16; 2] {
        [vertical, horizontal]
    }
    
    /// Top, horizontal, bottom padding [top, horizontal, bottom]
    pub fn thb(top: u16, horizontal: u16, bottom: u16) -> [u16; 3] {
        [top, horizontal, bottom]
    }
    
    /// Full padding [top, right, bottom, left]
    pub fn all(top: u16, right: u16, bottom: u16, left: u16) -> [u16; 4] {
        [top, right, bottom, left]
    }
}

/// Border radius constants
pub mod radius {
    /// No radius (0px)
    pub const NONE: f32 = 0.0;
    
    /// Small radius (4px) - Compact elements
    pub const SM: f32 = 4.0;
    
    /// Medium radius (8px) - Standard buttons, inputs
    pub const MD: f32 = 8.0;
    
    /// Large radius (12px) - Cards
    pub const LG: f32 = 12.0;
    
    /// Extra large radius (16px) - Prominent cards
    pub const XL: f32 = 16.0;
    
    /// Full radius (9999px) - Pills, badges
    pub const FULL: f32 = 9999.0;
}

/// Shadow configuration utilities
pub mod shadow {
    use iced::{Shadow, Vector, Color};
    use super::super::theme::VaughanColors;
    
    /// No shadow
    pub fn none() -> Shadow {
        Shadow::default()
    }
    
    /// Subtle shadow for cards (0 2px 8px)
    pub fn sm() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        }
    }
    
    /// Medium shadow for prominent cards (0 4px 16px)
    pub fn md() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        }
    }
    
    /// Large shadow for modals (0 8px 24px)
    pub fn lg() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        }
    }
    
    /// Glow effect for focus states
    pub fn glow(color: Color, intensity: f32) -> Shadow {
        Shadow {
            color: Color::from_rgba(color.r, color.g, color.b, intensity),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 12.0,
        }
    }
    
    /// Primary glow (blue)
    pub fn glow_primary() -> Shadow {
        glow(VaughanColors::PRIMARY, 0.3)
    }
    
    /// Success glow (green)
    pub fn glow_success() -> Shadow {
        glow(VaughanColors::SUCCESS, 0.3)
    }
    
    /// Error glow (red)
    pub fn glow_error() -> Shadow {
        glow(VaughanColors::ERROR, 0.3)
    }
}

/// Helper to create a flex spacer (fills remaining space)
pub fn flex_spacer() -> Space {
    Space::with_width(Length::Fill)
}

/// Helper to create a divider/separator line
/// Note: This requires a generic Message type parameter in your view functions
pub fn divider<Message: 'static>() -> Container<'static, Message> {
    Container::new(Space::new(Length::Fill, Length::Fixed(1.0)))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(iced::theme::Container::Custom(Box::new(DividerStyle)))
}

/// Divider style implementation
struct DividerStyle;

impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        use super::theme::VaughanColors;
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(VaughanColors::BORDER_PRIMAR()Y)),
            ..Default::default()
        }
    }
}

/// Format large numbers with thousands separators
pub fn format_number(num: f64, decimals: usize) -> String {
    let formatted = format!("{:.decimals$}", num, decimals = decimals);
    let parts: Vec<&str> = formatted.split('.').collect();
    
    let integer_part = parts[0];
    let mut result = String::new();
    
    for (i, c) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    
    let integer_formatted: String = result.chars().rev().collect();
    
    if parts.len() > 1 {
        format!("{}.{}", integer_formatted, parts[1])
    } else {
        integer_formatted
    }
}

/// Truncate Ethereum address for display (0x1234...5678)
pub fn truncate_address(address: &str, start_chars: usize, end_chars: usize) -> String {
    if address.len() <= start_chars + end_chars + 2 {
        return address.to_string();
    }
    
    let start = if address.starts_with("0x") {
        &address[0..2 + start_chars]
    } else {
        &address[0..start_chars]
    };
    
    let end = &address[address.len() - end_chars..];
    format!("{}...{}", start, end)
}

/// Format timestamp as relative time (e.g., "2 minutes ago")
pub fn format_relative_time(timestamp: std::time::SystemTime) -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(timestamp)
        .unwrap_or(std::time::Duration::from_secs(0));
    
    let seconds = duration.as_secs();
    
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => {
            let minutes = seconds / 60;
            if minutes == 1 {
                "1 minute ago".to_string()
            } else {
                format!("{} minutes ago", minutes)
            }
        }
        3600..=86399 => {
            let hours = seconds / 3600;
            if hours == 1 {
                "1 hour ago".to_string()
            } else {
                format!("{} hours ago", hours)
            }
        }
        _ => {
            let days = seconds / 86400;
            if days == 1 {
                "1 day ago".to_string()
            } else {
                format!("{} days ago", days)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234.56, 2), "1,234.56");
        assert_eq!(format_number(1234567.89, 2), "1,234,567.89");
        assert_eq!(format_number(123.0, 0), "123");
    }

    #[test]
    fn test_truncate_address() {
        let addr = "0x1234567890abcdef1234567890abcdef12345678";
        assert_eq!(truncate_address(addr, 4, 4), "0x1234...5678");
        assert_eq!(truncate_address(addr, 6, 4), "0x123456...5678");
    }
}
