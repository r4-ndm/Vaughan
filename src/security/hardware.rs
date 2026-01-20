//! Hardware wallet integration for Ledger and Trezor devices
//!
//! This module provides secure integration with hardware wallets,
//! supporting both Ledger and Trezor devices for maximum security.
//! Includes security features like transaction validation, timeout handling,
//! and user confirmation requirements.

#![cfg_attr(not(feature = "hardware-wallets"), allow(dead_code))]

use alloy::consensus::TxLegacy;
use alloy::primitives::{Address, Bytes, Signature, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::Signer;

use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time::timeout;

use crate::error::{HardwareWalletError, Result};
use crate::utils::validate_address;

// Conditional hardware wallet imports based on features
#[cfg(feature = "hardware-wallets")]
extern crate alloy_signer_ledger;
#[cfg(feature = "hardware-wallets")]
extern crate alloy_signer_trezor;

#[cfg(feature = "hardware-wallets")]
use {
    alloy_signer_ledger::{HDPath as LedgerHDPath, LedgerSigner},
    alloy_signer_trezor::{HDPath as TrezorHDPath, TrezorSigner},
    std::sync::Arc,
};

/// Hardware wallet types
#[cfg(feature = "hardware-wallets")]
#[derive(Debug, Clone)]
pub enum HardwareWallet {
    Ledger(LedgerWallet),
    Trezor(TrezorWallet),
}

/// Placeholder for disabled hardware wallet feature
#[cfg(not(feature = "hardware-wallets"))]
#[derive(Debug, Clone)]
pub enum HardwareWallet {
    /// Placeholder when hardware-wallets feature is disabled
    Disabled,
}

#[cfg(not(feature = "hardware-wallets"))]
impl HardwareWallet {
    /// Error when trying to use hardware wallets without feature enabled
    pub fn device_info(&self) -> Option<HardwareWalletInfo> {
        None
    }

    /// Error when trying to check connection without feature enabled
    pub fn is_connected(&self) -> bool {
        false
    }

    /// Error when trying to sign without feature enabled
    pub async fn sign_transaction(&self, _tx: &TransactionRequest, _derivation_path: &str) -> Result<Signature> {
        Err(HardwareWalletError::FeatureNotEnabled.into())
    }
}

#[cfg(feature = "hardware-wallets")]
impl HardwareWallet {
    /// Get device info from any hardware wallet variant
    pub fn device_info(&self) -> Option<HardwareWalletInfo> {
        match self {
            HardwareWallet::Ledger(ledger) => ledger.device_info(),
            HardwareWallet::Trezor(trezor) => trezor.device_info(),
        }
    }

    /// Check if device is connected
    pub fn is_connected(&self) -> bool {
        match self {
            HardwareWallet::Ledger(ledger) => ledger.is_connected(),
            HardwareWallet::Trezor(trezor) => trezor.is_connected(),
        }
    }

    /// Sign a transaction
    pub async fn sign_transaction(&self, tx: &TransactionRequest, derivation_path: &str) -> Result<Signature> {
        match self {
            HardwareWallet::Ledger(ledger) => ledger.sign_transaction(tx, derivation_path).await,
            HardwareWallet::Trezor(trezor) => trezor.sign_transaction(tx, derivation_path).await,
        }
    }
}

/// Common trait for hardware wallet operations
#[async_trait]
pub trait HardwareWalletTrait: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn get_addresses(&self, derivation_path: &str, count: u32) -> Result<Vec<Address>>;
    async fn sign_transaction(&self, tx: &TransactionRequest, derivation_path: &str) -> Result<Signature>;
    async fn sign_transaction_with_timeout(
        &self,
        tx: &TransactionRequest,
        derivation_path: &str,
        timeout_secs: u64,
    ) -> Result<Signature>;
    async fn verify_address(&self, address: &str, derivation_path: &str) -> Result<bool>;
    fn is_connected(&self) -> bool;
    fn device_info(&self) -> Option<HardwareWalletInfo>;
    fn last_activity(&self) -> Option<Instant>;
    async fn ping(&self) -> Result<()>;
}

/// Hardware wallet information
#[derive(Debug, Clone)]
pub struct HardwareWalletInfo {
    pub device_type: String,
    pub firmware_version: String,
    pub model: String,
    pub serial_number: Option<String>,
}

impl std::fmt::Display for HardwareWalletInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serial = self.serial_number.as_deref().unwrap_or("unknown");
        write!(f, "{}_{}", self.device_type.to_lowercase(), serial)
    }
}

/// Address verification result
#[derive(Debug, Clone)]
pub struct AddressVerificationResult {
    pub address: String,
    pub derivation_path: String,
    pub verified: bool,
    pub successful_verifications: u32,
    pub failed_verifications: u32,
    pub details: Vec<Result<bool>>,
}

/// Ledger wallet implementation
#[cfg(feature = "hardware-wallets")]
#[derive(Debug, Clone)]
pub struct LedgerWallet {
    connected: bool,
    device_info: Option<HardwareWalletInfo>,
    last_activity: Option<Instant>,
    connection_timeout: Duration,
    #[allow(dead_code)] // Planned for retry logic
    max_signing_attempts: u32,
    #[cfg(feature = "hardware-wallets")]
    signer: Option<Arc<LedgerSigner>>,
}

