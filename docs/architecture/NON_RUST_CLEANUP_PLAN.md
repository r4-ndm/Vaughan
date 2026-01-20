# Vaughan Wallet - Non-Rust Code Cleanup & Organization Plan
## Professional Standards Edition

## Executive Summary

**Current State:**
- 43 Python/Shell scripts in root directory (4,706 lines)
- 101 Markdown documentation files in root
- Solidity contracts scattered across 3 locations
- No clear organization or lifecycle management
- GitHub language stats showing 30% Solidity, obscuring Rust focus

**Target State:**
- Organized directory structure following industry standards
- Clear separation of active vs archived tooling
- Proper documentation hierarchy
- Accurate GitHub language statistics
- Professional project presentation

---

## Current Inventory

### Python Scripts (15 files, ~2,000 lines)
**Security & Analysis:**
- `analyze_security.py`
- `fix_unwraps.py`
- `fix_private_key_exposure.py`

**Debugging:**
- `debug_pulsechain.py`
- `debug_transaction.py`
- `network_debug.py`
- `diagnose_insufficient_funds.py`

**Code Refactoring:**
- `fix_colors.py`
- `fix_result_imports.py`
- `fix_network_connection.py`

**Testing:**
- `check_real_balances.py`
- `simple_network_test.py`
- `test_network_selection.py`
- `find_accounts.py`

### Shell Scripts (28 files, ~2,500 lines)
**Launch Scripts:**
- `launch-vaughan.sh` (duplicate: `launch_vaughan.sh`)
- `launch_dapp_fresh.sh`
- `restart_app.sh`

**Build Scripts:**
- `dev-rebuild.sh`
- `force_rebuild_and_launch.sh`
- `watch-dapp.sh`

**Debugging:**
- `debug_deployment.sh`
- `diagnose_gui.sh`
- `fix_rpc.sh`
- `fix_code_quality.sh`
- `fix_gui_artifacts.sh`

**Testing (20+ files):**
- `test_13_tabs.sh`
- `test_auto_verification.sh`
- `test_balance_fix.sh`
- `test_dapp_network_sync.sh`
- `test_ethw.sh`
- `test_final_fix.sh`
- `test_network_connectivity.sh`
- `test_pulsechain_api.sh`
- `test_testnet_balance.sh`
- `test_token_launcher.sh`
- `test_tx_history.sh`
- `verify_pulsechain_config.sh`
- `verify_terminal_removed.sh`
- `simple_test_deploy.sh`

**Setup:**
- `setup_api_keys.sh`
- `get_verification_info.sh`

### Solidity Contracts (Multiple Locations)
**Root Directory:**
- `SimpleToken_flattened.sol`

**src/ Directory (Wrong Location):**
- `src/Counter.sol`
- `src/CustomToken.sol`
- `src/MinimalERC20.sol`
- `src/SimpleToken.sol`

**contracts/ Directory (Correct Location):**
- `contracts/BasicERC20Token.sol`
- `contracts/BurnableERC20Token.sol`
- `contracts/src/BasicToken.sol`
- `contracts/src/Counter.sol`
- `contracts/src/VaughanToken.sol`
- `contracts/script/Counter.s.sol`
- `contracts/test/Counter.t.sol`

**lib/ Directory:**
- `lib/forge-std/` (23K+ lines - Foundry standard library)

### Markdown Documentation (101 files)
All in root directory - no organization

### Build Configuration
- `Makefile` (root)
- `foundry.toml` (root)
- `Cargo.toml` (root)

---

## Professional Directory Structure

