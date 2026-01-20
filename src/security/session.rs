//! Session Management with Auto-Lock
//!
//! This module provides session management functionality including automatic
//! wallet locking after a period of inactivity.
//!
//! # Design
//!
//! The `SessionManager` tracks user activity and automatically triggers a lock
//! callback when the configured timeout expires without any recorded activity.
//!
//! # Requirements Addressed
//!
//! - **Requirement 2.5**: Auto-lock with configurable timeout periods
//!
//! # Usage
//!
//! ```rust,ignore
//! let session = SessionManager::new(Duration::from_secs(300)); // 5-minute timeout
//! session.record_activity().await; // Update last activity timestamp
//!
//! // Start auto-lock monitoring
//! session.start_auto_lock_monitor(move || async {
//!     wallet.lock().await.ok();
//! }).await;
//! ```

use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use uuid::Uuid;

/// Session state for tracking activity
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Session start time
    pub session_start: DateTime<Utc>,
    /// Unique session ID for correlation
    pub session_id: String,
    /// Whether the session is active
    pub is_active: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            last_activity: Instant::now(),
            session_start: Utc::now(),
            session_id: Uuid::new_v4().to_string(),
            is_active: true,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Auto-lock timeout duration (None = disabled)
    pub auto_lock_timeout: Option<Duration>,
    /// Interval for checking auto-lock condition
    pub check_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_lock_timeout: Some(Duration::from_secs(300)), // 5 minutes default
            check_interval: Duration::from_secs(10),           // Check every 10 seconds
        }
    }
}

impl SessionConfig {
    /// Create a new session config with specified timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            auto_lock_timeout: Some(timeout),
            check_interval: Duration::from_secs(10),
        }
    }

    /// Create a session config with no auto-lock (for testing)
    pub fn no_auto_lock() -> Self {
        Self {
            auto_lock_timeout: None,
            check_interval: Duration::from_secs(60),
        }
    }
}

/// Session manager for tracking activity and auto-locking
///
/// The SessionManager is responsible for:
/// - Tracking last activity timestamp
/// - Checking for timeout expiration
/// - Triggering auto-lock when timeout is reached
///
/// # Thread Safety
///
/// All state is protected by `Arc<RwLock<>>` for safe concurrent access.
#[derive(Debug)]
pub struct SessionManager {
    /// Session state
    state: Arc<RwLock<SessionState>>,
    /// Configuration
    config: SessionConfig,
    /// Whether the auto-lock monitor is running
    monitor_running: Arc<RwLock<bool>>,
}

impl SessionManager {
    /// Create a new SessionManager with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        let session_id = Uuid::new_v4().to_string();
        tracing::info!(
            session_id = %session_id,
            timeout_secs = ?config.auto_lock_timeout.map(|d| d.as_secs()),
            "🔐 Creating new session manager"
        );

        Self {
            state: Arc::new(RwLock::new(SessionState {
                session_id,
                ..Default::default()
            })),
            config,
            monitor_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new SessionManager with a specific timeout duration
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::new(SessionConfig::with_timeout(timeout))
    }

    /// Create a SessionManager with no auto-lock (for testing)
    pub fn no_auto_lock() -> Self {
        Self::new(SessionConfig::no_auto_lock())
    }

    /// Record user activity - resets the inactivity timer
    ///
    /// Call this whenever the user performs an action that should
    /// reset the auto-lock timer.
    pub async fn record_activity(&self) {
        let mut state = self.state.write().await;
        state.last_activity = Instant::now();
        tracing::trace!(
            session_id = %state.session_id,
            "📝 Activity recorded"
        );
    }

    /// Get the current session state
    pub async fn get_state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// Get the session ID
    pub async fn session_id(&self) -> String {
        self.state.read().await.session_id.clone()
    }

