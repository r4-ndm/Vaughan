# GUI Improvements for Vaughan DApp Platform

## Quick Fixes You Can Apply:

### 1. Fix Tab Naming Consistency
In `src/gui/tabs/mod.rs`, update line 65-66:
```rust
// Change from:
TabId::CommandReference => ">_",
TabId::AaveLending => "T13: AAVE",

// To:
TabId::CommandReference => "T12: CMD",
TabId::AaveLending => "T13: AAVE",
```

### 2. Increase Window Width for Better Tab Display
In `src/gui/dapp_platform.rs`, line 2356:
```rust
// Change from:
size: iced::Size::new(1200.0, 800.0),

// To (wider window for 13 tabs):
size: iced::Size::new(1600.0, 900.0),
```

### 3. Consider Using a Tab Overflow Menu
With 13 tabs, you might want to implement:
- A dropdown menu for less-used tabs
- Multi-row tab bar
- Collapsible tab groups

### 4. Optional: Reduce Tab Font Size
In `src/gui/tabs/mod.rs`, line 104:
```rust
// Change from:
.size(11)

// To:
.size(10)  // Slightly smaller to fit more tabs
```

## Current Tab List:
1. T1: Portfolio (PulseChain Dashboard)
2. T2: NFT Manager
3. T3: DEX (Unified Trading)
4. T4: Token Launcher
5. T5: LP Positions
6. T6: Railgun Privacy
7. T7: Transaction History
8. T8: Bridge
9. T9: Perpetual DEX
10. T10: Staking
11. T11: Yield Farming
12. T12: Command Reference (currently shows as ">_")
13. T13: AAVE Lending

## Additional Recommendations:
- Clean up unused imports to reduce compilation warnings
- Consider grouping related tabs (e.g., all DeFi tabs together)
- Add tooltips for tab buttons to show full names on hover
- Implement tab icons instead of text for a cleaner look