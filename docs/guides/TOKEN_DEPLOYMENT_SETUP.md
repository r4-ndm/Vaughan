# Token Launcher - Real Blockchain Deployment Setup

## Overview

The token launcher has been updated to use **REAL blockchain deployment** instead of mock/simulated deployments. This means your tokens will actually be deployed to the blockchain network you select!

## 🚨 IMPORTANT: Before You Start

### The Mock Deployment Problem (FIXED)

**Old Behavior (before this fix):**
- Clicking "Deploy" showed fake addresses like `0x0000000000000000000000000000000068EB3a98`
- Message said "Would deploy with args..." - it wasn't actually deploying
- Error: "Deployment not completed" because nothing was actually deployed

**New Behavior (after this fix):**
- Uses `RealTokenLauncher` with Alloy library for actual blockchain interaction
- Connects to real RPC endpoints
- Signs and broadcasts actual deployment transactions
- Returns real contract addresses and transaction hashes

## How It Works Now

**Wallet Connect Integration:**
1. Select an account from the dropdown in the DApp Platform
2. Click the "🟢 Connect" button
3. The wallet verifies the account is available
4. Token deployment uses the connected account

**Temporary Setup Requirement:**

For now, you still need to set the `PRIVATE_KEY` environment variable because the keystore private key export feature is not yet fully implemented. Once you connect your wallet account, the system will:

1. ✅ Verify you have a connected account
2. ✅ Get the account details (address, balance, etc.)
3. ⚠️  Use PRIVATE_KEY from environment (temporary workaround)
4. ✅ Deploy the token using your connected account's address

**Future:** The PRIVATE_KEY requirement will be removed once we implement secure private key export from the keystore.

##Setup Steps

### Step 1: Connect Your Wallet Account

1. Launch the wallet application
2. Open the DApp Platform (7-tab interface)
3. Go to the **T4: Token Launcher** tab
4. At the top of the page, you'll see account selection dropdown
5. Select the account you want to use for deployment
6. Click the **"🟢 Connect"** button
7. Wait for the "Account connected" message

### Step 2: Get a Private Key (Temporary Requirement)

You need a private key with funds on the network you want to deploy to.

**⚠️  SECURITY WARNING:**
- **NEVER** use a private key from your main wallet with significant funds
- **ALWAYS** create a new wallet specifically for development/deployment
- **NEVER** commit your `.env` file to version control

#### Creating a Test Wallet

```bash
# Option 1: Using your existing wallet (RECOMMENDED for testnets only)
# In the Vaughan wallet, create a new account called "Deployment Test"
# Export the private key (Make sure it's a TEST account with minimal funds!)

# Option 2: Generate a new key with cast (Foundry)
cast wallet new

# Option 3: Generate online (use ONLY for testnets!)
# Visit: https://vanity-eth.tk/ (open source, run locally for security)
```

###Step 2: Fund Your Deployment Wallet

You need native tokens (ETH, PLS, BNB, etc.) to pay for gas.

**For Testnets (RECOMMENDED for learning):**

```bash
# PulseChain Testnet v4 (Chain ID: 943)
# Faucet: https://faucet.v4.testnet.pulsechain.com/
# You'll get test tPLS tokens

# Ethereum Sepolia Testnet (Chain ID: 11155111)
# Faucet: https://sepoliafaucet.com/

# BSC Testnet (Chain ID: 97)
# Faucet: https://testnet.bnbchain.org/faucet-smart
```

**For Mainnets (⚠️ COSTS REAL MONEY):**

| Network | Chain ID | Currency | Typical Cost |
|---------|----------|----------|--------------|
| Ethereum | 1 | ETH | $20-100+ |
| BSC | 56 | BNB | $0.50-2 |
| Polygon | 137 | MATIC | $0.01-0.10 |
| PulseChain | 369 | PLS | ~$0.01 |
| PulseChain Testnet v4 | 943 | tPLS | FREE |

### Step 3: Set Environment Variable

Create or edit `.env` file in the project root:

```bash
cd /home/r4/Desktop/Vaughan_V1

# Create .env file
nano .env
```

Add your private key (WITHOUT the "0x" prefix):

```env
# Example .env file
PRIVATE_KEY=1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef

# Your actual key should be 64 hexadecimal characters
# DO NOT include "0x" prefix
# DO NOT share this file or commit it to git
```

