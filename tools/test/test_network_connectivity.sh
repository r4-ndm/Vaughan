#!/bin/bash

echo "🌐 Testing Network Connectivity for Vaughan Wallet"
echo "=================================================="

# Test Ethereum mainnet public endpoint
echo "🔍 Testing Ethereum mainnet..."
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://ethereum.publicnode.com | jq '.'

echo ""

# Test PulseChain
echo "🔍 Testing PulseChain..."
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://rpc.pulsechain.com | jq '.'

echo ""

# Test BSC
echo "🔍 Testing Binance Smart Chain..."
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://bsc-dataseed1.binance.org | jq '.'

echo ""

# Test Polygon
echo "🔍 Testing Polygon..."
curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://polygon-rpc.com | jq '.'

echo ""
echo "✅ Network connectivity test complete!"
echo "If you see block numbers above, the RPC endpoints are working."