### Target Organization
```
vaughan/
├── .github/                    # GitHub specific files
│   ├── workflows/             # CI/CD pipelines
│   └── ISSUE_TEMPLATE/        # Issue templates
│
├── contracts/                  # Smart contracts (Foundry project)
│   ├── src/                   # Contract source files
│   │   ├── tokens/           # Token contracts
│   │   │   ├── VaughanToken.sol
│   │   │   ├── BasicToken.sol
│   │   │   └── BurnableToken.sol
│   │   └── test/             # Test contracts
│   │       └── MockERC20.sol
│   ├── script/               # Deployment scripts
│   │   └── Deploy.s.sol
│   ├── test/                 # Contract tests
│   │   └── Token.t.sol
│   └── lib/                  # Contract dependencies (git submodules)
│       └── forge-std/
│
├── docs/                      # Documentation
│   ├── README.md             # Main documentation index
│   ├── architecture/         # Architecture docs
│   │   ├── overview.md
│   │   ├── security.md
│   │   └── modules.md
│   ├── guides/               # User guides
│   │   ├── quickstart.md
│   │   ├── custom-networks.md
│   │   ├── hardware-wallets.md
│   │   └── deployment.md
│   ├── development/          # Developer docs
│   │   ├── building.md
│   │   ├── testing.md
│   │   ├── contributing.md
│   │   └── debugging.md
│   ├── features/             # Feature documentation
│   │   ├── dex-integration.md
│   │   ├── staking.md
│   │   └── bridge.md
│   └── changelog/            # Version history
│       ├── CHANGELOG.md
│       └── migration-guides/
│
├── scripts/                   # Development scripts
│   ├── dev/                  # Development utilities
│   │   ├── launch.sh        # Main launcher
│   │   ├── rebuild.sh       # Development rebuild
│   │   └── watch.sh         # File watcher
│   ├── build/               # Build automation
│   │   ├── build.sh
│   │   └── release.sh
│   ├── test/                # Testing scripts
│   │   ├── integration/
│   │   │   ├── test-networks.sh
│   │   │   ├── test-balances.sh
│   │   │   └── test-transactions.sh
│   │   └── e2e/
│   │       └── test-wallet-flow.sh
│   ├── setup/               # Setup and configuration
│   │   ├── setup-env.sh
│   │   └── setup-api-keys.sh
│   ├── tools/               # Development tools
│   │   ├── analyze-security.py
│   │   ├── check-balances.py
│   │   └── network-debug.py
│   └── archive/             # Archived/deprecated scripts
│       ├── README.md        # Why these are archived
│       └── old-fixes/
│
├── src/                      # Rust source code (NO .sol files)
│   ├── bin/
│   ├── blockchain/
│   ├── config/
│   ├── error/
│   ├── foundry/
│   ├── gui/
│   ├── network/
│   ├── security/
│   ├── tokens/
│   ├── wallet/
│   ├── lib.rs
│   └── main.rs
│
├── tests/                    # Rust integration tests
│   ├── integration/
│   └── e2e/
│
├── benches/                  # Rust benchmarks
│
├── assets/                   # Static assets
│   ├── icons/
│   ├── images/
│   └── fonts/
│
├── config/                   # Configuration files
│   └── api_config.toml
│
├── .cargo/                   # Cargo configuration
│   └── config.toml
│
├── Cargo.toml               # Rust dependencies
├── Cargo.lock
├── foundry.toml             # Foundry configuration
├── Makefile                 # Build automation
├── .gitignore
├── .gitattributes           # Git attributes (language stats)
├── .env.example             # Environment template
├── README.md                # Project README
├── LICENSE
└── SECURITY.md              # Security policy

# NOT in repository:
├── target/                  # Build artifacts (gitignored)
├── lib/forge-std/          # Git submodule (not counted in stats)
└── node_modules/           # If any JS tooling (gitignored)
```

---

## Implementation Plan

### Phase 1: Create Structure (30 minutes)
**Goal:** Set up new directory structure

#### 1.1 Create Directory Tree
```bash
# Documentation
mkdir -p docs/{architecture,guides,development,features,changelog}

# Scripts organization
mkdir -p scripts/{dev,build,test/{integration,e2e},setup,tools,archive}

# Contracts cleanup
mkdir -p contracts/src/tokens

# GitHub workflows
mkdir -p .github/workflows
```

