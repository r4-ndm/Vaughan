#!/usr/bin/env python3
"""
Find Vaughan Wallet Accounts and Check Balances
Helps identify the correct account addresses and their balances
"""

import json
import urllib.request
import urllib.parse
import os
import sys
from pathlib import Path

# RPC endpoint that's working
RPC_URL = "https://rpc.v4.testnet.pulsechain.com"

def make_rpc_call(method, params=None, timeout=15):
    """Make a JSON-RPC call"""
    if params is None:
        params = []
    
    data = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    }
    
    json_data = json.dumps(data).encode('utf-8')
    req = urllib.request.Request(
        RPC_URL,
        data=json_data,
        headers={'Content-Type': 'application/json'}
    )
    
    try:
        response = urllib.request.urlopen(req, timeout=timeout)
        if response.status == 200:
            result = json.loads(response.read().decode())
            return result.get('result')
        return None
    except Exception as e:
        print(f"RPC call failed: {e}")
        return None

def check_balance(address):
    """Check balance of an address"""
    print(f"Checking balance for {address}...")
    
    balance_hex = make_rpc_call("eth_getBalance", [address, "latest"])
    if balance_hex:
        balance_wei = int(balance_hex, 16)
        balance_eth = balance_wei / 10**18
        print(f"  Balance: {balance_eth:.6f} tPLS")
        if balance_eth > 0:
            print(f"  ✅ FOUND BALANCE: {balance_eth:.6f} tPLS")
        return balance_eth
    else:
        print(f"  ❌ Failed to check balance")
        return 0

def find_wallet_config_files():
    """Find potential wallet configuration files"""
    print("🔍 Searching for wallet configuration files...")
    
    # Common locations where wallet data might be stored
    search_paths = [
        Path.home() / ".config" / "vaughan",
        Path.home() / ".local" / "share" / "vaughan", 
        Path.home() / ".vaughan",
        Path("/home/r4/Desktop/Vaughan_V1"),
        Path("/tmp"),
        Path.home() / ".cache" / "vaughan"
    ]
    
    config_files = []
    for search_path in search_paths:
        if search_path.exists():
            print(f"  Checking {search_path}...")
            # Look for JSON, TOML, or other config files
            for pattern in ["*.json", "*.toml", "*.yaml", "*.yml", "*.cfg", "*.conf"]:
                try:
                    for file_path in search_path.rglob(pattern):
                        if file_path.is_file() and file_path.stat().st_size > 0:
                            config_files.append(file_path)
                            print(f"    Found: {file_path}")
                except Exception as e:
                    print(f"    Error searching {search_path}: {e}")
    
    return config_files

def search_for_addresses_in_file(file_path):
    """Search for Ethereum addresses in a file"""
    addresses = []
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
            # Look for Ethereum addresses (0x followed by 40 hex characters)
            import re
            eth_addresses = re.findall(r'0x[a-fA-F0-9]{40}', content)
            
            for addr in eth_addresses:
                if addr not in addresses:
                    addresses.append(addr)
                    print(f"    Found address: {addr}")
                    
    except Exception as e:
        print(f"    Error reading {file_path}: {e}")
    
    return addresses

def check_common_test_addresses():
    """Check some common test addresses that might have been used"""
    print("\n🧪 Checking common test addresses...")
    
    # These are commonly used test addresses - replace with any you remember using
    test_addresses = [
        "0x742D35B4aC0EA09d926D0e37a59eAeE71D3E4143",  # The one from previous tests
        "0x0000000000000000000000000000000000000000",  # Zero address (just in case)
    ]
    
    balances = {}
    for addr in test_addresses:
        balance = check_balance(addr)
        if balance > 0:
            balances[addr] = balance
            
    return balances

def suggest_recovery_options():
    """Suggest ways to recover account information"""
    print("\n💡 ACCOUNT RECOVERY OPTIONS:")
    print("1. Check browser history for PulseChain faucet requests")
    print("2. Look in your downloads folder for wallet backup files") 
    print("3. Check if you used MetaMask - accounts are in browser storage")
    print("4. Search email for faucet confirmations or transaction notifications")
    print("5. Look for seed phrases or private keys you might have saved")
    print("6. Check if you used a hardware wallet (Ledger, Trezor)")
    print("7. Look for screenshots of wallet interfaces you might have taken")
    
    print("\n🔑 IF YOU FIND YOUR SEED PHRASE OR PRIVATE KEY:")
    print("You can import it into the Vaughan wallet or MetaMask")
    
    print("\n📱 QUICK METAMASK CHECK:")
    print("If you used MetaMask, check these locations:")
    print("- Chrome: ~/.config/google-chrome/Default/Local Storage/leveldb/")
    print("- Firefox: ~/.mozilla/firefox/*/storage/default/moz-extension*/")

def get_user_input_addresses():
    """Allow user to input addresses to check"""
    print("\n✋ MANUAL ADDRESS INPUT:")
    print("If you know or remember any Ethereum addresses you used, enter them below.")
    print("Enter addresses one per line, or press Enter with no input to skip:")
    
    addresses = []
    while True:
        addr = input("Enter address (or press Enter to finish): ").strip()
        if not addr:
            break
            
        # Basic validation
        if addr.startswith('0x') and len(addr) == 42:
            addresses.append(addr)
            print(f"Added: {addr}")
        else:
            print("Invalid address format. Should be 0x followed by 40 hex characters.")
    
    return addresses

def main():
    print("🔍 Vaughan Wallet Account Finder")
    print("=" * 50)
    print("This tool will help you find your wallet accounts and their balances.\n")
    
    all_addresses = []
    balances_found = {}
    
    # 1. Check config files
    config_files = find_wallet_config_files()
    if config_files:
        print(f"\n📁 Found {len(config_files)} config files, searching for addresses...")
        for config_file in config_files:
            print(f"\n  Searching {config_file}:")
            addresses = search_for_addresses_in_file(config_file)
            all_addresses.extend(addresses)
    else:
        print("\n📁 No wallet config files found in common locations")
    
    # 2. Check common test addresses  
    test_balances = check_common_test_addresses()
    balances_found.update(test_balances)
    
    # 3. Allow manual input
    manual_addresses = get_user_input_addresses()
    all_addresses.extend(manual_addresses)
    
    # 4. Check all found addresses
    if all_addresses:
        print(f"\n💰 Checking balances for {len(all_addresses)} found addresses...")
        for addr in all_addresses:
            balance = check_balance(addr)
            if balance > 0:
                balances_found[addr] = balance
    
    # 5. Summary
    print("\n" + "=" * 60)
    print("📊 SUMMARY")
    print("=" * 60)
    
    if balances_found:
        print(f"✅ Found {len(balances_found)} addresses with balance:")
        for addr, balance in balances_found.items():
            print(f"  {addr}: {balance:.6f} tPLS")
            
        print(f"\n🚀 NEXT STEPS:")
        print(f"1. Use one of these addresses in your wallet configuration")
        print(f"2. Update your wallet's RPC to: {RPC_URL}")
        print(f"3. Make sure you're on PulseChain Testnet v4 (Chain ID: 943)")
    else:
        print("❌ No addresses with balance found")
        suggest_recovery_options()
        
        print(f"\n🆘 IF YOU'RE SURE YOU HAD 1 tPLS:")
        print("1. The funds might have been on a different network")
        print("2. You might have used a different account address")
        print("3. The funds might have been transferred out")
        print("4. Try checking PulseChain Mainnet or other testnets")

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n⏹️ Search cancelled by user")
    except Exception as e:
        print(f"\n\n💥 Search failed: {e}")
        import traceback
        traceback.print_exc()