# Design Document: Comprehensive Debloat Plan

## Overview

This design document outlines the systematic approach to reducing code complexity, binary size, and technical debt in the Vaughan wallet application. This enhanced version includes additional large files identified (dialogs.rs, keystore.rs), improved consolidation strategies based on the existing handler pattern, better feature gating approach, and comprehensive security considerations for critical modules.

The design prioritizes safety and feature preservation. Each phase includes verification steps to ensure the wallet remains fully functional. The approach is inspired by MetaMask's modular architecture and follows Rust best practices for code organization.

## Architecture - Current State Analysis (Enhanced)

### Identified Bloat Files

```
src/
├── gui/
│   ├── working_wallet.rs    (5,242 lines) ← BLOAT: Monolithic GUI
│   ├── theme.rs             (1,620 lines) ← BLOAT: Many unused styles
│   ├── views/
│   │   └── dialogs.rs        (1,327 lines) ← BLOAT: Multiple dialogs (NEW)
│   └── handlers/           (1,908 lines) ← GOOD: Already organized
│       ├── mod.rs              (50 lines)
│       ├── network.rs          (600 lines)
│       ├── security.rs         (700 lines)
│       ├── transaction.rs      (800 lines)
│       ├── ui_state.rs        (400 lines)
│       └── wallet_ops.rs      (600 lines)
│   └── components/
│       └── dialogs/
│           ├── password_dialog.rs        (8KB)  ┐
│           ├── master_password_dialog.rs (7KB)  ├─ CONSOLIDATE
│           └── wallet_password_dialog.rs (27KB) ┘
├── security/
│   ├── seed.rs              (2,918 lines) ← BLOAT: Multiple responsibilities
│   ├── keystore.rs          (1,110 lines) ← BLOAT: Account storage + encryption (NEW)
│   ├── hardware.rs           (1,703 lines) ← BLOAT: Future-planned code
│   └── keychain.rs          (797 lines) ← MEDIUM: Close to threshold
├── wallet/
│   └── manager.rs          (787 lines)
├── tokens/
│   └── mod.rs              (732 lines)
└── network/
    ├── professional.rs        (1,310 lines) ← BLOAT: Unused monitoring
    └── config.rs
```

### Key Discoveries

1. **Handler Pattern is Excellent**: The existing `src/gui/handlers/` directory demonstrates good organizational practices that should be extended to working_wallet.rs decomposition.

2. **dialogs.rs Contains Multiple Dialog Types**: Custom token dialog, confirmation dialogs, and receive dialog are all in one file.

3. **keystore.rs Has Multiple Responsibilities**: Account management, encryption, key storage, and network storage are all mixed.

4. **Password Dialog Duplication**: Three separate password dialog files with significant code duplication.

5. **State Management Overlap**: security_state.rs and session_state.rs both handle passwords, sessions, and lockout logic.

## Architecture - Target State (Enhanced)

```
src/
├── gui/
│   ├── wallet/                    ← NEW: Decomposed from working_wallet.rs
│   │   ├── mod.rs                 (150 lines) - WalletState, Message enum
│   │   ├── update.rs              (200 lines) - Message routing to handlers
│   │   ├── view.rs                (250 lines) - Main view composition
│   │   ├── handlers/              ← MOVED & ENHANCED from src/gui/handlers/
│   │   │   ├── mod.rs             (50 lines) - Handler exports
│   │   │   ├── network.rs         (600 lines) - Network messages
│   │   │   ├── security.rs        (700 lines) - Security/auth messages
│   │   │   ├── transaction.rs     (800 lines) - Transaction messages
│   │   │   ├── ui_state.rs        (400 lines) - UI state messages
│   │   │   └── wallet_ops.rs     (600 lines) - Wallet operations
│   │   └── components/            ← NEW: UI components extracted
│   │       ├── send.rs            (400 lines) - Send form
│   │       ├── receive.rs         (200 lines) - Receive/QR
│   │       └── tokens.rs         (300 lines) - Token management
│   ├── theme/                     ← NEW: Decomposed from theme.rs
│   │   ├── mod.rs                 (100 lines) - Theme exports
│   │   ├── colors.rs              (300 lines) - Color definitions
│   │   ├── buttons.rs             (250 lines) - Button styles
│   │   ├── containers.rs          (250 lines) - Container styles
│   │   ├── text.rs                (200 lines) - Text styles
│   │   └── unused.rs             (50 lines) - Unused styles for reference
│   ├── components/
│   │   └── dialogs/              ← NEW: Decomposed from views/dialogs.rs
│   │       ├── mod.rs             (100 lines) - Dialog exports
│   │       ├── custom_token.rs     (500 lines) - Custom token dialog
│   │       ├── confirmation.rs     (300 lines) - Confirmation dialogs
│   │       ├── receive.rs         (200 lines) - Receive dialog
│   │       └── password_dialog.rs ← CONSOLIDATED: Single configurable dialog
│   └── state/
│       ├── auth_state.rs          ← CONSOLIDATED: security + session (800 lines)
│       ├── network_state.rs       (unchanged)
│       ├── transaction_state.rs   (unchanged)
│       └── ui_state.rs         (single source of truth)
├── security/
│   ├── seed/                      ← NEW: Decomposed from seed.rs
│   │   ├── mod.rs                 (200 lines) - Public API, SecureSeed
│   │   ├── encryption.rs          (400 lines) - Seed encryption/decryption
│   │   ├── derivation.rs          (400 lines) - Key derivation
│   │   ├── backup.rs              (350 lines) - Backup/restore
│   │   ├── validation.rs          (250 lines) - Seed validation
│   │   └── zeroization.rs        ← NEW: Memory zeroization (150 lines)
│   ├── keystore/                  ← NEW: Decomposed from keystore.rs
│   │   ├── mod.rs                 (150 lines) - Keystore trait, exports
│   │   ├── storage.rs             (300 lines) - File persistence
│   │   ├── encryption.rs          (250 lines) - Key encryption/decryption
│   │   ├── account.rs             (250 lines) - Account management
│   │   └── network.rs             (150 lines) - Custom network storage
│   └── hardware.rs               ← CLEANED: Removed dead code, feature-gated
├── wallet/
│   └── manager.rs                (unchanged)
├── tokens/
│   └── mod.rs                   (unchanged)
└── network/
    ├── professional.rs            ← CLEANED: Removed dead code, feature-gated
    └── config.rs                (unchanged)

tests/
├── password_integration_tests.rs  ← RELOCATED from src/
├── security_state_tests.rs        ← RELOCATED from src/
├── session_property_tests.rs      ← RELOCATED from src/
├── integration_test.rs            ← RELOCATED from src/
├── professional_test.rs           ← RELOCATED from src/
├── hardware_tests.rs              ← RELOCATED from src/
└── multicall_test.rs              ← RELOCATED from src/
```

