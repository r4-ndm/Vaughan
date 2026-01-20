//! Alloy Signer Integration
//!
//! This module provides integration with Alloy's signer ecosystem, enabling
//! seamless composition of Vaughan accounts with Alloy providers.
//!
//! # Design Inspiration
//! - **Alloy**: Native signer trait integration
//! - **MetaMask**: Account abstraction patterns
//!
//! # Requirements Addressed
//! - Recommendation 1: Alloy Signer Integration Enhancement
//! - Support PrivateKeySigner for software accounts
//! - Support LedgerSigner/TrezorSigner for hardware wallets
//! - Enable EthereumWallet composition

use crate::error::{Result, VaughanError, WalletError};
use crate::security::{KeyReference, SecureAccount};
use crate::telemetry::account_events::OperationSpan;
use alloy::primitives::{Address, B256};
use alloy::signers::{
    local::PrivateKeySigner,
    Signature, Signer, SignerSync,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn, error};

/// Vaughan signer that wraps various signing backends.
///
/// This enum provides a unified interface for signing operations,
/// supporting both software (private key) and hardware wallet signers.
#[derive(Debug, Clone)]
pub enum VaughanSigner {
    /// Software signer using in-memory private key
    PrivateKey(PrivateKeySigner),
    /// Placeholder for Ledger hardware wallet
    /// Note: Actual Ledger integration requires hardware feature
    #[cfg(feature = "hardware-wallets")]
    Ledger(LedgerSignerWrapper),
    /// Placeholder for Trezor hardware wallet
    /// Note: Actual Trezor integration requires hardware feature
    #[cfg(feature = "hardware-wallets")]
    Trezor(TrezorSignerWrapper),
}

impl VaughanSigner {
    /// Create a software signer from a private key
    pub fn from_private_key(private_key: B256) -> Result<Self> {
        let signer = PrivateKeySigner::from_bytes(&private_key)
            .map_err(|e| VaughanError::Wallet(WalletError::InvalidPrivateKey))?;
        Ok(Self::PrivateKey(signer))
    }

    /// Get the address associated with this signer
    pub fn address(&self) -> Address {
        match self {
            Self::PrivateKey(signer) => signer.address(),
            #[cfg(feature = "hardware-wallets")]
            Self::Ledger(signer) => signer.address(),
            #[cfg(feature = "hardware-wallets")]
            Self::Trezor(signer) => signer.address(),
        }
    }

    /// Check if this is a hardware wallet signer
    pub fn is_hardware(&self) -> bool {
        match self {
            Self::PrivateKey(_) => false,
            #[cfg(feature = "hardware-wallets")]
            Self::Ledger(_) | Self::Trezor(_) => true,
        }
    }
}

#[async_trait]
impl Signer for VaughanSigner {
    /// Sign a message hash asynchronously
    async fn sign_hash(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        match self {
            Self::PrivateKey(signer) => signer.sign_hash(hash).await,
            #[cfg(feature = "hardware-wallets")]
            Self::Ledger(signer) => signer.sign_hash(hash).await,
            #[cfg(feature = "hardware-wallets")]
            Self::Trezor(signer) => signer.sign_hash(hash).await,
        }
    }

    /// Get the address of this signer
    fn address(&self) -> Address {
        VaughanSigner::address(self)
    }

    /// Get the chain ID this signer is configured for
    fn chain_id(&self) -> Option<u64> {
        match self {
            Self::PrivateKey(signer) => signer.chain_id(),
            #[cfg(feature = "hardware-wallets")]
            Self::Ledger(signer) => signer.chain_id(),
            #[cfg(feature = "hardware-wallets")]
            Self::Trezor(signer) => signer.chain_id(),
        }
    }

