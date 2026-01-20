use std::fs;
use std::path::Path;

#[test]
fn test_no_test_files_in_src() {
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        // If src doesn't exist, we can't test it, but it should exist.
        // For the purpose of this test, we assume it exists or we fail.
        panic!("src directory not found");
    }

    let mut test_files = Vec::new();
    find_test_files(src_dir, &mut test_files);

    if !test_files.is_empty() {
        panic!("Found test files in src directory:\n{}", test_files.join("\n"));
    }
}

fn find_test_files(dir: &Path, test_files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_test_files(&path, test_files);
            } else if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.ends_with("_test.rs") || filename.ends_with("_tests.rs") {
                    test_files.push(path.display().to_string());
                }
            }
        }
    }
}