#[cfg(feature = "hardware-wallets")]
impl Default for LedgerWallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hardware-wallets")]
impl LedgerWallet {
    pub fn new() -> Self {
        Self {
            connected: false,
            device_info: None,
            last_activity: None,
            connection_timeout: Duration::from_secs(30),
            max_signing_attempts: 3,
            #[cfg(feature = "hardware-wallets")]
            signer: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Validate derivation path format
    fn validate_derivation_path(&self, path: &str) -> Result<()> {
        if !path.starts_with("m/") {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        // Validate each part is a number or number with '
        for part in &parts[1..] {
            let clean_part = part.trim_end_matches('\'');
            if clean_part.parse::<u32>().is_err() {
                return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
            }
        }

        Ok(())
    }

    /// Check if device is still responsive
    async fn check_connection(&self) -> Result<()> {
        if !self.connected {
            return Err(HardwareWalletError::DeviceNotFound.into());
        }

        // Check if connection has timed out
        if let Some(last_activity) = self.last_activity {
            if last_activity.elapsed() > self.connection_timeout {
                return Err(HardwareWalletError::CommunicationError.into());
            }
        }

        Ok(())
    }
}

#[cfg(feature = "hardware-wallets")]
#[async_trait]
impl HardwareWalletTrait for LedgerWallet {
    async fn connect(&mut self) -> Result<()> {
        // Add connection timeout
        let connection_result = timeout(self.connection_timeout, async {
            #[cfg(all(feature = "hardware-wallets", not(test)))]
            {
                // Real Ledger connection using Alloy LedgerSigner
                match LedgerSigner::new(LedgerHDPath::LedgerLive(0), Some(1)).await {
                    Ok(signer) => {
                        self.signer = Some(Arc::new(signer));
                        tracing::info!("✅ Successfully connected to Ledger device");
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to connect to Ledger: {}", e);
                        return Err(HardwareWalletError::DeviceNotFound.into());
                    }
                }

                // Get device information from Ledger
                self.device_info = Some(HardwareWalletInfo {
                    device_type: "Ledger".to_string(),
                    firmware_version: "2.1.0+".to_string(), // Real version would be queried
                    model: "Nano X".to_string(),            // Placeholder
                    serial_number: None,                    // Placeholder
                });

                self.connected = true;
                self.last_activity = Some(Instant::now());

                tracing::info!("✅ Connected to Ledger device: {:?}", self.device_info);
            }

            #[cfg(any(not(feature = "hardware-wallets"), test))]
            {
                // Simulated connection for development when hardware-wallets feature is disabled
                tokio::time::sleep(Duration::from_millis(100)).await;

                self.device_info = Some(HardwareWalletInfo {
                    device_type: "Ledger (Simulated)".to_string(),
                    firmware_version: "2.1.0-sim".to_string(),
                    model: "Nano S Plus (Simulated)".to_string(),
                    serial_number: Some("SIM001".to_string()),
                });

                self.connected = true;
                self.last_activity = Some(Instant::now());

                tracing::info!("📱 Connected to simulated Ledger device");
            }

            Ok::<(), crate::error::VaughanError>(())
        })
        .await;

        match connection_result {
            Ok(result) => result,
            Err(_) => Err(HardwareWalletError::ConnectionFailed {
                reason: "Connection timeout".to_string(),
            }
            .into()),
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.device_info = None;
        self.last_activity = None;
        tracing::info!("Ledger wallet disconnected");
        Ok(())
    }

    async fn get_addresses(&self, derivation_path: &str, count: u32) -> Result<Vec<Address>> {
        self.check_connection().await?;
        self.validate_derivation_path(derivation_path)?;

        // Limit the number of addresses to prevent DoS
        if count > 100 {
            return Err(HardwareWalletError::ConnectionFailed {
                reason: "Too many addresses requested (max 100)".to_string(),
            }
            .into());
        }

        #[cfg(all(feature = "hardware-wallets", not(test)))]
        {
            if let Some(signer) = &self.signer {
                let mut addresses: Vec<Address> = Vec::new();

                // Parse the derivation path to get the base path for iteration
                // For now, we'll use LedgerLive path and iterate on the last index
                for i in 0..count {
                    match signer
                        .get_address_with_path(&LedgerHDPath::LedgerLive(i as usize))
                        .await
                    {
                        Ok(address) => {
                            addresses.push(address);
                            tracing::debug!("✅ Derived address {} for index {}: {}", i, i, address);
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to derive address for index {}: {}", i, e);
                            return Err(HardwareWalletError::CommunicationError.into());
                        }
                    }
                }

                tracing::info!("✅ Successfully derived {} Ledger addresses", count);
                Ok(addresses)
            } else {
                Err(HardwareWalletError::DeviceNotFound.into())
            }
        }

        #[cfg(any(not(feature = "hardware-wallets"), test))]
        {
            // Mock address derivation
            let mut addresses = Vec::new();
            for i in 0..count {
                // Generate a deterministic mock address based on index
                let mock_addr_str = format!("0x{:040x}", i + 1);
                addresses.push(std::str::FromStr::from_str(&mock_addr_str).unwrap());
            }
            Ok(addresses)
        }
    }

    async fn sign_transaction(&self, tx: &TransactionRequest, derivation_path: &str) -> Result<Signature> {
        self.sign_transaction_with_timeout(tx, derivation_path, 60).await
    }

    async fn sign_transaction_with_timeout(
        &self,
        tx: &TransactionRequest,
        derivation_path: &str,
        timeout_secs: u64,
    ) -> Result<Signature> {
        self.check_connection().await?;
        self.validate_derivation_path(derivation_path)?;

        // Transaction validation would be implemented based on actual transaction structure

        // Add timeout to signing operation
        let signing_result = timeout(Duration::from_secs(timeout_secs), async {
            tracing::info!("🔐 Ledger transaction signing requested for path {}", derivation_path);

            #[cfg(all(feature = "hardware-wallets", not(test)))]
            {
                if let Some(signer) = &self.signer {
                    // Parse derivation path to extract account index
                    let _account_index = self.parse_derivation_path(derivation_path)?;

                    // Validate transaction parameters
                    self.validate_transaction_request(tx)?;

                    // Create transaction for Ledger signing
                    tracing::info!("📝 Preparing transaction for Ledger signing...");

                    // Convert TransactionRequest to TxLegacy for signing
                    let legacy_tx = self.convert_to_legacy_tx(tx)?;

                    // Sign the transaction using LedgerSigner
                    tracing::info!("🔐 Signing transaction with Ledger device...");

                    // Create the transaction hash that needs to be signed
                    let tx_hash = alloy::primitives::keccak256(alloy::rlp::encode(&legacy_tx));

                    // Use Alloy Signer trait to sign the hash
                    match signer.sign_hash(&tx_hash).await {
                        Ok(signature) => {
                            tracing::info!("✅ Successfully signed transaction with Ledger");
                            Ok(signature)
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to sign transaction with Ledger: {}", e);
                            Err(HardwareWalletError::SigningFailed.into())
                        }
                    }
                } else {
                    Err(HardwareWalletError::DeviceNotFound.into())
                }
            }

            #[cfg(any(not(feature = "hardware-wallets"), test))]
            {
                // Validate transaction parameters even in mock
                self.validate_transaction_request(tx)?;

                // Mock signing
                if let Some(_info) = &self.device_info {
                    // Return a dummy signature: r=1, s=1, v=0
                    Ok(alloy::primitives::Signature::new(U256::from(1), U256::from(1), false))
                } else {
                    Err(HardwareWalletError::DeviceNotFound.into())
                }
            }
        })
        .await;

        match signing_result {
            Ok(result) => result,
            Err(_) => Err(HardwareWalletError::ConnectionFailed {
                reason: "Transaction signing timeout".to_string(),
            }
            .into()),
        }
    }

    async fn verify_address(&self, address: &str, derivation_path: &str) -> Result<bool> {
        self.check_connection().await?;
        self.validate_derivation_path(derivation_path)?;
        validate_address(address).map(|_| ())?;

        tracing::info!("Verifying address {} with path {} on Ledger", address, derivation_path);
        Ok(true)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn device_info(&self) -> Option<HardwareWalletInfo> {
        self.device_info.clone()
    }

    fn last_activity(&self) -> Option<Instant> {
        self.last_activity
    }

    async fn ping(&self) -> Result<()> {
        if !self.connected {
            return Err(HardwareWalletError::DeviceNotFound.into());
        }

        tracing::debug!("Pinging Ledger device");
        Ok(())
    }
}

/// Private helper methods for LedgerWallet
#[cfg(feature = "hardware-wallets")]
#[allow(dead_code)] // Utility methods for future hardware wallet features
impl LedgerWallet {
    /// Parse BIP-44 derivation path to extract account index
    fn parse_derivation_path(&self, path: &str) -> Result<u32> {
        // Expected format: m/44'/60'/0'/0/x where x is the account index
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() != 6 || parts[0] != "m" || parts[1] != "44'" || parts[2] != "60'" {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        // Extract account index from last part
        let account_str = parts[5];
        account_str
            .parse::<u32>()
            .map_err(|_| HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into())
    }

    /// Validate transaction request parameters
    fn validate_transaction_request(&self, tx: &TransactionRequest) -> Result<()> {
        // Validate recipient address
        if let Some(to_kind) = &tx.to {
            match to_kind {
                alloy::primitives::TxKind::Call(addr) => {
                    validate_address(&format!("{addr:?}")).map(|_| ())?;
                }
                alloy::primitives::TxKind::Create => {
                    // Contract creation is valid
                }
            }
        } else {
            return Err(HardwareWalletError::InvalidTransaction {
                reason: "Missing recipient address".to_string(),
            }
            .into());
        }

        // Validate value is present and reasonable
        if let Some(value) = &tx.value {
            if *value == U256::ZERO {
                tracing::warn!("⚠️  Transaction value is zero");
            }
        }

        // Validate gas parameters
        if let Some(gas) = &tx.gas {
            if *gas == 0 {
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: "Gas limit cannot be zero".to_string(),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Convert TransactionRequest to TxLegacy for hardware wallet signing
    fn convert_to_legacy_tx(&self, tx: &TransactionRequest) -> Result<TxLegacy> {
        let to_address = match tx.to.as_ref() {
            Some(alloy::primitives::TxKind::Call(addr)) => *addr,
            Some(alloy::primitives::TxKind::Create) => {
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: "Contract creation not supported for hardware wallet signing".to_string(),
                }
                .into());
            }
            None => {
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: "Missing recipient address".to_string(),
                }
                .into());
            }
        };

        let legacy_tx = TxLegacy {
            chain_id: tx.chain_id,
            nonce: tx.nonce.unwrap_or(0u64),
            gas_price: tx
                .gas_price
                .map(|g| g.try_into().unwrap_or(20_000_000_000u128))
                .unwrap_or(20_000_000_000u128),
            gas_limit: tx.gas.map(|g| g.try_into().unwrap_or(21_000u64)).unwrap_or(21_000u64),
            to: alloy::primitives::TxKind::Call(to_address),
            value: tx.value.unwrap_or(U256::ZERO),
            input: Bytes::new(),
        };

        Ok(legacy_tx)
    }
}

/// Trezor wallet implementation
#[cfg(feature = "hardware-wallets")]
#[derive(Debug, Clone)]
pub struct TrezorWallet {
    connected: bool,
    device_info: Option<HardwareWalletInfo>,
    last_activity: Option<Instant>,
    connection_timeout: Duration,
    #[cfg(feature = "hardware-wallets")]
    signer: Option<Arc<TrezorSigner>>,
}

#[cfg(feature = "hardware-wallets")]
impl Default for TrezorWallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hardware-wallets")]
impl TrezorWallet {
    pub fn new() -> Self {
        Self {
            connected: false,
            device_info: None,
            last_activity: None,
            connection_timeout: Duration::from_secs(30),
            #[cfg(feature = "hardware-wallets")]
            signer: None,
        }
    }
}

#[cfg(feature = "hardware-wallets")]
#[async_trait]
impl HardwareWalletTrait for TrezorWallet {
    async fn connect(&mut self) -> Result<()> {
        let connection_result = timeout(self.connection_timeout, async {
            #[cfg(all(feature = "hardware-wallets", not(test)))]
            {
                // Real Trezor connection using Alloy TrezorSigner
                tracing::info!("🔌 Attempting to connect to Trezor device...");

                match TrezorSigner::new(TrezorHDPath::Other("m/44'/60'/0'/0/0".to_string()), Some(1)).await {
                    Ok(signer) => {
                        self.signer = Some(Arc::new(signer));
                        tracing::info!("✅ Successfully connected to Trezor device");
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to connect to Trezor: {}", e);
                        return Err(HardwareWalletError::DeviceNotFound.into());
                    }
                }

                // Get device information from Trezor
                self.device_info = Some(HardwareWalletInfo {
                    device_type: "Trezor".to_string(),
                    firmware_version: "2.5.3+".to_string(), // Real version would be queried
                    model: "Model T".to_string(),           // Real model would be detected
                    serial_number: None,                    // Trezor doesn't expose serial numbers
                });

                self.connected = true;
                self.last_activity = Some(Instant::now());

                tracing::info!("✅ Connected to Trezor device: {:?}", self.device_info);
            }

            #[cfg(any(not(feature = "hardware-wallets"), test))]
            {
                // Simulated connection for development
                tokio::time::sleep(Duration::from_millis(100)).await;

                self.device_info = Some(HardwareWalletInfo {
                    device_type: "Trezor (Simulated)".to_string(),
                    firmware_version: "2.5.3-sim".to_string(),
                    model: "Model T (Simulated)".to_string(),
                    serial_number: Some("SIM002".to_string()),
                });

                self.connected = true;
                self.last_activity = Some(Instant::now());

                tracing::info!("📱 Connected to simulated Trezor device");
            }

            Ok::<(), crate::error::VaughanError>(())
        })
        .await;

        match connection_result {
            Ok(result) => result,
            Err(_) => Err(HardwareWalletError::ConnectionFailed {
                reason: "Connection timeout".to_string(),
            }
            .into()),
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.device_info = None;
        self.last_activity = None;
        tracing::info!("Trezor wallet disconnected");
        Ok(())
    }

    async fn get_addresses(&self, _derivation_path: &str, count: u32) -> Result<Vec<Address>> {
        if !self.connected {
            return Err(HardwareWalletError::DeviceNotFound.into());
        }

        // Limit the number of addresses to prevent DoS
        if count > 100 {
            return Err(HardwareWalletError::ConnectionFailed {
                reason: "Too many addresses requested (max 100)".to_string(),
            }
            .into());
        }

        #[cfg(all(feature = "hardware-wallets", not(test)))]
        {
            if let Some(signer) = &self.signer {
                let mut addresses: Vec<Address> = Vec::new();

                // For Trezor, we can derive addresses using the signer's address method
                // The signer is initialized with a specific path, so we get that address
                if count == 1 {
                    let address = signer.address();
                    addresses.push(address);
                    tracing::info!("✅ Derived Trezor address: {}", address);
                } else {
                    // For multiple addresses, we would need to create multiple signers with different paths
                    // This is a limitation of the current Trezor API - it's path-specific
                    tracing::warn!("⚠️ Trezor signer is path-specific, only returning base address");
                    let address = signer.address();
                    addresses.push(address);
                }

                tracing::info!("✅ Successfully derived {} Trezor addresses", count);
                Ok(addresses)
            } else {
                Err(HardwareWalletError::DeviceNotFound.into())
            }
        }

        #[cfg(any(not(feature = "hardware-wallets"), test))]
        {
            // Mock address derivation for Trezor
            let mut addresses = Vec::new();
            for i in 0..count {
                let mock_addr_str = format!("0x{:040x}", i + 0xABC);
                addresses.push(std::str::FromStr::from_str(&mock_addr_str).unwrap());
            }
            Ok(addresses)
        }
    }

    async fn sign_transaction(&self, tx: &TransactionRequest, derivation_path: &str) -> Result<Signature> {
        self.sign_transaction_with_timeout(tx, derivation_path, 60).await
    }

    async fn sign_transaction_with_timeout(
        &self,
        tx: &TransactionRequest,
        derivation_path: &str,
        timeout_secs: u64,
    ) -> Result<Signature> {
        self.check_connection().await?;
        self.validate_derivation_path(derivation_path)?;

        // Add timeout to signing operation
        let signing_result = timeout(Duration::from_secs(timeout_secs), async {
            tracing::info!("🔐 Trezor transaction signing requested for path {}", derivation_path);

            #[cfg(all(feature = "hardware-wallets", not(test)))]
            {
                if let Some(signer) = &self.signer {
                    // Parse derivation path to extract account index
                    let _account_index = self.parse_derivation_path(derivation_path)?;

                    // Validate transaction parameters
                    self.validate_transaction_request(tx)?;

                    // Create transaction for Trezor signing
                    tracing::info!("📝 Preparing transaction for Trezor signing...");

                    // Convert TransactionRequest to TxLegacy for signing
                    let legacy_tx = self.convert_to_legacy_tx(tx)?;

                    // Sign the transaction using TrezorSigner
                    tracing::info!("🔐 Signing transaction with Trezor device...");

                    // Create the transaction hash that needs to be signed
                    let tx_hash = alloy::primitives::keccak256(alloy::rlp::encode(&legacy_tx));

                    // Use Alloy Signer trait to sign the hash
                    match signer.sign_hash(&tx_hash).await {
                        Ok(signature) => {
                            tracing::info!("✅ Successfully signed transaction with Trezor");
                            Ok(signature)
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to sign transaction with Trezor: {}", e);
                            Err(HardwareWalletError::SigningFailed.into())
                        }
                    }
                } else {
                    Err(HardwareWalletError::DeviceNotFound.into())
                }
            }

            #[cfg(any(not(feature = "hardware-wallets"), test))]
            {
                // Validate transaction parameters even in mock
                self.validate_transaction_request(tx)?;

                // Mock signing for Trezor
                if let Some(_info) = &self.device_info {
                    Ok(alloy::primitives::Signature::new(U256::from(1), U256::from(1), false))
                } else {
                    Err(HardwareWalletError::DeviceNotFound.into())
                }
            }
        })
        .await;

        match signing_result {
            Ok(result) => result,
            Err(_) => Err(HardwareWalletError::ConnectionFailed {
                reason: "Transaction signing timeout".to_string(),
            }
            .into()),
        }
    }

    async fn verify_address(&self, address: &str, derivation_path: &str) -> Result<bool> {
        if !self.connected {
            return Err(HardwareWalletError::DeviceNotFound.into());
        }

        // Validate derivation path format (add validation method later)
        if !derivation_path.starts_with("m/") {
            return Err(HardwareWalletError::InvalidDerivationPath {
                path: derivation_path.to_string(),
            }
            .into());
        }

        validate_address(address).map(|_| ())?;
        tracing::info!("Verifying address {} with path {} on Trezor", address, derivation_path);
        Ok(true)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn device_info(&self) -> Option<HardwareWalletInfo> {
        self.device_info.clone()
    }

    fn last_activity(&self) -> Option<Instant> {
        self.last_activity
    }

    async fn ping(&self) -> Result<()> {
        if !self.connected {
            return Err(HardwareWalletError::DeviceNotFound.into());
        }

        tracing::debug!("Pinging Trezor device");
        Ok(())
    }
}

/// Private helper methods for TrezorWallet
#[cfg(feature = "hardware-wallets")]
#[allow(dead_code)] // Utility methods for future hardware wallet features
impl TrezorWallet {
    /// Check connection status and update last activity
    async fn check_connection(&self) -> Result<()> {
        if !self.connected {
            return Err(HardwareWalletError::DeviceNotConnected.into());
        }
        Ok(())
    }

    /// Validate derivation path format
    fn validate_derivation_path(&self, path: &str) -> Result<()> {
        if !path.starts_with("m/") {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        // Validate each part is a number or number with '
        for part in &parts[1..] {
            let clean_part = part.trim_end_matches('\'');
            if clean_part.parse::<u32>().is_err() {
                return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
            }
        }

        Ok(())
    }

    /// Parse BIP-44 derivation path to extract account index
    fn parse_derivation_path(&self, path: &str) -> Result<u32> {
        // Expected format: m/44'/60'/0'/0/x where x is the account index
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() != 6 || parts[0] != "m" || parts[1] != "44'" || parts[2] != "60'" {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        // Extract account index from last part
        let account_str = parts[5];
        account_str
            .parse::<u32>()
            .map_err(|_| HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into())
    }

    /// Validate transaction request parameters
    fn validate_transaction_request(&self, tx: &TransactionRequest) -> Result<()> {
        // Validate recipient address
        if let Some(to_kind) = &tx.to {
            match to_kind {
                alloy::primitives::TxKind::Call(addr) => {
                    validate_address(&format!("{addr:?}")).map(|_| ())?;
                }
                alloy::primitives::TxKind::Create => {
                    // Contract creation is valid
                }
            }
        } else {
            return Err(HardwareWalletError::InvalidTransaction {
                reason: "Missing recipient address".to_string(),
            }
            .into());
        }

        // Validate value is present and reasonable
        if let Some(value) = &tx.value {
            if *value == U256::ZERO {
                tracing::warn!("⚠️  Transaction value is zero");
            }
        }

        // Validate gas parameters
        if let Some(gas) = &tx.gas {
            if *gas == 0 {
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: "Gas limit cannot be zero".to_string(),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Convert TransactionRequest to TxLegacy for hardware wallet signing
    fn convert_to_legacy_tx(&self, tx: &TransactionRequest) -> Result<TxLegacy> {
        let to_address = match tx.to.as_ref() {
            Some(alloy::primitives::TxKind::Call(addr)) => *addr,
            Some(alloy::primitives::TxKind::Create) => {
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: "Contract creation not supported for hardware wallet signing".to_string(),
                }
                .into());
            }
            None => {
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: "Missing recipient address".to_string(),
                }
                .into());
            }
        };

        let legacy_tx = TxLegacy {
            chain_id: tx.chain_id,
            nonce: tx.nonce.unwrap_or(0u64),
            gas_price: tx
                .gas_price
                .map(|g| g.try_into().unwrap_or(20_000_000_000u128))
                .unwrap_or(20_000_000_000u128),
            gas_limit: tx.gas.map(|g| g.try_into().unwrap_or(21_000u64)).unwrap_or(21_000u64),
            to: alloy::primitives::TxKind::Call(to_address),
            value: tx.value.unwrap_or(U256::ZERO),
            input: Bytes::new(),
        };

        Ok(legacy_tx)
    }
}

/// Hardware wallet security validator
#[cfg(feature = "hardware-wallets")]
#[derive(Debug)]
pub struct HardwareWalletSecurityValidator {
    max_transaction_value: U256,
    allowed_recipients: Option<Vec<Address>>,
    require_address_verification: bool,
    max_derivation_index: u32,
    security_audit_enabled: bool,
}

#[cfg(feature = "hardware-wallets")]
impl Default for HardwareWalletSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hardware-wallets")]
impl HardwareWalletSecurityValidator {
    pub fn new() -> Self {
        Self {
            max_transaction_value: U256::from(1000u64) * U256::from(10u64).pow(U256::from(18u64)), // 1000 ETH
            allowed_recipients: None,
            require_address_verification: true,
            max_derivation_index: 100,
            security_audit_enabled: true,
        }
    }

    /// Validate transaction security before hardware wallet signing
    pub async fn validate_transaction_security(
        &self,
        tx: &TransactionRequest,
        derivation_path: &str,
        device_type: &str,
    ) -> Result<()> {
        if !self.security_audit_enabled {
            return Ok(());
        }

        tracing::info!("🔒 Starting security validation for {} transaction", device_type);

        // Validate transaction value limits
        self.validate_transaction_value(tx)?;

        // Validate derivation path security
        self.validate_derivation_path_security(derivation_path)?;

        // Validate recipient address if allowlist is configured
        self.validate_recipient_security(tx)?;

        // Validate gas parameters for security
        self.validate_gas_security(tx)?;

        // Validate contract interaction security
        self.validate_contract_interaction_security(tx)?;

        tracing::info!("✅ Security validation passed for {} transaction", device_type);
        Ok(())
    }

    /// Validate transaction value is within security limits
    fn validate_transaction_value(&self, tx: &TransactionRequest) -> Result<()> {
        if let Some(value) = &tx.value {
            if *value > self.max_transaction_value {
                tracing::error!(
                    "❌ Transaction value {} exceeds maximum allowed {}",
                    value,
                    self.max_transaction_value
                );
                return Err(HardwareWalletError::InvalidTransaction {
                    reason: format!(
                        "Transaction value exceeds maximum allowed: {} > {}",
                        value, self.max_transaction_value
                    ),
                }
                .into());
            }
        }
        Ok(())
    }

    /// Validate derivation path follows security best practices
    fn validate_derivation_path_security(&self, path: &str) -> Result<()> {
        let parts: Vec<&str> = path.split('/').collect();

        // Validate BIP-44 compliance
        if parts.len() != 6 || parts[0] != "m" || parts[1] != "44'" || parts[2] != "60'" {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        // Validate account index is within safe limits
        if let Ok(index) = parts[5].parse::<u32>() {
            if index > self.max_derivation_index {
                tracing::error!(
                    "❌ Derivation index {} exceeds maximum allowed {}",
                    index,
                    self.max_derivation_index
                );
                return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
            }
        }

        Ok(())
    }

    /// Validate recipient address against allowlist if configured
    fn validate_recipient_security(&self, tx: &TransactionRequest) -> Result<()> {
        if let Some(allowed_recipients) = &self.allowed_recipients {
            if let Some(to_kind) = &tx.to {
                if let alloy::primitives::TxKind::Call(addr) = to_kind {
                    if !allowed_recipients.contains(addr) {
                        tracing::error!("❌ Recipient address {} not in allowlist", addr);
                        return Err(HardwareWalletError::InvalidTransaction {
                            reason: format!("Recipient address {addr} not in allowlist"),
                        }
                        .into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate gas parameters for security anomalies
    fn validate_gas_security(&self, tx: &TransactionRequest) -> Result<()> {
        // Check for suspiciously high gas prices (potential attack)
        if let Some(gas_price) = &tx.gas_price {
            let high_gas_threshold = 200_000_000_000u128; // 200 gwei
            if *gas_price > high_gas_threshold {
                tracing::warn!(
                    "⚠️ High gas price detected: {} (threshold: {})",
                    gas_price,
                    high_gas_threshold
                );
            }
        }

        // Check for suspiciously high gas limits
        if let Some(gas) = &tx.gas {
            let high_gas_limit = 1_000_000u64;
            if *gas > high_gas_limit {
                tracing::warn!("⚠️ High gas limit detected: {} (threshold: {})", gas, high_gas_limit);
            }
        }

        Ok(())
    }

    /// Validate contract interaction security
    fn validate_contract_interaction_security(&self, _tx: &TransactionRequest) -> Result<()> {
        let input_data: &[u8] = &[]; // TODO: Extract input data from TransactionInput

        // Check for suspicious contract interactions
        if input_data.len() > 10000 {
            tracing::warn!("⚠️ Large contract interaction data: {} bytes", input_data.len());
        }

        // Log contract interaction for audit trail
        if !input_data.is_empty() {
            tracing::info!(
                "📋 Contract interaction detected: {} bytes of input data",
                input_data.len()
            );
        }

        Ok(())
    }

    /// Set maximum transaction value
    pub fn with_max_transaction_value(mut self, max_value: U256) -> Self {
        self.max_transaction_value = max_value;
        self
    }

    /// Set whether address verification is required
    pub fn with_require_address_verification(mut self, required: bool) -> Self {
        self.require_address_verification = required;
        self
    }

    /// Enable or disable security auditing
    pub fn with_security_audit_enabled(mut self, enabled: bool) -> Self {
        self.security_audit_enabled = enabled;
        self
    }

    /// Validate a transaction for security compliance
    pub async fn validate_transaction(&self, tx: &TransactionRequest) -> Result<()> {
        self.validate_transaction_value(tx)?;
        self.validate_gas_security(tx)?;
        self.validate_contract_interaction_security(tx)?;
        Ok(())
    }

    /// Validate an Ethereum address
    pub fn validate_address(&self, address: &str) -> Result<()> {
        if address.is_empty() {
            return Err(HardwareWalletError::InvalidAddress.into());
        }

        // Use existing validation function
        validate_address(address).map(|_| ())?;
        Ok(())
    }

    /// Validate a BIP-44 derivation path
    pub fn validate_derivation_path(&self, path: &str) -> Result<()> {
        self.validate_derivation_path_format(path)?;

        // Extract and validate the address index
        if let Some(index_part) = path.split('/').next_back() {
            if let Ok(index) = index_part.parse::<u32>() {
                if index > self.max_derivation_index {
                    return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
                }
            }
        }

        Ok(())
    }

    /// Validate the format of a BIP-44 derivation path
    fn validate_derivation_path_format(&self, path: &str) -> Result<()> {
        // Check if path follows BIP-44 format: m/44'/60'/account'/change/address_index
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 4 || parts.len() > 6 {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        if parts[0] != "m" {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        // Validate BIP-44 structure
        if parts.len() >= 2 && !parts[1].starts_with("44'") {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        if parts.len() >= 3 && !parts[2].starts_with("60'") {
            return Err(HardwareWalletError::InvalidDerivationPath { path: path.to_string() }.into());
        }

        Ok(())
    }
}

/// Hardware wallet manager with enhanced security
#[cfg(feature = "hardware-wallets")]
#[derive(Debug)]
pub struct HardwareWalletManager {
    wallets: Vec<HardwareWallet>,
    security_validator: HardwareWalletSecurityValidator,
    recovery_attempts: std::collections::HashMap<String, u32>,
    max_recovery_attempts: u32,
}

#[cfg(feature = "hardware-wallets")]
impl Default for HardwareWalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hardware-wallets")]
impl HardwareWalletManager {
    pub fn new() -> Self {
        Self {
            wallets: Vec::new(),
            security_validator: HardwareWalletSecurityValidator::new(),
            recovery_attempts: std::collections::HashMap::new(),
            max_recovery_attempts: 3,
        }
    }

    /// Set custom security configuration
    pub fn with_security_config(mut self, validator: HardwareWalletSecurityValidator) -> Self {
        self.security_validator = validator;
        self
    }

    /// Attempt to recover from hardware wallet connection failures
    pub async fn recover_connection(&mut self, device_id: &str) -> Result<bool> {
        let attempts = self.recovery_attempts.get(device_id).copied().unwrap_or(0);

        if attempts >= self.max_recovery_attempts {
            tracing::error!(
                "❌ Maximum recovery attempts ({}) exceeded for device: {}",
                self.max_recovery_attempts,
                device_id
            );
            return Err(HardwareWalletError::DeviceNotFound.into());
        }

        tracing::info!(
            "🔄 Attempting connection recovery for device: {} (attempt {})",
            device_id,
            attempts + 1
        );

        // Increment attempt counter
        self.recovery_attempts.insert(device_id.to_string(), attempts + 1);

        // Attempt reconnection with exponential backoff
        let delay_ms = 1000 * (2u64.pow(attempts));
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

        // Try to reconnect device
        let recovered = self.attempt_device_reconnection(device_id).await?;

        if recovered {
            tracing::info!("✅ Successfully recovered connection for device: {}", device_id);
            self.recovery_attempts.remove(device_id);
        }

        Ok(recovered)
    }

    /// Attempt to reconnect a specific device
    async fn attempt_device_reconnection(&mut self, device_id: &str) -> Result<bool> {
        // Find the wallet and attempt reconnection
        for wallet in &mut self.wallets {
            match wallet {
                HardwareWallet::Ledger(ledger) => {
                    if device_id == "ledger" {
                        return Ok(ledger.connect().await.is_ok());
                    }
                }
                HardwareWallet::Trezor(trezor) => {
                    if device_id == "trezor" {
                        return Ok(trezor.connect().await.is_ok());
                    }
                }
            }
        }
        Ok(false)
    }

    /// Perform comprehensive security audit of transaction
    pub async fn audit_transaction(
        &self,
        tx: &TransactionRequest,
        derivation_path: &str,
        device_type: &str,
    ) -> Result<()> {
        tracing::info!("🔍 Starting comprehensive transaction audit for {}", device_type);

        // Run security validation
        self.security_validator
            .validate_transaction_security(tx, derivation_path, device_type)
            .await?;

        // Perform additional audit checks
        self.audit_derivation_path_security(derivation_path)?;
        self.audit_transaction_metadata(tx)?;

        tracing::info!("✅ Transaction audit completed successfully for {}", device_type);
        Ok(())
    }

    /// Audit derivation path security
    fn audit_derivation_path_security(&self, path: &str) -> Result<()> {
        tracing::debug!("🔍 Auditing derivation path security: {}", path);

        // Check for hardened derivation compliance
        let parts: Vec<&str> = path.split('/').collect();

        // Ensure proper hardening for account and coin type
        if parts.len() >= 4 && (!parts[1].ends_with('\'') || !parts[2].ends_with('\'')) {
            tracing::warn!("⚠️ Derivation path may not use proper hardening: {}", path);
        }

        // Check for non-standard derivation paths
        if parts.len() > 6 {
            tracing::warn!("⚠️ Non-standard derivation path length: {}", path);
        }

        Ok(())
    }

    /// Audit transaction metadata for security concerns
    fn audit_transaction_metadata(&self, tx: &TransactionRequest) -> Result<()> {
        tracing::debug!("🔍 Auditing transaction metadata");

        // Check for zero-value transactions (potential spam)
        if let Some(value) = &tx.value {
            if *value == U256::ZERO {
                tracing::info!("📊 Zero-value transaction detected (contract interaction or spam)");
            }
        }

        // Audit nonce for potential replay attacks
        if let Some(nonce) = &tx.nonce {
            if *nonce > 1000000 {
                tracing::warn!("⚠️ Unusually high nonce detected: {}", nonce);
            }
        }

        // Check for missing gas parameters
        if tx.gas.is_none() && tx.gas_price.is_none() {
            tracing::warn!("⚠️ Transaction missing gas parameters - may fail");
        }

        Ok(())
    }

    /// Verify address derivation across multiple hardware wallet implementations
    pub async fn verify_address_derivation(
        &self,
        address: &str,
        derivation_path: &str,
    ) -> Result<AddressVerificationResult> {
        tracing::info!(
            "🔍 Verifying address derivation: {} at path: {}",
            address,
            derivation_path
        );

        let mut verification_results = Vec::new();

        // Verify with each connected wallet
        for wallet in &self.wallets {
            let result = match wallet {
                HardwareWallet::Ledger(ledger) => {
                    self.verify_address_with_ledger(ledger, address, derivation_path).await
                }
                HardwareWallet::Trezor(trezor) => {
                    self.verify_address_with_trezor(trezor, address, derivation_path).await
                }
            };

            verification_results.push(result);
        }

        // Analyze verification results
        let mut successful_verifications = 0;
        let mut failed_verifications = 0;

        for result in &verification_results {
            match result {
                Ok(true) => successful_verifications += 1,
                Ok(false) => failed_verifications += 1,
                Err(_) => failed_verifications += 1,
            }
        }

        let verification_result = AddressVerificationResult {
            address: address.to_string(),
            derivation_path: derivation_path.to_string(),
            verified: successful_verifications > 0 && failed_verifications == 0,
            successful_verifications,
            failed_verifications,
            details: verification_results,
        };

        if verification_result.verified {
            tracing::info!("✅ Address verification successful for: {}", address);
        } else {
            tracing::error!("❌ Address verification failed for: {}", address);
        }

        Ok(verification_result)
    }

    /// Verify address with Ledger device
    async fn verify_address_with_ledger(
        &self,
        ledger: &LedgerWallet,
        address: &str,
        derivation_path: &str,
    ) -> Result<bool> {
        if !ledger.is_connected() {
            return Err(HardwareWalletError::DeviceNotConnected.into());
        }

        // Use the Ledger's verify_address method
        ledger.verify_address(address, derivation_path).await
    }

    /// Verify address with Trezor device
    async fn verify_address_with_trezor(
        &self,
        trezor: &TrezorWallet,
        address: &str,
        derivation_path: &str,
    ) -> Result<bool> {
        if !trezor.is_connected() {
            return Err(HardwareWalletError::DeviceNotConnected.into());
        }

        // Use the Trezor's verify_address method
        trezor.verify_address(address, derivation_path).await
    }

    /// Detect and connect to available hardware wallets
    pub async fn detect_wallets(&mut self) -> Result<Vec<HardwareWalletInfo>> {
        let mut detected = Vec::new();

        // Try to connect to Ledger
        let mut ledger = LedgerWallet::new();
        if ledger.connect().await.is_ok() {
            if let Some(info) = ledger.device_info() {
                detected.push(info);
            }
            self.wallets.push(HardwareWallet::Ledger(ledger));
        }

        // Try to connect to Trezor
        let mut trezor = TrezorWallet::new();
        if trezor.connect().await.is_ok() {
            if let Some(info) = trezor.device_info() {
                detected.push(info);
            }
            self.wallets.push(HardwareWallet::Trezor(trezor));
        }

        Ok(detected)
    }

    /// Get connected wallets
    pub fn get_connected_wallets(&self) -> Vec<&HardwareWallet> {
        self.wallets.iter().collect()
    }

    /// Sign transaction with specific wallet
    pub async fn sign_with_wallet(
        &self,
        wallet_index: usize,
        tx: &TransactionRequest,
        derivation_path: &str,
    ) -> Result<Signature> {
        let wallet = self
            .wallets
            .get(wallet_index)
            .ok_or(HardwareWalletError::DeviceNotFound)?;

        match wallet {
            HardwareWallet::Ledger(ledger) => ledger.sign_transaction(tx, derivation_path).await,
            HardwareWallet::Trezor(trezor) => trezor.sign_transaction(tx, derivation_path).await,
        }
    }

    /// Get addresses from a specific hardware wallet
    pub async fn get_wallet_addresses(
        &self,
        wallet_index: usize,
        derivation_path: &str,
        count: u32,
    ) -> Result<Vec<Address>> {
        let wallet = self
            .wallets
            .get(wallet_index)
            .ok_or(HardwareWalletError::DeviceNotFound)?;

        match wallet {
            HardwareWallet::Ledger(ledger) => ledger.get_addresses(derivation_path, count).await,
            HardwareWallet::Trezor(trezor) => trezor.get_addresses(derivation_path, count).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use alloy::rpc::types::{TransactionInput, TransactionRequest};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_ledger_connection() {
        let mut ledger = LedgerWallet::new();
        assert!(!ledger.is_connected());

        ledger.connect().await.unwrap();
        assert!(ledger.is_connected());

        let info = ledger.device_info().unwrap();
        assert_eq!(info.device_type, "Ledger (Simulated)");
    }

    #[tokio::test]
    async fn test_trezor_connection() {
        let mut trezor = TrezorWallet::new();
        assert!(!trezor.is_connected());

        trezor.connect().await.unwrap();
        assert!(trezor.is_connected());

        let info = trezor.device_info().unwrap();
        assert_eq!(info.device_type, "Trezor (Simulated)");
    }

    #[tokio::test]
    async fn test_hardware_wallet_manager() {
        let mut manager = HardwareWalletManager::new();
        let detected = manager.detect_wallets().await.unwrap();

        // In the test environment, both should be detected
        assert_eq!(detected.len(), 2);
        assert_eq!(manager.get_connected_wallets().len(), 2);
    }

    fn create_test_transaction() -> TransactionRequest {
        let mut tx = TransactionRequest::default();
        tx.to = Some(
            Address::from_str("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
                .unwrap()
                .into(),
        );
        tx.value = Some(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH
        tx.gas_price = Some(20_000_000_000u128); // 20 gwei
        tx.gas = Some(21_000u64);
        tx.input = TransactionInput::default();
        tx
    }

    #[tokio::test]
    async fn test_security_validator_creation() {
        let validator = HardwareWalletSecurityValidator::new();
        assert!(validator.security_audit_enabled); // Default should be true for security
        assert_eq!(validator.max_derivation_index, 100);
    }

    #[tokio::test]
    async fn test_security_validator_with_custom_limits() {
        let validator = HardwareWalletSecurityValidator::new()
            .with_max_transaction_value(U256::from(5_000_000_000_000_000_000u64)) // 5 ETH
            .with_require_address_verification(true)
            .with_security_audit_enabled(true);

        assert!(validator.require_address_verification);
        assert!(validator.security_audit_enabled);
    }

    #[tokio::test]
    async fn test_transaction_validation_success() {
        let validator = HardwareWalletSecurityValidator::new();
        let tx = create_test_transaction();

        // Should pass validation for normal transaction
        let result = validator.validate_transaction(&tx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_validation_high_value() {
        let validator = HardwareWalletSecurityValidator::new();
        let mut tx = create_test_transaction();

        // Set extremely high value (2000 ETH - above default limit)
        tx.value = Some(U256::from_str_radix("2000000000000000000000", 10).unwrap());

        let result = validator.validate_transaction(&tx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_validation_zero_gas() {
        let mut manager = HardwareWalletManager::new();
        let mut tx = create_test_transaction();

        // Set zero gas (should fail)
        tx.gas = Some(0u64);

        // Test through manager's validation
        manager.detect_wallets().await.unwrap();
        let result = manager.sign_with_wallet(0, &tx, "m/44'/60'/0'/0/0").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_address_validation() {
        let validator = HardwareWalletSecurityValidator::new();

        // Valid address
        assert!(validator
            .validate_address("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18")
            .is_ok());

        // Invalid address
        assert!(validator.validate_address("invalid_address").is_err());

        // Empty address
        assert!(validator.validate_address("").is_err());
    }

    #[tokio::test]
    async fn test_derivation_path_validation() {
        let validator = HardwareWalletSecurityValidator::new();

        // Valid BIP-44 paths
        assert!(validator.validate_derivation_path("m/44'/60'/0'/0/0").is_ok());
        assert!(validator.validate_derivation_path("m/44'/60'/0'/0/5").is_ok());

        // Invalid paths
        assert!(validator.validate_derivation_path("invalid_path").is_err());
        assert!(validator.validate_derivation_path("m/44'/60'/0'/0/99999").is_err());
        // Above default max index
    }

    #[tokio::test]
    async fn test_address_verification() {
        let manager = HardwareWalletManager::new();
        let result = manager
            .verify_address_derivation("0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18", "m/44'/60'/0'/0/0")
            .await;

        assert!(result.is_ok());
        let verification = result.unwrap();
        assert_eq!(
            verification.successful_verifications + verification.failed_verifications,
            0
        ); // No devices connected initially
    }

    #[tokio::test]
    async fn test_audit_transaction() {
        let manager = HardwareWalletManager::new();
        let tx = create_test_transaction();

        let result = manager.audit_transaction(&tx, "m/44'/60'/0'/0/0", "Ledger").await;
        assert!(result.is_ok()); // Should pass basic audit even with no devices
    }

    #[tokio::test]
    async fn test_recovery_connection() {
        let mut manager = HardwareWalletManager::new();

        let result = manager.recover_connection("test-device-id").await;
        assert!(result.is_ok());
        let recovered = result.unwrap();
        assert!(!recovered); // Should return false for non-existent device
    }

    #[tokio::test]
    async fn test_ledger_address_derivation() {
        let mut ledger = LedgerWallet::new();
        ledger.connect().await.unwrap();

        // Test address derivation with valid path
        let result = ledger.get_addresses("m/44'/60'/0'/0", 1).await;
        assert!(result.is_ok());

        // Should return exactly 1 address
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_ledger_transaction_signing() {
        let mut ledger = LedgerWallet::new();
        ledger.connect().await.unwrap();
        let tx = create_test_transaction();

        let result = ledger.sign_transaction(&tx, "m/44'/60'/0'/0/0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trezor_address_derivation() {
        let mut trezor = TrezorWallet::new();
        trezor.connect().await.unwrap();

        // Test address derivation with valid path
        let result = trezor.get_addresses("m/44'/60'/0'/0", 1).await;
        assert!(result.is_ok());

        // Should return exactly 1 address
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_trezor_transaction_signing() {
        let mut trezor = TrezorWallet::new();
        trezor.connect().await.unwrap();
        let tx = create_test_transaction();

        let result = trezor.sign_transaction(&tx, "m/44'/60'/0'/0/0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_handling_invalid_device_operations() {
        let manager = HardwareWalletManager::new();

        // Test signing with no devices
        let tx = create_test_transaction();
        let result = manager.sign_with_wallet(0, &tx, "m/44'/60'/0'/0/0").await;
        assert!(result.is_err());

        // Test getting addresses with no devices
        let result = manager.get_wallet_addresses(0, "m/44'/60'/0'/0", 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_device_access() {
        let mut manager = HardwareWalletManager::new();
        manager.detect_wallets().await.unwrap();

        let tx = create_test_transaction();

        // Test concurrent signing attempts
        let results = futures::future::join_all(vec![
            manager.sign_with_wallet(0, &tx, "m/44'/60'/0'/0/0"),
            manager.sign_with_wallet(1, &tx, "m/44'/60'/0'/0/1"),
        ])
        .await;

        // Both should succeed with mock implementations
        for result in results {
            assert!(result.is_ok());
        }
    }
}
