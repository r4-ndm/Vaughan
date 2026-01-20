# Dependency Cleanup Plan

## Summary
Analysis of Cargo.toml dependencies for cleanup opportunities, version unification, and feature flag optimizations.

## Critical Issues Found

### 1. Major Version Conflicts
#### `base64` - Two Major Versions
```toml
# Direct dependency (line 93)
base64 = "0.21"

# Transitive dependencies using different versions
base64 v0.21.7 ← reqwest v0.11.27
base64 v0.22.1 ← reqwest v0.12.25, alloy-transport, hyper-util
```

**Impact**: Binary size increase, potential compilation issues
**Solution**: Upgrade to 0.22 and pin version

#### `reqwest` - Two Major Versions
```toml
# Direct dependency (line 54)
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }

# But transitive dependencies also bring in reqwest 0.12
reqwest v0.11.27 ← vaughan
reqwest v0.12.25 ← alloy-transport-http, hyper-util
```

**Impact**: Two HTTP client libraries in binary
**Solution**: Upgrade to 0.12 and unify

#### `thiserror` - Two Major Versions
```toml
# Two major versions being pulled in
thiserror v1.0.69 ← multiple dependencies
thiserror v2.0.17 ← alloy libraries
```

**Impact**: Compilation conflicts, larger binary
**Solution**: Pin to version 2.x

#### `serde` - Core/Core Duplication
```toml
# Multiple versions of serde_core
serde_core v1.0.228 ← const-hex, hashbrown, ruint
serde_core v1.0.228 ← serde_json
```

**Impact**: Minor but indicates potential dependency resolution issues

### 2. Feature Flag Issues

#### `alloy` - Over-specified Features
```toml
alloy = { version = "1.1", features = ["full", "provider-http", "provider-ws", "signer-local", "rlp", "consensus", "contract"] }
```

**Problem**: "full" feature already includes most others, causing feature duplication
**Solution**: Remove redundant features or use only specific features

#### `iced` - Potential Feature Over-specification
```toml
iced = { version = "0.12", features = ["tokio", "image", "advanced"] }
```

**Problem**: "advanced" may include unused features
**Solution**: Review actual usage

### 3. Unused Dependencies

#### Potentially Unused Dependencies
Based on `cargo tree` analysis:
- `plotters` and `plotters-iced` - Optional charting, may not be used
- `qrcode` and `image` - Optional features, check if actually used
- `alloy-signer-ledger` and `alloy-signer-trezor` - Hardware wallet support

### 4. Heavy Dependency Chains

#### GUI Rendering Dependencies
```
iced → wgpu → naga → glyphon → cosmic-text → rustybuzz
```
**Impact**: Large dependency chain for GUI
**Optimization**: Consider minimal renderer options

#### Blockchain Dependencies
```
alloy → 50+ sub-crates for full Ethereum support
```
**Impact**: Massive dependency tree
**Optimization**: Use only required alloy features

## Cleanup Recommendations

### Priority 1: Critical Version Conflicts

#### 1.1 Base64 Version Unification
```toml
# Current (line 93)
base64 = "0.21"

# Recommended
base64 = "0.22"
```

**Commands**:
```bash
# Update Cargo.toml
sed -i 's/base64 = "0.21"/base64 = "0.22"/' Cargo.toml

# Update any 0.21-specific code if needed
cargo check
```

#### 1.2 Reqwest Version Unification
```toml
# Current (line 54)
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }

# Recommended
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

**Impact**: May require API updates in code
**Risk**: Medium - reqwest 0.11 → 0.12 has breaking changes

#### 1.3 Thiserror Version Pinning
```toml
# Add to dependencies
thiserror = "2.0"
```

### Priority 2: Feature Flag Optimization

#### 2.1 Alloy Features Cleanup
```toml
# Current (line 14)
alloy = { version = "1.1", features = ["full", "provider-http", "provider-ws", "signer-local", "rlp", "consensus", "contract"] }

# Option A: Minimal approach
alloy = { version = "1.1", features = ["provider-http", "signer-local", "consensus", "contract"] }

# Option B: Keep full but remove redundancy  
alloy = { version = "1.1", features = ["full"] }
```

#### 2.2 Iced Features Review
```toml
# Current (line 22)
iced = { version = "0.12", features = ["tokio", "image", "advanced"] }

