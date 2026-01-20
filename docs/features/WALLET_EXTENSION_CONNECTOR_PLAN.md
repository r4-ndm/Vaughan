# Wallet Extension Connector Plan for Vaughan Wallet

## Overview

This document outlines a comprehensive plan to connect a MetaMask-compatible browser extension/connector to the Vaughan wallet, enabling dApp integration and web3 provider functionality.

## Current State Analysis

Based on codebase examination:
- Vaughan wallet has robust hardware wallet support
- Comprehensive GUI wallet management exists in `src/gui/`
- Account management, transaction signing, and network switching are implemented
- **Missing**: Browser extension connector and web3 provider interface

## Phase 1: Architecture Design

### 1.1 Component Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Browser Extension│◄──►│ Wallet Connector │◄──►│ Vaughan Wallet  │
│ (Content Script) │    │   (Rust Module)   │    │   (Core Logic)  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
   Web3 Provider           Message Passing        Secure Storage
   (JavaScript)           (IPC/Bridge)           (Hardware/Software)
```

### 1.2 Communication Flow
1. **dApp Request** → Browser Extension (JavaScript)
2. **Extension** → Wallet Connector (Native messaging/IPC)
3. **Connector** → Vaughan Core (Rust calls)
4. **Vaughan Response** → Extension → dApp

## Phase 2: Core Components

### 2.1 Web3 Provider Interface
**Location**: `src/wallet/extension/`

**Files to Create**:
- `web3_provider.rs` - Main web3.js compatible interface
- `eth_api.rs` - Ethereum JSON-RPC method implementations
- `account_manager.rs` - Account selection and permissions
- `permission_manager.rs` - dApp authorization system

**Key Methods**:
```rust
pub trait Web3Provider {
    async fn request_accounts(&self) -> Result<Vec<Address>, ProviderError>;
    async fn send_transaction(&self, tx: TransactionRequest) -> Result<TxHash, ProviderError>;
    async fn sign_message(&self, message: Vec<u8>, address: Address) -> Result<Signature, ProviderError>;
    async fn switch_chain(&self, chain_id: U256) -> Result<(), ProviderError>;
    async fn add_chain(&self, chain_config: ChainConfig) -> Result<(), ProviderError>;
}
```

### 2.2 Browser Extension Bridge
**Location**: `src/extension/`

**Files to Create**:
- `native_messaging.rs` - Native messaging host for browser communication
- `message_handler.rs` - Process extension requests/responses
- `protocol.rs` - Define communication protocol

**Native Message Format**:
```rust
#[derive(Serialize, Deserialize)]
pub struct ExtensionMessage {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub origin: String, // dApp domain for permissions
}
```

### 2.3 Permission System
**Location**: `src/security/permissions/`

**Files to Create**:
- `mod.rs` - Permission management module
- `dapp_permissions.rs` - dApp-specific permissions
- `approval_dialog.rs` - GUI for user permission approval

**Permission Types**:
- `RequestAccounts` - Access to account addresses
- `SignTransaction` - Transaction signing
- `SignMessage` - Message signing
- `SwitchChain` - Network switching

## Phase 3: Browser Extension Implementation

### 3.1 Content Script
**Location**: `extension/content/` (separate from Rust project)

**Files to Create**:
- `provider.js` - Injects web3 provider into dApps
- `message_sender.js` - Communicates with native host
- `utils.js` - Helper functions

**Key Features**:
- `window.ethereum` object injection
- Event emission for account/network changes
- Request/response correlation

### 3.2 Background Script
**Location**: `extension/background/`

**Files to Create**:
- `native_host.js` - Manages native messaging connection
- `storage.js` - Extension state persistence
- `ui.js` - Notification and approval UI

### 3.3 Extension Manifest
**Location**: `extension/manifest.json`

**Features**:
- Native messaging host registration
- Content script injection rules
- Permissions for storage and native messaging

## Phase 4: Integration Points

### 4.1 GUI Integration
**File**: `src/gui/wallet_messages.rs` (extend)
**Add Messages**:
```rust
// Extension integration messages
ExtensionRequest(ExtensionMessage),
ExtensionResponse(ExtensionMessage),
DappPermissionRequested(DappPermissionRequest),
DappPermissionApproved(String, bool), // origin, approved
AccountSwitchRequested(String), // account_id
NetworkSwitchRequested(NetworkId),
```

### 4.2 Wallet Core Integration
**File**: `src/wallet/mod.rs` (extend)
**Add Methods**:
```rust
impl Vaughan {
    pub async fn handle_extension_request(&self, request: ExtensionMessage) -> Result<serde_json::Value, WalletError>;
    pub fn get_extension_accounts(&self) -> Result<Vec<Address>, WalletError>;
    pub async fn sign_extension_transaction(&self, tx: TransactionRequest, from: Address) -> Result<TxHash, WalletError>;
    pub fn check_permission(&self, origin: &str, method: &str) -> Result<bool, WalletError>;
}
```

### 4.3 Network Integration
**File**: `src/network/mod.rs` (extend)
**Add Support**:
- Chain switching notifications to extension
- Custom network addition from dApps
- Network state synchronization

## Phase 5: Security Model

### 5.1 Permission Granting
1. **First Request**: Show approval dialog
2. **Remember Choice**: Store permission preferences
3. **Revocation**: Allow users to revoke permissions

### 5.2 Isolation
- **Origin Validation**: Check dApp domain for all requests
- **Permission Scoping**: Limit access per dApp and method
- **Time-based Expiration**: Optional permission expiration

### 5.3 Security Measures
- **Message Authentication**: Verify message integrity
- **Rate Limiting**: Prevent abuse of wallet operations
- **Audit Logging**: Log all extension interactions

## Phase 6: Implementation Steps

### Step 1: Core Web3 Provider (Week 1)
1. Create `src/wallet/extension/` module structure
2. Implement basic `Web3Provider` trait
3. Add core Ethereum JSON-RPC methods:
   - `eth_requestAccounts`
   - `eth_sendTransaction`
   - `personal_sign`
   - `eth_chainId`

### Step 2: Native Messaging (Week 1-2)
1. Create `src/extension/` module
2. Implement native messaging host
3. Set up IPC communication protocol
4. Test basic message passing

### Step 3: Permission System (Week 2)
1. Create permission management system
2. Implement approval dialog in GUI
3. Add permission persistence
4. Test permission workflows

### Step 4: Browser Extension (Week 2-3)
1. Create extension directory structure
2. Implement content script with web3 provider
3. Create background script for native communication
4. Set up manifest.json

### Step 5: GUI Integration (Week 3)
1. Add extension messages to GUI
2. Create permission approval UI
3. Add extension status indicators
4. Implement account/network switching notifications

### Step 6: Testing & Refinement (Week 4)
1. Test with popular dApps (Uniswap, OpenSea, etc.)
2. Fix compatibility issues
3. Optimize performance
4. Complete error handling

## Phase 7: Testing Strategy

### 7.1 Unit Tests
- Web3 provider method implementations
- Permission system logic
- Message serialization/deserialization

### 7.2 Integration Tests
- Extension ↔ Native host communication
- Native host ↔ Vaughan wallet integration
- End-to-end dApp interaction flows

### 7.3 dApp Compatibility Testing
- **DeFi**: Uniswap, Aave, Compound
- **NFT**: OpenSea, Rarible
- **Gaming**: Web3 games
- **Tools**: Etherscan, Dextools

## Phase 8: Configuration & Deployment

### 8.1 Native Host Registration
**Platform-specific installers**:
- **Windows**: Registry entries for native messaging
- **macOS**: Native messaging host manifest in ~/Library/Application Support/
- **Linux**: Native messaging host manifest in ~/.config/

### 8.2 Extension Distribution
- **Chrome Web Store**: Chrome/Brave/Edge support
- **Firefox Add-ons**: Firefox support
- **Safari App Store**: Safari support (future)

## Phase 9: File Structure

```
src/
├── wallet/
│   ├── extension/
│   │   ├── mod.rs
│   │   ├── web3_provider.rs
│   │   ├── eth_api.rs
│   │   ├── account_manager.rs
│   │   └── permission_manager.rs
│   └── mod.rs (extended)
├── extension/
│   ├── mod.rs
│   ├── native_messaging.rs
│   ├── message_handler.rs
│   └── protocol.rs
├── security/
│   └── permissions/
│       ├── mod.rs
│       ├── dapp_permissions.rs
│       └── approval_dialog.rs
└── gui/
    └── wallet_messages.rs (extended)