#### 1.2 Create Index Files
Create README.md in each major directory explaining its purpose.

---

### Phase 2: Organize Scripts (2 hours)

#### 2.1 Active Development Scripts
**Move to `scripts/dev/`:**
```bash
mv launch-vaughan.sh scripts/dev/launch.sh
mv dev-rebuild.sh scripts/dev/rebuild.sh
mv watch-dapp.sh scripts/dev/watch.sh
mv restart_app.sh scripts/dev/restart.sh
```

**Update script paths and make executable:**
```bash
chmod +x scripts/dev/*.sh
```

#### 2.2 Build Scripts
**Move to `scripts/build/`:**
```bash
mv force_rebuild_and_launch.sh scripts/build/clean-rebuild.sh
mv scripts/build.sh scripts/build/build.sh  # Already in scripts/
```

#### 2.3 Testing Scripts
**Move to `scripts/test/integration/`:**
```bash
mv test_network_connectivity.sh scripts/test/integration/
mv test_balance_fix.sh scripts/test/integration/
mv test_pulsechain_api.sh scripts/test/integration/
mv test_testnet_balance.sh scripts/test/integration/
mv test_tx_history.sh scripts/test/integration/
mv test_dapp_network_sync.sh scripts/test/integration/
```

**Move to `scripts/test/e2e/`:**
```bash
mv test_13_tabs.sh scripts/test/e2e/
mv test_token_launcher.sh scripts/test/e2e/
```

#### 2.4 Setup Scripts
**Move to `scripts/setup/`:**
```bash
mv setup_api_keys.sh scripts/setup/
mv get_verification_info.sh scripts/setup/
```

#### 2.5 Development Tools
**Move to `scripts/tools/`:**
```bash
mv analyze_security.py scripts/tools/
mv check_real_balances.py scripts/tools/
mv network_debug.py scripts/tools/
mv find_accounts.py scripts/tools/
```

#### 2.6 Archive Old Scripts
**Move to `scripts/archive/`:**
```bash
# One-off fixes (already applied)
mv fix_*.py scripts/archive/
mv fix_*.sh scripts/archive/

# Old debugging scripts
mv debug_*.py scripts/archive/
mv debug_*.sh scripts/archive/
mv diagnose_*.py scripts/archive/
mv diagnose_*.sh scripts/archive/

# Verification scripts (if no longer needed)
mv verify_*.sh scripts/archive/

# Old test scripts
mv test_ethw.sh scripts/archive/
mv test_final_fix.sh scripts/archive/
mv test_auto_verification.sh scripts/archive/
mv simple_test_deploy.sh scripts/archive/
mv simple_network_test.py scripts/archive/
```

#### 2.7 Remove Duplicates
```bash
# Remove duplicate launch script
rm launch_vaughan.sh  # Keep launch-vaughan.sh (moved to scripts/dev/)
```

#### 2.8 Create Archive README
```bash
cat > scripts/archive/README.md << 'EOF'
# Archived Scripts

This directory contains scripts that are no longer actively used but kept for reference.

## Categories

### One-off Fixes
Scripts that fixed specific issues and are no longer needed:
- `fix_*.py` - Code refactoring scripts (already applied)
- `fix_*.sh` - Build/config fixes (already applied)

### Debugging Scripts
Temporary debugging scripts for specific issues:
- `debug_*.py` - Network/transaction debugging
- `diagnose_*.py` - Issue diagnosis

### Deprecated Tests
Old test scripts replaced by better alternatives:
- `test_ethw.sh` - ETHW network testing
- `test_final_fix.sh` - Final fix verification

## Cleanup Policy
Scripts in this directory can be deleted after 6 months if not referenced.
EOF
```

---

### Phase 3: Organize Solidity Contracts (1 hour)

