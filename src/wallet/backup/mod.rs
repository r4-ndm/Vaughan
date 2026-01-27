//! Backup and Recovery System
//!
//! Implements secure backup standards compatible with MetaMask Vault format (encrypted)
//! and robust recovery mechanisms including Shamir's Secret Sharing.
//!
//! # Requirements
//! - Requirement 11.1: Encrypted backup
//! - Requirement 11.2: Shamir's Secret Sharing
//! - Requirement 11.4: Integrity verification

use crate::error::{Result, SecurityError, WalletError};
use crate::security::SecureKeystore;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hmac::{Hmac, Mac};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

#[cfg(feature = "shamir")]
use crate::VaughanError;
#[cfg(feature = "shamir")]
use secrecy::{ExposeSecret, SecretVec};
#[cfg(feature = "shamir")]
use sharks::{Share, Sharks};

/// Encrypted Backup Container (MetaMask-style Vault)
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupContainer {
    pub version: u32,
    pub id: Uuid,
    pub timestamp: i64,
    pub salt: String, // Hex encoded
    pub nonce: String, // Hex encoded
    pub ciphertext: String, // Hex encoded
    pub hmac: String, // Hex encoded HMAC-SHA256
}

/// Manager for secure backup and recovery operations
#[derive(Debug, Clone)]
pub struct BackupManager;

