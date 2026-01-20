# Block Explorer APIs Setup

## Overview

To get **real transaction history** in your Vaughan wallet (instead of sample data), you need API keys from block explorer services. These APIs are the "refined petrol" that makes your wallet run smoothly!

## 🚫 **Web Scraping vs ✅ API Access**

| Method | Reliability | Performance | Legality | Maintenance |
|--------|-------------|-------------|----------|-------------|
| **Web Scraping** | ❌ Breaks often | ❌ Slow & heavy | ❌ ToS violation | ❌ Constant fixes |
| **Official APIs** | ✅ Stable | ✅ Fast JSON | ✅ Officially supported | ✅ Long-term stable |

**Bottom line**: APIs are the professional way to access blockchain data.

## 🔑 **Getting API Keys** (Free!)

### 1. Etherscan (Ethereum Mainnet)
- Go to: https://etherscan.io/apis
- Create free account
- Generate API key
- **Limits**: 100,000 requests/day (plenty for a wallet!)

### 2. BSCScan (Binance Smart Chain)
- Go to: https://bscscan.com/apis
- Create free account  
- Generate API key
- **Limits**: 100,000 requests/day

### 3. PolygonScan (Polygon)
- Go to: https://polygonscan.com/apis
- Create free account
- Generate API key
- **Limits**: 100,000 requests/day

### 4. PulseChain (No API Key Needed!)
- Uses Blockscout API
- No registration required
- Free to use

## 🛠️ **Easy Setup**

### Option 1: Automated Setup (Recommended)
```bash
cd /home/r4/Desktop/Vaughan_V1
./setup_api_keys.sh
```

This interactive script will:
- Guide you through API key setup
- Create the configuration file
- Set up rate limiting
- Configure fallback options

### Option 2: Manual Configuration

1. Copy the template:
```bash
mkdir -p ~/.config/vaughan
cp config/explorer_apis.json.template ~/.config/vaughan/explorer_apis.json
```

2. Edit the file:
```bash
nano ~/.config/vaughan/explorer_apis.json
```

3. Replace the placeholder values:
```json
{
  "api_keys": {
    "etherscan": "YOUR_ACTUAL_API_KEY_HERE",
    "bscscan": "YOUR_ACTUAL_API_KEY_HERE", 
    "polygonscan": "YOUR_ACTUAL_API_KEY_HERE"
  },
  "rate_limit": 5,
  "request_timeout": {
    "secs": 30,
    "nanos": 0
  },
  "max_retries": 3,
  "use_sample_fallback": true
}
```

## ⚙️ **Configuration Options**

| Setting | Default | Description |
|---------|---------|-------------|
| `rate_limit` | 5 | Requests per minute (conservative for free tiers) |
| `request_timeout` | 30s | HTTP timeout for API calls |
| `max_retries` | 3 | Retry attempts on failure |
| `use_sample_fallback` | true | Use sample data when all APIs fail |

## 🔄 **How It Works**

The wallet tries APIs in this order:

1. **Primary API** (e.g., Etherscan for Ethereum)
2. **Secondary APIs** (if available)
3. **Sample Data Fallback** (if enabled)

### Rate Limiting
- **Automatic**: Waits between requests to respect limits
- **Per-service**: Each API has its own rate limiter
- **Configurable**: Adjust based on your API tier

### Error Handling  
- **Exponential backoff** on failures
- **Multiple endpoints** per network
- **Graceful degradation** to sample data

## 📊 **Supported Networks**

| Network | Primary API | Requires Key | Fallback APIs |
|---------|-------------|--------------|---------------|
| Ethereum (1) | Etherscan | ✅ Yes | Blockchair |
| BSC (56) | BSCScan | ✅ Yes | None |
| Polygon (137) | PolygonScan | ✅ Yes | None |
| PulseChain (369) | PulseScan | ❌ No | None |

## 🎯 **Testing Your Setup**

After configuration:

1. **Build the wallet**:
```bash
cargo build --release
```

2. **Run the wallet**:
```bash
./target/release/vaughan
```

3. **Test transaction history**:
   - Click the "History" button
   - Check if real data loads (not sample data)
   - Verify no API errors in logs

## 📝 **Logs & Debugging**

The wallet logs API activity:

```
✅ Successfully fetched 15 transactions from Etherscan
⚠️  BSCScan API failed: API key required
📝 All APIs failed, returning sample data
```

Check logs to verify your API keys are working.

## 💡 **Pro Tips**

### Free Tier Optimization
- **5 requests/minute** is conservative and safe
- **100k requests/day** = ~70 requests/minute sustained
- **Monitor your usage** on the API provider dashboards

### Production Deployment
- **Paid tiers** available for higher limits
- **Multiple API keys** for load distribution  
- **Caching** reduces API calls (built-in)

### Security
- **Never commit API keys** to version control
- **Use environment variables** in production
- **Rotate keys periodically**

## 🔧 **Troubleshooting**

### "API key required but not configured"
- Check your `~/.config/vaughan/explorer_apis.json` file
- Verify the API key is correctly entered (no extra spaces)
- Make sure the service name matches exactly ("etherscan", not "Etherscan")

### "Rate limit reached"
- Wait for the rate limit window to reset (60 seconds)
- Consider reducing `rate_limit` in config
- Check if you have multiple instances running

### "All block explorer APIs failed"
- Verify your internet connection
- Check if the API services are down
- Enable `use_sample_fallback` for development

## 🚀 **What's Next?**

Once APIs are configured:
1. Your wallet will show **real transaction history**
2. **No more sample data** (unless APIs fail)
3. **Fast, reliable** transaction loading
4. **Professional-grade** blockchain data access

You've now transformed your wallet from using "crude oil" (RPC) to "refined petrol" (APIs)! 🛢️➡️⛽