//! GUI Application Launcher
//!
//! Graphics backend detection and application launching functionality
//! extracted from working_wallet.rs

use iced::Application;
use tracing;

use crate::gui::working_wallet::WorkingWalletApp;

pub fn launch_working_gui() -> iced::Result {
    tracing::info!("🎨 Initializing Vaughan GUI with graphics backend detection");

    // Check environment variables for forced graphics mode
    let force_software = std::env::var("VAUGHAN_SOFTWARE_RENDERING").is_ok();
    let force_minimal = std::env::var("VAUGHAN_MINIMAL_MODE").is_ok();

    // Detect graphics capabilities
    let graphics_info = detect_graphics_capabilities();
    tracing::info!("🖼️ Graphics Detection: {:?}", graphics_info);

    // Select the best settings based on environment and capabilities
    let (backend_name, settings) = if force_minimal {
        ("Minimal Safe Mode (Forced)", create_minimal_safe_settings())
    } else if force_software {
        ("Software Fallback (Forced)", create_software_fallback_settings())
    } else if graphics_info.suspected_hang_risk || !graphics_info.display_available {
        ("Software Fallback (Safe)", create_software_fallback_settings())
    } else {
        ("Hardware Accelerated", create_hardware_accelerated_settings())
    };

    tracing::info!("🔧 Launching GUI with: {}", backend_name);

    // Launch the application - only call run() once to avoid event loop recreation
    match std::panic::catch_unwind(|| WorkingWalletApp::run(settings)) {
        Ok(result) => {
            tracing::info!("✅ Successfully launched GUI with: {}", backend_name);
            result
        }
        Err(panic_info) => {
            tracing::error!("❌ GUI panic during startup: {:?}", panic_info);
            tracing::error!("   📌 Try running with fallback modes:");
            tracing::error!("   1. VAUGHAN_SOFTWARE_RENDERING=1 cargo run --bin vaughan");
            tracing::error!("   2. VAUGHAN_MINIMAL_MODE=1 cargo run --bin vaughan");
            std::process::exit(1);
        }
    }
}

/// Create hardware accelerated settings (default)
fn create_hardware_accelerated_settings() -> iced::Settings<()> {
    iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(600.0, 868.0),
            min_size: Some(iced::Size::new(600.0, 868.0)),
            resizable: true,
            decorations: true,
            transparent: false,
            position: iced::window::Position::Centered,
            visible: true,
            exit_on_close_request: true,
            ..Default::default()
        },
        antialiasing: true,
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        ..Default::default()
    }
}

/// Create software rendering fallback settings
fn create_software_fallback_settings() -> iced::Settings<()> {
    // Set environment variable to force software rendering in wgpu
    std::env::set_var("WGPU_BACKEND", "gl");
    std::env::set_var("WGPU_POWER_PREF", "low");

    iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(600.0, 868.0),
            min_size: Some(iced::Size::new(600.0, 868.0)),
            resizable: true,
            decorations: true,
            transparent: false,
            position: iced::window::Position::Centered,
            visible: true,
            exit_on_close_request: true,
            ..Default::default()
        },
        antialiasing: false, // Disable antialiasing for software rendering
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        ..Default::default()
    }
}

/// Create minimal safe mode settings
fn create_minimal_safe_settings() -> iced::Settings<()> {
    // Force the most compatible rendering settings
    std::env::set_var("WGPU_BACKEND", "gl");
    std::env::set_var("WGPU_POWER_PREF", "low");
    std::env::set_var(
        "WGPU_FEATURES",
        "TEXTURE_BINDING_ARRAY,SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING",
    );

    iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(600.0, 868.0),           // Standard size
            min_size: Some(iced::Size::new(600.0, 868.0)), // Minimum 850 height
            resizable: true,
            decorations: true,
            transparent: false,
            position: iced::window::Position::default(), // Let system decide
            visible: true,
            exit_on_close_request: true,
            ..Default::default()
        },
        antialiasing: false,
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(12.0), // Smaller text
        ..Default::default()
    }
}

/// Graphics capabilities information
#[derive(Debug)]
struct GraphicsCapabilities {
    display_available: bool,
    vulkan_available: bool,
    opengl_available: bool,
    driver_type: String,
    recommended_backend: String,
    nvidia_detected: bool,
    suspected_hang_risk: bool,
}

/// Detect system graphics capabilities (simplified to avoid hanging)
fn detect_graphics_capabilities() -> GraphicsCapabilities {
    let mut capabilities = GraphicsCapabilities {
        display_available: false,
        vulkan_available: false,
        opengl_available: false,
        driver_type: "unknown".to_string(),
        recommended_backend: "minimal".to_string(),
        nvidia_detected: false,
        suspected_hang_risk: false,
    };

    // Check if DISPLAY is set
    if std::env::var("DISPLAY").is_ok() {
        capabilities.display_available = true;
    }

    // Check for Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        capabilities.display_available = true;
    }

    // Simplified detection - avoid potentially hanging commands
    // Just check for environment variables and assume basic capabilities
    if capabilities.display_available {
        // If we have a display, assume we can try software rendering at least
        capabilities.opengl_available = true;
        capabilities.recommended_backend = "software".to_string();

        // Try to detect if we're on a common Linux system with basic graphics
        if std::path::Path::new("/dev/dri").exists() {
            capabilities.vulkan_available = true;
            capabilities.recommended_backend = "hardware".to_string();
        }

        // Enhanced driver detection
        capabilities.nvidia_detected = detect_nvidia_driver();
        if capabilities.nvidia_detected {
            capabilities.driver_type = "nvidia".to_string();
            // NVIDIA drivers sometimes have hardware acceleration issues
            capabilities.suspected_hang_risk = true;
            tracing::info!("🖼️ NVIDIA GPU detected - will try hardware first, then software fallback");
        }

        // Simple driver detection without running external commands
        if let Ok(contents) = std::fs::read_to_string("/proc/version") {
            let contents_lower = contents.to_lowercase();
            if contents_lower.contains("amd") {
                capabilities.driver_type = "amd".to_string();
            } else if contents_lower.contains("intel") {
                capabilities.driver_type = "intel".to_string();
            }
        }
    }

    capabilities
}

/// Detect NVIDIA graphics driver without hanging commands
fn detect_nvidia_driver() -> bool {
    // Check for NVIDIA kernel module
    if let Ok(contents) = std::fs::read_to_string("/proc/modules") {
        if contents.contains("nvidia") {
            return true;
        }
    }

    // Check for NVIDIA devices in /proc/driver
    if std::path::Path::new("/proc/driver/nvidia").exists() {
        return true;
    }

    // Check for NVIDIA in lspci output (but avoid running lspci directly)
    if std::path::Path::new("/sys/bus/pci/devices").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") {
            for entry in entries.flatten() {
                if let Ok(vendor) = std::fs::read_to_string(entry.path().join("vendor")) {
                    // NVIDIA vendor ID is 0x10de
                    if vendor.trim() == "0x10de" {
                        return true;
                    }
                }
            }
        }
    }

    false
}
