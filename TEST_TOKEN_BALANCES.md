# Token Balance Testing Guide

## Quick Test Checklist

### Before Testing
- [ ] Build completed successfully: `cargo build --release`
- [ ] Wallet has accounts on multiple networks
- [ ] Have some tokens to test with (or check zero balances display correctly)

### Test 1: Ethereum Mainnet
1. [ ] Launch Vaughan wallet
2. [ ] Switch to "Ethereum Mainnet" network
3. [ ] Check native ETH balance displays
4. [ ] Check USDC balance displays (should not be "Error")
5. [ ] Check USDT balance displays
6. [ ] Check WETH balance displays
7. [ ] Look for log: `✅ Successfully fetched ERC20 balance`

**Expected**: All balances show numbers (even if zero), no "Error" messages

### Test 2: BSC (Binance Smart Chain)
1. [ ] Switch to "Binance Smart Chain" network
2. [ ] Check native BNB balance displays
3. [ ] Check USDT balance displays
4. [ ] Check BUSD balance displays
5. [ ] Check CAKE balance displays
6. [ ] Look for log: `✅ Successfully fetched ERC20 balance`

**Expected**: All balances show numbers, no "Error" messages

### Test 3: Polygon
1. [ ] Switch to "Polygon" network
2. [ ] Check native MATIC balance displays
3. [ ] Check USDC balance displays
4. [ ] Check USDT balance displays
5. [ ] Check WETH balance displays
6. [ ] Look for log: `✅ Successfully fetched ERC20 balance`

**Expected**: All balances show numbers, no "Error" messages

### Test 4: PulseChain Testnet v4 (Regression)
1. [ ] Switch to "PulseChain Testnet v4" network
2. [ ] Check native tPLS balance displays
3. [ ] Check USD balance displays
4. [ ] Check WPLS balance displays
5. [ ] Verify no regression from previous working state

**Expected**: Everything still works as before

### Test 5: Network Switching
1. [ ] Switch between networks multiple times
2. [ ] Verify balances update correctly each time
3. [ ] Check no memory leaks or slowdowns
4. [ ] Verify token list updates for each network

**Expected**: Smooth switching, correct balances on each network

## What to Look For

### ✅ Success Indicators
- Balance numbers display (even if zero)
- No "Error loading balance" messages
- Logs show: `✅ Successfully fetched ERC20 balance`
- Logs show: `✅ Successfully fetched balance: X wei`
- Token symbols match network (ETH on Ethereum, BNB on BSC, etc.)

### ❌ Failure Indicators
- "Error loading balance" displayed
- Logs show: `❌ ERC20 balance call failed`
- Logs show: `❌ Failed to fetch balance`
- Balances stuck at zero when they shouldn't be
- Wrong token symbols for network

## Debugging

### If Balances Show "Error"
1. Check logs for RPC errors
2. Verify network RPC endpoint is responding
3. Check if token contract address is correct
4. Try switching networks and back

### If Balances Show Zero (But Shouldn't)
1. Verify you actually have tokens on that network
2. Check the account address is correct
3. Verify on block explorer (Etherscan, BSCScan, etc.)
4. Check logs for successful balance fetch

### If Logs Show RPC Errors
1. Check internet connection
2. Try different RPC endpoint
3. Check if network is congested
4. Wait and retry

## Log Examples

### Good Logs
```
🔍 Fetching native balance for 0x... on Ethereum Mainnet (Chain ID: 1)
✅ Successfully fetched balance: 1000000000000000000 wei (1.000000 ETH)
🪙 Fetching ERC20 token balance for 0x... on token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 (Ethereum Mainnet)
✅ Successfully fetched ERC20 balance: 1000000 tokens
```

### Bad Logs
```
❌ ERC20 balance call failed: contract not found
❌ Failed to fetch balance after 3 attempts: timeout
⚠️ Balance fetch attempt 3/3 failed: RPC error
```

## Network-Specific Notes

### Ethereum Mainnet
- May be slower due to network congestion
- Gas prices higher, but doesn't affect balance fetching
- USDC is most common token to test

### BSC
- Usually fast responses
- USDT and BUSD are most common
- Lower gas fees than Ethereum

### Polygon
- Fast and cheap
- USDC (PoS) is bridged from Ethereum
- Good for testing

### PulseChain Testnet v4
- Should work exactly as before
- This is our baseline for comparison

## Reporting Issues

If you find issues, report:
1. Which network
2. Which token
3. Expected vs actual balance
4. Relevant log messages
5. Block explorer link showing actual balance

## Success Criteria

✅ **Fix is successful if**:
- Ethereum USDC balance displays correctly
- BSC USDT/BUSD balances display correctly
- Polygon USDC balance displays correctly
- PulseChain Testnet v4 still works
- No new errors in logs

## Time Estimate
- Full test suite: 10-15 minutes
- Quick smoke test: 3-5 minutes
