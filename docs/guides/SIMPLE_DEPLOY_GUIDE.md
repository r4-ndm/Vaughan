# Simple Token Deployment Guide

## Step 1: Deploy Token (No API Key Needed!)

```bash
cd /home/r4/Desktop/Vaughan_V1

# Set your private key
export PRIVATE_KEY="0x..."

# Deploy to PulseChain Testnet v4
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy

# Clean up
unset PRIVATE_KEY
```

## Step 2: Note the Contract Address

After deployment, you'll see:
```
Deployed to: 0xYOUR_CONTRACT_ADDRESS_HERE
Transaction hash: 0x...
```

## Step 3: Manual Verification (Easy!)

1. **Go to block explorer**:
   - PulseChain Testnet: https://scan.v4.testnet.pulsechain.com/address/YOUR_ADDRESS
   - Click "Contract" tab
   - Click "Verify & Publish"

2. **Fill in the form**:
   - **Compiler Type**: Solidity (Single file)
   - **Compiler Version**: v0.8.26+commit.8a97fa7a
   - **Open Source License**: MIT
   - **Optimization**: Yes
   - **Runs**: 200

3. **Paste the flattened source**:
   ```bash
   # Copy the flattened source to clipboard
   cat /home/r4/Desktop/Vaughan_V1/SimpleToken_flattened.sol | xclip -selection clipboard
   # or just open the file and copy it
   ```

4. **Constructor Arguments**: (Leave empty - our SimpleToken has no constructor args)

5. **Click "Verify and Publish"**

## That's It!

✅ No API keys needed  
✅ No complex setup  
✅ Works on any block explorer  
✅ Takes 2 minutes  

## Flattened Source Location

The flattened source code is in:
```
/home/r4/Desktop/Vaughan_V1/SimpleToken_flattened.sol
```

## For Other Networks

### Ethereum (Sepolia Testnet)
```bash
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://ethereum-sepolia.publicnode.com \
  --private-key $PRIVATE_KEY \
  --legacy
```
Verify at: https://sepolia.etherscan.io

### BSC Testnet
```bash
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://data-seed-prebsc-1-s1.binance.org:8545 \
  --private-key $PRIVATE_KEY \
  --legacy
```
Verify at: https://testnet.bscscan.com

## Quick Command (All-in-One)

```bash
#!/bin/bash
cd /home/r4/Desktop/Vaughan_V1

# Deploy
echo "Deploying..."
RESULT=$(forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy 2>&1)

# Extract contract address
ADDRESS=$(echo "$RESULT" | grep "Deployed to:" | awk '{print $3}')
TX=$(echo "$RESULT" | grep "Transaction hash:" | awk '{print $3}')

echo ""
echo "✅ Deployed!"
echo "📍 Contract: $ADDRESS"
echo "🔗 TX: $TX"
echo ""
echo "🔍 Verify at:"
echo "https://scan.v4.testnet.pulsechain.com/address/$ADDRESS"
echo ""
echo "📋 Flattened source:"
echo "/home/r4/Desktop/Vaughan_V1/SimpleToken_flattened.sol"
```

Save as `deploy.sh`, then run:
```bash
chmod +x deploy.sh
export PRIVATE_KEY="0x..."
./deploy.sh
```

## Done!

You now have a deployed OpenZeppelin ERC20 token that you can manually verify! 🎉
