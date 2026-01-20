# Service Layer Architecture Design

## Summary
Analysis and design of service layer structure for Vaughan wallet to improve separation of concerns, reduce code duplication, and establish clear service boundaries.

## Current Service Landscape

### Existing Services by Size and Complexity

#### 1. Account Service (`src/gui/services/account_service.rs`) - 753 lines
**Responsibilities**:
- Seed-based account detection
- Account type classification (Seed vs Private Key)
- Account creation (seed, private key, hardware)
- Account validation and metadata management
- Export functionality

**Dependencies**:
- `security::keychain::OSKeychain` (repeated 6+ times)
- `security::keystore::SecureKeystoreImpl`
- `security::{SecureAccount, SeedStrength}`

#### 2. Network Service (`src/gui/services/network_service.rs`) - 463 lines
**Responsibilities**:
- Custom network configuration
- Network validation
- Network addition/removal
- Default network management
- Network switching

**Dependencies**:
- `network::{NetworkConfig, NetworkId}`
- `network::validation` module
- `security::create_keychain_interface`

#### 3. Wallet Service (`src/gui/services/wallet_service.rs`) - 140 lines
**Responsibilities**:
- Wallet initialization
- Account loading from persistent storage
- Configuration management

**Dependencies**:
- `security::{SecureAccount, KeychainInterface}`
- `wallet::{Vaughan, WalletConfig}`

#### 4. Explorer Service (`src/gui/services/explorer_service.rs`) - 371 lines
**Responsibilities**:
- Block explorer API interactions
- Transaction history fetching
- Multi-network explorer support (Etherscan, PulseScan, BSC)
- Transaction status tracking

**Dependencies**:
- `reqwest` for HTTP calls
- `serde_json` for API responses
- `chrono` for timestamps
- Multiple network-specific APIs

#### 5. QR Service (`src/gui/services/qr_service.rs`) - 101 lines
**Responsibilities**:
- QR code generation for addresses
- QR code image creation for UI
- Error handling for QR operations

**Dependencies**:
- `qrcode` crate
- `iced::widget::image::Handle`
- `image` crate for processing

#### 6. Auto Balance Service (`src/gui/services/auto_balance_service.rs`) - 23 lines
**Responsibilities**:
- Message definitions for balance monitoring
- Event notifications for balance changes
- Transaction detection notifications

**Dependencies**:
- Pure data structures, no external dependencies

## Critical Issues Identified

### 1. Service Overlap & Duplication

#### Keychain Management Overlap
```rust
// account_service.rs - Multiple keychain instantiations
use crate::security::keychain::OSKeychain;
let keychain = Box::new(OSKeychain::new(...)); // Repeated 6+ times

// wallet_service.rs - Separate keychain handling
use crate::security::create_keychain_interface;
```

**Problem**: Each service creates its own keychain instance
**Impact**: Inefficient resource usage, potential configuration inconsistencies

#### Account Loading Duplication
```rust
// wallet_service.rs - Account loading
pub async fn load_available_accounts()

// account_service.rs - Account listing
let accounts = keystore.list_accounts().await
```

**Problem**: Two different approaches to account access
**Impact**: Inconsistent account state, potential sync issues

### 2. Missing Service Boundaries

#### Security Module Access Patterns
```rust
// Direct security module access across services
use crate::security::keychain::OSKeychain;
use crate::security::keystore::SecureKeystoreImpl;
use crate::security::{SecureAccount, SeedStrength};
```

**Problem**: No centralized security interface
**Impact**: Tight coupling to security implementation

#### Transaction Management Gaps
- No dedicated transaction service
- Transaction logic scattered across GUI, wallet, and services
- Transaction validation in multiple places

### 3. Dependency Injection Issues

#### Hard-coded Dependencies
```rust
// Hard-coded OSKeychain creation
let keychain = Box::new(OSKeychain::new("vaughan-wallet".to_string()));

// Hard-coded keystore creation
let keystore = SecureKeystoreImpl::new(keychain)
```

**Problem**: No dependency injection, difficult to test
**Impact**: Monolithic architecture, poor testability

## Proposed Service Layer Architecture

### 1. Core Services (Domain-focused)

#### Account Management Service
```rust
pub struct AccountService {
    security: Arc<dyn SecurityInterface>,
    storage: Arc<dyn StorageInterface>,
}

impl AccountService {
    // Unified account operations
    pub async fn create_account(&self, params: AccountParams) -> Result<SecureAccount>;
    pub async fn list_accounts(&self) -> Result<Vec<SecureAccount>>;
    pub async fn get_account(&self, id: &str) -> Result<Option<SecureAccount>>;
    pub async fn delete_account(&self, id: &str) -> Result<()>;
    pub async fn update_account(&self, id: &str, updates: AccountUpdates) -> Result<SecureAccount>;
}
```

