#!/bin/bash
# Vaughan Wallet Rollback Script
# Usage: ./scripts/rollback_wallet.sh [backup-dir]

set -e

# Configuration
VAUGHAN_DIR="$HOME/.vaughan"

# Check for backup directory parameter
if [ -z "$1" ]; then
    echo "Usage: $0 [backup-directory]"
    echo ""
    echo "Available backups:"
    ls -d "$VAUGHAN_DIR"/backup-pre-migration-* 2>/dev/null || echo "  (no backups found)"
    exit 1
fi

BACKUP_DIR="$1"

# Verify backup exists
if [ ! -d "$BACKUP_DIR" ]; then
    echo "❌ Error: Backup directory not found: $BACKUP_DIR"
    exit 1
fi

# Verify checksums
echo "Verifying backup integrity..."
cd "$BACKUP_DIR"
if ! sha256sum -c checksums.txt 2>/dev/null; then
    echo "⚠️  Warning: Checksum verification failed"
    echo "Backup may be corrupted. Please verify manually."
    echo ""
    read -p "Continue anyway? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Rollback cancelled."
        exit 0
    fi
fi

# Confirm rollback
echo ""
echo "⚠️  WARNING: This will REPLACE current wallet data with backup!"
echo "   Backup: $BACKUP_DIR"
echo "   Current: $VAUGHAN_DIR"
echo ""
read -p "Do you want to continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Rollback cancelled."
    exit 0
fi

# Create safety backup
echo "Creating safety backup..."
SAFETY_BACKUP="$VAUGHAN_DIR/safety-before-rollback-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$SAFETY_BACKUP"
cp "$VAUGHAN_DIR"/*.json "$SAFETY_BACKUP/" 2>/dev/null || true

# Perform rollback
echo "Restoring from backup..."
cp "$BACKUP_DIR/wallet.json" "$VAUGHAN_DIR/" 2>/dev/null || echo "No wallet.json to restore"
cp "$BACKUP_DIR/accounts.json" "$VAUGHAN_DIR/" 2>/dev/null || echo "No accounts.json to restore"
cp "$BACKUP_DIR/networks.json" "$VAUGHAN_DIR/" 2>/dev/null || echo "No networks.json to restore"

echo "✅ Rollback completed successfully!"
echo "   Safety backup: $SAFETY_BACKUP"
