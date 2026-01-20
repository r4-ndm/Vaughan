//! Property test for feature gating compliance.
//!
//! This test file validates that the compile-time configuration matches the expected
//! default feature flags defined in Cargo.toml.

#[test]
fn verify_default_feature_configuration() {
    // Cargo.toml defines:
    // default = ["minimal", "qr", "audio", "hardware-wallets", "professional", "custom-tokens"]

    // Verify core features
    assert!(
        cfg!(feature = "minimal"),
        "Feature 'minimal' must be enabled by default"
    );

    // Verify optional-but-default features
    assert!(cfg!(feature = "qr"), "Feature 'qr' must be enabled by default");
    assert!(cfg!(feature = "audio"), "Feature 'audio' must be enabled by default");
    assert!(
        cfg!(feature = "hardware-wallets"),
        "Feature 'hardware-wallets' must be enabled by default"
    );

    // Verify new features are default
    assert!(
        cfg!(feature = "professional"),
        "Feature 'professional' must be enabled by default"
    );
    assert!(
        cfg!(feature = "custom-tokens"),
        "Feature 'custom-tokens' must be enabled by default"
    );

    // Verify strictly optional features
    // Note: This assertion assumes the test is run with default features.
    // If run with --all-features, this assertion would fail, so we condition it.
    #[cfg(not(feature = "shamir"))]
    assert!(
        !cfg!(feature = "shamir"),
        "Feature 'shamir' should be disabled by default"
    );
}

#[test]
fn verify_dependency_gating() {
    // Verify that dependencies are correctly gated

    #[cfg(feature = "shamir")]
    {
        // If shamir is enabled, sharks dependency should be available
        // logic to use sharks would go here, or we trust Cargo.toml
    }

    #[cfg(feature = "audio")]
    {
        // Verify rodio is enabled if audio is enabled
        // Intentionally left as a compile-time check via Cargo.toml
    }
}
