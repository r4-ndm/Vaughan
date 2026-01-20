# Vaughan Wallet Password Workflow User Guide

## Overview

The Vaughan wallet implements a secure password workflow to protect your seed-based accounts from unauthorized access. This guide explains how the password system works and how to use it effectively.

## Password Protection Types

### Seed-Based Accounts
- **Protection Level**: Require password authentication
- **When Protected**: On wallet startup and transaction signing
- **How Created**: Generated from or imported via BIP-39 seed phrases
- **Security**: Highest - seeds are encrypted and require password to decrypt

### Private-Key Accounts
- **Protection Level**: No password required
- **When Protected**: Never (direct private key access)
- **How Created**: Imported directly via private key
- **Security**: Medium - relies on device security only

## Password Workflow

### 1. Startup Authentication

**When you launch Vaughan:**

✅ **If you have seed-based accounts:**
- Password dialog appears with "Enter password to unlock wallet"
- Enter your master password to decrypt seed phrases
- Wallet loads your accounts and balances
- Session remains unlocked for 15 minutes (configurable)

✅ **If you only have private-key accounts:**
- No password prompt appears
- Wallet starts immediately
- All functionality available instantly

### 2. Transaction Signing

**When sending transactions:**

✅ **With unlocked session (seed accounts):**
- Transaction proceeds directly to confirmation
- No additional password required
- Fast and seamless experience

✅ **With locked session (seed accounts):**
- Password dialog appears with transaction details
- Enter password to authorize the specific transaction
- Transaction proceeds to confirmation after authentication

✅ **With private-key accounts:**
- No password required at any point
- Direct access to transaction confirmation

### 3. Session Management

**Automatic Session Locking:**
- Sessions auto-lock after **15 minutes of inactivity**
- Timer resets with any wallet interaction
- Locked sessions require password re-entry for transactions

**Manual Session Locking:**
- Click the lock icon in the top bar
- Immediately clears cached keys from memory
- Requires password for next transaction

**"Remember for 15 minutes" Option:**
- ✅ **Checked**: Caches decrypted seed for 15 minutes
- ✅ **Unchecked**: Requires password for each transaction
- Recommended: Keep checked for normal use

## Security Features

### Password Attempt Protection
- **Failed attempts**: Shows remaining tries (5 attempts maximum)
- **Too many failures**: Temporary account lockout (5 minutes)
- **Attempt reset**: Successful password clears attempt counter

### Secure Key Handling
- **Memory only**: Decrypted seeds never written to disk
- **Automatic clearing**: Keys erased on lock/timeout/shutdown
- **No logging**: Passwords and seeds never appear in logs
- **Secure transport**: Uses SecretString for password handling

### Session Security
- **Timeout enforcement**: Absolute 15-minute limit regardless of activity
- **Clean shutdown**: Keys cleared before application exit
- **Lock on minimize**: Optional security feature
- **Visual indicators**: Lock/unlock status always visible

## Common Scenarios

### Daily Wallet Use
1. **Launch wallet** → Enter password once
2. **Use normally** for 15 minutes without re-authentication
3. **Session expires** → Enter password for next transaction
4. **Continue using** with another 15-minute session

### High Security Mode
1. **Uncheck "Remember for 15 minutes"**
2. **Enter password for every transaction**
3. **Maximum security** at cost of convenience

### Mixed Account Types
1. **Seed accounts** require password when session locked
2. **Private-key accounts** never require password
3. **Switch freely** between account types
4. **Security adjusts automatically** based on selected account

## Error Messages & Troubleshooting

### "Incorrect password (X attempts remaining)"
- **Cause**: Wrong password entered
- **Solution**: Double-check password, watch caps lock
- **Prevention**: Use secure password manager

### "Too many failed attempts - please wait X seconds"
- **Cause**: Exceeded maximum password attempts (5)
- **Solution**: Wait for lockout timer to expire
- **Prevention**: Ensure you know correct password

### "Session expired - please unlock wallet"
- **Cause**: 15-minute inactivity timeout reached
- **Solution**: Enter password to unlock new session
- **Prevention**: Use wallet more frequently or enable longer timeout

### "Failed to decrypt - password may be incorrect or data corrupted"
- **Cause**: Keystore corruption or wrong password
- **Solution**: Try backup recovery or contact support
- **Prevention**: Regular backups of wallet data

## Best Practices

### Password Security
- ✅ Use a strong, unique password
- ✅ Store password in secure password manager
- ✅ Never share password with others
- ❌ Don't use simple or reused passwords

### Session Management
- ✅ Keep "Remember for 15 minutes" checked for normal use
- ✅ Manually lock when leaving device unattended
- ✅ Monitor session indicator for remaining time
- ❌ Don't rely on auto-lock for immediate security needs

### Account Management
- ✅ Use seed-based accounts for maximum security
- ✅ Keep seed phrases backed up securely
- ✅ Consider private-key accounts only for temporary use
- ❌ Don't mix account types unless necessary

### Emergency Procedures
- ✅ Keep secure backups of seed phrases
- ✅ Test password regularly to avoid lockouts
- ✅ Know your recovery procedure
- ❌ Don't panic if locked out - wait for timeout

## Advanced Features

### Session Indicators
- **🔓 Green Unlock Icon**: Session active, time remaining shown
- **🔒 Red Lock Icon**: Session locked, password required
- **⏱️ Timer Display**: Shows minutes until auto-lock
- **Manual Lock Button**: Immediate session termination

### Account Type Detection
- **Visual indicators**: Account type shown in account selector
- **Automatic adjustment**: Security requirements adjust per account
- **Seamless switching**: No disruption when changing accounts
- **Smart authentication**: Only prompts when actually needed

### Password Dialog Features
- **Context awareness**: Shows reason for password request
- **Transaction details**: Displays what you're authorizing
- **Remember option**: Session caching preference
- **Clear error messages**: Specific guidance for resolution

This password workflow ensures your funds remain secure while providing a smooth user experience for daily wallet operations.