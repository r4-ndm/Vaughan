#!/bin/bash

echo "🔍 Vaughan Token Deployment Debug Script"
echo "========================================"
echo ""

echo "📋 Step 1: Environment Check"
echo "----------------------------"

# Check if .env exists
if [ -f .env ]; then
    echo "✅ .env file exists"
else 
    echo "❌ .env file missing"
    echo "   Creating from template..."
    cp .env.example .env
    echo "✅ Created .env from template"
fi

# Check private key
if [ -n "$PRIVATE_KEY" ]; then
    echo "✅ PRIVATE_KEY environment variable is set (length: ${#PRIVATE_KEY})"
    if [ ${#PRIVATE_KEY} -eq 64 ]; then
        echo "✅ Private key length is correct (64 characters)"
    elif [ ${#PRIVATE_KEY} -eq 66 ]; then
        echo "⚠️  Private key has 0x prefix (should be removed)"
    else
        echo "❌ Private key length is incorrect (should be 64 hex characters)"
    fi
else
    echo "❌ PRIVATE_KEY environment variable not set"
    echo ""
    echo "🔧 To fix this:"
    echo "   1. Get a test private key from MetaMask or any wallet"
    echo "   2. Export it (64 hex characters, no 0x prefix)"
    echo "   3. Set it with: export PRIVATE_KEY=your_64_char_hex_key"
    echo "   4. Or add it to your .env file"
fi

echo ""
echo "🌐 Step 2: Network Connectivity Test"
echo "-----------------------------------"

# Test PulseChain Testnet RPC
echo "Testing PulseChain Testnet v4 RPC..."
if curl -s -X POST -H "Content-Type: application/json" \
   --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
   https://rpc.v4.testnet.pulsechain.com | grep -q result; then
    echo "✅ PulseChain Testnet v4 RPC is accessible"
else
    echo "❌ PulseChain Testnet v4 RPC connection failed"
fi

# Test Ethereum RPC
echo "Testing Ethereum RPC..."
if curl -s -X POST -H "Content-Type: application/json" \
   --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
   https://ethereum.publicnode.com | grep -q result; then
    echo "✅ Ethereum RPC is accessible"
else
    echo "⚠️  Ethereum RPC connection issues (may affect mainnet deployments)"
fi

echo ""
echo "🏗️ Step 3: Build Status"
echo "----------------------"
if cargo build --release --quiet 2>/dev/null; then
    echo "✅ Project builds successfully"
else
    echo "❌ Build failed - check compilation errors"
fi

echo ""
echo "💰 Step 4: Test Token Funding"
echo "----------------------------"
if [ -n "$PRIVATE_KEY" ]; then
    echo "ℹ️  To deploy on PulseChain Testnet v4, you need test tokens:"
    echo "   🚰 Get tPLS from: https://faucet.v4.testnet.pulsechain.com/"
    echo "   📍 Your address will be derived from your private key"
else
    echo "⚠️  Set PRIVATE_KEY first to check balance"
fi

echo ""
echo "🚀 Step 5: Quick Deployment Test"
echo "-------------------------------"
echo "After setting up your environment:"
echo "1. Set PRIVATE_KEY environment variable"
echo "2. Get test tokens from PulseChain faucet"
echo "3. Run: cargo run --bin dapp-platform --release"
echo "4. Go to 'T4: Token Launcher' tab"
echo "5. Select 'PulseChain Testnet v4' network"
echo "6. Fill in token details and deploy!"

echo ""
echo "🔧 Common Issues & Solutions:"
echo "============================="
echo ""
echo "❌ 'Contract call failed' usually means:"
echo "   • No private key configured"
echo "   • Insufficient balance for gas fees"
echo "   • Network connectivity issues"
echo "   • Wrong network selected"
echo ""
echo "❌ 'No provider for network' means:"
echo "   • RPC endpoint is down"
echo "   • Network ID mismatch"
echo "   • Connection timeout"
echo ""
echo "❌ 'Invalid private key' means:"
echo "   • Wrong format (needs 64 hex chars, no 0x)"
echo "   • Missing PRIVATE_KEY environment variable"
echo ""
echo "✅ For successful deployment:"
echo "   • Use PulseChain Testnet v4 (network ID: 943)"
echo "   • Get test tokens first"
echo "   • Start with basic ERC20 template"
echo "   • Set reasonable gas limit (2M+)"

echo ""
echo "🎯 Ready to deploy? Run:"
echo "   export PRIVATE_KEY=your_key_here"
echo "   cargo run --bin dapp-platform --release"