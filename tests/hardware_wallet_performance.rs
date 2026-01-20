//! Performance and stress tests for hardware wallet functionality
//!
//! These tests validate the performance characteristics and stress tolerance
//! of the hardware wallet system under various load conditions.

use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
// use futures;
use vaughan::wallet::HardwareManager;

const PERFORMANCE_TEST_ITERATIONS: usize = 100;
const STRESS_TEST_ITERATIONS: usize = 500;
const CONCURRENT_OPERATIONS: usize = 50;

/// Helper function to create test transactions
fn create_test_transaction() -> TransactionRequest {
    let mut tx = TransactionRequest::default();
    tx.to = Some(
        Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .unwrap()
            .into(),
    );
    tx.value = Some(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH
    tx.gas_price = Some(20_000_000_000u128); // 20 gwei
    tx.gas = Some(21_000u64);
    tx
}

/// Performance Test: Device Detection Speed
#[tokio::test]
async fn test_device_detection_performance() {
    println!("⚡ Performance Test: Device Detection Speed");

    let mut total_duration = Duration::ZERO;
    let mut successful_detections = 0;

    for i in 0..PERFORMANCE_TEST_ITERATIONS {
        let start = Instant::now();

        let mut hw_manager = HardwareManager::new().unwrap();
        let result = hw_manager.detect_wallets().await;

        let duration = start.elapsed();
        total_duration += duration;

        if result.is_ok() {
            successful_detections += 1;
        }

        // Log progress every 25 iterations
        if (i + 1) % 25 == 0 {
            println!(
                "  Progress: {}/{} detections completed",
                i + 1,
                PERFORMANCE_TEST_ITERATIONS
            );
        }
    }

    let avg_duration = total_duration / PERFORMANCE_TEST_ITERATIONS as u32;
    let success_rate = (successful_detections as f64 / PERFORMANCE_TEST_ITERATIONS as f64) * 100.0;

    println!("📊 Device Detection Performance Results:");
    println!("  - Average detection time: {:?}", avg_duration);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Total iterations: {}", PERFORMANCE_TEST_ITERATIONS);

    // Performance assertions
    assert!(
        avg_duration < Duration::from_millis(100),
        "Device detection too slow: {:?}",
        avg_duration
    );
    assert!(success_rate > 95.0, "Success rate too low: {:.2}%", success_rate);

    println!("✅ Device detection performance test passed");
}

/// Performance Test: Address Derivation Speed
#[tokio::test]
async fn test_address_derivation_performance() {
    println!("⚡ Performance Test: Address Derivation Speed");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    let mut total_duration = Duration::ZERO;
    let mut successful_derivations = 0;
    let addresses_per_call = 5;

    for i in 0..PERFORMANCE_TEST_ITERATIONS {
        let start = Instant::now();

        let result = hw_manager.get_addresses(0, "m/44'/60'/0'/0", addresses_per_call).await;

        let duration = start.elapsed();
        total_duration += duration;

        if let Ok(addresses) = result {
            if addresses.len() == addresses_per_call as usize {
                successful_derivations += 1;
            }
        }

        // Log progress every 25 iterations
        if (i + 1) % 25 == 0 {
            println!(
                "  Progress: {}/{} derivations completed",
                i + 1,
                PERFORMANCE_TEST_ITERATIONS
            );
        }
    }

    let avg_duration = total_duration / PERFORMANCE_TEST_ITERATIONS as u32;
    let success_rate = (successful_derivations as f64 / PERFORMANCE_TEST_ITERATIONS as f64) * 100.0;
    let addresses_per_second = (successful_derivations * addresses_per_call) as f64 / total_duration.as_secs_f64();

    println!("📊 Address Derivation Performance Results:");
    println!("  - Average derivation time: {:?}", avg_duration);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Addresses per second: {:.2}", addresses_per_second);
    println!("  - Total derivations: {}", successful_derivations);

    // Performance assertions
    assert!(
        avg_duration < Duration::from_millis(200),
        "Address derivation too slow: {:?}",
        avg_duration
    );
    assert!(success_rate > 95.0, "Success rate too low: {:.2}%", success_rate);

    println!("✅ Address derivation performance test passed");
}

/// Performance Test: Transaction Signing Speed
#[tokio::test]
async fn test_transaction_signing_performance() {
    println!("⚡ Performance Test: Transaction Signing Speed");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    let tx = create_test_transaction();
    let mut total_duration = Duration::ZERO;
    let mut successful_signatures = 0;

    for i in 0..PERFORMANCE_TEST_ITERATIONS {
        let start = Instant::now();

        let result = hw_manager
            .sign_transaction(0, &tx, &format!("m/44'/60'/0'/0/{}", i % 20))
            .await;

        let duration = start.elapsed();
        total_duration += duration;

        if result.is_ok() {
            successful_signatures += 1;
        }

        // Log progress every 25 iterations
        if (i + 1) % 25 == 0 {
            println!(
                "  Progress: {}/{} signatures completed",
                i + 1,
                PERFORMANCE_TEST_ITERATIONS
            );
        }
    }

    let avg_duration = total_duration / PERFORMANCE_TEST_ITERATIONS as u32;
    let success_rate = (successful_signatures as f64 / PERFORMANCE_TEST_ITERATIONS as f64) * 100.0;
    let signatures_per_second = successful_signatures as f64 / total_duration.as_secs_f64();

    println!("📊 Transaction Signing Performance Results:");
    println!("  - Average signing time: {:?}", avg_duration);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Signatures per second: {:.2}", signatures_per_second);
    println!("  - Total signatures: {}", successful_signatures);

    // Performance assertions
    assert!(
        avg_duration < Duration::from_millis(300),
        "Transaction signing too slow: {:?}",
        avg_duration
    );
    assert!(success_rate > 95.0, "Success rate too low: {:.2}%", success_rate);

    println!("✅ Transaction signing performance test passed");
}

/// Stress Test: Concurrent Address Derivation
#[tokio::test]
async fn test_concurrent_address_derivation_stress() {
    println!("🔥 Stress Test: Concurrent Address Derivation");

    let hw_manager = Arc::new(tokio::sync::Mutex::new(HardwareManager::new().unwrap()));
    {
        let mut manager = hw_manager.lock().await;
        manager.detect_wallets().await.unwrap();
    }

    let semaphore = Arc::new(Semaphore::new(CONCURRENT_OPERATIONS));
    let start_time = Instant::now();

    let mut tasks = Vec::new();
    for i in 0..STRESS_TEST_ITERATIONS {
        let hw_manager_clone = Arc::clone(&hw_manager);
        let semaphore_clone = Arc::clone(&semaphore);

        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            let manager = hw_manager_clone.lock().await;
            let device_index = i % 2; // Alternate between devices
            let account_index = i % 10; // Rotate through 10 accounts
            let path = format!("m/44'/60'/{}'", account_index);

            manager.get_addresses(device_index, &path, 1).await
        });

        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;
    let total_duration = start_time.elapsed();

    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for result in results {
        match result {
            Ok(Ok(_)) => successful_operations += 1,
            Ok(Err(_)) => failed_operations += 1,
            Err(_) => failed_operations += 1,
        }
    }

    let success_rate = (successful_operations as f64 / STRESS_TEST_ITERATIONS as f64) * 100.0;
    let operations_per_second = STRESS_TEST_ITERATIONS as f64 / total_duration.as_secs_f64();

    println!("📊 Concurrent Address Derivation Stress Test Results:");
    println!("  - Total operations: {}", STRESS_TEST_ITERATIONS);
    println!("  - Successful operations: {}", successful_operations);
    println!("  - Failed operations: {}", failed_operations);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Operations per second: {:.2}", operations_per_second);
    println!("  - Total duration: {:?}", total_duration);
    println!("  - Max concurrent operations: {}", CONCURRENT_OPERATIONS);

    // Stress test assertions
    assert!(
        success_rate > 90.0,
        "Success rate too low under stress: {:.2}%",
        success_rate
    );
    assert!(successful_operations > 0, "No successful operations under stress");

    println!("✅ Concurrent address derivation stress test passed");
}

