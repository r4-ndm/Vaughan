#!/usr/bin/env python3
"""
PulseChain Testnet Transaction Debug Script
This script identifies network-related issues for PulseChain Testnet v4.
"""
import subprocess
import json
import sys
import re

def test_pulsechain_rpc():
    """Test PulseChain Testnet v4 RPC connectivity"""
    print("🔗 Testing PulseChain Testnet v4 RPC connectivity...")
    
    import requests
    import json
    
    rpc_url = "https://rpc.v4.testnet.pulsechain.com"
    
    # Test eth_chainId
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_chainId",
        "params": [],
        "id": 1
    }
    
    try:
        response = requests.post(rpc_url, json=payload, timeout=10)
        if response.status_code == 200:
            result = response.json()
            if "result" in result:
                chain_id = int(result["result"], 16)
                if chain_id == 943:
                    print(f"✅ PulseChain Testnet RPC working - Chain ID: {chain_id}")
                    return True
                else:
                    print(f"❌ Wrong Chain ID: {chain_id} (expected 943)")
                    return False
            else:
                print(f"❌ Invalid response: {result}")
                return False
        else:
            print(f"❌ HTTP error: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ RPC connection failed: {e}")
        return False

def test_balance_on_pulsechain(account_address):
    """Test balance fetch directly from PulseChain RPC"""
    print(f"💰 Testing balance fetch for {account_address}...")
    
    import requests
    
    rpc_url = "https://rpc.v4.testnet.pulsechain.com"
    payload = {
        "jsonrpc": "2.0", 
        "method": "eth_getBalance",
        "params": [account_address, "latest"],
        "id": 1
    }
    
    try:
        response = requests.post(rpc_url, json=payload, timeout=10)
        if response.status_code == 200:
            result = response.json()
            if "result" in result:
                balance_wei = int(result["result"], 16)
                balance_tpls = balance_wei / 1e18
                print(f"✅ Balance on PulseChain Testnet: {balance_tpls:.6f} tPLS ({balance_wei} wei)")
                return balance_tpls > 0
            else:
                print(f"❌ Invalid response: {result}")
                return False
        else:
            print(f"❌ HTTP error: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Balance fetch failed: {e}")
        return False

def analyze_network_issues():
    """Analyze potential network configuration issues"""
    print("\n🔍 NETWORK CONFIGURATION ANALYSIS")
    print("=" * 50)
    
    print("📍 Expected Configuration:")
    print("  • Network: PulseChain Testnet v4")
    print("  • Chain ID: 943") 
    print("  • RPC URL: https://rpc.v4.testnet.pulsechain.com")
    print("  • Currency: tPLS")
    
    print("\n❗ Potential Issues:")
    print("1. 🌐 NetworkManager defaults to Ethereum (Chain ID 1)")
    print("2. 🔄 Wallet might not have switched to PulseChain Testnet")
    print("3. 💰 Balance being fetched from wrong network")
    print("4. 🚀 Transactions being sent to wrong network")
    
    print("\n✅ Solutions:")
    print("1. Manually switch to PulseChain Testnet v4 in wallet UI")
    print("2. Verify network dropdown shows 'PulseChain Testnet v4'")
    print("3. Check if balance shows 'tPLS' (not ETH)")
    print("4. Ensure transaction uses Chain ID 943")

def main():
    print("🚀 PulseChain Testnet Transaction Debug")
    print("=" * 50)
    
    # Test RPC connectivity
    rpc_working = test_pulsechain_rpc()
    
    if rpc_working:
        # Test balance for Primary Account
        primary_account = "0xa8c2be786892a7c36158c34d0b51091db3520598"
        has_balance = test_balance_on_pulsechain(primary_account)
        
        if has_balance:
            print("✅ Account has tPLS balance on PulseChain Testnet")
            print("\n🎯 LIKELY ISSUE: Network Mismatch")
            print("Your wallet is probably using Ethereum instead of PulseChain Testnet")
        else:
            print("❌ No balance found on PulseChain Testnet")
    
    # Provide analysis
    analyze_network_issues()
    
    print("\n📋 IMMEDIATE ACTION NEEDED:")
    print("1. Open wallet UI")
    print("2. Check network dropdown (top-left)")
    print("3. Switch to 'PulseChain Testnet v4'")
    print("4. Verify balance shows as 'tPLS' (not ETH)")
    print("5. Try transaction again")

if __name__ == "__main__":
    main()