**Important:**
- Private key must be exactly 64 hexadecimal characters
- No "0x" prefix
- No spaces or newlines

### Step 4: Verify Setup

Before deploying to mainnet, test your setup:

```bash
# Load environment variables
source .env

# Verify private key is set
echo $PRIVATE_KEY | wc -c  # Should output 65 (64 chars + newline)

# Build the wallet
cargo build --release

# Run the wallet
./target/release/vaughan
```

### Step 5: Deploy Your Token

1. Launch the wallet
2. Open DApp Platform (7-tab interface)
3. Go to **T4: Token Launcher** tab
4. Fill in token details:
   - **Name**: Your token name (e.g., "My Test Token")
   - **Symbol**: Token symbol (e.g., "MTT")
   - **Total Supply**: Initial supply (e.g., "1000000")
   - **Decimals**: Usually 18 (standard)
   - **Network**: Select your target network
   
5. Click **"Launch Token"**
6. Watch the console output for deployment progress

### Expected Console Output

#### Successful Deployment:
```
🚀 Starting REAL blockchain token deployment...
🌐 Connecting to PulseChain Testnet v4 (943): https://rpc.v4.testnet.pulsechain.com
✅ Connected to PulseChain Testnet v4 - Latest block: 12345678
🚀 Starting real deployment of Burnable ERC20 Token token...
📝 Deploying from address: 0x1234...5678
💰 Account balance: 1000000000000000000 wei (1.000000 tPLS)
🔨 Compiling Burnable ERC20 Token contract...
✅ Contract compiled - Bytecode size: 1234 bytes
⛽ Estimated gas: 500000
💲 Gas price: 1000000000 wei (1 gwei)
💰 Estimated deployment cost: 500000000000000 wei (0.000500 tPLS)
📡 Broadcasting transaction...
✅ Transaction broadcasted: 0xabcd...ef01
⏳ Waiting for transaction confirmation...
✅ Transaction confirmed in block 12345679
🎉 Contract deployed successfully!
📍 Contract Address: 0x9876...5432
🔗 Transaction Hash: 0xabcd...ef01
📦 Block Number: 12345679
⛽ Gas Used: 485123 (97.02%)
💰 Actual Cost: 485123000000000 wei (0.000485 tPLS)
🔍 Initiating contract verification...
✅ REAL Token deployment completed successfully!
```

#### Common Errors and Solutions:

**Error: "No private key configured"**
```
❌ DEPLOYMENT FAILED: No private key configured
🔧 To fix this:
   1. Set PRIVATE_KEY environment variable: export PRIVATE_KEY=your_64_char_hex_key
   2. Or add PRIVATE_KEY=your_key to .env file
   3. Make sure key has 64 hex characters (no 0x prefix)
```
**Solution:** Set the PRIVATE_KEY environment variable as described in Step 3

**Error: "Insufficient balance"**
```
❌ DEPLOYMENT FAILED: Insufficient balance
💰 Balance needed: 500000000000000 wei (0.000500 ETH)
💰 Balance available: 0 wei (0.000000 ETH)
🔧 To fix this:
   1. Get test tPLS from: https://faucet.v4.testnet.pulsechain.com/
   2. Your address: 0x1234...5678
   3. Wait 1-2 minutes for tokens to arrive
```
**Solution:** Fund your deployment wallet using the appropriate faucet (see Step 2)

**Error: "No provider available for network"**
```
❌ DEPLOYMENT FAILED: No provider for network 1
🌍 Available networks:
   • PulseChain Testnet v4 (ID: 943)
   • PulseChain (ID: 369)
🔧 To fix this:
   1. Make sure you selected the correct network in the GUI
   2. Recommended: Use PulseChain Testnet v4 (ID: 943) for testing
   3. Check network connectivity if no networks are available
```
**Solution:** Select a supported network or check your internet connection

## Supported Networks

The wallet connects to these networks automatically:

| Network | Chain ID | RPC Endpoint | Status |
|---------|----------|--------------|--------|
| Ethereum | 1 | https://ethereum-rpc.publicnode.com | ✅ Active |
| BSC | 56 | https://bsc-dataseed1.binance.org | ✅ Active |
| Polygon | 137 | https://polygon-rpc.com | ✅ Active |
| PulseChain | 369 | https://rpc.pulsechain.com | ✅ Active |
| **PulseChain Testnet v4** | **943** | **https://rpc.v4.testnet.pulsechain.com** | **✅ RECOMMENDED** |

