//! Memory protection utilities for sensitive data
//!
//! This module provides memory locking, secure allocation, and other
//! memory protection mechanisms to prevent sensitive data from being
//! swapped to disk or accessed by unauthorized processes.

// Allow unsafe code - all unsafe blocks are documented with SAFETY comments
// See Phase 4 Task 4.3 completion for full unsafe code audit
#![allow(unsafe_code)]

use crate::error::{Result, SecurityError};
use std::ptr;

/// Platform-specific memory protection implementation
pub struct MemoryProtection;

impl MemoryProtection {
    /// Lock memory pages to prevent swapping to disk
    #[cfg(unix)]
    pub fn lock_memory(addr: *mut u8, len: usize) -> Result<()> {
        // SAFETY: mlock is safe when called with valid memory addresses and lengths.
        // The addr pointer comes from valid Rust allocations and len represents the actual size.
        let result = unsafe { libc::mlock(addr as *const libc::c_void, len) };

        if result != 0 {
            let error = std::io::Error::last_os_error();
            tracing::warn!("Failed to lock memory: {}", error);
            // Don't fail hard on mlock failure - it's not always available
            // but log the warning so users can investigate
        } else {
            tracing::debug!("Successfully locked {} bytes of memory", len);
        }

        Ok(())
    }

    /// Unlock previously locked memory pages
    #[cfg(unix)]
    pub fn unlock_memory(addr: *mut u8, len: usize) -> Result<()> {
        // SAFETY: munlock is safe when called with valid memory addresses and lengths.
        // The addr pointer comes from valid Rust allocations and len represents the actual size.
        let result = unsafe { libc::munlock(addr as *const libc::c_void, len) };

        if result != 0 {
            let error = std::io::Error::last_os_error();
            tracing::warn!("Failed to unlock memory: {}", error);
        } else {
            tracing::debug!("Successfully unlocked {} bytes of memory", len);
        }

        Ok(())
    }

    /// Windows implementation - use VirtualLock
    #[cfg(windows)]
    pub fn lock_memory(addr: *mut u8, len: usize) -> Result<()> {
        use winapi::um::memoryapi::VirtualLock;

        // SAFETY: VirtualLock is safe when called with valid memory addresses and lengths.
        // - addr pointer comes from valid Rust allocations
        // - len represents the actual allocated size
        // This is a standard Windows API for preventing memory from being swapped to disk.
        let result = unsafe { VirtualLock(addr as *mut winapi::ctypes::c_void, len) };

        if result == 0 {
            let error = std::io::Error::last_os_error();
            tracing::warn!("Failed to lock memory: {}", error);
            // Don't fail hard - memory locking might not be available
        } else {
            tracing::debug!("Successfully locked {} bytes of memory", len);
        }

        Ok(())
    }

    /// Windows implementation - use VirtualUnlock
    #[cfg(windows)]
    pub fn unlock_memory(addr: *mut u8, len: usize) -> Result<()> {
        use winapi::um::memoryapi::VirtualUnlock;

        // SAFETY: VirtualUnlock is safe when called with valid memory addresses and lengths.
        // This unlocks previously locked memory. Same safety guarantees as VirtualLock.
        let result = unsafe { VirtualUnlock(addr as *mut winapi::ctypes::c_void, len) };

        if result == 0 {
            let error = std::io::Error::last_os_error();
            tracing::warn!("Failed to unlock memory: {}", error);
        } else {
            tracing::debug!("Successfully unlocked {} bytes of memory", len);
        }

        Ok(())
    }

    /// No-op for unsupported platforms
    #[cfg(not(any(unix, windows)))]
    pub fn lock_memory(_addr: *mut u8, _len: usize) -> Result<()> {
        tracing::warn!("Memory locking not supported on this platform");
        Ok(())
    }

    /// No-op for unsupported platforms
    #[cfg(not(any(unix, windows)))]
    pub fn unlock_memory(_addr: *mut u8, _len: usize) -> Result<()> {
        tracing::warn!("Memory unlocking not supported on this platform");
        Ok(())
    }

    /// Disable core dumps for the current process
    #[cfg(unix)]
    pub fn disable_core_dumps() -> Result<()> {
        let rlimit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        // SAFETY: setrlimit is safe when called with valid rlimit structure.
        // We're setting core dump limits for security - this is a standard security practice.
        let result = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlimit) };

        if result != 0 {
            return Err(SecurityError::KeystoreError {
                message: "Failed to disable core dumps".to_string(),
            }
            .into());
        }

        tracing::info!("Core dumps disabled for security");
        Ok(())
    }

    /// Windows doesn't have core dumps in the same way
    #[cfg(windows)]
    pub fn disable_core_dumps() -> Result<()> {
        tracing::info!("Core dump protection not needed on Windows");
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    pub fn disable_core_dumps() -> Result<()> {
        tracing::warn!("Core dump protection not supported on this platform");
        Ok(())
    }
}

