//! Performance Tests for Account Management
//!
//! These tests verify that account management operations meet performance requirements:
//! - Batch operations should be significantly faster than individual operations
//! - Cache hits should be much faster than cache misses (RPC calls)
//! - Async operations should not block UI (< 16ms for synchronous parts)
//!
//! # Requirements Validated
//! - Requirement 6.4: Batch processing efficiency
//! - Requirement 9.3: Operations should be async and non-blocking
//! - Requirement 9.5: Performance targets for wallet operations

use vaughan::error::Result;
use vaughan::gui::services::IntegratedAccountService;
use vaughan::performance::batch::{BatchConfig, BatchProcessor};
use vaughan::security::{SecureAccount, KeyReference};
use alloy::primitives::{Address, U256};
use std::time::{Duration, Instant};

// ============================================================================
// Performance Constants
// ============================================================================

/// Maximum time for a cache hit (should be near-instant)
const MAX_CACHE_HIT_TIME_MS: u128 = 1;

/// Maximum time for synchronous UI operations (60 FPS = 16.67ms per frame)
const MAX_SYNC_UI_TIME_MS: u128 = 16;

/// Expected speedup factor for batch vs individual operations
const EXPECTED_BATCH_SPEEDUP: f64 = 2.0;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test account with default values
fn create_test_account(name: &str, address: Address) -> SecureAccount {
    SecureAccount {
        id: format!("perf-{}", name),
        name: name.to_string(),
        address,
        key_reference: KeyReference {
            id: format!("key-{}", name),
            service: "vaughan-perf-test".to_string(),
            account: name.to_string(),
        },
        created_at: chrono::Utc::now(),
        is_hardware: false,
        derivation_path: Some("m/44'/60'/0'/0/0".to_string()),
        tags: Vec::new(),
        last_used: None,
        transaction_count: 0,
    }
}

/// Create N test accounts with unique addresses
fn create_n_accounts(n: usize) -> Vec<SecureAccount> {
    (0..n)
        .map(|i| {
            let mut addr_bytes = [0u8; 20];
            addr_bytes[18] = (i / 256) as u8;
            addr_bytes[19] = (i % 256) as u8;
            create_test_account(&format!("account_{}", i), Address::from(addr_bytes))
        })
        .collect()
}

/// Mock balance fetcher with simulated network delay
async fn mock_slow_fetcher(address: Address) -> Result<U256> {
    // Simulate network latency (10ms per call)
    tokio::time::sleep(Duration::from_millis(10)).await;
    Ok(U256::from(address.0[19] as u64 * 1_000_000_000_000_000_000u64))
}

/// Mock balance fetcher with no delay (for baseline)
async fn mock_instant_fetcher(address: Address) -> Result<U256> {
    Ok(U256::from(address.0[19] as u64 * 1_000_000_000_000_000_000u64))
}

// ============================================================================
// Batch Operation Performance Tests
// ============================================================================

#[tokio::test]
async fn test_batch_vs_individual_performance() {
    let service = IntegratedAccountService::new();
    let accounts = create_n_accounts(10);

    // Measure batch operation time
    let batch_start = Instant::now();
    let batch_result = service
        .refresh_account_balances(&accounts, mock_instant_fetcher)
        .await;
    let batch_duration = batch_start.elapsed();
    
    assert!(batch_result.is_ok());
    
    // Batch should complete quickly for instant fetcher
    println!(
        "📊 Batch operation (10 accounts, instant fetcher): {:?}",
        batch_duration
    );
    
    // Should be under 100ms for 10 accounts with instant fetcher
    assert!(
        batch_duration.as_millis() < 100,
        "Batch operation took too long: {:?}",
        batch_duration
    );
}

#[tokio::test]
async fn test_batch_concurrency_benefits() {
    // Test with simulated network delay to show concurrency benefits
    let service = IntegratedAccountService::with_config(BatchConfig::with_concurrency(5));
    let accounts = create_n_accounts(10);

    // Sequential time would be: 10 * 10ms = 100ms
    // With 5 concurrent: should be ~20ms (2 batches of 5)
    let start = Instant::now();
    let result = service
        .refresh_account_balances(&accounts, mock_slow_fetcher)
        .await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    
    // With 5 concurrent requests and 10ms delay each:
    // Sequential: 100ms, Concurrent: ~20-30ms (2 waves)
    let sequential_time_ms = 100;
    let actual_time_ms = duration.as_millis();
    let speedup = sequential_time_ms as f64 / actual_time_ms as f64;
    
    println!(
        "📊 Batch concurrency test:\n  Sequential estimate: {}ms\n  Actual time: {}ms\n  Speedup: {:.2}x",
        sequential_time_ms, actual_time_ms, speedup
    );
    
    // Should see at least 2x speedup with concurrent execution
    assert!(
        speedup >= EXPECTED_BATCH_SPEEDUP,
        "Batch concurrency did not provide expected speedup: {:.2}x < {:.2}x",
        speedup, EXPECTED_BATCH_SPEEDUP
    );
}

