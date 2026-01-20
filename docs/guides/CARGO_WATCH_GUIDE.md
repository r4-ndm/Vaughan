# Cargo Watch - Real-Time GUI Development

## ✅ Installation Complete
cargo-watch has been installed and is ready to use for real-time development!

## 🚀 **Quick Start Commands**

### 1. **Auto-rebuild and run wallet on file changes**
```bash
cd /home/r4/Desktop/Vaughan_V1
cargo watch -x "run --bin vaughan"
```
This will:
- ✅ Watch all `.rs` files for changes
- ✅ Automatically recompile when you save
- ✅ Restart the wallet GUI immediately
- ✅ Show compilation errors in terminal

### 2. **Just check compilation (faster)**
```bash
cargo watch -x check
```
This will:
- ✅ Only check for compilation errors (no running)
- ✅ Much faster feedback
- ✅ Good for rapid iteration on code

### 3. **Watch specific files only**
```bash
cargo watch -w src/gui/ -x "run --bin vaughan"
```
This will:
- ✅ Only watch `src/gui/` directory
- ✅ Ignore changes in other directories
- ✅ Faster for GUI-only development

## 🎯 **Best Development Workflow**

### Terminal Setup (2 terminals)
```bash
# Terminal 1: Auto-rebuild
cd /home/r4/Desktop/Vaughan_V1
cargo watch -x "run --bin vaughan"

# Terminal 2: Manual testing/commands
cd /home/r4/Desktop/Vaughan_V1
# Use for git, documentation, manual runs, etc.
```

### Development Process
1. **Start cargo watch** in Terminal 1
2. **Edit GUI files** (working_wallet.rs, theme.rs, etc.)
3. **Save the file** (Ctrl+S)
4. **Watch Terminal 1** - it will automatically:
   - Rebuild the project
   - Close old wallet window
   - Open new wallet window with changes
5. **Repeat** - instant feedback on every save!

## 📁 **Files to Watch For**

When you edit these files, cargo-watch will auto-rebuild:
- `src/gui/working_wallet.rs` - Main wallet interface
- `src/gui/theme.rs` - Colors and styling
- `src/gui/warp_helpers.rs` - Design system helpers
- Any `.rs` file in `src/` directory

## ⚡ **Performance Tips**

### For Fast Iteration
```bash
# Check only (fastest)
cargo watch -x check

# Build only (no run)
cargo watch -x build

# Run in release mode (slower compile, faster runtime)
cargo watch -x "run --release --bin vaughan"
```

### For GUI-Only Changes
```bash
# Watch only GUI directory
cargo watch -w src/gui/ -x "run --bin vaughan"
```

## 🐛 **Debugging with cargo-watch**

### See Detailed Output
```bash
cargo watch -x "run --bin vaughan" --why
```

### Clear Screen on Rebuild
```bash
cargo watch -c -x "run --bin vaughan"
```

### Watch with Notifications
```bash
cargo watch -x "run --bin vaughan" -N
```

## 🎨 **Current Window Settings (750px)**

With the latest update:
- **Window size**: 600×750px
- **All buttons visible**: Create, Import, Export, Hardware
- **Real-time testing**: Make changes and see them instantly!

## 📊 **Example Development Session**

```bash
# Start watching
cargo watch -x "run --bin vaughan"

# Make changes to working_wallet.rs
# Save file (Ctrl+S)
# Watch terminal automatically rebuild and restart wallet

# Make color changes to theme.rs  
# Save file (Ctrl+S)
# See color changes instantly in GUI

# Continue making changes...
# Each save = instant preview!
```

## ⚠️ **Important Notes**

### File Watching
- cargo-watch monitors all `.rs` files by default
- Save any `.rs` file to trigger rebuild
- Non-Rust files (`.md`, `.txt`) are ignored

### Window Behavior
- Old wallet window closes automatically
- New window opens with fresh changes
- Window position may reset on each rebuild

### Performance
- Initial compile may be slow
- Subsequent rebuilds are incremental (faster)
- Only changed files are recompiled

## 🎉 **Benefits**

✅ **Instant feedback** - See changes immediately
✅ **No manual rebuilds** - Saves time and clicks  
✅ **Error catching** - Compilation errors show instantly
✅ **Efficient workflow** - Stay focused on coding
✅ **GUI prototyping** - Perfect for visual adjustments

## 🔧 **Troubleshooting**

### If cargo-watch stops working:
```bash
# Restart cargo-watch
Ctrl+C  # Stop current watch
cargo watch -x "run --bin vaughan"  # Start again
```

### If builds are too slow:
```bash
# Use check instead of full build
cargo watch -x check
```

### If too many rebuilds:
```bash
# Watch only GUI files
cargo watch -w src/gui/ -x "run --bin vaughan"
```

---

## ✨ **Ready to Use!**

Your development environment is now optimized for real-time GUI development. Start cargo-watch and begin making changes to see instant results! 🚀

**Next steps**:
1. Run `cargo watch -x "run --bin vaughan"` 
2. Open `src/gui/working_wallet.rs` in your editor
3. Make a small change and save
4. Watch the magic happen! ✨