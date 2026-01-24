//! Transaction Module
//!
//! This module provides transaction-related functionality including:
//! - Transaction simulation (dry-run execution)
//! - Revert reason decoding
//! - Gas estimation
//!
//! # Task Reference
//!
//! Implements: Task 6.1 (Create transaction simulator)
//! Implements: Task 6.2 (Add simulation result types)

pub mod simulator;
pub mod fees;

pub use simulator::*;
pub use fees::*;
