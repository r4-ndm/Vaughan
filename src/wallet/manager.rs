//! Wallet manager implementation
//!
//! This module provides the core functionality for managing the wallet, including
//! creation, locking, unlocking, and exporting keys. It orchestrates the
//! interaction between the keystore file, encryption/decryption logic, and
//! the in-memory signer.

use crate::wallet::errors::{WalletManagerError, WalletResult};
use crate::wallet::keystore_format::{CipherParams, CryptoSection, KdfParams, MetaMaskKeystore};
use aes::Aes256;
use alloy::primitives::{Address, B256};
use alloy::signers::local::PrivateKeySigner;
use bip32::{secp256k1::SecretKey, DerivationPath, ExtendedPrivateKey};
use bip39::Mnemonic;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr64BE;
use hex;
use k256::ecdsa::SigningKey;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uuid::Uuid;
use tracing::instrument;

/// PBKDF2 iteration count - follows MetaMask standard
/// 262144 iterations provides strong key stretching while remaining reasonable for UX
const PBKDF2_ITERATIONS: u32 = 262144;

/// Wallet manager
///
/// Handles high-level wallet operations such as:
/// - Creating new wallets (generating mnemonic, deriving keys, encrypting)
/// - Unlocking wallets (loading keystore, decrypting private key)
/// - Locking wallets (clearing sensitive data from memory)
/// - Exporting sensitive data (seed phrase, private key)
///
/// This struct holds the current state of the wallet (locked/unlocked) and
/// manages the persistence of the keystore file.
#[derive(Debug)]
pub struct WalletManager {
    /// Path to the keystore file
    keystore_path: PathBuf,

    /// Loaded keystore data (if file exists)
    keystore: Option<MetaMaskKeystore>,

    /// Active signer (only present when unlocked)
    signer: Option<PrivateKeySigner>,

    /// Decrypted mnemonic (only present when unlocked)
    /// Wrapped in SecretString for memory protection
    mnemonic: Option<SecretString>,
}

impl WalletManager {
    /// Create a new wallet manager instance
    ///
    /// # Arguments
    ///
    /// * `keystore_path` - Path to the keystore file
    pub fn new(keystore_path: PathBuf) -> Self {
        Self {
            keystore_path,
            keystore: None,
            signer: None,
            mnemonic: None,
        }
    }

    /// Create a new wallet
    ///
    /// Generates a new random mnemonic, derives the private key,
    /// encrypts it with the provided password, and saves the keystore to disk.
    #[instrument(skip(self, password), fields(keystore_path = ?self.keystore_path))]
    pub fn create_wallet(&mut self, password: SecretString) -> WalletResult<Address> {
        use std::str::FromStr;

        // 1. Generate random entropy for 12-word mnemonic (128 bits = 16 bytes)
        let mut entropy = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut entropy);

        // 2. Generate BIP39 mnemonic from entropy
        let mnemonic = Mnemonic::from_entropy(&entropy).map_err(|e| WalletManagerError::EncryptionFailed {
            reason: format!("Failed to generate mnemonic: {}", e),
        })?;

        let mnemonic_str = mnemonic.to_string();

        // 3. Derive seed from mnemonic (no passphrase)
        let seed = mnemonic.to_seed("");

        // 4. Create master extended private key
        let master_key =
            ExtendedPrivateKey::<SecretKey>::new(seed).map_err(|e| WalletManagerError::EncryptionFailed {
                reason: format!("Failed to create master key: {}", e),
            })?;

        // 5. Derive along standard Ethereum path: m/44'/60'/0'/0/0
        let derivation_path =
            DerivationPath::from_str("m/44'/60'/0'/0/0").map_err(|e| WalletManagerError::EncryptionFailed {
                reason: format!("Invalid derivation path: {}", e),
            })?;

