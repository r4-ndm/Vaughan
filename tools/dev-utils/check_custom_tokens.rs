#!/usr/bin/env cargo +stable

//! Check if PEPE token is properly in custom tokens and token_balances

use std::collections::HashMap;

fn main() {
    println!("🔍 Custom Token Import Debug");
    println!("============================");

    // Expected PEPE address
    let expected_pepe = "0x11ef259d30f4937b7eda119157e489c3c9e463f2";
    println!("   Expected PEPE Address: {}", expected_pepe);

    // Simulate what should happen when you import PEPE
    println!("\n💡 What should happen when you import PEPE:");
    println!("   1. PEPE gets added to custom_tokens list");
    println!("   2. PEPE gets added to token_balances list");
    println!("   3. PEPE gets added to balance_available_tickers");
    println!("   4. Balance refresh queries: token_balances");
    println!("   5. If token_balances is empty → no tokens get balance checked");

    // Simulate the import process
    println!("\n🔧 Simulating PEPE import...");

    #[derive(Clone, Debug)]
    struct TokenInfo {
        symbol: String,
        name: String,
        address: String,
        decimals: u8,
    }

    #[derive(Clone, Debug)]
    struct SimpleTokenBalance {
        symbol: String,
        name: String,
        contract_address: Option<String>,
        balance: String,
        decimals: u8,
    }

    let mut custom_tokens = Vec::new();
    let mut token_balances = Vec::new();
    let mut balance_available_tickers = Vec::new();

    // Simulate successful token info fetch
    let token_info = TokenInfo {
        symbol: "PEPE".to_string(),
        name: "Pepe".to_string(),
        address: expected_pepe.to_string(),
        decimals: 18,
    };

    // Add to custom tokens (line 966 in working_wallet.rs)
    custom_tokens.push(token_info.clone());

    // Add to token_balances (line 978 in working_wallet.rs)
    token_balances.push(SimpleTokenBalance {
        symbol: token_info.symbol.clone(),
        name: token_info.name.clone(),
        contract_address: Some(token_info.address.clone()),
        balance: "0.0000".to_string(),
        decimals: token_info.decimals,
    });

    // Add to balance_available_tickers (line 988)
    balance_available_tickers.push(token_info.symbol.clone());

    println!("   ✅ PEPE added to custom_tokens: {}", custom_tokens.len());
    println!("   ✅ PEPE added to token_balances: {}", token_balances.len());
    println!("   ✅ PEPE added to balance_available_tickers: {}", balance_available_tickers.len());

    println!("\n📊 Final state:");
    println!("   custom_tokens: {:?}", custom_tokens);
    println!("   token_balances: {:?}", token_balances);
    println!("   balance_available_tickers: {:?}", balance_available_tickers);

    println!("\n🎯 Your Issue:");
    println!("   Your log shows: 'Received updated token balances: []'");
    println!("   This means token_balances is empty!");
    println!("   Either:");
    println!("     1. Token import failed (check logs for 'TokenInfoFetched')");
    println!("     2. Token was added but then cleared somehow");
    println!("     3. Token fetch succeeded but parse/address failed");

    println!("\n🔍 Next Steps:");
    println!("   1. Check wallet logs for 'TokenInfoFetched' messages");
    println!("   2. Look for any errors in token address parsing");
    println!("   3. Verify network matches where your PEPE token exists");
}