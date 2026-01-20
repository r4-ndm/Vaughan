#!/bin/bash

echo "Testing PulseChain Block Explorer API Access"
echo "============================================="
echo ""

# Test address with known transactions on PulseChain
TEST_ADDRESS="0x0000000000000000000000000000000000000000"

echo "1. Testing PulseChain Mainnet API"
echo "================================="

# Test Blockscout v2 API (no API key needed)
echo "Testing v2 API endpoint..."
curl -s -X GET "https://scan.pulsechain.com/api/v2/addresses/${TEST_ADDRESS}/transactions?filter=to%20%7C%20from&type=coin_transfer" \
  -H "Accept: application/json" \
  --max-time 5 \
  | head -c 200

echo ""
echo ""

# Test v1 API (Etherscan-compatible format)
echo "Testing v1 API endpoint (Etherscan-compatible)..."
curl -s "https://scan.pulsechain.com/api/v1/result?module=account&action=txlist&address=${TEST_ADDRESS}&sort=desc&limit=1" \
  --max-time 5 \
  | head -c 200

echo ""
echo ""

echo "2. Testing PulseChain Testnet API"
echo "=================================="

# Test Testnet v2 API
echo "Testing Testnet v2 API endpoint..."
curl -s -X GET "https://scan.v4.testnet.pulsechain.com/api/v2/addresses/${TEST_ADDRESS}/transactions" \
  -H "Accept: application/json" \
  --max-time 5 \
  | head -c 200

echo ""
echo ""

# Also test the alternative endpoint format
echo "Testing alternative API format..."
curl -s "https://api.scan.pulsechain.com/api?module=account&action=txlist&address=${TEST_ADDRESS}&page=1&offset=1" \
  --max-time 5 \
  | head -c 200

echo ""
echo ""

echo "============================================="
echo "Notes:"
echo "- PulseChain uses Blockscout, which provides free public API access"
echo "- No API keys are required for basic transaction queries"
echo "- The v2 API format is recommended for better performance"
echo "- Rate limits may apply (typically 10-100 requests per second)"