use vaughan::telemetry::opentelemetry::init_telemetry;
use vaughan::wallet::manager::WalletManager;
use tempfile::tempdir;

#[tokio::test]
async fn test_telemetry_initialization() {
    // This test verifies that telemetry initialization code flows correctly.
    // It does NOT verify that data is received by a collector, as that requires external infrastructure.
    
    // Attempt init
    // Note: This might fail if run multiple times in same process due to global subscriber.
    // So we accept Err or Ok, but ensuring it doesn't panic.
    let _ = init_telemetry();
}

#[tokio::test]
async fn test_instrumented_wallet_manager() {
    // Verify that instrumented methods still work correctly
    let dir = tempdir().unwrap();
    let path = dir.path().join("keystore.json");
    let mut manager = WalletManager::new(path);
    
    // Should work without panic even if telemetry is initialized
    let _ = manager.lock();
    assert!(!manager.is_unlocked());
}
