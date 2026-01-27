//! Account Display Service - Account formatting and display logic
//!
//! This service extracts account display logic from view components,
//! providing consistent formatting for addresses and account information.

/// Information about an account formatted for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDisplayInfo {
    /// Account identifier
    pub id: String,
    /// Account name
    pub name: String,
    /// Full address (e.g., "0x1234...abcd")
    pub address: String,
    /// Shortened address for display (e.g., "0x1234...abcd")
    pub short_address: String,
    /// Human-readable account type label
    pub account_type: String,
}

/// Trait defining the account display service interface for testability.
pub trait AccountDisplayServiceTrait: Send + Sync {
    /// Format an address for short display (first 6 + last 4 characters).
    fn format_address_short(&self, address: &str) -> String;
    
    /// Get a human-readable label for an account type.
    fn get_account_type_label(&self, account_type: &str) -> String;
    
    /// Create display info from account data.
    fn create_display_info(
        &self,
        id: &str,
        name: &str,
        address: &str,
        account_type: &str,
    ) -> AccountDisplayInfo;
}

/// Account display service implementation.
#[derive(Debug, Default)]
pub struct AccountDisplayService;

impl AccountDisplayService {
    /// Create a new account display service.
    pub fn new() -> Self {
        Self
    }
}

impl AccountDisplayServiceTrait for AccountDisplayService {
    fn format_address_short(&self, address: &str) -> String {
        let trimmed = address.trim();
        
        // Handle empty or very short addresses
        if trimmed.len() <= 10 {
            return trimmed.to_string();
        }
        
        // Standard format: first 6 chars + "..." + last 4 chars
        // For addresses like "0x1234567890abcdef..." this gives "0x1234...cdef"
        let prefix_len = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            6 // Include "0x" + 4 hex chars
        } else {
            6 // Just 6 chars
        };
        
        let suffix_len = 4;
        
        if trimmed.len() <= prefix_len + suffix_len {
            return trimmed.to_string();
        }
        
        format!(
            "{}...{}",
            &trimmed[..prefix_len],
            &trimmed[trimmed.len() - suffix_len..]
        )
    }
    
    fn get_account_type_label(&self, account_type: &str) -> String {
        match account_type.to_lowercase().as_str() {
            "keystore" => "Keystore".to_string(),
            "seed" | "hd" | "hdwallet" => "HD Wallet".to_string(),
            "hardware" | "ledger" | "trezor" => "Hardware".to_string(),
            "imported" => "Imported".to_string(),
            "watch" | "watchonly" => "Watch Only".to_string(),
            _ => account_type.to_string(),
        }
    }
    
    fn create_display_info(
        &self,
        id: &str,
        name: &str,
        address: &str,
        account_type: &str,
    ) -> AccountDisplayInfo {
        AccountDisplayInfo {
            id: id.to_string(),
            name: name.to_string(),
            address: address.to_string(),
            short_address: self.format_address_short(address),
            account_type: self.get_account_type_label(account_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> AccountDisplayService {
        AccountDisplayService::new()
    }

    #[test]
    fn test_format_address_short_standard() {
        let s = service();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let short = s.format_address_short(address);
        assert_eq!(short, "0x1234...5678");
    }

    #[test]
    fn test_format_address_short_already_short() {
        let s = service();
        let address = "0x1234";
        let short = s.format_address_short(address);
        assert_eq!(short, "0x1234");
    }

    #[test]
    fn test_format_address_short_empty() {
        let s = service();
        let short = s.format_address_short("");
        assert_eq!(short, "");
    }

    #[test]
    fn test_format_address_short_no_prefix() {
        let s = service();
        let address = "1234567890abcdef1234567890abcdef12345678";
        let short = s.format_address_short(address);
        assert_eq!(short, "123456...5678");
    }

    #[test]
    fn test_format_address_short_with_whitespace() {
        let s = service();
        let address = "  0x1234567890abcdef1234567890abcdef12345678  ";
        let short = s.format_address_short(address);
        assert_eq!(short, "0x1234...5678");
    }

    #[test]
    fn test_format_address_short_contains_ellipsis() {
        let s = service();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let short = s.format_address_short(address);
        assert!(short.contains("..."));
    }

    #[test]
    fn test_format_address_short_length() {
        let s = service();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let short = s.format_address_short(address);
        assert!(short.len() < address.len());
        assert_eq!(short.len(), 13); // "0x1234...5678" = 13 chars
    }

    #[test]
    fn test_get_account_type_label_keystore() {
        let s = service();
        assert_eq!(s.get_account_type_label("keystore"), "Keystore");
        assert_eq!(s.get_account_type_label("KEYSTORE"), "Keystore");
    }

    #[test]
    fn test_get_account_type_label_seed() {
        let s = service();
        assert_eq!(s.get_account_type_label("seed"), "HD Wallet");
        assert_eq!(s.get_account_type_label("hd"), "HD Wallet");
        assert_eq!(s.get_account_type_label("hdwallet"), "HD Wallet");
    }

    #[test]
    fn test_get_account_type_label_hardware() {
        let s = service();
        assert_eq!(s.get_account_type_label("hardware"), "Hardware");
        assert_eq!(s.get_account_type_label("ledger"), "Hardware");
        assert_eq!(s.get_account_type_label("trezor"), "Hardware");
    }

    #[test]
    fn test_get_account_type_label_unknown() {
        let s = service();
        assert_eq!(s.get_account_type_label("custom"), "custom");
    }

    #[test]
    fn test_create_display_info() {
        let s = service();
        let info = s.create_display_info(
            "acc-1",
            "My Account",
            "0x1234567890abcdef1234567890abcdef12345678",
            "keystore",
        );
        
        assert_eq!(info.id, "acc-1");
        assert_eq!(info.name, "My Account");
        assert_eq!(info.address, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(info.short_address, "0x1234...5678");
        assert_eq!(info.account_type, "Keystore");
    }

    #[test]
    fn test_account_display_info_equality() {
        let s = service();
        let info1 = s.create_display_info("1", "Test", "0xabc", "keystore");
        let info2 = s.create_display_info("1", "Test", "0xabc", "keystore");
        assert_eq!(info1, info2);
    }
}
