use super::*;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_rate_limiter_allows_under_limit() {
    let mut configs = HashMap::new();
    configs.insert("test_op".to_string(), RateLimitConfig {
        capacity: 5,
        refill_rate_per_second: 1.0,
    });
    
    let limiter = RateLimiter::new(configs);
    
    // Should allow 5 immediate calls
    for _ in 0..5 {
        assert!(limiter.check("test_op").is_ok());
    }
}

#[test]
fn test_rate_limiter_blocks_over_limit() {
    let mut configs = HashMap::new();
    configs.insert("test_op".to_string(), RateLimitConfig {
        capacity: 1,
        refill_rate_per_second: 0.1, // Very slow refill
    });
    
    let limiter = RateLimiter::new(configs);
    
    assert!(limiter.check("test_op").is_ok());
    
    // Should block immediately
    match limiter.check("test_op") {
        Err(VaughanError::Security(SecurityError::RateLimitExceeded { .. })) => {},
        _ => panic!("Should have returned RateLimitExceeded"),
    }
}

#[test]
fn test_rate_limiter_refills_over_time() {
    let mut configs = HashMap::new();
    configs.insert("test_op".to_string(), RateLimitConfig {
        capacity: 1,
        refill_rate_per_second: 10.0, // Fast refill
    });
    
    let limiter = RateLimiter::new(configs);
    
    assert!(limiter.check("test_op").is_ok());
    assert!(limiter.check("test_op").is_err());
    
    // Wait for refill (at least 0.1s for 1 token)
    sleep(Duration::from_millis(150));
    
    assert!(limiter.check("test_op").is_ok());
}

#[test]
fn test_rate_limiter_missing_config() {
    let limiter = RateLimiter::new(HashMap::new());
    match limiter.check("unknown") {
        Err(VaughanError::Configuration(_)) => {},
        _ => panic!("Should error on missing config"),
    }
}

#[test]
fn test_rate_limiter_persistence() {
    let mut configs = HashMap::new();
    let op_name = "persist_op";
    configs.insert(op_name.to_string(), RateLimitConfig {
        capacity: 10,
        refill_rate_per_second: 0.0, // No refill
    });
    
    let temp_file = std::env::temp_dir().join(format!("rate_limit_{}.json", Uuid::new_v4()));
    
    // 1. Create limiter, set path, consume 5
    {
        let mut limiter = RateLimiter::new(configs.clone());
        limiter.set_persistence_path(temp_file.clone());
        
        for _ in 0..5 {
            assert!(limiter.check(op_name).is_ok());
        }
        // State should be saved: 5 tokens consumed, 5 remaining
    }

    // 2. Create NEW limiter with same path
    {
        let mut limiter = RateLimiter::new(configs.clone());
        limiter.set_persistence_path(temp_file.clone());
        
        // Should only have 5 tokens left
        for _ in 0..5 {
             assert!(limiter.check(op_name).is_ok());
        }
        
        // 11th check should fail (5 pre-consumed + 5 now)
        assert!(limiter.check(op_name).is_err());
    }
    
    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}

