# Constructor Arguments - Quick Reference

## 🚀 Deploy Custom Token in 3 Steps

### Step 1: Set Private Key
```bash
export DEPLOYER_PRIVATE_KEY="0x..."
```

### Step 2: Choose Method

#### Option A: Test Script (Fastest)
```bash
./scripts/test_custom_token.sh
```

#### Option B: Direct CLI
```bash
forge create src/CustomToken.sol:CustomToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $DEPLOYER_PRIVATE_KEY \
  --legacy \
  --constructor-args "Token Name" "SYMBOL" 1000000 18
```

#### Option C: Rust Example
```bash
cargo run --example deploy_custom_token
```

### Step 3: Verify
Check the deployed contract on the block explorer URL shown in output.

---

## 📝 Constructor Arguments Order

| Position | Type | Name | Example |
|----------|------|------|---------|
| 1 | `string` | Token Name | `"My Custom Token"` |
| 2 | `string` | Symbol | `"MCT"` |
| 3 | `uint256` | Initial Supply | `1000000` |
| 4 | `uint8` | Decimals | `18` |

---

## 🌐 Supported Networks

| Network | Chain ID | RPC URL |
|---------|----------|---------|
| Ethereum Mainnet | 1 | `https://eth.llamarpc.com` |
| BSC Mainnet | 56 | `https://bsc-dataseed.binance.org` |
| Polygon | 137 | `https://polygon-rpc.com` |
| PulseChain | 369 | `https://rpc.pulsechain.com` |
| PulseChain Testnet v4 | 943 | `https://rpc.v4.testnet.pulsechain.com` |
| Sepolia | 11155111 | `https://ethereum-sepolia.publicnode.com` |

---

## 💻 Rust Code Snippet

```rust
use vaughan_v1::launcher::forge_deployment::{deploy_with_forge, ForgeDeployConfig};

let config = ForgeDeployConfig {
    contract_path: "src/CustomToken.sol:CustomToken".to_string(),
    rpc_url: "https://rpc.v4.testnet.pulsechain.com".to_string(),
    chain_id: 943,
    private_key: private_key,
    constructor_args: vec![
        "My Token".to_string(),
        "MTK".to_string(),
        "1000000".to_string(),
        "18".to_string(),
    ],
    legacy: true,
    verify: false,
    gas_limit: None,
    gas_price: None,
    etherscan_api_key: None,
};

let result = deploy_with_forge(config).await?;
println!("Deployed at: {}", result.deployed_to);
```

---

## ⚡ Common Issues

| Error | Solution |
|-------|----------|
| "intrinsic gas too low" | Remove `gas_limit` or increase to 500000 |
| "insufficient funds" | Get test tokens from faucet |
| "forge: command not found" | Install Foundry: `curl -L https://foundry.paradigm.xyz \| bash` |
| Wrong token parameters | Check constructor args order matches docs |

---

## 📚 Full Documentation

- Complete Guide: [`docs/custom_token_deployment.md`](./custom_token_deployment.md)
- Implementation Summary: [`docs/CONSTRUCTOR_ARGS_SUMMARY.md`](./CONSTRUCTOR_ARGS_SUMMARY.md)
- Contract Source: [`src/CustomToken.sol`](../src/CustomToken.sol)
- Example Code: [`examples/deploy_custom_token.rs`](../examples/deploy_custom_token.rs)
- Test Script: [`scripts/test_custom_token.sh`](../scripts/test_custom_token.sh)

---

## ✅ Pre-Deployment Checklist

- [ ] Private key environment variable set
- [ ] Sufficient balance on target network
- [ ] Token name, symbol, supply decided
- [ ] Contract compiled (`forge build`)
- [ ] forge installed and in PATH
- [ ] Testing on testnet first

---

## 🎯 What's Different from SimpleToken?

| Feature | SimpleToken | CustomToken |
|---------|-------------|-------------|
| Name | Hardcoded "Simple Token" | Custom via constructor |
| Symbol | Hardcoded "SMPL" | Custom via constructor |
| Supply | Hardcoded 1M tokens | Custom via constructor |
| Decimals | Hardcoded 18 | Custom via constructor |
| Deployment | No args needed | 4 args required |

---

**Need Help?** Check the full documentation or run the test script to verify your setup.
