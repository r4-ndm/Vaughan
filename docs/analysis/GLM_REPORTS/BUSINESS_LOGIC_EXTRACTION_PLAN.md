# Business Logic Extraction Plan

## Summary
Analysis of business logic embedded in view components that should be moved to service layer for better separation of concerns.

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

## Service Layer Design

### 1. Asset Service
```rust
pub struct AssetService;

impl AssetService {
    pub fn is_logo_available() -> bool {
        std::path::Path::new("assets/vaughan-logo-513x76-thor.png").exists()
    }
    
    pub fn is_icon_available(icon_name: &str) -> bool {
        std::path::Path::new(&format!("assets/{}.png", icon_name)).exists()
    }
    
    pub fn get_logo_element() -> Element<Message> {
        if Self::is_logo_available() {
            // Return logo image button
        } else {
            // Return text fallback
        }
    }
}
```

### 2. Account Selection Service
```rust
pub struct AccountSelectionService;

impl AccountSelectionService {
    pub fn get_current_account_display(state: &AppState) -> AccountDisplayInfo {
        // Extract account selection logic
    }
    
    pub fn format_address_display(account: &Account) -> String {
        // Extract address formatting logic
    }
}
```

### 3. Network Configuration Service
```rust
pub struct NetworkConfigurationService;

impl NetworkConfigurationService {
    pub fn validate_network_config(config: &NetworkConfig) -> Result<(), NetworkValidationError> {
        // Extract validation logic
    }
    
    pub fn create_network_form_data(state: &AppState) -> NetworkFormData {
        // Extract form data preparation
    }
}
```

### 4. Transaction Form Service
```rust
pub struct TransactionFormService;

impl TransactionFormService {
    pub fn prepare_send_form(state: &AppState) -> SendFormData {
        // Extract form preparation logic
    }
    
    pub fn validate_transaction_data(data: &SendFormData) -> Result<(), TransactionValidationError> {
        // Extract validation logic
    }
}
```

## Dependencies to Inject

### View Layer Dependencies
```rust
pub struct ViewServices {
    asset_service: Arc<AssetService>,
    account_service: Arc<AccountSelectionService>,
    network_service: Arc<NetworkConfigurationService>,
    transaction_service: Arc<TransactionFormService>,
}
```

### Function Categorization

#### View-Pure Functions (Keep in Views)
- Layout composition
- Widget styling
- UI event routing
- Display formatting

#### Service-Bound Functions (Move to Services)
- File system operations
- Data validation
- Complex business rules
- State calculations
- Network operations

## Migration Strategy

### Phase 1: Create Service Interfaces
1. Define service traits
2. Create service implementations
3. Add dependency injection to AppState

### Phase 2: Extract File Operations
1. Create `AssetService`
2. Replace all file checks in views
3. Update UI components to use service

### Phase 3: Extract Business Logic
1. Move data processing to services
2. Update view methods to use services
3. Remove complex logic from views

### Phase 4: Validation
1. Ensure views are pure UI
2. Test business logic separation
3. Verify UI functionality unchanged

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

## Risk Assessment

### Low Risk
- File system operations (clear separation)
- Asset availability checks
- Simple data transformations

### Medium Risk
- Complex state interactions
- Form validation logic
- Network configuration

### High Risk
- Transaction form logic (critical path)
- Account selection (core functionality)
- Balance calculations (financial impact)

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
- **Zero file operations in views**
- **Zero network calls in views**
- **All business logic in services**
- **Views only handle UI composition**
- **Services have comprehensive unit tests**