## Components and Interfaces (Enhanced)

### 1. Dead Code Audit System (Enhanced)

**Purpose:** Systematically identify and handle dead code annotations with security considerations.

**Process:**
```rust
// Category 1: Serde fields - KEEP with documentation
#[allow(dead_code)] // Required for serde deserialization
pub struct TokenPrice {
    symbol: String,
    price: f64,
}

// Category 2: Planned features - EVALUATE WITH TIMELINE
// If not implementing within 3 months, remove
#[allow(dead_code)] // TODO: Implement in v0.2.0 (due: 2026-03-01)
fn future_feature() {}

// Category 3: Future keychain operations - DECIDE WITH ROADMAP
// If roadmap includes this within 6 months: document timeline
// If not: remove and re-implement when needed
#[allow(dead_code)] // Planned for Q2 2026 (roadmap: multi-device support)
fn keychain_operation() {}

// Category 4: Security-critical unused - DOCUMENT SECURITY REASON
// Only keep if zeroization strategy exists
#[allow(dead_code)] // Stored for future keychain operations (see: SECURITY.md)
fn secure_unused() {}

// Category 5: Unused handlers - REMOVE ENTIRELY
// DELETE THIS CODE COMPLETELY
// fn unused_handler() {}
```

**Files to Audit (Enhanced):**
| File | Annotations | Action | Security Consideration |
|------|-------------|--------|----------------------|
| tokens/pricing.rs | 6 | Keep (serde) | No |
| network/professional.rs | 2 | Evaluate/Remove | No |
| security/hardware.rs | 3 | Evaluate/Remove | Yes - feature gate |
| security/keystore.rs | 1 | Evaluate/Remove | Yes - zeroization |
| security/seed.rs | 1 | Evaluate/Remove | Yes - zeroization |
| security/password_validator.rs | 1 | Evaluate/Remove | No |
| gui/handlers/network.rs | 4 | Remove | No |
| gui/theme.rs | 1 | Evaluate/Remove | No |
| gui/simple_transaction.rs | 2 | Evaluate/Remove | No |

### 2. dialogs.rs Decomposition (NEW)

**Purpose:** Split 1,327-line dialogs.rs into logical dialog components.

**Module Structure:**

```rust
// src/gui/components/dialogs/mod.rs
pub mod custom_token;
pub mod confirmation;
pub mod receive;

// Re-export commonly used types
pub use custom_token::CustomTokenDialogState;
pub use confirmation::{ConfirmationDialog, ConfirmationDialogType};
pub use receive::ReceiveDialogState;

/// Common dialog utilities
pub fn dialog_container<'a, Message>(
    title: &str,
    content: Element<'a, Message>,
    buttons: Row<'a, Message>,
) -> Container<'a, Message> {
    // Common dialog styling
}
```

```rust
// src/gui/components/dialogs/custom_token.rs
//! Custom token dialog component

use iced::{Element, ...};
use crate::gui::{Message, AppState};

pub struct CustomTokenDialogState {
    pub visible: bool,
    pub address_input: String,
    pub symbol_input: String,
    pub decimals_input: String,
    pub validation_error: Option<String>,
    pub fetching_info: bool,
}

impl CustomTokenDialogState {
    pub fn show(&mut self) {
        self.visible = true;
        self.reset();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.clear_sensitive_data();
    }

    fn reset(&mut self) {
        self.address_input.clear();
        self.symbol_input.clear();
        self.decimals_input.clear();
        self.validation_error = None;
        self.fetching_info = false;
    }

    fn clear_sensitive_data(&mut self) {
        // Clear any sensitive inputs
    }

    pub fn validate(&self) -> Result<(), TokenValidationError> {
        // Validate address format
        // Validate symbol format
        // Validate decimals input
    }
}

pub fn custom_token_dialog_view(state: &CustomTokenDialogState) -> Element<Message> {
    // UI rendering for custom token dialog
}
```

```rust
// src/gui/components/dialogs/confirmation.rs
//! Generic confirmation dialog components

use iced::Element;
use crate::gui::Message;

#[derive(Debug, Clone)]
pub enum ConfirmationDialogType {
    ResetWallet,
    DeleteAccount { account_name: String },
    DeleteNetwork { network_name: String },
    ClearLogs,
}

pub struct ConfirmationDialogState {
    pub visible: bool,
    pub dialog_type: Option<ConfirmationDialogType>,
    pub confirmed: bool,
}

impl ConfirmationDialogState {
    pub fn show(&mut self, dialog_type: ConfirmationDialogType) {
        self.visible = true;
        self.dialog_type = Some(dialog_type);
        self.confirmed = false;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.dialog_type = None;
        self.confirmed = false;
    }

    pub fn confirm(&mut self) {
        self.confirmed = true;
    }
}

pub fn confirmation_dialog_view(state: &ConfirmationDialogState) -> Element<Message> {
    let title = match &state.dialog_type {
        Some(ConfirmationDialogType::ResetWallet) => "Reset Wallet",
        Some(ConfirmationDialogType::DeleteAccount { account_name }) => {
            &format!("Delete Account - {}", account_name)
        }
        Some(ConfirmationDialogType::DeleteNetwork { network_name }) => {
            &format!("Delete Network - {}", network_name)
        }
        Some(ConfirmationDialogType::ClearLogs) => "Clear Error Logs",
        None => "Confirm",
    };

    let message = match &state.dialog_type {
        Some(ConfirmationDialogType::ResetWallet) => {
            "This will reset your wallet and delete all accounts. This action cannot be undone."
        }
        Some(ConfirmationDialogType::DeleteAccount { .. }) => {
            "This will delete the selected account. This action cannot be undone."
        }
        Some(ConfirmationDialogType::DeleteNetwork { .. }) => {
            "This will delete the selected custom network. This action cannot be undone."
        }
        Some(ConfirmationDialogType::ClearLogs) => {
            "This will clear all error logs. This action cannot be undone."
        }
        None => "Are you sure you want to continue?",
    };

    // Build confirmation dialog UI
}
```