## Security Best Practices

### 🔐 Protecting Your Private Key

1. **Never commit `.env` to version control:**
   ```bash
   # Add to .gitignore
   echo ".env" >> .gitignore
   ```

2. **Use different keys for different purposes:**
   - Development/Testing: Minimal funds, testnet only
   - Staging: Small amounts for final testing
   - Production: Main deployment key, stored securely

3. **Consider using hardware wallets for production:**
   - Ledger
   - Trezor
   - Hardware security modules (HSM)

4. **Rotate keys periodically:**
   - Create new deployment keys every few months
   - Transfer remaining funds before rotation

5. **Monitor deployment addresses:**
   - Set up alerts for unusual activity
   - Keep audit logs of all deployments

### 🛡️ Testing Strategy

1. **Start with testnets (FREE):**
   - PulseChain Testnet v4 (recommended)
   - Ethereum Sepolia
   - BSC Testnet

2. **Test small amounts first:**
   - Deploy with minimal token supply
   - Verify contract functionality
   - Test all features before mainnet

3. **Use contract verification:**
   - The wallet automatically verifies contracts
   - Check block explorer after deployment
   - Ensure source code matches

## Troubleshooting

### Problem: Wallet won't start after setting PRIVATE_KEY

**Diagnosis:**
```bash
# Check if .env is properly formatted
cat .env

# Should show:
# PRIVATE_KEY=your64characterhexadecimalkey
```

**Solution:**
- Ensure no spaces around the `=` sign
- Ensure private key is exactly 64 hex characters
- No `0x` prefix
- No quotes around the key

### Problem: Deployment succeeds but contract doesn't work

**Possible causes:**
1. **Wrong network selected** - Double check chain ID
2. **Insufficient initial supply** - Try larger supply value
3. **Bytecode compilation issue** - Check console for compilation errors

**Solution:**
- Review deployment logs carefully
- Verify contract on block explorer
- Test contract functions using block explorer interface

### Problem: High gas costs

**Solutions:**
1. **Use cheaper networks:**
   - PulseChain (cheapest)
   - Polygon
   - BSC
   - Avoid Ethereum mainnet for testing

2. **Deploy during low activity:**
   - Weekend mornings (UTC)
   - Avoid market volatility periods

3. **Optimize contract:**
   - Remove unnecessary features
   - Use simpler token templates

## Next Steps

After successful deployment:

1. **Verify your contract:**
   - Check block explorer link in deployment output
   - Verify source code is published

2. **Test contract functions:**
   - Try transferring tokens
   - Test burn/mint functions if enabled
   - Verify total supply

3. **Add token to wallet:**
   - Use the contract address from deployment
   - Add to MetaMask/Trust Wallet/etc.

4. **Document your deployment:**
   - Save contract address
   - Save transaction hash
   - Save deployment parameters
   - Record gas costs

## FAQ

### Q: Can I deploy to multiple networks?

A: Yes! Just select a different network from the dropdown and deploy again. Each network will have a separate contract address.

### Q: What if I lose my private key after deployment?

A: The deployed contract will continue to exist and function, but you won't be able to interact with owner-only functions (like mint/burn if enabled). **Always back up your private key securely!**

### Q: Can I update a deployed contract?

A: No, smart contracts are immutable once deployed. If you need to make changes, you must deploy a new contract.

### Q: How much does it cost?

A: Depends on the network:
- **Testnet**: FREE (use faucets)
- **PulseChain**: ~$0.01-0.05
- **BSC**: ~$0.50-2
- **Polygon**: ~$0.05-0.50
- **Ethereum**: ~$20-100+ (avoid for testing!)

### Q: Is this safe for production use?

A: The code is functional but should be audited before mainnet deployment with real funds. **Use at your own risk.** Start with testnets first!

##Links and Resources

- **PulseChain Testnet v4 Faucet**: https://faucet.v4.testnet.pulsechain.com/
- **PulseChain Block Explorer**: https://scan.pulsechain.com/
- **PulseChain Testnet Explorer**: https://scan.v4.testnet.pulsechain.com/
- **Ethereum Block Explorer**: https://etherscan.io/
- **BSC Block Explorer**: https://bscscan.com/
- **Polygon Block Explorer**: https://polygonscan.com/

---

**Remember: Always test on testnets first! Never use keys with significant funds for development.**

Good luck with your token deployment! 🚀