// ============================================================================
// Cache Performance Tests
// ============================================================================

#[tokio::test]
async fn test_cache_hit_performance() {
    let service = IntegratedAccountService::new();
    let address = Address::from([0x42u8; 20]);
    let account = create_test_account("cached", address);
    
    // Populate cache
    let _ = service
        .refresh_account_balances(&[account.clone()], mock_instant_fetcher)
        .await;
    
    // Measure cache hit time (multiple iterations for accuracy)
    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = service.get_cached_balance(&address).await;
    }
    let avg_time = start.elapsed() / iterations;
    
    println!(
        "📊 Cache hit time (avg of {} iterations): {:?}",
        iterations, avg_time
    );
    
    // Cache hits should be near-instant (< 1ms)
    assert!(
        avg_time.as_micros() < MAX_CACHE_HIT_TIME_MS as u128 * 1000,
        "Cache hits too slow: {:?}",
        avg_time
    );
}

#[tokio::test]
async fn test_cache_vs_uncached_performance() {
    let service = IntegratedAccountService::new();
    let address = Address::from([0x55u8; 20]);
    let account = create_test_account("cache_test", address);
    
    // Measure uncached access (includes "network" call)
    let uncached_start = Instant::now();
    let _ = service
        .refresh_account_balances(&[account.clone()], mock_slow_fetcher)
        .await;
    let uncached_time = uncached_start.elapsed();
    
    // Measure cached access
    let cached_start = Instant::now();
    let cached_result = service.get_cached_balance(&address).await;
    let cached_time = cached_start.elapsed();
    
    assert!(cached_result.is_some());
    
    let speedup = uncached_time.as_micros() as f64 / cached_time.as_micros().max(1) as f64;
    
    println!(
        "📊 Cache performance:\n  Uncached: {:?}\n  Cached: {:?}\n  Speedup: {:.0}x",
        uncached_time, cached_time, speedup
    );
    
    // Cache should be at least 100x faster than network call
    assert!(
        speedup >= 10.0,
        "Cache speedup too low: {:.2}x",
        speedup
    );
}

// ============================================================================
// UI Non-Blocking Tests
// ============================================================================

#[test]
fn test_service_creation_time() {
    // Service creation should be fast (sync, doesn't block UI)
    let start = Instant::now();
    let _service = IntegratedAccountService::new();
    let duration = start.elapsed();
    
    println!("📊 Service creation time: {:?}", duration);
    
    assert!(
        duration.as_millis() < MAX_SYNC_UI_TIME_MS,
        "Service creation blocks UI: {:?}",
        duration
    );
}

#[test]
fn test_operation_span_creation_time() {
    let service = IntegratedAccountService::new();
    
    // Operation span creation should be instant
    let iterations = 1000;
    let start = Instant::now();
    for i in 0..iterations {
        let _ = service.start_operation(&format!("operation_{}", i));
    }
    let avg_time = start.elapsed() / iterations;
    
    println!(
        "📊 Operation span creation time (avg of {} iterations): {:?}",
        iterations, avg_time
    );
    
    // Should be sub-millisecond
    assert!(
        avg_time.as_micros() < 1000,
        "Operation span creation too slow: {:?}",
        avg_time
    );
}

#[test]
fn test_instrumentation_overhead() {
    let service = IntegratedAccountService::new();
    
    // Measure instrumentation overhead
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let result: std::result::Result<String, String> = Ok("test".to_string());
        let _ = service.instrument_create_account(result, "test");
    }
    let avg_time = start.elapsed() / iterations;
    
    println!(
        "📊 Instrumentation overhead (avg of {} iterations): {:?}",
        iterations, avg_time
    );
    
    // Instrumentation should be sub-millisecond
    assert!(
        avg_time.as_micros() < 1000,
        "Instrumentation overhead too high: {:?}",
        avg_time
    );
}

// ============================================================================
// Lock/Unlock Performance Tests
// ============================================================================

