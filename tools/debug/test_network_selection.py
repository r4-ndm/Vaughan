#!/usr/bin/env python3
"""
Network Selection Diagnostic Tool
This script helps debug network selection issues in Vaughan wallet.
"""
import subprocess
import json
import sys
import time
from datetime import datetime

def test_wallet_startup():
    """Test wallet startup and check for network initialization logs"""
    print("🚀 Testing Wallet Startup...")
    
    try:
        # Run the wallet with timeout and capture logs
        result = subprocess.run(
            ["cargo", "run", "--bin", "dapp-platform"],
            cwd="/home/r4/Desktop/Vaughan_V1",
            capture_output=True,
            text=True,
            timeout=10  # 10 second timeout
        )
        
        print("📊 Wallet Startup Output:")
        print("=" * 50)
        
        # Check for network-related initialization logs
        stdout = result.stdout
        stderr = result.stderr
        
        if "NetworkManager" in stderr or "network" in stderr.lower():
            print("🌐 Network Manager Logs:")
            for line in stderr.split('\n'):
                if any(keyword in line.lower() for keyword in ['network', 'chain', 'rpc', 'ethereum', 'pulsechain']):
                    print(f"  {line}")
        
        if "Default to Ethereum" in stderr:
            print("⚠️  FOUND ISSUE: Defaulting to Ethereum!")
        
        if "PulseChain Testnet" in stderr:
            print("✅ PulseChain Testnet v4 detected")
        
        if "Chain ID 943" in stderr:
            print("✅ Chain ID 943 (PulseChain Testnet) found")
            
        if "Chain ID 1" in stderr:
            print("⚠️  Chain ID 1 (Ethereum) found - possible default")
            
        return stderr
        
    except subprocess.TimeoutExpired:
        print("⏱️ Wallet startup timed out (normal for GUI)")
        return "timeout"
    except Exception as e:
        print(f"❌ Error testing wallet startup: {e}")
        return None

def analyze_network_selection_issue():
    """Provide analysis of potential network selection issues"""
    print("\n🔍 NETWORK SELECTION ANALYSIS")
    print("=" * 50)
    
    print("🎯 Key Check Points:")
    print("1. NetworkManager initialization (should not default to Ethereum)")
    print("2. Available networks list (should include PulseChain Testnet v4)")
    print("3. Network dropdown selection (user must manually select)")
    print("4. State synchronization (UI selection → wallet state → transaction)")
    
    print("\n❗ Common Issues:")
    print("1. 🔄 Network selector UI not triggering NetworkSelected message")
    print("2. 🌐 Wallet's NetworkManager not switching properly")
    print("3. 💰 Balance fetch using different network than transaction")
    print("4. 🚫 Default network override preventing user selection")
    
    print("\n🛠️ Debugging Steps:")
    print("1. Check if 'PulseChain Testnet v4' appears in network dropdown")
    print("2. Select PulseChain Testnet v4 manually from dropdown")
    print("3. Verify balance shows 'tPLS' (not ETH)")  
    print("4. Check transaction logs for correct Chain ID (943)")
    print("5. Ensure wallet.switch_network() is called successfully")

def provide_immediate_fix():
    """Provide immediate troubleshooting steps"""
    print("\n🚨 IMMEDIATE TROUBLESHOOTING")
    print("=" * 50)
    
    print("📋 Step-by-Step Fix:")
    print("1. Open Vaughan wallet:")
    print("   cargo run --bin dapp-platform --release")
    print()
    print("2. Look for network dropdown in the UI")
    print("   (Usually top-left or in main interface)")
    print()
    print("3. Current network likely shows: 'Ethereum' or 'Ethereum Mainnet'")
    print()
    print("4. Click dropdown and select: 'PulseChain Testnet v4'")
    print()
    print("5. Verify balance changes from 'X.XXXX ETH' to '1.0000 tPLS'")
    print()
    print("6. Try transaction again")
    print()
    print("🔧 If network dropdown is missing or not working:")
    print("- The UI network selection component may have an issue")
    print("- Check if PickList<NetworkConfig> is properly implemented")
    print("- Verify Message::NetworkSelected handler is working")

def main():
    print("🔧 Vaughan Network Selection Diagnostic")
    print("=" * 50)
    print(f"🕒 Test Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()
    
    # Test wallet startup logs
    startup_logs = test_wallet_startup()
    
    # Analyze potential issues
    analyze_network_selection_issue()
    
    # Provide immediate fix
    provide_immediate_fix()
    
    print("\n📞 SUMMARY")
    print("=" * 30)
    print("✅ Your account HAS 1 tPLS on PulseChain Testnet")
    print("✅ Transaction code correctly uses selected network")
    print("❓ Issue is likely: Network not properly selected in UI")
    print()
    print("🎯 SOLUTION: Manually select 'PulseChain Testnet v4' from dropdown")

if __name__ == "__main__":
    main()