extension/ (browser extension directory)
├── manifest.json
├── content/
│   ├── provider.js
│   ├── message_sender.js
│   └── utils.js
├── background/
│   ├── native_host.js
│   ├── storage.js
│   └── ui.js
└── popup/
    ├── popup.html
    ├── popup.js
    └── style.css
```

## Phase 10: Success Criteria

### 10.1 Functional Requirements
- [ ] dApps can request account access
- [ ] Transaction signing works seamlessly
- [ ] Network switching is functional
- [ ] Permissions are properly managed
- [ ] Security measures are enforced

### 10.2 Compatibility Requirements
- [ ] Works with major dApps (Uniswap, OpenSea)
- [ ] Compatible with Chrome, Firefox, Edge
- [ ] Supports all Vaughan wallet networks
- [ ] MetaMask API compatibility

### 10.3 Performance Requirements
- [ ] Response time < 500ms for most operations
- [ ] Minimal impact on wallet performance
- [ ] Stable connection with extension

## Risks & Mitigations

### Risk 1: Browser API Changes
**Mitigation**: Keep abstraction layer, monitor browser updates

### Risk 2: Security Vulnerabilities
**Mitigation**: Comprehensive security audit, permission model review

### Risk 3: dApp Compatibility Issues
**Mitigation**: Extensive testing with popular dApps, community feedback

### Risk 4: Performance Bottlenecks
**Mitigation**: Performance testing, async operations, caching

## Open Source Libraries and Reference Implementations

### Core Web3 Provider Libraries

#### 1. Alloy (Recommended Primary Choice)
**Repository**: https://github.com/alloy-rs/alloy  
**Status**: ✅ Actively Maintained (Current Standard)  
**Why Choose Alloy**:
- Complete replacement for deprecated `ethers-rs`
- Comprehensive Ethereum JSON-RPC client support
- Full transaction signing capabilities
- Hardware wallet integration (Ledger, Trezor)
- Multi-chain support with built-in network configurations
- Modern async/await architecture
- Strong type safety and performance

**Key Components for Our Use**:
```toml
[dependencies]
alloy = { version = "1", features = ["full"] }
```

- `alloy-provider` - JSON-RPC client for blockchain communication
- `alloy-signer` - Wallet implementations and signing
- `alloy-transport` - Network transport layer (HTTP, WebSocket, IPC)
- `alloy-rpc-client` - RPC client implementation
- `alloy-contract` - Smart contract interaction utilities

#### 2. Alternative Options

**ethcontract-rs**: Specialized for contract interaction, strong type safety
- **Best for**: Contract-heavy applications
- **Limitations**: Less comprehensive wallet functionality compared to Alloy

### Browser Extension Reference Implementations

#### 1. MetaMask Extension (Reference Architecture)
**Repository**: https://github.com/MetaMask/metamask-extension  
**Key Learnings**:
- Provider injection patterns into dApps
- Permission management system
- Event emission for account/network changes
- Content script and background script architecture
- Security isolation between trusted and untrusted components

#### 2. Extensionizer (Cross-Browser Compatibility)
**Repository**: https://github.com/MetaMask/extensionizer  
**Purpose**: Cross-browser extension development utilities
**Benefits**: Abstracts browser differences for Chrome, Firefox, Edge

#### 3. Browser Native Messaging Patterns

**MDN Documentation**: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging  
**Key Implementation Patterns**:

**JSON Message Format**:
- 32-bit length header + UTF-8 JSON content
- Maximum 1MB messages (app to browser)
- Maximum 4GB messages (browser to app)

**Communication Models**:
```javascript
// Connection-based (persistent)
const port = browser.runtime.connectNative("vaughan_wallet");
port.onMessage.addListener(handleMessage);
port.postMessage(request);

