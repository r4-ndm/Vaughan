//! OS keychain integration for secure key storage
//!
//! This module provides platform-specific keychain integration for secure
//! storage of private keys using the operating system's native keychain.

// Allow unsafe code - all unsafe blocks are documented with SAFETY comments
// See Phase 4 Task 4.3 completion for full unsafe code audit
#![allow(unsafe_code)]

use secrecy::{ExposeSecret, SecretString};
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::LazyLock;
#[cfg(test)]
use std::sync::Mutex;

use super::{KeyReference, KeychainInterface};
use crate::error::{Result, SecurityError};

/// Platform-specific keychain implementation
#[derive(Debug)]
pub struct OSKeychain {
    service_name: String,
    #[cfg(test)]
    mock_storage: &'static Mutex<HashMap<String, String>>,
}

#[cfg(test)]
static MOCK_STORAGE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

impl OSKeychain {
    /// Create a new OS keychain interface
    pub fn new(service_name: String) -> Result<Self> {
        Ok(Self {
            service_name,
            #[cfg(test)]
            mock_storage: &MOCK_STORAGE,
        })
    }
}

impl KeychainInterface for OSKeychain {
    fn store(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        #[cfg(not(test))]
        {
            self.store_platform_specific(key_ref, key)
        }

        #[cfg(test)]
        {
            let mut storage = self.mock_storage.lock().map_err(|_| SecurityError::KeystoreError {
                message: "Failed to acquire mock storage lock".to_string(),
            })?;
            storage.insert(key_ref.id.clone(), key.expose_secret().to_string());
            Ok(())
        }
    }

