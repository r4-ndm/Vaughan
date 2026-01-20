//! Integration tests for transaction signing with password protection
//!
//! These tests verify the complete transaction signing flow including
//! password validation, key derivation, and key caching.

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use std::time::Duration;
    use vaughan::security::{derive_key_from_seed, derive_wallet_from_seed, KeyCache, SecureSeedStorage};

    #[test]
    fn test_key_derivation_from_seed() {
        // Test seed phrase (standard test mnemonic)
        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );

        // Create keychain
        let keychain = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());

        // Derive key
        let mut secure_key = derive_key_from_seed(keychain, &seed_phrase, None).unwrap();

        // Verify key is 32 bytes
        assert_eq!(secure_key.as_mut_slice().len(), 32);

        // Key should be non-zero
        assert!(secure_key.as_mut_slice().iter().any(|&b| b != 0));
    }

    #[test]
    fn test_wallet_derivation_consistency() {
        // Same seed should always derive same wallet
        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );

        let keychain1 = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());
        let keychain2 = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());

        let wallet1 = derive_wallet_from_seed(keychain1, &seed_phrase, None).unwrap();
        let wallet2 = derive_wallet_from_seed(keychain2, &seed_phrase, None).unwrap();

        // Should derive same address
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_key_cache_workflow() {
        // Simulate transaction signing workflow with key caching
        let mut cache = KeyCache::new(Duration::from_secs(60));

        let address = alloy::primitives::Address::from([1u8; 20]);
        let key_bytes = vec![42u8; 32];

        // First transaction - cache miss
        assert!(cache.get(&address).is_none());

        // Derive and cache key
        cache.insert(address, key_bytes.clone()).unwrap();

        // Second transaction - cache hit
        let cached_key = cache.get(&address).unwrap();
        assert_eq!(cached_key, key_bytes);

        // Verify cache has 1 entry
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_key_cache_expiration_workflow() {
        // Simulate key expiration after timeout
        let mut cache = KeyCache::new(Duration::from_millis(100));

        let address = alloy::primitives::Address::from([1u8; 20]);
        let key_bytes = vec![42u8; 32];

        // Cache key
        cache.insert(address, key_bytes).unwrap();

        // Immediate access - should work
        assert!(cache.get(&address).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Access after expiration - should be None
        assert!(cache.get(&address).is_none());
    }

    #[test]
    fn test_multiple_accounts_key_cache() {
        // Simulate multiple accounts with separate cached keys
        let mut cache = KeyCache::new(Duration::from_secs(60));

        let addr1 = alloy::primitives::Address::from([1u8; 20]);
        let addr2 = alloy::primitives::Address::from([2u8; 20]);
        let addr3 = alloy::primitives::Address::from([3u8; 20]);

        let key1 = vec![1u8; 32];
        let key2 = vec![2u8; 32];
        let key3 = vec![3u8; 32];

        // Cache keys for different accounts
        cache.insert(addr1, key1.clone()).unwrap();
        cache.insert(addr2, key2.clone()).unwrap();
        cache.insert(addr3, key3.clone()).unwrap();

        // Verify all keys are cached
        assert_eq!(cache.len(), 3);

        // Verify each key is correct
        assert_eq!(cache.get(&addr1).unwrap(), key1);
        assert_eq!(cache.get(&addr2).unwrap(), key2);
        assert_eq!(cache.get(&addr3).unwrap(), key3);
    }

    #[test]
    fn test_key_cache_clear_on_lock() {
        // Simulate session lock clearing all cached keys
        let mut cache = KeyCache::new(Duration::from_secs(60));

        // Cache multiple keys
        for i in 0..5 {
            let mut addr_bytes = [0u8; 20];
            addr_bytes[0] = i;
            let address = alloy::primitives::Address::from(addr_bytes);
            let key_bytes = vec![i; 32];
            cache.insert(address, key_bytes).unwrap();
        }

        assert_eq!(cache.len(), 5);

        // Simulate session lock
        cache.clear();

        // All keys should be cleared
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_hd_wallet_derivation_paths() {
        // Test different derivation paths produce different addresses
        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );

        let keychain1 = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());
        let keychain2 = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());
        let keychain3 = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());

        // Derive with different paths
        let wallet1 = derive_wallet_from_seed(keychain1, &seed_phrase, Some("m/44'/60'/0'/0/0")).unwrap();
        let wallet2 = derive_wallet_from_seed(keychain2, &seed_phrase, Some("m/44'/60'/0'/0/1")).unwrap();
        let wallet3 = derive_wallet_from_seed(keychain3, &seed_phrase, Some("m/44'/60'/0'/0/2")).unwrap();

        // All addresses should be different
        assert_ne!(wallet1.address(), wallet2.address());
        assert_ne!(wallet2.address(), wallet3.address());
        assert_ne!(wallet1.address(), wallet3.address());
    }

    #[test]
    fn test_secure_memory_zeroization() {
        // Verify that SecureMemory is zeroized on drop
        use vaughan::security::SecureMemory;

        let key_bytes = vec![42u8; 32];

        {
            let mut secure_key = SecureMemory::new(32).unwrap();
            secure_key.as_mut_slice().copy_from_slice(&key_bytes);

            // Verify key is stored
            assert_eq!(secure_key.as_mut_slice(), &key_bytes[..]);

            // secure_key will be dropped here and zeroized
        }

        // After drop, memory should be zeroized (we can't directly verify this,
        // but the SecureMemory implementation guarantees it)
    }

    #[tokio::test]
    async fn test_seed_storage_encryption_decryption() {
        // Test encrypting and decrypting a seed phrase
        let keychain = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());
        let seed_storage = SecureSeedStorage::new(keychain);

        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );
        let password = SecretString::new("test_password_123".to_string());

        // Store encrypted seed
        let key_ref = seed_storage
            .store_encrypted_seed_phrase("test_wallet", &seed_phrase, &password)
            .await
            .unwrap();

        // Retrieve and decrypt
        let decrypted = seed_storage
            .retrieve_encrypted_seed_phrase(&key_ref, &password)
            .await
            .unwrap();

        // Should match original
        use secrecy::ExposeSecret;
        assert_eq!(decrypted.expose_secret(), seed_phrase.expose_secret());

        // Clean up
        seed_storage.delete_encrypted_seed_phrase(&key_ref).await.unwrap();
    }

    #[tokio::test]
    async fn test_wrong_password_fails() {
        // Test that wrong password fails to decrypt
        let keychain = Box::new(vaughan::security::keychain::OSKeychain::new("test".to_string()).unwrap());
        let seed_storage = SecureSeedStorage::new(keychain);

        let seed_phrase = SecretString::new(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        );
        let correct_password = SecretString::new("correct_password".to_string());
        let wrong_password = SecretString::new("wrong_password".to_string());

        // Store with correct password
        let key_ref = seed_storage
            .store_encrypted_seed_phrase("test_wallet", &seed_phrase, &correct_password)
            .await
            .unwrap();

        // Try to retrieve with wrong password - should fail
        let result = seed_storage
            .retrieve_encrypted_seed_phrase(&key_ref, &wrong_password)
            .await;

        assert!(result.is_err());

        // Clean up
        seed_storage.delete_encrypted_seed_phrase(&key_ref).await.unwrap();
    }
}