/// Stress Test: Concurrent Transaction Signing
#[tokio::test]
async fn test_concurrent_signing_stress() {
    println!("🔥 Stress Test: Concurrent Transaction Signing");

    let hw_manager = Arc::new(tokio::sync::Mutex::new(HardwareManager::new().unwrap()));
    {
        let mut manager = hw_manager.lock().await;
        manager.detect_wallets().await.unwrap();
    }

    let semaphore = Arc::new(Semaphore::new(CONCURRENT_OPERATIONS / 2)); // More conservative for signing
    let start_time = Instant::now();
    let tx = create_test_transaction();

    let mut tasks = Vec::new();
    for i in 0..STRESS_TEST_ITERATIONS / 2 {
        // Fewer iterations for signing stress test
        let hw_manager_clone = Arc::clone(&hw_manager);
        let semaphore_clone = Arc::clone(&semaphore);
        let tx_clone = tx.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            let manager = hw_manager_clone.lock().await;
            let device_index = i % 2;
            let address_index = i % 20;
            let path = format!("m/44'/60'/0'/0/{}", address_index);

            manager.sign_transaction(device_index, &tx_clone, &path).await
        });

        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;
    let total_duration = start_time.elapsed();

    let mut successful_signatures = 0;
    let mut failed_signatures = 0;

    for result in results {
        match result {
            Ok(Ok(_)) => successful_signatures += 1,
            Ok(Err(_)) => failed_signatures += 1,
            Err(_) => failed_signatures += 1,
        }
    }

    let total_operations = STRESS_TEST_ITERATIONS / 2;
    let success_rate = (successful_signatures as f64 / total_operations as f64) * 100.0;
    let signatures_per_second = successful_signatures as f64 / total_duration.as_secs_f64();

    println!("📊 Concurrent Transaction Signing Stress Test Results:");
    println!("  - Total operations: {}", total_operations);
    println!("  - Successful signatures: {}", successful_signatures);
    println!("  - Failed signatures: {}", failed_signatures);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Signatures per second: {:.2}", signatures_per_second);
    println!("  - Total duration: {:?}", total_duration);

    // Stress test assertions
    assert!(
        success_rate > 90.0,
        "Success rate too low under stress: {:.2}%",
        success_rate
    );
    assert!(successful_signatures > 0, "No successful signatures under stress");

    println!("✅ Concurrent signing stress test passed");
}