### 3. keystore.rs Decomposition (NEW)

**Purpose:** Split 1,110-line keystore.rs into focused, auditable modules.

**Module Structure:**

```rust
// src/security/keystore/mod.rs
pub mod storage;
pub mod encryption;
pub mod account;
pub mod network;

pub use storage::{SecureKeystoreImpl, load_keystore, save_keystore};
pub use encryption::{encrypt_key, decrypt_key};
pub use account::{StoredAccountMeta, create_account, delete_account};
pub use network::{StoredNetworkMeta, save_network, delete_network};

/// Public keystore interface
#[async_trait]
pub trait Keystore: Send + Sync {
    async fn create_account(&mut self, name: String) -> Result<SecureAccount>;
    async fn delete_account(&mut self, address: Address) -> Result<()>;
    async fn get_account(&self, address: Address) -> Option<SecureAccount>;
    async fn list_accounts(&self) -> Vec<SecureAccount>;
    async fn save_network(&mut self, network: NetworkConfig) -> Result<()>;
    async fn delete_network(&mut self, id: NetworkId) -> Result<()>;
    async fn list_networks(&self) -> Vec<NetworkConfig>;
}
```

```rust
// src/security/keystore/storage.rs
//! Keystore file persistence

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Keystore file format
#[derive(Debug, Serialize, Deserialize)]
pub struct KeystoreFile {
    pub version: String,
    pub accounts: Vec<StoredAccountMeta>,
    pub networks: Vec<StoredNetworkMeta>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl KeystoreFile {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            accounts: Vec::new(),
            networks: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }
}

pub async fn load_keystore(path: PathBuf) -> Result<KeystoreFile> {
    // Load from file with proper error handling
    let content = tokio::fs::read_to_string(&path).await
        .map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to read keystore: {}", e),
        })?;

    serde_json::from_str(&content)
        .map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to parse keystore: {}", e),
        }.into())
}

pub async fn save_keystore(path: PathBuf, keystore: &KeystoreFile) -> Result<()> {
    // Serialize and save with atomic write
    let content = serde_json::to_string_pretty(keystore)
        .map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to serialize keystore: {}", e),
        })?;

    // Atomic write: write to temp file then rename
    let temp_path = path.with_extension("tmp");
    tokio::fs::write(&temp_path, content).await
        .map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to write keystore: {}", e),
        })?;

    tokio::fs::rename(temp_path, path).await
        .map_err(|e| SecurityError::KeystoreError {
            message: format!("Failed to commit keystore: {}", e),
        })?;

    Ok(())
}
```

```rust
// src/security/keystore/encryption.rs
//! Key encryption and decryption

use crate::error::Result;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Argon2, PasswordHasher};

/// Encrypted key format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
    pub argon2_params: Argon2Params,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Params {
    pub t_cost: u32,
    pub m_cost: u32,
    pub p_cost: u32,
    pub output_len: usize,
}

pub fn encrypt_key(key: &[u8], password: &str) -> Result<EncryptedKey> {
    // Argon2 key derivation
    let salt = random_bytes::<16>();
    let mut key_bytes = [0u8; 32];

    let argon2 = Argon2::default();
    argon2.hash_password_into_raw(
        password.as_bytes(),
        &salt,
        &mut key_bytes,
    )
    .map_err(|e| SecurityError::EncryptionError {
        message: format!("Key derivation failed: {}", e),
    })?;

    let key = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| SecurityError::EncryptionError {
            message: format!("Failed to initialize cipher: {}", e),
        })?;

    let nonce = random_bytes::<12>();
    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher.encrypt(&nonce.into(), key)
        .map_err(|e| SecurityError::EncryptionError {
            message: format!("Encryption failed: {}", e),
        })?
        .to_vec();

    Ok(EncryptedKey {
        ciphertext,
        nonce: nonce.to_vec(),
        salt: salt.to_vec(),
        argon2_params: Argon2Params {
            t_cost: argon2.t_cost(),
            m_cost: argon2.m_cost(),
            p_cost: argon2.p_cost(),
            output_len: key_bytes.len(),
        },
    })
}

pub fn decrypt_key(encrypted: &EncryptedKey, password: &str) -> Result<Vec<u8>> {
    // Argon2 key derivation with stored params
    let mut key_bytes = [0u8; 32];

    let argon2 = Argon2::new(
        encrypted.argon2_params.t_cost,
        encrypted.argon2_params.m_cost,
        encrypted.argon2_params.p_cost,
        argon2::Version::V0x13,
        &encrypted.argon2_params.output_len,
    );

    argon2.hash_password_into_raw(
        password.as_bytes(),
        &encrypted.salt,
        &mut key_bytes,
    )
    .map_err(|e| SecurityError::EncryptionError {
        message: format!("Key derivation failed: {}", e),
    })?;

    let key = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| SecurityError::EncryptionError {
            message: format!("Failed to initialize cipher: {}", e),
        })?;

    let nonce = <[u8; 12]>::try_from(encrypted.nonce.as_slice())
        .map_err(|_| SecurityError::EncryptionError {
            message: "Invalid nonce length".to_string(),
        })?;

    let cipher = Aes256Gcm::new(&key);
    cipher.decrypt(&nonce.into(), &encrypted.ciphertext[..], key)
        .map_err(|e| SecurityError::EncryptionError {
            message: format!("Decryption failed: {}", e),
        })
}
```