#### Network Management Service
```rust
pub struct NetworkService {
    storage: Arc<dyn StorageInterface>,
    validator: Arc<dyn NetworkValidator>,
}

impl NetworkService {
    // Network operations
    pub async fn add_network(&self, config: NetworkConfig) -> Result<NetworkId>;
    pub async fn remove_network(&self, id: NetworkId) -> Result<()>;
    pub async fn get_networks(&self) -> Result<Vec<NetworkConfig>>;
    pub async fn set_active_network(&self, id: NetworkId) -> Result<()>;
    pub async fn get_active_network(&self) -> Result<Option<NetworkConfig>>;
}
```

#### Transaction Service (New)
```rust
pub struct TransactionService {
    network: Arc<dyn NetworkInterface>,
    security: Arc<dyn SecurityInterface>,
    explorer: Arc<dyn ExplorerInterface>,
}

impl TransactionService {
    // Transaction lifecycle
    pub async fn create_transaction(&self, params: TxParams) -> Result<Transaction>;
    pub async fn sign_transaction(&self, tx: &Transaction, account: &SecureAccount) -> Result<SignedTransaction>;
    pub async fn submit_transaction(&self, tx: SignedTransaction) -> Result<TxHash>;
    pub async fn get_transaction_history(&self, address: &str) -> Result<Vec<Transaction>>;
    pub async fn get_transaction_status(&self, hash: &str) -> Result<TransactionStatus>;
}
```

### 2. Infrastructure Services (Cross-cutting)

#### Security Interface Service
```rust
pub struct SecurityService {
    keychain: Box<dyn KeychainInterface>,
    keystore: Box<dyn KeystoreInterface>,
}

impl SecurityService {
    // Centralized security operations
    pub async fn store_secret(&self, key: &str, value: &[u8]) -> Result<()>;
    pub async fn retrieve_secret(&self, key: &str) -> Result<Vec<u8>>;
    pub async fn delete_secret(&self, key: &str) -> Result<()>;
    pub async fn create_secure_account(&self, params: AccountParams) -> Result<SecureAccount>;
}
```

#### Storage Service (New)
```rust
pub struct StorageService {
    file_storage: Arc<dyn FileStorageInterface>,
    preferences: Arc<dyn PreferencesInterface>,
}

impl StorageService {
    // Unified storage operations
    pub async fn save_data<T: Serialize>(&self, key: &str, data: &T) -> Result<()>;
    pub async fn load_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    pub async fn delete_data(&self, key: &str) -> Result<()>;
    pub async fn sync_preferences(&self) -> Result<()>;
}
```

#### Notification Service (Enhanced)
```rust
pub struct NotificationService {
    event_bus: Arc<dyn EventBus>,
    audio: Arc<dyn AudioInterface>,
}

impl NotificationService {
    // Unified notifications
    pub async fn notify_balance_change(&self, address: &str, old_balance: &str, new_balance: &str);
    pub async fn notify_transaction(&self, tx: &Transaction);
    pub async fn notify_error(&self, error: &str, context: Option<&str>);
}
```

### 3. Utility Services (Specialized)

#### QR Service (Enhanced)
```rust
pub struct QRService {
    renderer: Arc<dyn QRRenderer>,
}

impl QRService {
    // Enhanced QR operations
    pub fn generate_address_qr(&self, address: &str) -> Result<ImageHandle>;
    pub fn generate_payment_qr(&self, address: &str, amount: Option<f64>, token: Option<&str>) -> Result<ImageHandle>;
    pub fn generate_seed_words_qr(&self, words: &[String]) -> Result<ImageHandle>;
}
```

#### Explorer Service (Refactored)
```rust
pub struct ExplorerService {
    clients: HashMap<NetworkId, Arc<dyn ExplorerClient>>,
}

impl ExplorerService {
    // Multi-network explorer support
    pub async fn get_transaction_history(&self, address: &str, network: NetworkId) -> Result<Vec<Transaction>>;
    pub async fn get_transaction_details(&self, hash: &str, network: NetworkId) -> Result<TransactionDetails>;
    pub async fn get_balance(&self, address: &str, network: NetworkId) -> Result<String>;
}
```

## Service Responsibility Matrix

