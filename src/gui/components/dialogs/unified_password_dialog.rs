//! Unified Password Dialog Component
//!
//! A single, flexible secure password input dialog that handles all authentication scenarios:
//! - Wallet unlocking
//! - Account unlocking
//! - Transaction signing
//! - Wallet setup/creation
//! - Password changes
//! - Sensitive data export
//!
//! Replaces:
//! - wallet_password_dialog.rs
//! - master_password_dialog.rs
//! - password_dialog.rs

use iced::{
    widget::{button, checkbox, column, container, row, text, Space, TextInput},
    Color, Element, Length,
};
use secrecy::ExposeSecret;

use crate::gui::{
    state::{
        auth_state::{AccountCreationType, PasswordDialogConfig, PasswordError, WalletOperation},
        AppState,
    },
    theme::styles::{dark_flat_container, primary_button, secondary_button},
    Message,
};

/// Main view function for the unified password dialog
pub fn password_dialog_view(state: &AppState) -> Element<'_, Message> {
    let dialog_state = &state.auth().password_dialog;

    if !dialog_state.visible || dialog_state.config.is_none() {
        return Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into();
    }

    let config = dialog_state.config.as_ref().unwrap();
    let (title, description, input_label, button_text) = get_dialog_content(config);
    let show_cancel = should_show_cancel(config);
    let show_remember = should_show_remember(config);
    let _is_confirm_flow = is_confirmation_flow(config);
    let is_new_password_flow = is_new_password_flow(config);

    // --- Content Column ---
    let mut content = column![];

    // 1. Title
    content = content.push(text(title).size(20).style(Color::WHITE));

    content = content.push(Space::with_height(Length::Fixed(10.0)));

    // 2. Description
    content = content.push(text(description).size(14).style(Color::from_rgb(0.8, 0.8, 0.8)));

    content = content.push(Space::with_height(Length::Fixed(20.0)));

    // 3. Error Display
    if let Some(error) = &dialog_state.error {
        content = content.push(error_view(error));
        content = content.push(Space::with_height(Length::Fixed(15.0)));
    }

    // 4. Input Fields

    // 4a. Current/Master Password Input (if not just changing password where old is not needed, or as first step)
    // For simple flows, 'input' is the standard password field.
    if !is_new_password_flow {
        content = content.push(text(&input_label).size(12).style(Color::from_rgb(0.7, 0.7, 0.7)));
        content = content.push(Space::with_height(Length::Fixed(5.0)));

        let input = TextInput::new(
            &format!("Enter {}", input_label.to_lowercase()),
            dialog_state.input.expose_secret(),
        )
        .secure(true)
        .padding(10)
        .on_input(|s| Message::PasswordInputChanged(secrecy::SecretString::new(s)))
        .on_submit(Message::SubmitPassword);

        content = content.push(input);
    }
    // 4b. New Password Flow (Change Password, Create Wallet)
    else {
        // If it's pure creation/reset, we utilize 'input' for the new password.
        // If it's change password, we utilize 'input' for OLD password, and new fields for NEW.

        if let PasswordDialogConfig::ChangePassword { .. } = config {
            content = content.push(text("Current Password").size(12).style(Color::from_rgb(0.7, 0.7, 0.7)));
            content = content.push(Space::with_height(Length::Fixed(5.0)));
            content = content.push(
                TextInput::new("Enter current password", dialog_state.input.expose_secret())
                    .secure(true)
                    .padding(10)
                    .on_input(|s| Message::PasswordInputChanged(secrecy::SecretString::new(s))),
            );
            content = content.push(Space::with_height(Length::Fixed(15.0)));
        }

        content = content.push(text("New Password").size(12).style(Color::from_rgb(0.7, 0.7, 0.7)));
        content = content.push(Space::with_height(Length::Fixed(5.0)));
        content = content.push(
            TextInput::new("Enter new password", dialog_state.new_password_input.expose_secret())
                .secure(true)
                .padding(10)
                .on_input(|s| Message::NewPasswordInputChanged(secrecy::SecretString::new(s))),
        );

        content = content.push(Space::with_height(Length::Fixed(15.0)));

        content = content.push(text("Confirm Password").size(12).style(Color::from_rgb(0.7, 0.7, 0.7)));
        content = content.push(Space::with_height(Length::Fixed(5.0)));
        content = content.push(
            TextInput::new(
                "Confirm new password",
                dialog_state.confirm_password_input.expose_secret(),
            )
            .secure(true)
            .padding(10)
            .on_input(|s| Message::ConfirmPasswordInputChanged(secrecy::SecretString::new(s)))
            .on_submit(Message::SubmitPassword),
        );

        // TODO: Password strength meter could go here
    }

    content = content.push(Space::with_height(Length::Fixed(15.0)));

    // 5. Remember Session Checkbox
    if show_remember {
        content = content.push(
            checkbox("Remember for this session", dialog_state.remember_session)
                .on_toggle(Message::PasswordRememberChanged)
                .size(16)
                .text_size(13),
        );
        content = content.push(Space::with_height(Length::Fixed(20.0)));
    } else {
        content = content.push(Space::with_height(Length::Fixed(10.0)));
    }

    // 6. Buttons
    let mut button_row = row![].spacing(10);

    if show_cancel {
        button_row = button_row.push(
            button(text("Cancel").size(14))
                .style(secondary_button())
                .padding([10, 20])
                .on_press(Message::HidePasswordDialog)
                .width(Length::Fill),
        );
    }

    button_row = button_row.push(
        button(text(button_text).size(14))
            .style(primary_button())
            .padding([10, 20])
            .on_press(Message::SubmitPassword)
            .width(Length::Fill),
    );

    content = content.push(button_row);

    // 7. Reset Option (only for unlock flows)
    if matches!(
        config,
        PasswordDialogConfig::WalletUnlock | PasswordDialogConfig::AccountUnlock { .. }
    ) {
        content = content.push(Space::with_height(Length::Fixed(15.0)));
        content = content.push(
            button(text("Forgot Password? Reset Wallet").size(12))
                .style(text_button_style()) // Need a subtle style
                .on_press(Message::ShowResetWalletConfirmation),
        );
    }

    // --- Container & Modal Wrapper ---
    let dialog_container = container(content)
        .padding(30)
        .width(Length::Fixed(450.0))
        .style(dark_flat_container());

    container(dialog_container)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.8))),
            ..Default::default()
        })
        .into()
}