### 4. Working Wallet Decomposition (Enhanced)

**Purpose:** Split 5,242-line working_wallet.rs following the handler pattern.

**Module Structure:**

```rust
// src/gui/wallet/mod.rs
pub mod update;
pub mod view;
pub mod handlers;
pub mod components;

pub use update::*;
pub use view::*;

/// Main wallet application state
pub struct WorkingWalletApp {
    pub state: AppState,
    pub wallet: Option<Arc<tokio::sync::RwLock<Vaughan>>>,
    pub api_manager: Option<ExplorerApiManager>,
}

/// All possible messages the wallet can handle
pub enum Message {
    // Core wallet messages
    WalletLoaded(Result<(), String>),
    AccountsLoaded(Vec<SecureAccount>),

    // Network messages
    NetworkChanged(NetworkId),
    NetworksLoaded(Vec<NetworkConfig>),

    // Transaction messages
    SendTransaction,
    TransactionConfirmed(Result<TxHash, String>),

    // UI messages
    TabChanged(Tab),
    DialogClosed,
    // ... etc
}
```

```rust
// src/gui/wallet/update.rs
//! Message routing to specialized handlers

impl WorkingWalletApp {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        // Route messages to specialized handlers
        match message.clone() {
            // Transaction-related messages
            Message::EstimateGas
            | Message::GasEstimated(_)
            | Message::ShowTransactionConfirmation
            | Message::HideTransactionConfirmation
            | Message::ConfirmTransaction
            | Message::SubmitTransaction
            | Message::TransactionSubmitted(_)
            | Message::TransactionMonitoringTick => {
                return self.handle_transaction_message(message);
            }

            // Network-related messages
            Message::NetworkSelected(_)
            | Message::SmartPollTick
            | Message::BalanceChanged(_, _) => {
                return self.handle_network_message(message);
            }

            // Security/Hardware wallet messages
            Message::ConnectHardwareWallet(_)
            | Message::HardwareWalletConnected(_)
            | Message::GetHardwareAddresses(_)
            | Message::HardwareAddressesReceived(_)
            | Message::ScanHardwareWallets
            | Message::RefreshHardwareWallets
            | Message::ConnectToHardwareWallet(_)
            // Password dialog messages
            | Message::ShowPasswordDialog { .. }
            | Message::HidePasswordDialog
            | Message::PasswordInputChanged(_)
            | Message::PasswordRememberChanged(_)
            | Message::SubmitPassword
            | Message::ShowResetWalletConfirmation
            | Message::HideResetWalletConfirmation
            | Message::ConfirmResetWallet
            | Message::WalletResetComplete
            | Message::PasswordValidated(_)
            // Session management messages
            | Message::SessionLocked
            | Message::SessionUnlocked
            | Message::ExtendSession
            | Message::ManualLock
            | Message::SessionTimeoutCheck => {
                return self.handle_security_message(message);
            }

            // UI state messages
            Message::ShowCreateDialog
            | Message::HideCreateDialog
            | Message::ShowImportDialog
            | Message::HideImportDialog
            | Message::ShowSettingsDialog
            | Message::HideSettingsDialog
            | Message::ShowImportWallet
            | Message::HideImportWallet
            | Message::ShowExportWallet
            | Message::HideExportWallet
            | Message::SendToAddressChanged(_)
            | Message::SendAmountChanged(_)
            | Message::SendGasLimitChanged(_)
            | Message::SendGasPriceChanged(_)
            | Message::SendTokenChanged(_)
            | Message::CreateAccountNameChanged(_)
            | Message::ImportPrivateKeyChanged(_)
            | Message::ImportAccountNameChanged(_)
            | Message::ClearStatusMessage
            | Message::UpdateLastActivity
            | Message::ToggleAccountDropdown
            | Message::SelectExportAccount(_) => {
                return self.handle_ui_state_message(message);
            }

            // Wallet operation messages
            Message::CreateAccount
            | Message::AccountCreated(_)
            | Message::ImportAccount
            | Message::AccountImported(_)
            | Message::AccountSelected(_)
            | Message::DeleteAccount(_)
            | Message::RefreshBalance
            | Message::InternalRefreshBalance
            | Message::BalanceRefreshed(_)
            | Message::TokenBalancesRefreshed(_)
            | Message::UpdateAccountBalance
            | Message::RefreshTransactionHistory
            | Message::TransactionHistoryRefreshed(_) => {
                return self.handle_wallet_ops_message(message);
            }

            // Receive dialog messages
            Message::ShowReceiveDialog
            | Message::HideReceiveDialog
            | Message::CopyToClipboard(_) => {
                return self.handle_receive_message(message);
            }

            // Core messages handled directly
            _ => {}
        }

        // Handle core messages that don't belong to specialized handlers
        match message {
            Message::SeedAccountsChecked(has_seed_accounts) => {
                // Core startup logic
            }
            // ... other core messages
            _ => Command::none(),
        }
    }
}
```

### 5. Password Dialog Consolidation (Enhanced)

**Purpose:** Merge three password dialog files into one configurable component with security considerations.

**Unified Interface (Enhanced):**