    /// Set the chain ID for this signer
    fn set_chain_id(&mut self, chain_id: Option<u64>) {
        match self {
            Self::PrivateKey(signer) => signer.set_chain_id(chain_id),
            #[cfg(feature = "hardware-wallets")]
            Self::Ledger(signer) => signer.set_chain_id(chain_id),
            #[cfg(feature = "hardware-wallets")]
            Self::Trezor(signer) => signer.set_chain_id(chain_id),
        }
    }
}

impl SignerSync for VaughanSigner {
    /// Sign a message hash synchronously  
    fn sign_hash_sync(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        match self {
            Self::PrivateKey(signer) => signer.sign_hash_sync(hash),
            #[cfg(feature = "hardware-wallets")]
            _ => Err(alloy::signers::Error::UnsupportedOperation(
                alloy::signers::UnsupportedSignerOperation::SignHash,
            )),
        }
    }

    fn chain_id_sync(&self) -> Option<u64> {
        match self {
            Self::PrivateKey(signer) => signer.chain_id_sync(),
            #[cfg(feature = "hardware-wallets")]
            Self::Ledger(signer) => signer.chain_id(),
            #[cfg(feature = "hardware-wallets")]
            Self::Trezor(signer) => signer.chain_id(),
        }
    }
}

/// Signer manager for retrieving signers from accounts
pub struct SignerManager {
    /// Correlation tracking
    span: Option<OperationSpan>,
}

impl SignerManager {
    /// Create a new signer manager
    pub fn new() -> Self {
        Self { span: None }
    }

    /// Create a signer manager with correlation tracking
    pub fn with_correlation(span: OperationSpan) -> Self {
        Self { span: Some(span) }
    }

    /// Get a signer for a software account given the private key bytes
    ///
    /// # Security Note
    /// The private key should be retrieved from secure storage and
    /// passed directly. The caller is responsible for zeroizing the
    /// key material after use.
    pub fn get_signer_from_key(&self, private_key: &[u8]) -> Result<VaughanSigner> {
        if private_key.len() != 32 {
            return Err(VaughanError::Wallet(WalletError::InvalidPrivateKey));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(private_key);
        let key = B256::from(key_bytes);
        
        let signer = VaughanSigner::from_private_key(key)?;
        
        if let Some(ref span) = self.span {
            info!(
                correlation_id = %span.correlation_id,
                address = %signer.address(),
                "Created software signer"
            );
        }
        
        Ok(signer)
    }

    /// Check if an account can provide a signer
    pub fn can_sign(&self, account: &SecureAccount) -> bool {
        // Software accounts can always sign if unlocked
        // Hardware accounts need device connection
        !account.is_hardware
    }
}

impl Default for SignerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for composing signers with Alloy providers
///
/// This struct provides the `to_ethereum_wallet()` pattern for
/// seamless integration with Alloy's provider ecosystem.
///
/// # Example
/// ```ignore
/// let wallet = EthereumWalletBuilder::new()
///     .with_signer(signer)
///     .build();
/// 
/// let provider = ProviderBuilder::new()
///     .wallet(wallet)
///     .on_http(url);
/// ```
pub struct EthereumWalletBuilder {
    signer: Option<VaughanSigner>,
    chain_id: Option<u64>,
}

impl EthereumWalletBuilder {
    /// Create a new wallet builder
    pub fn new() -> Self {
        Self {
            signer: None,
            chain_id: None,
        }
    }

    /// Set the signer for this wallet
    pub fn with_signer(mut self, signer: VaughanSigner) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Set the chain ID for this wallet
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Build the configured wallet
    pub fn build(self) -> Result<VaughanSigner> {
        let mut signer = self.signer.ok_or_else(|| {
            VaughanError::Wallet(WalletError::WalletError {
                message: "No signer configured".to_string(),
            })
        })?;

        if let Some(chain_id) = self.chain_id {
            signer.set_chain_id(Some(chain_id));
        }

        Ok(signer)
    }
}

impl Default for EthereumWalletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hardware-wallets")]
/// Ledger signer wrapper (placeholder for actual implementation)
#[derive(Debug, Clone)]
pub struct LedgerSignerWrapper {
    address: Address,
    chain_id: Option<u64>,
}

#[cfg(feature = "hardware-wallets")]
impl LedgerSignerWrapper {
    pub fn address(&self) -> Address {
        self.address
    }
    
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }
    
