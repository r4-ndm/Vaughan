use proptest::prelude::*;
use secrecy::{ExposeSecret, SecretString};
use tempfile::tempdir;
use vaughan::security::{ExportAuthenticator, SecureKeystoreImpl, KeyReference, KeychainInterface};
use vaughan::wallet::account_manager::export::AccountExporter;
use vaughan::wallet::account_manager::import::{AccountImporter, ImportMetadata};
use vaughan::error::{Result, SecurityError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Local MockKeychain since the one in lib is cfg(test) internal
#[derive(Debug, Clone)]
pub struct LocalMockKeychain {
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl LocalMockKeychain {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl KeychainInterface for LocalMockKeychain {
    fn store(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(key_ref.id.clone(), key.expose_secret().to_string());
        Ok(())
    }

    fn retrieve(&self, key_ref: &KeyReference) -> Result<SecretString> {
        let storage = self.storage.lock().map_err(|_| vaughan::error::VaughanError::Security(
             SecurityError::KeystoreError { message: "Lock poisoned".into() }
        ))?;
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
        let mut storage = self.storage.lock().map_err(|_| vaughan::error::VaughanError::Security(
             SecurityError::KeystoreError { message: "Lock poisoned".into() }
        ))?;
        storage.remove(&key_ref.id);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn KeychainInterface> {
        Box::new(self.clone())
    }
}

proptest! {
    // Property 2.4: V3 keystore round-trip with 100 iterations
    // Note: Using 50 cases for PBKDF2 operations (262,144 iterations each)
    // to balance thorough testing with reasonable test duration
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Test V3 keystore round-trip: export -> import produces same account
    /// Tests various password strengths from 8-64 characters
    /// Validates: Requirements 8.2 (Seed Phrase Import Determinism for key material)
    #[test]
    fn test_round_trip_v3_keystore(
        _pk_bytes in proptest::collection::vec(any::<u8>(), 32),
        password in "([a-zA-Z0-9!@#$%^&*]){8,64}"  // Extended password range with special chars
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Setup
            let keychain = Box::new(LocalMockKeychain::new());
            let mut keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
            let authenticator = ExportAuthenticator::new();
            let importer = AccountImporter::new();

            // Create account FIRST (mutation)
            let account_name = "test_acc";
            let created_account = keystore.create_account(account_name.into()).await.unwrap();
            let created_address = created_account.address;

            // THEN Initialize exporter (immutable borrow)
            let exporter = AccountExporter::new(&keystore, &authenticator);

            // Authenticate
            let token = authenticator.authenticate(true).await.unwrap();
            
            // Export
            let wallet_password = None; 
            let ks_password = SecretString::new(password.clone());
            
            let json = exporter.export_to_v3_keystore(
                created_address,
                &token,
                wallet_password,
                &ks_password
            ).await.expect("Export failed");
            
            // Import using eth-keystore logic (via import_from_keystore)
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("keystore.json");
            std::fs::write(&file_path, &json).unwrap();
            
            let (imported_account, imported_signer) = importer.import_from_keystore(
                &json,
                &ks_password,
                ImportMetadata::new()
            ).expect("Import failed");
            
            // Verify address matches
            assert_eq!(imported_account.address, created_address);
            
            // Verify pk matches
            let original_pk = exporter.export_private_key(created_address, &token, wallet_password).await.unwrap();
            let imported_pk_bytes = imported_signer.to_bytes();
            
            // Fix assertion: imported_pk_bytes is [u8; 32], original_pk is SecretString
            assert_eq!(hex::encode(imported_pk_bytes.as_slice()), *original_pk.expose_secret());
        });
    }
}
