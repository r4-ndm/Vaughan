# Business Logic Extraction - Task List

## Overview
This task list implements the Business Logic Extraction Plan to separate business logic from view components into a dedicated service layer.

**Estimated Duration**: 4-6 weeks
**Priority**: High
**Risk Level**: Medium-High (Transaction logic is critical path)

---

## Phase 1: Foundation (Week 1)

### 1.1 Create Service Infrastructure
- [ ] Create `src/gui/services/` directory structure
- [ ] Create `src/gui/services/mod.rs` with service registry
- [ ] Define service trait pattern and conventions
- [ ] Add service registry to `AppState`
- [ ] Document service architecture in README

### 1.2 Define Service Traits
- [ ] Create `AssetServiceTrait` in `src/gui/services/asset_service.rs`
- [ ] Create `AccountDisplayServiceTrait` in `src/gui/services/account_display_service.rs`
- [ ] Create `NetworkConfigServiceTrait` in `src/gui/services/network_config_service.rs`
- [ ] Create `TransactionFormServiceTrait` in `src/gui/services/transaction_form_service.rs`
- [ ] Add trait documentation with usage examples

### 1.3 Implement Basic Services
- [ ] Implement `AssetService` with file system operations
- [ ] Implement `AccountDisplayService` with formatting logic
- [ ] Implement `NetworkConfigService` with validation
- [ ] Implement `TransactionFormService` with form preparation
- [ ] Add error types for each service

### 1.4 Dependency Injection Setup
- [ ] Add `ServiceRegistry` struct to `AppState`
- [ ] Implement lazy initialization for services
- [ ] Add service accessor methods to `AppState`
- [ ] Test service creation and access patterns
- [ ] Document DI pattern for team

### 1.5 Foundation Testing
- [ ] Write unit tests for `AssetService`
- [ ] Write unit tests for `AccountDisplayService`
- [ ] Write unit tests for `NetworkConfigService`
- [ ] Write unit tests for `TransactionFormService`
- [ ] Achieve >90% test coverage for all services

---

## Phase 2: Asset Service Migration (Week 1-2)

### 2.1 Identify Asset Operations
- [ ] Audit `src/gui/views/main_wallet.rs` for file operations
- [ ] Audit `src/gui/views/dialogs.rs` for file operations
- [ ] Document all asset paths used in views
- [ ] Create migration checklist for each file operation
- [ ] Prioritize by usage frequency

### 2.2 Implement Asset Service Methods
- [ ] Implement `is_logo_available()` method
- [ ] Implement `get_logo_path()` method
- [ ] Implement `is_icon_available(icon_name)` method
- [ ] Implement `get_icon_path(icon_name)` method
- [ ] Add caching for asset availability checks

### 2.3 Migrate Logo Operations
- [ ] Replace logo check in `main_wallet.rs` line 29
- [ ] Update logo rendering logic to use service
- [ ] Test logo display with existing asset
- [ ] Test logo fallback when asset missing
- [ ] Remove `std::path` import from `main_wallet.rs`

### 2.4 Migrate Icon Operations
- [ ] Replace hamburger icon check (line 100)
- [ ] Replace account delete icon check (line 209)
- [ ] Replace network selector icon check (line 267)
- [ ] Replace copy address icon check (line 343)
- [ ] Replace copy transaction hash icon check (line 386)

### 2.5 Asset Service Testing
- [ ] Write integration tests for asset loading
- [ ] Test with missing assets
- [ ] Test with corrupted asset paths
- [ ] Add property-based tests for path handling
- [ ] Benchmark asset loading performance

### 2.6 Asset Service Cleanup
- [ ] Remove all `std::path::Path` imports from views
- [ ] Remove all direct file system calls from views
- [ ] Update documentation for asset usage
- [ ] Code review for asset service migration
- [ ] Merge asset service changes

---

## Phase 3: Account Display Migration (Week 2)

### 3.1 Identify Account Display Logic
- [ ] Audit account selection logic (lines 58-84 in `main_wallet.rs`)
- [ ] Audit account formatting in dialogs
- [ ] Document all account display patterns
- [ ] Identify address formatting requirements
- [ ] Create migration plan for account logic

### 3.2 Implement Account Display Methods
- [ ] Implement `format_account_display(account)` method
- [ ] Implement `format_address_short(address)` method
- [ ] Implement `get_account_type_label(account)` method
- [ ] Add `AccountDisplayInfo` struct
- [ ] Handle edge cases (empty names, invalid addresses)

### 3.3 Migrate Account Selector
- [ ] Extract account selection logic from `main_wallet.rs`
- [ ] Update account selector to use service
- [ ] Test account switching functionality
- [ ] Test with multiple account types
- [ ] Verify account display consistency