        let mut derived_key = master_key;
        for child in derivation_path.into_iter() {
            derived_key = derived_key
                .derive_child(child)
                .map_err(|e| WalletManagerError::EncryptionFailed {
                    reason: format!("Failed to derive child key: {}", e),
                })?;
        }

        // 6. Convert to signing key and signer
        let secret_bytes = derived_key.private_key().to_bytes();
        let signing_key = SigningKey::from_bytes(&secret_bytes).map_err(|e| WalletManagerError::EncryptionFailed {
            reason: format!("Failed to create signing key: {}", e),
        })?;

        let signer = PrivateKeySigner::from(signing_key);
        let address = signer.address();

        // 7. Encrypt private key with password
        let private_key_hex = hex::encode(secret_bytes);
        let crypto = self.encrypt_with_password(&private_key_hex, &password)?;

        // 8. Create MetaMask-compatible keystore
        let keystore = MetaMaskKeystore {
            version: 3,
            id: Uuid::new_v4().to_string(),
            address,
            crypto,
            timestamp: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ),
        };

        // 9. Store in memory
        self.keystore = Some(keystore);
        self.signer = Some(signer);
        self.mnemonic = Some(SecretString::new(mnemonic_str));

        // 10. Save to file
        self.save_keystore()?;

        tracing::info!("🔐 Wallet created successfully: {}", address);

        Ok(address)
    }

    /// Unlock the wallet
    ///
    /// Loads the keystore from disk (if not already loaded) and attempts
    /// to decrypt the private key with the provided password.
    #[instrument(skip(self, password), fields(keystore_path = ?self.keystore_path))]
    pub fn unlock(&mut self, password: SecretString) -> WalletResult<()> {
        // Ensure keystore is loaded
        if self.keystore.is_none() {
            self.load_keystore()?;
        }

        if let Some(keystore) = &self.keystore {
            let plaintext = self.decrypt_with_password(&keystore.crypto, &password)?;

            // Convert to fixed bytes
            if plaintext.len() != 32 {
                return Err(WalletManagerError::DecryptionFailed {
                    reason: format!("Invalid private key length: {}", plaintext.len()),
                });
            }
            let key_b256 = B256::from_slice(&plaintext);

            let signer = PrivateKeySigner::from_bytes(&key_b256)
                .map_err(|e| WalletManagerError::DecryptionFailed { reason: e.to_string() })?;

            if signer.address() != keystore.address {
                return Err(WalletManagerError::DecryptionFailed {
                    reason: "Decrypted key address mismatch".to_string(),
                });
            }

            self.signer = Some(signer);
            // Note: Mnemonic is not recoverable from standard V3 keystore
            self.mnemonic = None;

            tracing::info!("🔓 Wallet unlocked successfully: {}", keystore.address);
            Ok(())
        } else {
            Err(WalletManagerError::AccountNotFound {
                address: "No keystore file found".to_string(),
            })
        }
    }

    /// Lock the wallet
    ///
    /// Clears all sensitive data (signer, mnemonic) from memory.
    /// The wallet must be unlocked again with the password to be used.
    #[instrument(skip(self))]
    pub fn lock(&mut self) -> WalletResult<()> {
        self.signer = None;
        self.mnemonic = None;
        tracing::info!("🔒 Wallet locked");
        Ok(())
    }

    /// Check if the wallet is currently unlocked
    pub fn is_unlocked(&self) -> bool {
        self.signer.is_some()
    }

    /// Get the wallet address
    ///
    /// Returns the address from the keystore (if loaded) or signer (if unlocked).
    /// Returns error if no wallet exists or is not loaded.
    pub fn address(&self) -> WalletResult<Address> {
        if let Some(signer) = &self.signer {
            Ok(signer.address())
        } else if let Some(keystore) = &self.keystore {
            Ok(keystore.address)
        } else {
            Err(WalletManagerError::AccountNotFound {
                address: "No wallet loaded".to_string(),
            })
        }
    }

    /// Export the seed phrase (mnemonic)
    ///
    /// Requires the wallet to be unlocked.
    #[instrument(skip(self))]
    pub fn export_seed_phrase(&self) -> WalletResult<String> {
        if !self.is_unlocked() {
            return Err(WalletManagerError::KeystoreLocked);
        }

        if let Some(mnemonic) = &self.mnemonic {
            Ok(mnemonic.expose_secret().clone())
        } else {
            // Mnemonic unavailable (e.g. restored from keystore which doesn't store it)
            Err(WalletManagerError::InvalidSeedPhrase {
                reason: "Mnemonic not available (restored from keystore)".to_string(),
            })
        }
    }

    /// Export the private key
    ///
    /// Requires the wallet to be unlocked.
    #[instrument(skip(self))]
    pub fn export_private_key(&self) -> WalletResult<SecretString> {
        if let Some(signer) = &self.signer {
            let bytes = signer.to_bytes();
            let hex_key = hex::encode(bytes);
            Ok(SecretString::new(hex_key))
        } else {
            Err(WalletManagerError::KeystoreLocked)
        }
    }

    /// Encrypt data with password
    fn encrypt_with_password(&self, data: &str, password: &SecretString) -> WalletResult<CryptoSection> {
        let password_bytes = password.expose_secret().as_bytes();
        let data_bytes =
            hex::decode(data).map_err(|e| WalletManagerError::EncryptionFailed { reason: e.to_string() })?;

        // 1. Generate Salt (32 bytes)
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        // 2. Derive Key (PBKDF2)
        let mut derived_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password_bytes, &salt, 262144, &mut derived_key);

        // 3. Generate IV (16 bytes)
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        // 4. Encrypt (AES-256-CTR)
        let mut ciphertext = data_bytes.clone();
        type Aes256Ctr = Ctr64BE<Aes256>;
        let mut cipher = Aes256Ctr::new(&derived_key.into(), &iv.into());
        cipher.apply_keystream(&mut ciphertext);

        // 5. Calculate MAC (SHA256(derived_key + ciphertext))
        let mut hasher = Sha256::new();
        hasher.update(derived_key);
        hasher.update(&ciphertext);
        let mac = hasher.finalize();

        Ok(CryptoSection {
            cipher: "aes-256-ctr".to_string(),
            ciphertext: hex::encode(ciphertext),
            cipherparams: CipherParams { iv: hex::encode(iv) },
            kdf: "pbkdf2".to_string(),
            kdfparams: KdfParams {
                salt: hex::encode(salt),
                dklen: 32,
                prf: "hmac-sha256".to_string(),
                c: PBKDF2_ITERATIONS,
            },
            mac: hex::encode(mac),
        })
    }

    /// Decrypt data with password
    fn decrypt_with_password(&self, crypto: &CryptoSection, password: &SecretString) -> WalletResult<Vec<u8>> {
        let password_bytes = password.expose_secret().as_bytes();

        // Decode fields
        let salt = hex::decode(&crypto.kdfparams.salt).map_err(|e| WalletManagerError::DecryptionFailed {
            reason: format!("Invalid salt: {}", e),
        })?;
        let iv = hex::decode(&crypto.cipherparams.iv).map_err(|e| WalletManagerError::DecryptionFailed {
            reason: format!("Invalid IV: {}", e),
        })?;
        let ciphertext = hex::decode(&crypto.ciphertext).map_err(|e| WalletManagerError::DecryptionFailed {
            reason: format!("Invalid ciphertext: {}", e),
        })?;
        let stored_mac = hex::decode(&crypto.mac).map_err(|e| WalletManagerError::DecryptionFailed {
            reason: format!("Invalid MAC: {}", e),
        })?;

        // Derive Key
        let mut derived_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password_bytes, &salt, 262144, &mut derived_key);

        // Verify MAC
        let mut hasher = Sha256::new();
        hasher.update(derived_key);
        hasher.update(&ciphertext);
        let calculated_mac = hasher.finalize();

        if calculated_mac.as_slice() != stored_mac.as_slice() {
            return Err(WalletManagerError::InvalidPassword);
        }

        // Decrypt
        let mut plaintext = ciphertext.clone();
        type Aes256Ctr = Ctr64BE<Aes256>;

        if iv.len() != 16 {
            return Err(WalletManagerError::DecryptionFailed {
                reason: "Invalid IV length".to_string(),
            });
        }
        // IV length is checked above, so this conversion is safe
        #[allow(clippy::expect_used)]
        let iv_array: [u8; 16] = iv
            .try_into()
            .expect("IV length is verified to be 16 bytes");

        let mut cipher = Aes256Ctr::new(&derived_key.into(), &iv_array.into());
        cipher.apply_keystream(&mut plaintext);

        Ok(plaintext)
    }

    /// Save keystore to file
    fn save_keystore(&self) -> WalletResult<()> {
        if let Some(keystore) = &self.keystore {
            if let Some(parent) = self.keystore_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let json = serde_json::to_string_pretty(keystore)?;
            std::fs::write(&self.keystore_path, json)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Load keystore from file
    fn load_keystore(&mut self) -> WalletResult<()> {
        if self.keystore_path.exists() {
            let json = std::fs::read_to_string(&self.keystore_path)?;
            let keystore: MetaMaskKeystore = serde_json::from_str(&json)?;
            self.keystore = Some(keystore);
            Ok(())
        } else {
            // No keystore found is fine, just not loaded
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Create a test wallet manager with a temporary directory
    /// Returns the manager, password, and the tempdir (must keep alive for the test)
    fn create_test_wallet() -> (WalletManager, SecretString, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("keystore.json");
        let manager = WalletManager::new(path);
        (manager, SecretString::new("1234".to_string()), dir)
    }

    // ========== Task 5.1: Unit Tests ==========

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (manager, password, _dir) = create_test_wallet();
        let data = "deadbeef";
        let encrypted = manager.encrypt_with_password(data, &password).unwrap();

        // Decrypt works
        let decrypted = manager.decrypt_with_password(&encrypted, &password).unwrap();
        assert_eq!(hex::encode(decrypted), data);
    }

    #[test]
    fn test_wrong_password() {
        let (manager, password, _dir) = create_test_wallet();
        let data = "deadbeef";
        let encrypted = manager.encrypt_with_password(data, &password).unwrap();

        let wrong_password = SecretString::new("wrong".to_string());
        let result = manager.decrypt_with_password(&encrypted, &wrong_password);
        assert!(result.is_err());
        match result {
            Err(WalletManagerError::InvalidPassword) => {}
            _ => panic!("Expected InvalidPassword error"),
        }
    }

    #[test]
    fn test_create_wallet() {
        let (mut manager, password, _dir) = create_test_wallet();

        // Create wallet
        let address = manager.create_wallet(password).unwrap();

        // Verify address is non-zero
        assert_ne!(address, Address::ZERO, "Address should not be zero");

        // Verify wallet is unlocked after creation
        assert!(manager.is_unlocked(), "Wallet should be unlocked after creation");

        // Verify keystore file was created
        assert!(manager.keystore_path.exists(), "Keystore file should exist");
    }

    #[test]
    fn test_unlock_wallet() {
        let (mut manager, password, dir) = create_test_wallet();

        // Create wallet first
        let address = manager.create_wallet(password.clone()).unwrap();

        // Lock the wallet
        manager.lock().unwrap();
        assert!(!manager.is_unlocked(), "Wallet should be locked");

        // Create a new manager instance (simulates app restart)
        let new_manager_path = dir.path().join("keystore.json");
        let mut new_manager = WalletManager::new(new_manager_path);

        // Unlock with correct password
        new_manager.unlock(password).unwrap();

        // Verify is_unlocked returns true
        assert!(new_manager.is_unlocked(), "Wallet should be unlocked");

        // Verify address matches
        assert_eq!(new_manager.address().unwrap(), address, "Address should match");
    }

    #[test]
    fn test_export_seed_phrase() {
        let (mut manager, password, _dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password).unwrap();

        // Export seed phrase
        let seed_phrase = manager.export_seed_phrase().unwrap();

        // Verify 12 words
        let word_count = seed_phrase.split_whitespace().count();
        assert_eq!(word_count, 12, "Seed phrase should have 12 words, got {}", word_count);

        // Verify all words are valid BIP39 words (simple check - no numbers)
        for word in seed_phrase.split_whitespace() {
            assert!(
                word.chars().all(|c| c.is_alphabetic()),
                "Word '{}' should be alphabetic",
                word
            );
        }
    }

    #[test]
    fn test_export_private_key() {
        let (mut manager, password, _dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password).unwrap();

        // Export private key
        let private_key = manager.export_private_key().unwrap();
        let key_hex = private_key.expose_secret();

        // Verify length is 64 hex chars (32 bytes)
        assert_eq!(
            key_hex.len(),
            64,
            "Private key should be 64 hex chars, got {}",
            key_hex.len()
        );

        // Verify it's valid hex
        assert!(hex::decode(key_hex).is_ok(), "Private key should be valid hex");
    }

    #[test]
    fn test_export_while_locked_fails() {
        let (mut manager, password, _dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password).unwrap();

        // Lock wallet
        manager.lock().unwrap();

        // Try to export seed phrase - should fail
        let result = manager.export_seed_phrase();
        assert!(result.is_err(), "Export seed phrase should fail when locked");

        // Try to export private key - should fail
        let result = manager.export_private_key();
        assert!(result.is_err(), "Export private key should fail when locked");
    }

    // ========== Task 5.2: Integration Tests ==========

    #[test]
    fn test_full_wallet_lifecycle() {
        let (mut manager, password, dir) = create_test_wallet();

        // Step 1: Create wallet with "1234"
        let address = manager.create_wallet(password.clone()).unwrap();
        assert_ne!(address, Address::ZERO);

        // Step 2: Export seed phrase
        let seed_phrase = manager.export_seed_phrase().unwrap();
        assert_eq!(seed_phrase.split_whitespace().count(), 12);

        // Step 3: Export private key
        let private_key = manager.export_private_key().unwrap();
        let key_hex = private_key.expose_secret().clone();
        assert_eq!(key_hex.len(), 64);

        // Step 4: Lock wallet
        manager.lock().unwrap();
        assert!(!manager.is_unlocked());

        // Step 5: Try export while locked (should fail)
        assert!(manager.export_seed_phrase().is_err());
        assert!(manager.export_private_key().is_err());

        // Step 6: Create new manager and unlock with same password
        let new_path = dir.path().join("keystore.json");
        let mut new_manager = WalletManager::new(new_path);
        new_manager.unlock(password.clone()).unwrap();
        assert!(new_manager.is_unlocked());

        // Step 7: Export again and verify data matches
        let new_private_key = new_manager.export_private_key().unwrap();
        assert_eq!(
            new_private_key.expose_secret(),
            &key_hex,
            "Private key should match after unlock"
        );

        // Note: Seed phrase won't be available after unlock from keystore (by design)
        // The V3 keystore format doesn't store the mnemonic
    }

    #[test]
    fn test_lock_unlock_cycle() {
        let (mut manager, password, _dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password.clone()).unwrap();

        // Lock/unlock cycle 5 times
        for i in 0..5 {
            // Lock
            manager.lock().unwrap();
            assert!(!manager.is_unlocked(), "Cycle {}: Should be locked", i);

            // Unlock
            manager.unlock(password.clone()).unwrap();
            assert!(manager.is_unlocked(), "Cycle {}: Should be unlocked", i);
        }
    }

    #[test]
    fn test_multiple_sessions() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("keystore.json");
        let password = SecretString::new("1234".to_string());

        // Session 1: Create wallet
        let address = {
            let mut manager = WalletManager::new(path.clone());
            let addr = manager.create_wallet(password.clone()).unwrap();
            manager.lock().unwrap();
            addr
        };

        // Session 2: New manager instance, unlock
        {
            let mut manager = WalletManager::new(path.clone());
            manager.unlock(password.clone()).unwrap();
            assert!(manager.is_unlocked());
            assert_eq!(manager.address().unwrap(), address);
        }

        // Session 3: Yet another instance
        {
            let mut manager = WalletManager::new(path.clone());
            manager.unlock(password.clone()).unwrap();
            assert_eq!(
                manager.address().unwrap(),
                address,
                "Address should persist across sessions"
            );
        }
    }

    // ========== Task 5.3: Security Tests ==========

    #[test]
    fn test_wrong_password_fails_gracefully() {
        let (mut manager, password, dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password).unwrap();
        manager.lock().unwrap();

        // Create new manager and try wrong password
        let new_path = dir.path().join("keystore.json");
        let mut new_manager = WalletManager::new(new_path);

        let wrong_password = SecretString::new("wrong".to_string());
        let result = new_manager.unlock(wrong_password);

        // Verify error is generic (no information leakage)
        match result {
            Err(WalletManagerError::InvalidPassword) => {
                // Error message should not reveal specific failure details
                let msg = WalletManagerError::InvalidPassword.to_string();
                assert!(!msg.contains("MAC"), "Error should not reveal MAC validation failure");
                assert!(!msg.contains("key"), "Error should not reveal key derivation details");
            }
            _ => panic!("Expected InvalidPassword error"),
        }
    }

    #[test]
    fn test_memory_is_cleared_after_lock() {
        let (mut manager, password, _dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password).unwrap();

        // Verify we can export
        assert!(manager.export_seed_phrase().is_ok());
        assert!(manager.export_private_key().is_ok());

        // Lock
        manager.lock().unwrap();

        // Try to export - should fail (memory cleared)
        assert!(manager.export_seed_phrase().is_err());
        assert!(manager.export_private_key().is_err());

        // Verify internal state is cleared
        assert!(!manager.is_unlocked());
    }

    #[test]
    fn test_password_1234_works_for_development() {
        // Per success criteria: password "1234" must work
        let (mut manager, _password, _dir) = create_test_wallet();

        let dev_password = SecretString::new("1234".to_string());
        let result = manager.create_wallet(dev_password);

        assert!(result.is_ok(), "Password '1234' should work for development");
    }

    #[test]
    fn test_mac_verification() {
        let (mut manager, password, dir) = create_test_wallet();

        // Create wallet
        manager.create_wallet(password.clone()).unwrap();
        manager.lock().unwrap();

        // Corrupt the MAC in the keystore file
        let keystore_path = dir.path().join("keystore.json");
        let mut keystore_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&keystore_path).unwrap()).unwrap();

        // Flip a character in the MAC
        if let Some(mac) = keystore_json["crypto"]["mac"].as_str() {
            let mut corrupted = mac.to_string();
            if corrupted.starts_with('a') {
                corrupted = format!("b{}", &corrupted[1..]);
            } else {
                corrupted = format!("a{}", &corrupted[1..]);
            }
            keystore_json["crypto"]["mac"] = serde_json::Value::String(corrupted);
        }

        std::fs::write(&keystore_path, serde_json::to_string_pretty(&keystore_json).unwrap()).unwrap();

        // Try to unlock - should fail due to MAC mismatch
        let mut new_manager = WalletManager::new(keystore_path);
        let result = new_manager.unlock(password);

        assert!(result.is_err(), "Unlock should fail with corrupted MAC");
    }

    /// Test 6.7: MetaMask Keystore Format Compatibility
    /// Verifies that the generated keystore follows MetaMask V3 format
    #[test]
    fn test_metamask_keystore_format() {
        let (mut manager, password, dir) = create_test_wallet();

        // Create wallet
        let address = manager.create_wallet(password).unwrap();

        // Read the keystore file
        let keystore_path = dir.path().join("keystore.json");
        let keystore_content = std::fs::read_to_string(&keystore_path).unwrap();
        let keystore_json: serde_json::Value = serde_json::from_str(&keystore_content).unwrap();

        // Verify MetaMask V3 format requirements:

        // 1. Version must be 3
        assert_eq!(
            keystore_json["version"].as_u64().unwrap(),
            3,
            "Version must be 3 for MetaMask compatibility"
        );

        // 2. Must have id (UUID)
        assert!(keystore_json["id"].is_string(), "Must have UUID id field");
        let id = keystore_json["id"].as_str().unwrap();
        assert!(!id.is_empty(), "ID must not be empty");

        // 3. Must have address field
        assert!(keystore_json["address"].is_string(), "Must have address field");
        let stored_address = keystore_json["address"].as_str().unwrap();
        assert!(stored_address.starts_with("0x"), "Address should have 0x prefix");
        assert_eq!(
            stored_address.to_lowercase(),
            format!("{:#x}", address).to_lowercase(),
            "Address must match"
        );

        // 4. Crypto section requirements
        let crypto = &keystore_json["crypto"];
        assert!(crypto.is_object(), "Must have crypto section");

        // 4a. Cipher must be aes-128-ctr or aes-256-ctr
        let cipher = crypto["cipher"].as_str().unwrap();
        assert!(
            cipher == "aes-128-ctr" || cipher == "aes-256-ctr",
            "Cipher must be AES-CTR"
        );

        // 4b. Must have ciphertext
        let ciphertext = crypto["ciphertext"].as_str().unwrap();
        assert!(!ciphertext.is_empty(), "Ciphertext must not be empty");
        assert_eq!(
            ciphertext.len(),
            64,
            "Ciphertext should be 32 bytes (64 hex chars) for a 32-byte private key"
        );

        // 4c. Cipherparams must have iv
        let cipherparams = &crypto["cipherparams"];
        let iv = cipherparams["iv"].as_str().unwrap();
        assert_eq!(iv.len(), 32, "IV should be 16 bytes (32 hex chars)");

        // 4d. KDF must be pbkdf2 or scrypt (we use pbkdf2)
        let kdf = crypto["kdf"].as_str().unwrap();
        assert!(kdf == "pbkdf2" || kdf == "scrypt", "KDF must be pbkdf2 or scrypt");

        // 4e. KDF params must be present
        let kdfparams = &crypto["kdfparams"];
        assert!(kdfparams["dklen"].as_u64().is_some(), "Must have dklen");
        assert!(kdfparams["salt"].as_str().is_some(), "Must have salt");

        if kdf == "pbkdf2" {
            assert!(kdfparams["c"].as_u64().is_some(), "PBKDF2 must have iterations (c)");
            let iterations = kdfparams["c"].as_u64().unwrap();
            assert!(
                iterations >= 100000,
                "PBKDF2 iterations should be at least 100000 for security"
            );
            assert!(kdfparams["prf"].as_str().is_some(), "PBKDF2 must have PRF");
        }

        // 4f. Must have MAC
        let mac = crypto["mac"].as_str().unwrap();
        assert!(!mac.is_empty(), "MAC must not be empty");
        assert_eq!(mac.len(), 64, "MAC should be 32 bytes (64 hex chars) SHA-256");

        tracing::info!("✅ Keystore format is MetaMask V3 compatible");
        tracing::info!("   Version: {}", keystore_json["version"]);
        tracing::info!("   ID: {}", id);
        tracing::info!("   Address: {}", stored_address);
        tracing::info!("   Cipher: {}", cipher);
        tracing::info!("   KDF: {} with {} iterations", kdf, kdfparams["c"]);
    }
}
