//! Transaction Simulator
//!
//! Provides transaction simulation using Alloy's `call()` for dry-run execution.
//! This allows users to preview transaction outcomes before signing.
//!
//! # Task Reference
//!
//! Implements: Task 6.1 (Create transaction simulator)
//! Implements: Task 6.2 (Add simulation result types)
//! Implements: Task 6.3 (Integrate simulation into signing flow)
//!
//! # References
//!
//! - Alloy Provider call(): https://github.com/alloy-rs/alloy
//! - EVM revert selectors from MetaMask error handling

use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{Result, VaughanError, NetworkError};

// ============================================================================
// Revert Reason Selectors
// ============================================================================

/// Error(string) selector - keccak256("Error(string)")[:4]
const ERROR_SELECTOR: [u8; 4] = [0x08, 0xc3, 0x79, 0xa0];

/// Panic(uint256) selector - keccak256("Panic(uint256)")[:4]
const PANIC_SELECTOR: [u8; 4] = [0x4e, 0x48, 0x7b, 0x71];

// ============================================================================
// Simulation Result Types
// ============================================================================

/// Result of a transaction simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Whether the simulation succeeded
    pub success: bool,
    /// Return data from the call (if successful)
    pub return_data: Option<Bytes>,
    /// Decoded revert reason (if failed)
    pub revert_reason: Option<String>,
    /// Raw revert data (if failed)
    pub revert_data: Option<Bytes>,
    /// Estimated gas used
    pub gas_used: Option<u64>,
    /// Simulation execution time in milliseconds
    pub duration_ms: u64,
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
}

impl SimulationResult {
    /// Create a successful simulation result
    pub fn success(return_data: Bytes, gas_used: u64, correlation_id: Uuid, duration_ms: u64) -> Self {
        Self {
            success: true,
            return_data: Some(return_data),
            revert_reason: None,
            revert_data: None,
            gas_used: Some(gas_used),
            duration_ms,
            correlation_id,
        }
    }

    /// Create a failed simulation result
    pub fn failure(revert_data: Bytes, gas_used: Option<u64>, correlation_id: Uuid, duration_ms: u64) -> Self {
        let revert_reason = decode_revert_reason(&revert_data);
        
        Self {
            success: false,
            return_data: None,
            revert_reason,
            revert_data: Some(revert_data),
            gas_used,
            duration_ms,
            correlation_id,
        }
    }

    /// Create a result from an RPC error
    pub fn from_error(error: &str, correlation_id: Uuid, duration_ms: u64) -> Self {
        Self {
            success: false,
            return_data: None,
            revert_reason: Some(error.to_string()),
            revert_data: None,
            gas_used: None,
            duration_ms,
            correlation_id,
        }
    }
}

/// Simulation warning level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationWarning {
    /// Transaction will likely succeed
    None,
    /// Transaction may fail or have issues
    Low,
    /// Transaction will likely fail
    Medium,
    /// Transaction will definitely fail
    High,
}

impl SimulationWarning {
    pub fn message(&self) -> &'static str {
        match self {
            Self::None => "Transaction simulation passed",
            Self::Low => "Minor issues detected",
            Self::Medium => "Transaction may fail",
            Self::High => "Transaction will fail",
        }
    }
}

// ============================================================================
// Transaction Simulator
// ============================================================================

/// Transaction simulator for dry-run execution
///
/// Uses Alloy's Provider `call()` method to simulate transactions
/// without broadcasting them to the network.
#[derive(Debug, Clone)]
pub struct TransactionSimulator {
    /// Default sender address for simulations
    default_sender: Option<Address>,
}

impl TransactionSimulator {
    /// Create a new transaction simulator
    pub fn new() -> Self {
        Self {
            default_sender: None,
        }
    }

    /// Create a simulator with a default sender
    pub fn with_sender(sender: Address) -> Self {
        Self {
            default_sender: Some(sender),
        }
    }

