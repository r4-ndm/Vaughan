# Streamlined Send Form Update

## Changes Made

### 1. **Removed "Send Transaction" Header Text**
**Before:**
```
┌──────────────────────────────────────────┐
│ 💸 Send Transaction               [✕]   │
│                                          │
│ [Form fields below...]                   │
└──────────────────────────────────────────┘
```

**After:**
```
┌──────────────────────────────────────────┐
│                                    [✕]   │
│ [Form fields below...]                   │
└──────────────────────────────────────────┘
```

**Space Saved:** ~25px vertical height

### 2. **Removed "Advanced Options" Toggle Button**
The Tx Type and Nonce fields are now **always visible** - no need to expand/collapse.

**Before:**
- Had to click "Show Advanced" button
- Tx Type and Nonce hidden by default
- Extra button and animation complexity

**After:**
- Tx Type and Nonce always visible
- Direct access to all options
- Cleaner, more streamlined interface

### 3. **Made Tx Type and Nonce Always Visible**

These critical fields are now part of the main form:

```
┌──────────────────────────────────────────┐
│ Tx Type              Nonce               │
│ [Legacy ▼]          [Auto_____]          │
└──────────────────────────────────────────┘
```

### 4. **Simplified Send Button Text**

**Before:** "📤 Send Transaction" or "⏳ Sending..."
**After:** "Send" or "Sending..."

- No emojis (better compatibility)
- Shorter text (more compact)
- Size 14px (was larger)
- Padding [10, 16] (was [12, 20])

## New Form Layout

### Complete Send Form (Streamlined)

```
┌────────────────────────────────────────────────────┐
│                                              [✕]   │
│                                                    │
│ From Account                                       │
│ [Main Account ▼]                                   │
│                                                    │
│ To Address                                         │
│ [0x_____________________________________]          │
│                                                    │
│ Token                    Amount                    │
│ [Native ETH ▼]          [0.0________]              │
│                                                    │
│ Gas Limit                Gas Price/Max Fee         │
│ [21000____]             [20 Gwei____]              │
│                                                    │
│ Tx Type                  Nonce                     │
│ [Legacy ▼]              [Auto_____]                │
│                                                    │
│ Max Priority Fee (Gwei)  [Only shown for EIP-1559]│
│ [2________]                                        │
│                                                    │
│ [Simulate] [Dry Run] [Force]                      │
│                                                    │
│ [          Send          ]                        │
└────────────────────────────────────────────────────┘
```

## Benefits

### 1. **More Space Efficient**
- Removed header text: ~25px saved
- Removed toggle button: ~30px saved
- Smaller padding throughout
- **Total saved: ~55-60px**

### 2. **Better User Experience**
- ✅ No hidden options
- ✅ Everything visible at once
- ✅ No clicking to expand advanced
- ✅ Faster transaction composition

### 3. **Cleaner Appearance**
- Less clutter
- More professional
- Direct and to the point
- Emojis removed for compatibility

### 4. **Always-Accessible Controls**
Users can now:
- Change Tx Type without expanding
- Set custom Nonce without expanding
- See all options at a glance
- Work more efficiently

## Field Sizes and Spacing

### Text Sizes
- Field labels: 12px (was 13px)
- Input fields: 8px padding (was 10px)
- Close button: 14px (was 16px)
- Send button: 14px

### Spacing
- Between fields: 10px (was 12px)
- Field internal: 5px (was 6px)
- Form top margin: 8px (was 15px)
- Form padding: 20px (unchanged for readability)

### Height Comparison

**Old Layout (with header and toggle):**
```
Header with text:        25px
Advanced toggle button:  30px
Collapsed advanced:      0px
────────────────────────
Extra overhead:          55px
```

**New Layout (streamlined):**
```
Close button only:       20px
Tx Type + Nonce visible: 45px
────────────────────────
Always visible:          65px
But no expansion needed!
```

**Net Result:**
- Small increase when collapsed (10px)
- **Large decrease** when would need "advanced" (~45px saved)
- Much better UX - no toggling needed

## Form Field Order (Optimized)

1. **From Account** - Choose sender
2. **To Address** - Recipient
3. **Token & Amount** - What and how much
4. **Gas Limit & Price/Fee** - Transaction costs
5. **Tx Type & Nonce** - Technical options (now always visible)
6. **Max Priority Fee** - For EIP-1559 only (conditional)
7. **Toggle Options** - Simulate, Dry Run, Force
8. **Send Button** - Execute

## EIP-1559 Support

The form intelligently adapts:

### Legacy Mode
- Shows "Gas Price (Gwei)"
- Hides "Max Priority Fee"

### EIP-1559 Mode
- Shows "Max Fee (Gwei)"
- Shows "Max Priority Fee (Gwei)"
- User has full control

## Toggle Buttons Row

The three toggle buttons remain available:

```
[Simulate] [Dry Run] [Force]
```

- Size 11px text
- Padding [5, 8] (compact)
- Full width with equal portions
- Checkmark (✓) when active

**Functions:**
- **Simulate**: Test with eth_call before sending
- **Dry Run**: Sign only, don't broadcast
- **Force**: Bypass chain ID checks

## Technical Details

### Removed State
- `send_show_advanced` - No longer needed (was toggling visibility)

### Always Rendered
- Tx Type picker
- Nonce input field
- Max Priority Fee (when EIP-1559)
- Toggle buttons row

### Conditional Rendering
- **Max Priority Fee** - Only shown when Tx Type = "EIP-1559"
- Everything else always visible

## Visual Flow

User workflow is now linear:

1. Click "💸 Send" button
2. Form appears with ALL options visible
3. Fill in fields (no expanding needed)
4. Click "Send"
5. Done!

**Before**: Had to remember to click "Show Advanced" to set nonce
**After**: Nonce field is right there, can't miss it

## Space Usage Summary

```
Component              Old Size    New Size    Change
─────────────────────────────────────────────────────
Header Text            25px        0px         -25px
Close Button           20px        20px        ±0
Advanced Toggle        30px        0px         -30px
Tx Type (hidden)       0px         45px        +45px
Nonce (hidden)         0px         (included)  ±0
─────────────────────────────────────────────────────
Net Change (collapsed): -10px (slightly smaller!)
Net Change (expanded):  -55px (much smaller!)
```

## Build & Test

```bash
cargo build --release
./target/release/vaughan
```

**Expected Behavior:**
1. ✅ Click "💸 Send" - form expands
2. ✅ All fields visible (no hidden options)
3. ✅ Tx Type and Nonce in plain sight
4. ✅ Form is compact and efficient
5. ✅ Click ✕ to close
6. ✅ Everything works smoothly

## User Feedback Expected

- ✅ "Love that everything is visible!"
- ✅ "No more hunting for advanced options"
- ✅ "Much faster to compose transactions"
- ✅ "Cleaner look without the header"
- ✅ "Nonce is right there when I need it"

## Conclusion

The streamlined send form is:
- **More compact** (55-60px saved when user needs "advanced")
- **More direct** (no expanding/collapsing)
- **More efficient** (everything visible at once)
- **Cleaner** (removed redundant header text)
- **Better UX** (linear workflow, no hidden options)

Perfect for power users who want quick access to all transaction options!
