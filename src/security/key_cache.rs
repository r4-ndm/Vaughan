//! Key Cache Module
//!
//! Provides secure caching of derived private keys in memory with automatic zeroization.
//! Keys are stored in SecureMemory which uses memory locking when available.

use crate::error::Result;
use crate::security::memory::SecureMemory;
use alloy::primitives::Address;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cache for derived private keys with automatic expiration and zeroization
#[derive(Debug)]
pub struct KeyCache {
    /// Cached keys stored in secure memory
    cached_keys: HashMap<Address, CachedKey>,

    /// Cache timeout duration
    cache_timeout: Duration,

    /// Whether memory locking is available
    memory_lock_available: bool,
}

/// A cached key with metadata
#[derive(Debug)]
struct CachedKey {
    /// The actual key data in secure memory
    key: SecureMemory,

    /// When this key was cached
    cached_at: Instant,

    /// Last time this key was accessed
    last_access: Instant,
}

impl KeyCache {
    /// Create a new key cache with the specified timeout
    pub fn new(timeout: Duration) -> Self {
        // Test if memory locking is available
        let memory_lock_available = Self::test_memory_locking();

        if !memory_lock_available {
            tracing::warn!("⚠️ Memory locking unavailable - using shorter cache timeout for security");
        }

        // Use shorter timeout (5 min) if memory locking fails
        let actual_timeout = if memory_lock_available {
            timeout
        } else {
            Duration::from_secs(5 * 60) // 5 minutes
        };

        tracing::info!(
            "🔑 Key cache initialized with {}min timeout (memory locking: {})",
            actual_timeout.as_secs() / 60,
            if memory_lock_available { "enabled" } else { "disabled" }
        );

        Self {
            cached_keys: HashMap::new(),
            cache_timeout: actual_timeout,
            memory_lock_available,
        }
    }

    /// Test if memory locking is available
    fn test_memory_locking() -> bool {
        // Try to create a small SecureMemory to test if mlock works
        match SecureMemory::new(32) {
            Ok(_) => true,
            Err(e) => {
                tracing::debug!("Memory locking test failed: {}", e);
                false
            }
        }
    }

    /// Insert a key into the cache
    pub fn insert(&mut self, address: Address, key_bytes: Vec<u8>) -> Result<()> {
        // Create secure memory for the key
        let mut secure_key = SecureMemory::new(key_bytes.len())?;
        secure_key.as_mut_slice().copy_from_slice(&key_bytes);

        let cached_key = CachedKey {
            key: secure_key,
            cached_at: Instant::now(),
            last_access: Instant::now(),
        };

        // Remove old key if it exists (will be zeroized on drop)
        if let Some(old_key) = self.cached_keys.remove(&address) {
            drop(old_key); // Explicit drop for clarity
        }

        self.cached_keys.insert(address, cached_key);

        tracing::debug!("🔑 Cached key for address: {}", address);

        Ok(())
    }

    /// Get a key from the cache (returns a copy for safety)
    pub fn get(&mut self, address: &Address) -> Option<Vec<u8>> {
        if let Some(cached_key) = self.cached_keys.get_mut(address) {
            // Check if key has expired
            if cached_key.cached_at.elapsed() >= self.cache_timeout {
                tracing::debug!("🔑 Key expired for address: {}", address);
                self.cached_keys.remove(address); // Will be zeroized on drop
                return None;
            }

            // Update last access time
            cached_key.last_access = Instant::now();

            tracing::debug!("🔑 Retrieved cached key for address: {}", address);

            // Return a copy of the key bytes
            Some(cached_key.key.as_mut_slice().to_vec())
        } else {
            None
        }
    }

    /// Remove a key from the cache
    pub fn remove(&mut self, address: &Address) -> bool {
        if let Some(key) = self.cached_keys.remove(address) {
            drop(key); // Explicit drop to zeroize
            tracing::debug!("🔑 Removed key for address: {}", address);
            true
        } else {
            false
        }
    }

    /// Clear all cached keys
    pub fn clear(&mut self) {
        let count = self.cached_keys.len();
        self.cached_keys.clear(); // All keys will be zeroized on drop

        if count > 0 {
            tracing::info!("🔑 Cleared {} cached keys", count);
        }
    }

    /// Remove expired keys
    pub fn remove_expired(&mut self) {
        let before_count = self.cached_keys.len();

        self.cached_keys.retain(|address, cached_key| {
            let expired = cached_key.cached_at.elapsed() >= self.cache_timeout;
            if expired {
                tracing::debug!("🔑 Removing expired key for address: {}", address);
            }
            !expired
        });

        let removed = before_count - self.cached_keys.len();
        if removed > 0 {
            tracing::debug!("🔑 Removed {} expired keys", removed);
        }
    }

    /// Get the number of cached keys
    pub fn len(&self) -> usize {
        self.cached_keys.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cached_keys.is_empty()
    }

    /// Check if memory locking is available
    pub fn has_memory_locking(&self) -> bool {
        self.memory_lock_available
    }

    /// Get the cache timeout duration
    pub fn timeout(&self) -> Duration {
        self.cache_timeout
    }
}

impl Drop for KeyCache {
    fn drop(&mut self) {
        // Clear all keys (will be zeroized)
        self.clear();
        tracing::debug!("🔑 Key cache dropped and zeroized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_cache_basic() {
        let mut cache = KeyCache::new(Duration::from_secs(60));

        let address = Address::from_slice(&[0u8; 20]);
        let key_bytes = vec![1, 2, 3, 4, 5];

        // Insert key
        cache.insert(address, key_bytes.clone()).unwrap();
        assert_eq!(cache.len(), 1);

        // Retrieve key
        let retrieved = cache.get(&address).unwrap();
        assert_eq!(retrieved, &key_bytes[..]);

        // Remove key
        assert!(cache.remove(&address));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_key_cache_expiration() {
        let mut cache = KeyCache::new(Duration::from_millis(100));

        let address = Address::from_slice(&[1u8; 20]);
        let key_bytes = vec![1, 2, 3, 4, 5];

        // Insert key
        cache.insert(address, key_bytes).unwrap();

        // Key should be available immediately
        assert!(cache.get(&address).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Key should be expired
        assert!(cache.get(&address).is_none());
    }

    #[test]
    fn test_key_cache_clear() {
        let mut cache = KeyCache::new(Duration::from_secs(60));

        // Insert multiple keys
        for i in 0..5 {
            let address = Address::from_slice(&[i as u8; 20]);
            let key_bytes = vec![i; 32];
            cache.insert(address, key_bytes).unwrap();
        }

        assert_eq!(cache.len(), 5);

        // Clear all
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
}
