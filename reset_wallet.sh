#!/bin/bash

echo "🗑️  Vaughan Wallet Reset Script"
echo "================================"
echo ""
echo "This will DELETE all wallet data and return you to the welcome screen."
echo ""
echo "Files that will be deleted:"
echo "  - ~/.config/vaughan/wallet_metadata.json"
echo "  - ~/.config/vaughan/selected-provider.txt"
echo ""
read -p "Are you sure you want to reset your wallet? (y/N): " confirm

if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
    echo ""
    echo "🗑️  Deleting wallet files..."

    # Delete wallet files
    rm -f ~/.config/vaughan/wallet_metadata.json
    rm -f ~/.config/vaughan/selected-provider.txt

    echo "✅  Wallet reset complete!"
    echo ""
    echo "You can now run Vaughan and it will show the welcome screen:"
    echo "  env VAUGHAN_SOFTWARE_RENDERING=1 ./target/release/vaughan"
    echo ""
    echo "🎉  Ready to create a new wallet!"
else
    echo ""
    echo "❌  Reset cancelled."
fi