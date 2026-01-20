# 🚀 Quick Start: Deploy Tokens with Custom Parameters

## ✅ Ready to Use!

The constructor arguments feature is now **fully integrated** into your Token Launcher UI.

---

## 📋 Before You Start

### Requirements

1. ✅ **Forge installed** (comes with Foundry)
   ```bash
   forge --version
   ```

2. ✅ **Wallet account** created/imported in the app
3. ✅ **Test tokens** on your chosen network

### Get Test Tokens

- **PulseChain Testnet v4**: [faucet.v4.testnet.pulsechain.com](https://faucet.v4.testnet.pulsechain.com)
- **Sepolia**: [sepoliafaucet.com](https://sepoliafaucet.com)
- **BSC Testnet**: [testnet.bnbchain.org/faucet-smart](https://testnet.bnbchain.org/faucet-smart)

---

## 🎯 Quick Start (2 Steps)

### Step 1: Start the Application

```bash
cd /home/r4/Desktop/Vaughan_V1
cargo run
```

### Step 2: Deploy Your Token

1. Open **Token Launcher** tab
2. Fill in your token details:
   - **Name**: `"My Test Token"`
   - **Symbol**: `"MTT"`
   - **Supply**: `1000000`
   - **Decimals**: `18`
3. Select **network** (e.g., "PulseChain Testnet v4")
4. **Connect wallet**
5. Click **"Deploy Token"**

**That's it!** 🎉

---

## 📊 What Happens

```
User fills form → Connect Wallet → Click Deploy → Validation
                     ↓                                ↓
            Gets private key                      Forge CLI
            from wallet                              ↓
                                              CustomToken.sol
                                                   ↓
                                              Blockchain!
```

---

## ✅ Verify Deployment

### In Terminal
Look for these logs:
```
🚀 Using Forge deployment with custom constructor arguments
🔨 Forge Deployment: My Test Token (MTT) to PulseChain Testnet v4...
🔧 Constructor args: name='My Test Token', symbol='MTT', supply=1000000, decimals=18
⚡ Executing forge create...
✅ Contract deployed successfully!
📍 Contract Address: 0x...
🔗 Transaction Hash: 0x...
🌐 Explorer: https://scan.v4.testnet.pulsechain.com/address/0x...
```

### On Block Explorer
1. Click the explorer link from the output
2. Verify:
   - ✅ Contract deployed
   - ✅ Token name matches
   - ✅ Token symbol matches
   - ✅ Total supply matches

### In Your Wallet
1. Go to MetaMask (or your wallet)
2. **Import Token** → **Custom Token**
3. Paste contract address
4. Token name, symbol, decimals auto-fill ✅
5. Check balance - you should see your tokens!

---

## 🎨 Example: Create "Rocket Coin"

```bash
# 1. Run app
cargo run

# 2. In UI:
# - Import/create account if needed
# - Connect account
# - Fill form:
Name:         Rocket Coin
Symbol:       ROCKET
Supply:       10000000
Decimals:     18
Network:      PulseChain Testnet v4

# 3. Deploy!
```

Result: **10 million ROCKET tokens** deployed to your address! 🚀

---

## 🔍 Troubleshooting

### "No wallet account selected"
- Make sure you've imported or created an account in the wallet
- Connect the account using the "Connect" button
- The deployment will use the connected account's private key automatically

### "insufficient funds"
- Get test tokens from faucet
- Make sure you're on the correct network

### "forge: command not found"
```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### Deployment hangs
- Check RPC endpoint is accessible
- Try a different network
- Check terminal logs for errors

---

## 🎓 Tips & Tricks

### Tip 1: Use Testnets First
Always test on testnets before mainnet:
- Free test tokens
- No financial risk
- Same functionality

### Tip 2: Standard Decimals
Use **18 decimals** (Ethereum standard) unless you have a specific reason not to.

### Tip 3: Supply Calculation
Your supply is multiplied by `10^decimals`:
- Supply: `1000000`, Decimals: `18` → 1,000,000.000000000000000000 tokens

### Tip 4: Verify on Explorer
Always verify your contract on the block explorer - it's free and builds trust.

### Tip 5: Save Contract Address
**Save your contract address immediately** - you'll need it to import to wallets and for interactions.

---

## 📚 More Information

- **Full Documentation**: [`docs/UI_INTEGRATION_COMPLETE.md`](./docs/UI_INTEGRATION_COMPLETE.md)
- **Deployment Guide**: [`docs/custom_token_deployment.md`](./docs/custom_token_deployment.md)
- **Quick Reference**: [`docs/QUICK_REFERENCE.md`](./docs/QUICK_REFERENCE.md)

---

## 🎉 Success!

You now have a **fully functional token deployment UI** that:

✅ Accepts custom parameters  
✅ Deploys to any EVM network  
✅ Uses secure Forge deployment  
✅ Shows real-time progress  
✅ Provides explorer links  

**Happy token launching!** 🚀

---

## 🆘 Need Help?

1. Check logs in terminal
2. Review [`docs/UI_INTEGRATION_COMPLETE.md`](./docs/UI_INTEGRATION_COMPLETE.md)
3. Test with CLI first: `./scripts/test_custom_token.sh`
4. Verify forge works: `forge --version`

---

**Remember**: 
- Always use **test networks** during development!
- The app automatically uses your **connected wallet account's private key**
- Private keys never leave your wallet's secure keystore
