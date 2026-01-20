# Cross-Chain Bridge Integration

This document provides guidance for integrating cross-chain bridge functionality into the Vaughan wallet, specifically focusing on the PulseChain Bridge and other major bridge providers.

## Overview

The bridge integration provides:
- Native API integration with PulseChain Bridge (no web interface needed)
- Support for multiple bridge providers (PulseChain, Multichain, Hop Protocol)
- Real-time quote fetching and comparison
- Transaction monitoring and status updates
- Secure bridge execution through the wallet's existing security infrastructure

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   Bridge Tab    │ -> │  Bridge Client   │ -> │  Bridge API Server  │
│     (GUI)       │    │   (src/bridge)   │    │  (External Service) │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
```

### Components

1. **Bridge Tab UI** (`src/gui/tabs/mod.rs`)
   - Native Iced-based interface
   - No web components or webview
   - Real-time updates and status indicators

2. **Bridge API Client** (`src/bridge/mod.rs`)
   - HTTP API integration using `reqwest`
   - Provider-specific implementations
   - Error handling and retry logic

3. **Bridge State Management** (`src/gui/mod.rs`)
   - Reactive state management
   - Transaction status tracking
   - Quote caching and comparison

## Supported Bridge Providers

### 1. PulseChain Bridge (Primary)
- **API Endpoint**: `https://bridge.mypinata.cloud/api`
- **Supported Networks**: Ethereum ↔ PulseChain
- **Supported Tokens**: PLS, HEX, USDC, ETH
- **Fee**: 0.10%
- **Status**: ✅ Implemented

### 2. Multichain
- **API Endpoint**: `https://bridgeapi.multichain.org`
- **Supported Networks**: Ethereum, BSC, Polygon, PulseChain
- **Fee**: 0.10%
- **Status**: 🔄 Placeholder (ready for implementation)

### 3. Hop Protocol
- **API Endpoint**: `https://api.hop.exchange`
- **Supported Networks**: Ethereum, Optimism, Arbitrum
- **Fee**: 0.04%
- **Status**: 🔄 Placeholder (ready for implementation)

## Features

### ✅ Currently Implemented

1. **Complete Bridge UI**
   - Network selection (source/target)
   - Token selection with balance display
   - Amount input with MAX button
   - Network swap functionality
   - Bridge provider selection
   - Quote request and display
   - Transaction execution
   - Status tracking with visual indicators

2. **Bridge State Management**
   - Reactive state updates
   - Transaction status enum
   - Quote caching
   - Balance tracking
   - Provider management

3. **API Infrastructure**
   - HTTP client setup
   - Request/response structures
   - Error handling
   - Provider abstraction

### 🚧 Ready for Integration

1. **PulseChain Bridge API**
   - Quote endpoint: `POST /api/quote`
   - Execute endpoint: `POST /api/execute`
   - Status endpoint: `GET /api/status/{tx_hash}`
   - Tokens endpoint: `GET /api/tokens`

2. **Transaction Monitoring**
   - Real-time status updates
   - Cross-chain confirmation tracking
   - Error handling and retry logic

## API Integration Guide

### Step 1: Configure API Endpoints

The bridge client is pre-configured with the PulseChain Bridge endpoint:

```rust
// In src/bridge/mod.rs
pulsechain_bridge_url: "https://bridge.mypinata.cloud/api".to_string(),
```

### Step 2: Implement Real API Calls

The current implementation includes simulation functions. To integrate with real APIs:

1. **Replace simulation with real API calls** in `src/gui/dapp_platform.rs`:
   ```rust
   // Replace this simulation:
   Command::perform(
       get_bridge_quote(...),  // Simulation function
       Message::BridgeQuoteReceived
   )
   
   // With real API integration:
   Command::perform(
       bridge_client.get_quote(...),  // Real API call
       Message::BridgeQuoteReceived
   )
   ```

2. **Update API request structures** to match the actual PulseChain Bridge API specification.

### Step 3: Handle Authentication (if required)

If the bridge API requires authentication:

```rust
// In BridgeClient::new()
client: Client::builder()
    .default_headers({
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", "Bearer YOUR_API_KEY".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    })
    .build()
    .unwrap(),
```

### Step 4: Error Handling

The bridge implementation includes comprehensive error handling:

```rust
pub enum BridgeError {
    Request(reqwest::Error),           // HTTP errors
    UnsupportedProvider(String),       // Invalid provider
    NotImplemented(String),            // API not ready
    InvalidResponse(String),           // Malformed response
    BridgeOperationFailed(String),     // Bridge-specific errors
}
```

## Testing the Bridge Tab

### 1. Run the DApp Platform

