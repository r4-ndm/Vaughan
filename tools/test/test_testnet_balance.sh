#!/bin/bash

echo "🧪 Testing PulseChain Testnet Balance Loading"
echo "============================================"

echo "📊 Your account balance on PulseChain Testnet v4:"
echo "Address: 0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f"
echo "Expected: 1.697427 tPLS"

# Direct RPC query to confirm balance
echo ""
echo "🔍 Querying balance directly from testnet RPC..."
BALANCE_HEX=$(curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f","latest"],"id":1}' https://rpc.v4.testnet.pulsechain.com | jq -r '.result')

if [ "$BALANCE_HEX" != "null" ] && [ "$BALANCE_HEX" != "" ]; then
    echo "✅ RPC Balance (hex): $BALANCE_HEX"
    BALANCE_DECIMAL=$(python3 -c "print(int('$BALANCE_HEX', 16) / 10**18)")
    echo "✅ RPC Balance (tPLS): $BALANCE_DECIMAL"
else
    echo "❌ Failed to query RPC balance"
fi

echo ""
echo "🚀 Launching Vaughan wallet (10 second test)..."
echo "   -> Check if PulseChain Testnet v4 appears in network dropdown"
echo "   -> Select PulseChain Testnet v4 network"  
echo "   -> Your balance should now display: 1.697427 tPLS"
echo ""

timeout 10 ./target/release/vaughan &
WALLET_PID=$!
sleep 8
kill $WALLET_PID 2>/dev/null
wait $WALLET_PID 2>/dev/null

echo ""
echo "✅ Test completed!"
echo ""
echo "🎯 INSTRUCTIONS FOR MANUAL TESTING:"
echo "1. Run: ./target/release/vaughan"
echo "2. Look for 'PulseChain Testnet v4' in the network dropdown"
echo "3. Select 'PulseChain Testnet v4' as your network"
echo "4. Your balance should update to show: 1.697427 tPLS"