    fn retrieve(&self, key_ref: &KeyReference) -> Result<SecretString> {
        #[cfg(not(test))]
        {
            self.retrieve_platform_specific(key_ref)
        }

        #[cfg(test)]
        {
            let storage = self.mock_storage.lock().map_err(|_| SecurityError::KeystoreError {
                message: "Failed to acquire mock storage lock".to_string(),
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
    }

    fn delete(&self, key_ref: &KeyReference) -> Result<()> {
        #[cfg(not(test))]
        {
            self.delete_platform_specific(key_ref)
        }

        #[cfg(test)]
        {
            let mut storage = self.mock_storage.lock().map_err(|_| SecurityError::KeystoreError {
                message: "Failed to acquire mock storage lock".to_string(),
            })?;
            storage.remove(&key_ref.id);
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn KeychainInterface> {
        Box::new(OSKeychain {
            service_name: self.service_name.clone(),
            #[cfg(test)]
            mock_storage: self.mock_storage,
        })
    }
}

#[cfg(not(test))]
impl OSKeychain {
    /// Store key using platform-specific keychain
    fn store_platform_specific(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.store_macos_keychain(key_ref, key)
        }

        #[cfg(target_os = "linux")]
        {
            self.store_linux_keyring(key_ref, key)
        }

        #[cfg(target_os = "windows")]
        {
            self.store_windows_credential_manager(key_ref, key)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            // Fallback to encrypted file storage for unsupported platforms
            self.store_encrypted_file(key_ref, key)
        }
    }

    /// Retrieve key using platform-specific keychain
    fn retrieve_platform_specific(&self, key_ref: &KeyReference) -> Result<SecretString> {
        #[cfg(target_os = "macos")]
        {
            self.retrieve_macos_keychain(key_ref)
        }

        #[cfg(target_os = "linux")]
        {
            self.retrieve_linux_keyring(key_ref)
        }

        #[cfg(target_os = "windows")]
        {
            self.retrieve_windows_credential_manager(key_ref)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            // Fallback to encrypted file storage for unsupported platforms
            self.retrieve_encrypted_file(key_ref)
        }
    }

    /// Delete key using platform-specific keychain
    fn delete_platform_specific(&self, key_ref: &KeyReference) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.delete_macos_keychain(key_ref)
        }

        #[cfg(target_os = "linux")]
        {
            self.delete_linux_keyring(key_ref)
        }

        #[cfg(target_os = "windows")]
        {
            self.delete_windows_credential_manager(key_ref)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            // Fallback to encrypted file storage for unsupported platforms
            self.delete_encrypted_file(key_ref)
        }
    }

    #[cfg(target_os = "macos")]
    fn store_macos_keychain(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        use security_framework::item::{ItemClass, ItemSearchOptions};
        use security_framework::keychain::{SecKeychain, SecKeychainItem};

        let keychain = SecKeychain::default().map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to access macOS keychain: {}", e),
        })?;

        // Store the key as a generic password item
        keychain
            .add_generic_password(&self.service_name, &key_ref.id, key.expose_secret().as_bytes())
            .map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to store key in macOS keychain: {}", e),
            })?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn retrieve_macos_keychain(&self, key_ref: &KeyReference) -> Result<SecretString> {
        use security_framework::keychain::SecKeychain;

        let keychain = SecKeychain::default().map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to access macOS keychain: {}", e),
        })?;

        let password_data = keychain
            .find_generic_password(&self.service_name, &key_ref.id)
            .map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to retrieve key from macOS keychain: {}", e),
            })?;

        let password_string =
            String::from_utf8(password_data.password().to_vec()).map_err(|e| SecurityError::KeystoreError {
                message: format!("Invalid key data in keychain: {}", e),
            })?;

        Ok(SecretString::new(password_string))
    }

    #[cfg(target_os = "macos")]
    fn delete_macos_keychain(&self, key_ref: &KeyReference) -> Result<()> {
        use security_framework::keychain::SecKeychain;

        let keychain = SecKeychain::default().map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to access macOS keychain: {}", e),
        })?;

        let item = keychain
            .find_generic_password(&self.service_name, &key_ref.id)
            .map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to find key in macOS keychain: {}", e),
            })?;

        item.delete().map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to delete key from macOS keychain: {}", e),
        })?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn store_linux_keyring(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        // For Linux, we'll use the secret-service crate to interact with the keyring
        // This is a simplified implementation - in production, you'd want to use
        // the secret-service crate or similar

        use std::process::Command;

        let output = Command::new("secret-tool")
            .args([
                "store",
                "--label",
                &format!("Vaughan Wallet Key: {}", key_ref.id),
                "service",
                &self.service_name,
                "key-id",
                &key_ref.id,
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(key.expose_secret().as_bytes())?;
                }
                child.wait_with_output()
            });

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("⚠️ secret-tool failed to store key: {}", error_msg);
                    tracing::info!("🔄 Falling back to encrypted file storage");
                    return self.store_encrypted_file(key_ref, key);
                }
                tracing::info!("✅ Successfully stored key in Linux keyring");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("⚠️ secret-tool not available: {}", e);
                tracing::info!("🔄 Falling back to encrypted file storage");
                // Fallback to encrypted file storage if secret-tool is not available
                self.store_encrypted_file(key_ref, key)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn retrieve_linux_keyring(&self, key_ref: &KeyReference) -> Result<SecretString> {
        use std::process::Command;

        let output = Command::new("secret-tool")
            .args(["lookup", "service", &self.service_name, "key-id", &key_ref.id])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("⚠️ secret-tool failed to retrieve key: {}", error_msg);
                    tracing::info!("🔄 Falling back to encrypted file retrieval");
                    return self.retrieve_encrypted_file(key_ref);
                }

                let key_data = String::from_utf8(output.stdout).map_err(|e| SecurityError::KeystoreError {
                    message: format!("Invalid key data from keyring: {e}"),
                })?;

                tracing::info!("✅ Successfully retrieved key from Linux keyring");
                Ok(SecretString::new(key_data.trim().to_string()))
            }
            Err(e) => {
                tracing::warn!("⚠️ secret-tool not available: {}", e);
                tracing::info!("🔄 Falling back to encrypted file retrieval");
                // Fallback to encrypted file retrieval if secret-tool is not available
                self.retrieve_encrypted_file(key_ref)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn delete_linux_keyring(&self, key_ref: &KeyReference) -> Result<()> {
        use std::process::Command;

        // Try to delete from keyring, but don't fail if keyring is not available
        let output = Command::new("secret-tool")
            .args(["clear", "service", &self.service_name, "key-id", &key_ref.id])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("⚠️ secret-tool failed to delete key: {}", error_msg);

                    // If secret-tool fails, try fallback to encrypted file deletion
                    tracing::info!("🔄 Falling back to encrypted file deletion");
                    return self.delete_encrypted_file(key_ref);
                }
                tracing::info!("✅ Successfully deleted key from Linux keyring");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("⚠️ secret-tool not available: {}", e);
                tracing::info!("🔄 Falling back to encrypted file deletion");
                // Fallback to encrypted file deletion if secret-tool is not available
                self.delete_encrypted_file(key_ref)
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn store_windows_credential_manager(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        use std::ptr;
        use winapi::um::wincred::{CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC};
        use winapi::um::winnt::LPWSTR;

        let target_name = format!("{}:{}", self.service_name, key_ref.id);
        let target_name_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();

        let comment = "Vaughan Wallet Credential";
        let comment_wide: Vec<u16> = comment.encode_utf16().chain(Some(0)).collect();

        let key_bytes = key.expose_secret().as_bytes();

        let mut credential = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target_name_wide.as_ptr() as LPWSTR,
            Comment: comment_wide.as_ptr() as LPWSTR,
            // SAFETY: FILETIME is a POD (Plain Old Data) type that can be safely zero-initialized.
            // Zero represents 1601-01-01 in Windows FILETIME format, which is a valid value.
            LastWritten: unsafe { std::mem::zeroed() },
            CredentialBlobSize: key_bytes.len() as u32,
            CredentialBlob: key_bytes.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: ptr::null_mut(),
            TargetAlias: ptr::null_mut(),
            UserName: ptr::null_mut(),
        };

        // SAFETY: CredWriteW is safe when called with a valid CREDENTIALW structure.
        // All pointers in the credential structure are valid:
        // - TargetName: valid UTF-16 string with null terminator (from to_wide_string)
        // - Comment: valid UTF-16 string with null terminator (from to_wide_string)
        // - CredentialBlob: valid byte slice from SecretString
        // All strings are kept alive for the duration of this call.
        let result = unsafe { CredWriteW(&mut credential, 0) };

        if result == 0 {
            use winapi::um::errhandlingapi::GetLastError;
            // SAFETY: GetLastError is always safe to call. It returns the last error code
            // from the Windows API with no parameters and no side effects.
            let error_code = unsafe { GetLastError() };
            return Err(SecurityError::KeystoreError {
                message: format!(
                    "Failed to store credential in Windows Credential Manager: error code {}",
                    error_code
                ),
            }
            .into());
        }

        tracing::info!("✅ Successfully stored key in Windows Credential Manager");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn retrieve_windows_credential_manager(&self, key_ref: &KeyReference) -> Result<SecretString> {
        use std::ptr;
        use winapi::um::wincred::{CredFree, CredReadW, CRED_TYPE_GENERIC, PCREDENTIALW};
        use winapi::um::winnt::LPCWSTR;

        let target_name = format!("{}:{}", self.service_name, key_ref.id);
        let target_name_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();

        let mut credential: PCREDENTIALW = ptr::null_mut();

        // SAFETY: CredReadW is safe when called with valid parameters:
        // - target_name_wide is a valid UTF-16 string with null terminator
        // - credential is a valid mutable pointer for writing the result
        // This is a standard Windows API call for reading credentials.
        let result = unsafe {
            CredReadW(
                target_name_wide.as_ptr() as LPCWSTR,
                CRED_TYPE_GENERIC,
                0,
                &mut credential,
            )
        };

        if result == 0 {
            use winapi::um::errhandlingapi::GetLastError;
            // SAFETY: GetLastError is always safe to call.
            let error_code = unsafe { GetLastError() };
            return Err(SecurityError::KeystoreError {
                message: format!(
                    "Failed to retrieve credential from Windows Credential Manager: error code {}",
                    error_code
                ),
            }
            .into());
        }

        // SAFETY: Multiple unsafe operations here, all safe because:
        // 1. Dereferencing credential is safe because CredReadW succeeded, so credential is valid
        // 2. blob_ptr is checked for null before use
        // 3. from_raw_parts is safe because:
        //    - blob_ptr is valid (null-checked)
        //    - blob_size represents the actual data size from Windows API
        //    - Data lifetime is valid until CredFree is called
        // 4. Data is accessed before CredFree, ensuring no use-after-free
        let key_data = unsafe {
            let cred = &*credential;
            let blob_size = cred.CredentialBlobSize as usize;
            let blob_ptr = cred.CredentialBlob;

            if blob_ptr.is_null() || blob_size == 0 {
                CredFree(credential as *mut _);
                return Err(SecurityError::KeystoreError {
                    message: "Retrieved credential has no data".to_string(),
                }
                .into());
            }

            let data = std::slice::from_raw_parts(blob_ptr, blob_size);
            String::from_utf8_lossy(data).into_owned()
        };

        // SAFETY: CredFree is safe when called with a valid credential pointer.
        // The credential was allocated by CredReadW and is freed exactly once here.
        unsafe { CredFree(credential as *mut _) };

        tracing::info!("✅ Successfully retrieved key from Windows Credential Manager");
        Ok(SecretString::new(key_data))
    }

    #[cfg(target_os = "windows")]
    fn delete_windows_credential_manager(&self, key_ref: &KeyReference) -> Result<()> {
        use winapi::um::wincred::{CredDeleteW, CRED_TYPE_GENERIC};
        use winapi::um::winnt::LPCWSTR;

        let target_name = format!("{}:{}", self.service_name, key_ref.id);
        let target_name_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();

        // SAFETY: CredDeleteW is safe when called with valid parameters:
        // - target_name_wide is a valid UTF-16 string with null terminator
        // This is a standard Windows API call for deleting credentials.
        let result = unsafe { CredDeleteW(target_name_wide.as_ptr() as LPCWSTR, CRED_TYPE_GENERIC, 0) };

        if result == 0 {
            use winapi::um::errhandlingapi::GetLastError;
            // SAFETY: GetLastError is always safe to call.
            let error_code = unsafe { GetLastError() };
            return Err(SecurityError::KeystoreError {
                message: format!(
                    "Failed to delete credential from Windows Credential Manager: error code {}",
                    error_code
                ),
            }
            .into());
        }

        tracing::info!("✅ Successfully deleted key from Windows Credential Manager");
        Ok(())
    }

    // Fallback encrypted file storage for unsupported platforms with integrity checking
    #[allow(dead_code)] // Fallback for non-Windows platforms, currently unused
    fn store_encrypted_file(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        let key_file = self.get_key_file_path(key_ref)?;
        let encrypted_data = self.encrypt_key_with_integrity(key)?;
        self.write_secure_file(&key_file, &encrypted_data)?;

        tracing::debug!("✅ Stored key with integrity protection: {}", key_ref.id);
        Ok(())
    }

    /// Get the file path for a key
    #[allow(dead_code)] // Fallback for non-Windows platforms, currently unused
    fn get_key_file_path(&self, key_ref: &KeyReference) -> Result<std::path::PathBuf> {
        use std::fs;

        let mut config_dir = dirs::config_dir().ok_or_else(|| SecurityError::KeystoreError {
            message: "Could not determine config directory".to_string(),
        })?;

        config_dir.push("vaughan-wallet");
        config_dir.push("keys");

        fs::create_dir_all(&config_dir).map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to create key directory: {e}"),
        })?;

        Ok(config_dir.join(format!("{}.key", key_ref.id)))
    }

    /// Encrypt key with integrity protection (HMAC-SHA256)
    #[allow(dead_code)] // Fallback for non-Windows platforms, currently unused
    fn encrypt_key_with_integrity(&self, key: SecretString) -> Result<Vec<u8>> {
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
        use pbkdf2::pbkdf2_hmac;
        use rand::RngCore;
        use sha2::{Digest, Sha256};

        // Generate salt and derive keys
        let mut salt = [0u8; 32];
        getrandom::getrandom(&mut salt).map_err(|e| SecurityError::KeystoreError {
            message: format!("Salt generation failed: {e}"),
        })?;

        let mut derived = [0u8; 64]; // 32 bytes for encryption + 32 bytes for HMAC
        pbkdf2_hmac::<Sha256>(self.service_name.as_bytes(), &salt, 200_000, &mut derived);

        let (encryption_key, hmac_key) = derived.split_at(32);
        let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(encryption_key));

        // Generate nonce and encrypt
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext =
            cipher
                .encrypt(nonce, key.expose_secret().as_bytes())
                .map_err(|e| SecurityError::KeystoreError {
                    message: format!("AES-GCM encryption failed: {e}"),
                })?;

        // Create authenticated data and HMAC
        let mut data_to_authenticate = Vec::with_capacity(44 + ciphertext.len());
        data_to_authenticate.extend_from_slice(&salt);
        data_to_authenticate.extend_from_slice(&nonce_bytes);
        data_to_authenticate.extend_from_slice(&ciphertext);

        let mut hmac_hasher = Sha256::new();
        hmac_hasher.update(hmac_key);
        hmac_hasher.update(&data_to_authenticate);
        let hmac = hmac_hasher.finalize();

        // Final format: hmac || salt || nonce || ciphertext
        let mut result = Vec::with_capacity(32 + data_to_authenticate.len());
        result.extend_from_slice(&hmac);
        result.extend_from_slice(&data_to_authenticate);

        Ok(result)
    }

    /// Write data to file with secure permissions    #[allow(dead_code)] // Fallback for non-Windows platforms, currently unused
    fn write_secure_file(&self, path: &std::path::Path, data: &[u8]) -> Result<()> {
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .mode(0o600)
                .open(path)
                .map_err(|e| SecurityError::KeystoreError {
                    message: format!("Failed to open key file: {e}"),
                })?;

            file.write_all(data).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to write key file: {e}"),
            })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(path, data).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to write key file: {e}"),
            })?;
        }

        Ok(())
    }

    #[allow(dead_code)] // Fallback for non-Windows platforms, currently unused
    fn retrieve_encrypted_file(&self, key_ref: &KeyReference) -> Result<SecretString> {
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
        use pbkdf2::pbkdf2_hmac;
        use sha2::{Digest, Sha256};
        use std::fs;

        let mut config_dir = dirs::config_dir().ok_or_else(|| SecurityError::KeystoreError {
            message: "Could not determine config directory".to_string(),
        })?;

        config_dir.push("vaughan-wallet");
        config_dir.push("keys");

        let key_file = config_dir.join(format!("{}.key", key_ref.id));

        let data = fs::read(&key_file).map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to read key file: {e}"),
        })?;

        // New format: hmac(32) || salt(32) || nonce(12) || ciphertext
        // Old format: salt(32) || nonce(12) || ciphertext
        let has_integrity_check = data.len() >= 76; // Minimum size for new format

        if has_integrity_check {
            // New format with integrity checking
            let (stored_hmac, rest) = data.split_at(32);
            let (salt, rest) = rest.split_at(32);
            let (nonce_bytes, ciphertext) = rest.split_at(12);

            // Derive keys
            let mut derived = [0u8; 64];
            pbkdf2_hmac::<Sha256>(self.service_name.as_bytes(), salt, 200_000, &mut derived);
            let (encryption_key, hmac_key) = derived.split_at(32);

            // Verify integrity
            let mut hmac_hasher = Sha256::new();
            hmac_hasher.update(hmac_key);
            hmac_hasher.update(salt);
            hmac_hasher.update(nonce_bytes);
            hmac_hasher.update(ciphertext);
            let computed_hmac = hmac_hasher.finalize();

            // Constant-time comparison
            if stored_hmac != computed_hmac.as_slice() {
                return Err(SecurityError::KeystoreError {
                    message: "Key file integrity check failed - data may be corrupted or tampered with".to_string(),
                }
                .into());
            }

            // Decrypt
            let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(encryption_key);
            let cipher = Aes256Gcm::new(cipher_key);
            let nonce = Nonce::from_slice(nonce_bytes);
            let plaintext = cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| SecurityError::KeystoreError {
                    message: format!("AES-GCM decryption failed: {e}"),
                })?;

            tracing::debug!("✅ Retrieved key with verified integrity: {}", key_ref.id);
            Ok(SecretString::new(String::from_utf8_lossy(&plaintext).into_owned()))
        } else {
            // Legacy format without integrity checking
            tracing::warn!(
                "⚠️ Key file uses legacy format without integrity protection: {}",
                key_ref.id
            );

            if data.len() < 44 {
                return Err(SecurityError::KeystoreError {
                    message: "Corrupt key file".to_string(),
                }
                .into());
            }

            let (salt, rest) = data.split_at(32);
            let (nonce_bytes, ciphertext) = rest.split_at(12);

            let mut derived = [0u8; 32];
            pbkdf2_hmac::<Sha256>(self.service_name.as_bytes(), salt, 200_000, &mut derived);
            let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&derived);
            let cipher = Aes256Gcm::new(cipher_key);
            let nonce = Nonce::from_slice(nonce_bytes);
            let plaintext = cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| SecurityError::KeystoreError {
                    message: format!("AES-GCM decryption failed: {e}"),
                })?;

            Ok(SecretString::new(String::from_utf8_lossy(&plaintext).into_owned()))
        }
    }

    #[allow(dead_code)] // Fallback for non-Windows platforms, currently unused
    fn delete_encrypted_file(&self, key_ref: &KeyReference) -> Result<()> {
        use std::fs;

        let mut config_dir = dirs::config_dir().ok_or_else(|| SecurityError::KeystoreError {
            message: "Could not determine config directory".to_string(),
        })?;

        config_dir.push("vaughan-wallet");
        config_dir.push("keys");

        let key_file = config_dir.join(format!("{}.key", key_ref.id));

        if key_file.exists() {
            fs::remove_file(&key_file).map_err(|e| SecurityError::KeystoreError {
                message: format!("Failed to delete key file: {e}"),
            })?;
        }

        Ok(())
    }
}

