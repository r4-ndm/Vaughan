# ETHW (EthereumPoW) Network Configuration Guide

## Problem
The ETHW RPC endpoint `https://mainnet.ethereumpow.org` may be experiencing connectivity issues or not responding to JSON-RPC requests properly.

## Alternative ETHW RPC Endpoints

Try these alternative endpoints if the main one fails:

### Option 1: Official Alternative
```
Network Name: ETHW-mainnet
RPC URL: https://mainnet.ethereumpow.net
Chain ID: 10001
Currency Symbol: ETHW
Block Explorer: https://www.oklink.com/ethw
```

### Option 2: Community Endpoints
```
Network Name: ETHW-mainnet
RPC URL: https://ethw-mainnet.nodereal.io/v1/YOUR_API_KEY
Chain ID: 10001
Currency Symbol: ETHW
Block Explorer: https://www.oklink.com/ethw
```
Note: NodeReal requires registration for an API key

### Option 3: Public Endpoints
```
Network Name: ETHW-mainnet
RPC URL: https://ethw.public-rpc.com
Chain ID: 10001
Currency Symbol: ETHW
Block Explorer: https://www.oklink.com/ethw
```

## Troubleshooting Steps

### 1. Test the RPC Endpoint First
Before adding to the wallet, test the endpoint:
```bash
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  YOUR_RPC_URL
```

Expected response:
```json
{"jsonrpc":"2.0","id":1,"result":"0x2711"}
```
(0x2711 = 10001 in decimal)

### 2. Common Issues and Solutions

#### Issue: "RPC endpoint is not responding"
**Solutions:**
- The endpoint may be down or rate-limited
- Try an alternative endpoint from the list above
- Check if you need to add a port number (e.g., `:8545`)
- Some endpoints require API keys

#### Issue: "Chain ID mismatch"
**Solutions:**
- Ensure you're using Chain ID: 10001
- The RPC must report this exact chain ID
- Some endpoints may be misconfigured

#### Issue: "Network validation failed"
**Possible causes:**
- Firewall blocking the connection
- DNS resolution issues
- SSL certificate problems with the endpoint

### 3. Working ETHW Configuration (Verified)

Based on community reports, these configurations have worked:

**Using Ankr (Free Tier Available):**
```
RPC URL: https://rpc.ankr.com/eth_pow
Chain ID: 10001
```

**Using Local Node:**
If you're running your own ETHW node:
```
RPC URL: http://localhost:8545
Chain ID: 10001
```

## Why the Main Endpoint Might Fail

1. **Rate Limiting**: Public endpoints often have strict rate limits
2. **Geographic Restrictions**: Some endpoints may be blocked in certain regions
3. **Maintenance**: The endpoint might be temporarily down
4. **Protocol Changes**: ETHW might have updated their RPC requirements

## Recommended Approach

1. **Test Multiple Endpoints**: Try each endpoint listed above
2. **Use a Provider Service**: Consider using a service like:
   - Ankr (https://www.ankr.com/)
   - NodeReal (https://nodereal.io/)
   - QuickNode (https://www.quicknode.com/)
   
3. **Run Your Own Node**: For best reliability, consider running an ETHW node

## Adding to Vaughan Wallet

Once you find a working endpoint:

1. Click the "+" button next to network selector
2. Enter the network details:
   - Name: ETHW-mainnet
   - RPC URL: [Your working endpoint]
   - Chain ID: 10001
   - Symbol: ETHW
   - Explorer: https://www.oklink.com/ethw
3. Click "Add Network"

## Verification

After adding, verify the network works by:
1. Switching to the ETHW network
2. Checking your balance loads
3. Testing a small transaction

## Additional Resources

- ETHW Official Site: https://ethereumpow.org/
- ETHW Documentation: https://ethereumpow.org/docs
- OKLink ETHW Explorer: https://www.oklink.com/ethw
- ETHW Community: Check Discord/Telegram for latest RPC endpoints

## Note on Network Stability

ETHW, being a fork of Ethereum, may have less infrastructure support than major networks. It's normal to experience occasional connectivity issues with public endpoints. For production use, consider using a paid RPC service or running your own node.