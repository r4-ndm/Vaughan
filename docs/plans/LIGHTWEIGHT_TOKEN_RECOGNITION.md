# Lightweight Token Recognition System
# Inspired by Talisman's Efficient Batch Approach

## Core Principles
- **Minimal code footprint** - no bloat
- **Performance focused** - efficient batch operations  
- **Pro user oriented** - no hand-holding, no risk assessments
- **Simple and fast** - just what advanced users need

## Implementation Plan

### Phase 1: Batch Balance Checker (Week 1)

#### Core Module: `src/tokens/batch_checker.rs`
```rust
use crate::network::NetworkId;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use std::collections::HashMap;

/// Lightweight batch balance checker inspired by Talisman
pub struct BatchBalanceChecker<P> {
    provider: P,
    multicall_address: Address,
    chain_id: u64,
}

impl<P: Provider> BatchBalanceChecker<P> {
    pub fn new(provider: P, multicall_address: Address, chain_id: u64) -> Self {
        Self {
            provider,
            multicall_address,
            chain_id,
        }
    }

    /// Check balances for multiple tokens efficiently using multicall
    pub async fn batch_balances(&self, user_address: Address, token_addresses: Vec<Address>) -> Result<Vec<(Address, U256)>, Error> {
        if token_addresses.is_empty() {
            return Ok(vec![]);
        }

        // Split into batches to avoid gas limits
        const BATCH_SIZE: usize = 100;
        let mut all_results = Vec::new();

        for chunk in token_addresses.chunks(BATCH_SIZE) {
            let batch_results = self.multicall_batch(user_address, chunk.to_vec()).await?;
            all_results.extend(batch_results);
            
            // Small delay between batches to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(all_results)
    }

    /// Single batch multicall execution
    async fn multicall_batch(&self, user_address: Address, token_addresses: Vec<Address>) -> Result<Vec<(Address, U256)>, Error> {
        let calls = token_addresses.iter().map(|&token_addr| {
            alloy::json_abi::Call {
                to: token_addr,
                data: alloy::dyn_abi::DynAbi::encode_function_call("balanceOf", &[alloy::dyn_abi::DynAbi::Token::Address(user_address)])
                    .map_err(|e| Error::Abi(e))?,
            }
        }).collect::<Vec<_>>();

        let multicall_calldata = alloy::json_abi::Call {
            to: self.multicall_address,
            data: alloy::dyn_abi::DynAbi::encode_function_call(
                "aggregate", 
                &[alloy::dyn_abi::DynAbi::Token::Array(calls)]
            ).map_err(|e| Error::Abi(e))?,
        };

        // Execute multicall
        let result = self.provider.call(&multicall_calldata).await?;
        
        // Decode results
        let decoded = alloy::dyn_abi::DynAbi::decode_function_return("aggregate", &result.data)
            .map_err(|e| Error::Abi(e))?;
        
        if let alloy::dyn_abi::DynAbi::Token::Array(results) = decoded.get(0).unwrap() {
            let mut balances = Vec::new();
            
            for (i, token_addr) in token_addresses.iter().enumerate() {
                if let Some(result_data) = results.get(i) {
                    if let alloy::dyn_abi::DynAbi::Token::Bool(success) = result_data.get(0).unwrap() {
                        if *success {
                            if let alloy::dyn_abi::DynAbi::Token::Bytes(return_data) = result_data.get(1).unwrap() {
                                // Decode balance from return_data
                                let balance = U256::from_be_bytes(return_data.as_ref().try_into().unwrap_or([0u8; 32]));
                                balances.push((*token_addr, balance));
                            }
                        }
                    }
                }
            }
            
            Ok(balances)
        } else {
            Err(Error::Decoding("Failed to decode multicall results".into()))
        }
    }

    /// Discover tokens with non-zero balances from known list
    pub async fn discover_token_balances(&self, user_address: Address) -> Result<Vec<(Address, U256)>, Error> {
        // Get known token addresses for this chain
        let token_addresses = self.get_known_token_addresses().await?;
        
        // Check all balances in batches
        let all_balances = self.batch_balances(user_address, token_addresses).await?;
        
        // Filter for non-zero balances
        Ok(all_balances.into_iter().filter(|(_, balance)| *balance > U256::ZERO).collect())
    }

    /// Get known token addresses for current network
    async fn get_known_token_addresses(&self) -> Result<Vec<Address>, Error> {
        // This would integrate with existing token list system
        let token_manager = crate::tokens::TokenManager::new();
        let tokens = token_manager.get_tokens_for_network(NetworkId(self.chain_id));
        Ok(tokens.into_iter().map(|token| token.address).collect())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Provider error: {0}")]
    Provider(#[from] alloy::providers::RpcError),
    
    #[error("ABI encoding/decoding error: {0}")]
    Abi(#[from] alloy::dyn_abi::Error),
    
    #[error("Decoding error: {0}")]
    Decoding(String),
    
    #[error("Timeout error")]
    Timeout,
}
```

