//! Asset Service - Centralized asset loading and availability checks
//!
//! This service extracts file system operations from view components,
//! providing a clean interface for checking asset availability and paths.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Trait defining the asset service interface for testability.
pub trait AssetServiceTrait: Send + Sync {
    /// Check if an asset file exists at the given path (relative to asset root).
    fn is_asset_available(&self, asset_path: &str) -> bool;
    
    /// Get the full path to the logo file if it exists.
    fn get_logo_path(&self) -> Option<PathBuf>;
    
    /// Get the full path to an icon file by name (e.g., "hamburger" -> "hamburger-128.png").
    fn get_icon_path(&self, icon_name: &str) -> Option<PathBuf>;
    
    /// Check if the logo asset is available.
    fn is_logo_available(&self) -> bool;
    
    /// Check if an icon is available by name.
    fn is_icon_available(&self, icon_name: &str) -> bool;
}

/// Asset service implementation with caching for performance.
#[derive(Debug)]
pub struct AssetService {
    /// Root directory for assets
    asset_root: PathBuf,
    /// Cache for asset availability checks
    cache: RwLock<HashMap<String, bool>>,
}

impl AssetService {
    /// Create a new asset service with the specified asset root directory.
    pub fn new(asset_root: PathBuf) -> Self {
        Self {
            asset_root,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get the logo filename.
    const LOGO_FILENAME: &'static str = "vaughan-logo-513x76-thor.png";
    
    /// Get the icon filename format.
    fn icon_filename(icon_name: &str) -> String {
        format!("{}-128.png", icon_name)
    }
}

impl Default for AssetService {
    fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}

impl AssetServiceTrait for AssetService {
    fn is_asset_available(&self, asset_path: &str) -> bool {
        // Check cache first
        if let Ok(cache) = self.cache.read() {
            if let Some(&exists) = cache.get(asset_path) {
                return exists;
            }
        }
        
        // Check filesystem
        let full_path = self.asset_root.join(asset_path);
        let exists = full_path.exists();
        
        // Update cache
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(asset_path.to_string(), exists);
        }
        
        exists
    }
    
    fn get_logo_path(&self) -> Option<PathBuf> {
        let path = self.asset_root.join(Self::LOGO_FILENAME);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
    
    fn get_icon_path(&self, icon_name: &str) -> Option<PathBuf> {
        let filename = Self::icon_filename(icon_name);
        let path = self.asset_root.join(&filename);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
    
    fn is_logo_available(&self) -> bool {
        self.is_asset_available(Self::LOGO_FILENAME)
    }
    
    fn is_icon_available(&self, icon_name: &str) -> bool {
        let filename = Self::icon_filename(icon_name);
        self.is_asset_available(&filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn setup_test_assets() -> (TempDir, AssetService) {
        let temp_dir = TempDir::new().unwrap();
        let service = AssetService::new(temp_dir.path().to_path_buf());
        (temp_dir, service)
    }

    #[test]
    fn test_is_asset_available_missing() {
        let (_temp_dir, service) = setup_test_assets();
        assert!(!service.is_asset_available("nonexistent.png"));
    }

    #[test]
    fn test_is_asset_available_exists() {
        let (temp_dir, service) = setup_test_assets();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.png");
        File::create(&test_file).unwrap();
        
        assert!(service.is_asset_available("test.png"));
    }

    #[test]
    fn test_caching_behavior() {
        let (temp_dir, service) = setup_test_assets();
        
        // First check - file doesn't exist
        assert!(!service.is_asset_available("cached.png"));
        
        // Create the file
        let test_file = temp_dir.path().join("cached.png");
        File::create(&test_file).unwrap();
        
        // Cache should still return false (cached result)
        assert!(!service.is_asset_available("cached.png"));
    }

    #[test]
    fn test_get_logo_path_missing() {
        let (_temp_dir, service) = setup_test_assets();
        assert!(service.get_logo_path().is_none());
    }

    #[test]
    fn test_get_logo_path_exists() {
        let (temp_dir, service) = setup_test_assets();
        
        // Create logo file
        let logo_path = temp_dir.path().join(AssetService::LOGO_FILENAME);
        File::create(&logo_path).unwrap();
        
        let result = service.get_logo_path();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), logo_path);
    }

    #[test]
    fn test_get_icon_path() {
        let (temp_dir, service) = setup_test_assets();
        
        // Create icon file
        let icon_path = temp_dir.path().join("hamburger-128.png");
        File::create(&icon_path).unwrap();
        
        let result = service.get_icon_path("hamburger");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), icon_path);
    }

    #[test]
    fn test_is_logo_available() {
        let (temp_dir, service) = setup_test_assets();
        
        assert!(!service.is_logo_available());
        
        // Create logo file
        let logo_path = temp_dir.path().join(AssetService::LOGO_FILENAME);
        File::create(&logo_path).unwrap();
        
        // Need new service instance due to caching
        let service2 = AssetService::new(temp_dir.path().to_path_buf());
        assert!(service2.is_logo_available());
    }

    #[test]
    fn test_is_icon_available() {
        let (temp_dir, service) = setup_test_assets();
        
        assert!(!service.is_icon_available("clipboard"));
        
        // Create icon file
        let icon_path = temp_dir.path().join("clipboard-128.png");
        File::create(&icon_path).unwrap();
        
        // Need new service instance due to caching
        let service2 = AssetService::new(temp_dir.path().to_path_buf());
        assert!(service2.is_icon_available("clipboard"));
    }

    #[test]
    fn test_default_asset_root() {
        let service = AssetService::default();
        assert_eq!(service.asset_root, PathBuf::from("assets"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    /// Integration test: Load real asset files
    #[test]
    fn test_load_real_assets() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create real asset files
        let logo_path = temp_dir.path().join("vaughan-logo-513x76-thor.png");
        let hamburger_path = temp_dir.path().join("hamburger-128.png");
        let clipboard_path = temp_dir.path().join("clipboard-128.png");
        
        File::create(&logo_path).unwrap();
        File::create(&hamburger_path).unwrap();
        File::create(&clipboard_path).unwrap();
        
        let service = AssetService::new(temp_dir.path().to_path_buf());
        
        // Test logo loading
        assert!(service.is_logo_available());
        assert!(service.get_logo_path().is_some());
        assert_eq!(service.get_logo_path().unwrap(), logo_path);
        
        // Test icon loading
        assert!(service.is_icon_available("hamburger"));
        assert!(service.is_icon_available("clipboard"));
        assert_eq!(service.get_icon_path("hamburger").unwrap(), hamburger_path);
        assert_eq!(service.get_icon_path("clipboard").unwrap(), clipboard_path);
    }

    /// Integration test: Caching behavior with multiple calls
    #[test]
    fn test_caching_performance() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test-asset.png");
        File::create(&test_file).unwrap();
        
        let service = AssetService::new(temp_dir.path().to_path_buf());
        
        // First call - should check filesystem and cache result
        let result1 = service.is_asset_available("test-asset.png");
        assert!(result1);
        
        // Second call - should use cached result
        let result2 = service.is_asset_available("test-asset.png");
        assert!(result2);
        
        // Verify cache contains the entry
        let cache = service.cache.read().unwrap();
        assert!(cache.contains_key("test-asset.png"));
        assert_eq!(cache.get("test-asset.png"), Some(&true));
    }

    /// Integration test: Handle missing assets gracefully
    #[test]
    fn test_missing_assets_graceful() {
        let temp_dir = TempDir::new().unwrap();
        let service = AssetService::new(temp_dir.path().to_path_buf());
        
        // Test missing logo
        assert!(!service.is_logo_available());
        assert!(service.get_logo_path().is_none());
        
        // Test missing icons
        assert!(!service.is_icon_available("nonexistent"));
        assert!(service.get_icon_path("nonexistent").is_none());
        
        // Verify cache stores negative results
        let cache = service.cache.read().unwrap();
        assert_eq!(cache.get("vaughan-logo-513x76-thor.png"), Some(&false));
        assert_eq!(cache.get("nonexistent-128.png"), Some(&false));
    }

    /// Integration test: Test with corrupted/invalid paths
    #[test]
    fn test_invalid_paths() {
        let service = AssetService::new(PathBuf::from("/invalid/nonexistent/path"));
        
        // Should handle invalid root gracefully
        assert!(!service.is_asset_available("any-file.png"));
        assert!(!service.is_logo_available());
        assert!(!service.is_icon_available("hamburger"));
        
        // Should return None for paths
        assert!(service.get_logo_path().is_none());
        assert!(service.get_icon_path("hamburger").is_none());
    }

    /// Integration test: Multiple asset types in same directory
    #[test]
    fn test_multiple_asset_types() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create various asset files
        File::create(temp_dir.path().join("vaughan-logo-513x76-thor.png")).unwrap();
        File::create(temp_dir.path().join("hamburger-128.png")).unwrap();
        File::create(temp_dir.path().join("clipboard-128.png")).unwrap();
        File::create(temp_dir.path().join("custom-icon-128.png")).unwrap();
        
        let service = AssetService::new(temp_dir.path().to_path_buf());
        
        // All assets should be available
        assert!(service.is_logo_available());
        assert!(service.is_icon_available("hamburger"));
        assert!(service.is_icon_available("clipboard"));
        assert!(service.is_icon_available("custom-icon"));
        
        // Verify paths are correct
        assert!(service.get_logo_path().unwrap().ends_with("vaughan-logo-513x76-thor.png"));
        assert!(service.get_icon_path("hamburger").unwrap().ends_with("hamburger-128.png"));
        assert!(service.get_icon_path("clipboard").unwrap().ends_with("clipboard-128.png"));
        assert!(service.get_icon_path("custom-icon").unwrap().ends_with("custom-icon-128.png"));
    }
}
