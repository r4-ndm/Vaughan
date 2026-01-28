# Token Balance Empty Array - Explained

## Log Message Observed
```
✅ Received updated token balances: []
```

## Why This Happened

### The Problem
Token addresses were defined in TWO separate locations in the code:

1. **Location 1**: `tokens_with_addresses` in `update_token_list_for_network()` (line ~3520)
   - Purpose: Display tokens in UI picker
   - Status: Fixed in first pass

2. **Location 2**: `initialize_token_balances_for_network()` (line ~3600)
   - Purpose: **Actually fetch token balances**
   - Status: Had wrong addresses (missed in first pass)

### The Flow

```
Network Switch
    ↓
update_token_list_for_network() called
    ↓
initialize_token_balances_for_network() called
    ↓
Populates self.state.token_balances with tokens
    ↓
handle_balance_refreshed() iterates over token_balances
    ↓
For each token with contract_address, calls wallet.get_balance()
    ↓
Results sent to TokenBalancesRefreshed message
```

### Why Array Was Empty

On PulseChain Testnet v4:
- `initialize_token_balances_for_network()` had correct addresses ✅
- `token_balances` was populated with USD and WPLS
- Balance fetching worked

On other networks (before second fix):
- `initialize_token_balances_for_network()` had wrong/missing addresses ❌
- `token_balances` was populated but with wrong addresses
- Balance fetching failed or returned zero
- Empty results array

### The Fix

Fixed addresses in BOTH locations:
1. ✅ `tokens_with_addresses` - for UI display
2. ✅ `initialize_token_balances_for_network()` - for balance fetching

Now when you switch networks:
- Correct tokens are added to `token_balances`
- Balance fetching uses correct contract addresses
- Results array contains actual balances

## Expected Logs After Fix

### PulseChain Testnet v4
```
✅ Initialized token_balances with 2 popular tokens for network 943
🪙 Fetching ERC20 token balance for 0x... on token 0x3e0Ad60c6D427191D66B6D168ddeF82A66F573B0
✅ Successfully fetched ERC20 balance: 1000000 tokens
✅ Received updated token balances: [("USD", "1.000000"), ("WPLS", "0.500000")]
```

### Ethereum Mainnet (After Fix)
```
✅ Initialized token_balances with 4 popular tokens for network 1
🪙 Fetching ERC20 token balance for 0x... on token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
✅ Successfully fetched ERC20 balance: 1000000 tokens
✅ Received updated token balances: [("USDC", "1.000000"), ("USDT", "0.000000"), ...]
```

### BSC (After Fix)
```
✅ Initialized token_balances with 3 popular tokens for network 56
🪙 Fetching ERC20 token balance for 0x... on token 0x55d398326f99059fF775485246999027B3197955
✅ Successfully fetched ERC20 balance: 5000000 tokens
✅ Received updated token balances: [("USDT", "5.000000"), ("BUSD", "0.000000"), ...]
```

## Code Locations

### Location 1: UI Display (line ~3520)
```rust
let tokens_with_addresses = match network_id.0 {
    1 => vec![
        ("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // ✅ Fixed
        // ...
    ],
    // ...
};
```

### Location 2: Balance Fetching (line ~3600)
```rust
pub fn initialize_token_balances_for_network(&mut self, network_id: NetworkId) {
    let tokens = match network_id.0 {
        1 => vec![
            ("USDC", "USD Coin", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", 6), // ✅ Fixed
            // ...
        ],
        // ...
    };
    
    for (symbol, name, address_str, decimals) in tokens {
        self.state.token_balances.push(SimpleTokenBalance {
            symbol: symbol.to_string(),
            name: name.to_string(),
            contract_address: Some(address),
            balance: "0.0000".to_string(),
            decimals,
        });
    }
}
```

## Lesson Learned

When fixing token addresses, always check for:
1. Display/UI token lists
2. Balance fetching token lists
3. Any other token configuration locations

Use grep to find all occurrences:
```bash
grep -r "0xA0b86a33E6443B9e3d5e563C780384aA470A37d2" .
```

## Professional Standards

Following MetaMask pattern:
- Single source of truth for token addresses
- Consider refactoring to eliminate duplication
- Use constants or configuration files for token lists

## Future Improvement

Refactor to use a single token configuration:
```rust
struct TokenConfig {
    symbol: &'static str,
    name: &'static str,
    address: &'static str,
    decimals: u8,
}

const ETHEREUM_TOKENS: &[TokenConfig] = &[
    TokenConfig {
        symbol: "USDC",
        name: "USD Coin",
        address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        decimals: 6,
    },
    // ...
];
```

This eliminates duplication and ensures consistency.