### Phase 2: On-Chain Metadata Fetcher (Week 2)

#### Simple Metadata Module: `src/tokens/metadata_fetcher.rs`
```rust
use alloy::primitives::Address;
use alloy::providers::Provider;
use std::collections::HashMap;

/// Lightweight on-chain metadata fetcher
pub struct MetadataFetcher<P> {
    provider: P,
    multicall_address: Address,
    cache: HashMap<Address, TokenMetadata>,
}

#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

impl<P: Provider> MetadataFetcher<P> {
    pub fn new(provider: P, multicall_address: Address) -> Self {
        Self {
            provider,
            multicall_address,
            cache: HashMap::new(),
        }
    }

    /// Fetch metadata for multiple tokens in one call
    pub async fn batch_fetch_metadata(&mut self, addresses: Vec<Address>) -> Result<Vec<(Address, TokenMetadata)>, Error> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }

        // Check cache first
        let uncached: Vec<Address> = addresses.iter()
            .filter(|addr| !self.cache.contains_key(addr))
            .copied()
            .collect();

        if uncached.is_empty() {
            // All cached, return from cache
            return Ok(addresses.into_iter()
                .filter_map(|addr| self.cache.get(&addr).map(|meta| (addr, meta.clone())))
                .collect());
        }

        // Fetch uncached metadata
        let new_metadata = self.multicall_metadata(uncached).await?;
        
        // Update cache and build final results
        for (addr, meta) in new_metadata {
            self.cache.insert(addr, meta.clone());
        }

        // Return complete results
        Ok(addresses.into_iter()
            .filter_map(|addr| self.cache.get(&addr).map(|meta| (addr, meta.clone())))
            .collect())
    }

    /// Fetch token metadata using multicall
    async fn multicall_metadata(&self, addresses: Vec<Address>) -> Result<Vec<(Address, TokenMetadata)>, Error> {
        const BATCH_SIZE: usize = 50;
        let mut all_metadata = Vec::new();

        for chunk in addresses.chunks(BATCH_SIZE) {
            let batch_metadata = self.metadata_batch(chunk.to_vec()).await?;
            all_metadata.extend(batch_metadata);
        }

        Ok(all_metadata)
    }

    /// Single batch metadata fetch
    async fn metadata_batch(&self, token_addresses: Vec<Address>) -> Result<Vec<(Address, TokenMetadata)>, Error> {
        let calls = token_addresses.iter().flat_map(|&token_addr| {
            [
                alloy::json_abi::Call {
                    to: token_addr,
                    data: alloy::dyn_abi::DynAbi::encode_function_call("name", &[])
                        .map_err(|e| Error::Abi(e))?,
                },
                alloy::json_abi::Call {
                    to: token_addr,
                    data: alloy::dyn_abi::DynAbi::encode_function_call("symbol", &[])
                        .map_err(|e| Error::Abi(e))?,
                },
                alloy::json_abi::Call {
                    to: token_addr,
                    data: alloy::dyn_abi::DynAbi::encode_function_call("decimals", &[])
                        .map_err(|e| Error::Abi(e))?,
                },
            ]
        }).collect::<Vec<_>>();

        let multicall_calldata = alloy::json_abi::Call {
            to: self.multicall_address,
            data: alloy::dyn_abi::DynAbi::encode_function_call("aggregate", &[alloy::dyn_abi::DynAbi::Token::Array(calls)])
                .map_err(|e| Error::Abi(e))?,
        };

        let result = self.provider.call(&multicall_calldata).await?;
        let decoded = alloy::dyn_abi::DynAbi::decode_function_return("aggregate", &result.data)
            .map_err(|e| Error::Abi(e))?;

        if let alloy::dyn_abi::DynAbi::Token::Array(results) = decoded.get(0).unwrap() {
            let mut metadata = Vec::new();
            
            for chunk in token_addresses.chunks(1).zip(results.chunks(3)) {
                if let Some(token_addr) = chunk.get(0) {
                    let name = Self::extract_string(&results[0]).unwrap_or_default();
                    let symbol = Self::extract_string(&results[1]).unwrap_or_default();
                    let decimals = Self::extract_u8(&results[2]).unwrap_or(18);
                    
                    metadata.push((*token_addr, TokenMetadata { name, symbol, decimals }));
                }
            }

            Ok(metadata)
        } else {
            Err(Error::Decoding("Failed to decode metadata results".into()))
        }
    }

    fn extract_string(token: &alloy::dyn_abi::DynAbi::Token) -> Option<String> {
        match token {
            alloy::dyn_abi::DynAbi::Token::String(s) => Some(s.clone()),
            alloy::dyn_abi::DynAbi::Token::Bytes(b) => String::from_utf8(b.as_ref().to_vec()).ok(),
            _ => None,
        }
    }

    fn extract_u8(token: &alloy::dyn_abi::DynAbi::Token) -> Option<u8> {
        match token {
            alloy::dyn_abi::DynAbi::Token::Uint(u) => Some(u.to::<u8>()),
            _ => None,
        }
    }
}
```

