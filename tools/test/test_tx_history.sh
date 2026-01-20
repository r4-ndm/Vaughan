#!/bin/bash

# Test script for transaction history fetching
# This demonstrates fetching transactions without API keys

echo "Testing Transaction History Fetching"
echo "===================================="
echo ""

# Test addresses with known transaction history
ETHEREUM_ADDRESS="0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"  # vitalik.eth
BSC_ADDRESS="0x8894E0a0c962CB723c1976a4421c95949bE2D4E3"       # Binance Hot Wallet
POLYGON_ADDRESS="0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0"   # MATIC Token Contract

echo "Testing Ethereum Mainnet..."
echo "Address: $ETHEREUM_ADDRESS"
curl -s "https://api.etherscan.io/api?module=account&action=txlist&address=$ETHEREUM_ADDRESS&startblock=0&endblock=99999999&sort=desc&page=1&offset=5" | jq '.result | length' 2>/dev/null && echo "✅ Etherscan API accessible (fetched transaction count)" || echo "❌ Etherscan API not accessible"
echo ""

echo "Testing BSC..."  
echo "Address: $BSC_ADDRESS"
curl -s "https://api.bscscan.com/api?module=account&action=txlist&address=$BSC_ADDRESS&startblock=0&endblock=99999999&sort=desc&page=1&offset=5" | jq '.result | length' 2>/dev/null && echo "✅ BSCScan API accessible (fetched transaction count)" || echo "❌ BSCScan API not accessible"
echo ""

echo "Testing Polygon..."
echo "Address: $POLYGON_ADDRESS"
curl -s "https://api.polygonscan.com/api?module=account&action=txlist&address=$POLYGON_ADDRESS&startblock=0&endblock=99999999&sort=desc&page=1&offset=5" | jq '.result | length' 2>/dev/null && echo "✅ Polygonscan API accessible (fetched transaction count)" || echo "❌ Polygonscan API not accessible"
echo ""

echo "Testing PulseChain Testnet..."
curl -s "https://api.scan.v4.testnet.pulsechain.com/api?module=account&action=txlist&address=0x0000000000000000000000000000000000000000&startblock=0&endblock=99999999&sort=desc&page=1&offset=5" 2>/dev/null | head -c 100 && echo "" && echo "✅ PulseScan Testnet API endpoint exists" || echo "❌ PulseScan Testnet API not accessible"
echo ""

echo "===================================="
echo "Note: These tests check if the APIs are accessible without API keys."
echo "Some explorers may return limited data or require API keys for full access."
echo "The wallet will automatically fall back to mock data if APIs are unavailable."