use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use vaughan::wallet::{AccountManager, AccountConfig, AccountManagerTrait, SeedStrength, ImportSource};
use vaughan::performance::batch::{BatchProcessor, BatchConfig};
use secrecy::SecretString;
use alloy::primitives::{Address, U256};
use tokio::runtime::Runtime;

// Benchmark Account Creation (Async)
fn bench_account_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("account_manager_create_seed_based", |b| {
        b.to_async(&rt).iter_batched(
            || {
                // Setup: New manager for each run to avoid state buildup affecting timing
                // In a real app we'd persistent, but here we want to measure just the creation cost
                // which includes PBKDF2 (expensive) and saving (file I/O)
                let temp_dir = tempfile::tempdir().unwrap();
                let manager = AccountManager::new_with_dir(temp_dir.path().to_path_buf());
                (manager, temp_dir) // Keep temp_dir alive
            },
            |mut context| async move {
                let config = AccountConfig::seed_based("Benchmark Wallet")
                    .with_seed_strength(SeedStrength::Words12); // Use 12 words for speed in bench
                let password = SecretString::new("benchmark-password".to_string());
                
                let _ = black_box(context.0.create_account(config, &password).await);
            },
            BatchSize::SmallInput,
        )
    });
}

// Benchmark Batch Processor
fn bench_batch_processor(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Create a processor once
    let processor = BatchProcessor::new(BatchConfig {
        max_concurrent: 10,
        max_retries: 2,
        base_delay_ms: 100,
        max_delay_ms: 400,
        timeout_secs: 5,
    });

    // Create a large list of addresses
    let addresses: Vec<Address> = (0..100)
        .map(|i| {
            let mut bytes = [0u8; 20];
            bytes[19] = i as u8;
            Address::from(bytes)
        })
        .collect();

    c.bench_function("batch_processor_100_items", |b| {
        b.to_async(&rt).iter(|| async {
            let addrs = addresses.clone();
            let _ = processor.batch_balance_queries(addrs, |_| async {
                // Simulate fast network call (1ms)
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                Ok(U256::ZERO)
            }).await;
        })
    });
}

criterion_group!(benches, bench_account_creation, bench_batch_processor);
criterion_main!(benches);
