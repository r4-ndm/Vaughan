# Requirements Document: Password Workflow Enhancement

## Introduction

This specification addresses the password authentication workflow in the Vaughan wallet application. Currently, the wallet has password dialog infrastructure but lacks proper integration for session management on startup and transaction signing. This spec defines a secure, user-friendly password workflow that protects seed-based accounts while maintaining usability.

## Glossary

- **Vaughan Wallet**: The multi-EVM cryptocurrency wallet application
- **Session**: An authenticated period where the user has unlocked their wallet with their password
- **Seed-Based Account**: An account created from or imported via a BIP-39 seed phrase, requiring password authentication
- **Private-Key Account**: An account imported directly via private key (currently bypasses password protection)
- **Password Dialog**: The UI component that prompts users to enter their password
- **Session Lock**: The state where the wallet requires password re-authentication
- **Auto-Lock**: Automatic session locking after a period of inactivity
- **Key Cache**: Temporary in-memory storage of decrypted keys during an active session

## Requirements

### Requirement 1: Startup Authentication

**User Story:** As a wallet user, I want to be prompted for my password when I launch Vaughan, so that my seed-based accounts are protected from unauthorized access.

#### Acceptance Criteria

1. WHEN the Vaughan application launches AND at least one seed-based account exists, THEN the Vaughan Wallet SHALL display the password dialog with reason "UnlockSession"
2. WHEN the user enters the correct password at startup, THEN the Vaughan Wallet SHALL unlock the session and cache the decrypted seed for the session duration
3. WHEN the user enters an incorrect password at startup, THEN the Vaughan Wallet SHALL display an error message and allow retry with attempt tracking
4. WHEN the user cancels the password dialog at startup, THEN the Vaughan Wallet SHALL remain in locked state and display a locked wallet view
5. WHEN no seed-based accounts exist (only private-key accounts), THEN the Vaughan Wallet SHALL skip the password prompt and start in unlocked state

### Requirement 2: Transaction Signing Authentication

**User Story:** As a wallet user, I want to confirm transactions with my password, so that no unauthorized transactions can be sent from my seed-based accounts.

#### Acceptance Criteria

1. WHEN the user initiates a transaction from a seed-based account AND the session is unlocked, THEN the Vaughan Wallet SHALL display the transaction confirmation dialog without requiring password re-entry
2. WHEN the user initiates a transaction from a seed-based account AND the session is locked, THEN the Vaughan Wallet SHALL display the password dialog with reason "SignTransaction" before showing transaction confirmation
3. WHEN the user confirms a transaction AND provides correct password, THEN the Vaughan Wallet SHALL sign and broadcast the transaction using the decrypted seed
4. WHEN the user confirms a transaction AND provides incorrect password, THEN the Vaughan Wallet SHALL display an error and prevent transaction submission
5. WHEN the user initiates a transaction from a private-key account, THEN the Vaughan Wallet SHALL proceed with transaction confirmation without password prompt

### Requirement 3: Session Management

**User Story:** As a wallet user, I want my session to remain unlocked for a reasonable time period, so that I don't have to re-enter my password for every action.

#### Acceptance Criteria

1. WHEN the user successfully authenticates, THEN the Vaughan Wallet SHALL maintain an unlocked session for 15 minutes of inactivity by default
2. WHEN the user performs any wallet action during an active session, THEN the Vaughan Wallet SHALL extend the session timeout by resetting the inactivity timer
3. WHEN the session timeout period elapses, THEN the Vaughan Wallet SHALL automatically lock the session and clear cached keys from memory
4. WHEN the session is locked due to timeout, THEN the Vaughan Wallet SHALL display a notification indicating session expiration
5. WHEN the user manually locks the session, THEN the Vaughan Wallet SHALL immediately clear cached keys and require password re-authentication

### Requirement 4: Remember Session Option

**User Story:** As a wallet user, I want the option to keep my session unlocked for a longer period, so that I can work efficiently without frequent re-authentication.

#### Acceptance Criteria

1. WHEN the password dialog is displayed, THEN the Vaughan Wallet SHALL show a "Remember for 15 minutes" checkbox option
2. WHEN the user checks "Remember for 15 minutes" AND enters correct password, THEN the Vaughan Wallet SHALL cache the decrypted seed in memory for 15 minutes
3. WHEN the "Remember for 15 minutes" option is active, THEN the Vaughan Wallet SHALL allow transaction signing without password re-entry during the cache period
4. WHEN the 15-minute cache period expires, THEN the Vaughan Wallet SHALL clear the cached seed and require password re-authentication for the next transaction
5. WHEN the user unchecks "Remember for 15 minutes", THEN the Vaughan Wallet SHALL require password entry for each sensitive operation

### Requirement 5: Account Type Differentiation

**User Story:** As a wallet developer, I want to differentiate between seed-based and private-key accounts, so that password requirements are applied appropriately.

#### Acceptance Criteria

