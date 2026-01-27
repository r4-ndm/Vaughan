//! Account Export Module
//!
//! This module provides secure export capabilities for sensitive account data.
//! All export operations are strictly controlled via `ExportAuthenticator` and require valid tokens.

use crate::error::{Result, SecurityError, VaughanError};
use crate::security::{ExportAuthenticator, SecureKeystore, AuthToken};
use alloy::primitives::Address;
use secrecy::{SecretString, ExposeSecret};
use uuid::Uuid;

/// Manages secure export of account secrets
#[derive(Debug, Clone)]
pub struct AccountExporter<'a> {
    keystore: &'a SecureKeystore,
    authenticator: &'a ExportAuthenticator,
}

impl<'a> AccountExporter<'a> {
    /// Create a new account exporter
    pub fn new(keystore: &'a SecureKeystore, authenticator: &'a ExportAuthenticator) -> Self {
        Self {
            keystore,
            authenticator,
        }
    }

    /// Export seed phrase
    ///
    /// Requires:
    /// 1. A valid authentication token.
    /// 2. The wallet password (to decrypt the seed).
    ///
    /// # Returns
    /// The decrypted seed phrase.
    pub async fn export_seed(
        &self,
        address: Address,
        token: &AuthToken,
        password: &SecretString,
    ) -> Result<SecretString> {
        let correlation_id = Uuid::new_v4();
        tracing::warn!(
            "🚨 EXPORT_SEED attempt. ID: {}, Address: {}, Token: {}",
            correlation_id,
            address,
            token.id
        );

        // 1. Verify Authentication Token
        self.authenticator.validate_token(token)?;

        // 2. Check Export Rate Limit (Task 10.2)
        self.authenticator.check_export_limit()?;

        // 3. Retrieve Seed
        let seed = self.keystore.get_decrypted_seed_phrase(&address, password).await.map_err(|e| {
             tracing::error!("❌ Export failed for {}: {}", correlation_id, e);
             e
        })?;

        tracing::warn!("✅ SEED EXPORT SUCCESS. ID: {}", correlation_id);
        Ok(seed)
    }

    /// Export private key
    ///
    /// Requires:
    /// 1. A valid authentication token.
    /// 2. Password (optional, strictly speaking needed for seed-based derivation).
    ///    Note: For pure private-key accounts, usually password isn't needed if 
    ///    unlocked keychain is enough, but Vaughan implies password-protected keystore.
    ///    `get_decrypted_private_key` needs password if seed-based.
    ///
    /// # Returns
    /// The private key as a hex string (SecretString).
    pub async fn export_private_key(
        &self,
        address: Address,
        token: &AuthToken,
        password: Option<&SecretString>,
    ) -> Result<SecretString> {
        let correlation_id = Uuid::new_v4();
        tracing::warn!(
             "🚨 EXPORT_PK attempt. ID: {}, Address: {}, Token: {}",
             correlation_id,
             address,
             token.id
        );

        // 1. Verify Authentication Token
        self.authenticator.validate_token(token)?;

        // 2. Check Export Rate Limit (Task 10.2)
        self.authenticator.check_export_limit()?;

        // 3. Retrieve Private Key
        let pk = self.keystore.get_decrypted_private_key(&address, password).await.map_err(|e| {
             tracing::error!("❌ Export failed for {}: {}", correlation_id, e);
             e
        })?;

        tracing::warn!("✅ PK EXPORT SUCCESS. ID: {}", correlation_id);
        Ok(pk)
    }

