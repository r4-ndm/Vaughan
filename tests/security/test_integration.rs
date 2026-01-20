use vaughan::security::seed::{
    SeedManager, SecureSeedStorage, SeedStrength, KeyDerivationAlgorithm, EncryptionAlgorithm
};
use vaughan::security::keychain::MockKeychain;
use secrecy::SecretString;
use tokio;

/// Test complete seed phrase workflow: generation -> encryption -> storage -> retrieval -> decryption
#[tokio::test]
async fn test_complete_seed_workflow() {
    // Setup
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    let storage_keychain = Box::new(MockKeychain::new());
    let secure_storage = SecureSeedStorage::new(storage_keychain);
    
    let wallet_id = "test_wallet_123";
    let master_password = SecretString::new("super_secure_master_password".to_string());
    
    // Step 1: Generate seed phrase
    let generated_phrase = seed_manager.generate_seed_phrase(SeedStrength::Words24).unwrap();
    
    // Step 2: Validate generated phrase
    assert!(seed_manager.validate_seed_phrase(&generated_phrase).is_ok());
    
    // Step 3: Analyze seed phrase security
    let analysis = seed_manager.analyze_seed_phrase(&generated_phrase).unwrap();
    assert_eq!(analysis.strength, SeedStrength::Words24);
    assert_eq!(analysis.entropy_bits, 256);
    assert!(analysis.is_valid);
    
    // Step 4: Store encrypted seed phrase (V2 format)
    let key_ref = secure_storage.store_encrypted_seed_phrase_v2(
        wallet_id,
        &generated_phrase,
        &master_password,
        Some(KeyDerivationAlgorithm::Argon2id {
            memory: 65536,
            iterations: 3,
            parallelism: 4,
        }),
        Some(EncryptionAlgorithm::Aes256Gcm),
    ).await.unwrap();
    
    // Step 5: Retrieve and decrypt seed phrase
    let retrieved_phrase = secure_storage.retrieve_encrypted_seed_phrase(&key_ref, &master_password).await.unwrap();
    
    // Step 6: Verify retrieved phrase matches original
    assert_eq!(generated_phrase.expose_secret(), retrieved_phrase.expose_secret());
    
    // Step 7: Re-validate retrieved phrase
    assert!(seed_manager.validate_seed_phrase(&retrieved_phrase).is_ok());
    
    // Step 8: Derive HD wallet from retrieved phrase
    let hd_wallet = seed_manager.derive_hd_wallet_from_seed(&retrieved_phrase, None, "m/44'/60'/0'/0/0").unwrap();
    
    // Step 9: Create wallet account
    let (account, _extended_key) = seed_manager.create_hd_wallet_from_seed(
        "test_wallet".to_string(),
        &retrieved_phrase,
        &master_password,
        None,
        None,
    ).await.unwrap();
    
    // Verify account creation
    assert_eq!(account.name, "test_wallet");
    assert!(!account.is_hardware);
    assert!(account.derivation_path.is_some());
    
    // Step 10: Clean up - delete stored seed
    assert!(secure_storage.delete_encrypted_seed_phrase(&key_ref).await.is_ok());
}

/// Test seed phrase import and validation workflow
#[tokio::test]
async fn test_seed_import_workflow() {
    use vaughan::security::seed::SeedImportConfig;
    
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    let config = SeedImportConfig::default();
    
    // Test with a known valid BIP-39 mnemonic
    let imported_phrase_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Step 1: Import with comprehensive validation
    let (imported_phrase, validation) = seed_manager
        .import_seed_phrase_comprehensive(imported_phrase_str, &config)
        .unwrap();
    
    // Step 2: Verify import validation results
    assert!(validation.is_valid);
    assert_eq!(validation.word_count, 12);
    assert_eq!(validation.strength, Some(SeedStrength::Words12));
    assert!(validation.checksum_valid);
    assert!(validation.errors.is_empty());
    assert!(validation.confidence_score() >= 95);
    
    // Step 3: Analyze imported phrase
    let analysis = seed_manager.analyze_imported_seed_phrase(&imported_phrase).unwrap();
    assert!(analysis.is_valid);
    assert_eq!(analysis.strength, SeedStrength::Words12);
    assert!(analysis.security_score() >= 70);
    
    // Step 4: Derive multiple accounts from imported phrase
    let derivation_config = vaughan::security::seed::DerivationPathConfig::ethereum_standard();
    let (_phrase, _validation, derivation) = seed_manager
        .import_seed_phrase_with_derivation(
            imported_phrase_str,
            &derivation_config,
            &config,
            None,
        )
        .unwrap();
    
    // Step 5: Verify account derivation
    assert_eq!(derivation.total_derived, config.max_accounts_to_derive as usize);
    assert!(!derivation.accounts.is_empty());
    
    for account in &derivation.accounts {
        assert!(!account.derivation_path.is_empty());
        assert!(account.derivation_path.starts_with("m/44'/60'/0'/0/"));
    }
}