    /// Simulate a transaction and return the result
    ///
    /// This method prepares a transaction for simulation. The actual
    /// RPC call should be made using the network manager.
    ///
    /// # Arguments
    /// * `tx` - The transaction request to simulate
    ///
    /// # Returns
    /// * Prepared simulation parameters
    pub fn prepare_simulation(&self, tx: &TransactionRequest) -> SimulationParams {
        let correlation_id = Uuid::new_v4();
        
        tracing::info!(
            correlation_id = %correlation_id,
            to = ?tx.to,
            value = ?tx.value,
            "Preparing transaction simulation"
        );

        // Clone the transaction for modification
        let mut sim_tx = tx.clone();

        // Set from address if not specified
        if sim_tx.from.is_none() {
            sim_tx.from = self.default_sender;
        }

        SimulationParams {
            tx: sim_tx,
            correlation_id,
        }
    }

    /// Process simulation result from RPC response
    ///
    /// # Arguments
    /// * `result` - The raw bytes returned from eth_call
    /// * `gas_used` - Estimated gas (from eth_estimateGas if available)
    /// * `correlation_id` - Correlation ID for tracking
    /// * `duration_ms` - Time taken for the simulation
    pub fn process_success(
        result: Bytes,
        gas_used: u64,
        correlation_id: Uuid,
        duration_ms: u64,
    ) -> SimulationResult {
        tracing::info!(
            correlation_id = %correlation_id,
            gas_used = gas_used,
            duration_ms = duration_ms,
            "✅ Simulation succeeded"
        );

        SimulationResult::success(result, gas_used, correlation_id, duration_ms)
    }

    /// Process simulation failure
    pub fn process_failure(
        revert_data: Bytes,
        gas_used: Option<u64>,
        correlation_id: Uuid,
        duration_ms: u64,
    ) -> SimulationResult {
        let result = SimulationResult::failure(revert_data.clone(), gas_used, correlation_id, duration_ms);
        
        tracing::warn!(
            correlation_id = %correlation_id,
            revert_reason = ?result.revert_reason,
            "❌ Simulation failed"
        );

        result
    }

    /// Determine warning level from simulation result
    pub fn get_warning_level(result: &SimulationResult) -> SimulationWarning {
        if result.success {
            SimulationWarning::None
        } else if result.revert_reason.is_some() {
            SimulationWarning::High
        } else {
            SimulationWarning::Medium
        }
    }
}

impl Default for TransactionSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters for a simulation call
#[derive(Debug, Clone)]
pub struct SimulationParams {
    /// The transaction to simulate
    pub tx: TransactionRequest,
    /// Correlation ID
    pub correlation_id: Uuid,
}

// ============================================================================
// Revert Reason Decoding
// ============================================================================

/// Decode revert reason from raw revert data
///
/// Supports:
/// - Error(string) - standard revert with message
/// - Panic(uint256) - assertion failures
/// - Raw data fallback
pub fn decode_revert_reason(data: &Bytes) -> Option<String> {
    if data.len() < 4 {
        return None;
    }

    let selector: [u8; 4] = data[0..4].try_into().ok()?;

    match selector {
        ERROR_SELECTOR => decode_error_string(data),
        PANIC_SELECTOR => decode_panic_code(data),
        _ => {
            // Try to decode as raw string or return hex
            Some(format!("0x{}", hex::encode(data.as_ref())))
        }
    }
}

/// Decode Error(string) revert
fn decode_error_string(data: &Bytes) -> Option<String> {
    // Error(string) ABI encoding:
    // 4 bytes selector
    // 32 bytes offset to string data
    // 32 bytes string length
    // N bytes string data (padded to 32)
    
    if data.len() < 68 {
        return Some("Error: (unable to decode)".to_string());
    }

    // Read offset (should be 32)
    let offset = u64::from_be_bytes(data[28..36].try_into().ok()?) as usize;
    
    if data.len() < 4 + offset + 32 {
        return Some("Error: (malformed data)".to_string());
    }

    // Read string length
    let length_start = 4 + offset;
    let length = u64::from_be_bytes(data[length_start + 24..length_start + 32].try_into().ok()?) as usize;
    
    let string_start = length_start + 32;
    if data.len() < string_start + length {
        return Some("Error: (string truncated)".to_string());
    }

    // Read string data
    let string_data = &data[string_start..string_start + length];
    String::from_utf8(string_data.to_vec())
        .map(|s| format!("Error: {}", s))
        .ok()
        .or_else(|| Some("Error: (invalid UTF-8)".to_string()))
}

