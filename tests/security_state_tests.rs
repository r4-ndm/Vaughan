//! Tests for security state module

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use vaughan::gui::state::auth_state::*;

    #[test]
    fn test_security_state_default() {
        let state = AuthState::default();

        assert!(!state.password_dialog.visible);
        assert!(state.password_dialog.config.is_none());
        assert!(state.password_dialog.error.is_none());
        assert_eq!(state.password_dialog.attempts, 0);
        assert!(state.password_dialog.remember_session);

        assert!(!state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_none());
        assert!(state.session.auto_lock_enabled);
        assert!(!state.session.lock_on_minimize);
    }

    #[test]
    fn test_password_dialog_show_hide() {
        let mut state = AuthState::default();

        // Show dialog
        state.password_dialog.show(PasswordDialogConfig::WalletUnlock);
        assert!(state.password_dialog.visible);
        assert_eq!(state.password_dialog.config, Some(PasswordDialogConfig::WalletUnlock));

        // Hide dialog
        state.password_dialog.hide();
        assert!(!state.password_dialog.visible);
        assert!(state.password_dialog.config.is_none());
    }

    #[test]
    fn test_password_dialog_error_handling() {
        let mut state = AuthState::default();

        // Set error
        state
            .password_dialog
            .set_error(PasswordError::IncorrectPassword { attempts_remaining: 3 });
        assert!(state.password_dialog.error.is_some());
        assert_eq!(state.password_dialog.attempts, 1);

        // Clear error
        state.password_dialog.clear_error();
        assert!(state.password_dialog.error.is_none());
    }

    #[test]
    fn test_session_lock_unlock() {
        let mut state = AuthState::default();

        // Unlock session
        state.session.unlock();
        assert!(state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_some());

        // Lock session
        state.session.lock();
        assert!(!state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_none());
    }

    #[test]
    fn test_session_timeout() {
        let mut state = AuthState::default();

        // Set short timeout for testing
        state.session.timeout_duration = Duration::from_millis(100);
        state.session.unlock();

        // Should not be timed out immediately
        assert!(!state.session.is_timed_out());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should be timed out now
        assert!(state.session.is_timed_out());
    }

    #[test]
    fn test_session_activity_update() {
        let mut state = AuthState::default();

        state.session.unlock();
        let initial_activity = state.session.last_activity;

        // Wait a bit
        std::thread::sleep(Duration::from_millis(10));

        // Update activity
        state.session.update_activity();
        let updated_activity = state.session.last_activity;

        // Activity should be updated
        assert!(updated_activity > initial_activity);
    }

    #[test]
    fn test_session_time_until_timeout() {
        let mut state = AuthState::default();

        // Not unlocked - should return None
        assert!(state.session.time_until_timeout().is_none());

        // Unlock with short timeout
        state.session.timeout_duration = Duration::from_secs(10);
        state.session.unlock();

        // Should have time remaining
        let remaining = state.session.time_until_timeout();
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= Duration::from_secs(10));
    }

    #[test]
    fn test_password_error_display() {
        let error1 = PasswordError::IncorrectPassword { attempts_remaining: 3 };
        assert!(error1.to_string().contains("3 attempts remaining"));

        let error2 = PasswordError::TooManyAttempts {
            retry_after_seconds: 60,
        };
        assert!(error2.to_string().contains("60 seconds"));

        let error3 = PasswordError::AccountLocked {
            retry_after_seconds: 900,
        };
        assert!(error3.to_string().contains("900 seconds"));

        let error4 = PasswordError::EmptyPassword;
        assert!(error4.to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_password_reason_variants() {
        let reason1 = PasswordDialogConfig::SignTransaction {
            tx_details: "Send 1 ETH".to_string(),
        };
        assert!(matches!(reason1, PasswordDialogConfig::SignTransaction { .. }));

        let reason2 = PasswordDialogConfig::ExportPrivateKey {
            account_name: "Account 1".to_string(),
        };
        assert!(matches!(reason2, PasswordDialogConfig::ExportPrivateKey { .. }));

        let reason3 = PasswordDialogConfig::WalletUnlock;
        assert!(matches!(reason3, PasswordDialogConfig::WalletUnlock));
    }

    #[test]
    fn test_password_dialog_reset() {
        let mut state = AuthState::default();

        // Set up dialog with data
        state.password_dialog.show(PasswordDialogConfig::WalletUnlock);
        state
            .password_dialog
            .set_error(PasswordError::IncorrectPassword { attempts_remaining: 2 });

        // Reset
        state.password_dialog.reset();

        // Verify reset
        assert!(!state.password_dialog.visible);
        assert!(state.password_dialog.config.is_none());
        assert!(state.password_dialog.error.is_none());
    }

    #[test]
    fn test_session_auto_lock_disabled() {
        let mut state = AuthState::default();

        // Disable auto-lock
        state.session.auto_lock_enabled = false;
        state.session.unlock();

        // Wait past timeout
        state.session.timeout_duration = Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(50));

        // Should not be timed out when auto-lock is disabled
        assert!(!state.session.is_timed_out());
    }
}
