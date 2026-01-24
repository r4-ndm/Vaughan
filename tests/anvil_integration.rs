
use alloy::providers::Provider;

mod utils;

#[tokio::test]
async fn test_anvil_spawning() {
    if let Some(context) = utils::anvil::AnvilContext::try_new() {
        let block_number = context.provider().get_block_number().await.expect("Failed to get block number");
        assert!(block_number >= 0);
    }
}

#[tokio::test]
async fn test_prefunded_account() {
    if let Some(context) = utils::anvil::AnvilContext::try_new() {
        let provider = context.provider();
        let signer = context.get_signer(0);
        let address = signer.address();
        
        let balance = provider.get_balance(address).await.expect("Failed to get balance");
        assert!(balance > alloy::primitives::U256::from(0));
    }
}
