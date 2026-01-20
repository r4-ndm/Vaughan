#!/bin/bash

# Vaughan Wallet - Block Explorer API Setup
# This script helps you configure API keys for block explorer services

set -e

echo "🚀 Vaughan Wallet - Block Explorer API Setup"
echo "============================================="
echo

CONFIG_DIR="$HOME/.config/vaughan"
CONFIG_FILE="$CONFIG_DIR/explorer_apis.json"
TEMPLATE_FILE="./config/explorer_apis.json.template"

# Create config directory if it doesn't exist
mkdir -p "$CONFIG_DIR"

echo "📋 This script will help you set up API keys for block explorers."
echo "   API keys enable transaction history and improve reliability."
echo

# Function to get API key from user
get_api_key() {
    local service=$1
    local url=$2
    local current_key=$3
    
    echo "🔑 $service API Key"
    echo "   Get your free API key from: $url"
    
    if [ -n "$current_key" ] && [ "$current_key" != "YOUR_${service^^}_API_KEY_HERE" ]; then
        echo "   Current key: ${current_key:0:8}..."
        read -p "   Enter new key (or press Enter to keep current): " new_key
        if [ -n "$new_key" ]; then
            echo "$new_key"
        else
            echo "$current_key"
        fi
    else
        read -p "   Enter API key (or press Enter to skip): " new_key
        echo "$new_key"
    fi
}

# Function to read current config
read_current_config() {
    if [ -f "$CONFIG_FILE" ]; then
        cat "$CONFIG_FILE"
    else
        cat "$TEMPLATE_FILE"
    fi
}

# Read current configuration
echo "📖 Reading current configuration..."
CURRENT_CONFIG=$(read_current_config)

# Extract current API keys (simple grep approach)
CURRENT_ETHERSCAN=$(echo "$CURRENT_CONFIG" | grep -o '"etherscan": "[^"]*"' | cut -d'"' -f4 || echo "")
CURRENT_BSCSCAN=$(echo "$CURRENT_CONFIG" | grep -o '"bscscan": "[^"]*"' | cut -d'"' -f4 || echo "")
CURRENT_POLYGONSCAN=$(echo "$CURRENT_CONFIG" | grep -o '"polygonscan": "[^"]*"' | cut -d'"' -f4 || echo "")

echo
echo "🌐 Let's configure your API keys:"
echo
echo "🌟 RECOMMENDATION: Use a multi-chain aggregator for simplicity!"
echo "   • Moralis: ONE key for all major chains (Free: 40k requests/month)"
echo "   • Alchemy: ONE key for Ethereum + L2s (Free: 300M requests/month)"
echo "   • Individual APIs: Better rate limits per chain"
echo
read -p "   Do you want to use a multi-chain aggregator? [Y/n]: " USE_AGGREGATOR
echo

if [[ ${USE_AGGREGATOR:-Y} =~ ^[Yy] ]]; then
    echo "🔑 Multi-Chain Aggregator Setup"
    echo "   Choose your aggregator:"
    echo "   1) Moralis (recommended for wallets)"
    echo "   2) Alchemy (best performance)" 
    echo "   3) Skip aggregator, use individual APIs"
    read -p "   Enter choice [1-3]: " AGG_CHOICE
    
    case $AGG_CHOICE in
        1)
            AGGREGATOR_KEY=$(get_api_key "Moralis" "https://moralis.io" "")
            PREFERRED_AGGREGATOR="moralis"
            ;;
        2)
            AGGREGATOR_KEY=$(get_api_key "Alchemy" "https://alchemy.com" "")
            PREFERRED_AGGREGATOR="alchemy"
            ;;
        *)
            AGGREGATOR_KEY=""
            PREFERRED_AGGREGATOR=""
            ;;
    esac
    echo
else
    AGGREGATOR_KEY=""
    PREFERRED_AGGREGATOR=""
fi

echo "📋 Individual Chain APIs (optional if using aggregator):"
echo

