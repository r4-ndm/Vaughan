#!/bin/bash

# PulseChain Testnet v4 Configuration Verification Script
echo "🔍 Verifying PulseChain Testnet v4 Configuration"
echo "================================================"

# Check if the project builds successfully
echo ""
echo "📦 Building project..."
if cargo build --release --quiet; then
    echo "✅ Build successful"
else
    echo "❌ Build failed - fix compilation errors first"
    exit 1
fi

# Check PulseChain Testnet v4 configuration
echo ""
echo "🌐 Checking PulseChain Testnet v4 (Chain ID 943) configuration..."

# Check network module
if grep -q "chain_id: 943" src/network/mod.rs; then
    echo "✅ PulseChain Testnet v4 configured in network module"
else
    echo "❌ PulseChain Testnet v4 missing from network module"
fi

# Check GUI integration
if grep -q "943 => \"PulseChain Testnet v4\"" src/gui/mod.rs; then
    echo "✅ PulseChain Testnet v4 name mapping configured"
else
    echo "❌ PulseChain Testnet v4 name mapping missing"
fi

# Check currency symbol
if grep -q "943 => \"tPLS\"" src/gui/mod.rs; then
    echo "✅ tPLS currency symbol configured"
else
    echo "❌ tPLS currency symbol missing"
fi

# Check DApp platform integration
if grep -q "PulseChain Testnet v4.*943" src/gui/dapp_platform.rs; then
    echo "✅ PulseChain Testnet v4 integrated in DApp platform"
else
    echo "❌ PulseChain Testnet v4 missing from DApp platform"
fi

# Check Token Launcher network support
if grep -q "PulseChain Testnet v4" src/gui/tabs/mod.rs; then
    echo "✅ PulseChain Testnet v4 available in Token Launcher"
else
    echo "❌ PulseChain Testnet v4 missing from Token Launcher"
fi

echo ""
echo "🚀 OpenZeppelin Template Verification..."

# Check available templates
TEMPLATES=(
    "BasicERC20"
    "BurnableERC20"
    "MintableERC20"
    "PausableERC20"
    "FullFeaturedERC20"
)

for template in "${TEMPLATES[@]}"; do
    if grep -q "$template" src/launcher/templates.rs; then
        echo "✅ $template template available"
    else
        echo "❌ $template template missing"
    fi
done

echo ""
echo "🔐 Security & Verification Check..."

# Check OpenZeppelin imports
if grep -q "@openzeppelin/contracts" src/launcher/templates.rs; then
    echo "✅ OpenZeppelin contracts imported"
else
    echo "❌ OpenZeppelin contracts missing"
fi

# Check auto-verification
if grep -q "VerificationManager" src/launcher/verification.rs; then
    echo "✅ Auto-verification system available"
else
    echo "❌ Auto-verification system missing"
fi

echo ""
echo "📋 Test Environment Checklist:"
echo "================================"

echo ""
echo "Before testing, ensure you have:"
echo "[ ] 🪙 Test tPLS tokens from PulseChain Testnet v4 faucet"
echo "[ ] 🌐 Stable internet connection"
echo "[ ] 💰 Sufficient tPLS for gas fees (deployment costs ~0.001-0.01 tPLS)"

echo ""
echo "📝 Recommended Test Parameters:"
echo "- Template: Basic ERC20 (for first test)"
echo "- Name: Test Token"
echo "- Symbol: TEST"
echo "- Decimals: 18"
echo "- Supply: 1000000"
echo "- Network: PulseChain Testnet v4"

echo ""
echo "🔗 PulseChain Testnet v4 Resources:"
echo "- RPC: https://rpc.v4.testnet.pulsechain.com"
echo "- Explorer: https://scan.v4.testnet.pulsechain.com"
echo "- Chain ID: 943"

echo ""
echo "🎯 Ready to Test!"
echo "=================="
echo "Run the following command to start testing:"
echo "cargo run --bin dapp-platform --release"

echo ""
echo "Then navigate to T4: Launcher tab and deploy your token!"
echo ""
echo "📄 Don't forget to document your results in the test guide:"
echo "cat pulsechain_testnet_setup.md"