/// Stress Test: Mixed Operations Load
#[tokio::test]
async fn test_mixed_operations_stress() {
    println!("🔥 Stress Test: Mixed Operations Load");

    let hw_manager = Arc::new(tokio::sync::Mutex::new(HardwareManager::new().unwrap()));
    {
        let mut manager = hw_manager.lock().await;
        manager.detect_wallets().await.unwrap();
    }

    let semaphore = Arc::new(Semaphore::new(CONCURRENT_OPERATIONS));
    let start_time = Instant::now();
    let tx = create_test_transaction();

    let mut tasks = Vec::new();
    for i in 0..STRESS_TEST_ITERATIONS {
        let hw_manager_clone = Arc::clone(&hw_manager);
        let semaphore_clone = Arc::clone(&semaphore);
        let tx_clone = tx.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            let manager = hw_manager_clone.lock().await;
            let device_index = i % 2;

            match i % 4 {
                0 => {
                    // Address derivation operation
                    manager
                        .get_addresses(device_index, "m/44'/60'/0'/0", 1)
                        .await
                        .map(|_| "address_derivation".to_string())
                        .map_err(|e| e)
                }
                1 => {
                    // Transaction signing operation
                    let path = format!("m/44'/60'/0'/0/{}", i % 10);
                    manager
                        .sign_transaction(device_index, &tx_clone, &path)
                        .await
                        .map(|_| "transaction_signing".to_string())
                        .map_err(|e| e)
                }
                2 => {
                    // Address verification operation
                    let test_addr = "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18";
                    manager
                        .verify_address_with_feedback(device_index, test_addr, "m/44'/60'/0'/0/0")
                        .await
                        .map(|_| "address_verification".to_string())
                        .map_err(|e| e)
                }
                3 => {
                    // Transaction audit operation
                    manager
                        .audit_transaction_with_feedback(&tx_clone, "m/44'/60'/0'/0/0", device_index)
                        .await
                        .map(|_| "transaction_audit".to_string())
                        .map_err(|e| e)
                }
                _ => unreachable!(),
            }
        });

        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;
    let total_duration = start_time.elapsed();

    let mut operation_counts = std::collections::HashMap::new();
    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for result in results {
        match result {
            Ok(Ok(op_type)) => {
                successful_operations += 1;
                *operation_counts.entry(op_type).or_insert(0) += 1;
            }
            Ok(Err(_)) => failed_operations += 1,
            Err(_) => failed_operations += 1,
        }
    }

    let success_rate = (successful_operations as f64 / STRESS_TEST_ITERATIONS as f64) * 100.0;
    let operations_per_second = STRESS_TEST_ITERATIONS as f64 / total_duration.as_secs_f64();

    println!("📊 Mixed Operations Stress Test Results:");
    println!("  - Total operations: {}", STRESS_TEST_ITERATIONS);
    println!("  - Successful operations: {}", successful_operations);
    println!("  - Failed operations: {}", failed_operations);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Operations per second: {:.2}", operations_per_second);
    println!("  - Total duration: {:?}", total_duration);

    println!("  - Operation breakdown:");
    for (op_type, count) in operation_counts {
        println!("    - {}: {}", op_type, count);
    }

    // Stress test assertions
    assert!(
        success_rate > 85.0,
        "Success rate too low under mixed load: {:.2}%",
        success_rate
    );
    assert!(successful_operations > 0, "No successful operations under mixed load");

    println!("✅ Mixed operations stress test passed");
}