# Recommended (if advanced features not used)
iced = { version = "0.12", features = ["tokio", "image"] }
```

### Priority 3: Optional Feature Cleanup

#### 3.1 Review Optional Dependencies
```toml
# Current optional features
[features]
default = ["minimal", "qr", "hardware-wallets"]
qr = ["dep:qrcode", "dep:image"]
charts = ["dep:plotters", "dep:plotters-iced"]
hardware-wallets = ["dep:alloy-signer-ledger", "dep:alloy-signer-trezor"]
```

**Recommended**:
```toml
[features]
default = ["minimal", "hardware-wallets"]  # Remove qr unless actively used
# qr = ["dep:qrcode", "dep:image"]  # Comment out if unused
charts = ["dep:plotters", "dep:plotters-iced"]  # Keep for future use
hardware-wallets = ["dep:alloy-signer-ledger", "dep:alloy-signer-trezor"]
```

### Priority 4: Alternative Dependencies

#### 4.1 Lighter HTTP Client
Consider replacing `reqwest` with lighter alternative for wallet use case:
```toml
# Current: reqwest (heavy but full-featured)
# Alternative: ureq or tiny_http for simple HTTP calls
```

**Risk**: High - requires code changes
**Benefit**: Significant binary size reduction

#### 4.2 Minimal GUI Framework
Consider if `iced` can be replaced with lighter alternative:
```toml
# Current: iced (feature-rich but heavy)
# Alternative: egui, slint, or custom minimalist UI
```

**Risk**: Very High - major rewrite
**Benefit**: Massive dependency reduction

## Implementation Strategy

### Phase 1: Safe Version Updates (1 day)
1. Update base64 to 0.22
2. Pin thiserror to 2.0
3. Test compilation and functionality
4. Update any API changes

### Phase 2: Major Version Updates (2-3 days)
1. Upgrade reqwest to 0.12
2. Update all reqwest usage in code
3. Fix breaking changes
4. Comprehensive testing

### Phase 3: Feature Flag Cleanup (1 day)
1. Optimize alloy features
2. Review iced features
3. Update feature flags
4. Test all feature combinations

### Phase 4: Optional Dependency Review (1 day)
1. Analyze actual usage of optional dependencies
2. Remove unused features
3. Update default features
4. Test reduced feature set

## Expected Impact

### Binary Size Reduction
- **Base64 unification**: ~500KB reduction
- **Reqwest unification**: ~2-3MB reduction  
- **Feature optimization**: ~5-10MB reduction
- **Total potential**: 10-15MB reduction

### Compilation Benefits
- **Faster builds**: Fewer duplicate dependencies to compile
- **Less memory**: Reduced dependency tree
- **Cleaner output**: Fewer version conflict warnings

### Dependency Maintenance
- **Easier updates**: Unified versions
- **Fewer conflicts**: No version mismatches
- **Better security**: Single set of vulnerabilities to track

## Risk Assessment

### Low Risk
- Base64 version update
- Thiserror pinning
- Feature flag cleanup

### Medium Risk
- Reqwest version update (breaking changes)
- Alloy feature changes

### High Risk
- Major dependency replacement
- GUI framework changes

## Validation Steps

1. **Before changes**:
   - `cargo build --release`
   - Benchmark compile time
   - Measure binary size

2. **During changes**:
   - Test after each change
   - Check for compilation errors
   - Run relevant tests

3. **After changes**:
   - Full integration testing
   - Performance benchmarking
   - Binary size measurement

## Success Metrics
- **Zero version conflicts** in `cargo tree --duplicates`
- **Compilation time reduction**: 10-20%
- **Binary size reduction**: 10-15MB
- **No functional regressions**
- **All tests passing**

## Follow-up Maintenance

### Dependency Monitoring
```bash
# Regular checks for new conflicts
cargo tree --duplicates

# Check for outdated dependencies  
cargo outdated

# Security audit
cargo audit
```

### Feature Flag Documentation
Document each feature flag:
- What functionality it enables
- What dependencies it adds
- Who should use it

This systematic cleanup will significantly reduce the dependency bloat while maintaining functionality.