### 3.4 Migrate Address Formatting
- [ ] Replace inline address formatting with service calls
- [ ] Update all address displays to use `format_address_short()`
- [ ] Test with various address lengths
- [ ] Test with invalid addresses
- [ ] Ensure consistent formatting across views

### 3.5 Account Display Testing
- [ ] Write unit tests for address formatting
- [ ] Write property-based tests for address truncation
- [ ] Test account type label generation
- [ ] Test with edge cases (empty accounts, null values)
- [ ] Integration tests for account selector

### 3.6 Account Display Cleanup
- [ ] Remove inline account formatting from views
- [ ] Simplify account rendering logic
- [ ] Update documentation for account display
- [ ] Code review for account service migration
- [ ] Merge account display changes

---

## Phase 4: Network Configuration Migration (Week 3)

### 4.1 Identify Network Logic
- [ ] Audit network selection logic (lines 86-91 in `main_wallet.rs`)
- [ ] Audit network configuration dialog (lines 1149-1216 in `dialogs.rs`)
- [ ] Document network validation requirements
- [ ] Identify all network-related business rules
- [ ] Create migration plan for network logic

### 4.2 Implement Network Validation
- [ ] Implement `validate_network_config(config)` method
- [ ] Implement RPC URL validation
- [ ] Implement chain ID validation
- [ ] Implement explorer URL validation
- [ ] Add `NetworkValidationError` enum

### 4.3 Implement Network Utilities
- [ ] Implement `is_network_name_unique(name, existing)` method
- [ ] Implement `sanitize_rpc_url(url)` method
- [ ] Implement network form data preparation
- [ ] Add network configuration helpers
- [ ] Handle edge cases (duplicate networks, invalid URLs)

### 4.4 Migrate Network Configuration Dialog
- [ ] Extract validation logic from dialog
- [ ] Update dialog to use service for validation
- [ ] Test network creation with valid data
- [ ] Test network creation with invalid data
- [ ] Verify error messages are user-friendly

### 4.5 Migrate Network Selector
- [ ] Update network selector to use service
- [ ] Test network switching functionality
- [ ] Test with custom networks
- [ ] Test with default networks
- [ ] Verify network persistence

### 4.6 Network Service Testing
- [ ] Write unit tests for URL validation
- [ ] Write unit tests for chain ID validation
- [ ] Write property-based tests for URL sanitization
- [ ] Test duplicate network detection
- [ ] Integration tests for network configuration

### 4.7 Network Service Cleanup
- [ ] Remove inline validation from dialogs
- [ ] Simplify network configuration UI
- [ ] Update documentation for network service
- [ ] Code review for network service migration
- [ ] Merge network service changes

---

## Phase 5: Transaction Form Migration (Week 3-4) ⚠️ HIGH RISK

### 5.1 Risk Mitigation Setup
- [ ] Create feature flag `use_transaction_service` in `AppState`
- [ ] Implement parallel transaction form rendering
- [ ] Set up extensive logging for transaction operations
- [ ] Create rollback plan document
- [ ] Schedule team review for transaction migration

### 5.2 Identify Transaction Logic
- [ ] Audit send form logic (lines 154-177 in `main_wallet.rs`)
- [ ] Audit transaction validation in dialogs
- [ ] Document all transaction business rules
- [ ] Identify balance calculation logic
- [ ] Create detailed migration plan

### 5.3 Implement Transaction Form Methods
- [ ] Implement `prepare_send_form(state)` method
- [ ] Implement `validate_recipient(address)` method
- [ ] Implement `validate_amount(amount, balance)` method
- [ ] Add `SendFormData` struct
- [ ] Add `TransactionValidationError` enum

### 5.4 Implement Transaction Validation
- [ ] Implement recipient address validation
- [ ] Implement amount validation (positive, non-zero)
- [ ] Implement balance sufficiency check
- [ ] Implement gas limit validation
- [ ] Handle edge cases (zero amounts, max values)

### 5.5 Migrate Send Form (Gradual Rollout)
- [ ] Implement service-based send form rendering
- [ ] Keep legacy send form rendering
- [ ] Add feature flag toggle in settings
- [ ] Test both implementations in parallel
- [ ] Compare outputs for consistency

### 5.6 Transaction Service Testing (Critical)
- [ ] Write comprehensive unit tests for validation
- [ ] Write property-based tests for amount validation
- [ ] Write property-based tests for address validation
- [ ] Test with edge cases (max uint, zero, negative)
- [ ] Integration tests for full transaction flow

### 5.7 Transaction Service Validation
- [ ] Test with real transaction data
- [ ] Test with invalid recipients
- [ ] Test with insufficient balance
- [ ] Test with various token types
- [ ] Verify no regressions in transaction submission

