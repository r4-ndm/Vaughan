# Architecture Improvement Plan: Controller-Based Design

This plan refines the existing "Priority 2 - Advanced Architecture" plan by adopting a strict **Controller-View Separation** (inspired by MetaMask's architecture) and deepening **Alloy Library** integration.

## Goal
Transform the monolithic `WorkingWalletApp` into a set of independent **Controllers** that manage state and logic, leaving the UI as a pure view layer. This ensures "Battle-Tested" stability by allowing core wallet logic to be tested without any GUI (headless).

## User Review Required
> [!IMPORTANT]
> **Architecture Shift**: We are moving from a "Handler" pattern (logic attached to UI events) to a "Controller" pattern (business logic independent of UI).
> **Alloy Core**: Controllers will communicate using strict Alloy types (`Address`, `U256`), forcing input validation to happen at the UI boundary.

## Proposed Changes

### 1. Architecture: The Controller Pattern
Instead of `handlers/*.rs` just consuming UI messages, we will introduce `controllers/*.rs`.

*   **View (UI)**: `src/gui/*.rs` - Handles rendering, user input, formatting strings.
*   **Controllers**: `src/controllers/*.rs` - Manages state, executes business logic, returns strict types.
*   **State**: `src/state/*.rs` - Pure data structures.

### 2. File Restructuring

#### [MODIFY] Source Organization
We will augment the existing plan's "Handlers" phase to be "Controllers".

```
src/
├── controllers/          (NEW: Business Logic Layer)
│   ├── mod.rs
│   ├── transaction.rs    (TransactionController)
│   ├── network.rs        (NetworkController)
│   ├── wallet.rs         (WalletController - Keyring)
│   └── price.rs          (PriceController)
├── gui/
│   ├── handlers/         (modified: acts as bridge between UI Message -> Controller)
│   └── working_wallet.rs (modified: delegation only)
```

### 3. Detailed Phases (Refined)

#### Phase D (Refined): Logic Extraction to Controllers
Instead of just moving code to handlers, we extract it to "Controllers" that don't know about `iced::Message`.

*   **TransactionController**:
    *   Input: `Address`, `U256`, `ChainId` (Alloy types)
    *   Output: `Result<TxHash, Error>`
    *   *Improvement*: Remove all `Message::...` dependency from the core logic. `handlers/transaction.rs` will parse the UI string to `Address`, then call `TransactionController`.

*   **NetworkController**:
    *   Manages providers and chain state.
    *   Uses `alloy::providers::Provider`.

#### Phase E (Refined): Alloy Deepening
*   Replace manual hex parsing with `alloy::primitives` parsing at the input field level.
*   Ensure `TransactionController` accepts `alloy::rpc::types::TransactionRequest`.

#### Phase F (Refined): Secure State Management
*   **State Manager**: Explicitly manage `SensitiveData` wrapped in `Secrecy`.
*   **Zeroizing**: Ensure controllers explicitly zeroize keys after signing operations.

## Verification Plan

### Automated Tests
The primary benefit of this architecture is **Headless Testing**. We can test the wallet logic without spawning a window.

1.  **Controller Tests**:
    ```bash
    cargo test --lib controllers
    ```
    *   *Test*: Create a `TransactionController`, feed it valid/invalid inputs, assert results. No UI mocking needed.

2.  **Integration Tests**:
    *   Simulate a full flow (Unlock -> Create Tx -> Sign -> Broadcast) using only Controllers.

### Manual Verification
1.  **UI Regression**: Open the wallet, perform a send, ensure feedback loops (spinners, success messages) still work via the "Handler -> Controller" bridge.