/// Helper to determine dialog text content
fn get_dialog_content(config: &PasswordDialogConfig) -> (String, String, String, String) {
    match config {
        PasswordDialogConfig::WalletUnlock => (
            "Unlock Wallet".into(),
            "Enter your master password to unlock the session.".into(),
            "Master Password".into(),
            "Unlock".into(),
        ),
        PasswordDialogConfig::AccountUnlock { account_name, .. } => (
            "Unlock Account".into(),
            format!("Enter password for account '{}' to unlock it.", account_name),
            "Account Password".into(),
            "Unlock".into(),
        ),
        PasswordDialogConfig::SignTransaction { tx_details } => (
            "Sign Transaction".into(),
            format!("Enter password to confirm transaction:\n{}", tx_details),
            "Password".into(),
            "Sign".into(),
        ),
        PasswordDialogConfig::WalletSetup { wallet_name } => (
            "Create Wallet".into(),
            format!("Set a secure master password for '{}'.", wallet_name),
            "Master Password".into(),
            "Create".into(),
        ),
        PasswordDialogConfig::ChangePassword { is_wallet_password } => (
            if *is_wallet_password {
                "Change Master Password"
            } else {
                "Change Account Password"
            }
            .into(),
            "Enter current password and choose a new one.".into(),
            "Current Password".into(), // Label for first field
            "Change".into(),
        ),
        PasswordDialogConfig::ExportPrivateKey { account_name } => (
            "Export Private Key".into(),
            format!("Enter password to reveal private key for '{}'.", account_name),
            "Password".into(),
            "Reveal".into(),
        ),
        PasswordDialogConfig::ExportSeedPhrase { account_name } => (
            "Export Seed Phrase".into(),
            format!("Enter password to reveal seed phrase for '{}'.", account_name),
            "Password".into(),
            "Reveal".into(),
        ),
        PasswordDialogConfig::WalletExport => (
            "Export Wallet Data".into(),
            "Enter master password to authorize export.".into(),
            "Master Password".into(),
            "Export".into(),
        ),
        PasswordDialogConfig::ImportWallet => (
            "Import Wallet".into(),
            "Enter the password for the wallet backup you are importing.".into(),
            "Backup Password".into(),
            "Import".into(),
        ),
        PasswordDialogConfig::DeleteAccount { account_name } => (
            "Delete Account".into(),
            format!("Enter password to confirm DELETION of '{}'.", account_name),
            "Password".into(),
            "Delete".into(),
        ),
        PasswordDialogConfig::ResetWallet => (
            "Factory Reset".into(),
            "Enter master password to confirm complete wallet reset. ALL DATA WILL BE LOST.".into(),
            "Master Password".into(),
            "Reset".into(),
        ),
        PasswordDialogConfig::AddAccount { creation_type } => {
            let desc = match creation_type {
                AccountCreationType::GenerateNew => "create a new account",
                AccountCreationType::ImportFromSeed => "import from seed",
                AccountCreationType::ImportFromPrivateKey => "import private key",
                AccountCreationType::HardwareWallet => "connect hardware wallet",
                AccountCreationType::DeriveFromExisting { .. } => "derive new account",
            };
            (
                "Add Account".into(),
                format!("Enter master password to {}.", desc),
                "Master Password".into(),
                "Add".into(),
            )
        }
        PasswordDialogConfig::ConfirmOperation { operation } => {
            let desc = match operation {
                WalletOperation::BackupWallet => "backup wallet",
                WalletOperation::RestoreWallet => "restore wallet",
                WalletOperation::ChangeSecuritySettings => "change security settings",
                WalletOperation::ViewAllSeeds => "view all seeds",
                WalletOperation::FactoryReset => "factory reset",
                WalletOperation::NetworkSecurityChanges => "change network settings",
            };
            (
                "Confirm Operation".into(),
                format!("Enter password to confirm {}.", desc),
                "Password".into(),
                "Confirm".into(),
            )
        }
    }
}