/// Secure memory allocation that locks memory and zeros on drop
#[derive(Debug)]
pub struct SecureMemory {
    ptr: *mut u8,
    len: usize,
    locked: bool,
}

impl SecureMemory {
    /// Allocate secure memory
    pub fn new(size: usize) -> Result<Self> {
        // Allocate aligned memory
        let layout = std::alloc::Layout::from_size_align(size, 8).map_err(|_| SecurityError::KeystoreError {
            message: "Invalid memory layout".to_string(),
        })?;

        // SAFETY: alloc_zeroed is safe when called with valid layout.
        // Layout was validated above and we check for null pointer below.
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };

        if ptr.is_null() {
            return Err(SecurityError::KeystoreError {
                message: "Failed to allocate secure memory".to_string(),
            }
            .into());
        }

        // Attempt to lock the memory
        let locked = MemoryProtection::lock_memory(ptr, size).is_ok();

        Ok(SecureMemory { ptr, len: size, locked })
    }

    /// Get a mutable pointer to the memory
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    /// Get an immutable pointer to the memory
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get a mutable slice view of the memory
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: Creating slice from raw parts is safe here because:
        // - self.ptr was allocated with self.len bytes and is valid for reads/writes
        // - The slice lifetime is tied to self's lifetime preventing use-after-free
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Get the size of allocated memory
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if memory is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Zero the memory contents
    pub fn zero(&mut self) {
        // SAFETY: write_bytes is safe here because:
        // - self.ptr was allocated with self.len bytes and is valid for writes
        // - We're writing zeros to the entire allocated region
        unsafe {
            ptr::write_bytes(self.ptr, 0, self.len);
        }
    }

    /// Check if memory is locked
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Drop for SecureMemory {
    fn drop(&mut self) {
        // Zero the memory before deallocation
        self.zero();

        // Unlock memory if it was locked
        if self.locked {
            let _ = MemoryProtection::unlock_memory(self.ptr, self.len);
        }

        // Deallocate
        // SAFETY: dealloc is safe here because:
        // - self.ptr was allocated with the exact same layout (size=self.len, align=8)
        // - We're calling dealloc exactly once per allocation
        unsafe {
            let layout = std::alloc::Layout::from_size_align_unchecked(self.len, 8);
            std::alloc::dealloc(self.ptr, layout);
        }
    }
}

// SecureMemory cannot be cloned for security reasons
// Rust automatically prevents cloning due to the lack of Clone implementation

// SAFETY: SecureMemory can be safely sent between threads because:
// - It owns its memory exclusively (no shared references)
// - All operations are thread-safe
// - Drop implementation properly cleans up regardless of thread
unsafe impl Send for SecureMemory {}

/// Initialize memory protection for the application
pub fn init_memory_protection() -> Result<()> {
    // Disable core dumps to prevent sensitive data from being written to disk
    MemoryProtection::disable_core_dumps()?;

    tracing::info!("Memory protection initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_memory_allocation() {
        let mut memory = SecureMemory::new(1024).unwrap();

        assert_eq!(memory.len(), 1024);
        assert!(!memory.is_empty());

        // Write some data
        {
            let slice = memory.as_mut_slice();
            slice[0] = 0xAA;
            slice[1023] = 0xBB;

            assert_eq!(slice[0], 0xAA);
            assert_eq!(slice[1023], 0xBB);
        }

        // Zero the memory
        memory.zero();

        // Check that memory was zeroed
        let slice = memory.as_mut_slice();
        assert_eq!(slice[0], 0);
        assert_eq!(slice[1023], 0);
    }

    #[test]
    fn test_memory_protection_calls() {
        // Just test that the functions don't panic
        // Actual locking may fail without privileges
        let mut memory = SecureMemory::new(4096).unwrap();
        let ptr = memory.as_mut_ptr();

        let _ = MemoryProtection::lock_memory(ptr, 4096);
        let _ = MemoryProtection::unlock_memory(ptr, 4096);
    }

    #[test]
    fn test_disable_core_dumps() {
        // Should not panic
        let _ = MemoryProtection::disable_core_dumps();
    }
}