    pub fn set_chain_id(&mut self, chain_id: Option<u64>) {
        self.chain_id = chain_id;
    }
    
    pub async fn sign_hash(&self, _hash: &B256) -> alloy::signers::Result<Signature> {
        // Placeholder implementation
         Err(alloy::signers::Error::UnsupportedOperation(
            alloy::signers::UnsupportedSignerOperation::SignHash,
        ))
    }
}

#[cfg(feature = "hardware-wallets")]
/// Trezor signer wrapper (placeholder for actual implementation)
#[derive(Debug, Clone)]
pub struct TrezorSignerWrapper {
    address: Address,
    chain_id: Option<u64>,
}

#[cfg(feature = "hardware-wallets")]
impl TrezorSignerWrapper {
    pub fn address(&self) -> Address {
        self.address
    }
    
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }
    
    pub fn set_chain_id(&mut self, chain_id: Option<u64>) {
        self.chain_id = chain_id;
    }
    
        pub async fn sign_hash(&self, _hash: &B256) -> alloy::signers::Result<Signature> {
        // Placeholder implementation
         Err(alloy::signers::Error::UnsupportedOperation(
            alloy::signers::UnsupportedSignerOperation::SignHash,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_from_private_key() {
        // Test private key (DO NOT USE IN PRODUCTION)
        let private_key = B256::from([
            0x4c, 0x0c, 0x89, 0x32, 0x33, 0x00, 0x26, 0x89,
            0x01, 0x04, 0x8e, 0xd2, 0x28, 0x55, 0x93, 0x90,
            0x5d, 0x72, 0xe1, 0xf9, 0x3b, 0x45, 0x47, 0x44,
            0x49, 0x6f, 0xde, 0x8d, 0x5d, 0x6e, 0x9f, 0x47,
        ]);

        let signer = VaughanSigner::from_private_key(private_key);
        assert!(signer.is_ok());
        
        let signer = signer.unwrap();
        assert!(!signer.is_hardware());
        assert!(!signer.address().is_zero());
    }

    #[test]
    fn test_signer_manager_from_key() {
        let manager = SignerManager::new();
        
        // Valid 32-byte key
        let key = [0x42u8; 32];
        let result = manager.get_signer_from_key(&key);
        assert!(result.is_ok());

        // Invalid key length
        let short_key = [0x42u8; 16];
        let result = manager.get_signer_from_key(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_ethereum_wallet_builder() {
        let private_key = B256::from([0x42u8; 32]);
        let signer = VaughanSigner::from_private_key(private_key).unwrap();
        
        let wallet = EthereumWalletBuilder::new()
            .with_signer(signer)
            .with_chain_id(1)
            .build();
        
        assert!(wallet.is_ok());
        let wallet = wallet.unwrap();
        assert_eq!(wallet.chain_id(), Some(1));
    }

    #[test]
    fn test_wallet_builder_no_signer_fails() {
        let result = EthereumWalletBuilder::new().build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_signer_signing() {
        let private_key = B256::from([0x42u8; 32]);
        let signer = VaughanSigner::from_private_key(private_key).unwrap();
        
        let hash = B256::from([0x01u8; 32]);
        let signature = signer.sign_hash(&hash).await;
        
        assert!(signature.is_ok());
    }

    #[test]
    fn test_signer_sync_signing() {
        let private_key = B256::from([0x42u8; 32]);
        let signer = VaughanSigner::from_private_key(private_key).unwrap();
        
        let hash = B256::from([0x01u8; 32]);
        let signature = signer.sign_hash_sync(&hash);
        
        assert!(signature.is_ok());
    }
}
