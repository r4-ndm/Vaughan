#!/usr/bin/env python3
"""
Quick balance checker for the actual wallet accounts found in the configuration
"""

import json
import urllib.request

# RPC endpoint
RPC_URL = "https://rpc.v4.testnet.pulsechain.com"

# Real account addresses found in wallet config
REAL_ACCOUNTS = [
    ("Primary Account", "0xa8c2be786892a7c36158c34d0b51091db3520598"),
    ("im7", "0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f")
]

def make_rpc_call(method, params=None):
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
        response = urllib.request.urlopen(req, timeout=15)
        if response.status == 200:
            result = json.loads(response.read().decode())
            return result.get('result')
        return None
    except Exception as e:
        print(f"RPC call failed: {e}")
        return None

def check_balance(address):
    """Check balance of an address"""
    balance_hex = make_rpc_call("eth_getBalance", [address, "latest"])
    if balance_hex:
        balance_wei = int(balance_hex, 16)
        balance_eth = balance_wei / 10**18
        return balance_eth
    return 0

def main():
    print("🔍 Checking balances for your real wallet accounts...")
    print("=" * 60)
    
    total_balance = 0
    account_with_balance = None
    
    for name, address in REAL_ACCOUNTS:
        print(f"\n📋 Account: {name}")
        print(f"   Address: {address}")
        
        balance = check_balance(address)
        print(f"   Balance: {balance:.6f} tPLS")
        
        if balance > 0:
            print(f"   ✅ FOUND FUNDS!")
            total_balance += balance
            account_with_balance = (name, address, balance)
        else:
            print(f"   💰 No balance")
    
    print("\n" + "=" * 60)
    print("📊 SUMMARY")
    print("=" * 60)
    
    if account_with_balance:
        name, address, balance = account_with_balance
        print(f"✅ Account with funds found!")
        print(f"   Account: {name}")
        print(f"   Address: {address}")
        print(f"   Balance: {balance:.6f} tPLS")
        
        print(f"\n🚀 SOLUTION:")
        print(f"   Your wallet should use this account for sending transactions:")
        print(f"   Account Name: {name}")
        print(f"   Account Address: {address}")
        print(f"   Available Balance: {balance:.6f} tPLS")
        
        print(f"\n🔧 NEXT STEPS:")
        print(f"1. Start the wallet: cargo run --bin dapp-platform --release")
        print(f"2. Make sure '{name}' account is selected in the send dialog")
        print(f"3. Make sure network is set to 'PulseChain Testnet v4'")
        print(f"4. You should see {balance:.6f} tPLS as available balance")
        print(f"5. Try sending a small amount (e.g., 0.1 tPLS)")
        
    else:
        print("❌ No accounts with balance found")
        print("   All your wallet accounts have 0 balance on PulseChain Testnet v4")
        
        print(f"\n💡 POSSIBLE SOLUTIONS:")
        print(f"1. Check if you have funds on a different network")
        print(f"2. Request testnet funds from PulseChain faucet")
        print(f"3. Import the account that actually has the 1 tPLS")

if __name__ == "__main__":
    main()