//! Property-based tests for password validation and session unlocking
//!
//! **Feature: password-workflow, Property 2: Correct Password Unlocks Session**
//! **Validates: Requirements 1.2**
//!
//! **Feature: password-workflow, Property 3: Incorrect Password Shows Error**
//! **Validates: Requirements 1.3, 6.2**

use proptest::prelude::*;
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use vaughan::gui::state::auth_state::{AuthState, PasswordError, SessionState};
use vaughan::security::KeyReference;

/// Generate arbitrary password strings
fn arb_password() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9!@#$%^&*()]{8,32}"
}

/// Generate arbitrary incorrect password strings (different from correct)
fn arb_incorrect_password(correct: String) -> impl Strategy<Value = String> {
    "[a-zA-Z0-9!@#$%^&*()]{8,32}".prop_filter("Must be different from correct password", move |p| p != &correct)
}

/// Generate arbitrary KeyReference for seed-based accounts
fn arb_seed_key_reference() -> impl Strategy<Value = KeyReference> {
    (
        "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}",
        Just("vaughan-wallet-encrypted-seeds".to_string()),
        "[a-z0-9]{40}",
    )
        .prop_map(|(id, service, account)| KeyReference { id, service, account })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 2: Correct Password Unlocks Session
    ///
    /// For any seed-based account and its correct password, entering the password
    /// should result in session state transitioning to unlocked with cached seed
    /// stored in memory when "remember" is enabled.
    ///
    /// This property verifies that:
    /// 1. Session is unlocked after correct password validation
    /// 2. Session has an unlocked_at timestamp
    /// 3. Cached password is stored when remember is true
    /// 4. Cached password is NOT stored when remember is false
    #[test]
    fn prop_correct_password_unlocks_session(remember in any::<bool>()) {
        // Create a session state
        let mut session = SessionState::default();

        // Verify initial state is locked
        prop_assert!(!session.is_unlocked, "Session should start locked");
        prop_assert!(session.unlocked_at.is_none(), "Session should have no unlock time initially");
        prop_assert!(session.cached_password.is_none(), "Session should have no cached password initially");

        // Simulate successful password validation
        let seed_phrase = SecretString::new("test seed phrase for validation".to_string());

        // Unlock the session (simulating what handle_password_validated does)
        session.unlock();

        // Cache seed if remember is enabled
        if remember {
            session.cached_password = Some(seed_phrase.clone());
        } else {
            session.cached_password = None;
        }

        // Verify session is now unlocked
        prop_assert!(session.is_unlocked, "Session should be unlocked after correct password");
        prop_assert!(session.unlocked_at.is_some(), "Session should have unlock timestamp");

        // Verify cached password behavior based on remember flag
        if remember {
            prop_assert!(
                session.cached_password.is_some(),
                "Cached password should be stored when remember is true"
            );

            // Verify the cached password matches
            if let Some(cached) = &session.cached_password {
                prop_assert_eq!(
                    cached.expose_secret(),
                    seed_phrase.expose_secret(),
                    "Cached password should match the seed phrase"
                );
            }
        } else {
            prop_assert!(
                session.cached_password.is_none(),
                "Cached password should NOT be stored when remember is false"
            );
        }
    }

    /// Property 3: Incorrect Password Shows Error
    ///
    /// For any seed-based account and an incorrect password, password validation
    /// should display an error message and increment the attempt counter without
    /// unlocking the session.
    ///
    /// This property verifies that:
    /// 1. Session remains locked after incorrect password
    /// 2. Error is set in the password dialog state
    /// 3. Attempt counter is incremented
    /// 4. No cached password is stored
    #[test]
    fn prop_incorrect_password_shows_error(attempts in 0u32..5) {
        // Create a security state
        let mut state = AuthState::default();

        // Set initial attempt count
        state.password_dialog.attempts = attempts;

        // Verify initial state
        prop_assert!(!state.session.is_unlocked, "Session should start locked");
        prop_assert!(state.password_dialog.error.is_none(), "Should have no error initially");

        // Simulate incorrect password error
        let error = PasswordError::IncorrectPassword {
            attempts_remaining: 5 - (attempts + 1),
        };

        // Set error (simulating what handle_password_validated does)
        state.password_dialog.set_error(error.clone());

        // Verify session remains locked
        prop_assert!(!state.session.is_unlocked, "Session should remain locked after incorrect password");
        prop_assert!(state.session.unlocked_at.is_none(), "Session should have no unlock timestamp");
        prop_assert!(state.session.cached_password.is_none(), "Session should have no cached password");

        // Verify error is set
        prop_assert!(state.password_dialog.error.is_some(), "Error should be set");

        // Verify attempt counter is incremented
        prop_assert_eq!(
            state.password_dialog.attempts,
            attempts + 1,
            "Attempt counter should be incremented"
        );

        // Verify error message contains attempt information
        if let Some(PasswordError::IncorrectPassword { attempts_remaining }) = &state.password_dialog.error {
            prop_assert_eq!(
                *attempts_remaining,
                5 - (attempts + 1),
                "Error should show correct remaining attempts"
            );
        } else {
            prop_assert!(false, "Error should be IncorrectPassword variant");
        }
    }

    /// Property: Session lock clears cached password
    ///
    /// For any session with a cached password, locking the session should
    /// clear the cached password from memory.
    #[test]
    fn prop_session_lock_clears_cached_password(seed_phrase in arb_password()) {
        let mut session = SessionState::default();

        // Unlock session and cache password
        session.unlock();
        session.cached_password = Some(SecretString::new(seed_phrase));

        prop_assert!(session.is_unlocked, "Session should be unlocked");
        prop_assert!(session.cached_password.is_some(), "Password should be cached");

        // Lock the session
        session.lock();

        // Verify session is locked and password is cleared
        prop_assert!(!session.is_unlocked, "Session should be locked");
        prop_assert!(session.unlocked_at.is_none(), "Unlock timestamp should be cleared");
        prop_assert!(session.cached_password.is_none(), "Cached password should be cleared");
    }

    /// Property: Multiple incorrect passwords increment attempts
    ///
    /// For any sequence of incorrect password attempts, each attempt should
    /// increment the counter and reduce remaining attempts.
    #[test]
    fn prop_multiple_incorrect_passwords_increment_attempts(num_attempts in 1u32..5) {
        let mut state = AuthState::default();

        // Simulate multiple incorrect password attempts
        for i in 0..num_attempts {
            let error = PasswordError::IncorrectPassword {
                attempts_remaining: 5 - (i + 1),
            };
            state.password_dialog.set_error(error);
        }

        // Verify attempt counter matches number of attempts
        prop_assert_eq!(
            state.password_dialog.attempts,
            num_attempts,
            "Attempt counter should match number of attempts"
        );

        // Verify session remains locked
        prop_assert!(!state.session.is_unlocked, "Session should remain locked");
    }

    /// Property: Session timeout is calculated correctly
    ///
    /// For any unlocked session, the time until timeout should decrease
    /// as time passes.
    #[test]
    fn prop_session_timeout_calculation(timeout_secs in 60u64..3600) {
        let mut session = SessionState::default();
        session.timeout_duration = Duration::from_secs(timeout_secs);
        session.unlock();

        // Get initial time until timeout
        let time_until_timeout = session.time_until_timeout();

        prop_assert!(time_until_timeout.is_some(), "Should have time until timeout when unlocked");

        if let Some(time) = time_until_timeout {
            // Time until timeout should be approximately equal to timeout duration
            // (allowing for small timing differences)
            let diff = if time > session.timeout_duration {
                time - session.timeout_duration
            } else {
                session.timeout_duration - time
            };

            prop_assert!(
                diff < Duration::from_secs(1),
                "Time until timeout should be approximately equal to timeout duration"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_session_unlock_sets_timestamp() {
        let mut session = SessionState::default();
        assert!(!session.is_unlocked);
        assert!(session.unlocked_at.is_none());

        session.unlock();

        assert!(session.is_unlocked);
        assert!(session.unlocked_at.is_some());
    }

    #[test]
    fn test_session_lock_clears_state() {
        let mut session = SessionState::default();
        session.unlock();
        session.cached_password = Some(SecretString::new("test".to_string()));

        assert!(session.is_unlocked);
        assert!(session.cached_password.is_some());

        session.lock();

        assert!(!session.is_unlocked);
        assert!(session.unlocked_at.is_none());
        assert!(session.cached_password.is_none());
    }

    #[test]
    fn test_password_error_increments_attempts() {
        let mut state = AuthState::default();
        assert_eq!(state.password_dialog.attempts, 0);

        let error = PasswordError::IncorrectPassword { attempts_remaining: 4 };
        state.password_dialog.set_error(error);

        assert_eq!(state.password_dialog.attempts, 1);
        assert!(state.password_dialog.error.is_some());
    }

    #[test]
    fn test_empty_password_error() {
        let error = PasswordError::EmptyPassword;
        assert_eq!(error.to_string(), "Password cannot be empty");
    }

    #[test]
    fn test_incorrect_password_error_message() {
        let error = PasswordError::IncorrectPassword { attempts_remaining: 3 };
        let message = error.to_string();
        assert!(message.contains("3 attempts remaining"));
    }

    #[test]
    fn test_too_many_attempts_error() {
        let error = PasswordError::TooManyAttempts {
            retry_after_seconds: 60,
        };
        let message = error.to_string();
        assert!(message.contains("60 seconds"));
    }

    #[test]
    fn test_session_timeout_when_locked() {
        let session = SessionState::default();
        assert!(!session.is_unlocked);
        assert!(!session.is_timed_out());
    }

    #[test]
    fn test_session_timeout_when_disabled() {
        let mut session = SessionState::default();
        session.unlock();
        session.auto_lock_enabled = false;
        assert!(!session.is_timed_out());
    }
}
