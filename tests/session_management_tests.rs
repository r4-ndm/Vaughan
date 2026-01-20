//! Integration tests for session management
//!
//! These tests verify the session management functionality including
//! timeout detection, activity tracking, and key cache integration.
//!
//! **Feature: password-workflow, Property 8: Session Timeout Triggers Auto-Lock**
//! **Validates: Requirements 3.1, 3.3, 7.2**

use proptest::prelude::*;
use secrecy::SecretString;
use std::time::Duration;
use vaughan::gui::state::auth_state::AuthState;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use vaughan::gui::state::auth_state::AuthState;
    use vaughan::security::key_cache::KeyCache;

    #[test]
    fn test_session_timeout_detection() {
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
    fn test_activity_extends_session() {
        let mut state = AuthState::default();

        // Set short timeout
        state.session.timeout_duration = Duration::from_millis(200);
        state.session.unlock();

        // Wait half the timeout
        std::thread::sleep(Duration::from_millis(100));

        // Update activity
        state.session.update_activity();

        // Wait another half timeout
        std::thread::sleep(Duration::from_millis(100));

        // Should not be timed out because we updated activity
        assert!(!state.session.is_timed_out());
    }

    #[test]
    fn test_auto_lock_disabled() {
        let mut state = AuthState::default();

        // Disable auto-lock
        state.session.auto_lock_enabled = false;
        state.session.timeout_duration = Duration::from_millis(10);
        state.session.unlock();

        // Wait past timeout
        std::thread::sleep(Duration::from_millis(50));

        // Should not be timed out when auto-lock is disabled
        assert!(!state.session.is_timed_out());
    }

    #[test]
    fn test_key_cache_expiration() {
        let mut cache = KeyCache::new(Duration::from_millis(100));

        // Create a test address
        let address = alloy::primitives::Address::from([1u8; 20]);
        let key_bytes = vec![1, 2, 3, 4, 5];

        // Insert key
        cache.insert(address, key_bytes.clone()).unwrap();

        // Key should be available immediately
        assert!(cache.get(&address).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Key should be expired
        assert!(cache.get(&address).is_none());
    }

    #[test]
    fn test_key_cache_clear_on_lock() {
        let mut cache = KeyCache::new(Duration::from_secs(60));

        // Insert multiple keys
        for i in 0..3 {
            let mut addr_bytes = [0u8; 20];
            addr_bytes[0] = i;
            let address = alloy::primitives::Address::from(addr_bytes);
            let key_bytes = vec![i; 32];
            cache.insert(address, key_bytes).unwrap();
        }

        assert_eq!(cache.len(), 3);

        // Simulate session lock - clear cache
        cache.clear();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_session_lock_unlock_cycle() {
        let mut state = AuthState::default();

        // Start locked
        assert!(!state.session.is_unlocked);

        // Unlock
        state.session.unlock();
        assert!(state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_some());

        // Lock
        state.session.lock();
        assert!(!state.session.is_unlocked);
        assert!(state.session.unlocked_at.is_none());
    }

    #[test]
    fn test_time_until_timeout() {
        let mut state = AuthState::default();

        // Not unlocked - should return None
        assert!(state.session.time_until_timeout().is_none());

        // Unlock with timeout
        state.session.timeout_duration = Duration::from_secs(10);
        state.session.unlock();

        // Should have time remaining
        let remaining = state.session.time_until_timeout();
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= Duration::from_secs(10));
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 8: Session Timeout Triggers Auto-Lock
    ///
    /// **Feature: password-workflow, Property 8: Session Timeout Triggers Auto-Lock**
    /// **Validates: Requirements 3.1, 3.3, 7.2**
    ///
    /// For any unlocked session, when the inactivity period exceeds the timeout
    /// duration (15 minutes), the session should automatically lock and clear all
    /// cached keys from memory.
    ///
    /// This property verifies that:
    /// 1. An unlocked session with auto-lock enabled times out after inactivity
    /// 2. The session correctly detects timeout condition
    /// 3. Cached passwords are cleared when session locks
    /// 4. Session state transitions from unlocked to locked
    #[test]
    fn prop_session_timeout_triggers_auto_lock(
        timeout_ms in 50u64..500u64,
        has_cached_password in any::<bool>()
    ) {
        // Create a state state with custom timeout
        let mut state = AuthState::default();
        state.session.timeout_duration = Duration::from_millis(timeout_ms);
        state.session.auto_lock_enabled = true;

        // Unlock the session
        state.session.unlock();
        prop_assert!(state.session.is_unlocked, "Session should be unlocked initially");

        // Optionally add a cached password to verify it gets cleared
        if has_cached_password {
            state.session.cached_password = Some(SecretString::new("test_seed_phrase".to_string()));
            prop_assert!(
                state.session.cached_password.is_some(),
                "Cached password should be set"
            );
        }

        // Verify session is not timed out immediately
        prop_assert!(
            !state.session.is_timed_out(),
            "Session should not be timed out immediately after unlock"
        );

        // Wait for timeout period plus a buffer
        std::thread::sleep(Duration::from_millis(timeout_ms + 100));

        // Verify session has timed out
        prop_assert!(
            state.session.is_timed_out(),
            "Session should be timed out after inactivity period"
        );

        // Simulate the auto-lock action (what handle_session_timeout_check does)
        state.session.lock();

        // Verify session is now locked
        prop_assert!(
            !state.session.is_unlocked,
            "Session should be locked after timeout"
        );

        // Verify unlocked_at is cleared
        prop_assert!(
            state.session.unlocked_at.is_none(),
            "Unlocked timestamp should be cleared"
        );

        // Verify cached password is cleared (Requirement 7.2)
        prop_assert!(
            state.session.cached_password.is_none(),
            "Cached password should be cleared when session locks"
        );
    }

    /// Property: Activity Extends Session Timeout
    ///
    /// For any unlocked session, performing a wallet action should reset the
    /// inactivity timer, extending the session timeout period.
    ///
    /// This property verifies that:
    /// 1. Activity updates reset the timeout timer
    /// 2. Session does not timeout if activity occurs within timeout period
    #[test]
    fn prop_activity_extends_session(
        timeout_ms in 100u64..500u64,
        activity_delay_ms in 50u64..250u64
    ) {
        // Ensure activity happens before timeout
        prop_assume!(activity_delay_ms < timeout_ms);

        let mut state = AuthState::default();
        state.session.timeout_duration = Duration::from_millis(timeout_ms);
        state.session.auto_lock_enabled = true;
        state.session.unlock();

        // Wait for part of the timeout period
        std::thread::sleep(Duration::from_millis(activity_delay_ms));

        // Simulate user activity
        state.session.update_activity();

        // Wait for the remaining time (which would have caused timeout without activity)
        let remaining = timeout_ms - activity_delay_ms + 50;
        std::thread::sleep(Duration::from_millis(remaining));

        // Session should NOT be timed out because activity extended it
        prop_assert!(
            !state.session.is_timed_out(),
            "Session should not timeout after activity update"
        );
    }

    /// Property: Auto-Lock Disabled Prevents Timeout
    ///
    /// For any session with auto-lock disabled, the session should never
    /// timeout regardless of inactivity period.
    #[test]
    fn prop_auto_lock_disabled_prevents_timeout(
        timeout_ms in 10u64..100u64,
        wait_multiplier in 2u64..5u64
    ) {
        let mut state = AuthState::default();
        state.session.timeout_duration = Duration::from_millis(timeout_ms);
        state.session.auto_lock_enabled = false; // Disable auto-lock
        state.session.unlock();

        // Wait for much longer than timeout period
        std::thread::sleep(Duration::from_millis(timeout_ms * wait_multiplier));

        // Session should NOT be timed out when auto-lock is disabled
        prop_assert!(
            !state.session.is_timed_out(),
            "Session should not timeout when auto-lock is disabled"
        );

        // Session should still be unlocked
        prop_assert!(
            state.session.is_unlocked,
            "Session should remain unlocked when auto-lock is disabled"
        );
    }

    /// Property: Manual Lock Clears Cached Keys Immediately
    ///
    /// For any unlocked session with cached keys, manually locking the session
    /// should immediately clear all cached keys from memory.
    ///
    /// **Validates: Requirements 3.5, 7.2**
    #[test]
    fn prop_manual_lock_clears_keys(
        has_cached_password in any::<bool>()
    ) {
        let mut state = AuthState::default();
        state.session.unlock();

        // Set cached password if specified
        if has_cached_password {
            state.session.cached_password = Some(SecretString::new("test_seed_phrase".to_string()));
            prop_assert!(
                state.session.cached_password.is_some(),
                "Cached password should be set before lock"
            );
        }

        // Manually lock the session
        state.session.lock();

        // Verify session is locked
        prop_assert!(
            !state.session.is_unlocked,
            "Session should be locked after manual lock"
        );

        // Verify cached password is cleared (Requirement 7.2)
        prop_assert!(
            state.session.cached_password.is_none(),
            "Cached password should be cleared immediately on manual lock"
        );

        // Verify unlocked_at is cleared
        prop_assert!(
            state.session.unlocked_at.is_none(),
            "Unlocked timestamp should be cleared on manual lock"
        );
    }
}
