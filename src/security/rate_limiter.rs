use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{Result, SecurityError, VaughanError};

/// Configuration for a rate limit bucket
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of tokens in the bucket
    pub capacity: u32,
    /// Number of tokens refilled per second
    pub refill_rate_per_second: f64,
}

/// Token bucket state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBucket {
    tokens: f64,
    last_refill: DateTime<Utc>,
    config: RateLimitConfig,
}

impl TokenBucket {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.capacity as f64,
            last_refill: Utc::now(),
            config,
        }
    }

    fn try_consume(&mut self, amount: u32) -> bool {
        self.refill();
        if self.tokens >= amount as f64 {
            self.tokens -= amount as f64;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = (now - self.last_refill)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs_f64();
        
        let new_tokens = elapsed * self.config.refill_rate_per_second;
        if new_tokens > 0.0 {
            self.tokens = (self.tokens + new_tokens).min(self.config.capacity as f64);
            self.last_refill = now;
        }
    }

    fn time_until_refill(&self, needed: u32) -> std::time::Duration {
        if self.tokens >= needed as f64 {
            return std::time::Duration::from_secs(0);
        }
        if self.config.refill_rate_per_second == 0.0 {
            return std::time::Duration::from_secs(3600 * 24 * 365); // Effectively forever (1 year)
        }
        let missing = needed as f64 - self.tokens;
        let seconds_needed = missing / self.config.refill_rate_per_second;
        std::time::Duration::from_secs_f64(seconds_needed)
    }
}

/// Centralized Rate Limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    configs: Arc<HashMap<String, RateLimitConfig>>,
    persistence_path: Option<PathBuf>,
}

impl RateLimiter {
    /// Create a new RateLimiter with defined configurations
    pub fn new(configs: HashMap<String, RateLimitConfig>) -> Self {
        // Determine persistence path
        let persistence_path = Self::get_default_path();

        let limiter = Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            configs: Arc::new(configs),
            persistence_path,
        };

        // Try to load existing state
        if let Err(e) = limiter.load() {
            tracing::warn!("Failed to load rate limiter state: {}", e);
        }

        limiter
    }

    /// Get default path for rate limit persistence
    fn get_default_path() -> Option<PathBuf> {
        #[cfg(test)]
        {
            return None;
        }

        #[cfg(not(test))]
        {
            if let Ok(home_dir) = std::env::var("HOME") {
                 Some(PathBuf::from(home_dir).join(".config").join("vaughan").join("rate_limits.json"))
            } else if let Ok(appdata_dir) = std::env::var("APPDATA") {
                 Some(PathBuf::from(appdata_dir).join("Vaughan").join("rate_limits.json"))
            } else {
                 None
            }
        }
    }

    /// Check if an operation is allowed
    pub fn check(&self, operation: &str) -> Result<()> {
        let correlation_id = Uuid::new_v4();
        let config = self.configs.get(operation).ok_or_else(|| {
             tracing::warn!(%correlation_id, operation, "Rate limit check for unknown operation");
             VaughanError::Configuration(crate::error::ConfigurationError::MissingKey { key: operation.to_string() })
        })?;
        
        // We need to initialize the bucket if it doesn't exist
        let mut buckets = self.buckets.lock().map_err(|_| {
            VaughanError::Security(SecurityError::RateLimitExceeded { 
                operation: operation.to_string(), 
                wait_time_seconds: 1 
            }) // Mutex poisoned
        })?;

        let bucket = buckets.entry(operation.to_string())
            .or_insert_with(|| TokenBucket::new(*config));

        let allowed = bucket.try_consume(1);
        
        // Save state after modification
        // Note: In high throughput scenarios, we would batch this.
        // For sensitive ops logic, immediate save reduces risk of bypass via crash/restart.
        drop(buckets); // Drop lock before saving to avoid holding it during I/O
        if let Err(e) = self.save() {
            tracing::warn!("Failed to persist rate limit state: {}", e);
        }

        if allowed {
            tracing::debug!(%correlation_id, operation, "Rate limit check passed");
            Ok(())
        } else {
            // Need to re-acquire lock to get wait time? Or we could have calculated it before.
            // Let's re-acquire read-only or just calculate based on config (approximate).
            // Better: calculate wait time while we had the lock.
            // Refactoring slightly to keep it clean.
            let Ok(buckets) = self.buckets.lock() else {
                // If mutex is poisoned, deny the request (fail closed for security)
                return Err(VaughanError::Security(SecurityError::RateLimitExceeded {
                    operation: operation.to_string(),
                    wait_time_seconds: 60,
                }));
            };
            let Some(bucket) = buckets.get(operation) else {
                // Operation not found, deny request
                return Err(VaughanError::Security(SecurityError::RateLimitExceeded {
                    operation: operation.to_string(),
                    wait_time_seconds: 60,
                }));
            };
            let wait_time = bucket.time_until_refill(1);
            
            tracing::warn!(%correlation_id, operation, wait_time = ?wait_time, "Rate limit exceeded");
            Err(VaughanError::Security(SecurityError::RateLimitExceeded {
                operation: operation.to_string(),
                wait_time_seconds: wait_time.as_secs().max(1),
            }))
        }
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let path = match &self.persistence_path {
            Some(p) => p,
            None => return Ok(()),
        };

        let buckets = self.buckets.lock().map_err(|_| {
             VaughanError::Security(SecurityError::KeystoreError { 
                message: "Lock poisoned".to_string() 
             })
        })?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                VaughanError::Security(SecurityError::KeystoreError { message: e.to_string() })
            })?;
        }

        let json = serde_json::to_string(&*buckets).map_err(|e| {
            VaughanError::Security(SecurityError::SerializationError { message: e.to_string() })
        })?;

        std::fs::write(path, json).map_err(|e| {
            VaughanError::Security(SecurityError::KeystoreError { message: e.to_string() })
        })?;

        Ok(())
    }

    /// Load state from disk
    fn load(&self) -> Result<()> {
        let path = match &self.persistence_path {
            Some(p) => p,
            None => return Ok(()),
        };

        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(path).map_err(|e| {
            VaughanError::Security(SecurityError::KeystoreError { message: e.to_string() })
        })?;

        let loaded_buckets: HashMap<String, TokenBucket> = serde_json::from_str(&json).map_err(|e| {
            VaughanError::Security(SecurityError::SerializationError { message: e.to_string() })
        })?;

        let mut buckets = self.buckets.lock().map_err(|_| {
            VaughanError::Security(SecurityError::KeystoreError { 
                message: "Lock poisoned".to_string() 
            })
        })?;

        // Merge loaded buckets with current configs
        // We only keep buckets that still have a valid config, or maybe we accept all?
        // Let's accept all loaded buckets but update their usage.
        // Actually, we should probably check if config matches or just blindly load state.
        // Safest is to load state.
        *buckets = loaded_buckets;

        Ok(())
    }

    /// Reset a rate limit (e.g. on successful login if we only limit failures)
    pub fn reset(&self, operation: &str) {
        if let Ok(mut buckets) = self.buckets.lock() {
            if let Some(config) = self.configs.get(operation) {
                buckets.insert(operation.to_string(), TokenBucket::new(*config));
            }
        }
        let _ = self.save();
    }

    #[cfg(test)]
    pub fn set_persistence_path(&mut self, path: PathBuf) {
        self.persistence_path = Some(path);
        // Reload from new path
        if let Err(_e) = self.load() {
             // Ignore error for tests if file invalid
        }
    }
}

#[cfg(test)]
#[path = "rate_limiter_tests.rs"]
mod tests;