```rust
// src/gui/components/dialogs/password_dialog.rs

/// Password dialog configuration
#[derive(Debug, Clone)]
pub enum PasswordDialogConfig {
    /// Unlock wallet session
    UnlockSession,
    /// Confirm transaction signing
    SignTransaction {
        tx_summary: String,
        to_address: String,
        amount: String,
    },
    /// Export private key
    ExportKey {
        account_name: String,
        account_address: String,
    },
    /// Create new wallet
    CreateWallet {
        is_hd: bool,
        mnemonic_check: Option<String>,
    },
    /// Import wallet
    ImportWallet {
        is_mnemonic: bool,
        is_private_key: bool,
    },
    /// Change password
    ChangePassword {
        current_required: bool,
    },
    /// HD wallet master password
    MasterPassword {
        creating_wallet: bool,
    },
}

/// Password dialog state
pub struct PasswordDialogState {
    pub visible: bool,
    pub config: Option<PasswordDialogConfig>,
    pub password_input: String,
    pub confirm_input: Option<String>,
    pub show_password: bool,
    pub remember_session: bool,
    pub error_message: Option<String>,
    pub attempts_remaining: u32,
    pub locked_until: Option<Instant>,
}

impl PasswordDialogState {
    pub fn show(&mut self, config: PasswordDialogConfig) {
        self.visible = true;
        self.config = Some(config);
        self.password_input.clear();
        self.confirm_input = match config {
            PasswordDialogConfig::CreateWallet { .. }
            | PasswordDialogConfig::ImportWallet { .. }
            | PasswordDialogConfig::ChangePassword { .. } => Some(String::new()),
            _ => None,
        };
        self.show_password = false;
        self.error_message = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.config = None;
        // SECURITY: Zeroize password inputs
        self.password_input.zeroize();
        if let Some(ref mut confirm) = self.confirm_input {
            confirm.zeroize();
            self.confirm_input = None;
        }
        self.show_password = false;
    }

    pub fn is_creation(&self) -> bool {
        matches!(
            self.config,
            Some(PasswordDialogConfig::CreateWallet { .. })
                | Some(PasswordDialogConfig::ImportWallet { .. })
                | Some(PasswordDialogConfig::ChangePassword { .. })
        )
    }

    pub fn requires_confirmation(&self) -> bool {
        self.confirm_input.is_some()
    }
}

/// Render the password dialog based on configuration
pub fn password_dialog_view(state: &PasswordDialogState) -> Element<Message> {
    let title = match &state.config {
        Some(PasswordDialogConfig::UnlockSession) => "Unlock Wallet",
        Some(PasswordDialogConfig::SignTransaction { .. }) => "Confirm Transaction",
        Some(PasswordDialogConfig::ExportKey { account_name, .. }) => {
            &format!("Export Key - {}", account_name)
        }
        Some(PasswordDialogConfig::CreateWallet { .. }) => "Create Password",
        Some(PasswordDialogConfig::ImportWallet { .. }) => "Set Password",
        Some(PasswordDialogConfig::ChangePassword { .. }) => "Change Password",
        Some(PasswordDialogConfig::MasterPassword { .. }) => "HD Wallet Password",
        None => return Container::new(Space::with_height(Length::Shrink)).into(),
    };

    let description = match &state.config {
        Some(PasswordDialogConfig::UnlockSession) => {
            "Enter your password to unlock the wallet."
        }
        Some(PasswordDialogConfig::SignTransaction {
            tx_summary,
            to_address,
            amount,
        }) => {
            &format!(
                "Confirm transaction:\n\nTo: {}\nAmount: {}\n\nSummary: {}",
                to_address, amount, tx_summary
            )
        }
        Some(PasswordDialogConfig::ExportKey { account_address, .. }) => {
            &format!("Export private key for:\n\n{}", account_address)
        }
        Some(PasswordDialogConfig::CreateWallet { .. }) => {
            "Create a strong password to protect your wallet."
        }
        Some(PasswordDialogConfig::ImportWallet { .. }) => {
            "Create a strong password for your imported wallet."
        }
        Some(PasswordDialogConfig::ChangePassword { .. }) => {
            "Enter your new password."
        }
        Some(PasswordDialogConfig::MasterPassword { .. }) => {
            "Create a master password for your HD wallet."
        }
        None => return Container::new(Space::with_height(Length::Shrink)).into(),
    };

    // Build dialog with appropriate fields based on config
    // SECURITY: Password inputs are never logged
    Container::new(
        Column::new()
            .push(Text::new(title).size(20))
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(Text::new(description).size(14))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(password_input_view(state, "Password"))
            .push(if state.requires_confirmation() {
                confirm_password_input_view(state)
            } else {
                Space::with_height(Length::Shrink)
            })
            .push(options_view(state))
            .push(buttons_view(state)),
    )
}
```

### 6. State Consolidation (Enhanced)

**Purpose:** Merge security_state.rs and session_state.rs into unified auth_state.rs with proper zeroization.

```rust
// src/gui/state/auth_state.rs
//! Unified authentication and session state

use std::time::{Duration, Instant};

/// Authentication and session state
pub struct AuthState {
    // Session management
    pub is_unlocked: bool,
    pub unlocked_at: Option<Instant>,
    pub last_activity: Instant,
    pub timeout_duration: Duration,
    pub auto_lock_enabled: bool,

    // Password dialog
    pub password_dialog: PasswordDialogState,

    // Key cache (memory only, never persisted)
    pub cached_seed: Option<secrecy::SecretString>>,

    // Attempt tracking
    pub failed_attempts: u32,
    pub locked_until: Option<Instant>,
    pub max_attempts: u32,
    pub lockout_duration: Duration,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            is_unlocked: false,
            unlocked_at: None,
            last_activity: Instant::now(),
            timeout_duration: Duration::from_secs(300), // 5 minutes
            auto_lock_enabled: true,
            password_dialog: PasswordDialogState::default(),
            cached_seed: None,
            failed_attempts: 0,
            locked_until: None,
            max_attempts: 3,
            lockout_duration: Duration::from_secs(30), // 30 seconds
        }
    }
}

impl AuthState {
    pub fn unlock(&mut self) {
        self.is_unlocked = true;
        self.unlocked_at = Some(Instant::now());
        self.last_activity = Instant::now();
        self.failed_attempts = 0;
        self.locked_until = None;
    }

    pub fn lock(&mut self) {
        self.is_unlocked = false;
        self.unlocked_at = None;
        self.failed_attempts = 0;
        // SECURITY: Clear cached seed with proper zeroization
        if let Some(mut seed) = self.cached_seed.take() {
            seed.expose_secret().zeroize();
        }
    }

    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_timed_out(&self) -> bool {
        self.is_unlocked
            && self.auto_lock_enabled
            && self.last_activity.elapsed() > self.timeout_duration
    }

    pub fn is_locked_out(&self) -> bool {
        match self.locked_until {
            Some(until) => Instant::now() < until,
            None => false,
        }
    }

    pub fn record_failed_attempt(&mut self) {
        self.failed_attempts += 1;
        if self.failed_attempts >= self.max_attempts {
            self.locked_until = Some(Instant::now() + self.lockout_duration);
        }
    }

    pub fn clear_lockout(&mut self) {
        self.failed_attempts = 0;
        self.locked_until = None;
    }
}
```

