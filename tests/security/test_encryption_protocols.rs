use vaughan::security::seed::{
    SecureSeedStorage, KeyDerivationAlgorithm, EncryptionAlgorithm, EncryptedSeedDataV2
};
use vaughan::security::keychain::MockKeychain;
use secrecy::SecretString;
use tokio;

/// Test enhanced encryption with versioning
#[tokio::test]
async fn test_enhanced_encryption_v2() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let master_password = SecretString::new("test_master_password_123".to_string());
    
    // Test with default algorithms
    let encrypted_data_v2 = storage.encrypt_seed_phrase_v2(
        &test_phrase,
        &master_password,
        None, // Use default KDF
        None, // Use default encryption
    ).unwrap();
    
    // Verify the encrypted data structure
    assert_eq!(encrypted_data_v2.version, 2);
    assert!(matches!(encrypted_data_v2.kdf_algorithm, KeyDerivationAlgorithm::Pbkdf2Sha256 { .. }));
    assert!(matches!(encrypted_data_v2.encryption_algorithm, EncryptionAlgorithm::Aes256Gcm));
    assert!(!encrypted_data_v2.ciphertext.is_empty());
    assert_ne!(encrypted_data_v2.salt, [0u8; 32]);
    assert_ne!(encrypted_data_v2.nonce, [0u8; 12]);
    assert_ne!(encrypted_data_v2.integrity_hash, [0u8; 32]);
    
    // Test decryption
    let decrypted_phrase = storage.decrypt_seed_phrase_v2(&encrypted_data_v2, &master_password).unwrap();
    assert_eq!(test_phrase.expose_secret(), decrypted_phrase.expose_secret());
}

/// Test different key derivation algorithms
#[tokio::test]
async fn test_key_derivation_algorithms() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "legal winner thank year wave sausage worth useful legal winner thank yellow".to_string()
    );
    let master_password = SecretString::new("secure_password_456".to_string());
    
    // Test PBKDF2-SHA256
    let pbkdf2_kdf = KeyDerivationAlgorithm::Pbkdf2Sha256 { iterations: 100_000 };
    let encrypted_pbkdf2 = storage.encrypt_seed_phrase_v2(
        &test_phrase,
        &master_password,
        Some(pbkdf2_kdf),
        None,
    ).unwrap();
    
    let decrypted_pbkdf2 = storage.decrypt_seed_phrase_v2(&encrypted_pbkdf2, &master_password).unwrap();
    assert_eq!(test_phrase.expose_secret(), decrypted_pbkdf2.expose_secret());
    
    // Test Argon2id
    let argon2_kdf = KeyDerivationAlgorithm::Argon2id {
        memory: 65536,    // 64 MB
        iterations: 3,
        parallelism: 4,
    };
    let encrypted_argon2 = storage.encrypt_seed_phrase_v2(
        &test_phrase,
        &master_password,
        Some(argon2_kdf),
        None,
    ).unwrap();
    
    let decrypted_argon2 = storage.decrypt_seed_phrase_v2(&encrypted_argon2, &master_password).unwrap();
    assert_eq!(test_phrase.expose_secret(), decrypted_argon2.expose_secret());
    
    // Verify that different KDFs produce different ciphertexts
    assert_ne!(encrypted_pbkdf2.ciphertext, encrypted_argon2.ciphertext);
}

/// Test encryption roundtrip with different parameters
#[tokio::test]
async fn test_encryption_roundtrip_variations() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrases = vec![
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "legal winner thank year wave sausage worth useful legal winner thank yellow",
        "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
    ];
    
    let passwords = vec![
        "password123",
        "super_secure_password!@#",
        "简单密码", // Unicode password
    ];
    
    for phrase_str in test_phrases {
        for password_str in &passwords {
            let phrase = SecretString::new(phrase_str.to_string());
            let password = SecretString::new(password_str.to_string());
            
            // Encrypt
            let encrypted = storage.encrypt_seed_phrase_v2(&phrase, &password, None, None).unwrap();
            
            // Decrypt
            let decrypted = storage.decrypt_seed_phrase_v2(&encrypted, &password).unwrap();
            
            // Verify
            assert_eq!(phrase.expose_secret(), decrypted.expose_secret());
        }
    }
}

/// Test integrity verification
#[tokio::test]
async fn test_integrity_verification() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let master_password = SecretString::new("test_password".to_string());
    
    // Encrypt data
    let mut encrypted_data = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, None, None).unwrap();
    
    // Tamper with ciphertext
    encrypted_data.ciphertext[0] ^= 0x01;
    
    // Decryption should fail due to integrity check
    let result = storage.decrypt_seed_phrase_v2(&encrypted_data, &master_password);
    assert!(result.is_err());
    
    // Restore ciphertext and tamper with integrity hash
    encrypted_data.ciphertext[0] ^= 0x01; // Restore
    encrypted_data.integrity_hash[0] ^= 0x01; // Tamper with hash
    
    // Decryption should fail due to integrity check
    let result = storage.decrypt_seed_phrase_v2(&encrypted_data, &master_password);
    assert!(result.is_err());
}

/// Test wrong password handling
#[tokio::test]
async fn test_wrong_password_handling() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let correct_password = SecretString::new("correct_password".to_string());
    let wrong_password = SecretString::new("wrong_password".to_string());
    
    // Encrypt with correct password
    let encrypted_data = storage.encrypt_seed_phrase_v2(&test_phrase, &correct_password, None, None).unwrap();
    
    // Try to decrypt with wrong password - should fail
    let result = storage.decrypt_seed_phrase_v2(&encrypted_data, &wrong_password);
    assert!(result.is_err());
    
    // Decrypt with correct password - should succeed
    let decrypted = storage.decrypt_seed_phrase_v2(&encrypted_data, &correct_password).unwrap();
    assert_eq!(test_phrase.expose_secret(), decrypted.expose_secret());
}

