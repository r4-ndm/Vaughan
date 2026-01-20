# Hard Wallet Button Changes

## Changes Made

### 1. Button Text Updated
- Changed button text from **"Hardware Wallet"** to **"Hard Wallet"**
- Location: Main wallet view, in the wallet management buttons row

### 2. Button Sizing Fixed
- The "Hard Wallet" button now uses the same sizing as the other three buttons in that row:
  - Same padding: `[10, 16]`
  - Same style: `secondary_button_style()`
  - Same width: `Length::FillPortion(1)`
- All four buttons (Create Wallet, Import Wallet, Export Wallet, Hard Wallet) now have consistent appearance

### 3. Dialog Title Updated
- Changed dialog title from "Hardware Wallet - Not implemented" to "Hard Wallet - Not implemented"

### 4. All Log Messages Updated
The following log messages were updated for consistency:
- "No hardware wallets detected" → "No hard wallets detected"
- "Hardware wallets detected" → "Hard wallets detected"
- "Found X hardware wallet(s)" → "Found X hard wallet(s)"
- "Failed to detect hardware wallets" → "Failed to detect hard wallets"
- "Connecting to hardware wallet" → "Connecting to hard wallet"
- "Hardware wallet connection failed" → "Hard wallet connection failed"
- "Hardware wallet connected" → "Hard wallet connected"

### 5. Error Messages Updated
- "Hardware wallet detection not implemented" → "Hard wallet detection not implemented"
- "Hardware wallet connection not implemented" → "Hard wallet connection not implemented"

### 6. Comments Updated
- "Export type buttons with hardware wallet awareness" → "Export type buttons with hard wallet awareness"
- "Automatically detect hardware wallets when dialog opens" → "Automatically detect hard wallets when dialog opens"

### 7. Export Dialog Updated
- "Seed Phrase (Not Available for Hardware Wallets)" → "Seed Phrase (Not Available for Hard Wallets)"

## Result
The wallet now consistently uses "Hard Wallet" terminology throughout the interface while maintaining the same functionality and visual consistency with the other buttons in the wallet management row.

## Files Modified
- `src/gui/working_wallet.rs` - All references updated from "Hardware Wallet" to "Hard Wallet"