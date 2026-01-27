
use crate::error::account::{AccountError, AccountResult};
use crate::performance::multicall::{MulticallBuilder, decode_balance_results};
use crate::wallet::hardware::derivation::DerivationStandard;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::MnemonicBuilder;
use async_trait::async_trait;
use futures_util::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::instrument;
use uuid::Uuid;

/// Configuration for account discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Maximum number of consecutive unused accounts to scan before stopping
    /// Default: 20 (BIP-44 standard)
    pub gap_limit: usize,
    /// Batch size for scanning
    /// Default: 10
    pub batch_size: usize,
    /// Derivation standard to use
    /// Default: BIP44
    pub derivation_standard: DerivationStandard,
    /// Concurrency limit for RPC calls
    /// Default: 5
    pub concurrency_limit: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            gap_limit: 20,
            batch_size: 10,
            derivation_standard: DerivationStandard::Bip44,
            concurrency_limit: 5,
        }
    }
}

/// Represents a discovered account with activity info
#[derive(Debug, Clone)]
pub struct DiscoveredAccount {
    /// The account address
    pub address: Address,
    /// The BIP-44 address index
    pub index: u32,
    /// The ETH balance
    pub balance: U256,
    /// The transaction count (nonce)
    pub nonce: u64,
    /// Whether the account is considered "active" (balance > 0 or nonce > 0)
    pub active: bool,
    /// The full derivation path used
    pub derivation_path: String,
}

/// Trait for blockchain interactions to facilitate testing
///
/// This abstracts the Alloy Provider to allow easy mocking of
/// network calls without spinning up a real node or complex provider mocks.
#[async_trait]
pub trait DiscoveryClient: Send + Sync {
    async fn get_balance(&self, address: Address) -> AccountResult<U256>;
    async fn get_transaction_count(&self, address: Address) -> AccountResult<u64>;
    async fn call(&self, tx: &TransactionRequest) -> AccountResult<Bytes>;
    async fn get_chain_id(&self) -> AccountResult<u64>;
}

/// Blanket implementation for any Alloy Provider
#[async_trait]
impl<P> DiscoveryClient for P 
where 
    P: Provider + Send + Sync + 'static,
{
    async fn get_balance(&self, address: Address) -> AccountResult<U256> {
        Provider::get_balance(self, address).await
            .map_err(|e| AccountError::operation_failed("get_balance", e.to_string()))
    }

    async fn get_transaction_count(&self, address: Address) -> AccountResult<u64> {
        Provider::get_transaction_count(self, address).await
            .map_err(|e| AccountError::operation_failed("get_transaction_count", e.to_string()))
    }

    async fn call(&self, tx: &TransactionRequest) -> AccountResult<Bytes> {
        // Need to clone tx because call takes ownership of request in some impls or we passed refs?
        // Alloy Provider trait: call(&self, tx: TransactionRequest) -> ...
        // Our trait takes ref. So we clone.
        Provider::call(self, tx.clone()).await
            .map_err(|e| AccountError::operation_failed("call", e.to_string()))
    }

    async fn get_chain_id(&self) -> AccountResult<u64> {
        Provider::get_chain_id(self).await
            .map_err(|e| AccountError::operation_failed("get_chain_id", e.to_string()))
    }
}

/// Service for discovering HD wallet accounts
pub struct AccountDiscovery {
    client: Arc<dyn DiscoveryClient>,
}

impl AccountDiscovery {
    /// Create a new AccountDiscovery service
    pub fn new(client: Arc<dyn DiscoveryClient>) -> Self {
        Self { client }
    }

    /// Discover active accounts from a seed phrase
    ///
    /// Scans addresses derived from the seed phrase until `gap_limit` consecutive
    /// inactive accounts are found.
    #[instrument(skip(self, seed_phrase), fields(correlation_id = %Uuid::new_v4()))]
    pub async fn discover_accounts(
        &self,
        seed_phrase: &str,
        config: DiscoveryConfig,
    ) -> AccountResult<Vec<DiscoveredAccount>> {
        let mut discovered = Vec::new();
        let mut gap_count = 0;
        let mut current_index = 0;
        let semaphore = Arc::new(Semaphore::new(config.concurrency_limit));

        tracing::info!(
            gap_limit = config.gap_limit,
            batch_size = config.batch_size,
            "Starting account discovery"
        );

        while gap_count < config.gap_limit {
            let start_index = current_index;
            let end_index = start_index + config.batch_size as u32;
            
            // 1. Derive addresses for this batch
            let batch_indices: Vec<u32> = (start_index..end_index).collect();
            let derived_accounts = self.derive_accounts(seed_phrase, &batch_indices, &config.derivation_standard)?;

            // 2. Check activity for the batch
            let checked_accounts = self.check_batch_activity(&derived_accounts, &semaphore).await?;

            // 3. Process results
            for account in checked_accounts {
                if account.active {
                    gap_count = 0; // Reset gap count on active account
                    discovered.push(account);
                } else {
                    gap_count += 1;
                    if gap_count >= config.gap_limit {
                        break;
                    }
                }
            }

            tracing::debug!(
                current_index = end_index,
                current_gap = gap_count,
                discovered_count = discovered.len(),
                "Processed batch"
            );

            current_index = end_index;
            
            // Safety break 
             if current_index > 2000 {
                tracing::warn!("Safety limit reached: scanned over 2000 indices");
                break;
            }
        }

        tracing::info!(
            total_discovered = discovered.len(),
            scanned_indices = current_index,
            "Account discovery completed"
        );

        Ok(discovered)
    }

