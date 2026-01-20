use vaughan::gui::state::auth_state::{AuthState, PasswordDialogConfig};

#[test]
fn test_auth_state_initialization() {
    let state = AuthState::default();
    assert!(matches!(state.session.is_unlocked, false));
    assert!(!state.session.is_unlocked);
}

#[test]
fn test_auth_state_locking_unlocking() {
    let mut state = AuthState::default();

    // Unlock
    state.unlock();
    assert!(state.session.is_unlocked);

    // Lock
    state.lock();
    assert!(!state.session.is_unlocked);
}

#[test]
fn test_auth_state_timeout() {
    let mut state = AuthState::default();
    state.unlock();

    // Verify default timeout is 15 minutes (900 seconds)
    assert_eq!(state.session.timeout_duration.as_secs(), 900);
}

#[test]
fn test_password_dialog_config_consolidation() {
    // Verify all config variants can be instantiated and handled
    let configs = vec![
        PasswordDialogConfig::WalletUnlock,
        PasswordDialogConfig::AccountUnlock {
            account_id: "id".into(),
            account_name: "name".into(),
        },
        PasswordDialogConfig::WalletExport,
        PasswordDialogConfig::SignTransaction {
            tx_details: "tx".into(),
        },
        PasswordDialogConfig::WalletSetup {
            wallet_name: "w".into(),
        },
        PasswordDialogConfig::ChangePassword {
            is_wallet_password: true,
        },
        PasswordDialogConfig::ExportPrivateKey {
            account_name: "acc".into(),
        },
        PasswordDialogConfig::ExportSeedPhrase {
            account_name: "acc".into(),
        },
        PasswordDialogConfig::ImportWallet,
        PasswordDialogConfig::DeleteAccount {
            account_name: "acc".into(),
        },
        PasswordDialogConfig::ResetWallet,
        PasswordDialogConfig::ConfirmOperation {
            operation: vaughan::gui::state::auth_state::WalletOperation::BackupWallet,
        },
    ];

    for config in configs {
        let mut state = AuthState::default();
        state.password_dialog.show(config);
        assert!(state.password_dialog.visible);
    }
}
