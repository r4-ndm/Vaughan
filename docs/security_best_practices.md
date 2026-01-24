# Security Best Practices

Security is the primary design constraint of the Vaughan wallet. This guide outlines the implemented security settings and recommendations for secure operation.

## Key Management

### Storage
-   **Keystore**: We use the MetaMask V3 KeyStore format (PBKDF2-SHA256 + AES-128-CTR).
-   **Memory**: Sensitive data (keys, seeds) is protected in memory using:
    -   `secrecy::Secret<T>` types.
    -   `zeroize` crate to clear memory on drop.
    -   Windows `VirtualLock` to prevent swapping to disk.

### Recommendations
-   **Passwords**: Use strong, unique passwords. The KDF is tuned for high cost (10k+ iterations), but weak passwords are still vulnerable to GPU cracking.
-   **Locking**: The wallet auto-locks after inactivity (configurable). Always manually lock when finished.

## Rate Limiting

We implement strict rate limiting on sensitive operations to prevent brute-force attacks and abuse.

| Operation | Limit | Action on Exceed |
|-----------|-------|------------------|
| **Export Secret** | 3 / hour | Blocked. Requires wait. |
| **Unlock (Password)** | 5 / minute | Exponential backoff. |
| **Sign Transaction** | 20 / minute | Blocked (Soft limit). |

**Configuration**:
Rate limits are persisted in `AppData/Vaughan/security/limits.json` (encrypted). Do not modify this file manually.

## Transaction Safety

### Simulation
Every transaction is simulated against the latest block before signing using `TransactionSimulator`.
-   **Check**: Reverts, gas usage, likely errors.
-   **Warning**: If simulation fails, the UI will present a high-severity warning. **Do not ignore this**.

### EIP-712
Prefer typed data signing (EIP-712) over `personal_sign` (blind signing) whenever possible. It provides human-readable context for what you are signing.

## Telemetry & Privacy

-   **Tracing**: We use `tracing` for logs.
-   **PII**: Sensitive data (keys, passwords, full addresses) is **never** logged.
-   **Correlation**: All logs include a `correlation_id` to trace operations without exposing user identity.
-   **Opt-Out**: Telemetry can be disabled via the `telemetry` feature flag or config settings.
