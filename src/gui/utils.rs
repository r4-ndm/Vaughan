//! GUI Utility Functions
//!
//! This module contains utility functions for theme management, storage, and other GUI helpers.

use crate::network::NetworkId;
use alloy::primitives::U256;
use iced::{Color, Theme};
use std::path::PathBuf;

/// Convert HSL color to RGB
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let h = h / 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::from_rgb(r + m, g + m, b + m)
}

/// Generate a custom pure black theme with vibrant accent colors
pub fn generate_black_theme(accent_color: Color, monochrome: bool) -> Theme {
    use iced::theme::Palette;

    let primary_color = if monochrome {
        Color::from_rgb(0.35, 0.35, 0.35) // Grey buttons for monochrome
    } else {
        accent_color
    };

    // Create a truly black palette
    let palette = Palette {
        background: Color::from_rgb(0.0, 0.0, 0.0), // Pure black
        text: Color::from_rgb(0.9, 0.9, 0.9),       // Light grey text
        primary: primary_color,                     // Vibrant or grey
        success: Color::from_rgb(0.0, 0.7, 0.35),   // Green
        danger: Color::from_rgb(0.8, 0.2, 0.2),     // Red
    };

    Theme::custom("PureBlack".to_string(), palette)
}

/// Generate a vibrant accent color for black themes
pub fn generate_vibrant_accent_color() -> Color {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Generate vibrant colors with high saturation
    let hue: f32 = rng.gen_range(0.0..360.0);
    let saturation = rng.gen_range(0.8..1.0); // High saturation for vibrancy
    let lightness = rng.gen_range(0.5..0.7); // Medium-bright for visibility on black

    hsl_to_rgb(hue, saturation, lightness)
}

/// Get human-readable name for a theme
pub fn get_theme_name(theme: &Theme) -> &'static str {
    match theme {
        Theme::Light => "Light",
        Theme::Dark => "Dark",
        Theme::Dracula => "Dracula",
        Theme::Nord => "Nord",
        Theme::SolarizedLight => "Solarized Light",
        Theme::SolarizedDark => "Solarized Dark",
        Theme::GruvboxLight => "Gruvbox Light",
        Theme::GruvboxDark => "Gruvbox Dark",
        Theme::CatppuccinLatte => "Catppuccin Latte",
        Theme::CatppuccinFrappe => "Catppuccin Frappe",
        Theme::CatppuccinMacchiato => "Catppuccin Macchiato",
        Theme::CatppuccinMocha => "Catppuccin Mocha",
        Theme::TokyoNight => "Tokyo Night",
        Theme::TokyoNightStorm => "Tokyo Night Storm",
        Theme::TokyoNightLight => "Tokyo Night Light",
        Theme::KanagawaWave => "Kanagawa Wave",
        Theme::KanagawaDragon => "Kanagawa Dragon",
        Theme::KanagawaLotus => "Kanagawa Lotus",
        Theme::Moonfly => "Moonfly",
        Theme::Nightfly => "Nightfly",
        Theme::Oxocarbon => "Oxocarbon",
        _ => "Custom",
    }
}

/// Get the storage path for custom tokens
pub fn get_tokens_storage_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vaughan");
    path.push("custom_tokens.json");
    path
}

/// Format balance from wei to human-readable format
///
/// Converts a U256 wei value to a properly formatted decimal string
/// using Alloy's format_units to avoid precision loss.
pub fn format_balance(wei_balance: U256, network_id: NetworkId) -> String {
    // Get network info for currency symbol
    let (symbol, _decimal_places) = match network_id.chain_id() {
        1 => ("ETH", 18),     // Ethereum Mainnet
        56 => ("BNB", 18),    // BSC
        137 => ("MATIC", 18), // Polygon
        369 => ("PLS", 18),   // PulseChain
        943 => ("tPLS", 18),  // PulseChain Testnet
        42161 => ("ETH", 18), // Arbitrum One
        10 => ("ETH", 18),    // Optimism
        _ => ("ETH", 18),     // Default
    };

    if wei_balance == U256::ZERO {
        return format!("0.0000 {symbol}");
    }

    // Use Alloy's format_units for precise conversion
    // Standard EVM chains use 18 decimals for native currency
    let formatted = alloy::primitives::utils::format_units(wei_balance, 18).unwrap_or_else(|_| "0.0000".to_string());

    // Truncate to 6 decimal places for display consistency (matching legacy behavior)
    let truncated = truncate_to_decimals(&formatted, 6);

    format!("{truncated} {symbol}")
}

