//! Account fixer - ensures current account has a working private key

use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Fixing account private key references...");

    // Available private key from keyring
    let available_key_id = "8f9e05e9-0df3-4e76-9512-b9803e1618e8";
    let available_private_key = "43332d23fb421000e772560c4a184aa755cb7243dffafd0b4c7189b3e17ebb28";

    // Derive address from this private key
    use alloy::signers::{local::PrivateKeySigner, Signer};

    let signer = PrivateKeySigner::from_str(available_private_key)?;
    let address = signer.address();

    println!("✅ Available key derives to address: 0x{address:x}");

    // Check if any account already uses this address
    let accounts_file = std::path::Path::new(&std::env::var("HOME")?)
        .join(".vaughan")
        .join("accounts.json");

    if accounts_file.exists() {
        let accounts_data = std::fs::read_to_string(&accounts_file)?;
        println!("📖 Current accounts.json content:\n{accounts_data}");

        // Simple check if this address is already present
        let address_str = format!("0x{address:x}");
        if accounts_data.contains(&address_str) {
            println!("✅ Address {address_str} is already in accounts.json");
        } else {
            println!("❌ Address {address_str} not found in accounts.json");
        }
    }

    // Check wallet.json
    let wallet_file = std::path::Path::new(&std::env::var("HOME")?)
        .join(".vaughan")
        .join("wallet.json");

    if wallet_file.exists() {
        let wallet_data = std::fs::read_to_string(&wallet_file)?;
        println!("📖 Current wallet.json content:\n{wallet_data}");
    }

    println!("\n💡 To fix: Update wallet.json to use address 0x{address:x}");
    println!("💡 And ensure an account in accounts.json references key ID {available_key_id}");

    Ok(())
}