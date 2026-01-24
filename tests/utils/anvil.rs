
use alloy_node_bindings::{Anvil, AnvilInstance};
use alloy::providers::*;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use url::Url;

/// A test context that wraps an Anvil instance
pub struct AnvilContext {
    pub anvil: AnvilInstance,
    pub provider: reqwest::Url, // We keep the URL for cloning providers
}

impl AnvilContext {
    /// Spawn a new Anvil instance, returning None if anvil is not installed
    pub fn try_new() -> Option<Self> {
        match Anvil::new().try_spawn() {
            Ok(anvil) => {
                let provider = anvil.endpoint_url();
                Some(Self {
                    anvil,
                    provider,
                })
            }
            Err(_) => {
                eprintln!("WARNING: Could not spawn Anvil instance. Is 'anvil' installed? Skipping integration test.");
                None
            }
        }
    }

    /// Get the RPC URL
    pub fn endpoint(&self) -> Url {
        self.anvil.endpoint_url()
    }

    /// Get a provider connected to this instance
    pub fn provider(&self) -> impl Provider {
        ProviderBuilder::new().connect_http(self.endpoint())
    }

    /// Get one of the pre-funded private keys (Anvil default)
    /// Returns the signer and the address
    pub fn get_signer(&self, index: usize) -> PrivateKeySigner {
        let key = self.anvil.keys()[index].clone();
        // Convert to bytes and recreate to handle version mismatches
        let bytes = key.to_bytes();
        let b256 = alloy::primitives::B256::from_slice(bytes.as_slice());
        PrivateKeySigner::from_bytes(&b256).expect("Failed to recreate signer")
    }

    /// Get a provider with a wallet connected
    pub fn provider_with_wallet(&self, index: usize) -> impl Provider {
        let signer = self.get_signer(index);
        let wallet = EthereumWallet::from(signer);
        
        ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(self.endpoint())
    }
}
