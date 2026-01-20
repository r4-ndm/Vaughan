# Deploy Token Using Forge (Foundry)

## Why This is Better

Instead of fighting with bytecode in Rust, we can use `forge create` directly:

✅ **Handles all bytecode automatically**  
✅ **Built-in verification**  
✅ **Works with any contract**  
✅ **Simpler and faster**

## Quick Deploy

### 1. Set up your private key (SECURE method)

```bash
# Option A: Use keystore (most secure)
cast wallet import deployer --interactive

# Option B: Use private key directly (for testing only)
export PRIVATE_KEY="your_private_key_here"
```

### 2. Deploy to PulseChain Testnet v4

```bash
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy
```

### 3. Deploy with verification (for supported networks)

```bash
# For Ethereum (with Etherscan)
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY \
  --private-key $PRIVATE_KEY \
  --verify \
  --etherscan-api-key YOUR_ETHERSCAN_KEY

# For BSC
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://bsc-dataseed.binance.org \
  --private-key $PRIVATE_KEY \
  --verify \
  --etherscan-api-key YOUR_BSCSCAN_KEY \
  --verifier-url https://api.bscscan.com/api
```

## Using Wallet from Vaughan

Since you have accounts in Vaughan, you can export the private key:

```bash
# 1. Get your private key from Vaughan (carefully!)
# In Vaughan GUI: Account → Export Private Key

# 2. Set it as environment variable (for one session only)
export PRIVATE_KEY="0x..."

# 3. Deploy
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy

# 4. Clear the variable after deploying
unset PRIVATE_KEY
```

## Safer Method: Use Cast Wallet

```bash
# 1. Import your key once (encrypted)
cast wallet import my-deployer --interactive
# Enter your private key when prompted
# Set a password to encrypt it

# 2. Deploy using the named wallet
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --account my-deployer \
  --sender 0xe3b3f4cE6d66411d4FeDFa2c2864b55C75f2ad8F \
  --legacy
# You'll be prompted for the password
```

## Expected Output

```
[⠊] Compiling...
[⠒] Compiling 1 files with 0.8.26
[⠢] Solc 0.8.26 finished in 324.51ms
Compiler run successful!
Deployer: 0xe3b3f4cE6d66411d4FeDFa2c2864b55C75f2ad8F
Deployed to: 0x1234567890abcdef1234567890abcdef12345678
Transaction hash: 0xabcdef...
```

## Integrate with Rust App (Better Approach)

Instead of handling bytecode in Rust, we can call `forge create` as a subprocess:

```rust
use std::process::Command;

pub async fn deploy_with_forge(
    contract_path: &str,
    rpc_url: &str,
    private_key: &str,
) -> Result<String> {
    let output = Command::new("forge")
        .args(&[
            "create",
            contract_path,
            "--rpc-url", rpc_url,
            "--private-key", private_key,
            "--legacy",
            "--json", // Get JSON output for parsing
        ])
        .output()?;
    
    // Parse JSON output to get contract address
    let stdout = String::from_utf8(output.stdout)?;
    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    
    Ok(json["deployedTo"].as_str().unwrap().to_string())
}
```

## Networks Configuration

### PulseChain Testnet v4
```bash
RPC: https://rpc.v4.testnet.pulsechain.com
Chain ID: 943
Explorer: https://scan.v4.testnet.pulsechain.com
Faucet: https://faucet.v4.testnet.pulsechain.com
```

### PulseChain Mainnet
```bash
RPC: https://rpc.pulsechain.com
Chain ID: 369
Explorer: https://scan.pulsechain.com
```

### Ethereum Sepolia (Testnet)
```bash
RPC: https://ethereum-sepolia.publicnode.com
Chain ID: 11155111
Explorer: https://sepolia.etherscan.io
```

### BSC Testnet
```bash
RPC: https://data-seed-prebsc-1-s1.binance.org:8545
Chain ID: 97
Explorer: https://testnet.bscscan.com
```

## Verify Existing Contract

If you deployed without verification:

```bash
forge verify-contract \
  --chain-id 1 \
  --etherscan-api-key YOUR_KEY \
  CONTRACT_ADDRESS \
  src/SimpleToken.sol:SimpleToken
```

## Create Different Token Types

### Burnable Token

```solidity
// src/BurnableToken.sol
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";

contract BurnableToken is ERC20Burnable {
    constructor() ERC20("Burnable Token", "BURN") {
        _mint(msg.sender, 1000000 * 10**decimals());
    }
}
```

Deploy:
```bash
forge create src/BurnableToken.sol:BurnableToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy
```

### Mintable Token

```solidity
// src/MintableToken.sol
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract MintableToken is ERC20, Ownable {
    constructor() ERC20("Mintable Token", "MINT") Ownable(msg.sender) {
        _mint(msg.sender, 1000000 * 10**decimals());
    }
    
    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}
```

### Pausable Token

```solidity
// src/PausableToken.sol
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Pausable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract PausableToken is ERC20, ERC20Pausable, Ownable {
    constructor() ERC20("Pausable Token", "PAUSE") Ownable(msg.sender) {
        _mint(msg.sender, 1000000 * 10**decimals());
    }
    
    function pause() public onlyOwner {
        _pause();
    }
    
    function unpause() public onlyOwner {
        _unpause();
    }
    
    function _update(address from, address to, uint256 value)
        internal
        override(ERC20, ERC20Pausable)
    {
        super._update(from, to, value);
    }
}
```

## Benefits of Using Forge

1. **No bytecode handling** - forge does it all
2. **Constructor args** - easily pass them
3. **Verification** - built-in support
4. **Testing** - can test before deploying
5. **Gas estimation** - automatic
6. **Multi-chain** - works on any EVM chain

## Try It Now!

```bash
# Quick test deployment
cd /home/r4/Desktop/Vaughan_V1

# Set your private key (CAREFUL!)
export PRIVATE_KEY="0x..."

# Deploy!
forge create src/SimpleToken.sol:SimpleToken \
  --rpc-url https://rpc.v4.testnet.pulsechain.com \
  --private-key $PRIVATE_KEY \
  --legacy

# Clean up
unset PRIVATE_KEY
```

## Integration with Your App

You can integrate this into your Rust app by:

1. **Option A**: Call `forge create` as a subprocess
2. **Option B**: Use forge's JSON output to extract contract address
3. **Option C**: Keep your current Rust implementation but use forge-compiled bytecode

The easiest is Option A - just call forge directly!