### 7. Seed Module Zeroization (NEW)

**Purpose:** Ensure secure memory handling for seed phrases.

```rust
// src/security/seed/zeroization.rs
//! Memory zeroization utilities for secure seed handling

use zeroize::Zeroize;

/// Securely zero a byte slice
pub fn zero_bytes(data: &mut [u8]) {
    data.zeroize();
}

/// Securely zero a string
pub fn zero_string(s: &mut String) {
    s.zeroize();
}

/// Secure seed wrapper with automatic zeroization
pub struct SecureSeed(String);

impl SecureSeed {
    pub fn new(phrase: String) -> Self {
        Self(phrase)
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl Drop for SecureSeed {
    fn drop(&mut self) {
        // SECURITY: Automatic zeroization on drop
        self.0.zeroize();
    }
}
```

## Data Models (Enhanced)

### Metrics Tracking

```rust
/// Debloat progress metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebloatMetrics {
    pub phase: String,
    pub timestamp: DateTime<Utc>,
    pub file_count: usize,
    pub lines_of_code: usize,
    pub binary_size_mb: f64,
    pub binary_size_minimal_mb: Option<f64>,
    pub warning_count: usize,
    pub dead_code_annotations: usize,
    pub modules_over_1000_lines: usize,
    pub code_reduction_percent: Option<f64>,
}

impl DebloatMetrics {
    pub fn capture(phase: &str) -> Self {
        Self {
            phase: phase.to_string(),
            timestamp: Utc::now(),
            file_count: count_rust_files("src/"),
            lines_of_code: count_lines("src/"),
            binary_size_mb: measure_binary_size("full"),
            binary_size_minimal_mb: Some(measure_binary_size("minimal")),
            warning_count: count_warnings(),
            dead_code_annotations: count_dead_code_annotations(),
            modules_over_1000_lines: count_large_modules(),
            code_reduction_percent: calculate_reduction(),
        }
    }
}
```

## Correctness Properties (Enhanced)

*Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Dead Code Annotation Compliance (Enhanced)

*For any* `#[allow(dead_code)]` annotation in the codebase, there must exist either:
- A comment on the same or preceding line explaining why the annotation is necessary, OR
- The annotation must be for a serde deserialization field (identifiable by `#[derive(Deserialize)]` on the containing struct), OR
- The annotation must be documented with a roadmap milestone for planned features, OR
- The annotation must be documented with a security reason for critical modules

**Validates: Requirements 1.1, 1.2, 1.6**

### Property 2: Module Size Constraint

*For any* Rust source file in the `src/gui/wallet/` directory after decomposition, the file must contain fewer than 1,000 lines of code (excluding blank lines and comments).

**Validates: Requirements 4.9**

### Property 3: Seed Module Size Constraint

*For any* Rust source file in the `src/security/seed/` directory after decomposition, the file must contain fewer than 800 lines of code (excluding blank lines and comments).

**Validates: Requirements 5.9**

### Property 4: Password Dialog Code Reduction (Enhanced)

*For any* measurement of password dialog code, the total lines of code in the consolidated `password_dialog.rs` must be at least 33% less than the sum of the original three files (password_dialog.rs + master_password_dialog.rs + wallet_password_dialog.rs).

**Validates: Requirements 7.5**

### Property 5: No Test Files in Source Directory

*For any* file in the `src/` directory tree, the filename must not match the pattern `*_test.rs` or `*_tests.rs`, and must not contain `#[cfg(test)]` module declarations for test-only modules.

**Validates: Requirements 9.9**

### Property 6: Binary Size Reduction (Enhanced)

*For any* release build after dependency optimization, the binary size must be strictly less than the binary size before optimization. Additionally, the full-features binary must be <14MB and the minimal-features binary must be <10MB.

**Validates: Requirements 10.5, 14.1, 14.3**

### Property 7: Zero Compiler Warnings

*For any* compilation of the codebase with `cargo check --all-features`, the output must contain zero warning messages (excluding notes and help messages).

**Validates: Requirements 11.1, 11.2, 11.3**

### Property 8: Target Binary Size (Enhanced)

*For any* release build after all debloat phases are complete, the binary size with full features must be less than 14MB (14,680,064 bytes), and the binary size with minimal features must be less than 10MB (10,485,760 bytes).

**Validates: Requirements 14.1, 14.3**

### Property 9: Compilation Success After Decomposition

*For any* file decomposition operation (dialogs.rs, keystore.rs, working_wallet.rs, seed.rs, theme.rs), the codebase must compile successfully with `cargo build --all-features` after the operation.

**Validates: Requirements 2.7, 3.8, 4.10, 5.10, 6.9**

### Property 10: Test Suite Integrity

*For any* refactoring operation (decomposition, consolidation, relocation), all existing tests must pass with `cargo test --all-features` after the operation.

