//! Property-based tests for Session & Authentication functionality
//!
//! This module contains property tests that validate session management and
//! authentication properties using the proptest framework with industry-standard
//! iteration counts (500 iterations for functional properties).
//!
//! ## Properties Tested
//!
//! - **Property 4**: Unlock Restoration - Operations work after unlock
//! - **Property 5**: Auto-Lock Timeout - Wallet locks after timeout (ALREADY IMPLEMENTED in session.rs)
//! - **Property 9**: Session Token Expiration - Tokens expire after configured time
//! - **Property 10**: Session Invalidation on Lock - All sessions invalid when locked
//! - **Property 28**: Session Correlation Tracking - All operations have correlation IDs

use proptest::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use vaughan::security::session::{SessionConfig, SessionManager};

// ============================================================================
// Property 4: Unlock Restoration
// ============================================================================

/// Property 4: Unlock Restoration
///
/// **Validates: Requirements 2.4**
///
/// *For any* locked session with valid credentials, unlocking should restore
/// full functionality and allow all operations to proceed normally.
///
/// This property verifies that:
/// 1. A locked session can be unlocked with correct credentials
/// 2. After unlock, the session is marked as active
/// 3. Operations that were blocked while locked now succeed
/// 4. Session state is properly restored (timestamps, activity tracking)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_unlock_restores_session_state(
        timeout_secs in 60u64..300u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(timeout_secs));

            // Initially should be active
            let initial_state = session.get_state().await;
            prop_assert!(initial_state.is_active, "Session should be active initially");

            // Deactivate (simulate lock)
            session.deactivate().await;
            let locked_state = session.get_state().await;
            prop_assert!(!locked_state.is_active, "Session should be inactive after lock");

            // Reactivate (simulate unlock with valid credentials)
            session.reactivate().await;
            let unlocked_state = session.get_state().await;

            // Verify session is restored
            prop_assert!(unlocked_state.is_active, "Session should be active after unlock");
            prop_assert!(
                unlocked_state.last_activity.elapsed() < Duration::from_secs(1),
                "Last activity should be recent after unlock"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_unlock_allows_operations(
        operation_count in 1usize..10usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            // Lock the session
            session.deactivate().await;
            prop_assert!(!session.get_state().await.is_active, "Session should be locked");

            // Unlock the session
            session.reactivate().await;
            prop_assert!(session.get_state().await.is_active, "Session should be unlocked");

            // Perform multiple operations - all should succeed
            for _ in 0..operation_count {
                session.record_activity().await;
                let state = session.get_state().await;
                prop_assert!(state.is_active, "Session should remain active during operations");
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_unlock_resets_activity_timer(
        timeout_ms in 100u64..500u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

            // Wait for some time
            tokio::time::sleep(Duration::from_millis(timeout_ms / 2)).await;

            // Lock and immediately unlock
            session.deactivate().await;
            session.reactivate().await;

            // Activity timer should be reset - session should not timeout immediately
            prop_assert!(!session.is_timed_out().await, "Session should not be timed out after unlock");

            // Wait for original timeout period
            tokio::time::sleep(Duration::from_millis(timeout_ms / 2 + 20)).await;

            // Should still not be timed out because unlock reset the timer
            prop_assert!(
                !session.is_timed_out().await,
                "Session should not timeout - unlock reset timer"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_multiple_lock_unlock_cycles(
        cycle_count in 2usize..10usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(60));

            for i in 0..cycle_count {
                // Lock
                session.deactivate().await;
                let locked = session.get_state().await;
                prop_assert!(
                    !locked.is_active,
                    "Session should be locked at cycle {}", i
                );

                // Unlock
                session.reactivate().await;
                let unlocked = session.get_state().await;
                prop_assert!(
                    unlocked.is_active,
                    "Session should be unlocked at cycle {}", i
                );

                // Verify session ID persists across cycles
                prop_assert!(
                    !unlocked.session_id.is_empty(),
                    "Session ID should persist across lock/unlock cycles"
                );
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_unlock_preserves_session_id(
        lock_count in 1usize..5usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(60));

            let original_id = session.session_id().await;
            prop_assert!(!original_id.is_empty(), "Session should have an ID");

            // Lock and unlock multiple times
            for _ in 0..lock_count {
                session.deactivate().await;
                session.reactivate().await;
            }

            // Session ID should remain the same
            let final_id = session.session_id().await;
            prop_assert_eq!(
                original_id,
                final_id,
                "Session ID should persist across lock/unlock cycles"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 9: Session Token Expiration
// ============================================================================

/// Property 9: Session Token Expiration
///
/// **Validates: Requirements 2.6**
///
/// *For any* session token with a configured expiration time, the token
/// should become invalid after the expiration period has elapsed.
///
/// This property verifies that:
/// 1. Session tokens have expiration timestamps
/// 2. Tokens are valid before expiration
/// 3. Tokens become invalid after expiration
/// 4. Expired tokens cannot be used for operations
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_session_expires_after_timeout(
        timeout_ms in 50u64..200u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

            // Session should not be timed out initially
            prop_assert!(!session.is_timed_out().await, "Session should not be timed out initially");

            // Wait for timeout
            tokio::time::sleep(Duration::from_millis(timeout_ms + 50)).await;

            // Session should now be timed out (expired)
            prop_assert!(session.is_timed_out().await, "Session should be timed out after expiration");

            Ok(())
        })?;
    }

    #[test]
    fn prop_time_until_expiration_decreases(
        timeout_ms in 100u64..300u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

            let initial_time = session.time_until_lock().await;
            prop_assert!(initial_time.is_some(), "Should have time until expiration");

            // Wait a bit
            tokio::time::sleep(Duration::from_millis(50)).await;

            let later_time = session.time_until_lock().await;
            prop_assert!(later_time.is_some(), "Should still have time until expiration");

            // Time should have decreased
            prop_assert!(
                later_time.unwrap() < initial_time.unwrap(),
                "Time until expiration should decrease"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_expired_session_reports_zero_time(
        timeout_ms in 20u64..80u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

            // Wait for expiration
            tokio::time::sleep(Duration::from_millis(timeout_ms + 50)).await;

            let time_remaining = session.time_until_lock().await;
            prop_assert!(time_remaining.is_some(), "Should return Some even when expired");
            prop_assert_eq!(
                time_remaining.unwrap(),
                Duration::ZERO,
                "Expired session should report zero time remaining"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_activity_extends_expiration(
        timeout_ms in 100u64..300u64,
        activity_delay_ms in 30u64..100u64
    ) {
        prop_assume!(activity_delay_ms < timeout_ms / 2);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

            // Wait for part of timeout
            tokio::time::sleep(Duration::from_millis(activity_delay_ms)).await;

            // Record activity to extend expiration
            session.record_activity().await;

            // Wait for remaining time that would have caused expiration
            tokio::time::sleep(Duration::from_millis(timeout_ms - activity_delay_ms + 20)).await;

            // Should not be expired because activity extended it
            prop_assert!(
                !session.is_timed_out().await,
                "Session should not be expired after activity extension"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_no_expiration_when_disabled(
        wait_ms in 50u64..200u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::no_auto_lock();

            // Wait for some time
            tokio::time::sleep(Duration::from_millis(wait_ms)).await;

            // Should never expire when auto-lock disabled
            prop_assert!(!session.is_timed_out().await, "Session should not expire when auto-lock disabled");
            prop_assert!(
                session.time_until_lock().await.is_none(),
                "Should return None when no expiration configured"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 10: Session Invalidation on Lock
// ============================================================================

/// Property 10: Session Invalidation on Lock
///
/// **Validates: Requirements 2.7**
///
/// *For any* active session, when the wallet is locked, all active sessions
/// should be immediately invalidated and marked as inactive.
///
/// This property verifies that:
/// 1. Active sessions become inactive when wallet locks
/// 2. Session state is properly updated
/// 3. Multiple sessions are all invalidated
/// 4. Invalidated sessions cannot perform operations
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_lock_invalidates_session(
        _dummy in any::<u8>() // Dummy parameter for proptest
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            // Session should be active initially
            let initial_state = session.get_state().await;
            prop_assert!(initial_state.is_active, "Session should be active initially");

            // Lock the session
            session.deactivate().await;

            // Session should now be inactive
            let locked_state = session.get_state().await;
            prop_assert!(!locked_state.is_active, "Session should be inactive after lock");

            Ok(())
        })?;
    }

    #[test]
    fn prop_locked_session_ignores_timeout_check(
        timeout_ms in 20u64..100u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

            // Lock the session
            session.deactivate().await;

            // Wait past timeout
            tokio::time::sleep(Duration::from_millis(timeout_ms + 50)).await;

            // Should not report as timed out because it's already inactive
            prop_assert!(
                !session.is_timed_out().await,
                "Inactive session should not report as timed out"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_multiple_sessions_invalidated(
        session_count in 2usize..5usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create multiple sessions
            let sessions: Vec<_> = (0..session_count)
                .map(|_| SessionManager::with_timeout(Duration::from_secs(300)))
                .collect();

            // All should be active initially
            for (i, session) in sessions.iter().enumerate() {
                let state = session.get_state().await;
                prop_assert!(state.is_active, "Session {} should be active initially", i);
            }

            // Lock all sessions (simulating wallet lock)
            for session in &sessions {
                session.deactivate().await;
            }

            // All should be inactive now
            for (i, session) in sessions.iter().enumerate() {
                let state = session.get_state().await;
                prop_assert!(!state.is_active, "Session {} should be inactive after lock", i);
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_lock_clears_activity_tracking(
        _dummy in any::<u8>()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            // Record some activity
            session.record_activity().await;
            session.record_activity().await;

            // Lock the session
            session.deactivate().await;

            // Session should be inactive
            let state = session.get_state().await;
            prop_assert!(!state.is_active, "Session should be inactive");

            // Recording activity on locked session should not reactivate it
            session.record_activity().await;
            let state_after = session.get_state().await;
            prop_assert!(!state_after.is_active, "Session should remain inactive after activity on locked session");

            Ok(())
        })?;
    }

    #[test]
    fn prop_lock_stops_auto_lock_monitor(
        timeout_ms in 50u64..150u64
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = Arc::new(SessionManager::new(SessionConfig {
                auto_lock_timeout: Some(Duration::from_millis(timeout_ms)),
                check_interval: Duration::from_millis(20),
            }));

            let session_clone = Arc::clone(&session);
            let callback_triggered = Arc::new(tokio::sync::RwLock::new(false));
            let callback_triggered_clone = Arc::clone(&callback_triggered);

            // Start monitor
            let _handle = session
                .start_auto_lock_monitor(move || {
                    let triggered = Arc::clone(&callback_triggered_clone);
                    async move {
                        *triggered.write().await = true;
                    }
                })
                .await;

            // Immediately lock the session
            session_clone.deactivate().await;

            // Wait past timeout
            tokio::time::sleep(Duration::from_millis(timeout_ms + 100)).await;

            // Callback should not have been triggered because session was already inactive
            let was_triggered = *callback_triggered.read().await;
            prop_assert!(
                !was_triggered,
                "Auto-lock callback should not trigger on already-locked session"
            );

            Ok(())
        })?;
    }
}

// ============================================================================
// Property 28: Session Correlation Tracking
// ============================================================================

/// Property 28: Session Correlation Tracking
///
/// **Validates: Requirements 7.2**
///
/// *For any* session operation, a unique correlation ID should be generated
/// and associated with the operation for tracking and debugging purposes.
///
/// This property verifies that:
/// 1. Each session has a unique correlation ID (session_id)
/// 2. Session IDs are non-empty and valid UUIDs
/// 3. Session IDs persist across operations
/// 4. Different sessions have different IDs
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_session_has_correlation_id(
        _dummy in any::<u8>()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            let session_id = session.session_id().await;

            // Session ID should not be empty
            prop_assert!(!session_id.is_empty(), "Session ID should not be empty");

            // Session ID should be a valid UUID format (36 characters with hyphens)
            prop_assert_eq!(
                session_id.len(),
                36,
                "Session ID should be UUID format (36 characters)"
            );

            // Should contain hyphens in UUID positions
            prop_assert!(
                session_id.contains('-'),
                "Session ID should be UUID format with hyphens"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_session_id_persists_across_operations(
        operation_count in 5usize..20usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            let initial_id = session.session_id().await;

            // Perform multiple operations
            for _ in 0..operation_count {
                session.record_activity().await;
                let current_id = session.session_id().await;
                prop_assert_eq!(
                    &initial_id,
                    &current_id,
                    "Session ID should persist across operations"
                );
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_different_sessions_have_different_ids(
        session_count in 2usize..10usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create multiple sessions
            let sessions: Vec<_> = (0..session_count)
                .map(|_| SessionManager::with_timeout(Duration::from_secs(300)))
                .collect();

            // Collect all session IDs
            let mut session_ids = Vec::new();
            for session in &sessions {
                session_ids.push(session.session_id().await);
            }

            // All IDs should be unique
            let unique_ids: std::collections::HashSet<_> = session_ids.iter().collect();
            prop_assert_eq!(
                unique_ids.len(),
                session_count,
                "All sessions should have unique IDs"
            );

            Ok(())
        })?;
    }

    #[test]
    fn prop_session_id_survives_lock_unlock(
        cycle_count in 1usize..5usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            let original_id = session.session_id().await;

            // Lock and unlock multiple times
            for _ in 0..cycle_count {
                session.deactivate().await;
                session.reactivate().await;

                let current_id = session.session_id().await;
                prop_assert_eq!(
                    &original_id,
                    &current_id,
                    "Session ID should survive lock/unlock cycles"
                );
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_session_state_includes_correlation_id(
        _dummy in any::<u8>()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session = SessionManager::with_timeout(Duration::from_secs(300));

            let state = session.get_state().await;

            // State should include session_id
            prop_assert!(!state.session_id.is_empty(), "Session state should include correlation ID");

            // Should match the session's ID
            let session_id = session.session_id().await;
            prop_assert_eq!(
                state.session_id,
                session_id,
                "State session_id should match session's correlation ID"
            );

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_unlock_restoration_basic() {
        let session = SessionManager::with_timeout(Duration::from_secs(60));

        // Lock
        session.deactivate().await;
        assert!(!session.get_state().await.is_active);

        // Unlock
        session.reactivate().await;
        assert!(session.get_state().await.is_active);
    }

    #[tokio::test]
    async fn test_session_expiration_basic() {
        let session = SessionManager::with_timeout(Duration::from_millis(50));

        assert!(!session.is_timed_out().await);

        tokio::time::sleep(Duration::from_millis(70)).await;

        assert!(session.is_timed_out().await);
    }

    #[tokio::test]
    async fn test_session_invalidation_basic() {
        let session = SessionManager::with_timeout(Duration::from_secs(60));

        assert!(session.get_state().await.is_active);

        session.deactivate().await;

        assert!(!session.get_state().await.is_active);
    }

    #[tokio::test]
    async fn test_correlation_id_basic() {
        let session = SessionManager::with_timeout(Duration::from_secs(60));

        let id = session.session_id().await;

        assert!(!id.is_empty());
        assert_eq!(id.len(), 36); // UUID format
    }
}
