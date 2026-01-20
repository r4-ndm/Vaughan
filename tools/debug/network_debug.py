#!/usr/bin/env python3
"""
Advanced Network Diagnostic Tool for Vaughan Wallet
Diagnoses RPC connectivity, DNS, SSL, and wallet configuration issues
"""

import asyncio
import aiohttp
import json
import time
import socket
import ssl
import subprocess
import sys
from urllib.parse import urlparse
import dns.resolver
import platform

# PulseChain Testnet v4 RPC endpoints to test
PULSECHAIN_TESTNET_RPCS = [
    "https://rpc.v4.testnet.pulsechain.com",
    "https://rpc-testnet-pulsechain.g4mm4.io",
    "https://pulsechain-testnet.publicnode.com"
]

# Your account address
ACCOUNT_ADDRESS = "0x742D35B4aC0EA09d926D0e37a59eAeE71D3E4143"

class NetworkDiagnostic:
    def __init__(self):
        self.results = {}
        self.session = None
        
    async def create_session(self):
        """Create HTTP session with various timeout configurations"""
        timeout = aiohttp.ClientTimeout(
            total=30,
            connect=10,
            sock_read=15,
            sock_connect=10
        )
        
        connector = aiohttp.TCPConnector(
            limit=10,
            limit_per_host=5,
            ttl_dns_cache=300,
            use_dns_cache=True,
            enable_cleanup_closed=True,
            force_close=True,
            ssl=ssl.create_default_context()
        )
        
        self.session = aiohttp.ClientSession(
            timeout=timeout,
            connector=connector,
            headers={
                'User-Agent': 'Vaughan-Wallet-Debug/1.0',
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            }
        )

    async def test_basic_connectivity(self):
        """Test basic internet connectivity"""
        print("🌐 Testing basic internet connectivity...")
        
        try:
            async with self.session.get('https://httpbin.org/ip', timeout=10) as response:
                if response.status == 200:
                    data = await response.json()
                    self.results['internet'] = {
                        'status': 'OK',
                        'ip': data.get('origin', 'Unknown')
                    }
                    print(f"  ✅ Internet connection OK (IP: {data.get('origin', 'Unknown')})")
                else:
                    raise Exception(f"HTTP {response.status}")
                    
        except Exception as e:
            self.results['internet'] = {
                'status': 'FAILED',
                'error': str(e)
            }
            print(f"  ❌ Internet connection FAILED: {e}")

    async def test_dns_resolution(self):
        """Test DNS resolution for PulseChain endpoints"""
        print("\n🔍 Testing DNS resolution...")
        
        dns_results = {}
        for rpc_url in PULSECHAIN_TESTNET_RPCS:
            hostname = urlparse(rpc_url).hostname
            try:
                # Test with system resolver
                result = socket.gethostbyname(hostname)
                dns_results[hostname] = {
                    'status': 'OK',
                    'ip': result
                }
                print(f"  ✅ {hostname} -> {result}")
                
                # Also test with different DNS resolver
                resolver = dns.resolver.Resolver()
                resolver.nameservers = ['8.8.8.8', '1.1.1.1']  # Google and Cloudflare DNS
                try:
                    answers = resolver.resolve(hostname, 'A')
                    alt_ips = [str(rdata) for rdata in answers]
                    dns_results[hostname]['alt_ips'] = alt_ips
                except:
                    pass
                    
            except Exception as e:
                dns_results[hostname] = {
                    'status': 'FAILED',
                    'error': str(e)
                }
                print(f"  ❌ {hostname} DNS resolution failed: {e}")
                
        self.results['dns'] = dns_results

    async def test_ssl_connectivity(self):
        """Test SSL/TLS connectivity to RPC endpoints"""
        print("\n🔐 Testing SSL/TLS connectivity...")
        
        ssl_results = {}
        for rpc_url in PULSECHAIN_TESTNET_RPCS:
            parsed = urlparse(rpc_url)
            hostname = parsed.hostname
            port = parsed.port or 443
            
            try:
                # Test SSL handshake
                context = ssl.create_default_context()
                with socket.create_connection((hostname, port), timeout=10) as sock:
                    with context.wrap_socket(sock, server_hostname=hostname) as ssock:
                        cert = ssock.getpeercert()
                        ssl_results[rpc_url] = {
                            'status': 'OK',
                            'cert_subject': dict(x[0] for x in cert['subject']),
                            'cert_issuer': dict(x[0] for x in cert['issuer']),
                            'version': ssock.version()
                        }
                        print(f"  ✅ {hostname} SSL OK (TLS {ssock.version()})")
                        
            except Exception as e:
                ssl_results[rpc_url] = {
                    'status': 'FAILED',
                    'error': str(e)
                }
                print(f"  ❌ {hostname} SSL failed: {e}")
                
        self.results['ssl'] = ssl_results

    async def test_rpc_endpoints(self):
        """Test RPC endpoints with various methods"""
        print("\n⚡ Testing RPC endpoints...")
        
        rpc_results = {}
        for rpc_url in PULSECHAIN_TESTNET_RPCS:
            print(f"\n  Testing {rpc_url}:")
            endpoint_results = {}
            
            # Test basic connectivity
            try:
                start_time = time.time()
                async with self.session.post(rpc_url, 
                    json={
                        "jsonrpc": "2.0",
                        "method": "web3_clientVersion",
                        "params": [],
                        "id": 1
                    },
                    timeout=15
                ) as response:
                    response_time = time.time() - start_time
                    
                    if response.status == 200:
                        data = await response.json()
                        endpoint_results['web3_clientVersion'] = {
                            'status': 'OK',
                            'response_time': response_time,
                            'result': data.get('result', 'Unknown')
                        }
                        print(f"    ✅ Client version OK ({response_time:.2f}s): {data.get('result', 'Unknown')}")
                    else:
                        raise Exception(f"HTTP {response.status}")
                        
            except Exception as e:
                endpoint_results['web3_clientVersion'] = {
                    'status': 'FAILED',
                    'error': str(e)
                }
                print(f"    ❌ Client version failed: {e}")

            # Test chain ID
            try:
                start_time = time.time()
                async with self.session.post(rpc_url,
                    json={
                        "jsonrpc": "2.0", 
                        "method": "eth_chainId",
                        "params": [],
                        "id": 1
                    },
                    timeout=15
                ) as response:
                    response_time = time.time() - start_time
                    
                    if response.status == 200:
                        data = await response.json()
                        chain_id = int(data.get('result', '0x0'), 16)
                        endpoint_results['eth_chainId'] = {
                            'status': 'OK',
                            'response_time': response_time,
                            'chain_id': chain_id
                        }
                        if chain_id == 943:
                            print(f"    ✅ Chain ID OK ({response_time:.2f}s): {chain_id} (PulseChain Testnet v4)")
                        else:
                            print(f"    ⚠️ Chain ID mismatch ({response_time:.2f}s): {chain_id} (expected 943)")
                    else:
                        raise Exception(f"HTTP {response.status}")
                        
            except Exception as e:
                endpoint_results['eth_chainId'] = {
                    'status': 'FAILED',
                    'error': str(e)
                }
                print(f"    ❌ Chain ID failed: {e}")

            # Test account balance
            try:
                start_time = time.time()
                async with self.session.post(rpc_url,
                    json={
                        "jsonrpc": "2.0",
                        "method": "eth_getBalance", 
                        "params": [ACCOUNT_ADDRESS, "latest"],
                        "id": 1
                    },
                    timeout=15
                ) as response:
                    response_time = time.time() - start_time
                    
                    if response.status == 200:
                        data = await response.json()
                        if 'result' in data:
                            balance_wei = int(data['result'], 16)
                            balance_eth = balance_wei / 10**18
                            endpoint_results['eth_getBalance'] = {
                                'status': 'OK',
                                'response_time': response_time,
                                'balance_wei': balance_wei,
                                'balance_eth': balance_eth
                            }
                            print(f"    ✅ Balance OK ({response_time:.2f}s): {balance_eth:.6f} tPLS")
                        else:
                            raise Exception(f"No result in response: {data}")
                    else:
                        raise Exception(f"HTTP {response.status}")
                        
            except Exception as e:
                endpoint_results['eth_getBalance'] = {
                    'status': 'FAILED', 
                    'error': str(e)
                }
                print(f"    ❌ Balance check failed: {e}")

            # Test latest block number
            try:
                start_time = time.time()
                async with self.session.post(rpc_url,
                    json={
                        "jsonrpc": "2.0",
                        "method": "eth_blockNumber",
                        "params": [],
                        "id": 1
                    },
                    timeout=15
                ) as response:
                    response_time = time.time() - start_time
                    
                    if response.status == 200:
                        data = await response.json()
                        block_number = int(data.get('result', '0x0'), 16)
                        endpoint_results['eth_blockNumber'] = {
                            'status': 'OK',
                            'response_time': response_time,
                            'block_number': block_number
                        }
                        print(f"    ✅ Latest block OK ({response_time:.2f}s): {block_number}")
                    else:
                        raise Exception(f"HTTP {response.status}")
                        
            except Exception as e:
                endpoint_results['eth_blockNumber'] = {
                    'status': 'FAILED',
                    'error': str(e)
                }
                print(f"    ❌ Latest block failed: {e}")
                
            rpc_results[rpc_url] = endpoint_results
            
        self.results['rpc_endpoints'] = rpc_results

    def check_system_configuration(self):
        """Check system configuration that might affect networking"""
        print("\n🔧 Checking system configuration...")
        
        system_config = {}
        
        # Check OS and kernel version
        system_config['os'] = {
            'platform': platform.platform(),
            'system': platform.system(),
            'release': platform.release(),
            'version': platform.version()
        }
        print(f"  OS: {platform.platform()}")
        
        # Check network configuration
        try:
            result = subprocess.run(['ip', 'route', 'show', 'default'], 
                                  capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                system_config['default_route'] = result.stdout.strip()
                print(f"  Default route: {result.stdout.strip().split()[0:3]}")
        except:
            pass
            
        # Check DNS configuration
        try:
            with open('/etc/resolv.conf', 'r') as f:
                dns_servers = []
                for line in f:
                    if line.startswith('nameserver'):
                        dns_servers.append(line.strip())
                system_config['dns_servers'] = dns_servers
                print(f"  DNS servers: {len(dns_servers)} configured")
        except:
            pass
            
        # Check if using proxy
        proxy_vars = ['HTTP_PROXY', 'HTTPS_PROXY', 'http_proxy', 'https_proxy']
        proxies = {}
        for var in proxy_vars:
            import os
            if os.environ.get(var):
                proxies[var] = os.environ[var]
        
        if proxies:
            system_config['proxy'] = proxies
            print(f"  ⚠️ Proxy detected: {list(proxies.keys())}")
        else:
            print(f"  No proxy detected")
            
        # Check firewall status
        try:
            result = subprocess.run(['ufw', 'status'], 
                                  capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                firewall_status = result.stdout.strip().split('\n')[0]
                system_config['firewall'] = firewall_status
                print(f"  Firewall: {firewall_status}")
        except:
            pass
            
        self.results['system_config'] = system_config

    def analyze_results(self):
        """Analyze all test results and provide recommendations"""
        print("\n" + "="*60)
        print("📊 DIAGNOSTIC SUMMARY")
        print("="*60)
        
        issues = []
        recommendations = []
        
        # Check internet connectivity
        if self.results.get('internet', {}).get('status') != 'OK':
            issues.append("❌ No internet connectivity")
            recommendations.append("1. Check your internet connection")
            recommendations.append("2. Verify network adapter is working")
            
        # Check DNS resolution
        dns_results = self.results.get('dns', {})
        failed_dns = [host for host, result in dns_results.items() 
                     if result.get('status') != 'OK']
        if failed_dns:
            issues.append(f"❌ DNS resolution failed for: {', '.join(failed_dns)}")
            recommendations.append("3. Try switching DNS servers (8.8.8.8, 1.1.1.1)")
            recommendations.append("4. Check /etc/resolv.conf configuration")
            
        # Check SSL connectivity
        ssl_results = self.results.get('ssl', {})
        failed_ssl = [url for url, result in ssl_results.items() 
                     if result.get('status') != 'OK']
        if failed_ssl:
            issues.append(f"❌ SSL/TLS connection failed for {len(failed_ssl)} endpoints")
            recommendations.append("5. Check system time and date")
            recommendations.append("6. Update CA certificates")
            
        # Check RPC endpoints
        rpc_results = self.results.get('rpc_endpoints', {})
        working_endpoints = []
        for url, tests in rpc_results.items():
            if all(test.get('status') == 'OK' for test in tests.values()):
                working_endpoints.append(url)
                
        if not working_endpoints:
            issues.append("❌ No RPC endpoints are working")
            recommendations.append("7. Check firewall settings")
            recommendations.append("8. Try connecting from a different network")
        else:
            print(f"✅ Working RPC endpoints: {len(working_endpoints)}")
            for endpoint in working_endpoints:
                print(f"   - {endpoint}")
                
        # Check for proxy issues
        if self.results.get('system_config', {}).get('proxy'):
            issues.append("⚠️ Proxy detected - may interfere with RPC calls")
            recommendations.append("9. Try disabling proxy temporarily")
            
        if issues:
            print("\n🚨 ISSUES FOUND:")
            for issue in issues:
                print(f"  {issue}")
                
            print("\n💡 RECOMMENDATIONS:")
            for rec in recommendations:
                print(f"  {rec}")
        else:
            print("\n✅ All network tests passed!")
            
        # Provide specific wallet configuration
        if working_endpoints:
            print(f"\n🔧 WALLET CONFIGURATION:")
            print(f"  Use this RPC endpoint in your wallet: {working_endpoints[0]}")
            print(f"  Chain ID: 943 (PulseChain Testnet v4)")
            print(f"  Network Name: PulseChain Testnet v4")
            print(f"  Currency Symbol: tPLS")

    async def run_all_tests(self):
        """Run all diagnostic tests"""
        print("🚀 Starting comprehensive network diagnostics...\n")
        
        await self.create_session()
        
        try:
            await self.test_basic_connectivity()
            await self.test_dns_resolution()
            await self.test_ssl_connectivity()
            await self.test_rpc_endpoints()
            self.check_system_configuration()
            self.analyze_results()
            
        finally:
            if self.session:
                await self.session.close()
                
        # Save detailed results
        with open('/home/r4/Desktop/Vaughan_V1/network_diagnostic_results.json', 'w') as f:
            json.dump(self.results, f, indent=2, default=str)
            
        print(f"\n📄 Detailed results saved to: network_diagnostic_results.json")

async def main():
    diagnostic = NetworkDiagnostic()
    await diagnostic.run_all_tests()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n⏹️ Diagnostic cancelled by user")
    except Exception as e:
        print(f"\n\n💥 Diagnostic failed: {e}")
        import traceback
        traceback.print_exc()