// Connectionless (one-off)
browser.runtime.sendNativeMessage("vaughan_wallet", request)
    .then(handleResponse);
```

**Platform Registration**:
- **Windows**: Registry entries pointing to manifest files
- **macOS**: Manifest files in `~/Library/Application Support/`
- **Linux**: Manifest files in `~/.config/`

### Native Messaging Host Implementations

#### 1. Reference Native Host Examples
**Repository**: https://github.com/mdn/webextensions-examples  
**Languages Available**: Python, Node.js, C++, Rust examples

**Key Rust Implementation Pattern**:
```rust
use std::io::{self, Read, Write};
use std::mem;

fn read_message() -> Result serde_json::Value {
    let mut buffer = [0u8; 4];
    io::stdin().read_exact(&mut buffer)?;
    let length = u32::from_le_bytes(buffer) as usize;
    
    let mut message = vec![0u8; length];
    io::stdin().read_exact(&mut buffer)?;
    
    Ok(serde_json::from_slice(&message)?)
}

fn send_message(message: &serde_json::Value) -> Result<()> {
    let json = serde_json::to_vec(message)?;
    let length = (json.len() as u32).to_le_bytes();
    
    io::stdout().write_all(&length)?;
    io::stdout().write_all(&json)?;
    io::stdout().flush()?;
    Ok(())
}
```

#### 2. Platform-Specific Integration

**Windows Registry Setup**:
```reg
HKEY_CURRENT_USER\Software\Mozilla\NativeMessagingHosts\vaughan_wallet
= "C:\\Program Files\\Vaughan\\vaughan_wallet.json"

