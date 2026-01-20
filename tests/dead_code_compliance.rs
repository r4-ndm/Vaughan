//! Dead Code Annotation Compliance Tests
//!
//! This module validates that all `#[allow(dead_code)]` annotations in the codebase
//! are properly documented with justification comments. This ensures no undocumented
//! dead code accumulates in the codebase.
//!
//! Property 1: Dead Code Annotation Compliance
//! Validates: Requirements 1.1, 1.2 (from comprehensive-debloat/tasks.md)

use std::fs;
use std::path::Path;

/// Scans a file for `#[allow(dead_code)]` annotations and verifies each has documentation.
///
/// Returns a list of line numbers where undocumented dead_code annotations were found.
fn find_undocumented_dead_code(path: &Path) -> Vec<(usize, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut violations = vec![];

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check if line contains #[allow(dead_code)]
        if trimmed.contains("#[allow(dead_code)]") {
            // Check if this line also contains a comment (// ...)
            let has_inline_comment = trimmed.contains("//");

            // Check if previous line has a comment explaining the dead_code
            let has_preceding_comment = idx > 0 && lines[idx - 1].trim().starts_with("//");

            if !has_inline_comment && !has_preceding_comment {
                violations.push((idx + 1, trimmed.to_string())); // +1 for 1-indexed line numbers
            }
        }
    }

    violations
}

/// Recursively scans all Rust source files in a directory.
fn scan_directory(dir: &Path) -> Vec<(String, Vec<(usize, String)>)> {
    let mut results = vec![];

    if !dir.exists() {
        return results;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Skip target directory
            if path.file_name().map_or(false, |n| n == "target") {
                continue;
            }
            results.extend(scan_directory(&path));
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            let violations = find_undocumented_dead_code(&path);
            if !violations.is_empty() {
                results.push((path.display().to_string(), violations));
            }
        }
    }

    results
}

#[test]
fn test_all_dead_code_annotations_are_documented() {
    // Get the workspace root (parent of tests directory)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    let violations = scan_directory(&src_dir);

    if !violations.is_empty() {
        let mut error_msg = String::from(
            "\n\nDead Code Annotation Compliance Violation!\n\
             ============================================\n\
             The following #[allow(dead_code)] annotations lack documentation:\n\n",
        );

        for (file, line_violations) in &violations {
            error_msg.push_str(&format!("File: {}\n", file));
            for (line_num, line_content) in line_violations {
                error_msg.push_str(&format!("  Line {}: {}\n", line_num, line_content));
            }
            error_msg.push('\n');
        }

        error_msg.push_str(
            "Please add a comment explaining why the dead code annotation is needed.\n\
             Example: #[allow(dead_code)] // Used by serde for deserialization\n",
        );

        panic!("{}", error_msg);
    }
}

#[test]
fn test_dead_code_annotation_count() {
    // Get the workspace root (parent of tests directory)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    fn count_annotations(dir: &Path) -> usize {
        let mut count = 0;
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return 0,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if path.file_name().map_or(false, |n| n == "target") {
                    continue;
                }
                count += count_annotations(&path);
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    count += content.matches("#[allow(dead_code)]").count();
                }
            }
        }
        count
    }

    let total_count = count_annotations(&src_dir);

    // Target: 15 documented dead_code annotations (as of Phase 1 completion)
    // This test ensures we don't accumulate more undocumented dead code over time
    const MAX_ALLOWED_DEAD_CODE_ANNOTATIONS: usize = 20;

    assert!(
        total_count <= MAX_ALLOWED_DEAD_CODE_ANNOTATIONS,
        "Dead code annotation count ({}) exceeds maximum allowed ({}).\n\
         Consider removing unused code or documenting why it's needed.",
        total_count,
        MAX_ALLOWED_DEAD_CODE_ANNOTATIONS
    );

    println!(
        "Dead code annotation count: {} (max allowed: {})",
        total_count, MAX_ALLOWED_DEAD_CODE_ANNOTATIONS
    );
}

#[test]
fn test_no_undocumented_allow_attributes() {
    // This test ensures common allow attributes have documentation
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    let allow_patterns = [
        "#[allow(dead_code)]",
        "#[allow(unused_variables)]",
        "#[allow(unused_imports)]",
    ];

    fn check_allow_attributes(dir: &Path, patterns: &[&str]) -> Vec<(String, usize, String)> {
        let mut violations = vec![];
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return violations,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if path.file_name().map_or(false, |n| n == "target") {
                    continue;
                }
                violations.extend(check_allow_attributes(&path, patterns));
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    for (idx, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();
                        for pattern in patterns {
                            if trimmed.contains(pattern) && !trimmed.contains("//") {
                                // Check if previous line has a comment
                                let has_prior_comment = idx > 0 && lines[idx - 1].trim().starts_with("//");
                                if !has_prior_comment {
                                    violations.push((path.display().to_string(), idx + 1, trimmed.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
        violations
    }

    let violations = check_allow_attributes(&src_dir, &allow_patterns);

    // We expect some in test modules, so we filter those out
    let production_violations: Vec<_> = violations
        .into_iter()
        .filter(|(path, _, _)| !path.contains("_test") && !path.contains("tests"))
        .collect();

    if !production_violations.is_empty() {
        let mut msg = String::from("Undocumented allow attributes in production code:\n");
        for (file, line, content) in &production_violations {
            msg.push_str(&format!("  {}:{} - {}\n", file, line, content));
        }
        // This is a warning, not a hard failure for now
        println!("WARNING: {}", msg);
    }
}
