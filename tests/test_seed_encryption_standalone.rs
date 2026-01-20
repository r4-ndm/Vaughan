// Integration test for seed encryption functionality
// This tests our professional seed phrase encryption system independently

use secrecy::{ExposeSecret, SecretString};
use vaughan::security::keychain::OSKeychain;
use vaughan::security::seed::{SecureSeedStorage, SeedManager, SeedStrength};

#[tokio::test]
async fn test_professional_seed_encryption_system() {
    println!("🔐 Testing Professional Seed Phrase Encryption System");

    // Initialize components
    let keychain = Box::new(OSKeychain::new("test-service".to_string()).unwrap());
    let seed_manager = SeedManager::new(keychain);
    let storage_keychain = Box::new(OSKeychain::new("test-storage".to_string()).unwrap());
    let secure_storage = SecureSeedStorage::new(storage_keychain);

    println!("✅ Initialized SeedManager and SecureSeedStorage");

    // Test 1: Generate seed phrase
    println!("📱 Test 1: Generate BIP-39 seed phrase");
    let generated_phrase = seed_manager.generate_seed_phrase(SeedStrength::Words12).unwrap();
    println!("✅ Generated 12-word seed phrase");

    // Test 2: Validate seed phrase
    println!("🔍 Test 2: Validate generated seed phrase");
    let validation_result = seed_manager.validate_seed_phrase(&generated_phrase);
    assert!(validation_result.is_ok(), "Seed phrase should be valid");
    println!("✅ Seed phrase validation passed");

    // Test 3: Analyze seed phrase
    println!("📊 Test 3: Analyze seed phrase strength");
    let analysis = seed_manager.analyze_seed_phrase(&generated_phrase).unwrap();
    assert_eq!(analysis.strength, SeedStrength::Words12);
    assert_eq!(analysis.entropy_bits, 128);
    assert!(analysis.is_valid);
    assert!(analysis.security_score() >= 70); // Should be at least 70 for 12 words
    println!(
        "✅ Seed analysis completed - entropy: {} bits, score: {}",
        analysis.entropy_bits,
        analysis.security_score()
    );

    // Test 4: Store encrypted seed phrase (V2 format)
    println!("🔒 Test 4: Store encrypted seed phrase (V2 format)");
    let wallet_id = "test_wallet_seed_encryption";
    let master_password = SecretString::new("super_secure_master_password_123".to_string());

    let key_ref = secure_storage
        .store_encrypted_seed_phrase(wallet_id, &generated_phrase, &master_password)
        .await
        .unwrap();

    assert!(!key_ref.id.is_empty());
    assert_eq!(key_ref.service, "vaughan-wallet-encrypted-seeds");
    assert_eq!(key_ref.account, format!("encrypted-seed-v2-{wallet_id}"));
    println!("✅ Stored seed phrase with enhanced encryption");

    // Test 5: Retrieve and decrypt seed phrase
    println!("🔓 Test 5: Retrieve and decrypt seed phrase");
    let retrieved_phrase = secure_storage
        .retrieve_encrypted_seed_phrase(&key_ref, &master_password)
        .await
        .unwrap();

    // Verify retrieved phrase matches original
    assert_eq!(generated_phrase.expose_secret(), retrieved_phrase.expose_secret());
    println!("✅ Retrieved seed phrase matches original");

    // Test 6: Test with known BIP-39 vector
    println!("🧪 Test 6: Validate known BIP-39 test vector");
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
    );

    seed_manager.validate_seed_phrase(&test_phrase).unwrap();
    println!("✅ Known BIP-39 test vector validated successfully");

    let test_analysis = seed_manager.analyze_seed_phrase(&test_phrase).unwrap();
    assert_eq!(test_analysis.strength, SeedStrength::Words12);
    assert_eq!(test_analysis.entropy_bits, 128);
    assert!(test_analysis.is_valid);
    println!("✅ Test vector analysis completed");

    // Test 7: Store and retrieve test vector with V2 encryption
    println!("🔄 Test 7: Store and retrieve test vector");
    let test_key_ref = secure_storage
        .store_encrypted_seed_phrase("test_vector_wallet", &test_phrase, &master_password)
        .await
        .unwrap();

    let retrieved_test_phrase = secure_storage
        .retrieve_encrypted_seed_phrase(&test_key_ref, &master_password)
        .await
        .unwrap();
    assert_eq!(test_phrase.expose_secret(), retrieved_test_phrase.expose_secret());
    println!("✅ Test vector storage and retrieval successful");

    // Test 8: Test wrong password fails
    println!("🚫 Test 8: Test wrong password fails");
    let wrong_password = SecretString::new("wrong_password".to_string());
    let wrong_result = secure_storage
        .retrieve_encrypted_seed_phrase(&key_ref, &wrong_password)
        .await;
    assert!(wrong_result.is_err(), "Wrong password should fail");
    println!("✅ Wrong password correctly rejected");

    // Test 9: Clean up
    println!("🧹 Test 9: Clean up stored data");
    secure_storage.delete_encrypted_seed_phrase(&key_ref).await.unwrap();
    secure_storage
        .delete_encrypted_seed_phrase(&test_key_ref)
        .await
        .unwrap();
    println!("✅ Cleanup completed successfully");

    println!("🎉 ALL TESTS PASSED!");
    println!("Professional seed encryption system is fully operational!");
}