/// Format balance without symbol (for cases where symbol is displayed separately)
pub fn format_balance_value_only(wei_balance: U256) -> String {
    if wei_balance == U256::ZERO {
        return "0.0000".to_string();
    }

    let formatted = alloy::primitives::utils::format_units(wei_balance, 18).unwrap_or_else(|_| "0.0000".to_string());

    truncate_to_decimals(&formatted, 6)
}

/// Helper to truncate decimal string without rounding
fn truncate_to_decimals(value: &str, decimals: usize) -> String {
    if let Some(idx) = value.find('.') {
        if idx + 1 + decimals < value.len() {
            value[..idx + 1 + decimals].to_string()
        } else {
            value.to_string()
        }
    } else {
        value.to_string()
    }
}

/// Extract token address from display format
pub fn extract_token_address(token_display: &str) -> Option<String> {
    if token_display.contains("NATIVE") {
        Some("NATIVE".to_string())
    } else if token_display.contains("(0x") && token_display.contains(")") {
        let start = token_display.find("(0x")? + 1;
        let end = token_display[start..].find(')')?;
        Some(token_display[start..start + end].to_string())
    } else {
        None
    }
}

/// Get token symbol from display format based on network
pub fn get_token_symbol(token_display: &str, network_id: u64) -> String {
    if token_display.contains("NATIVE") {
        match network_id {
            137 => "MATIC".to_string(),
            _ => "ETH".to_string(),
        }
    } else if token_display.contains("USDC") {
        "USDC".to_string()
    } else if token_display.contains("USDT") {
        "USDT".to_string()
    } else {
        "TOKEN".to_string()
    }
}

/// Parse balance string to extract numeric value
pub fn parse_balance(balance_str: &str) -> Result<f64, std::num::ParseFloatError> {
    balance_str.split_whitespace().next().unwrap_or("0").parse::<f64>()
}

/// Create a clickable colored address display with copy functionality
pub fn create_full_address_display(
    address: String,
    just_copied: bool,
) -> iced::widget::Row<'static, crate::gui::Message> {
    use crate::gui::Message;
    use iced::{
        widget::{Button, Container, Row, Text},
        Color, Length,
    };

    if just_copied {
        // Show "Copied!" but keep the button clickable
        Row::new().push(
            Button::new(
                Container::new(Text::new("Copied!").size(18).style(Color::from_rgb(0.0, 0.9, 0.0)))
                    .width(Length::Fixed(420.0))
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .on_press(Message::CopyAddress(address))
            .padding([4, 8])
            .style(iced::theme::Button::Text)
            .width(Length::Fixed(440.0)),
        )
    } else {
        let addr_str = if address.starts_with("0x") {
            address[2..].to_string()
        } else {
            address.clone()
        };

        if addr_str.len() >= 40 {
            let first_part = addr_str[0..5].to_string();
            let non_colored_part = addr_str[5..18].to_string();
            let orange_part = addr_str[18..23].to_string();
            let grey_part = addr_str[23..35].to_string();
            let purple_part = addr_str[35..40].to_string();

            Row::new().push(
                Button::new(
                    Container::new(
                        Row::new()
                            .push(Text::new("0x").size(18).style(Color::from_rgb(0.5, 0.5, 0.5)))
                            .push(Text::new(first_part).size(18).style(Color::from_rgb(0.2, 0.8, 0.2)))
                            .push(
                                Text::new(non_colored_part)
                                    .size(18)
                                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
                            )
                            .push(Text::new(orange_part).size(18).style(Color::from_rgb(1.0, 0.6, 0.2)))
                            .push(Text::new(grey_part).size(18).style(Color::from_rgb(0.5, 0.5, 0.5)))
                            .push(Text::new(purple_part).size(18).style(Color::from_rgb(0.7, 0.3, 1.0))),
                    )
                    .width(Length::Fixed(420.0)),
                )
                .on_press(Message::CopyAddress(address.clone()))
                .padding([4, 8])
                .style(iced::theme::Button::Text)
                .width(Length::Fixed(440.0)),
            )
        } else {
            Row::new().push(
                Button::new(
                    Text::new(address.clone())
                        .size(18)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                )
                .on_press(Message::CopyAddress(address))
                .padding([4, 8])
                .style(iced::theme::Button::Text),
            )
        }
    }
}