#### 3.1 Consolidate Token Contracts
```bash
# Move token contracts to proper location
mv contracts/BasicERC20Token.sol contracts/src/tokens/BasicERC20.sol
mv contracts/BurnableERC20Token.sol contracts/src/tokens/BurnableERC20.sol

# Move from src/ to contracts/src/tokens/
mv src/CustomToken.sol contracts/src/tokens/CustomToken.sol
mv src/MinimalERC20.sol contracts/src/tokens/MinimalERC20.sol
mv src/SimpleToken.sol contracts/src/tokens/SimpleToken.sol

# Move VaughanToken
mv contracts/src/VaughanToken.sol contracts/src/tokens/VaughanToken.sol
mv contracts/src/BasicToken.sol contracts/src/tokens/BasicToken.sol
```

#### 3.2 Organize Test Contracts
```bash
# Keep test contracts in contracts/test/
# Counter.t.sol is already there
```

#### 3.3 Clean Up Root Directory
```bash
# Remove flattened contract from root
rm SimpleToken_flattened.sol

# Remove example contracts
rm src/Counter.sol  # Example contract, not needed
rm contracts/src/Counter.sol  # Duplicate
```

#### 3.4 Update Foundry Configuration
Update `foundry.toml`:
```toml
[profile.default]
src = "contracts/src"
out = "out"
libs = ["lib"]
test = "contracts/test"
script = "contracts/script"
```

---

### Phase 4: Organize Documentation (2 hours)

#### 4.1 Categorize Documentation Files
**Architecture Documentation → `docs/architecture/`:**
```bash
mv DAPP_PLATFORM_ARCHITECTURE.md docs/architecture/dapp-platform.md
mv COMPREHENSIVE_SECURITY_AUDIT.md docs/architecture/security-audit.md
mv HARDWARE_WALLET_INTEGRATION.md docs/architecture/hardware-wallets.md
mv WALLET_EXTENSION_CONNECTOR_PLAN.md docs/architecture/extension-connector.md
```

**User Guides → `docs/guides/`:**
```bash
mv QUICKSTART_UI.md docs/guides/quickstart.md
mv CUSTOM_NETWORKS_GUIDE.md docs/guides/custom-networks.md
mv ETHW_NETWORK_GUIDE.md docs/guides/ethw-network.md
mv ETHW_NODE_GUIDE.md docs/guides/ethw-node-setup.md
mv pulsechain_testnet_setup.md docs/guides/pulsechain-testnet.md
mv CARGO_WATCH_GUIDE.md docs/guides/development-workflow.md
```

**Development Documentation → `docs/development/`:**
```bash
mv DEPLOY_WITH_FORGE.md docs/development/forge-deployment.md
mv FOUNDRY_AUTO_VERIFICATION.md docs/development/auto-verification.md
mv REAL_DEPLOYMENT_SETUP.md docs/development/deployment-setup.md
mv SIMPLE_DEPLOY_GUIDE.md docs/development/simple-deploy.md
mv HOW_TO_LAUNCH_13_TABS.md docs/development/testing-ui.md
mv DEBLOAT_PLAN.md docs/development/debloat-plan.md
```

**Feature Documentation → `docs/features/`:**
```bash
mv DEX_IMPLEMENTATION_GUIDE.md docs/features/dex-integration.md
mv ENHANCED_DEX_IMPLEMENTATION_COMPLETE.md docs/features/dex-complete.md
mv META_DEX_AGGREGATOR.md docs/features/dex-aggregator.md
mv STAKING_FARMING_FEATURES.md docs/features/staking-farming.md
mv BRIDGE_INTEGRATION.md docs/features/bridge.md
mv DAPP_DEVELOPMENT.md docs/features/dapp-platform.md
```

**Changelog → `docs/changelog/`:**
```bash
mv REFACTORING_COMPLETE.md docs/changelog/refactoring-complete.md
mv PHASE_2_COMPLETE.md docs/changelog/phase-2-complete.md
mv PRIORITY_1_COMPLETE.md docs/changelog/priority-1-complete.md
mv IMPLEMENTATION_SUMMARY.md docs/changelog/implementation-summary.md
mv CLEANUP_SUMMARY.md docs/changelog/cleanup-summary.md
```