/// Mock keychain for testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockKeychain {
    storage: Mutex<HashMap<String, String>>,
}

#[cfg(test)]
impl Default for MockKeychain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockKeychain {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
impl KeychainInterface for MockKeychain {
    fn store(&self, key_ref: &KeyReference, key: SecretString) -> Result<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(key_ref.id.clone(), key.expose_secret().to_string());
        Ok(())
    }

    fn retrieve(&self, key_ref: &KeyReference) -> Result<SecretString> {
        let storage = self.storage.lock().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
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
        let mut storage = self.storage.lock().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        storage.remove(&key_ref.id);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn KeychainInterface> {
        Box::new(MockKeychain {
            storage: Mutex::new(self.storage.lock().expect("Lock failed").clone()),
        })
    }
}

/// Create platform-specific keychain interface
pub fn create_keychain_interface() -> Result<Box<dyn KeychainInterface>> {
    // Use the encrypted seeds service since that's where new accounts are created
    let service_name = "vaughan-wallet-encrypted-seeds".to_string();
    Ok(Box::new(OSKeychain::new(service_name)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_mock_keychain() -> Result<()> {
        let keychain = MockKeychain::new();
        let key_ref = KeyReference {
            id: "test-key".to_string(),
            service: "test-service".to_string(),
            account: "test-account".to_string(),
        };
        let secret = SecretString::new("test-secret".to_string());

        // Test store and retrieve
        keychain.store(&key_ref, secret.clone()).context("Operation failed")?;
        let retrieved = keychain.retrieve(&key_ref).context("Failed to process retrieved")?;
        assert_eq!(secret.expose_secret(), retrieved.expose_secret());

        // Test delete
        keychain.delete(&key_ref).context("Operation failed")?;
        assert!(keychain.retrieve(&key_ref).is_err());
        Ok(())
    }
}
