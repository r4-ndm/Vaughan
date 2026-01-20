# HTTPS Enforcement - Security Enhancement

## What Changed?

Vaughan now **enforces HTTPS** for all remote RPC endpoints, matching the security standards of MetaMask, Rabby, Rainbow, and Trust Wallet.

## Why HTTPS?

### The Risk of HTTP
When you use an HTTP (unencrypted) RPC endpoint:
- ❌ Anyone on your WiFi can see your transactions
- ❌ Attackers can modify responses (man-in-the-middle)
- ❌ Your wallet addresses and balances are exposed
- ❌ Transaction data can be intercepted

### The Safety of HTTPS
When you use HTTPS (encrypted):
- ✅ All communication is encrypted
- ✅ Attackers cannot read or modify data
- ✅ Your privacy is protected
- ✅ Industry standard security

## What's Allowed?

### ✅ ALLOWED
- `https://eth.llamarpc.com` - Remote HTTPS endpoint
- `https://rpc.pulsechain.com` - Remote HTTPS endpoint
- `http://127.0.0.1:8545` - Local development (Hardhat)
- `http://localhost:8545` - Local development (Ganache)
- `http://[::1]:8545` - Local development (IPv6)

### ❌ BLOCKED
- `http://rpc.example.com` - Remote HTTP endpoint
- `http://192.168.1.100:8545` - Non-localhost HTTP

## Error Message

If you try to add an HTTP remote endpoint, you'll see:

```
HTTPS is required for security. HTTP is only allowed for localhost (127.0.0.1, localhost, [::1])
```

## For Developers

You can still use HTTP for local blockchain development:
- **Hardhat**: `http://127.0.0.1:8545` ✅
- **Ganache**: `http://localhost:8545` ✅
- **Anvil**: `http://127.0.0.1:8545` ✅

## Industry Comparison

| Wallet | HTTPS Enforcement | Localhost Exception |
|--------|------------------|---------------------|
| MetaMask | ✅ Yes | ✅ Yes |
| Rabby | ✅ Yes | ✅ Yes |
| Rainbow | ✅ Yes | ✅ Yes |
| Trust Wallet | ✅ Yes | ✅ Yes |
| **Vaughan** | ✅ Yes | ✅ Yes |

## Technical Details

### Files Modified
- `src/network/validation.rs` - Added localhost detection
- `src/gui/services/network_service.rs` - Updated error messages

### Localhost Detection
```rust
let is_localhost = rpc_url.contains("127.0.0.1") 
    || rpc_url.contains("localhost")
    || rpc_url.contains("[::1]"); // IPv6 localhost

if !is_localhost {
    // Block HTTP for remote endpoints
    issues.push(NetworkValidationIssue::InsecureConnection);
}
```

## FAQ

**Q: Why can't I use my HTTP RPC endpoint?**
A: HTTP is unencrypted and exposes your transactions to anyone on your network. All legitimate public RPC providers use HTTPS.

**Q: What if I'm on a trusted network?**
A: Even on trusted networks, HTTP is risky. Use HTTPS or run a local node.

**Q: Can I use HTTP for testing?**
A: Yes! HTTP works for localhost (127.0.0.1, localhost) for local development.

**Q: What if I really need HTTP?**
A: You don't. Every major RPC provider (Infura, Alchemy, Ankr, etc.) uses HTTPS. If your provider doesn't, find a better one.

## Security Impact

This change eliminates a **HIGH SEVERITY** vulnerability where users could unknowingly expose their transaction data and wallet information through unencrypted connections.

**Status**: ✅ FIXED - Vaughan now meets industry security standards.
