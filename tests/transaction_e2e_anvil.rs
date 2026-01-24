use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use vaughan::wallet::account_manager::signer_integration::{EthereumWalletBuilder, VaughanSigner};

mod utils;
use utils::anvil::AnvilContext;

#[tokio::test]
async fn test_e2e_transaction_flow() {
    // 1. Spawn Anvil
    let context = match AnvilContext::try_new() {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Anvil not found");
            return;
        }
    };

    // 2. Setup sender (Alice) - using Anvil's pre-funded account 0
    let signer = context.get_signer(0);
    let alice_address = signer.address();
    
    // Create VaughanSigner wrapper
    let v_signer = VaughanSigner::PrivateKey(signer);

    // Build Alloy Wallet
    let wallet = EthereumWalletBuilder::new()
        .with_signer(v_signer)
        .build_wallet()
        .expect("Failed to build wallet");

    // Create Provider with wallet
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(context.endpoint());

    // 3. Setup receiver (Bob) - random address
    let bob_address = Address::from([0xBd; 20]);

    // 4. Check initial balances
    let initial_balance = provider.get_balance(alice_address).await.expect("Failed to get balance");
    println!("Alice initial balance: {}", initial_balance);
    
    let bob_initial = provider.get_balance(bob_address).await.expect("Failed to get Bob balance");
    assert_eq!(bob_initial, U256::ZERO);

    // 5. Send transaction (1 ETH)
    let value = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
    let tx = TransactionRequest::default()
        .to(bob_address)
        .value(value);

    println!("Sending 1 ETH to {}", bob_address);
    let receipt = provider.send_transaction(tx)
        .await
        .expect("Failed to send tx")
        .get_receipt()
        .await
        .expect("Failed to get receipt");

    // 6. Verify result
    assert!(receipt.status());
    
    let bob_final = provider.get_balance(bob_address).await.expect("Failed to get Bob balance");
    assert_eq!(bob_final, value);
    println!("Bob final balance verified: {}", bob_final);
}