#### 4.2 Archive Implementation Details
**Move to `docs/archive/` (historical reference):**
```bash
mkdir -p docs/archive/implementation-details
mkdir -p docs/archive/fixes
mkdir -p docs/archive/ui-updates

# Implementation details
mv *_PLAN.md docs/archive/implementation-details/
mv *_ANALYSIS.md docs/archive/implementation-details/
mv *_ACHIEVEMENTS.md docs/archive/implementation-details/

# Fix documentation
mv *_FIX.md docs/archive/fixes/
mv *_FIXES_*.md docs/archive/fixes/

# UI updates
mv *_UPDATE.md docs/archive/ui-updates/
mv *_LAYOUT*.md docs/archive/ui-updates/
mv BUTTON_*.md docs/archive/ui-updates/
mv TAB_*.md docs/archive/ui-updates/
```

#### 4.3 Delete Obsolete Documentation
```bash
# Temporary files
rm yy.txt zz.txt
rm build_output*.txt
rm build_error.txt
rm FILES_TO_COPY.txt
rm LAUNCH_WITH_FIX.txt
rm RUN_ME_FOR_13_TABS.txt
rm SHARE_WITH_AI.txt

# Duplicate/redundant docs
rm COPY_OF_RESPONSE.md
rm gui_improvements.md  # If covered elsewhere
```

#### 4.4 Create Documentation Index
```bash
cat > docs/README.md << 'EOF'
# Vaughan Wallet Documentation

## Quick Links
- [Quickstart Guide](guides/quickstart.md)
- [Architecture Overview](architecture/overview.md)
- [Development Guide](development/building.md)
- [Security Audit](architecture/security-audit.md)

## Documentation Structure

### Architecture
Technical architecture and design decisions.

### Guides
User-facing guides for wallet features and setup.

### Development
Developer documentation for building and contributing.

### Features
Detailed documentation for specific features.

### Changelog
Version history and migration guides.

## Contributing
See [Contributing Guide](development/contributing.md) for how to contribute to documentation.
EOF
```

---

### Phase 5: Configure Git Attributes (15 minutes)
**Goal:** Fix GitHub language statistics

#### 5.1 Create .gitattributes
```bash
cat > .gitattributes << 'EOF'
# Vaughan Wallet - Git Attributes Configuration

# Mark vendored code (don't count in language stats)
lib/** linguist-vendored
contracts/lib/** linguist-vendored

# Mark documentation
*.md linguist-documentation

# Mark generated files
src/generated/** linguist-generated
out/** linguist-generated
target/** linguist-generated

# Solidity is for testing, not primary language
*.sol linguist-language=Solidity

# Rust is the primary language
*.rs linguist-language=Rust

# Scripts are utilities
scripts/** linguist-detectable=false

# Line ending configuration
* text=auto
*.rs text eol=lf
*.toml text eol=lf
*.md text eol=lf
*.sh text eol=lf
*.py text eol=lf

# Binary files
*.png binary
*.jpg binary
*.ico binary
*.woff binary
*.woff2 binary
EOF
```

#### 5.2 Update .gitignore
Add to `.gitignore`:
```bash
# Build artifacts
/target/
/out/
/cache/
/bytecode/

# Environment
.env
.env.local

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Temporary files
*.tmp
*.log
*.bak
*.backup

# Foundry
cache/
broadcast/
```

---

### Phase 6: Update Build Configuration (30 minutes)

