//! Performance Optimizations for Account Management
//!
//! This module provides performance optimization utilities including:
//! - Batch processing for RPC calls using Alloy
//! - LRU caching for frequently accessed data
//!
//! # Requirements Addressed
//!
//! - **Requirement 6**: Batch processing for blockchain operations
//! - **Requirement 9**: Performance optimization with caching
//!
//! # Design Principles
//!
//! - **Alloy-First**: All blockchain operations use Alloy providers
//! - **Async**: All operations are async to prevent UI blocking
//! - **Correlation Tracking**: All operations include correlation IDs
//! - **Graceful Degradation**: Partial failures are handled gracefully

pub mod batch;
pub mod cache;

pub use batch::*;
pub use cache::*;
