# Vaughan Wallet Development Rules

**These rules are MANDATORY for all development work. Any agent or developer working on this codebase MUST follow these guidelines.**

## 🚨 CRITICAL RULES - NO EXCEPTIONS

### 1. **USE ALLOY FOR EVERYTHING ETHEREUM**
- ✅ **ALWAYS** use Alloy libraries for wallet operations
- ✅ **ALWAYS** use Alloy for transaction handling
- ✅ **ALWAYS** use Alloy for network communication
- ❌ **NEVER** create custom wallet/crypto implementations
- ❌ **NEVER** reinvent Ethereum protocol handling

**Examples:**
```rust
// ✅ CORRECT - Use Alloy
use alloy::signers::local::PrivateKeySigner;
let wallet = PrivateKeySigner::random();

// ❌ WRONG - Custom wallet implementation
struct CustomWallet { /* ... */ }
```

### 2. **SIMPLICITY OVER COMPLEXITY**
- ✅ Use the simplest solution that works
- ✅ Prefer standard library functions
- ✅ Use established patterns from Alloy documentation
- ❌ No custom security layers unless absolutely necessary
- ❌ No over-abstraction

### 3. **NO CUSTOM CRYPTO**
- ✅ Use Alloy's built-in security
- ✅ Use standard Ethereum key derivation
- ❌ **NEVER** implement custom encryption
- ❌ **NEVER** create custom key management
- ❌ **NEVER** write custom cryptographic functions

## 📋 IMPLEMENTATION STANDARDS

### Wallet Operations
```rust
// ✅ CORRECT - Simple Alloy wallet creation
use alloy::signers::local::PrivateKeySigner;
let wallet = PrivateKeySigner::random();
let address = wallet.address();

// ✅ CORRECT - From seed phrase
let wallet = PrivateKeySigner::from_phrase(&mnemonic)?;

// ❌ WRONG - Custom wallet config systems
let config = WalletConfig::new(name, password)?;
storage.save_wallet_config(config).await?;
```

### Transaction Handling
```rust
// ✅ CORRECT - Use Alloy transaction builder
let tx = TransactionRequest::default()
    .to(to_address)
    .value(amount)
    .gas_limit(21000);

// ❌ WRONG - Custom transaction types
struct CustomTransaction { /* ... */ }
```

### Network Communication
```rust
// ✅ CORRECT - Use Alloy providers
let provider = ProviderBuilder::new()
    .connect_http(rpc_url)
    .await?;

// ❌ WRONG - Custom RPC implementations
struct CustomRpcClient { /* ... */ }
```

## 🛠️ DEVELOPMENT WORKFLOW

### Before Adding ANY New Code:
1. **Check Alloy documentation first**
2. **Look for existing Alloy examples**
3. **Use the simplest Alloy approach**
4. **If custom code needed, justify why Alloy can't do it**

### Code Review Checklist:
- [ ] Does this use Alloy where possible?
- [ ] Is this the simplest solution?
- [ ] Are we reinventing any wheels?
- [ ] Could this be 10x simpler?

## 📚 REQUIRED READING

### Essential Alloy Resources:
- [Alloy Book](https://alloy.sh/book/) - READ THIS FIRST
- [Alloy Examples](https://github.com/alloy-rs/examples)
- [Alloy API Documentation](https://docs.rs/alloy/)

### Key Sections to Study:
1. **Signers**: How to create and manage wallets
2. **Providers**: How to connect to Ethereum networks
3. **Transactions**: How to build and send transactions
4. **Contracts**: How to interact with smart contracts

## ⚠️ WARNING SIGNS

### Red Flags That Indicate Over-Engineering:
- Creating custom `WalletConfig` structs
- Implementing custom encryption/decryption
- Building custom RPC clients
- Writing async storage layers for simple data
- Creating multiple abstraction layers
- Reinventing transaction types

### When You See These - STOP and Simplify:
- `async fn save_wallet_config()`
- `struct CustomWallet`
- `impl Encryption for`
- `custom_crypto_function()`
- Multiple trait abstractions for simple operations

## 🎯 SUCCESS METRICS

### Good Code Indicators:
- ✅ Uses Alloy extensively
- ✅ Under 10 lines for common operations
- ✅ Directly follows Alloy examples
- ✅ No custom crypto implementations
- ✅ Easy to understand and maintain

### Bad Code Indicators:
- ❌ Custom wallet implementations
- ❌ Complex async storage systems
- ❌ Multiple security layers
- ❌ Reinvented Ethereum functionality
- ❌ Hundreds of lines for simple operations

## 📝 ENFORCEMENT

### For Human Developers:
- All PRs must be reviewed against these rules
- Any custom implementation must justify why Alloy can't be used
- Complex solutions require architectural review

### For AI Agents:
- **MUST** reference this file before any wallet/crypto work
- **MUST** check Alloy documentation first
- **MUST** use simplest Alloy approach
- **MUST** explain why if not using Alloy

## 🔄 EXAMPLES OF GOOD VS BAD

### Wallet Creation
```rust
// ✅ GOOD - 3 lines, uses Alloy
let wallet = PrivateKeySigner::random();
let address = wallet.address();
println!("Wallet created: {}", address);

// ❌ BAD - 40+ lines, custom implementation
let config = WalletConfig::new(name, password)?;
let storage = WalletStorage::new()?;
storage.save_config_async(config).await?;
let metadata = WalletMetadata::create()?;
// ... 35 more lines of complexity
```

### Transaction Sending
```rust
// ✅ GOOD - Direct Alloy usage
let tx = TransactionRequest::default()
    .to(to_address)
    .value(amount);
let result = provider.send_transaction(tx).await?;

// ❌ BAD - Custom transaction system
let custom_tx = CustomTransactionBuilder::new()
    .with_validation_layer()
    .with_security_checks()
    .build_async().await?;
```

## 🏆 GUIDING PRINCIPLE

> **"If Alloy can do it, use Alloy. If Alloy can't do it, are you sure you need it?"**

This codebase should be a **showcase of how to use Alloy properly**, not a demonstration of custom Ethereum implementations.

---

**Remember: Battle-tested libraries exist for a reason. Use them.**