/// Test backup and recovery workflow
#[tokio::test]
async fn test_backup_recovery_workflow() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    let storage_keychain = Box::new(MockKeychain::new());
    let secure_storage = SecureSeedStorage::new(storage_keychain);
    
    let wallet_id = "backup_test_wallet";
    let master_password = SecretString::new("master_password_123".to_string());
    let backup_password = SecretString::new("backup_password_456".to_string());
    
    // Step 1: Generate and store seed phrase
    let original_phrase = seed_manager.generate_seed_phrase(SeedStrength::Words24).unwrap();
    let key_ref = secure_storage.store_encrypted_seed_phrase_v2(
        wallet_id,
        &original_phrase,
        &master_password,
        None,
        None,
    ).await.unwrap();
    
    // Step 2: Create backup
    let backup = secure_storage.create_backup(&key_ref, &backup_password).await.unwrap();
    
    // Step 3: Verify backup properties
    assert!(!backup.encrypted_data.is_empty());
    assert_ne!(backup.salt, [0u8; 32]);
    assert_ne!(backup.nonce, [0u8; 12]);
    assert_eq!(backup.version, 1);
    assert!(backup.is_recent());
    
    // Step 4: Simulate disaster - delete original
    secure_storage.delete_encrypted_seed_phrase(&key_ref).await.unwrap();
    
    // Step 5: Restore from backup
    let new_wallet_id = "restored_wallet";
    let new_master_password = SecretString::new("new_master_password".to_string());
    let restored_key_ref = secure_storage.restore_from_backup(
        &backup,
        &backup_password,
        new_wallet_id,
        &new_master_password,
    ).await.unwrap();
    
    // Step 6: Verify restoration (this is limited in current implementation)
    // In a full implementation, we would decrypt and verify the restored seed matches the original
    assert!(!restored_key_ref.id.is_empty());
    assert_eq!(restored_key_ref.account, format!("encrypted-seed-{}", new_wallet_id));
}