```bash
cargo run --bin dapp-platform --release
```

### 2. Navigate to Bridge Tab

Click on "T8: Bridge" in the tab navigation.

### 3. Test Bridge Operations

1. **Select Networks**: Choose source (PulseChain) and target (Ethereum)
2. **Select Token**: Choose from PLS, HEX, ETH, USDC
3. **Enter Amount**: Input amount or click MAX
4. **Get Quote**: Click "🔍 Get Bridge Quote" 
5. **Execute Bridge**: Click "🌉 Execute Bridge"

### 4. Monitor Transaction

The status section will show:
- Quote fetching progress
- Transaction execution status
- Bridge route steps
- Completion confirmation

## API Response Examples

### Quote Request
```json
{
  "source_chain": "pulsechain",
  "target_chain": "ethereum",
  "token": "PLS",
  "amount": "1000.0"
}
```

### Quote Response
```json
{
  "source_amount": "1000.0",
  "target_amount": "0.045000",
  "exchange_rate": 0.000045,
  "estimated_minutes": 8,
  "gas_cost_usd": 0.01,
  "bridge_fee": 1.0,
  "total_fee_usd": 15.01,
  "quote_id": "quote_123456",
  "route": [
    {
      "step_type": "lock",
      "description": "Lock 1000.0 PLS on source network",
      "estimated_minutes": 2
    },
    {
      "step_type": "validate",
      "description": "Cross-chain validation and confirmation",
      "estimated_minutes": 4
    },
    {
      "step_type": "mint",
      "description": "Mint 0.044000 tokens on target network",
      "estimated_minutes": 2
    }
  ]
}
```

## Security Considerations

1. **API Key Management**
   - Store API keys securely using the wallet's keychain integration
   - Never hardcode API keys in source code

2. **Transaction Validation**
   - Always validate bridge quotes before execution
   - Implement slippage protection
   - Verify destination addresses

3. **Network Security**
   - Use HTTPS for all API calls
   - Implement request timeouts
   - Validate SSL certificates

## Customization Options

### Adding New Bridge Providers

1. **Add provider configuration**:
   ```rust
   BridgeProvider {
       id: "new_bridge".to_string(),
       name: "New Bridge".to_string(),
       description: "A new bridge provider".to_string(),
       supported_networks: vec![NetworkId(1), NetworkId(56)],
       fee_rate: 0.05,
       is_available: true,
   }
   ```

2. **Implement API methods** in `BridgeClient`
3. **Add to provider matching** in API calls

### Custom Network Support

Update `network_id_to_chain_name()` function to support additional networks:

```rust
match network_id.0 {
    1 => "ethereum".to_string(),
    369 => "pulsechain".to_string(),
    // Add new networks
    1337 => "custom_network".to_string(),
    _ => format!("chain_{}", network_id.0),
}
```

## Troubleshooting

### Common Issues

1. **"Bridge provider not available"**
   - Check provider `is_available` flag
   - Verify API endpoint connectivity

2. **"Quote request failed"**
   - Check network connectivity
   - Verify API key authentication
   - Validate request parameters

3. **"Transaction execution failed"**
   - Check wallet balance
   - Verify gas limit settings
   - Check network congestion

### Debug Mode

Enable debug logging:

```rust
// In src/bridge/mod.rs
pub async fn get_quote(...) -> Result<BridgeQuote, BridgeError> {
    println!("DEBUG: Requesting quote from {}", provider);
    println!("DEBUG: Request: {:?}", request);
    
    let response = self.client.post(&url).json(&request).send().await?;
    println!("DEBUG: Response status: {}", response.status());
    
    // ... rest of implementation
}
```

## Future Enhancements

1. **Multi-Provider Quote Comparison**
   - Fetch quotes from multiple providers simultaneously
   - Display comparison table
   - Recommend best route

2. **Advanced Bridge Features**
   - Slippage protection
   - MEV protection
   - Gas optimization
   - Batch bridging

3. **Bridge History**
   - Transaction history tracking
   - Status persistence
   - Retry failed transactions

## Contributing

When contributing to bridge functionality:

1. **Follow existing patterns** in `src/bridge/mod.rs`
2. **Add comprehensive error handling**
3. **Include unit tests** for new providers
4. **Update documentation** for new features
5. **Test with mainnet and testnet** networks

## API Reference

For complete API documentation, refer to:
- PulseChain Bridge API: [Documentation Link]
- Multichain API: [Documentation Link]
- Hop Protocol API: [Documentation Link]

---

This integration provides a solid foundation for cross-chain bridging in the Vaughan wallet while maintaining the security and performance standards of the existing codebase.