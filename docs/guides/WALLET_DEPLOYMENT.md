# Wallet-Integrated Token Deployment with Sourcify Verification

## Overview

The Vaughan wallet system now has **fully integrated token deployment** with automatic Sourcify verification. You don't need to manually handle private keys - the wallet manages everything securely!

## How It Works

### 1. **Wallet Integration**
- The `RealTokenLauncher` is integrated with your Vaughan wallet
- Private keys are securely retrieved from the wallet when needed
- No need to expose or manually manage private keys

### 2. **Automatic Verification**
- **Sourcify verification is enabled by default** (no API key required!)
- If you have block explorer API keys set, they will be used instead
- Verification happens automatically during deployment

### 3. **Fixed Working Directory**
- The forge subprocess now runs from the correct project root
- This allows forge to find contract source files during verification
- No more "Failed to get standard json input" errors!

## Using the GUI

### Step 1: Launch the Vaughan GUI
```bash
cargo run --release
```

### Step 2: Connect Your Wallet
1. Go to the **Token Launcher** tab
2. Click the **"Connect"** button
3. Select an account from your wallet
4. The account will be unlocked for deployment

### Step 3: Configure Your Token
Fill in the token details:
- **Token Name**: e.g., "My Custom Token"
- **Symbol**: e.g., "MCT"
- **Total Supply**: e.g., 1000000
- **Decimals**: Usually 18 (default)

### Step 4: Select Network
Choose your target network from the dropdown:
- Ethereum Mainnet (Chain ID 1)
- BSC (Chain ID 56)
- Polygon (Chain ID 137)
- **PulseChain Testnet v4 (Chain ID 943)** ← Recommended for testing!
- PulseChain Mainnet (Chain ID 369)

### Step 5: Deploy!
1. Click **"Deploy Token"**
2. The wallet will:
   - Retrieve your private key securely
   - Compile the contract with forge
   - Deploy to the blockchain
   - **Automatically verify with Sourcify** (no API keys needed!)
3. View your deployed contract on the block explorer

## Sourcify Verification

### What is Sourcify?
- **Decentralized** contract verification service
- Works with **any EVM network**
- **No API keys required**
- Contracts are automatically indexed and verifiable at `sourcify.dev`

### How to Check Verification
After deployment, your contract will be verified at:
```
https://repo.sourcify.dev/contracts/full_match/{chain_id}/{contract_address}
```

For example, on PulseChain Testnet (943):
```
https://repo.sourcify.dev/contracts/full_match/943/0xYourContractAddress
```

### Verification Process
1. Forge compiles your contract with metadata
2. During deployment, forge submits the source code to Sourcify
3. Sourcify verifies the bytecode matches the source
4. Your contract is now publicly verifiable!

## API Key Support (Optional)

If you prefer to use block explorer verification instead of Sourcify, you can set these environment variables:

### For Ethereum/Sepolia:
```bash
export ETHERSCAN_API_KEY="your_etherscan_api_key"
```

### For BSC:
```bash
export BSCSCAN_API_KEY="your_bscscan_api_key"
```

### For Polygon:
```bash
export POLYGONSCAN_API_KEY="your_polygonscan_api_key"
```

### For Multi-Chain (Moralis):
```bash
export MORALIS_API_KEY="your_moralis_api_key"
```

**Note**: If no API keys are set, Sourcify will be used automatically (which is often better anyway!).

## Technical Details

### How Private Keys are Retrieved
From `src/launcher/real_token_launcher.rs` (line 309):
```rust
// Retrieve private key from wallet for forge deployment
let private_key_secret = wallet_guard.get_private_key_for_deployment().await
    .map_err(|e| crate::error::VaughanError::Wallet(crate::error::WalletError::WalletError {
        message: format!("Failed to retrieve private key from wallet: {}", e),
    }))?;
```

The private key is:
1. Securely retrieved from the wallet keystore
2. Used only during the deployment transaction
3. Never logged or exposed to the user
4. Immediately dropped after use

### Forge Working Directory Fix
From `src/launcher/forge_deployment.rs` (line 170):
```rust
// Set the working directory to the project root
// This ensures forge can find contract source files during verification
cmd.current_dir("/home/r4/Desktop/Vaughan_V1");
```

This fix ensures that forge can:
- Find `src/CustomToken.sol`
- Read `foundry.toml` configuration
- Generate proper source metadata for verification
- Submit to Sourcify successfully

### Verification Configuration
From `src/launcher/real_token_launcher.rs` (lines 335-337):
```rust
// Enable verification - use Sourcify if no API key is available
let should_verify = true;
let use_sourcify = api_key.is_none(); // Use Sourcify when no API key
```

Verification is always enabled, and Sourcify is used as the default fallback.

## Troubleshooting

### "No wallet account selected"
**Solution**: Click the "Connect" button in the Token Launcher tab and select an account.

### "Failed to retrieve private key from wallet"
**Solution**: Make sure your wallet is unlocked and you've selected an account.

### "Forge deployment failed"
**Solution**: 
- Check that you have sufficient balance on the target network
- Verify the RPC endpoint is accessible
- Make sure `forge` is installed: `forge --version`

### Verification Failed
**Solution**: 
- Sourcify verification is automatic and doesn't require configuration
- If it fails, the contract is still deployed - you can verify manually later
- Check the Sourcify status at their website

## Example Networks

### PulseChain Testnet v4 (Recommended for Testing)
- **Chain ID**: 943
- **RPC**: https://rpc.v4.testnet.pulsechain.com
- **Explorer**: https://scan.v4.testnet.pulsechain.com
- **Faucet**: https://faucet.v4.testnet.pulsechain.com
- **Sourcify**: Automatic (no config needed)

### PulseChain Mainnet
- **Chain ID**: 369
- **RPC**: https://rpc.pulsechain.com
- **Explorer**: https://scan.pulsechain.com
- **Sourcify**: Automatic (no config needed)

## Summary

✅ **Wallet manages private keys** - No manual key handling  
✅ **Sourcify verification enabled** - No API keys required  
✅ **Working directory fixed** - Forge can find contract files  
✅ **Multi-network support** - Deploy to any EVM chain  
✅ **GUI integrated** - Easy point-and-click deployment  

Your token deployment is now secure, verified, and fully automated! 🚀