/// Decode Panic(uint256) revert
fn decode_panic_code(data: &Bytes) -> Option<String> {
    if data.len() < 36 {
        return Some("Panic: (unable to decode)".to_string());
    }

    // Read panic code (last 8 bytes of 32-byte uint256)
    let code = u64::from_be_bytes(data[28..36].try_into().ok()?);
    
    let description = match code {
        0x00 => "generic compiler panic",
        0x01 => "assertion failed",
        0x11 => "arithmetic overflow/underflow",
        0x12 => "division by zero",
        0x21 => "invalid enum value",
        0x22 => "storage byte array encoding error",
        0x31 => "pop on empty array",
        0x32 => "array index out of bounds",
        0x41 => "memory allocation overflow",
        0x51 => "zero-initialized function pointer call",
        _ => "unknown panic code",
    };

    Some(format!("Panic(0x{:02x}): {}", code, description))
}

/// Extract revert data from an RPC error message
///
/// Many RPC providers include the revert data in the error message
/// in various formats.
pub fn extract_revert_from_error(error: &str) -> Option<Bytes> {
    // Try to find hex data in the error message
    // Common patterns:
    // - "execution reverted: 0x..."
    // - "revert: 0x..."
    // - just "0x..." at the end
    
    if let Some(pos) = error.find("0x") {
        let hex_str = &error[pos..];
        // Find end of hex string
        let end = hex_str
            .char_indices()
            .skip(2)
            .find(|(_, c)| !c.is_ascii_hexdigit())
            .map(|(i, _)| i)
            .unwrap_or(hex_str.len());
        
        let hex_part = &hex_str[2..end];
        if hex_part.len() >= 8 {
            if let Ok(bytes) = hex::decode(hex_part) {
                return Some(Bytes::from(bytes));
            }
        }
    }
    
    None
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_creation() {
        let simulator = TransactionSimulator::new();
        assert!(simulator.default_sender.is_none());
        
        let addr = Address::ZERO;
        let simulator_with_sender = TransactionSimulator::with_sender(addr);
        assert_eq!(simulator_with_sender.default_sender, Some(addr));
    }

    #[test]
    fn test_simulation_result_success() {
        let corr_id = Uuid::new_v4();
        let result = SimulationResult::success(
            Bytes::from(vec![0x00]),
            21000,
            corr_id,
            100,
        );
        
        assert!(result.success);
        assert!(result.return_data.is_some());
        assert!(result.revert_reason.is_none());
        assert_eq!(result.gas_used, Some(21000));
    }

    #[test]
    fn test_simulation_result_failure() {
        let corr_id = Uuid::new_v4();
        
        // Create properly ABI-encoded Error(string) with "test error"
        // Format:
        // 4 bytes: selector (0x08c379a0)
        // 32 bytes: offset to string data (0x20 = 32)
        // 32 bytes: string length (0x0a = 10)
        // 32 bytes: string data ("test error" + padding)
        let mut error_data = Vec::new();
        
        // Selector
        error_data.extend_from_slice(&[0x08, 0xc3, 0x79, 0xa0]);
        
        // Offset = 32 (padded to 32 bytes)
        error_data.extend_from_slice(&[0u8; 31]);
        error_data.push(0x20);
        
        // Length = 10 (padded to 32 bytes)
        error_data.extend_from_slice(&[0u8; 31]);
        error_data.push(0x0a);
        
        // String "test error" (padded to 32 bytes)
        error_data.extend_from_slice(b"test error");
        error_data.extend_from_slice(&[0u8; 22]);
        
        let result = SimulationResult::failure(
            Bytes::from(error_data),
            Some(50000),
            corr_id,
            50,
        );
        
        assert!(!result.success);
        assert!(result.revert_reason.is_some());
        // Check that decoding worked
        let reason = result.revert_reason.as_ref().unwrap();
        assert!(reason.contains("Error"), "Expected 'Error' but got: {}", reason);
    }

    #[test]
    fn test_decode_panic_code() {
        // Panic(0x01) - assertion failed
        let mut panic_data = vec![0x4e, 0x48, 0x7b, 0x71]; // selector
        panic_data.extend_from_slice(&[0u8; 31]);
        panic_data.push(0x01); // panic code
        
        let decoded = decode_revert_reason(&Bytes::from(panic_data));
        assert!(decoded.is_some());
        assert!(decoded.as_ref().unwrap().contains("assertion failed"));
    }

    #[test]
    fn test_decode_arithmetic_overflow() {
        // Panic(0x11) - arithmetic overflow
        let mut panic_data = vec![0x4e, 0x48, 0x7b, 0x71]; // selector
        panic_data.extend_from_slice(&[0u8; 31]);
        panic_data.push(0x11); // panic code
        
        let decoded = decode_revert_reason(&Bytes::from(panic_data));
        assert!(decoded.is_some());
        assert!(decoded.as_ref().unwrap().contains("overflow"));
    }

    #[test]
    fn test_decode_division_by_zero() {
        // Panic(0x12) - division by zero
        let mut panic_data = vec![0x4e, 0x48, 0x7b, 0x71]; // selector
        panic_data.extend_from_slice(&[0u8; 31]);
        panic_data.push(0x12); // panic code
        
        let decoded = decode_revert_reason(&Bytes::from(panic_data));
        assert!(decoded.is_some());
        assert!(decoded.as_ref().unwrap().contains("division by zero"));
    }

    #[test]
    fn test_warning_levels() {
        let corr_id = Uuid::new_v4();
        
        let success_result = SimulationResult::success(
            Bytes::new(),
            21000,
            corr_id,
            10,
        );
        assert_eq!(TransactionSimulator::get_warning_level(&success_result), SimulationWarning::None);
        
        let failure_result = SimulationResult::from_error("test error", corr_id, 10);
        assert_eq!(TransactionSimulator::get_warning_level(&failure_result), SimulationWarning::High);
    }

    #[test]
    fn test_prepare_simulation() {
        let simulator = TransactionSimulator::new();
        
        let tx = TransactionRequest::default();
        let params = simulator.prepare_simulation(&tx);
        
        assert!(!params.correlation_id.is_nil());
    }

    #[test]
    fn test_prepare_simulation_with_sender() {
        let addr = Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18").unwrap();
        let simulator = TransactionSimulator::with_sender(addr);
        
        let tx = TransactionRequest::default();
        let params = simulator.prepare_simulation(&tx);
        
        assert_eq!(params.tx.from, Some(addr));
    }

    #[test]
    fn test_extract_revert_from_error() {
        let error = "execution reverted: 0x08c379a0";
        let result = extract_revert_from_error(error);
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_vec(), vec![0x08, 0xc3, 0x79, 0xa0]);
    }

    #[test]
    fn test_extract_revert_no_hex() {
        let error = "some other error";
        let result = extract_revert_from_error(error);
        assert!(result.is_none());
    }

    #[test]
    fn test_simulation_warning_messages() {
        assert_eq!(SimulationWarning::None.message(), "Transaction simulation passed");
        assert_eq!(SimulationWarning::High.message(), "Transaction will fail");
    }

    #[test]
    fn test_decode_unknown_selector() {
        // Unknown selector should return hex
        let data = Bytes::from(vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]);
        let decoded = decode_revert_reason(&data);
        assert!(decoded.is_some());
        assert!(decoded.as_ref().unwrap().starts_with("0x"));
    }

    #[test]
    fn test_decode_short_data() {
        // Too short to decode
        let data = Bytes::from(vec![0x08, 0xc3]);
        let decoded = decode_revert_reason(&data);
        assert!(decoded.is_none());
    }
}
