//! Hardware wallet management modules
//!
//! This module provides comprehensive hardware wallet support including
//! device management for multiple simultaneous connections.

pub mod device_manager;
pub mod manager;

pub use device_manager::*;
pub use manager::*;
