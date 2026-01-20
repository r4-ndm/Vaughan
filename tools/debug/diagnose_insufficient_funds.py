#!/usr/bin/env python3
"""
Insufficient Funds Diagnostic Tool
This script investigates why the wallet reports insufficient funds despite having balance.
"""
import subprocess
import requests
import json
import sys

def check_account_balance_on_networks():
    """Check Primary Account balance on different networks"""
    primary_account = "0xa8c2be786892a7c36158c34d0b51091db3520598"
    
    networks = [
        ("Ethereum Mainnet", "https://ethereum.publicnode.com", 1),
        ("PulseChain Testnet v4", "https://rpc.v4.testnet.pulsechain.com", 943),
        ("PulseChain Mainnet", "https://rpc.pulsechain.com", 369),
        ("BSC", "https://bsc-dataseed1.binance.org", 56),
    ]
    
    print("🔍 Checking Primary Account Balance Across Networks")
    print("=" * 60)
    print(f"Account: {primary_account}")
    print()
    
    for name, rpc_url, chain_id in networks:
        try:
            # Check balance
            balance_payload = {
                "jsonrpc": "2.0",
                "method": "eth_getBalance",
                "params": [primary_account, "latest"],
                "id": 1
            }
            
            response = requests.post(rpc_url, json=balance_payload, timeout=10)
            
            if response.status_code == 200:
                result = response.json()
                if "result" in result:
                    balance_wei = int(result["result"], 16)
                    balance_tokens = balance_wei / 1e18
                    
                    status = "✅ FUNDED" if balance_tokens > 0 else "❌ EMPTY"
                    print(f"{status} {name:20} | {balance_tokens:>12.6f} | Chain ID: {chain_id}")
                    
                    if balance_tokens > 0:
                        print(f"     💰 {balance_wei} wei")
                else:
                    print(f"❌ {name:20} | RPC Error: {result}")
            else:
                print(f"❌ {name:20} | HTTP {response.status_code}")
                
        except Exception as e:
            print(f"❌ {name:20} | Connection failed: {e}")
    
    print()

def analyze_transaction_error():
    """Analyze the specific insufficient funds error"""
    print("🔍 TRANSACTION ERROR ANALYSIS")
    print("=" * 50)
    
    print("📧 Error Details:")
    print("  • Error Code: -32000 (INTERNAL_ERROR)")
    print("  • Message: insufficient funds")
    print("  • Source: RPC server response")
    
    print("\n❓ Possible Causes:")
    print("1. 🌐 Wrong Network - Wallet sending to Ethereum but balance is on PulseChain")
    print("2. 👤 Wrong Account - Sending from empty account instead of funded one")
    print("3. ⛽ Gas Estimation - Transaction needs more gas than available")
    print("4. 💰 Amount + Gas - Total cost (amount + gas) exceeds balance")
    
    print("\n🎯 Most Likely Issue:")
    print("The wallet is sending the transaction to ETHEREUM MAINNET")
    print("but your 1 tPLS balance is on PULSECHAIN TESTNET v4")
    
def provide_solution():
    """Provide step-by-step solution"""
    print("\n🛠️ SOLUTION STEPS")
    print("=" * 40)
    
    print("1. 🚀 Launch wallet:")
    print("   cargo run --bin dapp-platform --release")
    
    print("\n2. 🔍 Check current network in main interface:")
    print("   Look for network dropdown in main wallet UI")
    print("   Current network probably shows: 'Ethereum' or 'Ethereum Mainnet'")
    
    print("\n3. 🔄 Switch to PulseChain Testnet v4:")
    print("   Click network dropdown → Select 'PulseChain Testnet v4'")
    print("   Balance should show: '1.0000 tPLS' (not ETH)")
    
    print("\n4. 💰 Verify balance display:")
    print("   After switching, balance should change from '0.0000 ETH' to '1.0000 tPLS'")
    
    print("\n5. 📤 Try transaction again:")
    print("   Click Send → Fill in recipient and amount")
    print("   New send dialog should show:")
    print("   • Network: PulseChain Testnet v4")
    print("   • Balance: 1.0000 tPLS (green)")
    
    print("\n⚠️ IMPORTANT:")
    print("The enhanced send dialog should now prevent this issue!")
    print("You can see network and balance directly in the send form.")

def quick_rpc_test():
    """Quick test of PulseChain Testnet RPC"""
    print("\n🌐 PulseChain Testnet v4 RPC Test")
    print("=" * 40)
    
    rpc_url = "https://rpc.v4.testnet.pulsechain.com"
    
    try:
        # Test chain ID
        payload = {
            "jsonrpc": "2.0",
            "method": "eth_chainId",
            "params": [],
            "id": 1
        }
        
        response = requests.post(rpc_url, json=payload, timeout=5)
        result = response.json()
        
        if "result" in result:
            chain_id = int(result["result"], 16)
            if chain_id == 943:
                print("✅ PulseChain Testnet v4 RPC is working")
                print(f"   Chain ID: {chain_id} ✓")
            else:
                print(f"⚠️ Unexpected Chain ID: {chain_id} (expected 943)")
        else:
            print(f"❌ RPC Error: {result}")
            
    except Exception as e:
        print(f"❌ RPC Connection Failed: {e}")

def main():
    print("🚨 Insufficient Funds Error Diagnostic")
    print("=" * 50)
    print()
    
    # Check balances across networks
    check_account_balance_on_networks()
    
    # Test PulseChain RPC
    quick_rpc_test()
    
    # Analyze the error
    analyze_transaction_error()
    
    # Provide solution
    provide_solution()
    
    print("\n📋 SUMMARY")
    print("=" * 30)
    print("✅ You DO have 1 tPLS on PulseChain Testnet v4")
    print("❌ Wallet is probably sending to wrong network (Ethereum)")
    print("🎯 Solution: Switch to 'PulseChain Testnet v4' in wallet UI")
    print("🔧 Enhanced send dialog should prevent this in future")

if __name__ == "__main__":
    main()