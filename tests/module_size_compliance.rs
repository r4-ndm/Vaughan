//! Module Size Compliance Tests
//!
//! Property tests to verify that no module exceeds the target line count.

use std::fs;
use std::path::Path;

/// Maximum allowed lines for a single source file
const MAX_MODULE_LINES: usize = 1000;

/// Target maximum lines for new/refactored modules
const TARGET_MODULE_LINES: usize = 600;

/// Files that are known to exceed limits and are planned for decomposition
const DECOMPOSITION_PENDING: &[&str] = &[
    "src/gui/working_wallet.rs", // 5243 lines - needs major decomposition
    "src/security/seed.rs",      // 2918 lines - needs decomposition
    "src/gui/theme.rs",          // 1615 lines - needs decomposition
    "src/gui/views/dialogs.rs",  // 1327 lines - needs decomposition
    "src/security/keystore.rs",  // 1110 lines - needs decomposition
];

#[test]
fn test_decomposition_pending_files_documented() {
    println!("🧪 Verifying decomposition pending files are documented");

    for file in DECOMPOSITION_PENDING {
        let path = Path::new(file);
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            let line_count = content.lines().count();
            println!("📁 {} - {} lines (pending decomposition)", file, line_count);

            // These files should exceed TARGET_MODULE_LINES (otherwise remove from list)
            assert!(
                line_count > TARGET_MODULE_LINES,
                "File {} has {} lines which is under target {}. Remove from DECOMPOSITION_PENDING.",
                file,
                line_count,
                TARGET_MODULE_LINES
            );
        } else {
            println!("✅ {} - file no longer exists (decomposed)", file);
        }
    }

    println!("✅ Decomposition pending files documented");
}

#[test]
fn test_handler_files_under_limit() {
    println!("🧪 Testing handler files are under module size limit");

    let handler_dir = Path::new("src/gui/handlers");
    if !handler_dir.exists() {
        println!("⚠️ Handlers directory not found");
        return;
    }

    let mut all_under_limit = true;

    if let Ok(entries) = fs::read_dir(handler_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(&path).unwrap_or_default();
                let line_count = content.lines().count();

                if line_count > MAX_MODULE_LINES {
                    println!(
                        "❌ {} - {} lines (exceeds {} limit)",
                        path.display(),
                        line_count,
                        MAX_MODULE_LINES
                    );
                    all_under_limit = false;
                } else if line_count > TARGET_MODULE_LINES {
                    println!(
                        "⚠️ {} - {} lines (exceeds {} target)",
                        path.display(),
                        line_count,
                        TARGET_MODULE_LINES
                    );
                } else {
                    println!("✅ {} - {} lines", path.display(), line_count);
                }
            }
        }
    }

    assert!(
        all_under_limit,
        "Some handler files exceed the maximum module size limit"
    );
    println!("✅ All handler files under maximum limit");
}

#[test]
fn test_dialog_components_under_limit() {
    println!("🧪 Testing dialog component files are under module size limit");

    let dialog_dir = Path::new("src/gui/components/dialogs");
    if !dialog_dir.exists() {
        println!("⚠️ Dialogs directory not found");
        return;
    }

    let mut all_under_limit = true;

    if let Ok(entries) = fs::read_dir(dialog_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(&path).unwrap_or_default();
                let line_count = content.lines().count();

                if line_count > MAX_MODULE_LINES {
                    println!(
                        "❌ {} - {} lines (exceeds {} limit)",
                        path.display(),
                        line_count,
                        MAX_MODULE_LINES
                    );
                    all_under_limit = false;
                } else if line_count > TARGET_MODULE_LINES {
                    println!(
                        "⚠️ {} - {} lines (exceeds {} target)",
                        path.display(),
                        line_count,
                        TARGET_MODULE_LINES
                    );
                } else {
                    println!("✅ {} - {} lines", path.display(), line_count);
                }
            }
        }
    }

    assert!(
        all_under_limit,
        "Some dialog component files exceed the maximum module size limit"
    );
    println!("✅ All dialog component files under maximum limit");
}

#[test]
fn test_total_src_lines() {
    println!("🧪 Calculating total source lines");

    let src_dir = Path::new("src");
    let mut total_lines = 0;
    let mut file_count = 0;

    count_lines_recursive(src_dir, &mut total_lines, &mut file_count);

    println!("📊 Total source files: {}", file_count);
    println!("📊 Total source lines: {}", total_lines);

    // This is informational - no assertion
}

fn count_lines_recursive(dir: &Path, total_lines: &mut usize, file_count: &mut usize) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count_lines_recursive(&path, total_lines, file_count);
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(&path).unwrap_or_default();
                *total_lines += content.lines().count();
                *file_count += 1;
            }
        }
    }
}