    /// Check if the session has timed out
    pub async fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.config.auto_lock_timeout {
            let state = self.state.read().await;
            if !state.is_active {
                return false; // Already inactive
            }
            state.last_activity.elapsed() >= timeout
        } else {
            false // No timeout configured
        }
    }

    /// Get time remaining until auto-lock (None if no timeout configured)
    pub async fn time_until_lock(&self) -> Option<Duration> {
        if let Some(timeout) = self.config.auto_lock_timeout {
            let state = self.state.read().await;
            let elapsed = state.last_activity.elapsed();
            if elapsed >= timeout {
                Some(Duration::ZERO)
            } else {
                Some(timeout - elapsed)
            }
        } else {
            None
        }
    }

    /// Deactivate the session (called when wallet is locked)
    pub async fn deactivate(&self) {
        let mut state = self.state.write().await;
        state.is_active = false;
        tracing::info!(
            session_id = %state.session_id,
            "🔒 Session deactivated"
        );
    }

    /// Reactivate the session (called when wallet is unlocked)
    pub async fn reactivate(&self) {
        let mut state = self.state.write().await;
        state.is_active = true;
        state.last_activity = Instant::now();
        tracing::info!(
            session_id = %state.session_id,
            "🔓 Session reactivated"
        );
    }

    /// Start the auto-lock monitor background task
    ///
    /// This spawns a background task that periodically checks if the
    /// session has timed out and calls the provided callback if so.
    ///
    /// # Arguments
    ///
    /// * `on_timeout` - Async callback to execute when timeout occurs
    ///
    /// # Returns
    ///
    /// A handle to the spawned task
    pub async fn start_auto_lock_monitor<F, Fut>(&self, on_timeout: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        // Check if already running
        {
            let mut running = self.monitor_running.write().await;
            if *running {
                tracing::warn!("Auto-lock monitor already running");
                // Return a dummy handle that completes immediately
                return tokio::spawn(async {});
            }
            *running = true;
        }

        let state = Arc::clone(&self.state);
        let monitor_running = Arc::clone(&self.monitor_running);
        let check_interval = self.config.check_interval;
        let timeout = self.config.auto_lock_timeout;

        let session_id = self.state.read().await.session_id.clone();
        tracing::info!(
            session_id = %session_id,
            check_interval_secs = check_interval.as_secs(),
            timeout_secs = ?timeout.map(|d| d.as_secs()),
            "🕐 Starting auto-lock monitor"
        );

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(check_interval).await;

                // Check if we should stop
                if !*monitor_running.read().await {
                    tracing::info!(session_id = %session_id, "Auto-lock monitor stopped");
                    break;
                }

                // Check timeout if configured
                if let Some(timeout_duration) = timeout {
                    let state_guard = state.read().await;

                    // Only check if session is active
                    if state_guard.is_active {
                        let elapsed = state_guard.last_activity.elapsed();
                        if elapsed >= timeout_duration {
                            drop(state_guard);

                            // Mark session as inactive before calling callback
                            {
                                let mut state_write = state.write().await;
                                state_write.is_active = false;
                            }

                            tracing::info!(
                                session_id = %session_id,
                                elapsed_secs = elapsed.as_secs(),
                                "⏰ Auto-lock timeout reached, triggering lock"
                            );

                            // Call the timeout callback
                            on_timeout().await;
                        }
                    }
                }
            }
        })
    }

    /// Stop the auto-lock monitor
    pub async fn stop_monitor(&self) {
        let mut running = self.monitor_running.write().await;
        *running = false;
        tracing::info!("🛑 Auto-lock monitor stop requested");
    }

    /// Check if the monitor is running
    pub async fn is_monitor_running(&self) -> bool {
        *self.monitor_running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn test_session_manager_creation() {
        let session = SessionManager::with_timeout(Duration::from_secs(60));
        let state = session.get_state().await;

        assert!(state.is_active);
        assert!(!state.session_id.is_empty());
    }

    #[tokio::test]
    async fn test_activity_recording() {
        let session = SessionManager::with_timeout(Duration::from_secs(60));

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = session.get_state().await.last_activity;

        // Record activity
        session.record_activity().await;

        let after = session.get_state().await.last_activity;

        // After should be newer
        assert!(after > before);
    }

    #[tokio::test]
    async fn test_timeout_detection() {
        // Very short timeout for testing
        let session = SessionManager::with_timeout(Duration::from_millis(50));

        // Should not be timed out initially
        assert!(!session.is_timed_out().await);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should now be timed out
        assert!(session.is_timed_out().await);
    }

    #[tokio::test]
    async fn test_activity_resets_timeout() {
        let session = SessionManager::with_timeout(Duration::from_millis(50));

        // Wait almost until timeout
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert!(!session.is_timed_out().await);

        // Record activity to reset timer
        session.record_activity().await;

        // Should not be timed out
        assert!(!session.is_timed_out().await);

        // Wait again
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert!(!session.is_timed_out().await);

        // Wait until actual timeout
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(session.is_timed_out().await);
    }

    #[tokio::test]
    async fn test_deactivate_reactivate() {
        let session = SessionManager::with_timeout(Duration::from_secs(60));

        assert!(session.get_state().await.is_active);

        session.deactivate().await;
        assert!(!session.get_state().await.is_active);

        session.reactivate().await;
        assert!(session.get_state().await.is_active);
    }

    #[tokio::test]
    async fn test_no_timeout_config() {
        let session = SessionManager::no_auto_lock();

        // Should never time out
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!session.is_timed_out().await);
        assert!(session.time_until_lock().await.is_none());
    }

    #[tokio::test]
    async fn test_time_until_lock() {
        let session = SessionManager::with_timeout(Duration::from_secs(10));

        let time_until = session.time_until_lock().await;
        assert!(time_until.is_some());
        let duration = time_until.unwrap();
        assert!(duration.as_secs() <= 10);
        assert!(duration.as_secs() >= 9);
    }

    #[tokio::test]
    async fn test_auto_lock_monitor() {
        let session = SessionManager::new(SessionConfig {
            auto_lock_timeout: Some(Duration::from_millis(50)),
            check_interval: Duration::from_millis(20),
        });

        let lock_triggered = Arc::new(AtomicBool::new(false));
        let lock_triggered_clone = Arc::clone(&lock_triggered);

        let _handle = session
            .start_auto_lock_monitor(move || {
                let triggered = Arc::clone(&lock_triggered_clone);
                async move {
                    triggered.store(true, Ordering::SeqCst);
                }
            })
            .await;

        // Wait for timeout to occur
        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(lock_triggered.load(Ordering::SeqCst));
        assert!(!session.get_state().await.is_active);
    }

    #[tokio::test]
    async fn test_stop_monitor() {
        let session = SessionManager::new(SessionConfig {
            auto_lock_timeout: Some(Duration::from_secs(1)),
            check_interval: Duration::from_millis(50),
        });

        let _handle = session
            .start_auto_lock_monitor(|| async {})
            .await;

        assert!(session.is_monitor_running().await);

        session.stop_monitor().await;

        // Give time for the monitor to notice the stop signal
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(!session.is_monitor_running().await);
    }
}

