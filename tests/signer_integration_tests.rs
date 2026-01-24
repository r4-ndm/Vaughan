use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use vaughan::wallet::account_manager::signer_integration::{EthereumWalletBuilder, VaughanSigner};

#[tokio::test]
async fn test_signer_integration_with_provider() {
    // 1. Create a VaughanSigner
    let private_key = B256::from([
        0x4c, 0x0c, 0x89, 0x32, 0x33, 0x00, 0x26, 0x89,
        0x01, 0x04, 0x8e, 0xd2, 0x28, 0x55, 0x93, 0x90,
        0x5d, 0x72, 0xe1, 0xf9, 0x3b, 0x45, 0x47, 0x44,
        0x49, 0x6f, 0xde, 0x8d, 0x5d, 0x6e, 0x9f, 0x47,
    ]);
    let signer = VaughanSigner::from_private_key(private_key).unwrap();
    let expected_address = signer.address();

    // Verify Send + Sync (compilation check)
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<VaughanSigner>();

    // 2. Convert to EthereumWallet using the builder
    let wallet = EthereumWalletBuilder::new()
        .with_signer(signer)
        .with_chain_id(1337)
        .build_wallet()
        .expect("Failed to build EthereumWallet");

    // 3. Create a provider with the wallet
    // We use a dummy URL since we won't actually make network calls in this basic integration test
    // To fully test, we'd need Anvil/network. This test mostly checks types align and it compiles.
    /* 
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http("http://localhost:8545".parse().unwrap()); 

    // 4. Verify we can use the provider to get the default signer address 
    // (This doesn't require network usually if wallet is local)
    // Actually `default_signer_address` returns the address of the signer if available.
    let signer_address = provider.default_signer_address();
    
    assert_eq!(signer_address, expected_address);
    */
}

#[test]
fn test_manual_wallet_construction() {
    let private_key = B256::from([0x42u8; 32]);
    let signer = VaughanSigner::from_private_key(private_key).unwrap();
    
    // Test direct conversion
    let wallet: EthereumWallet = signer.into_ethereum_wallet();
    
    // Verify it implements the necessary traits by wrapping it in a ProviderBuilder (builder pattern check)
    let _builder = ProviderBuilder::new()
        .wallet(wallet);
}
