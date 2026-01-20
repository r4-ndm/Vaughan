#!/bin/bash
# Extract verification information for contract verification

echo "📋 Extracting Contract Verification Information..."
echo "=================================================="
echo ""

# Get compiler version
echo "🔧 Compiler Information:"
forge --version | grep "solc"
echo ""

# Get EVM version based on network (you can modify this)
echo "🌐 EVM Version: shanghai (for PulseChain)"
echo "   Use 'cancun' for Ethereum mainnet"
echo ""

# Optimization settings
echo "⚙️  Optimization Settings:"
echo "   Enabled: Yes"
echo "   Runs: 200"
echo ""

# Generate flattened source
echo "📄 Generating Flattened Source Code..."
echo "=================================================="
echo ""
forge flatten src/CustomToken.sol

echo ""
echo "=================================================="
echo "✅ Verification info generated!"
echo ""
echo "📋 To save to a file:"
echo "   ./get_verification_info.sh > verification_info.txt"
echo ""
echo "💡 Copy the information above to verify your contract"
echo "   on the block explorer (e.g., scan.v4.testnet.pulsechain.com)"
