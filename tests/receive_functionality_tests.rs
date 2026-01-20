//! Receive Functionality Tests
//!
//! Tests for the receive dialog and QR code generation functionality.

use vaughan::gui::services::qr_service;
use vaughan::gui::state::{wallet_state::ReceiveDialogState, AppState};

#[test]
fn test_receive_dialog_state_default() {
    let state = ReceiveDialogState::default();
    assert!(!state.visible, "Receive dialog should be hidden by default");
}

#[test]
fn test_receive_dialog_in_app_state() {
    let state = AppState::default();
    assert!(
        !state.wallet().receive_dialog.visible,
        "Receive dialog should be hidden in default app state"
    );
}

#[test]
fn test_qr_code_generation_valid_address() {
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let result = qr_service::generate_address_qr_code(address);
    assert!(result.is_ok(), "QR code generation should succeed for valid address");
}

#[test]
fn test_qr_code_generation_empty_string() {
    let result = qr_service::generate_address_qr_code("");
    // Empty string should still generate a QR code (qrcode crate allows it)
    assert!(result.is_ok(), "QR code generation should handle empty string");
}

#[test]
fn test_qr_code_generation_long_address() {
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let result = qr_service::generate_address_qr_code(address);
    assert!(result.is_ok(), "QR code generation should handle long addresses");
}

#[test]
fn test_payment_request_qr_basic() {
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let result = qr_service::generate_payment_request_qr_code(address, None, None);
    assert!(result.is_ok(), "Payment request QR should generate for basic address");
}

#[test]
fn test_payment_request_qr_with_chain_id() {
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let result = qr_service::generate_payment_request_qr_code(address, Some(1), None);
    assert!(result.is_ok(), "Payment request QR should generate with chain ID");
}

#[test]
fn test_payment_request_qr_with_amount() {
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let result = qr_service::generate_payment_request_qr_code(address, None, Some("1.5"));
    assert!(result.is_ok(), "Payment request QR should generate with amount");
}

#[test]
fn test_payment_request_qr_full() {
    let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let result = qr_service::generate_payment_request_qr_code(address, Some(369), Some("10.0"));
    assert!(result.is_ok(), "Payment request QR should generate with all parameters");
}

#[test]
fn test_receive_dialog_visibility_toggle() {
    let mut state = ReceiveDialogState::default();
    assert!(!state.visible);

    state.visible = true;
    assert!(state.visible);

    state.visible = false;
    assert!(!state.visible);
}

#[test]
fn test_qr_code_special_characters() {
    // Test with EIP-681 format
    let uri = "ethereum:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb@369?value=1.5";
    let result = qr_service::generate_address_qr_code(uri);
    assert!(
        result.is_ok(),
        "QR code should handle EIP-681 format with special characters"
    );
}
