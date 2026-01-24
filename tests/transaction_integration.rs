use alloy::network::TransactionBuilder;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;

mod utils;

#[tokio::test]
async fn test_eth_transfer() {
    // Only run if Anvil is available
    if let Some(context) = utils::anvil::AnvilContext::try_new() {
        let provider = context.provider();
        
        // Setup accounts
        let sender = context.get_signer(0);
        let receiver = context.get_signer(1);
        let sender_addr = sender.address();
        let receiver_addr = receiver.address();
        
        // Initial balances
        let initial_sender_bal = provider.get_balance(sender_addr).await.expect("Failed to get sender balance");
        let initial_receiver_bal = provider.get_balance(receiver_addr).await.expect("Failed to get receiver balance");
        
        // Create transaction: Send 1 ETH
        let value = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        
        // We need a provider with the wallet to sign and send
        let wallet_provider = context.provider_with_wallet(0);
        
        let tx = TransactionRequest::default()
            .with_to(receiver_addr)
            .with_value(value);
            
        // Send transaction
        let receipt = wallet_provider.send_transaction(tx)
            .await
            .expect("Failed to send transaction")
            .get_receipt()
            .await
            .expect("Failed to get receipt");
            
        assert!(receipt.status(), "Transaction failed");
        
        // Check final balances
        let final_sender_bal = provider.get_balance(sender_addr).await.expect("Failed to get final sender balance");
        let final_receiver_bal = provider.get_balance(receiver_addr).await.expect("Failed to get final receiver balance");
        
        // Receiver should have +1 ETH
        assert_eq!(final_receiver_bal, initial_receiver_bal + value);
        
        // Sender should have less than (initial - value) due to gas
        assert!(final_sender_bal < initial_sender_bal - value);
    } else {
        println!("Skipping test_eth_transfer: Anvil not available");
    }
}

#[tokio::test]
async fn test_gas_estimation() {
    if let Some(context) = utils::anvil::AnvilContext::try_new() {
        let provider = context.provider();
        let sender = context.get_signer(0);
        let receiver = context.get_signer(1);
        
        let tx = TransactionRequest::default()
            .with_from(sender.address())
            .with_to(receiver.address())
            .with_value(U256::from(100)); // Wei
            
        let estimate = provider.estimate_gas(tx).await.expect("Failed to estimate gas");
        
        // Standard transfer is 21000, usually slightly higher in estimate due to padding
        assert!(estimate >= 21000);
    } else {
        println!("Skipping test_gas_estimation: Anvil not available");
    }
}
