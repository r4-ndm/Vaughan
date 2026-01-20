# Custom Token Deployment with Constructor Arguments

## Overview

This guide explains how to deploy ERC20 tokens with custom parameters (name, symbol, supply, decimals) using the `CustomToken.sol` contract and Foundry's `forge create` command.

## What Changed?

### Previous System
- Used hardcoded token parameters in `SimpleToken.sol`
- All deployments created tokens with the same name, symbol, and supply
- No flexibility for custom token creation

### New System
- **Parameterized contract** (`CustomToken.sol`) accepts constructor arguments
- **Dynamic deployment** - specify token details at deployment time
- **Full customization** - name, symbol, supply, and decimals

---

## Contract: CustomToken.sol

### Constructor Signature

```solidity path=/home/r4/Desktop/Vaughan_V1/src/CustomToken.sol start=16
constructor(
    string memory name_,
    string memory symbol_,
    uint256 initialSupply_,
    uint8 decimals_
) ERC20(name_, symbol_) {
    _customDecimals = decimals_;
    _mint(msg.sender, initialSupply_ * 10**decimals_);
}
```

### Parameters

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `name_` | `string` | Full name of the token | `"My Custom Token"` |
| `symbol_` | `string` | Token ticker symbol | `"MCT"` |
| `initialSupply_` | `uint256` | Initial supply (multiplied by 10^decimals) | `5000000` (5 million) |
| `decimals_` | `uint8` | Number of decimal places | `18` (standard) |

### Key Features

✅ **Fully customizable** - All parameters set at deployment  
✅ **OpenZeppelin-based** - Uses battle-tested ERC20 implementation  
✅ **Standard-compliant** - Follows ERC20 standard  
✅ **Mints to deployer** - All tokens sent to `msg.sender`  

---

## Deployment Methods

### Method 1: Using Forge CLI Directly

The simplest way to deploy with constructor arguments:

```bash
# Set your private key
export PRIVATE_KEY="0x..."

# Deploy with constructor arguments
forge create src/CustomToken.sol:CustomToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy \
  --constructor-args "My Token" "MTK" 1000000 18
```

**Arguments must be in order:**
1. Token name (string)
2. Token symbol (string)
3. Initial supply (uint256)
4. Decimals (uint8)

### Method 2: Using Rust `forge_deployment` Module

For programmatic deployment from your Rust application:

```rust
use vaughan_v1::launcher::forge_deployment::{deploy_with_forge, ForgeDeployConfig};

let config = ForgeDeployConfig {
    contract_path: "src/CustomToken.sol:CustomToken".to_string(),
    rpc_url: "https://rpc.v4.testnet.pulsechain.com".to_string(),
    chain_id: 943,
    private_key: deployer_key,
    gas_limit: None, // Auto-estimate
    gas_price: None, // Use network default
    legacy: true,    // Required for PulseChain
    verify: false,   // Enable for Etherscan verification
    etherscan_api_key: None,
    
    // Constructor arguments (ORDER MATTERS!)
    constructor_args: vec![
        "My Custom Token".to_string(),  // name
        "MCT".to_string(),              // symbol
        "5000000".to_string(),          // supply
        "18".to_string(),               // decimals
    ],
};

let result = deploy_with_forge(config).await?;
println!("Contract deployed at: {}", result.deployed_to);
```

See `examples/deploy_custom_token.rs` for a complete working example.

---

## Integration with UI

To integrate constructor arguments into your token launcher UI, you'll need to:

### 1. Update the Deployment Function

Modify the deployment call to pass constructor arguments from the UI form:

```rust
// In real_token_launcher.rs or similar
let constructor_args = vec![
    config.name.clone(),
    config.symbol.clone(),
    config.total_supply.to_string(),
    config.decimals.to_string(),
];

let forge_config = ForgeDeployConfig {
    contract_path: "src/CustomToken.sol:CustomToken".to_string(),
    // ... other config fields ...
    constructor_args,
};

let result = deploy_with_forge(forge_config).await?;
```

### 2. Update Token Launcher State

The UI already collects these parameters in `TokenLauncherState`:

```rust path=/home/r4/Desktop/Vaughan_V1/src/gui/dapp_platform.rs start=30
pub struct TokenLauncherState {
    pub token_name: String,      // ✅ Already exists
    pub token_symbol: String,     // ✅ Already exists
    pub total_supply: String,     // ✅ Already exists
    pub decimals: String,         // ✅ Already exists
    // ...
}
```

These values just need to be passed through to the deployment function.

### 3. Switch from SimpleToken to CustomToken

Update the contract path in deployment calls:

```diff
- contract_path: "src/SimpleToken.sol:SimpleToken"
+ contract_path: "src/CustomToken.sol:CustomToken"
```

---

## Testing Your Deployment

### 1. Compile the Contract

```bash
forge build --root /home/r4/Desktop/Vaughan_V1
```

Expected output:
```
Compiler run successful with warnings
```

### 2. Test Deployment (Testnet)

Use a testnet with test funds:

```bash
# PulseChain Testnet v4
PRIVATE_KEY="0x..." forge create src/CustomToken.sol:CustomToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy \
  --constructor-args "Test Token" "TST" 1000000 18
```

### 3. Verify on Block Explorer

After deployment, verify the contract:

- **PulseChain Testnet v4**: https://scan.v4.testnet.pulsechain.com
- **Ethereum**: https://etherscan.io
- **BSC**: https://bscscan.com

You should see:
- ✅ Contract address
- ✅ Transaction hash
- ✅ Token name, symbol, supply in contract details

### 4. Add to Wallet

Import the token to MetaMask or your wallet:
1. Copy the deployed contract address
2. Go to wallet → Import Token → Custom Token
3. Paste the address
4. Verify name, symbol, decimals auto-populate

---

## Common Issues & Solutions

### ❌ "intrinsic gas too low"
**Cause**: Gas limit too low for deployment  
**Solution**: Remove `gas_limit` to let forge estimate, or use `--gas-limit 500000`

### ❌ "insufficient funds"
**Cause**: Not enough native tokens for gas  
**Solution**: Get test tokens from faucet or switch to funded account

### ❌ "replacement transaction underpriced"
**Cause**: Pending transaction with same nonce  
**Solution**: Wait for pending tx to complete or increase gas price

### ❌ "invalid constructor arguments"
**Cause**: Wrong number or type of arguments  
**Solution**: Check constructor signature and ensure args match exactly

### ❌ "contract creation code storage out of gas"
**Cause**: Contract too large or gas limit too low  
**Solution**: Increase gas limit or optimize contract

---

## Constructor Argument Encoding Reference

Forge automatically encodes constructor arguments based on their Solidity types:

| Solidity Type | Rust/CLI Value | Example |
|---------------|----------------|---------|
| `string` | String | `"My Token"` |
| `uint256` | String (decimal) | `"1000000"` |
| `uint8` | String (0-255) | `"18"` |
| `address` | String (hex) | `"0x123..."` |
| `bool` | `true`/`false` | `true` |

**Important**: All arguments are passed as strings to the CLI, but forge handles type conversion.

---

## Next Steps

1. ✅ **Deploy a test token** using the example code
2. ✅ **Verify** it appears on the block explorer
3. ✅ **Import** to your wallet
4. ✅ **Integrate** into your DApp UI
5. 🚀 **Deploy** to mainnet when ready

---

## Additional Resources

- [Foundry Forge Documentation](https://book.getfoundry.sh/forge/)
- [OpenZeppelin ERC20](https://docs.openzeppelin.com/contracts/4.x/erc20)
- [Solidity ABI Encoding](https://docs.soliditylang.org/en/latest/abi-spec.html)

---

## Summary

You now have:
- ✅ `CustomToken.sol` - Parameterized ERC20 contract
- ✅ Updated `forge_deployment.rs` - Supports constructor args
- ✅ `deploy_custom_token.rs` - Working example
- ✅ This documentation - Complete guide

**Deploy your first custom token:**
```bash
cd /home/r4/Desktop/Vaughan_V1
export DEPLOYER_PRIVATE_KEY="0x..."
cargo run --example deploy_custom_token
```
