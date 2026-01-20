# Custom Networks Guide for Vaughan Wallet

## Overview
The Vaughan wallet supports adding custom EVM-compatible networks, allowing you to connect to any Ethereum-based blockchain including private networks, testnets, and alternative chains.

## How to Add a Custom Network

### 1. Access the Network Dialog
- In the main wallet interface, look for the network dropdown
- Click the **"+"** button next to the network selector
- The "Add Custom Network" dialog will appear

### 2. Required Information
You'll need the following information about your network:

| Field | Description | Example |
|-------|-------------|---------|
| **Network Name** | A friendly name for the network | "My Local Network" |
| **RPC URL** | The JSON-RPC endpoint URL | "https://rpc.mynetwork.com" |
| **Chain ID** | The unique chain identifier | 1337 |
| **Currency Symbol** | Native token symbol | "ETH" |
| **Block Explorer URL** | (Optional) Explorer website | "https://explorer.mynetwork.com" |

### 3. Fill in the Form
- Enter all required fields (marked with *)
- The wallet will validate your inputs in real-time
- Red text indicates validation errors
- Yellow warnings for non-HTTPS connections

### 4. Validation Process
When you click "Add Network", the wallet will:
1. **Validate form inputs** - Check for valid URLs and chain ID
2. **Test connectivity** - Connect to the RPC endpoint
3. **Verify chain ID** - Ensure the RPC reports the expected chain ID
4. **Save configuration** - Store the network for future use

## Common Issues and Solutions

### Issue: "Chain ID mismatch"
**Problem**: The RPC endpoint reports a different chain ID than what you entered.

**Solution**: 
- Verify the correct chain ID with your network provider
- Common chain IDs:
  - Ethereum Mainnet: 1
  - Polygon: 137
  - BSC: 56
  - Local development: 1337

### Issue: "RPC endpoint is not responding"
**Problem**: The wallet cannot connect to the RPC URL.

**Solutions**:
- Check if the URL is correct and accessible
- Ensure your firewall isn't blocking the connection
- Try using curl to test: `curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' YOUR_RPC_URL`

### Issue: "Invalid RPC URL format"
**Problem**: The URL format is incorrect.

**Solution**: 
- URLs must start with `http://` or `https://`
- Examples of valid URLs:
  - ✅ `https://rpc.ankr.com/eth`
  - ✅ `http://localhost:8545`
  - ❌ `rpc.ankr.com/eth` (missing protocol)
  - ❌ `localhost:8545` (missing protocol)

### Issue: "Network validation failed"
**Problem**: The RPC endpoint validation failed.

**Possible causes**:
- Network latency too high (>5 seconds)
- RPC returns invalid responses
- Authentication required but not provided

## Popular Custom Networks

### Local Development
```
Name: Local Ganache
RPC URL: http://127.0.0.1:8545
Chain ID: 1337
Symbol: ETH
```

### Arbitrum One
```
Name: Arbitrum One
RPC URL: https://arb1.arbitrum.io/rpc
Chain ID: 42161
Symbol: ETH
Explorer: https://arbiscan.io
```

### Optimism
```
Name: Optimism
RPC URL: https://mainnet.optimism.io
Chain ID: 10
Symbol: ETH
Explorer: https://optimistic.etherscan.io
```

### Avalanche C-Chain
```
Name: Avalanche
RPC URL: https://api.avax.network/ext/bc/C/rpc
Chain ID: 43114
Symbol: AVAX
Explorer: https://snowtrace.io
```

### Base
```
Name: Base
RPC URL: https://mainnet.base.org
Chain ID: 8453
Symbol: ETH
Explorer: https://basescan.org
```

## Security Considerations

### ⚠️ Use HTTPS When Possible
- HTTP connections are unencrypted
- Only use HTTP for local development
- The wallet will warn you about insecure connections

### 🔍 Verify RPC Providers
- Use official RPC endpoints from the network's documentation
- Be cautious with third-party RPC providers
- Consider running your own node for maximum security

### 🔐 Private Networks
- For private/corporate networks, ensure proper access controls
- Never share private RPC endpoints publicly
- Use VPN if accessing over public internet

## Advanced Features

### Edit Existing Networks
1. Click the "+" button
2. Select "Edit Existing Network"
3. Choose the network from the dropdown
4. Modify the settings
5. Click "Update Network"

### Delete Custom Networks
1. Open the edit dialog for the network
2. Click "Delete Network"
3. Confirm the deletion

### Network Switching
- Use the dropdown to switch between networks
- The wallet remembers your last selected network
- Balance and transactions update automatically

## Troubleshooting Steps

1. **Test RPC connectivity manually**:
```bash
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  YOUR_RPC_URL
```

2. **Verify chain ID**:
The response should contain the chain ID in hex format:
```json
{"jsonrpc":"2.0","id":1,"result":"0x1"}  // Chain ID 1
{"jsonrpc":"2.0","id":1,"result":"0x89"} // Chain ID 137 (Polygon)
```

3. **Check network status**:
```bash
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' \
  YOUR_RPC_URL
```

## Bug Fixed in Latest Version

### ✅ Fixed: Network Validation Using Wrong Chain ID
**Previous Issue**: The wallet was incorrectly validating new networks against the current network's chain ID instead of the new network's chain ID.

**Status**: FIXED - The validation now correctly uses the chain ID of the network being added.

## Need Help?

If you're still having issues adding a custom network:

1. Check the wallet logs (History → Wallet Logs)
2. Verify your network details with the official documentation
3. Test the RPC endpoint independently
4. Ensure no firewall/proxy is blocking connections

Remember: The wallet validates that the RPC endpoint is working and reports the correct chain ID before adding the network. This prevents configuration errors and ensures reliable operation.