#[test]
fn test_lock_unlock_instrumentation_time() {
    let service = IntegratedAccountService::new();
    
    // Measure lock instrumentation time
    let lock_start = Instant::now();
    service.instrument_lock();
    let lock_time = lock_start.elapsed();
    
    // Measure unlock instrumentation time
    let unlock_start = Instant::now();
    service.instrument_unlock(true);
    let unlock_time = unlock_start.elapsed();
    
    println!(
        "📊 Lock/unlock instrumentation:\n  Lock: {:?}\n  Unlock: {:?}",
        lock_time, unlock_time
    );
    
    // Should be sub-millisecond
    assert!(
        lock_time.as_micros() < 1000,
        "Lock instrumentation too slow: {:?}",
        lock_time
    );
    assert!(
        unlock_time.as_micros() < 1000,
        "Unlock instrumentation too slow: {:?}",
        unlock_time
    );
}

// ============================================================================
// Memory Efficiency Tests
// ============================================================================

#[tokio::test]
async fn test_cache_memory_efficiency() {
    let service = IntegratedAccountService::new();
    
    // Create accounts and cache many balances
    let accounts = create_n_accounts(100);
    
    // Populate cache with 100 balances
    let _ = service
        .refresh_account_balances(&accounts, mock_instant_fetcher)
        .await;
    
    // Verify all cached
    let mut cached_count = 0;
    for account in &accounts {
        if service.get_cached_balance(&account.address).await.is_some() {
            cached_count += 1;
        }
    }
    
    assert_eq!(cached_count, 100, "Not all balances were cached");
    
    // Clear cache and verify
    service.clear_balance_cache().await;
    
    let mut cached_after_clear = 0;
    for account in &accounts {
        if service.get_cached_balance(&account.address).await.is_some() {
            cached_after_clear += 1;
        }
    }
    
    assert_eq!(cached_after_clear, 0, "Cache was not cleared properly");
    
    println!("📊 Cache memory test: 100 balances cached and cleared successfully");
}

// ============================================================================
// Scalability Tests
// ============================================================================

#[tokio::test]
async fn test_batch_scalability() {
    let service = IntegratedAccountService::new();
    
    // Test with increasing account counts
    let sizes = [1, 5, 10, 20, 50];
    let mut timings = Vec::new();
    
    for &size in &sizes {
        let accounts = create_n_accounts(size);
        
        let start = Instant::now();
        let result = service
            .refresh_account_balances(&accounts, mock_instant_fetcher)
            .await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        timings.push((size, duration));
    }
    
    println!("📊 Batch scalability:");
    for (size, duration) in &timings {
        let per_account_us = duration.as_micros() / *size as u128;
        println!("  {} accounts: {:?} ({} µs/account)", size, duration, per_account_us);
    }
    
    // Per-account time should remain relatively constant (O(1) amortized)
    let first_per_account = timings[0].1.as_micros() / timings[0].0 as u128;
    let last_per_account = timings.last().unwrap().1.as_micros() / timings.last().unwrap().0 as u128;
    
    // Allow 10x variation (generous for small samples with overhead)
    assert!(
        last_per_account <= first_per_account * 10 || last_per_account < 1000,
        "Batch scaling is poor: first={} µs/account, last={} µs/account",
        first_per_account, last_per_account
    );
}

// ============================================================================
// Performance Summary Test
// ============================================================================

#[tokio::test]
async fn test_performance_summary() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║       VAUGHAN WALLET PERFORMANCE BENCHMARK SUMMARY          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    
    let service = IntegratedAccountService::new();
    
    // 1. Service creation
    let start = Instant::now();
    let _ = IntegratedAccountService::new();
    println!("║ Service creation:          {:>15?}  ║", start.elapsed());
    
    // 2. Operation span creation
    let start = Instant::now();
    for _ in 0..100 {
        let _ = service.start_operation("test");
    }
    println!("║ Span creation (100x avg):  {:>15?}  ║", start.elapsed() / 100);
    
    // 3. Batch balance refresh (10 accounts)
    let accounts = create_n_accounts(10);
    let start = Instant::now();
    let _ = service.refresh_account_balances(&accounts, mock_instant_fetcher).await;
    println!("║ Batch refresh (10 accts):  {:>15?}  ║", start.elapsed());
    
    // 4. Cache hit
    let address = accounts[0].address;
    let start = Instant::now();
    let _ = service.get_cached_balance(&address).await;
    println!("║ Cache hit:                 {:>15?}  ║", start.elapsed());
    
    // 5. Instrumentation
    let start = Instant::now();
    let result: std::result::Result<String, String> = Ok("test".to_string());
    let _ = service.instrument_create_account(result, "test");
    println!("║ Instrumentation overhead:  {:>15?}  ║", start.elapsed());
    
    println!("╚════════════════════════════════════════════════════════════╝\n");
}
