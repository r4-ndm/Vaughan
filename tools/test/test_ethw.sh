#!/bin/bash

echo "Testing ETHW (EthereumPoW) Network Connectivity"
echo "================================================"
echo ""

ETHW_RPC="https://mainnet.ethereumpow.org"
EXPECTED_CHAIN_ID=10001

echo "1. Testing RPC endpoint: $ETHW_RPC"
echo "   Expected Chain ID: $EXPECTED_CHAIN_ID"
echo ""

# Test 1: Basic connectivity
echo "Test 1: Basic HTTP connectivity..."
if curl -s --max-time 5 -o /dev/null -w "%{http_code}" "$ETHW_RPC" | grep -q "405\|200\|301\|302"; then
    echo "✅ Endpoint is reachable"
else
    echo "❌ Endpoint is not reachable"
fi
echo ""

# Test 2: eth_chainId method
echo "Test 2: Testing eth_chainId method..."
RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    "$ETHW_RPC" --max-time 5 2>/dev/null)

if [ ! -z "$RESPONSE" ]; then
    echo "Response: $RESPONSE"
    CHAIN_ID_HEX=$(echo $RESPONSE | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    if [ ! -z "$CHAIN_ID_HEX" ]; then
        CHAIN_ID_DEC=$(printf "%d" "$CHAIN_ID_HEX" 2>/dev/null)
        echo "Chain ID (hex): $CHAIN_ID_HEX"
        echo "Chain ID (decimal): $CHAIN_ID_DEC"
        
        if [ "$CHAIN_ID_DEC" = "$EXPECTED_CHAIN_ID" ]; then
            echo "✅ Chain ID matches expected value ($EXPECTED_CHAIN_ID)"
        else
            echo "❌ Chain ID mismatch! Expected: $EXPECTED_CHAIN_ID, Got: $CHAIN_ID_DEC"
        fi
    else
        echo "❌ Could not extract chain ID from response"
    fi
else
    echo "❌ No response from RPC endpoint"
fi
echo ""

# Test 3: net_version method
echo "Test 3: Testing net_version method..."
RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' \
    "$ETHW_RPC" --max-time 5 2>/dev/null)

if [ ! -z "$RESPONSE" ]; then
    echo "Response: $RESPONSE"
    NET_VERSION=$(echo $RESPONSE | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    if [ ! -z "$NET_VERSION" ]; then
        echo "Network version: $NET_VERSION"
    fi
else
    echo "❌ No response from net_version"
fi
echo ""

# Test 4: Alternative ETHW endpoints
echo "Test 4: Testing alternative ETHW endpoints..."
ALTERNATIVE_RPCS=(
    "https://mainnet.ethereumpow.net"
    "https://rpc.ethw.com"
    "https://mainnet.ethereumpow.org:8545"
)

for RPC in "${ALTERNATIVE_RPCS[@]}"; do
    echo "Testing: $RPC"
    if curl -s --max-time 3 -o /dev/null -w "%{http_code}" "$RPC" | grep -q "405\|200\|301\|302"; then
        echo "  ✅ Reachable"
        
        # Try to get chain ID
        RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            "$RPC" --max-time 3 2>/dev/null)
        
        if [ ! -z "$RESPONSE" ]; then
            echo "  Response: $(echo $RESPONSE | head -c 100)..."
        fi
    else
        echo "  ❌ Not reachable"
    fi
    echo ""
done

echo "================================================"
echo "Summary:"
echo "If the main ETHW RPC is not working, try one of the alternative endpoints."
echo "The wallet requires the RPC to respond with the correct chain ID (10001)."