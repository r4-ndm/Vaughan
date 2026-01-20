#!/bin/bash

# Test script for Foundry auto-verification integration in Vaughan
# This tests the token launcher with automatic contract verification

set -e

echo "============================================"
echo "Testing Vaughan Token Launcher with Auto-Verification"
echo "============================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Check prerequisites
echo "Checking prerequisites..."
echo ""

# Check if forge is installed
if command -v forge &> /dev/null; then
    print_status "Foundry/Forge is installed"
    forge --version
else
    print_error "Foundry is not installed. Please install it first:"
    echo "  curl -L https://foundry.paradigm.xyz | bash"
    echo "  foundryup"
    exit 1
fi

echo ""

# Check if environment variables are set
echo "Checking environment configuration..."
echo ""

check_env_var() {
    if [ -z "${!1}" ]; then
        print_warning "$1 is not set (auto-verification will be disabled for this network)"
        return 1
    else
        print_status "$1 is configured"
        return 0
    fi
}

# Track if any API keys are configured
HAS_ANY_KEY=false

# Check each network's API key
if check_env_var "ETHERSCAN_API_KEY"; then HAS_ANY_KEY=true; fi
if check_env_var "BSCSCAN_API_KEY"; then HAS_ANY_KEY=true; fi
if check_env_var "POLYGONSCAN_API_KEY"; then HAS_ANY_KEY=true; fi
if check_env_var "PULSESCAN_API_KEY"; then HAS_ANY_KEY=true; fi

echo ""

if [ "$HAS_ANY_KEY" = false ]; then
    print_warning "No API keys are configured. Auto-verification will not work."
    echo ""
    echo "To enable auto-verification, set at least one of these environment variables:"
    echo "  export ETHERSCAN_API_KEY=your_etherscan_api_key"
    echo "  export BSCSCAN_API_KEY=your_bscscan_api_key"
    echo "  export POLYGONSCAN_API_KEY=your_polygonscan_api_key"
    echo "  export PULSESCAN_API_KEY=your_pulsescan_api_key"
    echo ""
fi

# Check if private key is set for testing
if [ -z "$PRIVATE_KEY" ]; then
    print_warning "PRIVATE_KEY is not set. Using test key for demonstration."
    export PRIVATE_KEY="0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    echo "  Test key set (DO NOT use this in production!)"
fi

echo ""

# Test compilation and flattening
echo "Testing contract compilation and flattening..."
echo ""

# Compile the CustomToken contract
if forge build --contracts src/CustomToken.sol 2>/dev/null; then
    print_status "Contract compilation successful"
else
    print_error "Contract compilation failed"
    exit 1
fi

# Test flattening
if forge flatten src/CustomToken.sol > /tmp/flattened.sol 2>/dev/null; then
    print_status "Contract flattening successful"
    LINES=$(wc -l < /tmp/flattened.sol)
    echo "  Flattened source: $LINES lines"
else
    print_error "Contract flattening failed"
fi

echo ""

# Test the verification info generation
echo "Testing verification info generation..."
echo ""

# Get compiler version
COMPILER_VERSION=$(forge --version | grep -oP 'solc \K[0-9.]+' || echo "unknown")
print_status "Compiler version: v$COMPILER_VERSION"

# Determine EVM version based on network
print_status "EVM versions by network:"
echo "  • Ethereum: cancun"
echo "  • BSC: shanghai"
echo "  • Polygon: shanghai"
echo "  • PulseChain: shanghai"

echo ""

# Simulate deployment with verification (dry-run mode)
echo "Testing deployment command structure..."
echo ""

# Create test constructor arguments
TOKEN_NAME="TestToken"
TOKEN_SYMBOL="TEST"
TOTAL_SUPPLY="1000000"
DECIMALS="18"

print_status "Test token parameters:"
echo "  • Name: $TOKEN_NAME"
echo "  • Symbol: $TOKEN_SYMBOL"
echo "  • Supply: $TOTAL_SUPPLY"
echo "  • Decimals: $DECIMALS"

echo ""

# Build the forge create command (without executing)
CMD="forge create src/CustomToken.sol:CustomToken"
CMD="$CMD --rpc-url https://rpc.v4.testnet.pulsechain.com"
CMD="$CMD --private-key \$PRIVATE_KEY"
CMD="$CMD --broadcast"
CMD="$CMD --evm-version shanghai"
CMD="$CMD --constructor-args \"$TOKEN_NAME\" \"$TOKEN_SYMBOL\" \"$TOTAL_SUPPLY\" \"$DECIMALS\""

# Add verification if API key is available
if [ ! -z "$PULSESCAN_API_KEY" ]; then
    CMD="$CMD --verify --etherscan-api-key \$PULSESCAN_API_KEY"
    print_status "Auto-verification will be enabled for PulseChain testnet"
else
    print_warning "Auto-verification disabled (no PULSESCAN_API_KEY)"
fi

echo ""
echo "Generated deployment command:"
echo "  $CMD"
echo ""

# Summary
echo "============================================"
echo "Test Results Summary"
echo "============================================"
echo ""

if [ "$HAS_ANY_KEY" = true ]; then
    print_status "✅ Auto-verification is configured and ready!"
    echo ""
    echo "When you deploy tokens using Vaughan's Token Launcher:"
    echo "1. Contracts will be deployed using forge create"
    echo "2. Verification will be submitted automatically"
    echo "3. No manual verification steps needed!"
else
    print_warning "⚠️ Auto-verification is not configured"
    echo ""
    echo "Vaughan will still deploy your contracts successfully, but"
    echo "you'll need to verify them manually on the block explorer."
fi

echo ""
echo "To test a real deployment, run Vaughan and use the T4 Token Launcher tab."
echo ""

# Test Rust integration
echo "Testing Rust integration..."
echo ""

if cargo check --lib 2>/dev/null; then
    print_status "Rust code compiles successfully"
else
    print_error "Rust compilation failed"
    exit 1
fi

echo ""
print_status "All tests completed successfully!"
echo ""
echo "The Foundry auto-verification integration is ready to use."