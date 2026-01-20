// Simple test for multicall addresses
#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;
    use vaughan::tokens::TokenManager;

    #[test]
    fn test_multicall_addresses() {
        let token_manager = TokenManager::new();

        // Test main networks
        let eth_address = token_manager.get_multicall_address(1);
        assert_eq!(
            eth_address,
            Address::from_str("0x5BA1e109517A9Db676D3435833F2FB74ea86faB9").unwrap()
        );

        let bsc_address = token_manager.get_multicall_address(56);
        assert_eq!(
            bsc_address,
            Address::from_str("0xcA11bde05977b363a7018c201E3a73A6EcE3C5D").unwrap()
        );

        let polygon_address = token_manager.get_multicall_address(137);
        assert_eq!(
            polygon_address,
            Address::from_str("0x910eFc8Ff6c998353354eE51D7942c27F5A8D1").unwrap()
        );

        let pulse_address = token_manager.get_multicall_address(369);
        assert_eq!(
            pulse_address,
            Address::from_str("0xcA11bde05977b363a7018c201E3a73A6EcE3C5D").unwrap()
        );

        // Test default (unknown network)
        let default_address = token_manager.get_multicall_address(999);
        assert_eq!(
            default_address,
            Address::from_str("0x5BA1e109517A9Db676D3435833F2FB74ea86faB9").unwrap()
        );

        println!("✅ All multicall addresses working correctly!");
    }
}
