# PulseChain Testnet v4 - Token Launcher Testing Guide

## 🚀 PulseChain Testnet v4 Setup

### Network Details
- **Network Name**: PulseChain Testnet v4
- **Chain ID**: 943
- **RPC URL**: `https://rpc.v4.testnet.pulsechain.com`
- **Block Explorer**: `https://scan.v4.testnet.pulsechain.com`
- **Native Token**: tPLS (Test PulseChain)

### Prerequisites

1. **Get Test tPLS Tokens**
   - Visit the PulseChain Testnet v4 faucet
   - You'll need tPLS for gas fees during token deployment
   - Faucet: Check PulseChain Discord or official docs for current faucet

2. **Verify Network Configuration**
   - Ensure Vaughan recognizes PulseChain Testnet v4 (Chain ID 943)
   - Check that the network dropdown includes "PulseChain Testnet v4"

## 🧪 Testing Process

### Step 1: Launch DApp Platform
```bash
cd /home/r4/Desktop/Vaughan_V1
cargo run --bin dapp-platform --release
```

### Step 2: Navigate to Token Launcher
- Click on **T4: Launcher** tab
- Verify the interface loads properly

### Step 3: Configure Token Deployment
1. **Template Selection**: Choose from:
   - Basic ERC20 (recommended for first test)
   - Burnable ERC20
   - Mintable ERC20
   - Pausable ERC20
   - Full-Featured ERC20

2. **Token Parameters**:
   - **Name**: `Test Token` (or your preference)
   - **Symbol**: `TEST` (3-5 characters recommended)
   - **Decimals**: `18` (standard)
   - **Total Supply**: `1000000` (1 million tokens)

3. **Network Selection**: Select **"PulseChain Testnet v4"**

### Step 4: Deploy Token
1. Click **"🚀 Deploy Token"**
2. Monitor deployment status
3. Wait for transaction confirmation

### Step 5: Verify Deployment
1. **Transaction Hash**: Note the deployment tx hash
2. **Contract Address**: Record the deployed contract address
3. **Block Explorer**: Visit `https://scan.v4.testnet.pulsechain.com`
4. **Verify Contract**: Check if auto-verification worked

## 📋 Expected Results

### Successful Deployment Should Show:
- ✅ Transaction hash
- ✅ Contract address (0x...)
- ✅ Deployment status: "Deployed"
- ✅ Gas used information
- ✅ Block confirmation

### Auto-Verification Status:
- 🟡 **In Progress**: Contract submitted for verification
- ✅ **Success**: Contract source visible on block explorer
- ❌ **Failed**: May need manual verification (rare)

## 🔍 Testing Checklist

### Pre-Deployment
- [ ] Wallet has sufficient tPLS for gas
- [ ] Network is set to PulseChain Testnet v4
- [ ] Token parameters are valid
- [ ] Template selected correctly

### During Deployment
- [ ] UI shows "⏳ Deploying..." status
- [ ] No error messages appear
- [ ] Transaction is broadcast successfully

### Post-Deployment
- [ ] Contract address is generated
- [ ] Transaction appears on block explorer
- [ ] Contract is verified (may take a few minutes)
- [ ] Token details match input parameters

## 🛠️ Troubleshooting

### Common Issues:

1. **Insufficient Gas**
   - **Error**: Transaction fails due to low tPLS balance
   - **Solution**: Get more tPLS from faucet

2. **Network Not Found**
   - **Error**: "PulseChain Testnet v4" not in dropdown
   - **Solution**: Check network configuration in code

3. **RPC Connection Issues**
   - **Error**: Cannot connect to PulseChain RPC
   - **Solution**: Try alternative RPC endpoints

4. **Verification Pending**
   - **Status**: Contract deployed but not verified
   - **Solution**: Wait 5-10 minutes, verification is async

### Debug Commands
```bash
# Check if build includes PulseChain Testnet v4
grep -r "943" src/network/ src/gui/

# Verify network configuration
grep -A5 -B5 "PulseChain Testnet" src/
```

## 📊 Test Results Documentation

### Template: [Template Name]
- **Token Name**: _____________
- **Symbol**: _____________
- **Total Supply**: _____________
- **Decimals**: _____________
- **Network**: PulseChain Testnet v4

### Deployment Results
- **Transaction Hash**: 0x_____________
- **Contract Address**: 0x_____________
- **Gas Used**: _____________
- **Block Number**: _____________
- **Verification Status**: _____________
- **Explorer Link**: https://scan.v4.testnet.pulsechain.com/address/0x_____________

### Performance Metrics
- **Deployment Time**: _____ seconds
- **Verification Time**: _____ minutes
- **UI Responsiveness**: ⭐⭐⭐⭐⭐ (1-5 stars)
- **Overall Experience**: ⭐⭐⭐⭐⭐ (1-5 stars)

## 🎯 Success Criteria

A successful test should demonstrate:
1. ✅ Smooth UI interaction in T4 tab
2. ✅ Successful token deployment on PulseChain Testnet v4
3. ✅ Auto-verification working (contract source visible)
4. ✅ Block explorer integration functional
5. ✅ Token parameters correctly deployed

## 📞 Support

If you encounter issues during testing:
1. Check the console output for error messages
2. Verify network connectivity to PulseChain RPC
3. Ensure sufficient tPLS balance for gas fees
4. Document any errors for debugging

Good luck with your Token Launcher test on PulseChain Testnet v4! 🚀