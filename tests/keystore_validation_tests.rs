//! MetaMask V3 Keystore Validation Tests
//!
//! Unit tests for V3 keystore validation covering:
//! - Invalid JSON rejection
//! - Corrupted keystore detection (bad MAC, bad ciphertext)
//! - Wrong password handling
//!
//! Implements: Task 2.5 - Write unit tests for V3 keystore validation

use secrecy::SecretString;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use vaughan::error::{Result, SecurityError};
use vaughan::security::{
    ExportAuthenticator, KeyReference, KeychainInterface, SecureKeystoreImpl,
};
use vaughan::wallet::account_manager::export::AccountExporter;
use vaughan::wallet::account_manager::import::{AccountImporter, ImportMetadata};
use vaughan::wallet::keystore_format::MetaMaskKeystore;

/// Local mock keychain for testing
#[derive(Debug, Clone)]
pub struct TestKeychain {
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl TestKeychain {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl KeychainInterface for TestKeychain {
    fn store(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        use secrecy::ExposeSecret;
        let mut storage = self.storage.lock().unwrap();
        storage.insert(key_ref.id.clone(), key.expose_secret().to_string());
        Ok(())
    }

    fn retrieve(&self, key_ref: &KeyReference) -> Result<SecretString> {
        let storage = self.storage.lock().map_err(|_| {
            vaughan::error::VaughanError::Security(SecurityError::KeystoreError {
                message: "Lock poisoned".into(),
            })
        })?;
        storage
            .get(&key_ref.id)
            .map(|s| SecretString::new(s.clone()))
            .ok_or_else(|| {
                SecurityError::KeystoreError {
                    message: "Key not found".to_string(),
                }
                .into()
            })
    }

    fn delete(&self, key_ref: &KeyReference) -> Result<()> {
        let mut storage = self.storage.lock().map_err(|_| {
            vaughan::error::VaughanError::Security(SecurityError::KeystoreError {
                message: "Lock poisoned".into(),
            })
        })?;
        storage.remove(&key_ref.id);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn KeychainInterface> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Task 2.5: Invalid JSON Rejection Tests
// ============================================================================

#[test]
fn test_invalid_json_rejected() {
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.json");

    // Write invalid JSON
    let invalid_json = "{ this is not valid json }";
    std::fs::write(&file_path, invalid_json).unwrap();

    let password = SecretString::new("testpassword123".to_string());
    let result = importer.import_from_keystore(invalid_json, &password, ImportMetadata::new());

    assert!(result.is_err(), "Should reject invalid JSON");
}

#[test]
fn test_empty_json_rejected() {
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("empty.json");

    // Write empty JSON
    let empty_json = "{}";
    std::fs::write(&file_path, empty_json).unwrap();

    let password = SecretString::new("testpassword123".to_string());
    let result = importer.import_from_keystore(empty_json, &password, ImportMetadata::new());

    assert!(result.is_err(), "Should reject empty JSON");
}

#[test]
fn test_missing_crypto_section_rejected() {
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("missing_crypto.json");

    // Write JSON missing crypto section
    let invalid_keystore = r#"{
        "version": 3,
        "id": "test-uuid",
        "address": "0x0000000000000000000000000000000000000000"
    }"#;
    std::fs::write(&file_path, invalid_keystore).unwrap();

    let password = SecretString::new("testpassword123".to_string());
    let result = importer.import_from_keystore(invalid_keystore, &password, ImportMetadata::new());

    assert!(result.is_err(), "Should reject keystore missing crypto section");
}

// ============================================================================
// Task 2.5: Corrupted Keystore Detection Tests
// ============================================================================

#[tokio::test]
async fn test_corrupted_mac_detected() {
    // Setup
    let keychain = Box::new(TestKeychain::new());
    let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
    let authenticator = ExportAuthenticator::new();

    // Create account
    let account = keystore.create_account("test".into()).await.unwrap();
    let exporter = AccountExporter::new(&keystore, &authenticator);

    // Authenticate and export
    let token = authenticator.authenticate(true).await.unwrap();
    let password = SecretString::new("testpassword123".to_string());
    let json = exporter
        .export_to_v3_keystore(account.address, &token, None, &password)
        .await
        .unwrap();

    // Corrupt the MAC
    let mut corrupted: serde_json::Value = serde_json::from_str(&json).unwrap();
    if let Some(crypto) = corrupted.get_mut("crypto") {
        if let Some(mac) = crypto.get_mut("mac") {
            // Replace first few chars of MAC
            let original = mac.as_str().unwrap();
            let corrupted_mac = format!("0000{}", &original[4..]);
            *mac = serde_json::Value::String(corrupted_mac);
        }
    }
    let corrupted_json = serde_json::to_string(&corrupted).unwrap();

    // Try to import
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("corrupted_mac.json");
    std::fs::write(&file_path, &corrupted_json).unwrap();

    let result = importer.import_from_keystore(&corrupted_json, &password, ImportMetadata::new());

    assert!(result.is_err(), "Should detect corrupted MAC");
}

#[tokio::test]
async fn test_corrupted_ciphertext_detected() {
    // Setup
    let keychain = Box::new(TestKeychain::new());
    let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
    let authenticator = ExportAuthenticator::new();

    // Create account
    let account = keystore.create_account("test".into()).await.unwrap();
    let exporter = AccountExporter::new(&keystore, &authenticator);

    // Authenticate and export
    let token = authenticator.authenticate(true).await.unwrap();
    let password = SecretString::new("testpassword123".to_string());
    let json = exporter
        .export_to_v3_keystore(account.address, &token, None, &password)
        .await
        .unwrap();

    // Corrupt the ciphertext
    let mut corrupted: serde_json::Value = serde_json::from_str(&json).unwrap();
    if let Some(crypto) = corrupted.get_mut("crypto") {
        if let Some(ct) = crypto.get_mut("ciphertext") {
            let original = ct.as_str().unwrap();
            if original.len() > 8 {
                let corrupted_ct = format!("deadbeef{}", &original[8..]);
                *ct = serde_json::Value::String(corrupted_ct);
            }
        }
    }
    let corrupted_json = serde_json::to_string(&corrupted).unwrap();

    // Try to import
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("corrupted_ct.json");
    std::fs::write(&file_path, &corrupted_json).unwrap();

    let result = importer.import_from_keystore(&corrupted_json, &password, ImportMetadata::new());

    assert!(result.is_err(), "Should detect corrupted ciphertext");
}

// ============================================================================
// Task 2.5: Wrong Password Handling Tests
// ============================================================================

#[tokio::test]
async fn test_wrong_password_rejected() {
    // Setup
    let keychain = Box::new(TestKeychain::new());
    let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
    let authenticator = ExportAuthenticator::new();

    // Create account
    let account = keystore.create_account("test".into()).await.unwrap();
    let exporter = AccountExporter::new(&keystore, &authenticator);

    // Authenticate and export with one password
    let token = authenticator.authenticate(true).await.unwrap();
    let correct_password = SecretString::new("correctpassword123".to_string());
    let json = exporter
        .export_to_v3_keystore(account.address, &token, None, &correct_password)
        .await
        .unwrap();

    // Try to import with wrong password
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("keystore.json");
    std::fs::write(&file_path, &json).unwrap();

    let wrong_password = SecretString::new("wrongpassword456".to_string());
    let result =
        importer.import_from_keystore(&json, &wrong_password, ImportMetadata::new());

    assert!(result.is_err(), "Should reject wrong password");
}

#[tokio::test]
async fn test_empty_password_rejected() {
    // Setup
    let keychain = Box::new(TestKeychain::new());
    let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
    let authenticator = ExportAuthenticator::new();

    // Create account
    let account = keystore.create_account("test".into()).await.unwrap();
    let exporter = AccountExporter::new(&keystore, &authenticator);

    // Export with a password
    let token = authenticator.authenticate(true).await.unwrap();
    let password = SecretString::new("testpassword123".to_string());
    let json = exporter
        .export_to_v3_keystore(account.address, &token, None, &password)
        .await
        .unwrap();

    // Try to import with empty password
    let importer = AccountImporter::new();
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("keystore.json");
    std::fs::write(&file_path, &json).unwrap();

    let empty_password = SecretString::new("".to_string());
    let result =
        importer.import_from_keystore(&json, &empty_password, ImportMetadata::new());

    assert!(result.is_err(), "Should reject empty password");
}

// ============================================================================
// V3 Format Structure Validation
// ============================================================================

#[test]
fn test_v3_keystore_format_structure() {
    // Verify keystore structure follows MetaMask V3 standard
    let keystore = MetaMaskKeystore::new();

    assert_eq!(keystore.version, 3, "Version must be 3 for MetaMask compatibility");
    assert!(!keystore.id.is_empty(), "ID must not be empty");
    assert_eq!(keystore.crypto.cipher, "aes-256-ctr", "Must use AES-256-CTR");
    assert_eq!(keystore.crypto.kdf, "pbkdf2", "Must use PBKDF2");
    assert_eq!(keystore.crypto.kdfparams.dklen, 32, "Derived key length must be 32 bytes");
    assert_eq!(
        keystore.crypto.kdfparams.prf, "hmac-sha256",
        "Must use HMAC-SHA256"
    );
    assert_eq!(
        keystore.crypto.kdfparams.c, 262144,
        "Must use 262,144 PBKDF2 iterations (MetaMask standard)"
    );
}

#[test]
fn test_v3_keystore_serialization_roundtrip() {
    let keystore = MetaMaskKeystore::new();

    // Serialize to JSON
    let json = serde_json::to_string(&keystore).expect("Should serialize");

    // Verify required fields are present
    assert!(json.contains(r#""version":3"#));
    assert!(json.contains(r#""cipher":"aes-256-ctr""#));
    assert!(json.contains(r#""kdf":"pbkdf2""#));
    assert!(json.contains(r#""c":262144"#));

    // Deserialize back
    let parsed: MetaMaskKeystore =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(parsed.version, keystore.version);
    assert_eq!(parsed.id, keystore.id);
}

