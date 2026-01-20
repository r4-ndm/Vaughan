//! Zeroization utilities for secure memory handling
//!
//! This module provides secure memory handling with automatic zeroization
//! to prevent sensitive data from lingering in memory after use.
//!
//! Security Design (inspired by MetaMask security practices):
//! - Automatic zeroization on Drop
//! - Secure byte and string zeroing
//! - Integration with secrecy crate

use zeroize::Zeroize;

// ============================================================================
// Secure Byte Utilities
// ============================================================================

/// Securely zero out a byte array
#[inline]
pub fn zero_bytes(bytes: &mut [u8]) {
    bytes.zeroize();
}

/// Securely zero out a fixed-size byte array
#[inline]
pub fn zero_array<const N: usize>(array: &mut [u8; N]) {
    array.zeroize();
}

/// Securely zero out a vector of bytes
#[inline]
pub fn zero_vec(vec: &mut Vec<u8>) {
    vec.zeroize();
}

/// Securely zero out a string
#[inline]
pub fn zero_string(string: &mut String) {
    string.zeroize();
}

// ============================================================================
// Secure Memory Guard
// ============================================================================

/// A guard that ensures sensitive data is zeroized when dropped
pub struct SecureGuard<T: Zeroize> {
    data: T,
}

impl<T: Zeroize> SecureGuard<T> {
    /// Create a new secure guard around the data
    pub fn new(data: T) -> Self {
        Self { data }
    }

    /// Get a reference to the protected data
    pub fn expose(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the protected data
    pub fn expose_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Zeroize> Drop for SecureGuard<T> {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

// ============================================================================
// Constants
// ============================================================================

/// Number of default overwrite passes for secure erasure
pub const SECURE_ERASE_PASSES: usize = 3;

/// Verify that zeroization is working (for tests)
#[cfg(test)]
pub fn verify_zeroed(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_bytes() {
        let mut bytes = vec![1u8, 2, 3, 4, 5];
        zero_bytes(&mut bytes);
        assert!(verify_zeroed(&bytes));
    }

    #[test]
    fn test_zero_array() {
        let mut array = [1u8, 2, 3, 4, 5, 6, 7, 8];
        zero_array(&mut array);
        assert!(verify_zeroed(&array));
    }

    #[test]
    fn test_secure_guard() {
        let mut guarded = SecureGuard::new(vec![1u8, 2, 3, 4]);
        assert_eq!(guarded.expose(), &vec![1u8, 2, 3, 4]);

        guarded.expose_mut()[0] = 5;
        assert_eq!(guarded.expose()[0], 5);
    }
}
