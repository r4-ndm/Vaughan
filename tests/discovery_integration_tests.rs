
use vaughan::wallet::account_manager::discovery::{
    AccountDiscovery, DiscoveredAccount, DiscoveryClient, DiscoveryConfig,
};
use vaughan::error::account::AccountResult;
use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::str::FromStr;

/// Comprehensive Mock Provider for Integration Tests
struct MockIntegrationProvider {
    balances: Mutex<HashMap<Address, U256>>,
    nonces: Mutex<HashMap<Address, u64>>,
    chain_id: u64,
    should_fail_balance: Mutex<Option<Address>>, // Address to fail on
}

impl MockIntegrationProvider {
    fn new(chain_id: u64) -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
            nonces: Mutex::new(HashMap::new()),
            chain_id,
            should_fail_balance: Mutex::new(None),
        }
    }

    fn set_balance(&self, address: Address, balance: U256) {
        self.balances.lock().unwrap().insert(address, balance);
    }

    fn set_nonce(&self, address: Address, nonce: u64) {
        self.nonces.lock().unwrap().insert(address, nonce);
    }

    fn set_failure(&self, address: Address) {
        *self.should_fail_balance.lock().unwrap() = Some(address);
    }
}

#[async_trait]
impl DiscoveryClient for MockIntegrationProvider {
    async fn get_balance(&self, address: Address) -> AccountResult<U256> {
        // Simulate network failure
        if let Some(fail_addr) = *self.should_fail_balance.lock().unwrap() {
            if fail_addr == address {
                return Err(vaughan::error::account::AccountError::operation_failed("get_balance", "Simulated network error"));
            }
        }
        Ok(self.balances.lock().unwrap().get(&address).cloned().unwrap_or(U256::ZERO))
    }

    async fn get_transaction_count(&self, address: Address) -> AccountResult<u64> {
        Ok(self.nonces.lock().unwrap().get(&address).cloned().unwrap_or(0))
    }

    async fn call(&self, _tx: &TransactionRequest) -> AccountResult<Bytes> {
        Ok(Bytes::new())
    }

    async fn get_chain_id(&self) -> AccountResult<u64> {
        Ok(self.chain_id)
    }
}

#[tokio::test]
async fn test_discovery_full_integration_flow() {
    // 1. Setup Mock Provider (Force parallel path by using unknown chain ID to avoid Multicall3 complexity in mock)
    // Using chain ID 99999 ensures is_multicall3_supported returns false
    let provider = Arc::new(MockIntegrationProvider::new(99999));

    // 2. Setup Test Data
    let seed = "test test test test test test test test test test test junk";
    
    // Derive expected addresses dynamically using the same logic as the implementation
    use alloy::signers::local::MnemonicBuilder;
    use coins_bip39::English;
    
    let derive = |index: u32| -> Address {
        MnemonicBuilder::<English>::default()
            .phrase(seed)
            .derivation_path(&format!("m/44'/60'/0'/0/{}", index))
            .unwrap()
            .build()
            .unwrap()
            .address()
    };
    
    let addr0 = derive(0);
    let addr2 = derive(2);
    let addr5 = derive(5);

    // Set activity
    provider.set_balance(addr0, U256::from(100)); // Index 0 active via balance
    provider.set_nonce(addr2, 5);                 // Index 2 active via nonce
    provider.set_balance(addr5, U256::from(50));  // Index 5 active via balance

    // 3. Initialize Service
    let service = AccountDiscovery::new(provider);
    let config = DiscoveryConfig {
        gap_limit: 5, // Gap limit of 5. Should find index 5, then scan optionally up to 5 + 5 = 10 calls.
        batch_size: 4,
        ..Default::default()
    };

    // 4. Execute Discovery
    let result = service.discover_accounts(seed, config).await.expect("Discovery failed");

    // 5. Assertions
    assert_eq!(result.len(), 3, "Should have found 3 accounts");
    
    let found_indices: Vec<u32> = result.iter().map(|a| a.index).collect();
    assert!(found_indices.contains(&0));
    assert!(found_indices.contains(&2));
    assert!(found_indices.contains(&5));
    
    // Verify metadata
    let acc0 = result.iter().find(|a| a.index == 0).unwrap();
    assert_eq!(acc0.address, addr0);
    assert_eq!(acc0.balance, U256::from(100));
}

#[tokio::test]
async fn test_discovery_error_propagation() {
    // Test that network errors are propagated correctly
    let provider = Arc::new(MockIntegrationProvider::new(99999));
    let seed = "test test test test test test test test test test test junk";
    let addr0 = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
    
    // Set failure for first address
    provider.set_failure(addr0);

    let service = AccountDiscovery::new(provider);
    let result = service.discover_accounts(seed, DiscoveryConfig::default()).await;

    assert!(result.is_err());
}
