# Transaction Balance Error Diagnostic Guide

## Issue
Transaction from Tim to Bob failed with "unable to read balance" error or similar.

## Enhanced Logging Added
Added detailed error logging to `src/gui/simple_transaction.rs` to help diagnose balance-related transaction failures.

### New Log Output
When a balance error occurs, you'll now see:
```
❌ Gas estimation failed: [error message]
❌ Error details: [full error details]
💰 Balance check failed - this may indicate:
   1. Insufficient native token (tPLS) for gas fees
   2. Insufficient token balance for transfer
   3. RPC provider unable to read balance
   From address: 0x...
   To address: 0x...
   Amount: 1.0
   Token contract: 0x... (if ERC-20)
```

## Diagnostic Steps

### 1. Check Console Logs
Run the wallet and look for these log entries:
```bash
cargo run --bin vaughan
```

Look for:
- `📋 Paste address button clicked` - Confirms paste button works
- `🔍 Starting industry-standard gas estimation` - Transaction initiated
- `❌ Gas estimation failed` - Shows the actual error
- `💰 Balance check failed` - Indicates balance issue

### 2. Common Causes

#### A. Insufficient tPLS for Gas
**Symptom**: Error mentions "insufficient funds" or "balance"
**Solution**: 
- Check Tim's tPLS balance (should show in wallet)
- Need at least ~0.001 tPLS for gas fees
- Get testnet tPLS from faucet if needed

#### B. RPC Provider Issue
**Symptom**: Error mentions "unable to read" or "RPC error"
**Solution**:
- Check network connection
- Try switching RPC endpoint in network settings
- PulseChain Testnet RPC: `https://rpc.v4.testnet.pulsechain.com`

#### C. Account Not Unlocked
**Symptom**: Transaction fails immediately
**Solution**:
- Make sure Tim's account is selected in account dropdown
- Verify account is unlocked (no lock icon)

#### D. Invalid Recipient Address
**Symptom**: Error mentions "invalid address"
**Solution**:
- Verify Bob's address is correct (0x...)
- Use paste button to avoid typos
- Address should be 42 characters (0x + 40 hex)

### 3. Manual Balance Check

You can manually verify balances using the test script:

```bash
cargo run --example test_clipboard
```

Or check directly on block explorer:
- PulseChain Testnet: https://scan.v4.testnet.pulsechain.com/

### 4. Test Transaction Flow

1. **Verify Tim has tPLS**:
   - Select Tim's account
   - Check balance shows > 0 tPLS
   - Click "Refresh" if balance shows 0

2. **Copy Bob's Address**:
   - Select Bob's account
   - Click his address to copy
   - Switch back to Tim's account

3. **Send Transaction**:
   - Paste Bob's address in "To Address" field
   - Enter amount: `1`
   - Click "Send"
   - Check console for detailed logs

### 5. Expected Console Output (Success)

```
📋 Paste address button clicked - attempting to read clipboard
📋 Successfully read from clipboard: 0x...
📋 Sending SendToAddressChanged message with: 0x...
📝 Transaction form submitted - initiating gas estimation
🔍 Starting industry-standard gas estimation
📊 Parameters: to=0x..., amount=1, from=0x..., token=None
💰 Estimating gas for native ETH transfer: 0x... → 0x...
💰 ETH amount conversion: 1 → 1000000000000000000 wei
🔧 Transaction request: from=0x..., to=0x..., value=1000000000000000000, data=None
✅ Raw gas estimate: 21000
🎯 Final gas estimate: 27300 (raw: 21000, buffer: 30%)
✅ Gas estimation successful: 27300 gas
```

### 6. Expected Console Output (Balance Error)

```
📝 Transaction form submitted - initiating gas estimation
🔍 Starting industry-standard gas estimation
📊 Parameters: to=0x..., amount=1, from=0x..., token=None
💰 Estimating gas for native ETH transfer: 0x... → 0x...
❌ Gas estimation failed: insufficient funds for gas * price + value
❌ Error details: RpcError { ... }
💰 Balance check failed - this may indicate:
   1. Insufficient native token (tPLS) for gas fees
   2. Insufficient token balance for transfer
   3. RPC provider unable to read balance
   From address: 0x...
   To address: 0x...
   Amount: 1.0
```

## Quick Fixes

### Fix 1: Get Testnet tPLS
If Tim has 0 tPLS:
1. Go to PulseChain testnet faucet
2. Enter Tim's address
3. Request testnet tPLS
4. Wait for confirmation
5. Click "Refresh" in wallet

### Fix 2: Check Network Selection
Make sure you're on PulseChain Testnet (943):
1. Check network dropdown at top
2. Should show "PulseChain Testnet" or similar
3. If not, select correct network

### Fix 3: Verify Account Selection
Make sure Tim is selected as sender:
1. Check account dropdown shows Tim
2. If not, select Tim from dropdown
3. Verify his address shows at top

### Fix 4: Lower Amount
If balance is low, try sending less:
1. Instead of 1 tPLS, try 0.1 tPLS
2. This leaves more for gas fees
3. Gas costs ~0.0005 tPLS typically

## Files Modified

### `src/gui/simple_transaction.rs`
Added detailed error logging for balance-related failures:
- Full error details logged
- Balance check detection
- Diagnostic information (from, to, amount, token)
- Helpful troubleshooting hints

## Next Steps

1. **Close the running wallet** (if still open)
2. **Rebuild**: `cargo build --bin vaughan`
3. **Run with logs**: `cargo run --bin vaughan`
4. **Try transaction again**
5. **Copy console output** and share if still failing

## Related Issues
- Clipboard paste button fix (completed)
- Phase E controller integration (in progress)

## Status
🔧 **DIAGNOSTIC LOGGING ADDED** - Ready for testing

## Date
January 28, 2026
