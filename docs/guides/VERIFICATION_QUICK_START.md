# Contract Verification Quick Start

## What You'll Get After Deployment

After deploying your token, you'll receive all the information needed to verify it on a block explorer:

```
✅ Token deployed successfully!

📍 Contract Address:
0x... (your contract address)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 VERIFICATION INFORMATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Contract Name: CustomToken
Compiler Version: v0.8.20+commit.a1b79de6
EVM Version: shanghai
Optimization Enabled: true
Optimization Runs: 200

📄 Flattened Source Code:
(Available in deployment logs)
```

## How to Verify on PulseChain Testnet v4

1. **Go to the explorer**: https://scan.v4.testnet.pulsechain.com
2. **Search for your contract address**
3. **Click "Contract" tab → "Verify & Publish"**
4. **Fill in the form using the info above:**
   - Contract Name: `CustomToken`
   - Compiler: Select the version shown (e.g., `v0.8.20+commit.a1b79de6`)
   - EVM Version: `shanghai` (for PulseChain)
   - Optimization: **Yes**
   - Runs: `200`
   - Source Code: Copy from deployment logs
5. **Submit and wait a few seconds**
6. **Done! Your contract is now verified** ✅

## Quick Copy Format

For easy reference, here's the format you need:

**Contract Settings:**
- Compiler: v0.8.20+commit.a1b79de6 (or as shown in your output)
- EVM: shanghai
- Optimization: Yes + 200 runs

**Source Code:**
Use the flattened code from deployment (includes all dependencies)

## Network-Specific Info

| Network | EVM Version | Explorer |
|---------|-------------|----------|
| PulseChain Testnet v4 | shanghai | scan.v4.testnet.pulsechain.com |
| PulseChain Mainnet | shanghai | scan.pulsechain.com |
| Ethereum | cancun | etherscan.io |
| BSC | shanghai | bscscan.com |
| Polygon | shanghai | polygonscan.com |

## Where to Find the Flattened Source

The full flattened source code is available in your deployment logs. It starts with:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// Flattened from: src/CustomToken.sol
...
```

## Troubleshooting

**Q: Verification failed with "bytecode mismatch"**  
A: Make sure you selected the exact compiler version, EVM version, and optimization settings shown in your deployment info.

**Q: I can't find the flattened source code**  
A: Check your terminal output after deployment. You can also generate it manually with: `forge flatten src/CustomToken.sol`

**Q: Which EVM version should I use?**  
A: Use the one shown in your deployment info - it's automatically selected for your target network.

**Q: Do I need to provide constructor arguments?**  
A: Usually no - the explorer can read them from the transaction. If asked, they're your token name, symbol, supply, and decimals.

## That's It!

You now have everything needed to verify your contract. The system automatically:
- ✅ Uses the right compiler version
- ✅ Selects the correct EVM version for your network
- ✅ Applies proper optimization settings
- ✅ Generates flattened source code

Just copy the information and paste it into the explorer's verification form! 🚀
