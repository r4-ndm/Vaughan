#!/usr/bin/env cargo +stable

//! Debug Wallet Addresses
//! Check what addresses the wallet is using vs our test

use std::process::Command;

fn main() {
    println!("🔍 Wallet Address Debug Test");
    println!("============================");

    println!("\n📝 Addresses from your logs:");
    println!("   Test address:   0x573065Fae4662930e19722730e793928eD7663fF");
    println!("   PEPE contract:  0x11ef259d30f4937b7eda119157e489c3c9e463f2");

    println!("\n🔧 Checking PEPE balance for different address formats...");

    let test_addresses = vec![
        "0x573065Fae4662930e19722730e793928eD7663fF", // From your logs (works)
        "0x573065fae4662930e19722730e793928ed7663ff", // Lowercase
        "573065Fae4662930e19722730e793928eD7663fF",   // No 0x prefix
    ];

    let pepe_contract = "0x11ef259d30f4937b7eda119157e489c3c9e463f2";
    let rpc_url = "https://rpc.v4.testnet.pulsechain.com";

    for addr in &test_addresses {
        println!("\n   🧪 Testing address: {}", addr);

        // Normalize address (add 0x if missing, ensure lowercase for padding)
        let normalized_addr = if addr.starts_with("0x") {
            addr.to_string()
        } else {
            format!("0x{}", addr)
        };

        // Create padded address for balanceOf call (lowercase for consistent padding)
        let addr_for_padding = normalized_addr.to_lowercase();
        let padded_address = format!("000000000000000000000000{}", &addr_for_padding[2..]);

        let balance_cmd = format!(
            r#"curl -s -X POST {} -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","method":"eth_call","params":[{{"to":"{}","data":"0x70a08231{}"}},\"latest\"],"id":1}}'"#,
            rpc_url, pepe_contract, padded_address
        );

        match Command::new("bash")
            .arg("-c")
            .arg(&balance_cmd)
            .output()
        {
            Ok(output) => {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.contains("\"result\":\"0x") {
                    if let Some(start) = response.find("\"result\":\"0x") {
                        if let Some(end) = response[start + 11..].find('\"') {
                            let hex_result = &response[start + 11..start + 11 + end];

                            if hex_result.len() >= 2 {
                                if let Ok(raw_balance) = u128::from_str_radix(&hex_result[2..], 16) {
                                    let balance_tokens = raw_balance as f64 / 1e18;
                                    println!("      ✅ PEPE balance: {} ({} raw)", balance_tokens, raw_balance);
                                    if raw_balance > 0 {
                                        println!("      🎯 This address HAS PEPE tokens!");
                                    } else {
                                        println!("      ❌ This address has ZERO PEPE tokens!");
                                    }
                                } else {
                                    println!("      ❌ Failed to parse: {}", hex_result);
                                }
                            }
                        }
                    }
                } else {
                    println!("      ❌ Invalid RPC response: {}", response.chars().take(100).collect::<String>());
                }
            }
            Err(e) => {
                println!("      ❌ Failed to call RPC: {}", e);
            }
        }
    }

    println!("\n🎯 Key Question:");
    println!("   Is the wallet using a different address than 0x573065...?");
    println!("   If the wallet's current account address is different,");
    println!("   that would explain why gas estimation fails!");

    println!("\n💡 Next Steps:");
    println!("   1. Check wallet logs for the actual 'from' address being used");
    println!("   2. Verify that address has PEPE tokens");
    println!("   3. If different address, that's the root cause!");
}