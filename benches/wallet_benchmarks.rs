use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vaughan::wallet::manager::WalletManager;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use alloy::primitives::U256;
use alloy::sol_types::SolValue;
use alloy::sol;
// Removed unused imports to reduce warnings
use secrecy::SecretString;

// Define Multicall3 interface for benchmarking
sol! {
    struct Result {
        bool success;
        bytes returnData;
    }
}

fn benchmark_account_creation(c: &mut Criterion) {
    c.bench_function("create_wallet_pbkdf2", |b| {
        b.iter(|| {
            // Setup temp directory for each iteration to ensure fresh creation
            let temp_dir = tempfile::tempdir().unwrap();
            let path = temp_dir.path().join("keystore.json");
            
            let mut manager = WalletManager::new(path);
            let password = SecretString::new("benchmark-password".to_string());
            
            // This runs PBKDF2 (262,144 iterations)
            let _ = black_box(manager.create_wallet(password));
        })
    });
}

fn benchmark_signing(c: &mut Criterion) {
    // Benchmark raw signer performance as baseline
    let signer = PrivateKeySigner::random();
    let msg = "Hello World".as_bytes();

    c.bench_function("sign_message_secp256k1", |b| {
        b.iter(|| {
            let _ = black_box(signer.sign_message_sync(msg));
        })
    });
}

fn benchmark_multicall_decoding(c: &mut Criterion) {
    let mut results = Vec::new();
    for i in 0..100 {
        let balance = U256::from(i * 1000);
        results.push(Result {
            success: true,
            returnData: balance.abi_encode().into(),
        });
    }
    
    let encoded = results.abi_encode();

    c.bench_function("decode_multicall_100_balances", |b| {
        b.iter(|| {
            let decoded: Vec<Result> = Vec::<Result>::abi_decode(&encoded).unwrap();
            black_box(decoded);
        })
    });
}

criterion_group!(benches, benchmark_account_creation, benchmark_signing, benchmark_multicall_decoding);
criterion_main!(benches);