**Validates: Requirements 2.7, 3.8, 5.10, 7.6, 8.8, 9.10**

### Property 11: Clippy Compliance

*For any* state of the codebase after code quality improvements, running `cargo clippy --all-features -- -D warnings` must produce zero errors.

**Validates: Requirements 12.4**

### Property 12: Format Compliance

*For any* state of the codebase after code quality improvements, running `cargo fmt --check` must produce no formatting differences.

**Validates: Requirements 12.5**

### Property 13: Security Zeroization (NEW)

*For any* password dialog or seed operation that handles sensitive data, the memory must be zeroized when no longer needed. Specifically:

- Password input fields must be zeroized when dialog is hidden
- Seed phrases must be wrapped in SecureSeed with Drop trait
- Cached keys must be zeroized when wallet is locked
- No sensitive data should ever be logged

**Validates: Requirements 5.10, 7.7, 17.1-17.3**

### Property 14: Feature Gating Compliance (NEW)

*For any* optional component (hardware wallet, professional monitoring, custom tokens, QR codes, audio), the code must be properly gated with `#[cfg(feature)]` attributes. When the feature is disabled, the component must either provide a minimal placeholder or return an appropriate error.

**Validates: Requirements 13.7**

## Error Handling (Enhanced)

### Decomposition Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| Circular dependency | Module A imports B, B imports A | Identify shared types, extract to common module |
| Missing re-export | Public item not exported from mod.rs | Add `pub use` statement |
| Visibility error | Private item accessed from new module | Change to `pub(crate)` or `pub(super)` |
| Type mismatch | Generic bounds differ after split | Ensure consistent trait bounds |

### Consolidation Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| Name collision | Two merged files have same function name | Rename with descriptive prefix |
| Missing functionality | Feature from original file not preserved | Review original file, add missing code |
| State inconsistency | Merged state has conflicting fields | Resolve conflicts, document decisions |

### Feature Gating Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| Missing fallback | Code not gated, crashes when feature disabled | Add `#[cfg(feature)]` and fallback |
| Dead code warnings | Feature not enabled but code exists | Use `#![cfg_attr(not(feature), allow(dead_code))]` |

## Testing Strategy (Enhanced)

### Unit Tests

Unit tests verify specific examples and edge cases:

1. **Dead Code Audit Tests** (Enhanced)
   - Verify specific files have no undocumented dead_code annotations
   - Verify serde fields have appropriate annotations
   - Verify planned features have documented timelines

2. **Module Structure Tests** (Enhanced)
   - Verify expected files exist after decomposition
   - Verify mod.rs files have correct re-exports
   - Verify all new modules are within line limits

3. **Consolidation Tests** (Enhanced)
   - Verify unified password dialog handles all configuration types
   - Verify unified auth state handles all scenarios
   - Verify state transitions work correctly

4. **Security Tests** (NEW)
   - Verify seed data is zeroized after use
   - Verify password inputs are zeroized when hidden
   - Verify cached keys are zeroized on lock
   - Verify no sensitive data is logged

### Property-Based Tests

Property-based tests verify universal properties across all inputs:

1. **Module Size Property Test** (Enhanced)
   - Generate list of all .rs files in target directories
   - Verify each file is under line limit
   - Verify no module exceeds 1000 lines

2. **Binary Size Property Test** (Enhanced)
   - Build release binary with full features
   - Build release binary with minimal features
   - Verify full <14MB and minimal <10MB

3. **Warning Count Property Test**
   - Run cargo check
   - Parse output for warning count
   - Verify count is zero

4. **Security Zeroization Property Test** (NEW)
   - Test all password dialog scenarios
   - Verify all inputs are zeroized
   - Test all lock/unlock scenarios
   - Verify cached data is zeroized

### Integration Tests

Integration tests verify end-to-end functionality:

1. **Wallet Operations**
   - Create wallet, verify mnemonic
   - Import wallet, verify address
   - Export key, verify format

2. **Transaction Flow**
   - Build transaction
   - Sign transaction
   - Verify signature

3. **Network Operations**
   - Connect to network
   - Fetch balance
   - Switch networks

4. **Feature Combination Tests** (NEW)
   - Test with full features
   - Test with minimal features
   - Test with individual optional features
   - Verify all builds work correctly

### Test Configuration

```toml
# Cargo.toml [dev-dependencies]
proptest = "1.0"  # For property-based testing
```

Property tests should run with minimum 100 iterations to ensure coverage.

## Implementation Notes (Enhanced)

### Phase Execution Order (Enhanced)

1. **Phase 1: Foundation** (Low Risk, 4-6 hours)
   - Dead code cleanup with security documentation
   - Feature flag enhancement (professional, custom-tokens, shamir)
   - Dependency optimization (alloy features, iced features)
   - Estimated time: 4-6 hours

2. **Phase 2: Test File Relocation** (Low Risk, 1-2 hours)
   - Move 7 test files to tests/
   - Update imports from `crate::` to `vaughan::`
   - Verify tests pass
   - Estimated time: 1-2 hours

3. **Phase 3: Dependency Optimization** (Low Risk, 2-3 hours)
   - Optimize alloy features (remove "full")
   - Verify and remove iced "advanced" if unused
   - Test both full and minimal builds
   - Estimated time: 2-3 hours

4. **Phase 4: File Decomposition** (Medium Risk, 12-16 hours)
   - Decompose dialogs.rs
   - Decompose keystore.rs
   - Decompose working_wallet.rs (following handler pattern)
   - Decompose seed.rs (with zeroization module)
   - Decompose theme.rs (remove unused styles)
   - Verify compilation and tests
   - Estimated time: 12-16 hours

5. **Phase 5: Module Consolidation** (Medium Risk, 8-12 hours)
   - Consolidate password dialogs (with zeroization)
   - Consolidate state files (security + session → auth)
   - Remove duplicate ui_state.rs
   - Verify functionality
   - Estimated time: 8-12 hours

