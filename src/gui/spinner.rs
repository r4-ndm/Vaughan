//! Loading spinner component for Vaughan wallet
//!
//! Provides animated loading indicators for various wallet operations

use iced::widget::{Container, Row, Text};
use iced::{alignment, time, Color, Element};
use std::time::{Duration, Instant};

/// Spinner styles matching different loading contexts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpinnerStyle {
    /// Small spinner for inline loading (e.g., balance refresh)
    Small,
    /// Medium spinner for dialog loading (e.g., transaction submission)
    Medium,
    /// Large spinner for full-screen loading (e.g., wallet initialization)
    Large,
}

/// Spinner animation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpinnerType {
    /// Classic rotating dots
    Dots,
    /// Pulsing circle
    Pulse,
    /// Loading bar
    Bar,
    /// Bouncing dots
    Bounce,
}

/// Spinner component state
#[derive(Debug, Clone)]
pub struct Spinner {
    pub style: SpinnerStyle,
    pub spinner_type: SpinnerType,
    pub message: Option<String>,
    pub start_time: Instant,
    pub color: Color,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            style: SpinnerStyle::Medium,
            spinner_type: SpinnerType::Dots,
            message: None,
            start_time: Instant::now(),
            color: Color::from_rgb(0.3, 0.7, 1.0), // Nice blue color
        }
    }
}

impl Spinner {
    /// Create a new spinner
    pub fn new(style: SpinnerStyle, spinner_type: SpinnerType) -> Self {
        Self {
            style,
            spinner_type,
            message: None,
            start_time: Instant::now(),
            color: Color::from_rgb(0.3, 0.7, 1.0),
        }
    }

    /// Set the loading message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Set the spinner color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Get the current animation frame based on elapsed time
    fn get_animation_frame(&self) -> usize {
        let elapsed = self.start_time.elapsed().as_millis();
        match self.spinner_type {
            SpinnerType::Dots => (elapsed / 200) as usize % 8,
            SpinnerType::Pulse => (elapsed / 100) as usize % 20,
            SpinnerType::Bar => (elapsed / 150) as usize % 10,
            SpinnerType::Bounce => (elapsed / 300) as usize % 6,
        }
    }

    /// Get spinner size based on style
    fn get_size(&self) -> (u16, u16) {
        match self.style {
            SpinnerStyle::Small => (16, 16),
            SpinnerStyle::Medium => (24, 24),
            SpinnerStyle::Large => (32, 32),
        }
    }

    /// Get text size based on style
    fn get_text_size(&self) -> u16 {
        match self.style {
            SpinnerStyle::Small => 12,
            SpinnerStyle::Medium => 14,
            SpinnerStyle::Large => 16,
        }
    }

    /// Generate the spinner visual based on type and frame
    fn get_spinner_text(&self) -> String {
        let frame = self.get_animation_frame();

        match self.spinner_type {
            SpinnerType::Dots => {
                let dots = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                dots[frame % dots.len()].to_string()
            }
            SpinnerType::Pulse => {
                let pulse = [
                    "●", "●", "●", "◐", "◐", "◑", "◑", "◒", "◒", "◓", "◓", "○", "○", "○", "○", "○", "○", "○", "○", "○",
                ];
                pulse[frame % pulse.len()].to_string()
            }
            SpinnerType::Bar => {
                let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆"];
                bars[frame % bars.len()].to_string()
            }
            SpinnerType::Bounce => {
                let bounce = ["⠁", "⠂", "⠄", "⠂", "⠁", "⠈"];
                bounce[frame % bounce.len()].to_string()
            }
        }
    }

    /// Create the spinner element
    pub fn view<Message: Clone + 'static>(&self) -> Element<'_, Message> {
        use iced::{Renderer, Theme};

        let spinner_text: iced::widget::Text<'_, Theme, Renderer> = Text::new(self.get_spinner_text())
            .size(self.get_size().0)
            .style(iced::theme::Text::Color(self.color));

        let content: Element<Message> = if let Some(ref message) = self.message {
            let message_text: iced::widget::Text<'_, Theme, Renderer> = Text::new(message)
                .size(self.get_text_size())
                .style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6)));

            Row::new()
                .spacing(8)
                .align_items(alignment::Alignment::Center)
                .push(spinner_text)
                .push(message_text)
                .into()
        } else {
            spinner_text.into()
        };

        Container::new(content).center_x().center_y().into()
    }
}

/// Predefined spinner configurations for common wallet operations
pub struct WalletSpinners;

impl WalletSpinners {
    /// Balance loading spinner
    pub fn balance_loading() -> Spinner {
        Spinner::new(SpinnerStyle::Small, SpinnerType::Dots)
            .with_message("Loading balance...".to_string())
            .with_color(Color::from_rgb(0.2, 0.8, 0.2))
    }

    /// Transaction submission spinner
    pub fn transaction_pending() -> Spinner {
        Spinner::new(SpinnerStyle::Medium, SpinnerType::Pulse)
            .with_message("Submitting transaction...".to_string())
            .with_color(Color::from_rgb(1.0, 0.6, 0.0))
    }

    /// Wallet initialization spinner
    pub fn wallet_initializing() -> Spinner {
        Spinner::new(SpinnerStyle::Large, SpinnerType::Dots)
            .with_message("Initializing wallet...".to_string())
            .with_color(Color::from_rgb(0.3, 0.7, 1.0))
    }

    /// Account loading spinner
    pub fn accounts_loading() -> Spinner {
        Spinner::new(SpinnerStyle::Medium, SpinnerType::Bar)
            .with_message("Loading accounts...".to_string())
            .with_color(Color::from_rgb(0.6, 0.4, 1.0))
    }

    /// Network connection spinner
    pub fn network_connecting() -> Spinner {
        Spinner::new(SpinnerStyle::Small, SpinnerType::Bounce)
            .with_message("Connecting...".to_string())
            .with_color(Color::from_rgb(0.0, 0.8, 0.8))
    }

    /// Retro green loading bar for pending transactions
    pub fn pending_transactions_retro() -> Spinner {
        Spinner::new(SpinnerStyle::Medium, SpinnerType::Bar)
            .with_message("Transaction pending...".to_string())
            .with_color(Color::from_rgb(0.0, 0.8, 0.0))
    }

    /// Generic loading spinner
    pub fn loading() -> Spinner {
        Spinner::new(SpinnerStyle::Medium, SpinnerType::Dots).with_message("Loading...".to_string())
    }
}

/// Subscription for spinner animation updates
pub fn spinner_subscription<Message: 'static>() -> iced::Subscription<Message>
where
    Message: Clone,
{
    time::every(Duration::from_millis(100)).map(|_| {
        // This is a placeholder - you'll need to adapt this to your Message type
        // For example: Message::SpinnerTick
        unreachable!("Implement spinner tick message in your app")
    })
}