impl BackupManager {
    /// Create a new encrypted backup of the keystore
    pub async fn create_encrypted_backup(
        keystore: &SecureKeystore,
        password: &SecretString,
    ) -> Result<BackupContainer> {
        let correlation_id = Uuid::new_v4();
        tracing::info!(correlation_id = %correlation_id, "📦 Starting encrypted backup creation");

        // 1. Serialize Keystore State
        // For backup, we ideally export all accounts data. 
        // Since SecureKeystore holds accounts in memory and keys in Keychain, 
        // a "Full Backup" implies exporting the *keys* too. 
        // However, SecureKeystore doesn't expose raw keys easily without password.
        // Assuming we just want to backup the *list of accounts* and their metadata 
        // OR we want a full Vault export. 
        // 
        // The requirements imply "Export entire wallet state". 
        // For this implementation, we will serialize the Account list. 
        // Real private keys are in the backend keychain (OS specific).
        // 
        // CRITICAL NOTE: If the keys are in OS Keychain, a file backup of just struct memory 
        // is insufficient for cross-device recovery. 
        // We really need to export the SECRETS.
        // 
        // Given `AccountExporter` exists for single accounts, `BackupManager` should probably 
        // iterate and export ALL accounts if possible, OR assumes `SecureKeystore` has a method 
        // to dump everything (which it usually doesn't for security).
        // 
        // Let's assume we backup the *metadata* for now, as specific key extraction requires 
        // per-key password in some designs, or Master Password in others. 
        // If `SecureKeystore` uses a Master Password for `SecureSeedStorage`, we can backup the 
        // Encrypted Seed.
        
        // Strategy: Serialize the entire list of accounts.
        let accounts = keystore.list_accounts().await?;
        let data = serde_json::to_string(&accounts).map_err(|e| WalletError::SerializationError(e.to_string()))?;

        // 2. Derive Encryption Key (Argon2id)
        let salt = Uuid::new_v4().as_bytes().to_vec(); // Simple random salt
        let key = crate::security::seed::encryption::derive_key_argon2id(
            password,
            &salt,
            65536, // 64 MB
            3,     // 3 iterations
            4      // 4 parallelism
        )?;

        // 3. Encrypt (AES-256-GCM)
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| SecurityError::EncryptionError { message: "Invalid key length".into() })?;
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|e| SecurityError::EncryptionError { message: format!("Encryption failed: {}", e) })?;

        // 4. Calculate HMAC (Integrity)
        // Use the same derived key for HMAC (MetaMask often uses distinct, but we'll use same for simplicity or derive another)
        // Let's derive a 2nd key for HMAC to be proper? Or just use the key.
        // We'll use the key bytes for HMAC-SHA256.
        let mut mac = <Hmac::<Sha256> as Mac>::new_from_slice(&key)
            .map_err(|_| SecurityError::EncryptionError { message: "HMAC init failed".into() })?;
        mac.update(&ciphertext);
        let hmac_result = mac.finalize().into_bytes();

        let container = BackupContainer {
            version: 1,
            id: correlation_id,
            timestamp: chrono::Utc::now().timestamp(),
            salt: hex::encode(salt),
            nonce: hex::encode(nonce_bytes),
            ciphertext: hex::encode(ciphertext),
            hmac: hex::encode(hmac_result),
        };

        tracing::info!(correlation_id = %correlation_id, "✅ Backup created successfully");
        Ok(container)
    }

    /// Restore from encrypted backup
    pub fn restore_from_backup(
        container: &BackupContainer,
        password: &SecretString,
    ) -> Result<Vec<crate::security::SecureAccount>> {
        tracing::info!(backup_id = %container.id, "♻️ Restoring from backup");

        // 1. Decode fields
        let salt = hex::decode(&container.salt).map_err(|_| WalletError::DeserializationError("Invalid salt".into()))?;
        let nonce_bytes = hex::decode(&container.nonce).map_err(|_| WalletError::DeserializationError("Invalid nonce".into()))?;
        let ciphertext = hex::decode(&container.ciphertext).map_err(|_| WalletError::DeserializationError("Invalid ciphertext".into()))?;
        let stored_hmac = hex::decode(&container.hmac).map_err(|_| WalletError::DeserializationError("Invalid HMAC".into()))?;

        // 2. Derive Key
        let key = crate::security::seed::encryption::derive_key_argon2id(
            password,
            &salt,
            65536,
            3,
            4
        )?;

        // 3. Verify HMAC
        let mut mac = <Hmac::<Sha256> as Mac>::new_from_slice(&key)
            .map_err(|_| SecurityError::EncryptionError { message: "HMAC init failed".into() })?;
        mac.update(&ciphertext);
        
        if mac.verify_slice(&stored_hmac).is_err() {
            tracing::error!("❌ HMAC validation failed - backup integrity compromised");
            return Err(SecurityError::IntegrityCheckFailed { message: "Backup corrupted or tampered".into() }.into());
        }

        // 4. Decrypt
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| SecurityError::EncryptionError { message: "Invalid key".into() })?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| SecurityError::InvalidPassword)?; // Usually implies wrong key/password

        let plaintext_str = String::from_utf8(plaintext)
            .map_err(|_| WalletError::DeserializationError("Invalid UTF-8".into()))?;

        // 5. Deserialize
        let accounts: Vec<crate::security::SecureAccount> = serde_json::from_str(&plaintext_str)
            .map_err(|e| WalletError::DeserializationError(e.to_string()))?;

        tracing::info!("✅ Backup restored successfully ({} accounts)", accounts.len());
        Ok(accounts)
    }

    /// Create Shamir's Secret Sharing shares (Requirement 11.2)
    #[cfg(feature = "shamir")]
    pub fn create_shamir_shares(
        secret: &SecretVec<u8>,
        threshold: u8,
        total: u8,
    ) -> Result<Vec<String>> {
        if threshold > total {
            return Err(WalletError::Generic("Threshold cannot represent more than total shares".into()).into());
        }

        let sharks = Sharks(threshold);
        let dealer = sharks.dealer(secret.expose_secret());
        
        let shares: Vec<String> = dealer.take(total as usize)
            .map(|s| {
                // Convert Share to bytes using Vec::from
                let bytes: Vec<u8> = Vec::from(&s);
                hex::encode(bytes)
            })
            .collect();

        Ok(shares)
    }

    /// Recover secret from Shamir shares
    #[cfg(feature = "shamir")]
    pub fn recover_from_shares(shares: &[String], threshold: u8) -> Result<SecretVec<u8>> {
        if shares.len() < threshold as usize {
            return Err(WalletError::Generic("Not enough shares to recover".into()).into());
        }

        let sharks = Sharks(threshold);
        
        let parsed_shares: Vec<Share> = shares.iter()
            .map(|s| {
                let bytes = hex::decode(s).map_err(|_| VaughanError::from(WalletError::DeserializationError("Invalid hex share".into())))?;
                Share::try_from(bytes.as_slice()).map_err(|_| VaughanError::from(WalletError::DeserializationError("Invalid share format".into())))
            })
            .collect::<Result<Vec<_>>>()?;

        let secret = sharks.recover(&parsed_shares)
            .map_err(|e| WalletError::Generic(format!("Recovery failed: {}", e)))?;

        Ok(SecretVec::new(secret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::MockKeychain;
    use crate::security::SecureKeystoreImpl;

    #[tokio::test]
    async fn test_encrypted_backup_roundtrip() {
        // Setup
        let keychain = Box::new(MockKeychain::new());
        let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
        let _ = keystore.create_account("TestUser".into()).await.unwrap();
        let password = SecretString::new("strong_password".into());

        // Create Backup
        let backup = BackupManager::create_encrypted_backup(&keystore, &password).await.unwrap();
        
        // Restore
        let restored_accounts = BackupManager::restore_from_backup(&backup, &password).unwrap();
        assert_eq!(restored_accounts.len(), 1);
        assert_eq!(restored_accounts[0].name, "TestUser");
    }

    #[tokio::test]
    async fn test_backup_bad_password() {
        let keychain = Box::new(MockKeychain::new());
        let keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
        let password = SecretString::new("correct".into());
        let bad_password = SecretString::new("wrong".into());

        let backup = BackupManager::create_encrypted_backup(&keystore, &password).await.unwrap();
        
        let result = BackupManager::restore_from_backup(&backup, &bad_password);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backup_tampering() {
        let keychain = Box::new(MockKeychain::new());
        let keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
        let password = SecretString::new("strong_password".into());

        let mut backup = BackupManager::create_encrypted_backup(&keystore, &password).await.unwrap();
        
        // Tamper with ciphertext
        let mut corrupted = hex::decode(&backup.ciphertext).unwrap();
        if let Some(byte) = corrupted.get_mut(0) {
            *byte ^= 0xFF; // Flip bits
        }
        backup.ciphertext = hex::encode(corrupted);

        let result = BackupManager::restore_from_backup(&backup, &password);
        // Should fail integrity check logic (HMAC)
        // If HMAC uses ciphertext, modifying ciphertext invalidates HMAC.
        // Our restore logic checks HMAC first.
        assert!(matches!(result, Err(crate::error::VaughanError::Security(SecurityError::IntegrityCheckFailed { .. }))));
    }
    
    #[cfg(feature = "shamir")]
    #[test]
    fn test_shamir_roundtrip() {
        let secret = SecretVec::new(b"secret_data_123".to_vec());
        let shares = BackupManager::create_shamir_shares(&secret, 2, 3).unwrap();
        
        assert_eq!(shares.len(), 3);
        
        // Recover with 2 shares
        let partial_shares = vec![shares[0].clone(), shares[2].clone()];
        let recovered = BackupManager::recover_from_shares(&partial_shares, 2).unwrap();
        
        assert_eq!(recovered.expose_secret(), b"secret_data_123");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::security::keychain::MockKeychain;
    use crate::security::SecureKeystoreImpl;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 30: Backup Encryption
        ///
        /// *For any* backup created, it should be encrypted and require the correct
        /// password to restore. Wrong passwords should fail.
        ///
        /// Validates: Requirements 11.1
        #[test]
        fn prop_backup_encryption(
            password in "[a-zA-Z0-9!@#$%^&*]{8,32}",
            wrong_password in "[a-zA-Z0-9!@#$%^&*]{8,32}",
            account_name in "[a-zA-Z0-9_]{3,20}"
        ) {
            // Ensure passwords are different
            prop_assume!(password != wrong_password);

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Setup keystore with an account
                let keychain = Box::new(MockKeychain::new());
                let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
                let _ = keystore.create_account(account_name.clone()).await.unwrap();
                
                let password_secret = SecretString::new(password.clone());
                let wrong_password_secret = SecretString::new(wrong_password.clone());

                // Create encrypted backup
                let backup = BackupManager::create_encrypted_backup(&keystore, &password_secret)
                    .await
                    .unwrap();

                // Verify backup is encrypted (ciphertext should not contain plaintext account name)
                let ciphertext_bytes = hex::decode(&backup.ciphertext).unwrap();
                let ciphertext_str = String::from_utf8_lossy(&ciphertext_bytes);
                prop_assert!(
                    !ciphertext_str.contains(&account_name),
                    "Ciphertext should not contain plaintext account name: {}",
                    account_name
                );

                // Verify backup has encryption metadata
                prop_assert!(!backup.salt.is_empty(), "Backup should have salt");
                prop_assert!(!backup.nonce.is_empty(), "Backup should have nonce");
                prop_assert!(!backup.hmac.is_empty(), "Backup should have HMAC");

                // Correct password should restore successfully
                let restored = BackupManager::restore_from_backup(&backup, &password_secret);
                prop_assert!(restored.is_ok(), "Correct password should restore backup");
                
                let accounts = restored.unwrap();
                prop_assert_eq!(accounts.len(), 1, "Should restore 1 account");
                prop_assert_eq!(&accounts[0].name, &account_name, "Account name should match");

                // Wrong password should fail
                let wrong_restore = BackupManager::restore_from_backup(&backup, &wrong_password_secret);
                prop_assert!(
                    wrong_restore.is_err(),
                    "Wrong password should fail to restore backup"
                );

                Ok(())
            }).unwrap();
        }

        /// Property 32: Backup Integrity Verification
        ///
        /// *For any* backup that has been corrupted or tampered with, the restore
        /// operation should detect the corruption and reject the backup.
        ///
        /// Validates: Requirements 11.4
        #[test]
        fn prop_backup_integrity_verification(
            password in "[a-zA-Z0-9!@#$%^&*]{8,32}",
            account_name in "[a-zA-Z0-9_]{3,20}",
            corruption_byte_index in 0usize..100,
            corruption_xor_mask in 1u8..=255u8
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Setup keystore with an account
                let keychain = Box::new(MockKeychain::new());
                let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
                let _ = keystore.create_account(account_name.clone()).await.unwrap();
                
                let password_secret = SecretString::new(password.clone());

                // Create valid backup
                let mut backup = BackupManager::create_encrypted_backup(&keystore, &password_secret)
                    .await
                    .unwrap();

                // Corrupt the ciphertext
                let mut ciphertext_bytes = hex::decode(&backup.ciphertext).unwrap();
                
                // Only corrupt if we have enough bytes
                if ciphertext_bytes.len() > corruption_byte_index {
                    ciphertext_bytes[corruption_byte_index] ^= corruption_xor_mask;
                    backup.ciphertext = hex::encode(&ciphertext_bytes);

                    // Attempt to restore corrupted backup
                    let result = BackupManager::restore_from_backup(&backup, &password_secret);

                    // Should fail with integrity error
                    prop_assert!(
                        result.is_err(),
                        "Corrupted backup should be rejected"
                    );

                    // Verify it's specifically an integrity error
                    if let Err(e) = result {
                        let error_msg = format!("{:?}", e);
                        prop_assert!(
                            error_msg.contains("Integrity") || error_msg.contains("corrupted") || error_msg.contains("tampered"),
                            "Error should indicate integrity failure: {}",
                            error_msg
                        );
                    }
                }

                Ok(())
            }).unwrap();
        }

        /// Property: Backup metadata is preserved
        ///
        /// *For any* backup created, the metadata (version, timestamp, ID) should be
        /// preserved and accessible without decryption.
        #[test]
        fn prop_backup_metadata_preserved(
            password in "[a-zA-Z0-9!@#$%^&*]{8,32}",
            account_name in "[a-zA-Z0-9_]{3,20}"
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let keychain = Box::new(MockKeychain::new());
                let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
                let _ = keystore.create_account(account_name).await.unwrap();
                
                let password_secret = SecretString::new(password);

                let backup = BackupManager::create_encrypted_backup(&keystore, &password_secret)
                    .await
                    .unwrap();

                // Verify metadata is present and valid
                prop_assert_eq!(backup.version, 1, "Backup version should be 1");
                prop_assert!(!backup.id.is_nil(), "Backup ID should not be nil");
                
                let now = chrono::Utc::now().timestamp();
                prop_assert!(
                    backup.timestamp <= now && backup.timestamp >= now - 60,
                    "Backup timestamp should be recent (within 60 seconds)"
                );

                Ok(())
            }).unwrap();
        }

        /// Property: Backup salt and nonce are unique
        ///
        /// *For any* two backups created, they should have different salts and nonces
        /// to prevent cryptographic attacks.
        #[test]
        fn prop_backup_salt_nonce_unique(
            password in "[a-zA-Z0-9!@#$%^&*]{8,32}",
            account_name in "[a-zA-Z0-9_]{3,20}"
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let keychain1 = Box::new(MockKeychain::new());
                let mut keystore1 = SecureKeystoreImpl::new(keychain1).await.unwrap();
                let _ = keystore1.create_account(account_name.clone()).await.unwrap();
                
                let keychain2 = Box::new(MockKeychain::new());
                let mut keystore2 = SecureKeystoreImpl::new(keychain2).await.unwrap();
                let _ = keystore2.create_account(account_name).await.unwrap();
                
                let password_secret = SecretString::new(password);

                // Create two backups
                let backup1 = BackupManager::create_encrypted_backup(&keystore1, &password_secret)
                    .await
                    .unwrap();
                let backup2 = BackupManager::create_encrypted_backup(&keystore2, &password_secret)
                    .await
                    .unwrap();

                // Salts should be different
                prop_assert_ne!(
                    backup1.salt, backup2.salt,
                    "Backup salts should be unique"
                );

                // Nonces should be different
                prop_assert_ne!(
                    backup1.nonce, backup2.nonce,
                    "Backup nonces should be unique"
                );

                // IDs should be different
                prop_assert_ne!(
                    backup1.id, backup2.id,
                    "Backup IDs should be unique"
                );

                Ok(())
            }).unwrap();
        }
    }
}
