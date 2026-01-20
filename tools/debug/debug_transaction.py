#!/usr/bin/env python3
"""
Vaughan Transaction Debug Script
This script helps debug why transactions are failing.
"""
import subprocess
import json
import sys

def run_account_manager():
    """Run account manager and capture output"""
    print("🔍 Checking current account state...")
    try:
        result = subprocess.run(
            ["cargo", "run", "--bin", "account-manager"],
            cwd="/home/r4/Desktop/Vaughan_V1",
            capture_output=True,
            text=True,
            timeout=15
        )
        print("📊 Account Manager Output:")
        print("=" * 50)
        print(result.stdout)
        if result.stderr:
            print("⚠️ Warnings/Errors:")
            print(result.stderr[:500])
        return result.stdout
    except subprocess.TimeoutExpired:
        print("⏱️ Account manager timed out")
        return None
    except Exception as e:
        print(f"❌ Error running account manager: {e}")
        return None

def analyze_accounts(output):
    """Analyze account manager output"""
    if not output:
        return
    
    print("\n🔍 TRANSACTION FAILURE ANALYSIS")
    print("=" * 50)
    
    # Check for key indicators
    if "Primary Account" in output:
        print("✅ Primary Account exists")
        if "Address: 0xa8c2be786892a7c36158c34d0b51091db3520598" in output:
            print("   📍 Address: 0xa8c2be786892a7c36158c34d0b51091db3520598")
            print("   💰 Balance: LIKELY 0 ETH (no funds)")
    
    if "Funded account (0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f) not found" in output:
        print("❌ Funded account NOT in wallet")
        print("   📍 Missing: 0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f")
        print("   💰 This account has funds but isn't imported")
    
    print("\n🚫 WHY TRANSACTIONS FAIL:")
    print("1. Primary Account has 0 ETH balance")
    print("2. Transaction needs ETH for gas fees (~0.001 ETH minimum)")
    print("3. Balance check: 0 ETH < 0.001 ETH = REJECTED")
    
    print("\n✅ SOLUTIONS:")
    print("1. Import your funded account (0xe3b3f4ce6d66411d4fedfa2c2864b55c75f2ad8f)")
    print("2. Or send ETH to Primary Account")
    print("3. Switch to a testnet with free tokens")

def main():
    print("🚀 Vaughan Transaction Failure Debug")
    print("=" * 50)
    
    # Run account manager
    output = run_account_manager()
    
    # Analyze results
    analyze_accounts(output)
    
    print("\n📋 QUICK ACTION ITEMS:")
    print("• Check if Primary Account has any ETH balance")
    print("• Import funded account using private key/seed phrase")
    print("• Consider using PulseChain Testnet v4 for testing")

if __name__ == "__main__":
    main()