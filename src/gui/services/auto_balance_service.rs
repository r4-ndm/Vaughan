//! Auto Balance Service Messages
//!
//! Messages for auto balance monitoring and audio notifications.
//! The actual monitoring is now handled through the smart polling system.

/// Messages sent by the auto balance service to notify the GUI of changes
#[derive(Debug, Clone)]
pub enum AutoBalanceMessage {
    /// A new incoming transaction was detected
    IncomingTransaction {
        hash: String,
        from: String,
        amount: String,
        token: Option<String>,
    },
    /// Account balance has changed
    BalanceChanged { address: String, new_balance: String },
    /// Service encountered an error
    ServiceError(String),
}
