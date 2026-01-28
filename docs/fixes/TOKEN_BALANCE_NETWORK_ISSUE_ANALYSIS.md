# Token Balance Network Issue - Root Cause Analysis

## Issue Description
Token balances only update correctly on PulseChain Testnet v4 (Chain ID: 943). Other networks show incorrect or zero balances for ERC20 tokens.

## Root Cause
The `update_token_list_for_network()` function in `src/gui/working_wallet.rs` (line 3433) contains incorrect or missing ERC20 token contract addresses for most networks.

## Current State Analysis

### Working Network
**PulseChain Testnet v4 (943)** ✅
- USD: `0x3e0Ad60c6D427191D66B6D168ddeF82A66F573B0`
- WPLS: `0xcF1Fc503CA35618E9b4C08b7847980b3e10FB53B`
- **Status**: Correct addresses, balances work

### Broken Networks

#### Ethereum Mainnet (1) ❌
**Current addresses (INCORRECT)**:
- USDC: `0xA0b86a33E6443B9e3d5e563C780384aA470A37d2` ❌
- USDT: `0xdAC17F958D2ee523a2206206994597C13D831ec7` ✅
- WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` ✅

**Correct addresses**:
- USDC: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- USDT: `0xdAC17F958D2ee523a2206206994597C13D831ec7`
- WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`

#### PulseChain Mainnet (369) ❌
**Current addresses (INCORRECT - using Ethereum addresses)**:
- WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` ❌
- USDC: `0xA0b86a33E6443B9e3d5e563C780384aA470A37d2` ❌
- WPLS: `0xA1077a294dDE1B09bB078844df40758a5D0f9a27` ❓

**Issue**: PulseChain has its own token ecosystem, not Ethereum tokens

#### BSC (56), Polygon (137), Arbitrum (42161), Optimism (10) ❌
**Current addresses**: NONE (empty vec)
**Status**: No token addresses configured at all

## Technical Flow

### How Token Balances Are Fetched

1. **Network Switch** (`handle_network_selected` in `network.rs`):
   ```rust
   self.update_token_list_for_network(network_id);
   ```

2. **Balance Refresh** (`handle_balance_refreshed` in `wallet_ops.rs` line 560-595):
   ```rust
   for token in &tokens_to_update {
       if let Some(contract_address) = token.contract_address {
           wallet_read.get_balance(Some(contract_address)).await
       }
   }
   ```

3. **ERC20 Balance Call** (`NetworkManager::get_balance` in `network/mod.rs` line 550-580):
   ```rust
   // Creates balanceOf(address) call
   let call_data = [0x70, 0xa0, 0x82, 0x31] + padded_address
   provider.call(call_request).await
   ```

### Why PulseChain Testnet v4 Works
- Has correct contract addresses for its tokens
- RPC endpoint responds correctly to `eth_call` for `balanceOf`
- Token contracts exist and are deployed at those addresses

### Why Other Networks Fail
1. **Wrong addresses**: Calling `balanceOf` on non-existent or wrong contracts returns zero
2. **Missing addresses**: No contract address means no balance fetch attempted
3. **Cross-chain confusion**: Using Ethereum addresses on PulseChain won't work

## Solution Options

### Option A: Use Token Lists (RECOMMENDED) ⭐
**Approach**: Integrate with standard token lists (Uniswap, CoinGecko, etc.)
- **Pros**: 
  - Accurate, community-maintained addresses
  - Supports thousands of tokens
  - Industry standard (MetaMask uses this)
- **Cons**: 
  - Requires external dependency
  - Need to handle list updates
- **Time**: 2-3 hours
- **Files**: 
  - `src/tokens/lists.rs` (already exists!)
  - `src/gui/working_wallet.rs`

### Option B: Manual Correction (QUICK FIX)
**Approach**: Manually fix the addresses in `update_token_list_for_network`
- **Pros**: 
  - Quick fix (30 minutes)
  - No new dependencies
- **Cons**: 
  - Limited to hardcoded tokens
  - Requires manual maintenance
  - Not scalable
- **Time**: 30 minutes
- **Files**: 
  - `src/gui/working_wallet.rs` only

### Option C: Hybrid Approach (BALANCED)
**Approach**: Fix critical tokens manually + add token list support
- **Pros**: 
  - Immediate fix for common tokens
  - Scalable for future
  - Best of both worlds
- **Cons**: 
  - More work upfront
- **Time**: 1-2 hours
- **Files**: 
  - `src/gui/working_wallet.rs`
  - `src/tokens/lists.rs`

## Recommended Action

**Do Option B (Quick Fix) NOW, then Option A later**

### Immediate Fix (30 minutes)
1. Fix Ethereum USDC address
2. Remove incorrect PulseChain mainnet addresses (or find correct ones)
3. Add basic BSC/Polygon token addresses
4. Test on each network

### Future Enhancement (Phase F or later)
1. Integrate token list standard
2. Add token discovery
3. Support custom token addition per network

## Verification Steps

After fix:
1. Switch to Ethereum Mainnet → Check USDC balance
2. Switch to BSC → Check USDT balance  
3. Switch to Polygon → Check USDC balance
4. Switch to PulseChain Testnet v4 → Verify still works
5. Check logs for any RPC errors

## Files to Modify

### Quick Fix (Option B)
- `src/gui/working_wallet.rs` (line 3520-3560)

### Full Solution (Option A)
- `src/tokens/lists.rs` (implement token list fetching)
- `src/gui/working_wallet.rs` (integrate token lists)
- `src/gui/handlers/network.rs` (trigger token list update on network switch)

## References
- Ethereum Token Lists: https://tokenlists.org/
- Uniswap Default List: https://tokens.uniswap.org/
- CoinGecko Token List: https://tokens.coingecko.com/
- MetaMask Token Detection: https://github.com/MetaMask/metamask-extension/tree/develop/app/scripts/controllers/detect-tokens

## Professional Standards Note
Following MetaMask pattern: They use token lists + user-added tokens. We should do the same for production-ready wallet.
