#!/usr/bin/env python3
"""
Simple Network Diagnostic Tool for Vaughan Wallet
Uses only built-in Python libraries to diagnose RPC connectivity issues
"""

import json
import socket
import ssl
import urllib.request
import urllib.parse
import urllib.error
import time
import subprocess
import sys
import platform

# PulseChain Testnet v4 RPC endpoints to test
PULSECHAIN_TESTNET_RPCS = [
    "https://rpc.v4.testnet.pulsechain.com",
    "https://rpc-testnet-pulsechain.g4mm4.io", 
    "https://pulsechain-testnet.publicnode.com"
]

# Your account address
ACCOUNT_ADDRESS = "0x742D35B4aC0EA09d926D0e37a59eAeE71D3E4143"

def test_basic_connectivity():
    """Test basic internet connectivity"""
    print("🌐 Testing basic internet connectivity...")
    
    try:
        response = urllib.request.urlopen('https://httpbin.org/ip', timeout=10)
        if response.status == 200:
            data = json.loads(response.read().decode())
            print(f"  ✅ Internet connection OK (IP: {data.get('origin', 'Unknown')})")
            return True
        else:
            print(f"  ❌ Internet connection failed: HTTP {response.status}")
            return False
    except Exception as e:
        print(f"  ❌ Internet connection FAILED: {e}")
        return False

def test_dns_resolution():
    """Test DNS resolution for PulseChain endpoints"""
    print("\n🔍 Testing DNS resolution...")
    
    success_count = 0
    for rpc_url in PULSECHAIN_TESTNET_RPCS:
        parsed = urllib.parse.urlparse(rpc_url)
        hostname = parsed.hostname
        
        try:
            ip = socket.gethostbyname(hostname)
            print(f"  ✅ {hostname} -> {ip}")
            success_count += 1
        except Exception as e:
            print(f"  ❌ {hostname} DNS resolution failed: {e}")
    
    return success_count > 0

def test_ssl_connectivity():
    """Test SSL/TLS connectivity to RPC endpoints"""
    print("\n🔐 Testing SSL/TLS connectivity...")
    
    success_count = 0
    for rpc_url in PULSECHAIN_TESTNET_RPCS:
        parsed = urllib.parse.urlparse(rpc_url)
        hostname = parsed.hostname
        port = parsed.port or 443
        
        try:
            context = ssl.create_default_context()
            with socket.create_connection((hostname, port), timeout=10) as sock:
                with context.wrap_socket(sock, server_hostname=hostname) as ssock:
                    print(f"  ✅ {hostname} SSL OK (TLS {ssock.version()})")
                    success_count += 1
        except Exception as e:
            print(f"  ❌ {hostname} SSL failed: {e}")
    
    return success_count > 0

def make_rpc_call(url, method, params=None, timeout=15):
    """Make a JSON-RPC call to an endpoint"""
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
        url,
        data=json_data,
        headers={
            'Content-Type': 'application/json',
            'User-Agent': 'Vaughan-Wallet-Debug/1.0'
        }
    )
    
    start_time = time.time()
    try:
        response = urllib.request.urlopen(req, timeout=timeout)
        response_time = time.time() - start_time
        
        if response.status == 200:
            result = json.loads(response.read().decode())
            return {
                'status': 'OK',
                'response_time': response_time,
                'result': result
            }
        else:
            return {
                'status': 'FAILED',
                'error': f'HTTP {response.status}'
            }
    except Exception as e:
        response_time = time.time() - start_time
        return {
            'status': 'FAILED', 
            'error': str(e),
            'response_time': response_time
        }

def test_rpc_endpoints():
    """Test RPC endpoints with various methods"""
    print("\n⚡ Testing RPC endpoints...")
    
    working_endpoints = []
    
    for rpc_url in PULSECHAIN_TESTNET_RPCS:
        print(f"\n  Testing {rpc_url}:")
        endpoint_working = True
        
        # Test client version
        result = make_rpc_call(rpc_url, "web3_clientVersion")
        if result['status'] == 'OK':
            client_version = result['result'].get('result', 'Unknown')
            print(f"    ✅ Client version OK ({result['response_time']:.2f}s): {client_version}")
        else:
            print(f"    ❌ Client version failed: {result['error']}")
            endpoint_working = False
        
        # Test chain ID
        result = make_rpc_call(rpc_url, "eth_chainId")
        if result['status'] == 'OK':
            chain_id_hex = result['result'].get('result', '0x0')
            chain_id = int(chain_id_hex, 16) if chain_id_hex else 0
            if chain_id == 943:
                print(f"    ✅ Chain ID OK ({result['response_time']:.2f}s): {chain_id} (PulseChain Testnet v4)")
            else:
                print(f"    ⚠️ Chain ID mismatch ({result['response_time']:.2f}s): {chain_id} (expected 943)")
                endpoint_working = False
        else:
            print(f"    ❌ Chain ID failed: {result['error']}")
            endpoint_working = False
            
        # Test account balance
        result = make_rpc_call(rpc_url, "eth_getBalance", [ACCOUNT_ADDRESS, "latest"])
        if result['status'] == 'OK':
            balance_hex = result['result'].get('result', '0x0')
            if balance_hex:
                balance_wei = int(balance_hex, 16)
                balance_eth = balance_wei / 10**18
                print(f"    ✅ Balance OK ({result['response_time']:.2f}s): {balance_eth:.6f} tPLS")
            else:
                print(f"    ❌ Balance check failed: No result in response")
                endpoint_working = False
        else:
            print(f"    ❌ Balance check failed: {result['error']}")
            endpoint_working = False
            
        # Test latest block number
        result = make_rpc_call(rpc_url, "eth_blockNumber")
        if result['status'] == 'OK':
            block_hex = result['result'].get('result', '0x0')
            if block_hex:
                block_number = int(block_hex, 16)
                print(f"    ✅ Latest block OK ({result['response_time']:.2f}s): {block_number}")
            else:
                print(f"    ❌ Latest block failed: No result in response")
                endpoint_working = False
        else:
            print(f"    ❌ Latest block failed: {result['error']}")
            endpoint_working = False
        
        if endpoint_working:
            working_endpoints.append(rpc_url)
    
    return working_endpoints

