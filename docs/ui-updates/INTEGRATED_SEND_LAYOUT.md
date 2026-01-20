# Integrated Send Transaction Layout

## Overview
The main wallet interface has been redesigned to integrate all send transaction functionality directly into the main window, eliminating the need to navigate to a separate dialog.

## Key Changes

### 1. **Unified Main View**
- All send functions are now accessible from the main wallet window
- The send form expands/collapses inline within the main interface
- Users no longer need to navigate away to send transactions

### 2. **New Layout Structure**

#### Top Section (Always Visible)
- **Logo & Settings**: Vaughan branding and settings button
- **Address Display**: Full colored address with copy functionality
- **Network Selection**: Network picker with add custom network button
- **Account Selection**: Account picker with delete account button
- **Balance Display**: Large, prominent balance with spinner when loading

#### Middle Section (Toggleable)
When the "💸 Send" button is clicked, an integrated send form appears inline:

**Send Transaction Form includes:**
- **From Account Selector**: Choose which account to send from
- **To Address Field**: Recipient's address input
- **Token & Amount**: Side-by-side token selector and amount input
- **Gas Settings**: Gas limit and gas price/max fee fields
- **Advanced Options** (Collapsible):
  - Transaction Type (Legacy / EIP-1559)
  - Nonce Override
  - Max Priority Fee (for EIP-1559)
  - Toggle Options: Simulate, Dry Run, Force Broadcast
- **Send Button**: Primary action button to submit the transaction
- **Close Button (✕)**: Collapse the send form

When collapsed, a single "💸 Send" button is shown instead.

#### Bottom Section (Always Visible)
- **Quick Actions**: Refresh, Receive, History buttons
- **DApp Platform**: Launch button for the DApp platform
- **Wallet Management**: Create, Import, Export, Hardware wallet buttons

### 3. **Technical Implementation**

#### Modified Functions
- **`main_wallet_view()`**: Now contains conditional rendering for the send form
  - Checks `self.state.show_send_dialog` to show/hide the send section
  - Renders full send form inline when enabled
  - Shows simple "Send" button when collapsed

#### View Routing
- **`view()` function**: Removed the separate `send_dialog_view()` routing
  - Send form is now part of the main view, not a separate screen
  - Comment added: "Send dialog is now integrated into main view, no longer a separate view"

#### Message Handlers
- **`Message::ShowSend`**: Sets `show_send_dialog = true`, expanding the form inline
- **`Message::HideSend`**: Sets `show_send_dialog = false`, collapsing the form
- All other send-related message handlers remain unchanged

### 4. **User Experience Benefits**

1. **No Context Switching**: Users stay in the main window throughout the send process
2. **Better Visibility**: Balance and network information remain visible while composing transactions
3. **Faster Workflow**: One less navigation step to send tokens
4. **Cleaner Interface**: The collapsed state keeps the UI clean when not sending
5. **Consistent Layout**: Network and account selections are always accessible

### 5. **Feature Parity**

All send features from the previous dialog are preserved:
- ✅ Multi-account support (send from any account)
- ✅ Token selection (native + ERC20 tokens)
- ✅ Custom token addresses
- ✅ Gas customization (limit and price)
- ✅ Advanced options (tx type, nonce, simulate, dry run, force)
- ✅ Transaction confirmation flow
- ✅ Gas estimation
- ✅ Status messages and error handling

### 6. **Visual Design**

The integrated send form uses:
- **Container styling**: Matches the balance container for visual consistency
- **Compact layout**: Reduced spacing to fit comfortably in the main view
- **Emoji icons**: 💸 for Send, 📤 for Send Transaction button
- **Collapsible sections**: Advanced options can be hidden/shown
- **Color coding**: Blue title text for the send section header
- **Button hierarchy**: Primary style for send action, secondary for auxiliary actions

## Testing

Build the application:
```bash
cargo build --release
```

Run the application:
```bash
./target/release/vaughan
```

### Test Cases
1. ✅ Click "💸 Send" - send form expands inline
2. ✅ Fill in transaction details - all fields work correctly
3. ✅ Click "✕" button - send form collapses
4. ✅ Toggle advanced options - collapsible section works
5. ✅ Submit transaction - confirmation and execution work
6. ✅ Network/account remain accessible while send form is open

## Future Enhancements

Potential improvements for future iterations:
- Add a "Quick Send" mode with minimal fields for regular users
- Implement transaction templates for common sends
- Add contact book integration for recipient addresses
- Include recent transaction history in the send section
- Add QR code scanning for recipient addresses

## Migration Notes

- No database changes required
- No state structure changes
- Fully backward compatible with existing wallets
- The old `send_dialog_view()` function can be removed in a future cleanup (currently unused)

## Code Location

Modified file: `src/gui/working_wallet.rs`
- Line ~3724: `main_wallet_view()` function
- Line ~3556: `view()` function routing
- Line ~1372: `Message::ShowSend` handler
- Line ~1411: `Message::HideSend` handler