#[tokio::test]
async fn test_different_seed_strengths() {
    println!("💪 Testing different seed phrase strengths");

    let keychain = Box::new(OSKeychain::new("test-service".to_string()).unwrap());
    let seed_manager = SeedManager::new(keychain);

    let strengths = [
        (SeedStrength::Words12, 128),
        (SeedStrength::Words15, 160),
        (SeedStrength::Words18, 192),
        (SeedStrength::Words21, 224),
        (SeedStrength::Words24, 256),
    ];

    for (strength, expected_entropy) in &strengths {
        println!("Testing {strength:?} (expected entropy: {expected_entropy} bits)");

        let phrase = seed_manager.generate_seed_phrase(*strength).unwrap();
        let analysis = seed_manager.analyze_seed_phrase(&phrase).unwrap();

        assert_eq!(analysis.strength, *strength);
        assert_eq!(analysis.entropy_bits, *expected_entropy);
        assert!(analysis.is_valid);

        // Check word count
        let word_count = phrase.expose_secret().split_whitespace().count();
        match strength {
            SeedStrength::Words12 => assert_eq!(word_count, 12),
            SeedStrength::Words15 => assert_eq!(word_count, 15),
            SeedStrength::Words18 => assert_eq!(word_count, 18),
            SeedStrength::Words21 => assert_eq!(word_count, 21),
            SeedStrength::Words24 => assert_eq!(word_count, 24),
        }

        println!("✅ {strength:?} test passed");
    }

    println!("✅ All seed strength tests passed");
}

#[tokio::test]
async fn test_concurrent_operations() {
    println!("🔄 Testing concurrent seed operations");

    use std::sync::Arc;

    let keychain = Arc::new(OSKeychain::new("test-service".to_string()).unwrap());
    let seed_manager = Arc::new(SeedManager::new(Box::new(
        OSKeychain::new("test-service-concurrent".to_string()).unwrap(),
    )));

    let mut handles = Vec::new();

    // Spawn concurrent seed generation tasks
    for i in 0..5 {
        let manager = seed_manager.clone();

        let handle = tokio::spawn(async move {
            let phrase = manager.generate_seed_phrase(SeedStrength::Words12).unwrap();
            assert!(manager.validate_seed_phrase(&phrase).is_ok());

            let analysis = manager.analyze_seed_phrase(&phrase).unwrap();
            assert_eq!(analysis.strength, SeedStrength::Words12);

            (i, phrase)
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 5);

    // Verify all generated phrases are unique
    let mut phrases = Vec::new();
    for (_, phrase) in results {
        phrases.push(phrase.expose_secret().to_string());
    }

    phrases.sort();
    phrases.dedup();
    assert_eq!(phrases.len(), 5); // All should be unique

    println!("✅ Concurrent operations test passed");
}
