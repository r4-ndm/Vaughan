# Business Logic Extraction Plan

## Executive Summary
This plan addresses architectural debt by extracting business logic from view components into a dedicated service layer. The current codebase has 841+ lines of view code containing file I/O, validation, and state management that violates separation of concerns principles. This refactoring will improve testability, maintainability, and enable independent evolution of UI and business logic.

## Goals
- Achieve zero file operations in view layer
- Extract all validation logic to services
- Enable comprehensive unit testing of business logic
- Reduce view complexity by 40-60%
- Maintain 100% functional equivalence during migration

## Files Analyzed
- `src/gui/views/main_wallet.rs` (841 lines)
- `src/gui/views/dialogs.rs` (1,383 lines)
- `src/gui/components/dialogs/*.rs`

## Business Logic Found in Views

### 1. File System Operations (Should Move to Asset Service)

#### Location: `src/gui/views/main_wallet.rs`
**Pattern**: Direct file system checks in view logic

**Issues Found**:
```rust
// Line 29: Logo file check
if std::path::Path::new("assets/vaughan-logo-513x76-thor.png").exists() {

// Line 100: Hamburger menu icon check
if std::path::Path::new("assets/hamburger-128.png").exists() {

// Line 209: Account delete icon check
if std::path::Path::new("assets/hamburger-128.png").exists() {

// Line 267: Network selector icon check
if std::path::Path::new("assets/hamburger-128.png").exists() {

// Line 343: Copy address icon check
if std::path::Path::new("assets/clipboard-128.png").exists() {

// Line 386: Copy transaction hash icon check
if std::path::Path::new("assets/clipboard-128.png").exists() {
```

**Problem**: Views should not perform file I/O operations

**Solution**: Create `AssetService` for asset availability

### 2. Data Processing in Views (Should Move to Services)

#### Location: `src/gui/views/main_wallet.rs`
**Pattern**: Complex data transformations in view methods

**Issues Found**:

1. **Account Selection Logic** (Lines 58-84):
```rust
if let Some(current_account_id) = self.current_account_id() {
    // Complex account matching and display logic
    let address_element: Element<Message> = if let Some(account) = self.accounts().iter().find(|acc| acc.id == current_account_id) {
        // Multiple nested conditions for address display
    }
}
```

2. **Network Selection Logic** (Lines 86-91):
```rust
PickList::new(
    &self.available_networks()[..],
    self.available_networks()
        .iter()
        .find(|n| &n.id == self.current_network())
        .cloned(),
    |config| Message::NetworkSelected(config.id),
)
```

3. **Balance and Token Logic** (Lines 183-184):
```rust
let selected_symbol = if self.balance_selected_ticker().contains('(') {
    self.balance_selected_ticker()
}
```

### 3. State Mutation Patterns (Should Use Service Layer)

#### Location: `src/gui/views/main_wallet.rs`
**Pattern**: Direct state access and complex business rules

**Issues Found**:

1. **Transaction State Access** (Lines 139-140):
```rust
if self.transaction().pending_transactions.iter().any(|tx| tx.cancellable) {
    content = content.push(self.pending_transactions_view());
}
```

2. **Complex Form View Methods** (Lines 154-177):
   - `send_form_view()` contains multiple business logic calls
   - Each sub-call performs complex state operations
   - Should be delegated to a `TransactionFormService`

### 4. Dialog Business Logic (Should Move to Services)

#### Location: `src/gui/views/dialogs.rs`
**Pattern**: Complex validation and data processing in dialogs

**Issues Found**:

1. **Custom Token Validation** (Lines 30-45):
```rust
let validation_message = if let Some(error) = &self.custom_token_validation_error {
    Container::new(Text::new(error).size(12).style(iced::Color::from_rgb(0.9, 0.3, 0.3)))
        // Complex styling and error display logic
}
```

2. **Network Configuration** (Lines 1149-1216):
   - Complex network form logic
   - Data validation in view layer
   - Should use `NetworkConfigurationService`

## Service Layer Architecture

### Design Principles
1. **Single Responsibility**: Each service handles one domain concern
2. **Dependency Injection**: Services injected via AppState, not global singletons
3. **Testability**: All services have trait interfaces for mocking
4. **Error Handling**: Consistent Result types with domain-specific errors
5. **Immutability**: Services are stateless where possible

### Service Hierarchy
```
src/gui/services/
├── mod.rs                    # Service registry and DI container
├── asset_service.rs          # Asset loading and availability
├── account_display_service.rs # Account formatting and display logic
├── network_config_service.rs  # Network validation and configuration
├── transaction_form_service.rs # Transaction form preparation and validation
└── validation/               # Shared validation utilities
    ├── mod.rs
    ├── address_validator.rs
    └── amount_validator.rs
```

