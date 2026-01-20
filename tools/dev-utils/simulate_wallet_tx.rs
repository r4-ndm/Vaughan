use alloy::consensus::{TxEnvelope, TxLegacy};
use alloy::network::TxSigner;
use alloy::primitives::{Address, Bytes, TxKind, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rlp::Decodable;
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use secrecy::ExposeSecret;
use std::str::FromStr;
use vaughan::security::{keychain::OSKeychain, KeyReference, KeychainInterface};

#[derive(Debug, Clone)]
struct CliArgs {
    rpc: String,
    from: String,
    to: String,
    amount_wei: u128,
    gas_price_wei: Option<u128>,
    chain_id: Option<u64>,
    nonce: Option<u64>,
    // 1559 support
    tx_type: Option<String>,
    max_fee_per_gas_wei: Option<u128>,
    max_priority_fee_wei: Option<u128>,
    // controls
    dry_run: bool,
    print_raw: bool,
    out: Option<String>,
    simulate: bool,
    force: bool,
}

fn parse_args() -> CliArgs {
    let mut args = std::env::args().skip(1);
    let mut cli = CliArgs {
        rpc: "https://rpc.v4.testnet.pulsechain.com".to_string(),
        from: "0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f".to_string(),
        to: "0x742d35cc6c9e4bfe8aa16fd2fde52c74c47b8f18".to_string(),
        amount_wei: 10_u64.pow(17) as u128,
        gas_price_wei: None,
        chain_id: None,
        nonce: None,
        tx_type: None,
        max_fee_per_gas_wei: None,
        max_priority_fee_wei: None,
        dry_run: false,
        print_raw: false,
        out: None,
        simulate: false,
        force: false,
    };
    while let Some(k) = args.next() {
        match k.as_str() {
            "--rpc" => {
                if let Some(v) = args.next() {
                    cli.rpc = v;
                }
            }
            "--from" => {
                if let Some(v) = args.next() {
                    cli.from = v;
                }
            }
            "--to" => {
                if let Some(v) = args.next() {
                    cli.to = v;
                }
            }
            "--amount-wei" => {
                if let Some(v) = args.next() {
                    cli.amount_wei = v.parse::<u128>().unwrap_or(cli.amount_wei);
                }
            }
            "--gas-price-wei" => {
                if let Some(v) = args.next() {
                    cli.gas_price_wei = v.parse::<u128>().ok();
                }
            }
            "--chain-id" => {
                if let Some(v) = args.next() {
                    cli.chain_id = v.parse::<u64>().ok();
                }
            }
            "--nonce" => {
                if let Some(v) = args.next() {
                    cli.nonce = v.parse::<u64>().ok();
                }
            }
            "--tx-type" => {
                if let Some(v) = args.next() {
                    cli.tx_type = Some(v);
                }
            }
            "--max-fee-per-gas-wei" => {
                if let Some(v) = args.next() {
                    cli.max_fee_per_gas_wei = v.parse::<u128>().ok();
                }
            }
            "--max-priority-fee-wei" => {
                if let Some(v) = args.next() {
                    cli.max_priority_fee_wei = v.parse::<u128>().ok();
                }
            }
            "--dry-run" => {
                cli.dry_run = true;
            }
            "--print-raw" => {
                cli.print_raw = true;
            }
            "--out" => {
                if let Some(v) = args.next() {
                    cli.out = Some(v);
                }
            }
            "--simulate" => {
                cli.simulate = true;
            }
            "--force" => {
                cli.force = true;
            }
            _ => {}
        }
    }
    cli
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Simulating Wallet GUI Transaction Flow");
    println!("============================================");

    let cli = parse_args();

    // Load accounts from storage
    let accounts_file = format!(
        "{}/.vaughan/accounts.json",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    );
    if !std::path::Path::new(&accounts_file).exists() {
        println!("❌ Accounts file not found: {accounts_file}");
        return Ok(());
    }

    let accounts_content = std::fs::read_to_string(&accounts_file)?;
    let accounts: serde_json::Value = serde_json::from_str(&accounts_content)?;

    // Connect to RPC
    println!("\n🌐 Setting up network connection...");
    let rpc_url = cli.rpc;
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    let provider_chain_id = provider.get_chain_id().await?;
    println!("✅ Connected to chain ID: {provider_chain_id}");
    let chain_id = cli.chain_id.unwrap_or(provider_chain_id);
    if let Some(cid) = cli.chain_id {
        if cid != provider_chain_id && !cli.force {
            println!(
                "❌ Provided --chain-id ({cid}) differs from RPC chain id ({provider_chain_id}). Use --force to override."
            );
            return Ok(());
        }
    }

    // Choose funded account
    println!("\n💰 Simulating transaction with funded account...");
    let from_addr = Address::from_str(&cli.from)?;
    let to_addr = Address::from_str(&cli.to)?;

    // Check balance
    let balance = provider.get_balance(from_addr).await?;
    let balance_eth = balance.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
    println!("✅ From address balance: {balance_eth:.6} tPLS");

    // Find key reference for funded account
    let mut key_ref: Option<KeyReference> = None;
    if let Some(accounts_array) = accounts.as_array() {
        for account in accounts_array {
            if let Some(address) = account.get("address").and_then(|a| a.as_str()) {
                if address.eq_ignore_ascii_case(&cli.from) {
                    key_ref = account
                        .get("key_reference")
                        .cloned()
                        .and_then(|v| serde_json::from_value(v).ok());
                    break;
                }
            }
        }
    }

    let key_ref = match key_ref {
        Some(kr) => kr,
        None => {
            println!("❌ Could not find key reference for address: {}", cli.from);
            return Ok(());
        }
    };

    // Retrieve the private key from OS keychain
    let keychain = OSKeychain::new("vaughan-wallet".to_string())?;
    let secret = KeychainInterface::retrieve(&keychain, &key_ref)?;
    let pk_str = secret.expose_secret();
    let clean = pk_str.strip_prefix("0x").unwrap_or(pk_str);
    let key_bytes = hex::decode(clean)?;
    let signer = PrivateKeySigner::from_slice(&key_bytes)?;
    println!("✅ Using signer address: {}", signer.address());

    // Gas price and nonce
    let gas_price: u128 = match cli.gas_price_wei {
        Some(g) => g,
        None => provider.get_gas_price().await?,
    };
    println!("✅ Current gas price: {:.3} Gwei", (gas_price as f64) / 1e9);

    let nonce: u64 = match cli.nonce {
        Some(n) => n,
        None => provider.get_transaction_count(from_addr).await?,
    };

    // Build request for estimation
    let mut tx_request = TransactionRequest::default();
    tx_request.from = Some(from_addr);
    tx_request.to = Some(to_addr.into());
    tx_request.value = Some(U256::from(cli.amount_wei));
    tx_request.chain_id = Some(chain_id);
    tx_request.nonce = Some(nonce);
    tx_request.input = TransactionInput::default();
    match cli.tx_type.as_deref() {
        Some("1559") => {
            if let (Some(max_fee), Some(max_prio)) = (cli.max_fee_per_gas_wei, cli.max_priority_fee_wei) {
                tx_request.max_fee_per_gas = Some(max_fee);
                tx_request.max_priority_fee_per_gas = Some(max_prio);
            } else {
                tx_request.max_fee_per_gas = Some(gas_price);
                tx_request.max_priority_fee_per_gas = Some(gas_price / 2);
            }
        }
        Some("legacy") => {
            tx_request.gas_price = Some(gas_price);
        }
        _ => {
            if let (Some(max_fee), Some(max_prio)) = (cli.max_fee_per_gas_wei, cli.max_priority_fee_wei) {
                tx_request.max_fee_per_gas = Some(max_fee);
                tx_request.max_priority_fee_per_gas = Some(max_prio);
            } else {
                tx_request.gas_price = Some(gas_price);
            }
        }
    }

    // Estimate gas
    let gas_estimate = provider.estimate_gas(tx_request.clone()).await?;
    println!("✅ Gas estimate: {gas_estimate} units");

    // Optional simulation
    if cli.simulate {
        match provider.call(tx_request.clone()).await {
            Ok(bytes) => println!("🧪 Simulation successful: {} bytes", bytes.len()),
            Err(e) => println!("⚠️ Simulation (eth_call) failed: {e}"),
        }
    }

    // Compute total cost
    let total_cost = U256::from(gas_estimate) * U256::from(gas_price) + U256::from(cli.amount_wei);
    let total_cost_eth = total_cost.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
    println!("✅ Total transaction cost: {total_cost_eth:.6} tPLS");

    if U256::from(balance) > total_cost {
        println!("✅ Sufficient balance for transaction");

        // Sign according to tx type
        println!("🔎 Signer address (from private key): {}", signer.address());
        println!("   Matches intended from: {}", signer.address() == from_addr);
        let signed_raw = match cli.tx_type.as_deref() {
            Some("1559") => {
                use alloy::consensus::TxEip1559;
                let (max_fee, max_prio) = match (cli.max_fee_per_gas_wei, cli.max_priority_fee_wei) {
                    (Some(mf), Some(mp)) => (mf, mp),
                    _ => (gas_price, gas_price / 2),
                };
                let mut eip1559 = TxEip1559 {
                    chain_id,
                    nonce,
                    max_priority_fee_per_gas: max_prio,
                    max_fee_per_gas: max_fee,
                    gas_limit: gas_estimate,
                    to: TxKind::Call(to_addr),
                    value: U256::from(cli.amount_wei),
                    input: Bytes::new(),
                    access_list: Default::default(),
                };
                signer.sign_transaction(&mut eip1559).await?
            }
            Some("legacy") => {
                let mut legacy = TxLegacy {
                    chain_id: Some(chain_id),
                    nonce,
                    gas_price,
                    gas_limit: gas_estimate,
                    to: TxKind::Call(to_addr),
                    value: U256::from(cli.amount_wei),
                    input: Bytes::new(),
                };
                signer.sign_transaction(&mut legacy).await?
            }
            _ => {
                if cli.max_fee_per_gas_wei.is_some() && cli.max_priority_fee_wei.is_some() {
                    use alloy::consensus::TxEip1559;
                    let (max_fee, max_prio) = (cli.max_fee_per_gas_wei.unwrap(), cli.max_priority_fee_wei.unwrap());
                    let mut eip1559 = TxEip1559 {
                        chain_id,
                        nonce,
                        max_priority_fee_per_gas: max_prio,
                        max_fee_per_gas: max_fee,
                        gas_limit: gas_estimate,
                        to: TxKind::Call(to_addr),
                        value: U256::from(cli.amount_wei),
                        input: Bytes::new(),
                        access_list: Default::default(),
                    };
                    signer.sign_transaction(&mut eip1559).await?
                } else {
                    let mut legacy = TxLegacy {
                        chain_id: Some(chain_id),
                        nonce,
                        gas_price,
                        gas_limit: gas_estimate,
                        to: TxKind::Call(to_addr),
                        value: U256::from(cli.amount_wei),
                        input: Bytes::new(),
                    };
                    signer.sign_transaction(&mut legacy).await?
                }
            }
        };
        let raw_bytes: Vec<u8> = signed_raw.into();

        // Optionally print/save raw hex
        let raw_hex = format!("0x{}", hex::encode(&raw_bytes));
        if cli.print_raw {
            println!("🧾 Raw signed transaction: {raw_hex}");
        }
        if let Some(path) = cli.out.clone() {
            std::fs::write(&path, &raw_hex)?;
            println!("💾 Saved raw signed transaction to {path}");
        }

        // Decode the signed tx and recover the signer address for verification
        let mut buf: &[u8] = &raw_bytes;
        match TxEnvelope::decode(&mut buf) {
            Ok(envelope) => {
                let recovered: Option<Address> = match &envelope {
                    TxEnvelope::Legacy(s) => s.recover_signer().ok(),
                    TxEnvelope::Eip2930(s) => s.recover_signer().ok(),
                    TxEnvelope::Eip1559(s) => s.recover_signer().ok(),
                    TxEnvelope::Eip4844(s) => s.recover_signer().ok(),
                    _ => {
                        println!("⚠️ Unhandled transaction envelope variant: {envelope:?}");
                        None
                    }
                };
                match recovered {
                    Some(addr) => {
                        println!("🔎 Recovered signer from raw bytes: {addr}");
                        println!("   Matches intended from: {}", addr == from_addr);
                    }
                    None => {
                        println!("⚠️ Could not recover signer from raw bytes.");
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Failed to decode signed transaction envelope: {e:?}");
            }
        }

        if cli.dry_run {
            println!("🏁 Dry run complete: transaction was signed but not broadcast.");
        } else {
            // Broadcast
            let pending = provider.send_raw_transaction(&raw_bytes).await?;
            println!("🎉 SUCCESS! Transaction sent!");
            println!("   Transaction hash: {:?}", pending.tx_hash());
        }
    } else {
        println!("❌ Insufficient balance for transaction");
        println!("   Required: {total_cost_eth:.6} tPLS");
        println!("   Available: {balance_eth:.6} tPLS");
    }

    println!("\n🏁 Simulation Complete");
    Ok(())
}