1. WHEN loading accounts at startup, THEN the Vaughan Wallet SHALL identify which accounts are seed-based and which are private-key based
2. WHEN a seed-based account is selected, THEN the Vaughan Wallet SHALL enforce password authentication for transaction signing
3. WHEN a private-key account is selected, THEN the Vaughan Wallet SHALL allow transaction signing without password authentication
4. WHEN displaying account information, THEN the Vaughan Wallet SHALL indicate the account type (seed-based or private-key) to the user
5. WHEN switching between account types, THEN the Vaughan Wallet SHALL adjust authentication requirements accordingly

### Requirement 6: Password Validation and Error Handling

**User Story:** As a wallet user, I want clear feedback when I enter an incorrect password, so that I understand what went wrong and can retry.

#### Acceptance Criteria

1. WHEN the user enters an empty password, THEN the Vaughan Wallet SHALL display error "Password cannot be empty" without attempting validation
2. WHEN the user enters an incorrect password, THEN the Vaughan Wallet SHALL display error "Incorrect password (X attempts remaining)" with remaining attempt count
3. WHEN the user exceeds maximum password attempts, THEN the Vaughan Wallet SHALL temporarily lock the account and display "Too many failed attempts - please wait X seconds"
4. WHEN password validation fails due to corrupted data, THEN the Vaughan Wallet SHALL display error "Failed to decrypt - password may be incorrect or data corrupted"
5. WHEN the user successfully enters the correct password after previous errors, THEN the Vaughan Wallet SHALL clear all error messages and proceed with authentication

### Requirement 7: Secure Key Caching

**User Story:** As a security-conscious user, I want my decrypted keys to be stored securely in memory and cleared when no longer needed, so that my funds remain protected.

#### Acceptance Criteria

1. WHEN the session is unlocked with "Remember for 15 minutes" enabled, THEN the Vaughan Wallet SHALL store the decrypted seed in a SecretString in memory only
2. WHEN the session is locked (manually or via timeout), THEN the Vaughan Wallet SHALL immediately clear all cached keys from memory using secure erasure
3. WHEN the application is closed, THEN the Vaughan Wallet SHALL clear all cached keys before termination
4. WHEN a transaction is signed using cached keys, THEN the Vaughan Wallet SHALL access the cached seed without writing it to disk or logs
5. WHEN the cache period expires, THEN the Vaughan Wallet SHALL securely erase the cached seed and set session state to locked

### Requirement 8: UI State Consistency

**User Story:** As a wallet user, I want the UI to clearly indicate whether my session is locked or unlocked, so that I understand the current security state.

#### Acceptance Criteria

1. WHEN the session is locked, THEN the Vaughan Wallet SHALL display a lock icon or indicator in the UI
2. WHEN the session is unlocked, THEN the Vaughan Wallet SHALL display an unlock icon or indicator with time remaining until auto-lock
3. WHEN the password dialog is visible, THEN the Vaughan Wallet SHALL prevent interaction with other wallet functions via modal overlay
4. WHEN the session state changes, THEN the Vaughan Wallet SHALL update all UI components to reflect the new state immediately
5. WHEN the user hovers over the session indicator, THEN the Vaughan Wallet SHALL display a tooltip with session status details

### Requirement 9: Startup Flow Integration

**User Story:** As a wallet user, I want a smooth startup experience that handles authentication before loading my account data, so that I don't see unauthorized information.

#### Acceptance Criteria

1. WHEN the application starts, THEN the Vaughan Wallet SHALL check for seed-based accounts before displaying the main UI
2. WHEN seed-based accounts exist AND session is locked, THEN the Vaughan Wallet SHALL display the password dialog before loading account balances
3. WHEN the user successfully authenticates at startup, THEN the Vaughan Wallet SHALL proceed to load accounts, networks, and balances
4. WHEN the user fails to authenticate at startup, THEN the Vaughan Wallet SHALL display a locked state view with option to retry authentication
5. WHEN the application is already running AND session locks, THEN the Vaughan Wallet SHALL blur or hide sensitive information until re-authentication

### Requirement 10: Transaction Confirmation Flow

**User Story:** As a wallet user, I want a clear flow from transaction creation to confirmation, with password prompts appearing at the right time.

#### Acceptance Criteria

1. WHEN the user clicks "Send" on a transaction, THEN the Vaughan Wallet SHALL first check if session is unlocked for seed-based accounts
2. WHEN the session is locked during transaction creation, THEN the Vaughan Wallet SHALL prompt for password before showing gas estimation
3. WHEN the user enters password during transaction flow, THEN the Vaughan Wallet SHALL validate it and proceed to transaction confirmation dialog
4. WHEN password validation succeeds during transaction flow, THEN the Vaughan Wallet SHALL automatically continue to the confirmation step without additional user action
5. WHEN the user cancels password entry during transaction flow, THEN the Vaughan Wallet SHALL cancel the entire transaction and return to the send dialog
