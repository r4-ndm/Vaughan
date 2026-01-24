# dApp Integration Guide

The Vaughan wallet exposes a standard [EIP-1193](https://eips.ethereum.org/EIPS/eip-1193) provider interface, allowing seamless integration with dApps and web interfaces.

## The `VaughanProvider`

The `VaughanProvider` implements the `Eip1193Provider` trait. It handles JSON-RPC requests, manages permissions, and emits events.

### Initialization

```rust
use vaughan::wallet::provider::VaughanProvider;

let provider = VaughanProvider::new(account_manager, chain_id);
```

## Handling Requests

The primary entry point is the `request` method.

```rust
use alloy::rpc::json_rpc::Request;

let request = Request {
    method: "eth_requestAccounts".to_string(),
    params: serde_json::json!([]),
    ..Default::default()
};

let response = provider.request(request).await?;
```

### Supported Methods

-   **Connection**: `eth_requestAccounts`, `eth_accounts`, `eth_chainId`
-   **Signing**: `personal_sign`, `eth_signTypedData_v4`
-   **Transactions**: `eth_sendTransaction`, `eth_estimateGas`
-   **State**: `wallet_switchEthereumChain`, `wallet_addEthereumChain`

## Permission Management

The provider includes a built-in `PermissionManager`.

-   **Origins**: Requests must provide an origin (e.g., "https://app.uniswap.org").
-   **Approval**: `eth_requestAccounts` triggers a permission check. If not approved, it returns error `4001` (User Rejected).
-   **Persistence**: Permissions are persisted across sessions.

## Event Listening

Use the `EventEmitter` to listen for state changes.

```rust
let mut rx = provider.events().subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        match event {
            ProviderEvent::AccountsChanged(accounts) => {
                println!("Active accounts changed: {:?}", accounts);
            }
            ProviderEvent::ChainChanged(chain_id) => {
                println!("Network switched to: {}", chain_id);
            }
            _ => {}
        }
    }
});
```

## Example: Web Integration (WASM)

When compiled to WASM, the `VaughanProvider` can be exposed to JavaScript window object.

```rust
// In your WASM glue code
#[wasm_bindgen]
pub fn inject_provider() {
    let provider = VaughanProvider::new(...);
    // Bind to window.ethereum
}
```