def check_system_configuration():
    """Check system configuration that might affect networking"""
    print("\n🔧 Checking system configuration...")
    
    # Check OS
    print(f"  OS: {platform.platform()}")
    
    # Check network configuration
    try:
        result = subprocess.run(['ip', 'route', 'show', 'default'], 
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            default_route = result.stdout.strip().split()[:3]
            print(f"  Default route: {' '.join(default_route)}")
    except:
        print("  Default route: Unable to check")
        
    # Check DNS configuration
    try:
        with open('/etc/resolv.conf', 'r') as f:
            dns_count = sum(1 for line in f if line.startswith('nameserver'))
            print(f"  DNS servers: {dns_count} configured")
    except:
        print("  DNS servers: Unable to check")
        
    # Check for proxy
    import os
    proxy_vars = ['HTTP_PROXY', 'HTTPS_PROXY', 'http_proxy', 'https_proxy']
    active_proxies = [var for var in proxy_vars if os.environ.get(var)]
    
    if active_proxies:
        print(f"  ⚠️ Proxy detected: {active_proxies}")
    else:
        print("  No proxy detected")
        
    # Check firewall status (basic check)
    try:
        result = subprocess.run(['ufw', 'status'], 
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            firewall_status = result.stdout.strip().split('\n')[0]
            print(f"  Firewall: {firewall_status}")
    except:
        print("  Firewall: Unable to check (ufw not found)")

def check_wallet_process():
    """Check if wallet process is running and may be interfering"""
    print("\n🔍 Checking for running wallet processes...")
    
    try:
        result = subprocess.run(['pgrep', '-f', 'dapp-platform'], 
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            pids = result.stdout.strip().split('\n')
            print(f"  ⚠️ Found {len(pids)} running wallet process(es): {', '.join(pids)}")
            print("    Consider stopping the wallet before running network tests")
        else:
            print("  ✅ No wallet processes currently running")
    except:
        print("  Unable to check for wallet processes")

def analyze_and_recommend(working_endpoints, has_internet, has_dns, has_ssl):
    """Analyze results and provide recommendations"""
    print("\n" + "="*60)
    print("📊 DIAGNOSTIC SUMMARY")
    print("="*60)
    
    issues = []
    recommendations = []
    
    if not has_internet:
        issues.append("❌ No internet connectivity")
        recommendations.append("1. Check your internet connection")
        recommendations.append("2. Restart your network manager")
        
    if not has_dns:
        issues.append("❌ DNS resolution problems")
        recommendations.append("3. Check DNS servers in /etc/resolv.conf")
        recommendations.append("4. Try: sudo systemctl restart systemd-resolved")
        
    if not has_ssl:
        issues.append("❌ SSL/TLS connection problems")
        recommendations.append("5. Check system time: timedatectl")
        recommendations.append("6. Update CA certificates: sudo pacman -S ca-certificates")
        
    if not working_endpoints:
        issues.append("❌ No RPC endpoints are working")
        recommendations.append("7. Try using a VPN or different network")
        recommendations.append("8. Check if your ISP blocks crypto-related traffic")
        recommendations.append("9. Temporarily disable firewall: sudo ufw disable")
        
    if issues:
        print("\n🚨 ISSUES FOUND:")
        for issue in issues:
            print(f"  {issue}")
            
        print("\n💡 RECOMMENDATIONS:")
        for rec in recommendations:
            print(f"  {rec}")
    else:
        print("\n✅ All network tests passed!")
        
    if working_endpoints:
        print(f"\n🔧 WALLET CONFIGURATION:")
        print(f"  Working RPC endpoints found: {len(working_endpoints)}")
        print(f"  Recommended endpoint: {working_endpoints[0]}")
        print(f"  Chain ID: 943")
        print(f"  Network: PulseChain Testnet v4")
        print(f"  Currency: tPLS")
        
        print(f"\n🛠️ QUICK FIX FOR WALLET:")
        print(f"  Edit your wallet's network configuration to use:")
        print(f"  RPC URL: {working_endpoints[0]}")
        
        # Provide Rust wallet configuration
        print(f"\n📋 FOR RUST WALLET CODE:")
        print(f"  Update NetworkConfig::pulsechain_testnet() in src/network/mod.rs:")
        print(f"  rpc_url: \"{working_endpoints[0]}\".to_string(),")

def main():
    """Run all network diagnostics"""
    print("🚀 Starting simple network diagnostics for Vaughan Wallet...\n")
    
    # Run tests
    has_internet = test_basic_connectivity()
    has_dns = test_dns_resolution()
    has_ssl = test_ssl_connectivity()
    working_endpoints = test_rpc_endpoints()
    
    check_system_configuration()
    check_wallet_process()
    
    # Analyze and provide recommendations
    analyze_and_recommend(working_endpoints, has_internet, has_dns, has_ssl)
    
    # Test with curl as fallback
    print(f"\n🧪 CURL TEST (as fallback):")
    print("If RPC tests failed, try this curl command manually:")
    print('curl -X POST https://rpc.v4.testnet.pulsechain.com \\')
    print('  -H "Content-Type: application/json" \\')
    print('  -d \'{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}\'')

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n⏹️ Diagnostic cancelled by user")
    except Exception as e:
        print(f"\n\n💥 Diagnostic failed: {e}")
        import traceback
        traceback.print_exc()