6. **Phase 6: Code Quality** (Low Risk, 4-6 hours)
   - Fix compiler warnings
   - Organize imports
   - Run clippy and fmt
   - Add documentation comments
   - Credit MetaMask where applicable
   - Estimated time: 4-6 hours

7. **Phase 7: Feature Gating** (Medium Risk, 6-8 hours)
   - Gate hardware.rs with hardware-wallets feature
   - Gate professional.rs with professional feature
   - Gate custom token dialogs with custom-tokens feature
   - Test all feature combinations
   - Estimated time: 6-8 hours

8. **Phase 8: Final Verification** (Low Risk, 2-3 hours)
   - Measure final metrics
   - Verify feature preservation (all wallet operations)
   - Verify security (zeroization, encryption)
   - Run comprehensive tests
   - Estimated time: 2-3 hours

### Rollback Strategy (Enhanced)

Each phase should be committed separately to allow rollback:

```bash
# Before each phase
git checkout -b debloat/phase-N-enhanced

# After successful phase
git add -A
git commit -m "Debloat Phase N (Enhanced): [description]"

# If phase fails
git checkout main
git branch -D debloat/phase-N-enhanced
git checkout -b debloat/phase-N-enhanced-fixed
# Fix issues
git commit -m "Fix: [description of fix]"
```

### Verification Checkpoints (Enhanced)

After each phase, run:

```bash
# Compilation check
cargo check --all-features

# Test suite
cargo test --all-features

# Clippy (after Phase 6)
cargo clippy --all-features -- -D warnings

# Binary size (after Phase 3+)
cargo build --release --all-features
du -h target/release/vaughan
cargo build --release --no-default-features --features minimal
du -h target/release/vaughan
```

### Security Considerations (Enhanced)

1. **dialogs.rs Decomposition**
   - Ensure no sensitive data is logged during refactoring
   - Verify password dialog consolidation maintains zeroization
   - Run security tests after each change

2. **keystore.rs Decomposition**
   - Ensure encryption logic is preserved
   - Verify Argon2 parameters are maintained
   - Ensure atomic file operations for persistence
   - Run security tests after each change

3. **seed.rs Decomposition** (CRITICAL)
   - Ensure no seed data is logged during refactoring
   - Verify zeroization module works correctly
   - Run security tests after each change
   - Test seed wrapper drop behavior

4. **Password Dialog Consolidation** (CRITICAL)
   - Ensure password input is still zeroized
   - Verify all dialog configurations work correctly
   - Test lockout functionality
   - Verify no passwords are logged

5. **State Consolidation** (CRITICAL)
   - Ensure cached keys are still cleared on lock
   - Verify session timeout still works
   - Test auto-lock functionality
   - Verify zeroization on lock

6. **Feature Gating**
   - Ensure disabled features return appropriate errors
   - Verify no code paths crash with missing features
   - Test all feature combinations

### MetaMask Inspiration Documentation

When using code patterns inspired by MetaMask (where Alloy doesn't suffice), document it:

```rust
//! Transaction gas estimation
//!
//! This implementation is inspired by MetaMask's gas estimation algorithm:
//! https://github.com/MetaMask/metamask-extension/blob/develop/ui/controllers/transactions/gas.js
//!
//! Key differences from MetaMask:
//! - Uses Alloy's `estimate_gas` instead of Web3.js
//! - Adds PulseChain-specific adjustments
//! - Implements caching to reduce RPC calls
```

## Alloy Library Best Practices (Enhanced)

### 1. Use Type-Safe Builders

```rust
// GOOD - Type-safe transaction builder
use alloy::rpc::types::TransactionRequest;

let tx = TransactionRequest::default()
    .with_to(Address::ZERO)
    .with_value(U256::from(100))
    .with_gas_limit(21_000)
    .with_chain_id(chain_id);

// BAD - Manual construction
let tx = TransactionRequest {
    to: Some(TxKind::Call(Address::ZERO)),
    value: Some(U256::from(100)),
    gas_limit: Some(21_000),
    ..Default::default()
};
```

### 2. Use Signer Traits

```rust
// GOOD - Use Signer trait
use alloy::signers::Signer;

let signature = signer.sign_transaction(&tx).await?;

// BAD - Manual signing
let signature = signer.sign_hash(tx_hash)?;
```

### 3. Use Provider Types

```rust
// GOOD - Use typed provider
use alloy::providers::Provider;

let balance = provider.get_balance(address).await?;

// BAD - Use raw RPC
let balance: U256 = provider.request("eth_getBalance", [address, "latest"]).await?;
```

### 4. Error Handling

```rust
// GOOD - Use Result types
use alloy::providers::RpcError;

match provider.get_balance(address).await {
    Ok(balance) => Ok(balance),
    Err(RpcError::Transport(err)) => Err(VaughanError::Network(err)),
    Err(RpcError::SerdeJson(err)) => Err(VaughanError::Deserialization(err)),
    Err(err) => Err(VaughanError::Unknown(err)),
}

// BAD - Unwrap everywhere
let balance = provider.get_balance(address).await.unwrap();
```

## Expected Outcomes (Enhanced)

| Metric | Before | After (Full) | After (Minimal) | Improvement |
|--------|--------|---------------|-----------------|-------------|
| Files | 117 | <75 | <60 | ~36% / ~49% |
| Lines | 49,741 | <38,000 | <32,000 | ~24% / ~36% |
| Binary | 21MB | <14MB | <10MB | ~33% / ~52% |
| Warnings | 9 | 0 | 0 | 100% |
| Dead code | 22 | 0 | 0 | 100% |
| Test files in src/ | 7 | 0 | 0 | 100% |
| Password dialogs | 3 → 1 | 1 | 1 | 66% reduction |
| Auth state files | 2 → 1 | 1 | 1 | 50% reduction |
| Modules >1000 lines | 6 | 0 | 0 | 100% |