### 5.8 Transaction Service Rollout
- [ ] Enable feature flag for 10% of operations
- [ ] Monitor for errors and performance issues
- [ ] Enable feature flag for 50% of operations
- [ ] Monitor for one week
- [ ] Enable feature flag for 100% of operations

### 5.9 Transaction Service Cleanup
- [ ] Remove legacy transaction form code
- [ ] Remove feature flag
- [ ] Simplify transaction UI logic
- [ ] Update documentation for transaction service
- [ ] Code review for transaction service migration
- [ ] Merge transaction service changes

---

## Phase 6: Cleanup and Optimization (Week 4)

### 6.1 Code Cleanup
- [ ] Remove unused imports from all view files
- [ ] Remove commented-out legacy code
- [ ] Simplify view methods using services
- [ ] Ensure consistent service usage patterns
- [ ] Run clippy and fix all warnings

### 6.2 Performance Optimization
- [ ] Add caching to AssetService
- [ ] Implement lazy initialization for all services
- [ ] Profile service call overhead
- [ ] Optimize hot paths in services
- [ ] Benchmark before/after performance

### 6.3 Documentation
- [ ] Write service layer architecture guide
- [ ] Document each service's API
- [ ] Add usage examples for each service
- [ ] Update view layer documentation
- [ ] Create migration guide for future services

### 6.4 Testing Completeness
- [ ] Verify >90% test coverage for all services
- [ ] Add missing edge case tests
- [ ] Add integration tests for service interactions
- [ ] Add end-to-end tests for critical paths
- [ ] Review and improve test quality

### 6.5 Code Review
- [ ] Schedule architecture review with team
- [ ] Address code review feedback
- [ ] Ensure all acceptance criteria met
- [ ] Verify no functional regressions
- [ ] Get approval from 2+ reviewers

### 6.6 Final Validation
- [ ] Run full test suite
- [ ] Run performance benchmarks
- [ ] Test in staging environment
- [ ] Verify all metrics achieved
- [ ] Prepare release notes

---

## Acceptance Criteria

### Must Have (Blocking)
- [ ] Zero file operations in view layer
- [ ] Zero validation logic in view layer
- [ ] All services have trait interfaces
- [ ] All services have >90% test coverage
- [ ] Zero functional regressions
- [ ] View files reduced by 40-60% in complexity

### Should Have (Important)
- [ ] Performance benchmarks show no degradation
- [ ] Documentation complete for all services
- [ ] Code review approved by 2+ team members
- [ ] Integration tests cover all service interactions
- [ ] Property-based tests for validation logic

### Nice to Have (Optional)
- [ ] Caching implemented for expensive operations
- [ ] Metrics/telemetry for service calls
- [ ] Service layer can be used outside GUI context
- [ ] Mock implementations for all services

---

## Risk Management

### High Risk Items
1. **Transaction Form Service** - Critical path for wallet functionality
   - Mitigation: Feature flag, parallel implementation, extensive testing
   
2. **Balance Calculations** - Financial impact if incorrect
   - Mitigation: Property-based tests, comparison with legacy code

3. **Account Selection** - Core functionality
   - Mitigation: Integration tests, manual testing

### Rollback Plan
If critical issues are discovered:
1. Disable feature flag immediately
2. Revert to legacy implementation
3. Investigate and fix issues
4. Re-test thoroughly before re-enabling

### Monitoring
- Add logging for all service calls
- Monitor error rates in production
- Track performance metrics
- Set up alerts for anomalies

---

## Success Metrics

### Quantitative
- [ ] 0 file operations in views (measured by grep)
- [ ] >90% test coverage for services (measured by cargo tarpaulin)
- [ ] 40-60% reduction in view file complexity (measured by LOC)
- [ ] <5% performance degradation (measured by benchmarks)

### Qualitative
- [ ] Code is easier to understand (team survey)
- [ ] Business logic is easier to test (developer feedback)
- [ ] Views are simpler and more maintainable (code review)

---

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | Foundation + Asset Service | Service infrastructure, Asset service complete |
| 2 | Account Display Service | Account formatting extracted |
| 3 | Network Configuration Service | Network validation extracted |
| 3-4 | Transaction Form Service | Transaction logic extracted (gradual) |
| 4 | Cleanup + Optimization | Documentation, testing, review |
| 5 | Testing + Review | Final validation, code review |
| 6 | Rollout | Gradual production rollout |

---

## Notes

- This is a living document - update as tasks are completed
- Mark tasks as complete with `[x]` when done
- Add notes for any blockers or issues encountered
- Update timeline if delays occur
- Communicate progress in daily standups

---

## Related Documents
- [Business Logic Extraction Plan](./BUSINESS_LOGIC_EXTRACTION_PLAN.md)
- [Service Layer Design Pattern](https://martinfowler.com/eaaCatalog/serviceLayer.html)
- [Testing Strategy](../../guides/DEVELOPMENT_RULES.md)
