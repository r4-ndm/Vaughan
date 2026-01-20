
// Safe utility functions to replace common unwrap patterns

/// Safe string conversion with default fallback
pub fn safe_to_string<T: ToString>(value: &T) -> String {
    value.to_string()
}

/// Safe UTF-8 conversion with lossy fallback
pub fn safe_from_utf8(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap_or_else(|e| {
        String::from_utf8_lossy(&e.into_bytes()).to_string()
    })
}

/// Safe parse with default value
pub fn safe_parse_or_default<T: std::str::FromStr + Default>(s: &str) -> T {
    s.parse().unwrap_or_default()
}

/// Safe lock acquisition with error handling
pub fn safe_lock<T>(lock: &std::sync::Mutex<T>) -> Result<std::sync::MutexGuard<T>> {
    lock.lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire lock: mutex poisoned"))
}