#### 6.1 Update Makefile
```makefile
# Vaughan Wallet - Development Makefile

.PHONY: help build test clean dev release docs

help:
	@echo "Vaughan Wallet - Development Commands"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Launch wallet in development mode"
	@echo "  make build        - Build in debug mode"
	@echo "  make release      - Build optimized release"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Testing:"
	@echo "  make test         - Run all tests"
	@echo "  make test-unit    - Run unit tests"
	@echo "  make test-int     - Run integration tests"
	@echo ""
	@echo "Contracts:"
	@echo "  make contracts    - Build smart contracts"
	@echo "  make test-contracts - Test smart contracts"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs         - Generate documentation"

dev:
	@./scripts/dev/launch.sh

build:
	@cargo build

release:
	@cargo build --release

clean:
	@cargo clean
	@cd contracts && forge clean

test:
	@cargo test
	@cd contracts && forge test

test-unit:
	@cargo test --lib

test-int:
	@./scripts/test/integration/run-all.sh

contracts:
	@cd contracts && forge build

test-contracts:
	@cd contracts && forge test -vvv

docs:
	@cargo doc --no-deps --open
```

#### 6.2 Create Launch Script
```bash
cat > scripts/dev/launch.sh << 'EOF'
#!/bin/bash
# Vaughan Wallet Development Launcher

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

echo "🚀 Launching Vaughan Wallet..."
echo "📁 Project: $PROJECT_ROOT"
echo ""

# Check if built
if [ ! -f "target/release/vaughan" ]; then
    echo "⚠️  Release binary not found. Building..."
    cargo build --release
fi

# Set environment
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

# Launch
echo "🎯 Starting wallet..."
./target/release/vaughan

echo "👋 Wallet closed."
EOF

chmod +x scripts/dev/launch.sh
```

---

### Phase 7: Create Professional README (1 hour)

#### 7.1 Update Root README.md
Create a professional README with:
- Project description
- Features
- Installation
- Quick start
- Documentation links
- Contributing
- License

#### 7.2 Add SECURITY.md
```markdown
# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability, please email security@vaughan-wallet.com

Do NOT open a public issue.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Features

- Hardware wallet support
- Encrypted keystore
- Secure memory handling
- Sandboxed contract execution
```

---

## Success Metrics

### Before:
- 43 scripts in root directory
- 101 markdown files in root
- Solidity files in 3 different locations
- GitHub shows 30% Solidity
- No clear organization

### After:
- 0 scripts in root (all in `scripts/`)
- ~5 markdown files in root (README, LICENSE, SECURITY, etc.)
- All Solidity in `contracts/src/`
- GitHub shows 85%+ Rust
- Professional directory structure

### Quality Improvements:
- ✅ Clear separation of concerns
- ✅ Easy to find documentation
- ✅ Proper script lifecycle management
- ✅ Accurate language statistics
- ✅ Professional project presentation
- ✅ Easy onboarding for contributors
- ✅ Maintainable structure

---

## Maintenance Guidelines

### Script Management
1. **Active scripts** go in `scripts/{dev,build,test,setup,tools}/`
2. **Deprecated scripts** move to `scripts/archive/`
3. **Delete archived scripts** after 6 months if unused
4. **Document purpose** in script header comments

### Documentation Management
1. **User docs** in `docs/guides/`
2. **Developer docs** in `docs/development/`
3. **Architecture docs** in `docs/architecture/`
4. **Historical docs** in `docs/archive/`
5. **Update docs** when features change

### Contract Management
1. **Production contracts** in `contracts/src/`
2. **Test contracts** in `contracts/test/`
3. **Deployment scripts** in `contracts/script/`
4. **Never put .sol files** in `src/` (Rust directory)

### Git Hygiene
1. **Review .gitattributes** when adding new file types
2. **Update .gitignore** for new build artifacts
3. **Keep submodules** in `lib/` or `contracts/lib/`
4. **Tag releases** with semantic versioning

---

## Rollout Strategy

### Week 1: Foundation
- **Day 1:** Phase 1 (Create structure)
- **Day 2:** Phase 2 (Organize scripts)
- **Day 3:** Phase 3 (Organize contracts)

### Week 2: Documentation
- **Day 1-2:** Phase 4 (Organize documentation)
- **Day 3:** Phase 5 (Git attributes)

