//! Command-line utility to delete an account from the keychain
//!
//! Usage: cargo run --bin delete_account -- <account_name>

use vaughan::security::keychain::OSKeychain;
use vaughan::security::keystore::SecureKeystoreImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get account name from command line
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin delete_account -- <account_name>");
        eprintln!("Example: cargo run --bin delete_account -- kate");
        std::process::exit(1);
    }

    let account_name = &args[1];

    println!("🔍 Looking for account: {account_name}");

    // Create keychain and keystore
    let keychain = Box::new(OSKeychain::new("vaughan-wallet".to_string())?);
    let mut keystore = SecureKeystoreImpl::new(keychain).await?;

    // List all accounts
    let accounts = keystore.list_accounts().await?;
    
    println!("\n📋 Found {} total accounts:", accounts.len());
    for account in &accounts {
        println!("  - {} ({:?}) [service: {}]", 
            account.name, 
            account.address,
            account.key_reference.service
        );
    }

    // Find the account to delete
    let account_to_delete = accounts
        .iter()
        .find(|a| a.name == *account_name)
        .ok_or_else(|| format!("Account '{account_name}' not found"))?;

    println!("\n⚠️  About to delete account:");
    println!("  Name: {}", account_to_delete.name);
    println!("  Address: {:?}", account_to_delete.address);
    println!("  Service: {}", account_to_delete.key_reference.service);
    
    print!("\nAre you sure? (yes/no): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "yes" {
        println!("❌ Deletion cancelled");
        return Ok(());
    }

    // Delete the account
    println!("\n🗑️  Deleting account...");
    keystore.remove_account(account_to_delete.address).await?;

    println!("✅ Account '{account_name}' has been deleted from the keychain");
    println!("\n📝 Note: You may also want to remove it from ~/.vaughan/accounts.json");
    
    // Try to remove from accounts.json
    let accounts_file = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".vaughan")
        .join("accounts.json");
    
    if accounts_file.exists() {
        println!("\n🔧 Attempting to remove from accounts.json...");
        let content = std::fs::read_to_string(&accounts_file)?;
        let mut all_accounts: Vec<serde_json::Value> = serde_json::from_str(&content)?;
        
        let original_len = all_accounts.len();
        all_accounts.retain(|a| a["name"].as_str() != Some(account_name));
        
        if all_accounts.len() < original_len {
            let json_content = serde_json::to_string_pretty(&all_accounts)?;
            std::fs::write(&accounts_file, json_content)?;
            println!("✅ Removed from accounts.json");
        } else {
            println!("ℹ️  Account not found in accounts.json");
        }
    }

    println!("\n🎉 Account deletion complete!");
    println!("You can now recreate the account with a new password if needed.");

    Ok(())
}
