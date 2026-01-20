//! Receive Dialog Interaction Tests
//!
//! Tests for receive dialog show/hide functionality.

use vaughan::gui::state::AppState;

#[test]
fn test_receive_dialog_show_hide_flow() {
    let mut state = AppState::default();

    // Initially hidden
    assert!(!state.wallet().receive_dialog.visible, "Dialog should start hidden");

    // Show dialog
    state.wallet_mut().receive_dialog.visible = true;
    assert!(
        state.wallet().receive_dialog.visible,
        "Dialog should be visible after showing"
    );

    // Hide dialog
    state.wallet_mut().receive_dialog.visible = false;
    assert!(
        !state.wallet().receive_dialog.visible,
        "Dialog should be hidden after closing"
    );
}

#[test]
fn test_receive_dialog_multiple_toggles() {
    let mut state = AppState::default();

    // Toggle multiple times
    for _ in 0..5 {
        state.wallet_mut().receive_dialog.visible = true;
        assert!(state.wallet().receive_dialog.visible);

        state.wallet_mut().receive_dialog.visible = false;
        assert!(!state.wallet().receive_dialog.visible);
    }
}

#[test]
fn test_receive_dialog_state_persistence() {
    let mut state = AppState::default();

    // Show dialog
    state.wallet_mut().receive_dialog.visible = true;

    // State should persist across reads
    assert!(state.wallet().receive_dialog.visible);
    assert!(state.wallet().receive_dialog.visible);
    assert!(state.wallet().receive_dialog.visible);
}
