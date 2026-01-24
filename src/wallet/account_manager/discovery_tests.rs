
use super::*;
use alloy::primitives::{Address, Bytes, U256}; 
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use crate::error::account::AccountResult;

/// Mock client for testing
struct MockDiscoveryClient {
    balances: Mutex<HashMap<Address, U256>>,
    nonces: Mutex<HashMap<Address, u64>>,
    chain_id: u64,
}

impl MockDiscoveryClient {
    fn new() -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
            nonces: Mutex::new(HashMap::new()),
            chain_id: 1, // Mainnet where Multicall3 is supported
        }
    }

    fn set_balance(&self, address: Address, balance: U256) {
        self.balances.lock().unwrap().insert(address, balance);
    }

    fn set_nonce(&self, address: Address, nonce: u64) {
        self.nonces.lock().unwrap().insert(address, nonce);
    }
}

#[async_trait]
impl DiscoveryClient for MockDiscoveryClient {
    async fn get_balance(&self, address: Address) -> AccountResult<U256> {
        Ok(self.balances.lock().unwrap().get(&address).cloned().unwrap_or(U256::ZERO))
    }

    async fn get_transaction_count(&self, address: Address) -> AccountResult<u64> {
        Ok(self.nonces.lock().unwrap().get(&address).cloned().unwrap_or(0))
    }

    async fn call(&self, _tx: &TransactionRequest) -> AccountResult<Bytes> {
        // Simple mock returning empty bytes for now.
        Ok(Bytes::new())
    }

    async fn get_chain_id(&self) -> AccountResult<u64> {
        Ok(self.chain_id)
    }
}

#[tokio::test]
async fn test_discovery_stops_at_gap_limit() {
    let mut client_impl = MockDiscoveryClient::new();
    client_impl.chain_id = 999; // Force parallel path
    let mock_client = Arc::new(client_impl);
    
    // Seed: "test test test test test test test test test test test junk"
    let seed = "test test test test test test test test test test test junk";
    // Address 0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    // Address 2: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
    let addr0: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
    let addr2: Address = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".parse().unwrap();
    
    // Set activity at index 0 and 2. Index 1 is empty.
    mock_client.set_balance(addr0, U256::from(100));
    mock_client.set_nonce(addr2, 5);
    
    let service = AccountDiscovery::new(mock_client);
    let config = DiscoveryConfig {
        gap_limit: 5, 
        batch_size: 10,
        concurrency_limit: 1,
        ..Default::default()
    };
    
    let result = service.discover_accounts(seed, config).await.unwrap();
    
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].index, 0);
    assert!(result[0].active);
    assert_eq!(result[1].index, 2);
    assert!(result[1].active);
}

#[tokio::test]
async fn test_discovery_gap_limit_enforcement() {
    let mut client_impl = MockDiscoveryClient::new();
    client_impl.chain_id = 999;
    let mock_client = Arc::new(client_impl);
    
    let seed = "test test test test test test test test test test test junk";
    let addr0: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
    mock_client.set_balance(addr0, U256::from(1));
    
    // Gap limit 2. 
    // Batch 1: [0, 1] -> 0 active, 1 inactive (gap=1).
    // Batch 2: [2, 3] -> 2 inactive (gap=2), 3 inactive (gap=3). stop.
    // Result: [0].
    
    let service = AccountDiscovery::new(mock_client);
    let config = DiscoveryConfig {
        gap_limit: 2,
        batch_size: 2,
        ..Default::default()
    };
    
    let result = service.discover_accounts(seed, config).await.unwrap();
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].index, 0);
}

#[tokio::test]
async fn test_discovery_activity_detection() {
    let mut client_impl = MockDiscoveryClient::new();
    client_impl.chain_id = 999;
    let mock_client = Arc::new(client_impl);
    
    let seed = "test test test test test test test test test test test junk";
    let addr0: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
    let addr1: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse().unwrap();
    
    // 0 has balance
    mock_client.set_balance(addr0, U256::from(100));
    // 1 has nonce (used but empty balance)
    mock_client.set_nonce(addr1, 10);
    
    let service = AccountDiscovery::new(mock_client);
    let result = service.discover_accounts(seed, DiscoveryConfig::default()).await.unwrap();
    
    assert!(result.iter().any(|a| a.index == 0 && a.active));
    assert!(result.iter().any(|a| a.index == 1 && a.active));
}