/// Create sample transaction data for UI demonstration purposes
pub fn create_sample_transaction_data(address: &str) -> Result<Vec<crate::gui::Transaction>, String> {
    use crate::gui::{Transaction, TransactionStatus};

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get current time: {e}"))?
        .as_secs();

    let sample_transactions = vec![
        Transaction {
            hash: "0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            from: "0x742d35cc6634c053292925a3e5f9c673b9ef0102".to_string(),
            to: address.to_string(),
            amount: "0.5 ETH".to_string(),
            timestamp: chrono::DateTime::from_timestamp(now as i64 - 7200, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            status: TransactionStatus::Confirmed,
        },
        Transaction {
            hash: "0x9876543210fedcba0987654321fedcba0987654321fedcba0987654321fedcba".to_string(),
            from: address.to_string(),
            to: "0x8ba1f109551bd432803012645hac136c34c25d99".to_string(),
            amount: "0.25 ETH".to_string(),
            timestamp: chrono::DateTime::from_timestamp(now as i64 - 18000, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            status: TransactionStatus::Confirmed,
        },
        Transaction {
            hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            from: address.to_string(),
            to: "0x123456789abcdef0123456789abcdef0123456789".to_string(),
            amount: "0.1 ETH".to_string(),
            timestamp: chrono::DateTime::from_timestamp(now as i64 - 600, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            status: TransactionStatus::Pending,
        },
    ];

    Ok(sample_transactions)
}

/// Hardware wallet detection (stub implementation)
pub async fn detect_hardware_wallets() -> Vec<String> {
    // TODO: Implement actual hardware wallet detection
    vec![]
}

/// Connect to hardware wallet (stub implementation)
pub async fn connect_hardware_wallet(_device_index: usize) -> Result<String, String> {
    // TODO: Implement actual hardware wallet connection
    Ok("Hardware wallet connected".to_string())
}

/// Get hardware wallet addresses (stub implementation)
pub async fn get_hardware_wallet_addresses(_device_index: usize) -> Result<Vec<alloy::primitives::Address>, String> {
    // TODO: Implement actual address derivation
    Ok(vec![])
}

// === SOUND ALERT SYSTEM ===

/// Play notification sound (full implementation)
pub fn play_notification_sound() -> Result<(), Box<dyn std::error::Error>> {
    let sounds = get_available_sounds();

    // Use default preferred sound or first available
    let sound_name = if sounds.contains(&"coin_ding.wav".to_string()) {
        "coin_ding"
    } else if !sounds.is_empty() {
        &sounds[0].replace(".wav", "")
    } else {
        return Err("No sound files found in config/sounds/".into());
    };

    play_notification_sound_by_name(sound_name)
}

/// Play specific notification sound by name
pub fn play_notification_sound_by_name(sound_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sound_path = get_sound_path(sound_name);

    // Check if file exists
    if !std::path::Path::new(&sound_path).exists() {
        return Err(format!("Sound file not found: {}", sound_path).into());
    }

    // Play sound using paplay
    let output = std::process::Command::new("paplay").arg(&sound_path).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                tracing::debug!("✅ Successfully played sound: {}", sound_path);
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("paplay failed: {}", error).into())
            }
        }
        Err(e) => Err(format!("Failed to execute paplay: {}", e).into()),
    }
}

/// Get all available sound files in config/sounds/
pub fn get_available_sounds() -> Vec<String> {
    let sounds_dir = "config/sounds";
    let mut sounds = Vec::new();

    if let Ok(entries) = std::fs::read_dir(sounds_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".wav") {
                    sounds.push(file_name.to_string());
                }
            }
        }
    }

    // Sort alphabetically
    sounds.sort();
    sounds
}

/// Get full path to sound file
pub fn get_sound_path(sound_name: &str) -> String {
    let filename = if sound_name.ends_with(".wav") {
        sound_name.to_string()
    } else {
        format!("{}.wav", sound_name)
    };

    format!("config/sounds/{}", filename)
}

/// Convert filename to display name (coin_ding.wav -> Coin Ding)
pub fn get_sound_display_name(sound_name: &str) -> String {
    let name = sound_name.replace(".wav", "");
    let words: Vec<String> = name
        .split('_')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if !chars.is_empty() {
                chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
            }
            chars.into_iter().collect()
        })
        .collect();

    words.join(" ")
}

/// List all available sounds with display information
pub fn list_available_sounds() {
    println!("📁 Scanning config/sounds/ directory...");

    let sounds = get_available_sounds();

    if sounds.is_empty() {
        println!("❌ No .wav files found in config/sounds/");
        println!("💡 Add .wav files to config/sounds/ to enable audio notifications");
        return;
    }

    println!("✅ Found {} sound file(s):", sounds.len());
    println!();

    for (i, sound) in sounds.iter().enumerate() {
        let display_name = get_sound_display_name(sound);
        let file_path = get_sound_path(sound);

        // Check file size
        let file_size = std::fs::metadata(&file_path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        let size_kb = file_size / 1024;

        println!("{}. 🎵 {} ({})", i + 1, display_name, sound);
        println!("   📁 Path: {}", file_path);
        println!("   📏 Size: {} KB", size_kb);

        if sound == "coin_ding.wav" {
            println!("   ⭐ DEFAULT sound");
        }

        println!();
    }
}