    /// Derive addresses for a batch of indices
    fn derive_accounts(
        &self,
        seed_phrase: &str,
        indices: &[u32],
        standard: &DerivationStandard,
    ) -> AccountResult<Vec<(u32, Address, String)>> {
        use coins_bip39::English;
        let mut accounts = Vec::new();

        for &index in indices {
            let path_string = standard.path_template().replace("{index}", &index.to_string());
            
            // Use MnemonicBuilder to derive address
            let signer = MnemonicBuilder::<English>::default()
                .phrase(seed_phrase)
                .derivation_path(&path_string)
                .map_err(|e| AccountError::import_failed(format!("Invalid derivation path: {}", e), "seed_phrase"))?
                .build()
                .map_err(|e| AccountError::import_failed(format!("Failed to derive key: {}", e), "seed_phrase"))?;
                
            accounts.push((index, signer.address(), path_string));
        }

        Ok(accounts)
    }

    /// Check activity (balance, nonce) for a batch of accounts
    #[instrument(skip(self, accounts, _semaphore))]
    async fn check_batch_activity(
        &self,
        accounts: &[(u32, Address, String)],
        _semaphore: &Semaphore,
    ) -> AccountResult<Vec<DiscoveredAccount>> {
        let addresses: Vec<Address> = accounts.iter().map(|(_, addr, _)| *addr).collect();
        // Allow chain_id failure by explicit default if needed, or propagate error.
        let chain_id = self.client.get_chain_id().await.unwrap_or(1);

        // 1. Get Balances (Try Multicall3 first)
        let balances = if crate::performance::multicall::is_multicall3_supported(chain_id) {
            match self.get_balances_multicall(chain_id, &addresses).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("Multicall failed, falling back to parallel: {}", e);
                    self.get_balances_parallel(&addresses).await?
                }
            }
        } else {
            self.get_balances_parallel(&addresses).await?
        };

        // 2. Get Nonces (Parallel calls)
        let nonces = self.get_nonces_parallel(&addresses).await?;

        // 3. Combine results
        let mut results = Vec::new();
        for (i, (index, address, path)) in accounts.iter().enumerate() {
            let balance = balances.get(i).cloned().unwrap_or_default();
            let nonce = nonces.get(i).cloned().unwrap_or_default();
            let active = balance > U256::ZERO || nonce > 0;

            results.push(DiscoveredAccount {
                address: *address,
                index: *index,
                balance,
                nonce,
                active,
                derivation_path: path.clone(),
            });
        }

        Ok(results)
    }

    async fn get_balances_multicall(
        &self, 
        chain_id: u64, 
        addresses: &[Address]
    ) -> AccountResult<Vec<U256>> {
        let correlation_id = Uuid::new_v4();
        let mut builder = MulticallBuilder::with_correlation(chain_id, correlation_id)
            .allow_failures(true);

        for addr in addresses {
            builder = builder.add_eth_balance_call(*addr);
        }

        let calls = builder.clone().build();
        let multicall_addr = builder.multicall_address();
        
        use crate::performance::multicall::IMulticall3;
        use alloy::sol_types::SolCall;
        
        let call_data = IMulticall3::aggregate3Call { calls }.abi_encode();
        
        let tx = TransactionRequest::default()
            .to(multicall_addr)
            .input(call_data.into());

        let result_bytes = self.client.call(&tx).await?;

        // Decode results - fixed abi_decode_returns arg count
        let decoded_call = IMulticall3::aggregate3Call::abi_decode_returns(&result_bytes)
            .map_err(|e| AccountError::operation_failed("multicall_decode", format!("Failed to decode multicall: {}", e)))?;
            
        // decoded_call is Vec<IMulticall3::Result> directly
        let balance_results = decode_balance_results(addresses, &decoded_call, correlation_id);
        
        Ok(balance_results.iter().map(|r| r.balance.unwrap_or_default()).collect())
    }

    async fn get_balances_parallel(
        &self, 
        addresses: &[Address],
    ) -> AccountResult<Vec<U256>> {
        let mut tasks = Vec::new();
        
        for &addr in addresses {
            let client = self.client.clone();
            tasks.push(async move {
                client.get_balance(addr).await
            });
        }

        let results = join_all(tasks).await;
        
        // Propagate errors - validation of discovery requires accuracy
        let mut balances = Vec::new();
        for result in results {
            match result {
                Ok(balance) => balances.push(balance),
                Err(e) => return Err(e),
            }
        }
        
        Ok(balances)
    }

    async fn get_nonces_parallel(
        &self, 
        addresses: &[Address],
    ) -> AccountResult<Vec<u64>> {
        let mut tasks = Vec::new();
        
        for &addr in addresses {
            let client = self.client.clone();
            tasks.push(async move {
                client.get_transaction_count(addr).await
            });
        }

        let results = join_all(tasks).await;
        Ok(results.into_iter().map(|r| r.unwrap_or(0)).collect())
    }
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