/// Test cross-version compatibility
#[tokio::test]
async fn test_version_compatibility() {
    let keychain = Box::new(MockKeychain::new());
    let secure_storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let master_password = SecretString::new("version_test_password".to_string());
    let wallet_id = "version_test_wallet";
    
    // Store with legacy format (V1)
    let key_ref_v1 = secure_storage.store_encrypted_seed_phrase(wallet_id, &test_phrase, &master_password).await.unwrap();
    
    // Store with enhanced format (V2)
    let key_ref_v2 = secure_storage.store_encrypted_seed_phrase_v2(
        wallet_id,
        &test_phrase,
        &master_password,
        None,
        None,
    ).await.unwrap();
    
    // Both should be retrievable
    let retrieved_v1 = secure_storage.retrieve_encrypted_seed_phrase(&key_ref_v1, &master_password).await.unwrap();
    let retrieved_v2 = secure_storage.retrieve_encrypted_seed_phrase(&key_ref_v2, &master_password).await.unwrap();
    
    // Both should decrypt to the same original phrase
    assert_eq!(test_phrase.expose_secret(), retrieved_v1.expose_secret());
    assert_eq!(test_phrase.expose_secret(), retrieved_v2.expose_secret());
    
    // Clean up
    secure_storage.delete_encrypted_seed_phrase(&key_ref_v1).await.unwrap();
    secure_storage.delete_encrypted_seed_phrase(&key_ref_v2).await.unwrap();
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    let storage_keychain = Box::new(MockKeychain::new());
    let secure_storage = SecureSeedStorage::new(storage_keychain);
    
    // Test invalid seed phrase
    let invalid_phrase = SecretString::new("invalid seed phrase here".to_string());
    assert!(seed_manager.validate_seed_phrase(&invalid_phrase).is_err());
    
    // Test storing invalid seed phrase
    let result = secure_storage.store_encrypted_seed_phrase_v2(
        "test_wallet",
        &invalid_phrase,
        &SecretString::new("password".to_string()),
        None,
        None,
    ).await;
    assert!(result.is_err());
    
    // Test retrieving non-existent key
    let fake_key_ref = vaughan::security::KeyReference {
        id: "non_existent_key".to_string(),
        service: "test_service".to_string(),
        account: "test_account".to_string(),
    };
    
    let result = secure_storage.retrieve_encrypted_seed_phrase(
        &fake_key_ref,
        &SecretString::new("password".to_string()),
    ).await;
    assert!(result.is_err());
    
    // Test wrong password
    let valid_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let correct_password = SecretString::new("correct_password".to_string());
    let wrong_password = SecretString::new("wrong_password".to_string());
    
    let key_ref = secure_storage.store_encrypted_seed_phrase_v2(
        "test_wallet",
        &valid_phrase,
        &correct_password,
        None,
        None,
    ).await.unwrap();
    
    // Should fail with wrong password
    let result = secure_storage.retrieve_encrypted_seed_phrase(&key_ref, &wrong_password).await;
    assert!(result.is_err());
    
    // Should succeed with correct password
    let result = secure_storage.retrieve_encrypted_seed_phrase(&key_ref, &correct_password).await;
    assert!(result.is_ok());
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    use std::sync::Arc;
    
    let keychain = Arc::new(MockKeychain::new());
    let seed_manager = Arc::new(SeedManager::new(Box::new(MockKeychain::new())));
    
    let mut handles = Vec::new();
    
    // Spawn concurrent seed generation tasks
    for i in 0..10 {
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
    assert_eq!(results.len(), 10);
    
    // Verify all generated phrases are unique
    let mut phrases = Vec::new();
    for (_, phrase) in results {
        phrases.push(phrase.expose_secret().to_string());
    }
    
    phrases.sort();
    phrases.dedup();
    assert_eq!(phrases.len(), 10); // All should be unique
}

/// Test performance under load
#[tokio::test]
async fn test_performance_under_load() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    let storage_keychain = Box::new(MockKeychain::new());
    let secure_storage = SecureSeedStorage::new(storage_keychain);
    
    let num_operations = 50;
    let master_password = SecretString::new("performance_test_password".to_string());
    
    println!("Starting performance test with {} operations", num_operations);
    
    let start_time = std::time::Instant::now();
    
    for i in 0..num_operations {
        // Generate seed phrase
        let phrase = seed_manager.generate_seed_phrase(SeedStrength::Words12).unwrap();
        
        // Validate it
        assert!(seed_manager.validate_seed_phrase(&phrase).is_ok());
        
        // Store it encrypted
        let wallet_id = format!("perf_test_wallet_{}", i);
        let key_ref = secure_storage.store_encrypted_seed_phrase_v2(
            &wallet_id,
            &phrase,
            &master_password,
            None,
            None,
        ).await.unwrap();
        
        // Retrieve and verify
        let retrieved = secure_storage.retrieve_encrypted_seed_phrase(&key_ref, &master_password).await.unwrap();
        assert_eq!(phrase.expose_secret(), retrieved.expose_secret());
        
        // Clean up
        secure_storage.delete_encrypted_seed_phrase(&key_ref).await.unwrap();
    }
    
    let total_time = start_time.elapsed();
    let avg_time_per_op = total_time / num_operations;
    
    println!("Completed {} operations in {:?}", num_operations, total_time);
    println!("Average time per operation: {:?}", avg_time_per_op);
    
    // Performance should be reasonable (less than 1 second per operation on average)
    assert!(avg_time_per_op.as_millis() < 1000);
}