### 1. Asset Service
**Responsibility**: Centralize all asset loading and availability checks

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait AssetServiceTrait: Send + Sync {
    fn is_asset_available(&self, asset_path: &str) -> bool;
    fn get_logo_path(&self) -> Option<PathBuf>;
    fn get_icon_path(&self, icon_name: &str) -> Option<PathBuf>;
}

pub struct AssetService {
    asset_root: PathBuf,
}

impl AssetService {
    pub fn new(asset_root: PathBuf) -> Self {
        Self { asset_root }
    }
    
    pub fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}

impl AssetServiceTrait for AssetService {
    fn is_asset_available(&self, asset_path: &str) -> bool {
        self.asset_root.join(asset_path).exists()
    }
    
    fn get_logo_path(&self) -> Option<PathBuf> {
        let path = self.asset_root.join("vaughan-logo-513x76-thor.png");
        if path.exists() { Some(path) } else { None }
    }
    
    fn get_icon_path(&self, icon_name: &str) -> Option<PathBuf> {
        let path = self.asset_root.join(format!("{}-128.png", icon_name));
        if path.exists() { Some(path) } else { None }
    }
}

// Mock for testing
#[cfg(test)]
pub struct MockAssetService {
    available_assets: std::collections::HashSet<String>,
}
```

### 2. Account Display Service
**Responsibility**: Format account information for display

```rust
use crate::wallet::account::Account;

#[derive(Debug, Clone)]
pub struct AccountDisplayInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub short_address: String,
    pub account_type: String,
}

pub trait AccountDisplayServiceTrait: Send + Sync {
    fn format_account_display(&self, account: &Account) -> AccountDisplayInfo;
    fn format_address_short(&self, address: &str) -> String;
    fn get_account_type_label(&self, account: &Account) -> String;
}

pub struct AccountDisplayService;

impl AccountDisplayService {
    pub fn new() -> Self {
        Self
    }
}

impl AccountDisplayServiceTrait for AccountDisplayService {
    fn format_account_display(&self, account: &Account) -> AccountDisplayInfo {
        AccountDisplayInfo {
            id: account.id.clone(),
            name: account.name.clone(),
            address: account.address.clone(),
            short_address: self.format_address_short(&account.address),
            account_type: self.get_account_type_label(account),
        }
    }
    
    fn format_address_short(&self, address: &str) -> String {
        if address.len() > 10 {
            format!("{}...{}", &address[..6], &address[address.len()-4..])
        } else {
            address.to_string()
        }
    }
    
    fn get_account_type_label(&self, account: &Account) -> String {
        match account.account_type {
            AccountType::Keystore => "Keystore".to_string(),
            AccountType::Seed => "HD Wallet".to_string(),
            AccountType::Hardware => "Hardware".to_string(),
        }
    }
}
```

### 3. Network Configuration Service
**Responsibility**: Validate and prepare network configurations

```rust
use crate::network::config::NetworkConfig;

#[derive(Debug, Clone)]
pub enum NetworkValidationError {
    InvalidRpcUrl(String),
    InvalidChainId,
    InvalidExplorerUrl(String),
    DuplicateNetwork(String),
}

pub trait NetworkConfigServiceTrait: Send + Sync {
    fn validate_network_config(&self, config: &NetworkConfig) -> Result<(), NetworkValidationError>;
    fn is_network_name_unique(&self, name: &str, existing: &[NetworkConfig]) -> bool;
    fn sanitize_rpc_url(&self, url: &str) -> Result<String, NetworkValidationError>;
}

pub struct NetworkConfigService;

impl NetworkConfigService {
    pub fn new() -> Self {
        Self
    }
}