/// Test encryption determinism and uniqueness
#[tokio::test]
async fn test_encryption_uniqueness() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let master_password = SecretString::new("test_password".to_string());
    
    // Encrypt the same data twice
    let encrypted1 = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, None, None).unwrap();
    let encrypted2 = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, None, None).unwrap();
    
    // Ciphertexts should be different due to random nonces
    assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    assert_ne!(encrypted1.nonce, encrypted2.nonce);
    assert_ne!(encrypted1.salt, encrypted2.salt);
    
    // But both should decrypt to the same plaintext
    let decrypted1 = storage.decrypt_seed_phrase_v2(&encrypted1, &master_password).unwrap();
    let decrypted2 = storage.decrypt_seed_phrase_v2(&encrypted2, &master_password).unwrap();
    
    assert_eq!(decrypted1.expose_secret(), decrypted2.expose_secret());
    assert_eq!(test_phrase.expose_secret(), decrypted1.expose_secret());
}

/// Test serialization and deserialization
#[tokio::test]
async fn test_serialization() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "legal winner thank year wave sausage worth useful legal winner thank yellow".to_string()
    );
    let master_password = SecretString::new("serialization_test".to_string());
    
    // Encrypt data
    let encrypted_data = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, None, None).unwrap();
    
    // Serialize to JSON
    let serialized = serde_json::to_string(&encrypted_data).unwrap();
    
    // Deserialize from JSON
    let deserialized: EncryptedSeedDataV2 = serde_json::from_str(&serialized).unwrap();
    
    // Verify all fields are preserved
    assert_eq!(encrypted_data.version, deserialized.version);
    assert_eq!(encrypted_data.ciphertext, deserialized.ciphertext);
    assert_eq!(encrypted_data.salt, deserialized.salt);
    assert_eq!(encrypted_data.nonce, deserialized.nonce);
    assert_eq!(encrypted_data.integrity_hash, deserialized.integrity_hash);
    
    // Verify decryption still works
    let decrypted = storage.decrypt_seed_phrase_v2(&deserialized, &master_password).unwrap();
    assert_eq!(test_phrase.expose_secret(), decrypted.expose_secret());
}

/// Performance test for encryption operations
#[tokio::test]
async fn test_encryption_performance() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let master_password = SecretString::new("performance_test".to_string());
    
    // Test PBKDF2 performance
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _encrypted = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, None, None).unwrap();
    }
    let pbkdf2_duration = start.elapsed();
    
    // Test Argon2id performance
    let argon2_kdf = KeyDerivationAlgorithm::Argon2id {
        memory: 65536,
        iterations: 3,
        parallelism: 4,
    };
    
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _encrypted = storage.encrypt_seed_phrase_v2(
            &test_phrase,
            &master_password,
            Some(argon2_kdf.clone()),
            None,
        ).unwrap();
    }
    let argon2_duration = start.elapsed();
    
    println!("PBKDF2 10 encryptions: {:?}", pbkdf2_duration);
    println!("Argon2id 10 encryptions: {:?}", argon2_duration);
    
    // Both should complete in reasonable time (under 30 seconds for 10 operations)
    assert!(pbkdf2_duration.as_secs() < 30);
    assert!(argon2_duration.as_secs() < 30);
}

/// Test key derivation parameter validation
#[tokio::test]
async fn test_key_derivation_parameters() {
    let keychain = Box::new(MockKeychain::new());
    let storage = SecureSeedStorage::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    let master_password = SecretString::new("parameter_test".to_string());
    
    // Test minimum PBKDF2 iterations
    let min_pbkdf2 = KeyDerivationAlgorithm::Pbkdf2Sha256 { iterations: 100_000 };
    let result = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, Some(min_pbkdf2), None);
    assert!(result.is_ok());
    
    // Test reasonable Argon2id parameters
    let reasonable_argon2 = KeyDerivationAlgorithm::Argon2id {
        memory: 65536,    // 64 MB
        iterations: 3,
        parallelism: 4,
    };
    let result = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, Some(reasonable_argon2), None);
    assert!(result.is_ok());
    
    // Test low Argon2id parameters (should still work but be less secure)
    let low_argon2 = KeyDerivationAlgorithm::Argon2id {
        memory: 1024,     // 1 MB (low but valid)
        iterations: 1,
        parallelism: 1,
    };
    let result = storage.encrypt_seed_phrase_v2(&test_phrase, &master_password, Some(low_argon2), None);
    assert!(result.is_ok());
}

/// Test algorithm defaults
#[tokio::test] 
async fn test_algorithm_defaults() {
    // Test default KDF
    let default_kdf = KeyDerivationAlgorithm::default();
    match default_kdf {
        KeyDerivationAlgorithm::Pbkdf2Sha256 { iterations } => {
            assert_eq!(iterations, 500_000);
        }
        _ => panic!("Default KDF should be PBKDF2-SHA256"),
    }
    
    // Test default encryption
    let default_enc = EncryptionAlgorithm::default();
    assert!(matches!(default_enc, EncryptionAlgorithm::Aes256Gcm));
    
    // Test default EncryptedSeedDataV2
    let default_data = EncryptedSeedDataV2::default();
    assert_eq!(default_data.version, 2);
    assert!(matches!(default_data.kdf_algorithm, KeyDerivationAlgorithm::Pbkdf2Sha256 { .. }));
    assert!(matches!(default_data.encryption_algorithm, EncryptionAlgorithm::Aes256Gcm));
}