//! Export Authentication System
//!
//! This module provides secure authentication for sensitive export operations.
//! It implements:
//! - Time-limited authentication tokens (2 minutes)
//! - Rate limiting (5 attempts per minute)
//! - Secure password verification hooks
//!
//! # Requirements
//! - **Requirement 2.2**: Secure authentication for critical operations

use chrono::{DateTime, Duration, Utc};
// secrecy imports removed as unused
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{Result, SecurityError, VaughanError};

/// Time-to-live for an authentication token
const TOKEN_TTL_SECONDS: i64 = 120; // 2 minutes

/// A time-limited authentication token
///
/// This token serves as proof of recent authentication efficiently,
/// avoiding the need to re-enter a password for a short sequence of operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Unique token ID
    pub id: String,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
    /// Secure signature/secret (simulated here with random bytes for uniqueness)
    pub signature: String, 
}

impl AuthToken {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Generate a new valid token
    fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            expires_at: Utc::now() + Duration::seconds(TOKEN_TTL_SECONDS),
            signature: Uuid::new_v4().to_string(),
        }
    }
}


use crate::security::{RateLimiter, RateLimitConfig};
use std::collections::HashMap;

/// Authenticator for sensitive operations
#[derive(Debug, Clone)]
pub struct ExportAuthenticator {
    rate_limiter: Arc<RateLimiter>,
}

impl ExportAuthenticator {
    /// Create a new authenticator with standard limits
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        // Task 10.3: Authentication limit (e.g., 5 attempts per minute)
        // Capacity 5, refill 5 per 60s (~0.083/s)
        configs.insert("auth_attempt".to_string(), RateLimitConfig {
            capacity: 5,
            refill_rate_per_second: 5.0 / 60.0,
        });

        // Task 10.2: Export limit (e.g., 3 exports per hour)
        // Capacity 3, refill 3 per 3600s (~0.00083/s)
        configs.insert("export_op".to_string(), RateLimitConfig {
            capacity: 3,
            refill_rate_per_second: 3.0 / 3600.0,
        });

        Self {
            rate_limiter: Arc::new(RateLimiter::new(configs)),
        }
    }

    /// Authenticate the user and issue a token
    pub async fn authenticate(&self, password_valid: bool) -> Result<AuthToken> {
        // Check rate limit for authentication attempts
        self.rate_limiter.check("auth_attempt")?;

        if !password_valid {
            return Err(VaughanError::Security(SecurityError::InvalidPassword));
        }

        Ok(AuthToken::new())
    }

    /// Validate a token for usage
    pub fn validate_token(&self, token: &AuthToken) -> Result<()> {
        if token.is_expired() {
            return Err(VaughanError::Security(SecurityError::TokenExpired));
        }
        Ok(())
    }

    /// Check if an export operation is allowed by rate limits
    pub fn check_export_limit(&self) -> Result<()> {
        self.rate_limiter.check("export_op")
    }
}

impl Default for ExportAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_auth_success() {
        let auth = ExportAuthenticator::new();
        // Simulate correct password
        let token = auth.authenticate(true).await;
        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(!token.is_expired());
    }

    #[tokio::test]
    async fn test_auth_failure() {
        let auth = ExportAuthenticator::new();
        // Simulate incorrect password
        let result = auth.authenticate(false).await;
        assert!(matches!(result, Err(VaughanError::Security(SecurityError::InvalidPassword))));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let auth = ExportAuthenticator::new();
        
        // Consume all attempts (5)
        for _ in 0..5 {
            let _ = auth.authenticate(false).await; // Failed attempts count too
        }

        // Next attempt should fail with rate limit
        let result = auth.authenticate(true).await;
        assert!(matches!(result, Err(VaughanError::Security(SecurityError::RateLimitExceeded { .. }))));
    }

    #[tokio::test]
    async fn test_token_expiration() {
        // Create a token that expires very soon (manually for test, normally constant)
        let expired_token = AuthToken {
            id: "expired".to_string(),
            expires_at: Utc::now() - Duration::seconds(1),
            signature: "sig".to_string(),
        };
        
        let auth = ExportAuthenticator::new();
        assert!(auth.validate_token(&expired_token).is_err());
    }
}