### Week 3: Polish
- **Day 1:** Phase 6 (Build configuration)
- **Day 2:** Phase 7 (Professional README)
- **Day 3:** Testing and validation

### Validation Checklist
- [ ] All scripts execute from new locations
- [ ] Documentation links work
- [ ] Contracts compile with new paths
- [ ] GitHub language stats updated
- [ ] CI/CD pipelines updated (if any)
- [ ] Team can find everything easily

---

## Risk Mitigation

### Backup Strategy
```bash
# Before starting, create backup
tar -czf vaughan-backup-$(date +%Y%m%d).tar.gz \
  --exclude=target \
  --exclude=lib \
  --exclude=.git \
  .
```

### Testing After Each Phase
```bash
# Verify builds still work
cargo build
cd contracts && forge build

# Verify scripts work
./scripts/dev/launch.sh --help
```

### Git Strategy
- Create branch: `cleanup/non-rust-organization`
- Commit after each phase
- Test before merging to main
- Tag before and after: `v0.1.0-pre-cleanup`, `v0.1.0-post-cleanup`

---

## Long-term Benefits

### Developer Experience
- New contributors can navigate easily
- Clear where to add new scripts/docs
- Reduced cognitive load
- Faster onboarding

### Project Health
- Professional appearance
- Accurate statistics
- Easier maintenance
- Better discoverability

### Community
- Clear contribution guidelines
- Well-organized documentation
- Professional presentation
- Increased trust

---

## Appendix: Automation Scripts

### A. Bulk Move Script
```bash
#!/bin/bash
# scripts/tools/bulk-organize.sh
# Automates the organization process

set -e

echo "🧹 Vaughan Cleanup Automation"
echo "=============================="
echo ""

# Phase 1: Create structure
echo "📁 Creating directory structure..."
mkdir -p docs/{architecture,guides,development,features,changelog,archive}
mkdir -p scripts/{dev,build,test/{integration,e2e},setup,tools,archive}
mkdir -p contracts/src/tokens

# Phase 2: Move scripts
echo "📜 Organizing scripts..."
# ... (add move commands from Phase 2)

# Phase 3: Move contracts
echo "📄 Organizing contracts..."
# ... (add move commands from Phase 3)

# Phase 4: Move documentation
echo "📚 Organizing documentation..."
# ... (add move commands from Phase 4)

echo ""
echo "✅ Organization complete!"
echo "🔍 Please review changes before committing."
```

### B. Validation Script
```bash
#!/bin/bash
# scripts/tools/validate-structure.sh
# Validates the new structure

set -e

echo "🔍 Validating project structure..."

# Check required directories exist
REQUIRED_DIRS=(
  "docs/architecture"
  "docs/guides"
  "docs/development"
  "scripts/dev"
  "scripts/build"
  "scripts/test"
  "contracts/src/tokens"
)

for dir in "${REQUIRED_DIRS[@]}"; do
  if [ ! -d "$dir" ]; then
    echo "❌ Missing directory: $dir"
    exit 1
  fi
done

# Check no scripts in root
SCRIPT_COUNT=$(find . -maxdepth 1 -name "*.sh" -o -name "*.py" | wc -l)
if [ "$SCRIPT_COUNT" -gt 0 ]; then
  echo "⚠️  Found $SCRIPT_COUNT scripts in root directory"
fi

# Check no .sol files in src/
SOL_IN_SRC=$(find src -name "*.sol" 2>/dev/null | wc -l)
if [ "$SOL_IN_SRC" -gt 0 ]; then
  echo "❌ Found .sol files in src/ directory"
  exit 1
fi

echo "✅ Structure validation passed!"
```

---

## Notes

This plan follows industry best practices from:
- Rust project conventions (Cargo book)
- Foundry project structure
- GitHub repository standards
- Open source project guidelines

**Estimated Total Time:** 8-10 hours of focused work

**Priority:** High - Do before public release or major feature additions

**Dependencies:** None - can be done independently of code refactoring
