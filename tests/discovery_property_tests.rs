
use vaughan::wallet::account_manager::discovery::{
    AccountDiscovery, DiscoveredAccount, DiscoveryClient, DiscoveryConfig,
};
use vaughan::error::account::AccountResult;
use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::MnemonicBuilder;
use coins_bip39::English;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use proptest::prelude::*;

// --- Mock Provider (Simplified for Property Tests) ---
struct ProptestProvider {
    active_indices: HashSet<u32>,
    chain_id: u64,
    seed: String,
}

impl ProptestProvider {
    fn new(active_indices: HashSet<u32>, seed: String) -> Self {
        Self {
            active_indices,
            chain_id: 1,
            seed,
        }
    }

    fn is_index_active(&self, address: Address) -> bool {
        // Reverse lookup could be slow, so we derive and check.
        // But for proptest, maybe we pre-calculate map?
        // Let's derive on the fly? No, that's slow.
        // We need a map Address -> bool.
        false
    }
}

// Better Mock for Proptest: Address Map
struct fastIndexProvider {
    active_addresses: HashSet<Address>,
}

#[async_trait]
impl DiscoveryClient for fastIndexProvider {
    async fn get_balance(&self, address: Address) -> AccountResult<U256> {
        if self.active_addresses.contains(&address) {
            Ok(U256::from(100))
        } else {
            Ok(U256::ZERO)
        }
    }

    async fn get_transaction_count(&self, _address: Address) -> AccountResult<u64> {
        Ok(0)
    }

    async fn call(&self, _tx: &TransactionRequest) -> AccountResult<Bytes> {
        Ok(Bytes::new())
    }

    async fn get_chain_id(&self) -> AccountResult<u64> {
        Ok(1)
    }
}

// Helper to derive address from index
fn derive_address(seed: &str, index: u32) -> Address {
    MnemonicBuilder::<English>::default()
        .phrase(seed)
        .derivation_path(&format!("m/44'/60'/0'/0/{}", index))
        .unwrap()
        .build()
        .unwrap()
        .address()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))] // 20 cases is enough for async heavy tests

    #[test]
    fn prop_discovery_finds_all_active_before_gap(
        // Generate a set of active indices between 0 and 100
        active_indices in proptest::collection::hash_set(0u32..100, 0..15),
        gap_limit in 5usize..20usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let seed = "test test test test test test test test test test test junk";
            
            // 1. Calculate Expected max index
            // Sort indices
            let mut sorted_indices: Vec<u32> = active_indices.iter().cloned().collect();
            sorted_indices.sort();
            
            // Calculate which indices should be found based on gap limit
            let mut expected_indices = Vec::new();
            let mut last_found_index = -1i64;
            
            // Also need to include index 0? The loop starts at 0.
            // Wait, discovery scans 0, 1, 2...
            // If 0 is active, gap reset.
            // If 0 inactive, gap increment.
            
            let mut current_scan = 0u32;
            let mut gap = 0;
            
            while gap < gap_limit {
                if active_indices.contains(&current_scan) {
                    expected_indices.push(current_scan);
                    gap = 0;
                } else {
                    gap += 1;
                }
                current_scan += 1;
                if current_scan > 200 { break; } // Safety
            }
            
            // 2. Setup Provider
            let mut active_addresses = HashSet::new();
            for &idx in &active_indices {
                active_addresses.insert(derive_address(seed, idx));
            }
            
            let provider = Arc::new(fastIndexProvider { active_addresses });
            let service = AccountDiscovery::new(provider);
            
            let config = DiscoveryConfig {
                gap_limit,
                batch_size: 10,
                ..Default::default()
            };
            
            // 3. Run Discovery
            let result = service.discover_accounts(seed, config).await.unwrap();
            let found_indices: Vec<u32> = result.iter().map(|a| a.index).collect();
            
            // 4. Verify
            // found_indices should exactly match expected_indices
            assert_eq!(found_indices, expected_indices, "Discovery result mismatch. Expected {:?}, got {:?}", expected_indices, found_indices);
        });
    }
}