/// Property-based tests for auto-lock timeout
///
/// These tests validate **Property 5: Auto-Lock Timeout** from design.md
/// and **Requirement 2.5** from requirements.md:
///
/// *For any* configured timeout period, if no activity occurs for that duration,
/// the wallet should automatically lock itself.
///
/// Uses proptest with minimum 100 iterations as specified in design.md.
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 5: Auto-Lock Timeout - Timeout Detection
        ///
        /// *For any* timeout duration (10-200ms for testing speed),
        /// the session should report is_timed_out() = true after
        /// the duration has elapsed.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_timeout_detected_after_duration(
            timeout_ms in 10u64..200
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (before, after) = rt.block_on(async {
                let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

                // Should not be timed out immediately
                let before = session.is_timed_out().await;

                // Wait for timeout plus some buffer
                tokio::time::sleep(Duration::from_millis(timeout_ms + 20)).await;

                // Should now be timed out
                let after = session.is_timed_out().await;

                (before, after)
            });

            prop_assert!(!before, "Should not be timed out before duration");
            prop_assert!(after, "Should be timed out after duration");
        }

        /// Property 5: Auto-Lock Timeout - Activity Resets Timer
        ///
        /// *For any* timeout duration, recording activity should reset
        /// the timer and prevent timeout.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_activity_resets_timeout(
            timeout_ms in 80u64..200
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let timed_out = rt.block_on(async {
                let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

                // Wait for 70% of timeout
                let wait_time = (timeout_ms * 70) / 100;
                tokio::time::sleep(Duration::from_millis(wait_time)).await;

                // Should not be timed out yet
                let mid_check = session.is_timed_out().await;
                if mid_check {
                    return true; // Already timed out, test fails
                }

                // Record activity to reset timer
                session.record_activity().await;

                // Wait for 70% of timeout again
                tokio::time::sleep(Duration::from_millis(wait_time)).await;

                // Should still not be timed out because we reset
                session.is_timed_out().await
            });

            prop_assert!(!timed_out, "Activity should reset timeout timer");
        }

        /// Property 5: Auto-Lock Timeout - Time Until Lock Decreases
        ///
        /// *For any* timeout duration, time_until_lock() should decrease
        /// as time passes.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_time_until_lock_decreases(
            timeout_ms in 50u64..200
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (initial, after) = rt.block_on(async {
                let session = SessionManager::with_timeout(Duration::from_millis(timeout_ms));

                let initial = session.time_until_lock().await.unwrap();

                // Wait a bit
                tokio::time::sleep(Duration::from_millis(20)).await;

                let after = session.time_until_lock().await.unwrap();

                (initial, after)
            });

            prop_assert!(
                after < initial,
                "Time until lock should decrease: initial={:?}, after={:?}",
                initial,
                after
            );
        }

        /// Property 5: Auto-Lock Timeout - Callback Triggered
        ///
        /// *For any* short timeout with monitoring, the callback
        /// should be triggered when timeout occurs.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_callback_triggered_on_timeout(
            timeout_ms in 20u64..80
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let callback_triggered = rt.block_on(async {
                let session = SessionManager::new(SessionConfig {
                    auto_lock_timeout: Some(Duration::from_millis(timeout_ms)),
                    check_interval: Duration::from_millis(10),
                });

                let triggered = Arc::new(AtomicBool::new(false));
                let triggered_clone = Arc::clone(&triggered);

                let _handle = session
                    .start_auto_lock_monitor(move || {
                        let t = Arc::clone(&triggered_clone);
                        async move {
                            t.store(true, Ordering::SeqCst);
                        }
                    })
                    .await;

                // Wait for timeout plus check interval plus buffer
                tokio::time::sleep(Duration::from_millis(timeout_ms + 50)).await;

                triggered.load(Ordering::SeqCst)
            });

            prop_assert!(callback_triggered, "Callback should be triggered on timeout");
        }

        /// Property 5: Auto-Lock Timeout - No Early Trigger
        ///
        /// *For any* timeout duration, the callback should NOT be
        /// triggered before the timeout period.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_no_early_callback_trigger(
            timeout_ms in 100u64..300
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let callback_triggered = rt.block_on(async {
                let session = SessionManager::new(SessionConfig {
                    auto_lock_timeout: Some(Duration::from_millis(timeout_ms)),
                    check_interval: Duration::from_millis(10),
                });

                let triggered = Arc::new(AtomicBool::new(false));
                let triggered_clone = Arc::clone(&triggered);

                let _handle = session
                    .start_auto_lock_monitor(move || {
                        let t = Arc::clone(&triggered_clone);
                        async move {
                            t.store(true, Ordering::SeqCst);
                        }
                    })
                    .await;

                // Wait for only 50% of timeout
                let wait_time = timeout_ms / 2;
                tokio::time::sleep(Duration::from_millis(wait_time)).await;

                // Stop the monitor to prevent later triggers
                session.stop_monitor().await;

                triggered.load(Ordering::SeqCst)
            });

            prop_assert!(!callback_triggered, "Callback should not trigger before timeout");
        }

        /// Property 5: Auto-Lock Timeout - Activity Prevents Callback
        ///
        /// *For any* timeout, continuous activity should prevent
        /// the callback from being triggered.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_activity_prevents_callback(
            timeout_ms in 50u64..150
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let callback_triggered = rt.block_on(async {
                let session = Arc::new(SessionManager::new(SessionConfig {
                    auto_lock_timeout: Some(Duration::from_millis(timeout_ms)),
                    check_interval: Duration::from_millis(10),
                }));

                let triggered = Arc::new(AtomicBool::new(false));
                let triggered_clone = Arc::clone(&triggered);

                let _handle = session
                    .start_auto_lock_monitor(move || {
                        let t = Arc::clone(&triggered_clone);
                        async move {
                            t.store(true, Ordering::SeqCst);
                        }
                    })
                    .await;

                // Keep recording activity every 30% of timeout
                let activity_interval = (timeout_ms * 30) / 100;
                for _ in 0..5 {
                    tokio::time::sleep(Duration::from_millis(activity_interval)).await;
                    session.record_activity().await;
                }

                // Stop monitoring
                session.stop_monitor().await;

                triggered.load(Ordering::SeqCst)
            });

            prop_assert!(!callback_triggered, "Continuous activity should prevent callback");
        }

        /// Property 5: Auto-Lock Timeout - Session Deactivated After Timeout
        ///
        /// *For any* timeout, when timeout occurs, the session should
        /// be marked as inactive.
        ///
        /// Validates: Requirement 2.5
        #[test]
        fn prop_session_deactivated_after_timeout(
            timeout_ms in 20u64..80
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (was_active, is_active) = rt.block_on(async {
                let session = SessionManager::new(SessionConfig {
                    auto_lock_timeout: Some(Duration::from_millis(timeout_ms)),
                    check_interval: Duration::from_millis(10),
                });

                let was_active = session.get_state().await.is_active;

                let _handle = session
                    .start_auto_lock_monitor(|| async {})
                    .await;

                // Wait for timeout plus buffer
                tokio::time::sleep(Duration::from_millis(timeout_ms + 50)).await;

                let is_active = session.get_state().await.is_active;

                (was_active, is_active)
            });

            prop_assert!(was_active, "Session should be active initially");
            prop_assert!(!is_active, "Session should be inactive after timeout");
        }
    }
}
