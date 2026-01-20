#!/bin/bash
# Vaughan Wallet Backup Script
# Usage: ./scripts/backup_wallet.sh

set -e

# Configuration
VAUGHAN_DIR="$HOME/.vaughan"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="$VAUGHAN_DIR/backup-pre-migration-$TIMESTAMP"
CHECKSUMS_FILE="$BACKUP_DIR/checksums.txt"
README_FILE="$BACKUP_DIR/README.txt"

# Create backup directory
echo "Creating backup directory: $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"

# Backup files
echo "Backing up wallet files..."
cp "$VAUGHAN_DIR/wallet.json" "$BACKUP_DIR/" 2>/dev/null || echo "No wallet.json"
cp "$VAUGHAN_DIR/accounts.json" "$BACKUP_DIR/" 2>/dev/null || echo "No accounts.json"
cp "$VAUGHAN_DIR/networks.json" "$BACKUP_DIR/" 2>/dev/null || echo "No networks.json"

# Calculate checksums
echo "Calculating checksums..."
cd "$BACKUP_DIR"
sha256sum wallet.json > "$CHECKSUMS_FILE" 2>/dev/null || true
sha256sum accounts.json >> "$CHECKSUMS_FILE" 2>/dev/null || true
sha256sum networks.json >> "$CHECKSUMS_FILE" 2>/dev/null || true

# Create README
cat > "$README_FILE" << 'EOREADME'
# Vaughan Wallet Backup

This directory contains a backup of your Vaughan wallet data created before migration.

## Files

- wallet.json: Main wallet configuration
- accounts.json: Account metadata
- networks.json: Network configuration
- checksums.txt: SHA256 checksums for verification

## Restoration

To restore from backup, run:
  cp -r . ~/.vaughan/

To verify backup integrity, run:
  sha256sum -c checksums.txt

## Migration Notes

If you already created a new MetaMask-compatible wallet, you can restore this backup later if needed.
EOREADME

echo "✅ Backup created successfully: $BACKUP_DIR"
echo "Please store this backup in a safe location."
