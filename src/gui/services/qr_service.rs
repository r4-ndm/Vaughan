//! QR Code Generation Service
//!
//! Generates QR codes for Ethereum addresses and payment requests.

#![cfg_attr(not(feature = "qr"), allow(dead_code))]

#[cfg(feature = "qr")]
use iced::widget::image::Handle;
#[cfg(feature = "qr")]
use image::{ImageBuffer, Rgba};
#[cfg(feature = "qr")]
use qrcode::{EcLevel, QrCode};

/// Placeholder when QR feature is disabled
#[cfg(not(feature = "qr"))]
pub fn generate_address_qr_code(_address: &str) -> Result<(), String> {
    Err("QR code generation is disabled. Enable the 'qr' feature in Cargo.toml".to_string())
}

/// Placeholder when QR feature is disabled
#[cfg(not(feature = "qr"))]
pub fn generate_payment_request_qr_code(
    _address: &str,
    _chain_id: Option<u64>,
    _amount: Option<&str>,
) -> Result<(), String> {
    Err("QR code generation is disabled. Enable the 'qr' feature in Cargo.toml".to_string())
}

/// Generate a QR code for an Ethereum address
///
/// Returns the QR code as an iced image Handle that can be displayed in the UI
#[cfg(feature = "qr")]
pub fn generate_address_qr_code(address: &str) -> Result<Handle, String> {
    // Create QR code with high error correction
    let qr = QrCode::with_error_correction_level(address, EcLevel::H)
        .map_err(|e| format!("Failed to generate QR code: {e}"))?;

    // Get the QR code matrix
    let qr_matrix = qr.to_colors();
    let width = qr.width();

    // Scale factor for better visibility
    let scale = 10;
    let img_size = width * scale;

    // Create RGBA image
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(img_size as u32, img_size as u32);

    // Fill the image
    for (y, row) in qr_matrix.chunks(width).enumerate() {
        for (x, color) in row.iter().enumerate() {
            let pixel_color = match color {
                qrcode::Color::Dark => Rgba([0, 0, 0, 255]),
                qrcode::Color::Light => Rgba([255, 255, 255, 255]),
            };

            // Draw scaled pixel
            for dy in 0..scale {
                for dx in 0..scale {
                    img.put_pixel((x * scale + dx) as u32, (y * scale + dy) as u32, pixel_color);
                }
            }
        }
    }

    // Create iced image handle
    let pixels = img.into_raw();
    Ok(Handle::from_pixels(img_size as u32, img_size as u32, pixels))
}

/// Generate a QR code for an Ethereum payment request (EIP-681)
///
/// Format: ethereum:<address>[@<chain_id>][?value=<amount>]
#[cfg(feature = "qr")]
pub fn generate_payment_request_qr_code(
    address: &str,
    chain_id: Option<u64>,
    amount: Option<&str>,
) -> Result<Handle, String> {
    // Build EIP-681 URI
    let mut uri = format!("ethereum:{address}");

    if let Some(chain) = chain_id {
        uri.push_str(&format!("@{chain}"));
    }

    if let Some(amt) = amount {
        uri.push_str(&format!("?value={amt}"));
    }

    generate_address_qr_code(&uri)
}

#[cfg(all(test, feature = "qr"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_address_qr_code() {
        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let result = generate_address_qr_code(address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_payment_request_qr_code() {
        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let result = generate_payment_request_qr_code(address, Some(1), Some("1.5"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_address_generates_qr() {
        let invalid_address = "not_an_address";
        // QR code will still generate (it's just text), but we could add validation
        let result = generate_address_qr_code(invalid_address);
        assert!(result.is_ok()); // QR code generation doesn't validate address format
    }
}
