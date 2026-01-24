//! EIP-1193 Event System
//!
//! Provides event emitter for EIP-1193 provider events.
//!
//! # Events
//!
//! - `accountsChanged` - Emitted when accounts change
//! - `chainChanged` - Emitted when chain changes  
//! - `connect` - Emitted on connection
//! - `disconnect` - Emitted on disconnection
//!
//! # Task Reference
//!
//! Implements: Task 5.4 (Implement EIP-1193 events)

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// EIP-1193 Provider Event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProviderEvent {
    /// Accounts have changed
    AccountsChanged {
        accounts: Vec<String>,
    },
    /// Chain has changed
    ChainChanged {
        chain_id: String,
    },
    /// Provider connected
    Connect {
        chain_id: String,
    },
    /// Provider disconnected
    Disconnect {
        code: i32,
        message: String,
    },
    /// Message event (for signing requests, etc.)
    Message {
        #[serde(rename = "type")]
        message_type: String,
        data: serde_json::Value,
    },
}

impl ProviderEvent {
    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::AccountsChanged { .. } => "accountsChanged",
            Self::ChainChanged { .. } => "chainChanged",
            Self::Connect { .. } => "connect",
            Self::Disconnect { .. } => "disconnect",
            Self::Message { .. } => "message",
        }
    }
}

/// Event emitter for EIP-1193 events
///
/// Uses tokio::sync::broadcast for multi-consumer event distribution
#[derive(Debug)]
pub struct EventEmitter {
    /// Broadcast sender for events
    sender: broadcast::Sender<ProviderEvent>,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        // Capacity of 100 events in buffer
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    /// Emit an event to all subscribers
    pub async fn emit(&self, event: ProviderEvent) {
        tracing::debug!(
            event_type = event.event_type(),
            "Emitting provider event"
        );
        
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ProviderEvent> {
        self.sender.subscribe()
    }

    /// Get number of current subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventEmitter {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_types() {
        let accounts_changed = ProviderEvent::AccountsChanged { 
            accounts: vec!["0x123".to_string()] 
        };
        assert_eq!(accounts_changed.event_type(), "accountsChanged");

        let chain_changed = ProviderEvent::ChainChanged { 
            chain_id: "0x1".to_string() 
        };
        assert_eq!(chain_changed.event_type(), "chainChanged");

        let connect = ProviderEvent::Connect { 
            chain_id: "0x1".to_string() 
        };
        assert_eq!(connect.event_type(), "connect");

        let disconnect = ProviderEvent::Disconnect { 
            code: 1000, 
            message: "User disconnected".to_string() 
        };
        assert_eq!(disconnect.event_type(), "disconnect");
    }

    #[tokio::test]
    async fn test_event_emitter_creation() {
        let emitter = EventEmitter::new();
        assert_eq!(emitter.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let emitter = EventEmitter::new();
        
        let _receiver = emitter.subscribe();
        assert_eq!(emitter.subscriber_count(), 1);
        
        let _receiver2 = emitter.subscribe();
        assert_eq!(emitter.subscriber_count(), 2);
    }

    #[tokio::test]
    async fn test_event_emission() {
        let emitter = EventEmitter::new();
        let mut receiver = emitter.subscribe();

        emitter.emit(ProviderEvent::ChainChanged { 
            chain_id: "0x89".to_string() 
        }).await;

        let event = receiver.recv().await.unwrap();
        assert_eq!(event.event_type(), "chainChanged");
        
        if let ProviderEvent::ChainChanged { chain_id } = event {
            assert_eq!(chain_id, "0x89");
        } else {
            panic!("Wrong event type");
        }
    }

    #[tokio::test]
    async fn test_multiple_events() {
        let emitter = EventEmitter::new();
        let mut receiver = emitter.subscribe();

        emitter.emit(ProviderEvent::Connect { 
            chain_id: "0x1".to_string() 
        }).await;
        
        emitter.emit(ProviderEvent::AccountsChanged { 
            accounts: vec!["0x1234".to_string()] 
        }).await;

        let event1 = receiver.recv().await.unwrap();
        assert_eq!(event1.event_type(), "connect");

        let event2 = receiver.recv().await.unwrap();
        assert_eq!(event2.event_type(), "accountsChanged");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let emitter = EventEmitter::new();
        let mut receiver1 = emitter.subscribe();
        let mut receiver2 = emitter.subscribe();

        emitter.emit(ProviderEvent::ChainChanged { 
            chain_id: "0x1".to_string() 
        }).await;

        // Both receivers should get the event
        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();
        
        assert_eq!(event1.event_type(), "chainChanged");
        assert_eq!(event2.event_type(), "chainChanged");
    }

    #[test]
    fn test_event_serialization() {
        let event = ProviderEvent::AccountsChanged { 
            accounts: vec!["0x1234".to_string(), "0x5678".to_string()] 
        };
        
        let json = serde_json::to_string(&event).unwrap();
        // serde uses the variant name as the type (PascalCase)
        assert!(json.contains("AccountsChanged"));
        assert!(json.contains("0x1234"));
    }

    #[test]
    fn test_emitter_clone() {
        let emitter1 = EventEmitter::new();
        let emitter2 = emitter1.clone();
        
        let _receiver = emitter1.subscribe();
        // Both emitters share the same channel
        assert_eq!(emitter2.subscriber_count(), 1);
    }
}
