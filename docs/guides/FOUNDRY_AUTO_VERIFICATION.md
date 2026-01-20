# Foundry Auto-Verification Integration

## Overview

The Vaughan Token Launcher now supports **automatic contract verification** using Foundry's built-in `--verify` flag. This eliminates the need for manual verification steps on block explorers after deployment.

## How It Works

When you deploy a token through the T4 Token Launcher tab, Vaughan:

1. **Detects available API keys** from environment variables
2. **Uses `forge create --verify`** to deploy and verify in one step
3. **Automatically submits verification** to the appropriate block explorer
4. **Shows real-time status** in the UI

## Setup

### 1. Install Foundry

If you haven't already installed Foundry:

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### 2. Configure API Keys

Set environment variables for the networks you want to use:

```bash
# For Ethereum
export ETHERSCAN_API_KEY=your_etherscan_api_key

# For BSC
export BSCSCAN_API_KEY=your_bscscan_api_key

# For Polygon
export POLYGONSCAN_API_KEY=your_polygonscan_api_key

# For PulseChain
export PULSESCAN_API_KEY=your_pulsescan_api_key
```

You can add these to your `.bashrc`, `.zshrc`, or `.env` file for persistence.

### 3. Verify Setup

Run the test script to check your configuration:

```bash
cd /home/r4/Desktop/Vaughan_V1
./test_auto_verification.sh
```

## UI Changes

The T4 Token Launcher tab has been updated with:

### Cleaner Layout
- Removed redundant header text "Token launcher" and subtitle
- More space for configuration and verification sections
- Status indicator moved to top-right corner

### Smart Verification Section
The right panel now shows:

#### When API Keys are Configured ✅
- "Auto-Verification Enabled" status
- List of networks with configured API keys
- Confirmation that contracts will be verified automatically

#### When API Keys are Missing ⚠️
- "Auto-Verification Not Configured" warning
- Instructions for setting up API keys
- Option to generate manual verification info as fallback

## Technical Implementation

### Key Changes

1. **`deploy_token_with_forge()` method** in `real_token_launcher.rs`:
   - Checks for API keys via `DeploymentEnvConfig::from_env()`
   - Enables `--verify` flag when API key is available
   - Passes API key to forge for auto-verification

2. **Forge Command Structure**:
   ```bash
   forge create src/CustomToken.sol:CustomToken \
     --rpc-url <network_rpc> \
     --private-key $PRIVATE_KEY \
     --broadcast \
     --evm-version <network_specific> \
     --constructor-args "name" "symbol" "supply" "decimals" \
     --verify \
     --etherscan-api-key $API_KEY
   ```

3. **Network-Specific EVM Versions**:
   - Ethereum: `cancun`
   - BSC: `shanghai`
   - Polygon: `shanghai`
   - PulseChain: `shanghai`

## Benefits

### Before (Manual Verification)
1. Deploy contract
2. Generate verification info
3. Copy flattened source code
4. Navigate to block explorer
5. Fill verification form manually
6. Submit and wait
7. Handle potential errors

### After (Auto-Verification)
1. Deploy contract with auto-verification ✅
2. Done! Contract verified automatically

## Fallback Options

If API keys are not configured, Vaughan still supports:

1. **Manual verification info generation** - Click "Generate Manual Verification Info" button
2. **Flattened source code export** - Copy to clipboard for manual submission
3. **Detailed verification parameters** - Compiler version, EVM version, optimization settings

## Troubleshooting

### Verification Failed?

1. **Check API key validity** - Ensure your API key is active and has permissions
2. **Network compatibility** - Some explorers may have different requirements
3. **Wait time** - Verification can take 1-5 minutes to complete
4. **Check explorer directly** - Visit the contract address on the block explorer

### Common Issues

**Issue**: "Auto-verification not working"
- **Solution**: Ensure API key environment variable is set and exported

**Issue**: "Forge command fails"
- **Solution**: Update Foundry to latest version: `foundryup`

**Issue**: "Wrong EVM version"
- **Solution**: The system auto-selects the correct EVM version per network

## Security Notes

⚠️ **Never commit API keys to version control**
- Use environment variables or `.env` files
- Add `.env` to `.gitignore`
- Use different API keys for development and production

## Testing

To test the integration without deploying to mainnet:

1. Use testnet API keys
2. Deploy to testnets (Sepolia, BSC Testnet, etc.)
3. Verify the auto-verification worked on testnet explorers

## Future Improvements

Potential enhancements:
- [ ] Multiple verification services support (Sourcify, etc.)
- [ ] Batch verification for multiple deployments
- [ ] Verification status webhooks
- [ ] Custom verification parameters per template

## Summary

The Foundry auto-verification integration makes token deployment in Vaughan:
- **Faster** - Single-step deployment and verification
- **Simpler** - No manual steps required
- **Reliable** - Uses battle-tested Foundry tooling
- **Flexible** - Fallback to manual verification if needed

Just set your API keys and deploy with confidence! 🚀