### Phase 3: Simple Token Discovery (Week 3)

#### Integration Module: `src/tokens/discovery.rs`
```rust
use crate::tokens::{batch_checker::BatchBalanceChecker, metadata_fetcher::MetadataFetcher};
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;

/// Simple token discovery for advanced users
pub struct TokenDiscovery<P> {
    batch_checker: BatchBalanceChecker<P>,
    metadata_fetcher: MetadataFetcher<P>,
}

impl<P: Provider + Clone> TokenDiscovery<P> {
    pub fn new(provider: P, multicall_address: Address, chain_id: u64) -> Self {
        let batch_checker = BatchBalanceChecker::new(provider.clone(), multicall_address, chain_id);
        let metadata_fetcher = MetadataFetcher::new(provider, multicall_address);
        
        Self {
            batch_checker,
            metadata_fetcher,
        }
    }

    /// Discover all tokens with balances for a user address
    pub async fn discover_user_tokens(&mut self, user_address: Address) -> Result<Vec<DiscoveredToken>, Error> {
        // Step 1: Get all token balances
        let balances = self.batch_checker.discover_token_balances(user_address).await?;
        
        if balances.is_empty() {
            return Ok(vec![]);
        }

        // Step 2: Get token addresses
        let token_addresses: Vec<Address> = balances.iter().map(|(addr, _)| *addr).collect();

        // Step 3: Fetch metadata for all tokens
        let metadata = self.metadata_fetcher.batch_fetch_metadata(token_addresses).await?;

        // Step 4: Combine results
        let mut discovered_tokens = Vec::new();
        for (token_addr, balance) in balances {
            if let Some(meta) = metadata.iter().find(|(addr, _)| *addr == token_addr).map(|(_, meta)| meta) {
                discovered_tokens.push(DiscoveredToken {
                    address: token_addr,
                    balance,
                    name: meta.name.clone(),
                    symbol: meta.symbol.clone(),
                    decimals: meta.decimals,
                });
            }
        }

        Ok(discovered_tokens)
    }

    /// Quick check if user has any token balances
    pub async fn has_token_balances(&self, user_address: Address) -> Result<bool, Error> {
        let balances = self.batch_checker.discover_token_balances(user_address).await?;
        Ok(!balances.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredToken {
    pub address: Address,
    pub balance: U256,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

impl DiscoveredToken {
    pub fn formatted_balance(&self) -> String {
        let balance_str = self.balance.to_string();
        if self.decimals == 0 {
            return balance_str;
        }

        let balance_num: f64 = balance_str.parse().unwrap_or(0.0) / 10_f64.powi(self.decimals as i32);
        
        if balance_num >= 1000.0 {
            format!("{:.2}", balance_num)
        } else {
            format!("{:.6}", balance_num)
        }
    }
}
```