fn should_show_cancel(config: &PasswordDialogConfig) -> bool {
    // Force some flows to be non-cancellable if desired (e.g., initial setup modal that covers screen)
    match config {
        PasswordDialogConfig::WalletSetup { .. } => false,
        _ => true,
    }
}

fn should_show_remember(config: &PasswordDialogConfig) -> bool {
    match config {
        PasswordDialogConfig::WalletUnlock | PasswordDialogConfig::AccountUnlock { .. } => true,
        _ => false,
    }
}

fn is_new_password_flow(config: &PasswordDialogConfig) -> bool {
    match config {
        PasswordDialogConfig::WalletSetup { .. } | PasswordDialogConfig::ChangePassword { .. } => true,
        _ => false,
    }
}

fn is_confirmation_flow(_config: &PasswordDialogConfig) -> bool {
    // Logic to distinguish flows where we just need a simple check
    // Currently used for styling tweaks if needed
    false
}

fn error_view<'a>(error: &PasswordError) -> Element<'a, Message> {
    container(text(error.to_string()).size(12).style(Color::from_rgb(1.0, 0.4, 0.4)))
        .padding(10)
        .style(|_theme: &iced::Theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.4, 0.4, 0.1))),
            border: iced::Border {
                color: Color::from_rgb(1.0, 0.4, 0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}

fn text_button_style() -> iced::theme::Button {
    iced::theme::Button::Text
}
