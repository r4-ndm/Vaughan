//! Standalone wallet reset utility
//!
//! This bypasses all GUI issues and directly deletes wallet files

use std::io::{self, Write};

fn main() {
    println!("🗑️  Vaughan Wallet Reset Utility");
    println!("==================================");
    println!();
    println!("This will permanently delete all wallet data.");
    println!("You will lose access to all accounts unless you have backups.");
    println!();

    // List files that will be deleted
    let config_dir = dirs::config_dir()
        .map(|dir| dir.join("vaughan"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/vaughan"));

    println!("Files to delete:");
    let files_to_delete = ["wallet_metadata.json", "selected-provider.txt"];
    let mut files_found = 0;

    for filename in &files_to_delete {
        let file_path = config_dir.join(filename);
        if file_path.exists() {
            println!("  ✓ {}", file_path.display());
            files_found += 1;
        } else {
            println!("  - {} (not found)", file_path.display());
        }
    }

    if files_found == 0 {
        println!();
        println!("❌ No wallet files found. Wallet may already be reset.");
        return;
    }

    println!();
    print!("Type 'DELETE' to confirm wallet reset: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input != "DELETE" {
        println!("❌ Reset cancelled. You typed: '{input}'");
        println!("   You must type exactly 'DELETE' to confirm.");
        return;
    }

    println!();
    println!("🗑️  Deleting wallet files...");

    let mut deleted_count = 0;
    for filename in &files_to_delete {
        let file_path = config_dir.join(filename);
        if file_path.exists() {
            match std::fs::remove_file(&file_path) {
                Ok(_) => {
                    println!("  ✅ Deleted: {}", file_path.display());
                    deleted_count += 1;
                }
                Err(e) => {
                    println!("  ❌ Failed to delete {}: {}", file_path.display(), e);
                }
            }
        }
    }

    println!();
    if deleted_count > 0 {
        println!("🎉 Wallet reset successful! {deleted_count} files deleted.");
        println!();
        println!("Next steps:");
        println!("1. Run Vaughan: env VAUGHAN_SOFTWARE_RENDERING=1 cargo run --bin vaughan --release");
        println!("2. You should see the welcome screen");
        println!("3. Create a new wallet or import existing accounts");
    } else {
        println!("⚠️  No files were deleted.");
    }
}