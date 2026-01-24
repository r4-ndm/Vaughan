//! EIP-1193 Provider Interface
//!
//! This module provides EIP-1193 compliant provider interface for dApp integration.
//! Enables Vaughan wallet to interact with web3 applications.
//!
//! # EIP-1193 Standard
//!
//! The Ethereum Provider JavaScript API (EIP-1193) defines a standard interface
//! for Ethereum providers (wallets) to expose to web applications.
//!
//! # Task Reference
//!
//! Implements: Task 5.1 (Create EIP-1193 provider trait)
//! Implements: Task 5.2 (Implement provider for Vaughan wallet)
//!
//! # Inspiration
//!
//! - **MetaMask**: Provider API design and method signatures
//! - **Alloy**: Used for transaction types and signing

pub mod eip1193;
pub mod events;
pub mod permissions;

pub use eip1193::*;
pub use events::*;
pub use permissions::*;
