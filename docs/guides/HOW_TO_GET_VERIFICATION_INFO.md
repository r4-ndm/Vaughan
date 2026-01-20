# How to Access Verification Information

## Quick Start

After deploying your token, the verification information will be printed to your **terminal/console** where you ran the Vaughan app.

## Step-by-Step Guide

### 1. Restart the App with the New Build

First, make sure you're running the latest version that includes verification info:

```bash
# Stop the current app (close the UI window or press Ctrl+C in the terminal)

# Start the new version
./restart_app.sh
```

Or manually:
```bash
pkill vaughan  # Stop current instance
./target/release/vaughan  # Start new release build
```

### 2. Deploy Your Token

Use the Token Launcher in the UI as normal:
1. Connect your wallet
2. Fill in token details (name, symbol, supply, decimals)
3. Click "Deploy Token"

### 3. Check Your Terminal

While the deployment happens, watch your **terminal/console output**. You'll see:

```
🔨 Forge Deployment: MyToken (MTK) to PulseChain Testnet v4...
✅ Using account: Main Account (0x...)
🔧 Constructor args: name='MyToken', symbol='MTK', supply=1000000, decimals=18
⚡ Executing forge create...
🛠️  EVM Version: shanghai
✅ Contract deployed successfully!
📍 Contract Address: 0x1234567890AbcdEF1234567890aBcdef12345678
🔗 Transaction Hash: 0x...
👤 Deployer: 0x...
📝 Generating verification information...
📄 Flattening contract: src/CustomToken.sol
✅ Contract flattened successfully (12456 bytes)
✅ Verification info ready
✅ Forge deployment completed successfully!
📍 Contract Address: 0x1234567890AbcdEF1234567890aBcdef12345678
🔗 Transaction Hash: 0x...
🌐 Explorer: https://scan.v4.testnet.pulsechain.com/address/0x...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 VERIFICATION INFORMATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Contract Name: CustomToken
Compiler Version: v0.8.20+commit.a1b79de6
EVM Version: shanghai
Optimization Enabled: true
Optimization Runs: 200

📄 Flattened Source Code (copy from below):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

[... FULL FLATTENED SOURCE CODE HERE ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💡 TIP: Copy the source code above to verify your contract on the block explorer.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4. Copy the Information

**In your terminal:**
1. Scroll to the "VERIFICATION INFORMATION" section
2. Select and copy:
   - Compiler Version
   - EVM Version
   - Optimization settings
3. Select and copy the entire flattened source code (between the separator lines)

**Pro Tip**: You can use terminal selection to copy. Most terminals let you:
- Click and drag to select text
- Right-click to copy
- Or use Ctrl+Shift+C to copy selected text

### 5. Verify on Block Explorer

Go to your explorer and paste:

**For PulseChain Testnet v4**: https://scan.v4.testnet.pulsechain.com

1. Search for your contract address
2. Click "Contract" → "Verify & Publish"
3. Fill in the form:
   - **Contract Name**: `CustomToken`
   - **Compiler**: `v0.8.20+commit.a1b79de6` (from your output)
   - **EVM Version**: `shanghai` (from your output)
   - **Optimization**: `Yes`
   - **Runs**: `200`
   - **Source Code**: Paste the flattened code from terminal
4. Submit and wait
5. Done! ✅

## Alternative: Generate Flattened Source Manually

If you didn't save the terminal output, you can regenerate it:

```bash
cd /home/r4/Desktop/Vaughan_V1
forge flatten src/CustomToken.sol
```

This will print the flattened source code to your terminal.

Or save it to a file:
```bash
forge flatten src/CustomToken.sol > flattened.sol
```

## Troubleshooting

### Q: I don't see the verification info in my terminal
**A**: You might be running the old build. Use `./restart_app.sh` to restart with the new build.

### Q: The terminal output is gone / scrolled away
**A**: 
- Look for your terminal's scrollback buffer
- Or run: `forge flatten src/CustomToken.sol` to regenerate the source
- Compiler version: Run `forge --version` and look for the "solc" line

### Q: Can I save the output to a file?
**A**: Yes! Run the app like this:
```bash
./target/release/vaughan 2>&1 | tee deployment.log
```
All output will be saved to `deployment.log`

### Q: I closed the terminal
**A**: Don't worry! You can regenerate the flattened source:
```bash
forge flatten src/CustomToken.sol > flattened.sol
forge --version  # To get compiler version
```

## UI Display

The UI will also show a summary of the verification info in the success dialog:

```
✅ Token deployed successfully!

📍 Contract Address: 0x...
🔗 Transaction Hash: 0x...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 VERIFICATION INFORMATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Contract Name: CustomToken
Compiler Version: v0.8.20+commit.a1b79de6
EVM Version: shanghai
Optimization Enabled: true
Optimization Runs: 200

📄 Flattened Source Code:
(Full source code available - copy from deployment result)

Source code length: 12456 characters

💡 TIP: Check your terminal for the full flattened source code.
```

## Summary

- ✅ **Verification info is printed to the terminal** after deployment
- ✅ **Full flattened source code is included** in the terminal output
- ✅ **All settings are automatically correct** for your network
- ✅ **Just copy from terminal and paste to explorer**

**Remember**: Always check your **terminal/console** where you started Vaughan. That's where the complete information appears!

## Need Help?

See also:
- `VERIFICATION_QUICK_START.md` - Quick reference guide
- `docs/VERIFICATION_INFO.md` - Complete technical documentation
- `docs/VERIFICATION_EXAMPLE.md` - Real examples with screenshots