    /// Export to MetaMask V3 Keystore JSON
    ///
    /// Requires:
    /// 1. A valid authentication token.
    /// 2. The wallet password (to decrypt the private key).
    /// 3. A *new* password for the keystore encryption (can be same as wallet, but separate arg).
    ///
    /// # Returns
    /// The keystore JSON string.
    pub async fn export_to_v3_keystore(
        &self,
        address: Address,
        token: &AuthToken,
        wallet_password: Option<&SecretString>,
        keystore_password: &SecretString,
    ) -> Result<String> {
        use tempfile::tempdir;

        let correlation_id = Uuid::new_v4();
        tracing::warn!(
             "🚨 EXPORT_KEYSTORE attempt. ID: {}, Address: {}, Token: {}",
             correlation_id,
             address,
             token.id
        );

        // 1. Verify Authentication Token
        self.authenticator.validate_token(token)?;

        // 2. Check Export Rate Limit (Task 10.2)
        self.authenticator.check_export_limit()?;

        // 3. Retrieve Private Key
        // We get it as a SecretString (hex)
        let pk_secret = self.keystore.get_decrypted_private_key(&address, wallet_password).await?;
        let pk_hex = pk_secret.expose_secret();
        
        // 3. Encrypt using eth-keystore
        // eth-keystore::encrypt_key writes to a directory. We'll use a temp dir.
        let dir = tempdir().map_err(VaughanError::from)?;
        
        // eth-keystore expects bytes or hex? Documentation says:
        // encrypt_key<P: AsRef<Path>, R: Rng + CryptoRng>(dir: P, rng: &mut R, pk: &[u8], password: &str) -> Result<String, Error>
        // Ideally we pass it the key bytes.
        let pk_bytes = hex::decode(pk_hex).map_err(|_| {
             VaughanError::Security(SecurityError::InvalidPrivateKey)
        })?;

        // Use a random generator
        let mut rng = rand::thread_rng();

        let filename = eth_keystore::encrypt_key(
            dir.path(),
            &mut rng,
            &pk_bytes,
            keystore_password.expose_secret(),
            None, // Use default options (standard pbkdf2)
        ).map_err(|e| {
             tracing::error!("eth-keystore encryption failed: {}", e);
             VaughanError::Security(SecurityError::KeystoreError { 
                 message: format!("Encryption failed: {}", e) 
             })
        })?;

        // 4. Read the file back
        let file_path = dir.path().join(filename);
        let json_content = std::fs::read_to_string(file_path).map_err(VaughanError::from)?;

        tracing::warn!("✅ KEYSTORE EXPORT SUCCESS. ID: {}", correlation_id);
        Ok(json_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::keychain::MockKeychain;
    use crate::security::SecureKeystoreImpl;
    use crate::security::ExportAuthenticator;
    use secrecy::ExposeSecret;

    // Helper to create functional environment
    async fn setup_env() -> (SecureKeystoreImpl, ExportAuthenticator) {
        let keychain = Box::new(MockKeychain::new());
        let keystore = SecureKeystoreImpl::new(keychain).await.unwrap();
        let auth = ExportAuthenticator::new();
        (keystore, auth)
    }

    #[tokio::test]
    async fn test_export_private_key_auth_failure() {
        let (mut keystore, auth) = setup_env().await;
        // Create an account
        let account = keystore.create_account("Test".into()).await.unwrap();
        
        let exporter = AccountExporter::new(&keystore, &auth);
        
        // Create invalid token (expired logic or just bad) - but we use "new" which is private in export_auth?
        // Wait, AuthToken::new() is private!
        // We must use `authenticate`.
        
        // But authenticate needs a password valid boolean.
        let _valid_token = auth.authenticate(true).await.unwrap();

        // Expire it? We can't easily wait 2 mins in test.
        // We can use a manually constructed expired token if fields were public?
        // AuthToken fields are public!
        let expired_token = AuthToken {
            id: "expired".into(),
            expires_at: chrono::Utc::now() - chrono::Duration::seconds(10),
            signature: "sig".into(),
        };

        let res = exporter.export_private_key(account.address, &expired_token, None).await;
        assert!(matches!(res, Err(crate::error::VaughanError::Security(SecurityError::TokenExpired))));
    }

    #[tokio::test]
    async fn test_export_private_key_success() {
        let (mut keystore, auth) = setup_env().await;
        let account = keystore.create_account("Test".into()).await.unwrap();
        let exporter = AccountExporter::new(&keystore, &auth);
        
        let token = auth.authenticate(true).await.unwrap();
        
        let pk = exporter.export_private_key(account.address, &token, None).await.unwrap();
        assert_eq!(pk.expose_secret().len(), 64);
    }
}
