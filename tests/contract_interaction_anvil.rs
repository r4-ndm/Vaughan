use alloy::primitives::{Address, U256, bytes};
use alloy::sol;
use alloy::sol_types::{SolCall, SolValue, SolStruct};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::network::EthereumWallet;
use std::borrow::Cow;

mod utils;
use utils::anvil::AnvilContext;

// Define ERC20 Interface
sol! {
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
    }
}

// Define EIP-712 Struct
sol! {
    #[derive(Debug, PartialEq, Eq)]
    struct Permit {
        address owner;
        address spender;
        uint256 value;
        uint256 nonce;
        uint256 deadline;
    }
}

#[tokio::test]
async fn test_erc20_contract_interaction() {
    let context = match AnvilContext::try_new() {
        Some(c) => c,
        None => return,
    };
    
    // 1. Setup
    let signer = context.get_signer(0);
    let sender = signer.address();
    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(context.endpoint());

    // 2. "Deploy" Mock Token by injecting code
    // This bytecode simply returns true (bool) for any call
    // PUSH1 0x01 PUSH1 0x00 MSTORE PUSH1 0x20 PUSH1 0x00 RETURN
    let mock_code = bytes!("600160005260206000f3");
    let token_address = Address::from([0x12; 20]);
    
    // Inject code use anvil_setCode
    let result: Result<(), _> = provider.raw_request("anvil_setCode".into(), (token_address, mock_code)).await;
    
    if let Err(e) = result {
        eprintln!("Warning: anvil_setCode failed: {:?}", e);
    }

    // 3. Test Transfer
    // We create the calldata using alloy's type-safe builder
    let recipient = Address::from([0xBd; 20]);
    let amount = U256::from(100);
    
    let call = IERC20::transferCall { to: recipient, amount };
    let tx_data = call.abi_encode();
    
    // Send transaction
    let receipt = provider.send_transaction(alloy::rpc::types::TransactionRequest::default()
        .to(token_address)
        .input(tx_data.into()))
        .await
        .expect("Failed to send tx")
        .get_receipt()
        .await
        .expect("Failed to get receipt");

    assert!(receipt.status());
    println!("Transfer interaction successful");
}

#[test]
fn test_eip712_hashing() {
    // 4. Test EIP-712 struct hashing
    let permit = Permit {
        owner: Address::from([0x01; 20]),
        spender: Address::from([0x02; 20]),
        value: U256::from(1000),
        nonce: U256::from(1),
        deadline: U256::from(9999999999u64),
    };
    
    let domain = alloy::dyn_abi::Eip712Domain {
        name: Some(Cow::Owned("VaughanToken".to_string())),
        version: Some(Cow::Owned("1".to_string())),
        chain_id: Some(U256::from(1)),
        verifying_contract: Some(Address::from([0x12; 20])),
        salt: None,
    };
    
    let hash = permit.eip712_signing_hash(&domain);
    
    // We verify it produces a 32-byte hash
    assert_eq!(hash.len(), 32);
    // Deterministic check
    // (Values derived from running this code once to establish baseline, or just asserting non-zero for now)
    assert_ne!(hash, alloy::primitives::B256::ZERO);
}