| Domain | Current Services | Proposed Services | Key Responsibilities |
|---------|----------------|------------------|---------------------|
| **Account Management** | account_service, wallet_service | AccountService | CRUD operations, type classification, validation |
| **Network Management** | network_service | NetworkService | Network CRUD, switching, validation |
| **Transaction Management** | scattered across GUI/wallet | TransactionService | TX lifecycle, signing, submission, history |
| **Security** | duplicated across services | SecurityService | Centralized keychain, keystore, encryption |
| **Storage** | wallet_service, scattered | StorageService | Unified file I/O, preferences, persistence |
| **Notifications** | auto_balance_service | NotificationService | Balance changes, TX events, errors |
| **External APIs** | explorer_service | ExplorerService | Block explorer integration, multi-network |
| **QR Operations** | qr_service | QRService | Address/payment QR, image generation |

## Dependency Injection Plan

### 1. Service Container
```rust
pub struct ServiceContainer {
    // Core services
    account_service: Arc<AccountService>,
    network_service: Arc<NetworkService>,
    transaction_service: Arc<TransactionService>,
    
    // Infrastructure services
    security_service: Arc<SecurityService>,
    storage_service: Arc<StorageService>,
    notification_service: Arc<NotificationService>,
    
    // Utility services
    qr_service: Arc<QRService>,
    explorer_service: Arc<ExplorerService>,
}

impl ServiceContainer {
    pub fn new() -> Result<Self> {
        // Build services with dependency injection
        let security_service = Arc::new(SecurityService::new()?);
        let storage_service = Arc::new(StorageService::new()?);
        
        let account_service = Arc::new(AccountService::new(
            security_service.clone(),
            storage_service.clone(),
        ));
        
        // ... initialize all services
        
        Ok(Self {
            account_service,
            network_service,
            transaction_service,
            security_service,
            storage_service,
            notification_service,
            qr_service,
            explorer_service,
        })
    }
}
```

### 2. Interface Definitions

#### Security Interface
```rust
#[async_trait]
pub trait SecurityInterface: Send + Sync {
    async fn store_secret(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn retrieve_secret(&self, key: &str) -> Result<Vec<u8>>;
    async fn create_account(&self, params: AccountParams) -> Result<SecureAccount>;
    async fn sign_transaction(&self, tx: &Transaction, account_id: &str) -> Result<SignedTransaction>;
}
```

#### Storage Interface
```rust
#[async_trait]
pub trait StorageInterface: Send + Sync {
    async fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<()>;
    async fn load<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
}
```

## Migration Strategy

### Phase 1: Interface Definition (1-2 days)
1. Define core service interfaces
2. Create trait definitions
3. Implement basic dependency injection
4. Create ServiceContainer skeleton

### Phase 2: Service Refactoring (3-4 days)
1. Refactor existing services to use interfaces
2. Implement new services (TransactionService, StorageService)
3. Create security abstraction layer
4. Migrate service dependencies

### Phase 3: Integration (2 days)
1. Update GUI to use ServiceContainer
2. Replace direct service calls
3. Update message handlers
4. Test integration

### Phase 4: Cleanup (1 day)
1. Remove duplicate code
2. Update imports
3. Add comprehensive tests
4. Documentation updates

## Expected Benefits

### Code Quality
- **Single Responsibility**: Each service has clear domain focus
- **Dependency Injection**: Easy testing and configuration
- **Interface Segregation**: Clean service boundaries
- **Reduced Duplication**: Centralized keychain and storage

### Maintainability
- **Clear Architecture**: Service responsibilities well-defined
- **Better Testing**: Interface-based testing possible
- **Easier Debugging**: Isolated service domains
- **Scalable Design**: New services easy to add

### Performance
- **Resource Efficiency**: Shared instances instead of multiple creations
- **Caching Opportunities**: Centralized services can implement caching
- **Lazy Loading**: Services initialized only when needed

## Risk Assessment

### Low Risk
- Interface definitions
- Service container creation
- Basic dependency injection

### Medium Risk
- Service refactoring (may affect existing functionality)
- Integration with existing GUI code
- Transaction service implementation (complex domain)

### High Risk
- Storage abstraction (affects data persistence)
- Security interface changes (affects encryption/keystore)
- Migration of existing account/network logic

## Success Metrics

### Quantitative
- **Services reduced**: 6 → 8 (better organized)
- **Duplicate code eliminated**: ~200 lines of duplicated keychain/storage logic
- **Interface coverage**: 90%+ of service operations through interfaces
- **Test coverage**: Service layer 80%+ unit test coverage

### Qualitative
- **Service boundaries**: Clear domain separation
- **Dependency management**: Interface-based rather than concrete
- **Maintainability**: New features easy to add to appropriate service
- **Code organization**: Related functionality properly grouped

This service layer architecture will significantly improve the Vaughan wallet's maintainability, testability, and scalability while reducing code duplication.