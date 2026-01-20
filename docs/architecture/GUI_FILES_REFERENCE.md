# Vaughan Wallet GUI - File Reference for AI Assistants

## 📁 Essential GUI Files to Share

### 🎯 **MOST IMPORTANT FILES** (Share these first)

#### 1. **`src/gui/working_wallet.rs`** (11,218 lines) ⭐⭐⭐
**Purpose**: Main wallet application interface
**Contains**:
- Main wallet view (lines 4205-4825)
- All UI state management
- Message handling
- Form logic (send transactions, account management)
- Button definitions and layouts
- Window settings (lines 11081-11141)

**Why critical**: This is THE main GUI file - contains the entire wallet interface

---

#### 2. **`src/gui/theme.rs`** (776 lines) ⭐⭐⭐
**Purpose**: Warp Terminal-inspired color scheme and styling
**Contains**:
- All Warp color constants (lines 1-90)
  - PRIMARY: `#00D9FF` (electric blue)
  - SUCCESS: `#3FB950` (green)
  - ERROR: `#FF6B9D` (pink-red)
  - Background colors (dark theme)
- Button style implementations
- Container styles
- Custom StyleSheet implementations

**Why critical**: Defines all visual styling and colors used throughout the app

---

#### 3. **`src/gui/warp_helpers.rs`** (441 lines) ⭐⭐
**Purpose**: Warp design system helpers
**Contains**:
- Spacing utilities (xs, sm, md, lg, xl, xxl)
- Typography helpers (display, hero, title, body, label, small)
- Padding constants (XS=4, SM=8, MD=16, LG=24, XL=32, XXL=48)
- Border radius presets (4px, 8px, 12px, 16px)
- Shadow utilities (sm, md, lg, glow effects)
- Utility functions (number formatting, address truncation)

**Why critical**: Provides consistent Warp styling throughout the app

---

### 📚 **SECONDARY FILES** (Share if needed)

#### 4. **`src/gui/mod.rs`** (947 lines)
**Purpose**: GUI module exports and common utilities
**Contains**:
- Module declarations
- Common validation functions
- Bridge/DEX configuration
- Utility functions for formatting

---

#### 5. **`src/gui/tabs/mod.rs`** (6,023 lines)
**Purpose**: Tab-based navigation (DApp Platform tabs)
**Contains**:
- Different wallet screens/tabs
- Token launcher interface
- Bridge interface
- DEX aggregator interface

**Note**: Only needed if working on specific tab features

---

#### 6. **`src/gui/dapp_platform.rs`** (1,945 lines)
**Purpose**: DApp platform integration
**Contains**:
- DApp platform state
- DApp launcher views
- Integration with external DApps

---

#### 7. **`src/gui/widgets.rs`** & **`src/gui/widgets/transaction.rs`**
**Purpose**: Reusable UI components
**Contains**:
- Custom widgets
- Transaction display components
- Reusable UI elements

---

## 🎨 **DOCUMENTATION FILES** (Context)

These provide context about recent changes:

1. **`WARP_DARK_THEME_COLORS.md`** - Complete color palette documentation
2. **`WARP_DESIGN_SYSTEM.md`** - Design system guidelines
3. **`WARP_QUICK_START.md`** - Quick examples for using Warp helpers
4. **`BUTTON_STYLING_UPDATE.md`** - Button styling implementation details
5. **`UI_COMPACTION_UPDATE.md`** - Spacing optimization details
6. **`650PX_HEIGHT_UPDATE.md`** - Latest 650px height optimization

---

## 📋 **Quick Reference Guide**

### For Layout Changes:
**Share**: `working_wallet.rs` (lines 4205-4825) + `warp_helpers.rs`

### For Styling/Colors:
**Share**: `theme.rs` + `WARP_DARK_THEME_COLORS.md`

### For New Components:
**Share**: `widgets.rs` + `warp_helpers.rs` + `theme.rs`

### For Complete Understanding:
**Share**: All files listed above + documentation files

---

## 🎯 **Specific Line Ranges** (for large files)

### `working_wallet.rs` Key Sections:

