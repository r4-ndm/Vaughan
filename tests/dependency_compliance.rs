//! Property test for dependency compliance.
//!
//! This test checks Cargo.toml to ensure that heavy features are not accidentally enabled.

use std::fs;

#[test]
fn verify_optimized_dependencies() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // Check Alloy features
    let alloy_line = cargo_toml
        .lines()
        .find(|line| line.trim().starts_with("alloy ="))
        .expect("Could not find alloy dependency");

    // Ensure 'full' feature is NOT present
    assert!(
        !alloy_line.contains("\"full\""),
        "Alloy 'full' feature should not be enabled"
    );
    // Ensure 'provider-ws' is NOT present (we optimized it out)
    assert!(
        !alloy_line.contains("\"provider-ws\""),
        "Alloy 'provider-ws' feature should not be enabled"
    );

    // Check Iced features
    let iced_line = cargo_toml
        .lines()
        .find(|line| line.trim().starts_with("iced ="))
        .expect("Could not find iced dependency");

    // Ensure 'advanced' feature is NOT present
    assert!(
        !iced_line.contains("\"advanced\""),
        "Iced 'advanced' feature should not be enabled"
    );
}