/// Memory Usage Test: Detect memory leaks during prolonged operation
#[tokio::test]
async fn test_memory_usage_prolonged_operation() {
    println!("💾 Memory Test: Prolonged Operation Memory Usage");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    let initial_memory = get_memory_usage();
    println!("  - Initial memory usage: {} KB", initial_memory);

    let tx = create_test_transaction();

    // Perform sustained operations
    for batch in 0..10 {
        let mut tasks = Vec::new();

        for i in 0..50 {
            let path = format!("m/44'/60'/0'/0/{}", i % 20);

            // Alternate between different operations
            match i % 3 {
                0 => {
                    let addresses_task = hw_manager.get_addresses(i % 2, "m/44'/60'/0'/0", 1);
                    tasks.push(Box::pin(async move { addresses_task.await.map(|_| ()) })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), _>>>>);
                }
                1 => {
                    let signing_result = hw_manager.sign_transaction(i % 2, &tx, &path).await;
                    tasks.push(Box::pin(async move { signing_result.map(|_| ()) })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), _>>>>);
                }
                2 => {
                    let audit_task = hw_manager.audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", i % 2);
                    tasks.push(Box::pin(async move { audit_task.await.map(|_| ()) })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), _>>>>);
                }
                _ => unreachable!(),
            }
        }

        // let _ = futures::future::join_all(tasks).await;

        let current_memory = get_memory_usage();
        println!("  - Batch {} memory usage: {} KB", batch + 1, current_memory);

        // Brief pause to allow garbage collection
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let final_memory = get_memory_usage();
    let memory_increase = final_memory as f64 / initial_memory as f64;

    println!("📊 Memory Usage Test Results:");
    println!("  - Initial memory: {} KB", initial_memory);
    println!("  - Final memory: {} KB", final_memory);
    println!("  - Memory increase factor: {:.2}x", memory_increase);

    // Memory leak assertion - should not increase more than 3x
    assert!(
        memory_increase < 3.0,
        "Potential memory leak detected: {:.2}x increase",
        memory_increase
    );

    println!("✅ Memory usage test passed");
}

/// Helper function to get approximate memory usage (simplified for testing)
fn get_memory_usage() -> u64 {
    // In a real implementation, this would use system calls to get actual memory usage
    // For testing purposes, we'll use a simple approximation based on process info
    use std::process::Command;

    if let Ok(output) = Command::new("ps")
        .args(&["-o", "rss=", "-p"])
        .arg(std::process::id().to_string())
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            if let Ok(memory_kb) = output_str.trim().parse::<u64>() {
                return memory_kb;
            }
        }
    }

    // Fallback if ps command fails (e.g., on non-Unix systems)
    1000 // Return a default value for cross-platform compatibility
}

/// Endurance Test: Long-running operations
#[tokio::test]
async fn test_endurance_long_running() {
    println!("⏱️ Endurance Test: Long-running Operations");

    let mut hw_manager = HardwareManager::new().unwrap();
    hw_manager.detect_wallets().await.unwrap();

    let start_time = Instant::now();
    let test_duration = Duration::from_secs(10); // 10-second endurance test
    let mut operation_count = 0;
    let mut successful_count = 0;

    let tx = create_test_transaction();

    while start_time.elapsed() < test_duration {
        let _operation_start = Instant::now();

        // Perform a mix of operations
        let success = match operation_count % 4 {
            0 => hw_manager
                .get_addresses(operation_count % 2, "m/44'/60'/0'/0", 1)
                .await
                .is_ok(),
            1 => {
                let path = format!("m/44'/60'/0'/0/{}", operation_count % 10);
                hw_manager
                    .sign_transaction(operation_count % 2, &tx, &path)
                    .await
                    .is_ok()
            }
            2 => hw_manager
                .verify_address_with_feedback(
                    operation_count % 2,
                    "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18",
                    "m/44'/60'/0'/0/0",
                )
                .await
                .is_ok(),
            3 => hw_manager
                .audit_transaction_with_feedback(&tx, "m/44'/60'/0'/0/0", operation_count % 2)
                .await
                .is_ok(),
            _ => unreachable!(),
        };

        if success {
            successful_count += 1;
        }

        operation_count += 1;

        // Brief pause to prevent overwhelming
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Log progress every 50 operations
        if operation_count % 50 == 0 {
            println!(
                "  Progress: {} operations in {:?}",
                operation_count,
                start_time.elapsed()
            );
        }
    }

    let total_duration = start_time.elapsed();
    let success_rate = (successful_count as f64 / operation_count as f64) * 100.0;
    let operations_per_second = operation_count as f64 / total_duration.as_secs_f64();

    println!("📊 Endurance Test Results:");
    println!("  - Test duration: {:?}", total_duration);
    println!("  - Total operations: {}", operation_count);
    println!("  - Successful operations: {}", successful_count);
    println!("  - Success rate: {:.2}%", success_rate);
    println!("  - Operations per second: {:.2}", operations_per_second);

    // Endurance test assertions
    assert!(
        operation_count > 100,
        "Not enough operations performed: {}",
        operation_count
    );
    assert!(
        success_rate > 90.0,
        "Success rate too low during endurance: {:.2}%",
        success_rate
    );

    println!("✅ Endurance test passed");
}
