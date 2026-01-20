# PulseChain API Integration

## Overview
PulseChain uses **Blockscout** as its block explorer, which provides **FREE public API access** without requiring any API keys or registration.

## API Endpoints

### PulseChain Mainnet
- **Explorer URL**: https://scan.pulsechain.com
- **API Base**: https://scan.pulsechain.com/api
- **Documentation**: https://scan.pulsechain.com/api-docs

### PulseChain Testnet v4
- **Explorer URL**: https://scan.v4.testnet.pulsechain.com
- **API Base**: https://scan.v4.testnet.pulsechain.com/api
- **Documentation**: https://scan.v4.testnet.pulsechain.com/api-docs

## API Versions Supported

### Blockscout v2 API (Recommended)
Modern REST API with better performance and features:
```
GET /api/v2/addresses/{address}/transactions
```

### Blockscout v1 API (Etherscan-compatible)
Legacy format for compatibility:
```
GET /api/v1/result?module=account&action=txlist&address={address}
```

## Key Advantages

✅ **No API Key Required** - Completely free to use
✅ **No Registration Needed** - Just make HTTP requests
✅ **Generous Rate Limits** - Suitable for wallet applications
✅ **Full Transaction Data** - All transaction details available
✅ **Both Mainnet & Testnet** - Same API structure for both

## Implementation in Vaughan Wallet

The wallet automatically:
1. Tries Blockscout v2 API first (better performance)
2. Falls back to v1 API if needed
3. Handles both mainnet and testnet
4. Requires no configuration from users

## Example Usage

### Fetch transactions for an address:
```bash
curl -X GET "https://scan.pulsechain.com/api/v2/addresses/0xYOUR_ADDRESS/transactions" \
  -H "Accept: application/json"
```

### Response includes:
- Transaction hash
- From/To addresses
- Value transferred
- Timestamp
- Status (confirmed/pending/failed)
- Gas information

## Rate Limits

While there are no strict API key requirements, reasonable rate limits apply:
- Typical limit: 10-100 requests per second
- The wallet's usage pattern (occasional fetches) is well within limits

## Comparison with Other Networks

| Network | API Provider | Requires API Key | Free Tier |
|---------|-------------|------------------|-----------|
| **PulseChain** | Blockscout | **No** ✅ | **Unlimited** |
| Ethereum | Etherscan | Yes (limited without) | 100K calls/day |
| BSC | BSCScan | Yes (limited without) | 100K calls/day |
| Polygon | Polygonscan | Yes (limited without) | 100K calls/day |

## Testing

Run the test script to verify API access:
```bash
./test_pulsechain_api.sh
```

## Conclusion

PulseChain's Blockscout-based API is the most accessible among all supported networks in the Vaughan wallet, providing completely free and open access to blockchain data without any registration or API key requirements.