impl NetworkConfigServiceTrait for NetworkConfigService {
    fn validate_network_config(&self, config: &NetworkConfig) -> Result<(), NetworkValidationError> {
        // Validate RPC URL
        if !config.rpc_url.starts_with("http://") && !config.rpc_url.starts_with("https://") {
            return Err(NetworkValidationError::InvalidRpcUrl(
                "RPC URL must start with http:// or https://".to_string()
            ));
        }
        
        // Validate chain ID
        if config.chain_id == 0 {
            return Err(NetworkValidationError::InvalidChainId);
        }
        
        // Validate explorer URL if present
        if let Some(explorer) = &config.explorer_url {
            if !explorer.starts_with("http://") && !explorer.starts_with("https://") {
                return Err(NetworkValidationError::InvalidExplorerUrl(
                    "Explorer URL must start with http:// or https://".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn is_network_name_unique(&self, name: &str, existing: &[NetworkConfig]) -> bool {
        !existing.iter().any(|n| n.name.eq_ignore_ascii_case(name))
    }
    
    fn sanitize_rpc_url(&self, url: &str) -> Result<String, NetworkValidationError> {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err(NetworkValidationError::InvalidRpcUrl("URL cannot be empty".to_string()));
        }
        Ok(trimmed.to_string())
    }
}
```

### 4. Transaction Form Service
**Responsibility**: Prepare and validate transaction forms

```rust
use crate::gui::state::AppState;
use ethers::types::{Address, U256};

#[derive(Debug, Clone)]
pub struct SendFormData {
    pub recipient: String,
    pub amount: String,
    pub token_symbol: String,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<U256>,
}

#[derive(Debug, Clone)]
pub enum TransactionValidationError {
    InvalidRecipient(String),
    InvalidAmount(String),
    InsufficientBalance,
    InvalidGasLimit,
}

pub trait TransactionFormServiceTrait: Send + Sync {
    fn prepare_send_form(&self, state: &AppState) -> SendFormData;
    fn validate_recipient(&self, address: &str) -> Result<Address, TransactionValidationError>;
    fn validate_amount(&self, amount: &str, balance: U256) -> Result<U256, TransactionValidationError>;
}

pub struct TransactionFormService;

impl TransactionFormService {
    pub fn new() -> Self {
        Self
    }
}

impl TransactionFormServiceTrait for TransactionFormService {
    fn prepare_send_form(&self, state: &AppState) -> SendFormData {
        SendFormData {
            recipient: state.send_recipient.clone(),
            amount: state.send_amount.clone(),
            token_symbol: state.balance_selected_ticker.clone(),
            gas_limit: state.send_gas_limit,
            gas_price: state.send_gas_price,
        }
    }
    
    fn validate_recipient(&self, address: &str) -> Result<Address, TransactionValidationError> {
        address.parse::<Address>()
            .map_err(|e| TransactionValidationError::InvalidRecipient(e.to_string()))
    }
    
    fn validate_amount(&self, amount: &str, balance: U256) -> Result<U256, TransactionValidationError> {
        let parsed = amount.parse::<f64>()
            .map_err(|e| TransactionValidationError::InvalidAmount(e.to_string()))?;
        
        if parsed <= 0.0 {
            return Err(TransactionValidationError::InvalidAmount("Amount must be positive".to_string()));
        }
        
        // Convert to wei and check balance
        let amount_wei = U256::from((parsed * 1e18) as u128);
        if amount_wei > balance {
            return Err(TransactionValidationError::InsufficientBalance);
        }
        
        Ok(amount_wei)
    }
}
```

## Testing Strategy

### Unit Tests
Each service must have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_service_logo_availability() {
        let service = AssetService::new(PathBuf::from("test_assets"));
        assert!(!service.is_asset_available("nonexistent.png"));
    }
    
    #[test]
    fn test_account_display_short_address() {
        let service = AccountDisplayService::new();
        let short = service.format_address_short("0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(short, "0x1234...5678");
    }
    
    #[test]
    fn test_network_validation_invalid_rpc() {
        let service = NetworkConfigService::new();
        let config = NetworkConfig {
            rpc_url: "invalid-url".to_string(),
            chain_id: 1,
            ..Default::default()
        };
        assert!(service.validate_network_config(&config).is_err());
    }
}
```

### Property-Based Tests
Use proptest for validation logic:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_address_formatting_always_valid(address in "[0-9a-fA-F]{40}") {
        let service = AccountDisplayService::new();
        let full_address = format!("0x{}", address);
        let short = service.format_address_short(&full_address);
        assert!(short.len() <= full_address.len());
        assert!(short.contains("..."));
    }
    
    #[test]
    fn test_amount_validation_never_negative(amount in 0.0f64..1000000.0f64) {
        let service = TransactionFormService::new();
        let balance = U256::from(u128::MAX);
        let result = service.validate_amount(&amount.to_string(), balance);
        assert!(result.is_ok());
    }
}
```

### Integration Tests
Test service interactions with AppState:

```rust
#[test]
fn test_transaction_form_end_to_end() {
    let mut state = AppState::default();
    let service = TransactionFormService::new();
    
    state.send_recipient = "0x1234567890abcdef1234567890abcdef12345678".to_string();
    state.send_amount = "1.5".to_string();
    
    let form_data = service.prepare_send_form(&state);
    assert_eq!(form_data.recipient, state.send_recipient);
    assert_eq!(form_data.amount, "1.5");
}
```

## Migration Strategy

### Phase 1: Foundation (Week 1)
**Goal**: Establish service infrastructure without breaking existing code

1. Create service directory structure
2. Define service traits and interfaces
3. Implement basic service implementations
4. Add services to AppState via dependency injection
5. Write comprehensive unit tests for each service

**Deliverables**:
- `src/gui/services/mod.rs` with service registry
- All four service implementations with tests
- Zero breaking changes to existing views

### Phase 2: Asset Service Migration (Week 1-2)
**Goal**: Eliminate all file I/O from views

1. Replace all `Path::new().exists()` calls with `asset_service.is_asset_available()`
2. Update logo loading in `main_wallet.rs`
3. Update icon loading for hamburger menu, clipboard, etc.
4. Add integration tests for asset loading
5. Remove all `std::path` imports from view files

**Success Criteria**:
- Zero file operations in view layer
- All asset checks go through AssetService
- Views remain functionally identical

### Phase 3: Account Display Migration (Week 2)
**Goal**: Extract account formatting logic

1. Move account display logic to AccountDisplayService
2. Update account selector in `main_wallet.rs`
3. Update account list rendering
4. Simplify address formatting code
5. Add property-based tests for address formatting

**Success Criteria**:
- All account formatting in service layer
- Views only call service methods
- Consistent address display across all views

### Phase 4: Network Configuration Migration (Week 3)
**Goal**: Extract network validation and configuration

1. Move network validation to NetworkConfigService
2. Update network configuration dialog
3. Update network selector logic
4. Add validation error handling
5. Test with invalid network configurations

**Success Criteria**:
- All network validation in service layer
- Consistent error messages
- Views handle only UI concerns

### Phase 5: Transaction Form Migration (Week 3-4)
**Goal**: Extract transaction form logic (highest risk)

1. Move form preparation to TransactionFormService
2. Extract validation logic
3. Update send form view
4. Add comprehensive validation tests
5. Test with edge cases (zero amounts, invalid addresses, etc.)

**Success Criteria**:
- All transaction validation in service layer
- Views only handle form rendering
- Zero regression in transaction functionality

### Phase 6: Cleanup and Optimization (Week 4)
**Goal**: Polish and optimize

1. Remove unused imports from views
2. Simplify view methods
3. Add performance benchmarks
4. Update documentation
5. Code review and refactoring

**Success Criteria**:
- View files reduced by 40-60% in complexity
- All services have >90% test coverage
- Documentation updated

## Expected Benefits

### Code Quality
- **Separation of Concerns**: Views handle UI, services handle business logic
- **Testability**: Business logic can be unit tested independently
- **Reusability**: Services can be used across different views

### Maintainability
- **Single Responsibility**: Each service has clear purpose
- **Easier Debugging**: Business logic isolated from UI concerns
- **Better Documentation**: Services have clear interfaces

### Performance
- **Lazy Loading**: Assets loaded only when needed
- **Caching**: Services can cache expensive operations
- **Reduced Re-renders**: Business logic changes don't trigger UI rebuilds

## Risk Mitigation

### Critical Path Protection
**Transaction Form Service** is the highest risk component:
- Implement feature flag for gradual rollout
- Maintain parallel implementation during migration
- Add extensive logging for debugging
- Create rollback plan

### Rollback Strategy
```rust
// Feature flag in AppState
pub struct AppState {
    use_service_layer: bool, // Default: false initially
    // ... other fields
}

// Conditional logic during migration
fn render_send_form(&self) -> Element<Message> {
    if self.use_service_layer {
        self.render_send_form_with_service()
    } else {
        self.render_send_form_legacy()
    }
}
```

### Monitoring and Validation
- Add metrics for service call performance
- Log all validation errors
- Compare service outputs with legacy outputs
- Monitor for regressions in CI/CD

### Backward Compatibility
- Keep legacy code until migration is 100% complete
- Run parallel tests comparing old vs new behavior
- Gradual feature flag rollout (10% → 50% → 100%)

## Performance Considerations

### Caching Strategy
Services should cache expensive operations:

```rust
pub struct AssetService {
    asset_root: PathBuf,
    cache: Arc<RwLock<HashMap<String, bool>>>,
}

impl AssetService {
    fn is_asset_available(&self, asset_path: &str) -> bool {
        // Check cache first
        if let Some(&exists) = self.cache.read().unwrap().get(asset_path) {
            return exists;
        }
        
        // Check filesystem and cache result
        let exists = self.asset_root.join(asset_path).exists();
        self.cache.write().unwrap().insert(asset_path.to_string(), exists);
        exists
    }
}
```

### Lazy Initialization
Services should be created only when needed:

```rust
pub struct ServiceRegistry {
    asset_service: OnceCell<Arc<AssetService>>,
    account_service: OnceCell<Arc<AccountDisplayService>>,
    // ... other services
}

impl ServiceRegistry {
    pub fn asset_service(&self) -> Arc<AssetService> {
        self.asset_service
            .get_or_init(|| Arc::new(AssetService::default()))
            .clone()
    }
}
```

### Memory Optimization
- Services should be stateless where possible
- Use `Arc` for shared services to avoid cloning
- Implement `Drop` for cleanup of cached data

## Implementation Priority

### Priority 1 (Safe to Extract)
1. Asset service operations
2. Icon availability checks
3. Simple formatting functions

### Priority 2 (Medium Impact)
1. Network configuration validation
2. Account selection logic
3. Custom token validation

### Priority 3 (Critical Path)
1. Transaction form service
2. Balance calculation services
3. Gas estimation logic

## Success Metrics

### Quantitative Metrics
- **Zero file operations in views**: All `std::path` imports removed from view files
- **Zero network calls in views**: All network operations delegated to services
- **Test coverage >90%**: All services have comprehensive unit tests
- **View complexity reduction**: 40-60% reduction in lines of code per view file
- **Performance**: No degradation in UI responsiveness (measured via benchmarks)

### Qualitative Metrics
- **Code maintainability**: New developers can understand service layer in <1 hour
- **Testability**: Business logic can be tested without UI framework
- **Separation of concerns**: Clear boundaries between UI and business logic
- **Documentation**: All services have clear API documentation

### Before/After Comparison

#### Before (Current State)
```rust
// main_wallet.rs - 841 lines with mixed concerns
fn view(&self) -> Element<Message> {
    // File I/O in view
    if std::path::Path::new("assets/logo.png").exists() {
        // Complex business logic
        if let Some(account) = self.accounts().iter().find(|a| a.id == self.current_account_id()) {
            // Validation in view
            if account.address.len() > 10 {
                // More nested logic...
            }
        }
    }
}
```

#### After (Target State)
```rust
// main_wallet.rs - ~500 lines, pure UI
fn view(&self) -> Element<Message> {
    let services = &self.services;
    
    // Clean service calls
    if services.asset.is_logo_available() {
        let account_info = services.account.format_account_display(&account);
        // Simple rendering logic only
    }
}
```

### Acceptance Criteria
- [ ] All file operations moved to AssetService
- [ ] All validation logic moved to appropriate services
- [ ] All services have trait interfaces for testing
- [ ] All services have >90% test coverage
- [ ] Zero functional regressions in UI
- [ ] View files reduced by 40-60% in complexity
- [ ] Documentation updated for all services
- [ ] Code review approved by 2+ team members

## Next Steps

### Immediate Actions (This Week)
1. Review and approve this plan with team
2. Create feature branch: `feature/service-layer-extraction`
3. Set up project tracking (GitHub issues/Jira)
4. Begin Phase 1: Foundation work

### Long-term Roadmap
- **Week 1**: Foundation + Asset Service
- **Week 2**: Account Display Service
- **Week 3**: Network Configuration Service
- **Week 4**: Transaction Form Service + Cleanup
- **Week 5**: Testing, documentation, code review
- **Week 6**: Gradual rollout with feature flags

### Team Coordination
- Daily standups to track progress
- Weekly architecture reviews
- Pair programming for high-risk components (Transaction Form)
- Code review for all service implementations

## References

### Related Documentation
- [Service Layer Design Pattern](https://martinfowler.com/eaaCatalog/serviceLayer.html)
- [Dependency Injection in Rust](https://docs.rs/shaku/latest/shaku/)
- [Testing Strategies for Rust](https://doc.rust-lang.org/book/ch11-00-testing.html)

### Related Issues
- Issue #XXX: View layer complexity
- Issue #XXX: Testability improvements
- Issue #XXX: Separation of concerns

### Code Review Checklist
- [ ] Service has clear single responsibility
- [ ] Service has trait interface for mocking
- [ ] Service has comprehensive unit tests
- [ ] Service has property-based tests where applicable
- [ ] Service is stateless or uses appropriate synchronization
- [ ] Service errors are well-defined and documented
- [ ] Service is integrated into AppState correctly
- [ ] View code is simplified and only handles UI
- [ ] No functional regressions introduced
- [ ] Documentation is complete and accurate