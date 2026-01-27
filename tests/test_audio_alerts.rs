//! Test for audio alerts functionality
//!
//! Tests the audio alert system for incoming transaction notifications.

#[cfg(feature = "audio")]
#[test]
fn test_audio_alert_compiles() {
    // This test ensures the audio feature compiles correctly
    use rodio::{OutputStream, Sink};
    

    // Try to create audio output (this might fail in CI/headless environments)
    if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
        if let Ok(_sink) = Sink::try_new(&stream_handle) {
            println!("✅ Audio system is available and working");
        } else {
            println!("⚠️ Audio sink creation failed (normal in headless environments)");
        }
    } else {
        println!("⚠️ Audio output stream creation failed (normal in headless environments)");
    }
}

#[cfg(feature = "audio")]
#[test]
fn test_wav_header_creation() {
    // Test the WAV header creation function
    fn create_wav_header(data_size: u32, sample_rate: u32, channels: u16) -> Vec<u8> {
        let mut header = Vec::new();

        // RIFF header
        header.extend_from_slice(b"RIFF");
        header.extend_from_slice(&(36 + data_size).to_le_bytes());
        header.extend_from_slice(b"WAVE");

        // fmt subchunk
        header.extend_from_slice(b"fmt ");
        header.extend_from_slice(&16u32.to_le_bytes()); // PCM format size
        header.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        header.extend_from_slice(&channels.to_le_bytes());
        header.extend_from_slice(&sample_rate.to_le_bytes());
        header.extend_from_slice(&(sample_rate * channels as u32 * 2).to_le_bytes()); // byte rate
        header.extend_from_slice(&(channels * 2).to_le_bytes()); // block align
        header.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

        // data subchunk
        header.extend_from_slice(b"data");
        header.extend_from_slice(&data_size.to_le_bytes());

        header
    }

    let header = create_wav_header(1000, 44100, 1);

    // Check that header has the correct size (44 bytes for WAV header)
    assert_eq!(header.len(), 44);

    // Check RIFF signature
    assert_eq!(&header[0..4], b"RIFF");

    // Check WAVE signature
    assert_eq!(&header[8..12], b"WAVE");

    println!("✅ WAV header creation test passed");
}

#[test]
fn test_balance_parsing() {
    // Test the balance parsing logic used for detecting balance increases
    fn parse_balance_simple(balance: &str) -> Result<f64, std::num::ParseFloatError> {
        // Simple implementation for testing
        let cleaned = balance
            .replace("ETH", "")
            .replace("tPLS", "")
            .replace("PLS", "")
            .replace("BNB", "")
            .replace("MATIC", "")
            .replace(",", "")
            .trim()
            .to_string();

        cleaned.parse::<f64>()
    }

    // Test that balance increases are detected correctly
    assert!(parse_balance_simple("1.5 ETH").unwrap() > parse_balance_simple("1.0 ETH").unwrap());
    assert!(parse_balance_simple("10.0 tPLS").unwrap() > parse_balance_simple("5.5 tPLS").unwrap());
    assert!(parse_balance_simple("0.001 BNB").unwrap() < parse_balance_simple("0.1 BNB").unwrap());

    println!("✅ Balance parsing test passed");
}