HKEY_CURRENT_USER\Software\Google\Chrome\\NativeMessagingHosts\vaughan_wallet  
= "C:\\Program Files\\Vaughan\\vaughan_wallet.json"
```

**Host Manifest (vaughan_wallet.json)**:
```json
{
  "name": "vaughan_wallet",
  "description": "Vaughan Wallet Extension Host",
  "path": "C:\\Program Files\\Vaughan\\vaughan_extension_host.exe",
  "type": "stdio",
  "allowed_extensions": ["vaughan_extension@yourdomain.com"],
  "allowed_origins": ["chrome-extension://extension_id/"]
}
```

### Web3 Provider Patterns

#### 1. EIP-1193 Provider Interface
**Standard**: https://eips.ethereum.org/EIPS/eip-1193  
**Required Methods**:
- `eth_requestAccounts()` - Request account access
- `eth_sendTransaction()` - Send transaction
- `personal_sign()` - Sign messages
- `eth_chainId()` - Get current chain ID
- `wallet_switchEthereumChain()` - Switch networks
- `wallet_addEthereumChain()` - Add custom networks

**Event Emissions**:
- `accountsChanged` - Account selection changed
- `chainChanged` - Network changed
- `connect` - Connection established
- `disconnect` - Connection lost

#### 2. Provider Implementation Pattern
```javascript
// Content script injection
const provider = {
    async request({ method, params }) {
        return await sendToNativeHost({ method, params });
    },
    on: { emit, addListener, removeListener }
};

// Inject into dApp
window.ethereum = provider;
window.dispatchEvent(new Event('ethereum#initialized'));
```

### Security Reference Implementations

#### 1. Permission Management
- Study MetaMask's origin-based permission system
- Implement domain validation and approval flows
- Persistent permission storage with revocation

#### 2. Isolation Patterns
- Separate untrusted UI from trusted wallet logic
- Use native host for all cryptographic operations
- Validate and sanitize all cross-boundary communications

### Updated Technology Stack Recommendations

#### Primary Stack:
- **Rust + Alloy**: Core wallet functionality and web3 provider
- **TypeScript**: Extension UI and browser integration
- **Native Messaging**: Secure communication bridge
- **MDN Examples**: Platform registration and messaging patterns

#### Development Tools:
- **web-ext**: Firefox extension development
- **Chrome Extensions**: Chrome Developer Tools
- **Rust**: Cargo for Rust package management
- **TypeScript**: npm/yarn for JavaScript dependencies

### Implementation Benefits

By using these battle-tested open source components:

1. **Alloy**: Proven production-ready Ethereum library with active maintenance
2. **MetaMask Patterns**: Industry-standard dApp integration approach  
3. **MDN Native Messaging**: Cross-browser compatible communication
4. **EIP-1193**: Standardized provider interface for maximum dApp compatibility

This approach significantly reduces development risk by leveraging well-established, security-audited components while maintaining flexibility for Vaughan-specific features.

## Conclusion

This plan provides a roadmap for implementing a MetaMask-compatible browser extension connector for Vaughan wallet using proven open source components. The implementation leverages Alloy for core Ethereum functionality, MetaMask patterns for dApp compatibility, and standard native messaging for secure browser communication.

The estimated timeline is 4 weeks for a complete implementation, with parallel development possible for the Rust backend and browser extension frontend.

Next steps:
1. Review and approve this plan including open source components
2. Begin Phase 1 architecture implementation using Alloy
3. Set up development environment for extension development
4. Start with core Web3 provider implementation
5. Reference MDN examples for native messaging setup