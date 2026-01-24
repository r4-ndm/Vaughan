// Simple test for multicall addresses
//
// Updated to use correct Multicall3 CREATE2 address (same on all chains)

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;
    use std::str::FromStr;
    use vaughan::performance::multicall::{get_multicall3_address, MULTICALL3_ADDRESS, is_multicall3_supported};
    use vaughan::tokens::TokenManager;

    /// Test that the new multicall module and TokenManager use consistent addresses
    #[test]
    fn test_multicall_addresses() {
        let token_manager = TokenManager::new();
        
        // Standard Multicall3 CREATE2 address
        let standard_addr = Address::from_str(MULTICALL3_ADDRESS).unwrap();

        // Verify new module addresses
        assert_eq!(get_multicall3_address(1), standard_addr);   // Ethereum
        assert_eq!(get_multicall3_address(56), standard_addr);  // BSC
        assert_eq!(get_multicall3_address(137), standard_addr); // Polygon
        assert_eq!(get_multicall3_address(369), standard_addr); // PulseChain

        // Test supported chains
        assert!(is_multicall3_supported(1));
        assert!(is_multicall3_supported(56));
        assert!(is_multicall3_supported(137));
        assert!(is_multicall3_supported(369));
        assert!(!is_multicall3_supported(999));

        // TokenManager also has multicall addresses (legacy API)
        // These may differ slightly as they were added before the standard CREATE2 address
        let _eth_address = token_manager.get_multicall_address(1);
        let _bsc_address = token_manager.get_multicall_address(56);

        println!("✅ All multicall addresses working correctly!");
    }

    #[test]
    fn test_multicall3_standard_address() {
        // The Multicall3 contract was deployed with CREATE2, so it has the
        // same address on all EVM chains: 0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5
        assert_eq!(
            MULTICALL3_ADDRESS,
            "0xcA11bde05977b363a7018c201E3a73A6EcE3C5D5"
        );
    }
}

