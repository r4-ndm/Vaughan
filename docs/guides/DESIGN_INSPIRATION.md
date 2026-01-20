# Design Inspiration Documentation

This document tracks the design patterns and inspiration sources used in Vaughan wallet development to ensure proper attribution and future reference for updates.

## Token Selection & Registry System

### Primary Inspiration: Trust Wallet
- **Source**: Trust Wallet Assets Repository (https://github.com/trustwallet/assets)
- **Pattern Used**: Token registry structure with symbol-to-address mapping
- **Implementation**: `src/gui/handlers/transaction.rs` lines 86-111 and 285-310
- **Date Added**: 2025-12-04
- **Rationale**: Industry-standard approach used by most major wallets

### Token Registry Structure:
```json
{
    "symbol": "TOKEN",
    "address": "0x123...",
    "name": "Token Name",
    "decimals": 18
}
```

### Secondary Research: MetaMask
- **Source**: MetaMask Documentation (https://support.metamask.io/)
- **Pattern Observed**: Dynamic token addition with contract address validation
- **Not Implemented Yet**: Auto-detection of popular tokens
- **Future Consideration**: Token list aggregation from multiple sources

## Gas Estimation Strategy

### Current Implementation: Conservative Buffer Approach
- **Buffer**: 30% above estimated gas
- **Minimum Gas**: 50,000 for tokens, 21,000 for native
- **Inspiration**: Common practice across MetaMask, Trust Wallet, Rainbow
- **Location**: `src/gui/simple_transaction.rs` lines 461-464

## Transaction Construction

### ERC-20 vs Native Separation
- **Pattern**: Separate code paths for ERC-20 and native transactions
- **Inspiration**: Ethereum best practices (not wallet-specific)
- **Implementation**: `src/gui/simple_transaction.rs` lines 203-226

## Cross-Language Pattern Translation

### How We Adapt Non-Rust Wallets:
- **Logic Flow**: Translate the decision tree and validation steps
- **Data Structures**: Map JavaScript objects to Rust structs
- **Error Handling**: Adapt error messages and recovery strategies
- **User Experience**: Replicate interaction patterns and feedback

### Example Translation Process:
1. **Study the pattern** in source wallet (MetaMask JS, Trust Wallet Swift)
2. **Extract the logic** independent of language syntax
3. **Implement in Rust** using appropriate libraries (Alloy, etc.)
4. **Document the mapping** in this file

## Future Update Strategy

### When to Update Inspiration Sources:
1. **Trust Wallet Assets**: Check quarterly for new token standards
2. **MetaMask Patterns**: Monitor for UX improvements in token management
3. **Industry Standards**: Follow EIP proposals for wallet standards

### Tracking Updates:
- Update this document when adding new patterns
- Include source links and implementation locations
- Note rationale for choosing specific patterns over alternatives
- Document the pattern translation approach used