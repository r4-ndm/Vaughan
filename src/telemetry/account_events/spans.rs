//! Operation Span Management and Correlation Context
//!
//! Implements Property 16: Operation Correlation Logging
//! Implements Property 17: Cross-Component Correlation
//!
//! This module provides span tracking for operations with correlation IDs
//! and cross-component context propagation.

use chrono::{DateTime, Utc};
use tracing::Span;
use uuid::Uuid;

/// Operation span for tracking a single operation with correlation ID
///
/// Implements Property 16: Operation Correlation Logging
#[derive(Debug, Clone)]
pub struct OperationSpan {
    /// Unique correlation ID for this operation
    pub correlation_id: Uuid,
    /// Name of the operation being performed
    pub operation: String,
    /// When the operation started
    pub started_at: DateTime<Utc>,
    /// Parent correlation ID for cross-component tracking
    pub parent_id: Option<Uuid>,
    /// Component where the operation originated
    pub component: Option<String>,
}

impl OperationSpan {
    /// Create a new operation span with auto-generated correlation ID
    ///
    /// Implements Property 16: Creates correlation ID at operation start
    pub fn new(operation: impl Into<String>) -> Self {
        let span = Self {
            correlation_id: Uuid::new_v4(),
            operation: operation.into(),
            started_at: Utc::now(),
            parent_id: None,
            component: None,
        };

        tracing::debug!(
            correlation_id = %span.correlation_id,
            operation = %span.operation,
            "📋 Operation span created"
        );

        span
    }

    /// Create a child span for cross-component tracking
    ///
    /// Implements Property 17: Cross-Component Correlation
    pub fn child(&self, operation: impl Into<String>) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            operation: operation.into(),
            started_at: Utc::now(),
            parent_id: Some(self.correlation_id),
            component: self.component.clone(),
        }
    }

    /// Set the component name for this span
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Get the elapsed time since operation start in milliseconds
    pub fn elapsed_ms(&self) -> i64 {
        (Utc::now() - self.started_at).num_milliseconds()
    }

    /// Create a tracing span for this operation
    pub fn tracing_span(&self) -> Span {
        tracing::info_span!(
            "operation",
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            parent_id = ?self.parent_id,
            component = ?self.component,
        )
    }
}

/// Correlation context for passing across async boundaries
///
/// Implements Property 17: Cross-Component Correlation
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Primary correlation ID
    pub correlation_id: Uuid,
    /// Stack of parent IDs for nested operations
    pub parent_ids: Vec<Uuid>,
    /// Component chain for tracking flow
    pub components: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            parent_ids: Vec::new(),
            components: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Create from an existing operation span
    pub fn from_span(span: &OperationSpan) -> Self {
        let mut ctx = Self::new();
        ctx.correlation_id = span.correlation_id;
        if let Some(parent) = span.parent_id {
            ctx.parent_ids.push(parent);
        }
        if let Some(ref component) = span.component {
            ctx.components.push(component.clone());
        }
        ctx
    }

    /// Add a component to the context
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.components.push(component.into());
        self
    }

    /// Create a child context
    pub fn child(&self) -> Self {
        let mut child = Self::new();
        child.parent_ids = self.parent_ids.clone();
        child.parent_ids.push(self.correlation_id);
        child.components = self.components.clone();
        child
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_span_creation() {
        let span = OperationSpan::new("test_operation");

        assert!(!span.correlation_id.is_nil());
        assert_eq!(span.operation, "test_operation");
        assert!(span.parent_id.is_none());
    }

    #[test]
    fn test_child_span_creation() {
        let parent = OperationSpan::new("parent_op");
        let child = parent.child("child_op");

        assert_eq!(child.parent_id, Some(parent.correlation_id));
        assert_ne!(child.correlation_id, parent.correlation_id);
    }

    #[test]
    fn test_operation_span_elapsed() {
        let span = OperationSpan::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = span.elapsed_ms();
        assert!(elapsed >= 10);
    }

    #[test]
    fn test_correlation_context_child() {
        let parent = CorrelationContext::new();
        let child = parent.child();

        assert!(child.parent_ids.contains(&parent.correlation_id));
        assert_ne!(child.correlation_id, parent.correlation_id);
    }

    #[test]
    fn test_correlation_context_from_span() {
        let span = OperationSpan::new("test").with_component("test_component");
        let ctx = CorrelationContext::from_span(&span);

        assert_eq!(ctx.correlation_id, span.correlation_id);
        assert_eq!(ctx.components.len(), 1);
        assert_eq!(ctx.components[0], "test_component");
    }
}