| Section | Lines | Description |
|---------|-------|-------------|
| Main view | 4205-4825 | The entire wallet UI layout |
| Button styles | 4828-5320 | Warp-themed button implementations |
| Window settings | 11081-11141 | Window size and configuration |
| Message enum | 120-350 | All UI messages/actions |
| State struct | 50-120 | Application state |

### `theme.rs` Key Sections:

| Section | Lines | Description |
|---------|-------|-------------|
| Color constants | 1-90 | All Warp colors |
| Button styles | 182-600 | Button StyleSheet implementations |
| Container styles | 600-770 | Container appearance definitions |

---

## 💡 **Tips for Sharing with Another AI**

### Option 1: Share Individual Files
Upload these files in order of priority:
1. `working_wallet.rs`
2. `theme.rs`
3. `warp_helpers.rs`
4. Documentation markdown files

### Option 2: Share Specific Sections
If file is too large, share specific line ranges:
- Main view: `working_wallet.rs` lines 4205-4825
- Button styles: `working_wallet.rs` lines 4828-5320
- Colors: `theme.rs` lines 1-90

### Option 3: Share Context
Start by sharing this file (`GUI_FILES_REFERENCE.md`) so the AI understands the structure, then share specific files as needed.

---

## 🔧 **Current GUI Architecture**

```
src/gui/
├── working_wallet.rs    ⭐ Main wallet interface (11K lines)
├── theme.rs            ⭐ Warp color scheme & styles (776 lines)
├── warp_helpers.rs     ⭐ Design system utilities (441 lines)
├── mod.rs              📦 Module exports (947 lines)
├── tabs/
│   └── mod.rs          📑 Tab navigation (6K lines)
├── dapp_platform.rs    🔌 DApp integration (1.9K lines)
├── portfolio.rs        💼 Portfolio view (814 lines)
├── widgets.rs          🎨 Reusable components (418 lines)
├── widgets/
│   └── transaction.rs  💸 Transaction widgets (638 lines)
├── window_manager.rs   🪟 Window management (509 lines)
├── spinner.rs          ⏳ Loading spinner (216 lines)
└── tx_utils.rs         🔧 Transaction utilities (50 lines)
```

---

## 📊 **Current State (After Optimizations)**

- **Window size**: 600×650px
- **Theme**: Warp Terminal Dark
- **Primary color**: `#00D9FF` (electric blue)
- **Spacing**: Ultra-compact (4px between sections)
- **Button padding**: Minimal (5-8px vertical)
- **Text sizes**: 11-13px (compact but readable)
- **Layout**: Single-screen, no scrolling required

---

## 🎨 **Design Philosophy**

The GUI follows Warp Terminal's design principles:
- **Dark theme** with high contrast
- **Electric blue** primary color (#00D9FF)
- **Minimal spacing** for compact layout
- **Rounded corners** (8px border radius)
- **Subtle shadows** with glow effects
- **Clean typography** with size hierarchy
- **Professional appearance** inspired by modern terminals

---

## 🚀 **Quick Start for Another AI**

**Prompt suggestion for another AI**:
```
I'm working on the Vaughan cryptocurrency wallet GUI built with Rust and iced. 
The main GUI file is `working_wallet.rs` which contains the wallet interface.
The styling uses a Warp Terminal-inspired dark theme defined in `theme.rs`.
The layout is optimized for 600×650px with ultra-compact spacing.

[Attach: working_wallet.rs, theme.rs, warp_helpers.rs]

Can you help me with [specific task]?
```

---

## 📝 **Notes**

- The GUI uses **iced 0.12** framework
- Built for **Linux** (CachyOS)
- Uses **custom StyleSheet** implementations for theming
- Window size is **locked at 600×650px** minimum
- All buttons use **Warp-themed custom styles**
- Layout is **ultra-compact** (4px spacing throughout)

---

## 🔗 **Related Files** (Outside GUI)

If working on functionality beyond just UI:
- `src/wallet/mod.rs` - Wallet logic
- `src/network/mod.rs` - Network handling
- `src/security/` - Security & keystore
- `Cargo.toml` - Dependencies

But for **pure GUI work**, stick to the files listed above!
