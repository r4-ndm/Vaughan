// Quick test to verify arboard clipboard works on Windows

fn main() {
    println!("Testing clipboard functionality...");
    
    // Test 1: Create clipboard
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            println!("✅ Clipboard created successfully");
            
            // Test 2: Try to read clipboard
            match clipboard.get_text() {
                Ok(text) => {
                    println!("✅ Successfully read from clipboard:");
                    println!("   Content: {}", text);
                }
                Err(e) => {
                    println!("❌ Failed to read from clipboard: {}", e);
                }
            }
            
            // Test 3: Try to write to clipboard
            match clipboard.set_text("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb") {
                Ok(_) => {
                    println!("✅ Successfully wrote test address to clipboard");
                    
                    // Test 4: Read it back
                    match clipboard.get_text() {
                        Ok(text) => {
                            println!("✅ Successfully read back from clipboard:");
                            println!("   Content: {}", text);
                        }
                        Err(e) => {
                            println!("❌ Failed to read back from clipboard: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to write to clipboard: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create clipboard: {}", e);
        }
    }
}
