//! EIP-712 Typed Data Signing
//!
//! This module provides support for EIP-712 typed data hashing and signing
//! using Alloy's `sol!` macro and `SolStruct` trait.
//!
//! # Task Reference
//!
//! Implements: Task 7.1 (Add EIP-712 signing to AccountManager)
//! Implements: Task 7.2 (Add EIP-712 domain builder)
//! Implements: Task 7.3 (Add common EIP-712 message types)
//!
//! # References
//!
//! - EIP-712: <https://eips.ethereum.org/EIPS/eip-712>
//! - Alloy SolStruct: <https://github.com/alloy-rs/alloy>

use alloy::primitives::{Address, B256, U256};
use alloy::sol;
use alloy::sol_types::Eip712Domain;
#[cfg(test)]
use alloy::sol_types::SolStruct;
use serde::{Deserialize, Serialize};

// ... imports cleaned up ...

// ============================================================================
// Common EIP-712 Structs
// ============================================================================

sol! {
    /// EIP-2612 Permit struct for ERC-20 tokens
    /// 
    /// Allows gasless approvals via signature
    #[derive(Debug, Serialize, Deserialize)]
    struct Permit {
        address owner;
        address spender;
        uint256 value;
        uint256 nonce;
        uint256 deadline;
    }

    /// DAO Vote struct (Governor compatibility)
    #[derive(Debug, Serialize, Deserialize)]
    struct Vote {
        address voter;
        uint256 proposalId;
        uint8 support;
        uint256 weight;
        string reason;
    }
}

// ============================================================================
// EIP-712 Domain Builder
// ============================================================================

/// Builder for EIP-712 Domains
///
/// Helper to construct standard EIP-712 domains for signing.
#[derive(Debug, Clone, Default)]
pub struct Eip712DomainBuilder {
    name: Option<String>,
    version: Option<String>,
    chain_id: Option<U256>,
    verifying_contract: Option<Address>,
    salt: Option<B256>,
}

impl Eip712DomainBuilder {
    /// Create a new domain builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the domain name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the domain version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the chain ID
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(U256::from(chain_id));
        self
    }

    /// Set the verifying contract address
    pub fn verifying_contract(mut self, contract: Address) -> Self {
        self.verifying_contract = Some(contract);
        self
    }

    /// Set the salt
    pub fn salt(mut self, salt: B256) -> Self {
        self.salt = Some(salt);
        self
    }

    /// Build the EIP-712 domain
    pub fn build(self) -> Eip712Domain {
        Eip712Domain {
            name: self.name.map(std::borrow::Cow::Owned),
            version: self.version.map(std::borrow::Cow::Owned),
            chain_id: self.chain_id,
            verifying_contract: self.verifying_contract,
            salt: self.salt,
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_permit_struct_hash() {
        // Test vector from EIP-2612 reference implementation or known good inputs
        let permit = Permit {
            owner: address!("0000000000000000000000000000000000000001"),
            spender: address!("0000000000000000000000000000000000000002"),
            value: U256::from(1000),
            nonce: U256::from(0),
            deadline: U256::from(9999999999u64),
        };

        // We're just testing that hashing works and is deterministic
        let hash1 = permit.eip712_hash_struct();
        let hash2 = permit.eip712_hash_struct();
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, B256::ZERO);
    }

    #[test]
    fn test_domain_builder() {
        let contract = address!("0000000000000000000000000000000000000001");
        
        let domain = Eip712DomainBuilder::new()
            .name("Vaughan")
            .version("1.0")
            .chain_id(1)
            .verifying_contract(contract)
            .build();

        assert_eq!(domain.name.as_deref(), Some("Vaughan"));
        assert_eq!(domain.version.as_deref(), Some("1.0"));
        assert_eq!(domain.chain_id, Some(U256::from(1)));
        assert_eq!(domain.verifying_contract, Some(contract));
        assert_eq!(domain.salt, None);
    }

    #[test]
    fn test_vote_struct_hash() {
        let vote = Vote {
            voter: address!("0000000000000000000000000000000000000001"),
            proposalId: U256::from(123),
            support: 1,
            weight: U256::from(100),
            reason: "Support expansion".to_string(),
        };

        let hash = vote.eip712_hash_struct();
        assert_ne!(hash, B256::ZERO);
    }

    #[test]
    fn test_type_hash_constants() {
        let permit = Permit {
            owner: Address::ZERO,
            spender: Address::ZERO,
            value: U256::ZERO,
            nonce: U256::ZERO,
            deadline: U256::ZERO,
        };

        let vote = Vote {
            voter: Address::ZERO,
            proposalId: U256::ZERO,
            support: 0,
            weight: U256::ZERO,
            reason: String::new(),
        };

        // Use eip712_type_hash() method from SolStruct trait
        let permit_hash = permit.eip712_type_hash();
        let vote_hash = vote.eip712_type_hash();

        assert_ne!(permit_hash, B256::ZERO);
        assert_ne!(vote_hash, B256::ZERO);
        
        // Different structs should have different type hashes
        assert_ne!(permit_hash, vote_hash);
    }

    #[test]
    fn test_signing_message_construction() {
        let domain = Eip712DomainBuilder::new()
            .name("Test")
            .chain_id(1)
            .build();
            
        let permit = Permit {
            owner: Address::ZERO,
            spender: Address::ZERO,
            value: U256::from(100),
            nonce: U256::from(0),
            deadline: U256::MAX,
        };
        
        // This generates the full EIP-712 signing hash (domain separator + struct hash)
        let signing_hash = permit.eip712_signing_hash(&domain);
        assert_ne!(signing_hash, B256::ZERO);
    }
}