# Get API keys from user
ETHERSCAN_KEY=$(get_api_key "Etherscan" "https://etherscan.io/apis" "$CURRENT_ETHERSCAN")
echo

BSCSCAN_KEY=$(get_api_key "BSCScan" "https://bscscan.com/apis" "$CURRENT_BSCSCAN")
echo

POLYGONSCAN_KEY=$(get_api_key "PolygonScan" "https://polygonscan.com/apis" "$CURRENT_POLYGONSCAN")
echo

# Ask about rate limiting
echo "⚙️  Configuration Options"
echo "   Current rate limit: 5 requests per minute (recommended for free tiers)"
read -p "   Enter new rate limit (or press Enter for default): " RATE_LIMIT
RATE_LIMIT=${RATE_LIMIT:-5}

# Ask about sample fallback
echo
echo "🔄 Fallback Options"
echo "   Use sample data when APIs fail? (recommended for development)"
echo "   Current: true"
read -p "   Enable sample fallback? [Y/n]: " USE_FALLBACK
case ${USE_FALLBACK:-Y} in
    [Yy]* ) USE_FALLBACK_BOOL="true";;
    * ) USE_FALLBACK_BOOL="false";;
esac

# Create configuration file
echo
echo "💾 Creating configuration file..."

cat > "$CONFIG_FILE" <<EOF
{
  "api_keys": {
    "etherscan": "${ETHERSCAN_KEY:-YOUR_ETHERSCAN_API_KEY_HERE}",
    "bscscan": "${BSCSCAN_KEY:-YOUR_BSCSCAN_API_KEY_HERE}",
    "polygonscan": "${POLYGONSCAN_KEY:-YOUR_POLYGONSCAN_API_KEY_HERE}"
  },
  "aggregator_key": ${AGGREGATOR_KEY:+"$AGGREGATOR_KEY"},
  "preferred_aggregator": ${PREFERRED_AGGREGATOR:+"$PREFERRED_AGGREGATOR"},
  "rate_limit": $RATE_LIMIT,
  "request_timeout": {
    "secs": 30,
    "nanos": 0
  },
  "max_retries": 3,
  "use_sample_fallback": $USE_FALLBACK_BOOL
}
EOF

echo "✅ Configuration saved to: $CONFIG_FILE"
echo

# Show what was configured
echo "📊 Summary:"
if [ -n "$ETHERSCAN_KEY" ] && [ "$ETHERSCAN_KEY" != "YOUR_ETHERSCAN_API_KEY_HERE" ]; then
    echo "   ✅ Etherscan API key configured"
else
    echo "   ⚠️  Etherscan API key not set (will use sample data)"
fi

if [ -n "$BSCSCAN_KEY" ] && [ "$BSCSCAN_KEY" != "YOUR_BSCSCAN_API_KEY_HERE" ]; then
    echo "   ✅ BSCScan API key configured"
else
    echo "   ⚠️  BSCScan API key not set (will use sample data)"
fi

if [ -n "$POLYGONSCAN_KEY" ] && [ "$POLYGONSCAN_KEY" != "YOUR_POLYGONSCAN_API_KEY_HERE" ]; then
    echo "   ✅ PolygonScan API key configured"
else
    echo "   ⚠️  PolygonScan API key not set (will use sample data)"
fi

echo "   🕐 Rate limit: $RATE_LIMIT requests/minute"
echo "   🔄 Sample fallback: $USE_FALLBACK_BOOL"
echo

echo "🎯 What's Next:"
echo "   1. Run 'cargo build --release' to compile with new configuration"
echo "   2. Launch your wallet: './target/release/vaughan'"
echo "   3. Test the transaction history feature"
echo
echo "💡 Pro Tips:"
echo "   • Free API keys give you 100,000 requests/day"
echo "   • PulseChain doesn't require API keys (uses Blockscout)"
echo "   • You can edit $CONFIG_FILE manually anytime"
echo
echo "🚀 Happy wallet building!"