### Phase 4: Integration with Existing System (Week 4)

#### Simple Integration: `src/tokens/mod.rs` additions
```rust
// Add to existing TokenManager
impl TokenManager {
    /// Discover user's tokens using batch operations
    pub async fn discover_user_tokens(&mut self, network_id: NetworkId, user_address: Address) -> Result<Vec<DiscoveredToken>, Error> {
        let multicall_address = self.get_multicall_address(network_id)?;
        let provider = self.get_provider_for_network(network_id)?;
        
        let mut discovery = TokenDiscovery::new(provider, multicall_address, network_id.chain_id());
        let discovered = discovery.discover_user_tokens(user_address).await?;
        
        // Convert existing TokenInfo format to DiscoveredToken
        Ok(discovered)
    }

    /// Get multicall address for network
    fn get_multicall_address(&self, network_id: NetworkId) -> Result<Address, Error> {
        match network_id.chain_id() {
            1 => Ok(Address::from_str("0x5BA1e109517A9Db676D3435833F2FB74ea86faB9")?), // Ethereum
            56 => Ok(Address::from_str("0xcA11bde05977b363a7018c201E3a73A6EcE3C5D")?), // BSC
            137 => Ok(Address::from_str("0x910eFc8Ff6c998353354eE51D7942c27F5A8D1")?), // Polygon
            369 => Ok(Address::from_str("0xcA11bde05977b363a7018c201E3a73A6EcE3C5D")?), // PulseChain (using BSC address)
            _ => Err(Error::NoMulticallForNetwork),
        }
    }
}

// Add to existing TokenInfo
impl TokenInfo {
    /// Create from DiscoveredToken
    pub fn from_discovered(discovered: DiscoveredToken, chain_id: u64) -> Self {
        Self {
            address: discovered.address,
            name: discovered.name,
            symbol: discovered.symbol,
            decimals: discovered.decimals,
            chain_id,
            logo_uri: None,
            tags: vec!["discovered".to_string()],
            is_native: false,
        }
    }
}
```

## Minimal UI Integration

### Add to Working Wallet: `src/gui/working_wallet.rs`
```rust
// Add simple auto-discovery on wallet load
impl WorkingWallet {
    pub async fn discover_tokens(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(current_address) = self.get_current_address() {
            let network_id = self.current_network_id();
            
            match self.token_manager.discover_user_tokens(network_id, current_address).await {
                Ok(discovered_tokens) => {
                    for token in discovered_tokens {
                        let token_info = TokenInfo::from_discovered(token, network_id.chain_id());
                        self.token_manager.add_custom_token(network_id, token_info);
                    }
                    
                    // Refresh token list in UI
                    self.refresh_token_list();
                }
                Err(e) => {
                    tracing::error!("Failed to discover tokens: {}", e);
                }
            }
        }
        Ok(())
    }
}
```

## Performance Benefits

1. **Batch Operations**: 100x fewer RPC calls vs individual queries
2. **Multicall Efficiency**: Single call for multiple balance checks
3. **Smart Caching**: Simple in-memory cache for metadata
4. **Parallel Processing**: All operations done concurrently
5. **Rate Limiting**: Built-in delays to avoid hitting limits

## Configuration

```rust
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub batch_size: usize,
    pub enable_on_load: bool,
    pub cache_limit: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            enable_on_load: true,
            cache_limit: 1000,
        }
    }
}
```

## Total New Code

- **BatchBalanceChecker**: ~200 lines
- **MetadataFetcher**: ~180 lines  
- **TokenDiscovery**: ~100 lines
- **Integration**: ~80 lines
- **Total**: ~560 lines (minimal and focused)

## Implementation Timeline

- **Week 1**: BatchBalanceChecker implementation
- **Week 2**: MetadataFetcher with multicall support
- **Week 3**: TokenDiscovery orchestration
- **Week 4**: Integration and testing

This approach gives you the Talisman-inspired efficiency without any bloat. Just fast, lightweight token discovery for advanced users.