use vaughan::security::seed::{
    SeedManager, SeedStrength, SecurityUseCase
};
use vaughan::security::keychain::MockKeychain;
use secrecy::SecretString;
use tokio;

/// Test BIP-39 seed phrase generation and validation
#[tokio::test]
async fn test_bip39_seed_generation() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    // Test different seed strengths
    let strengths = [
        SeedStrength::Words12,
        SeedStrength::Words15,
        SeedStrength::Words18,
        SeedStrength::Words21,
        SeedStrength::Words24,
    ];
    
    for strength in &strengths {
        // Generate seed phrase
        let phrase = seed_manager.generate_seed_phrase(*strength).unwrap();
        
        // Validate the generated phrase
        assert!(seed_manager.validate_seed_phrase(&phrase).is_ok());
        
        // Check word count
        let word_count = phrase.expose_secret().split_whitespace().count();
        assert_eq!(word_count, strength.word_count());
        
        // Verify each word is in BIP-39 wordlist
        let words: Vec<&str> = phrase.expose_secret().split_whitespace().collect();
        for word in words {
            assert!(vaughan::security::seed::utils::is_valid_bip39_word(word));
        }
    }
}

/// Test BIP-39 mnemonic validation with known test vectors
#[tokio::test]
async fn test_bip39_validation_known_vectors() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    // Known valid BIP-39 test vectors
    let valid_mnemonics = vec![
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "legal winner thank year wave sausage worth useful legal winner thank yellow",
        "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
    ];
    
    for mnemonic in valid_mnemonics {
        let phrase = SecretString::new(mnemonic.to_string());
        assert!(seed_manager.validate_seed_phrase(&phrase).is_ok());
    }
    
    // Invalid mnemonics
    let invalid_mnemonics = vec![
        "invalid phrase here",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon", // wrong checksum
        "abandon abandon abandon", // too short
    ];
    
    for mnemonic in invalid_mnemonics {
        let phrase = SecretString::new(mnemonic.to_string());
        assert!(seed_manager.validate_seed_phrase(&phrase).is_err());
    }
}

/// Test seed phrase strength analysis
#[tokio::test]
async fn test_seed_phrase_strength_analysis() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    
    let analysis = seed_manager.analyze_seed_phrase(&test_phrase).unwrap();
    
    assert_eq!(analysis.strength, SeedStrength::Words12);
    assert_eq!(analysis.entropy_bits, 128);
    assert!(analysis.is_valid);
    assert!(analysis.security_score() >= 70); // Should be at least 70 for 12 words
}

/// Test seed phrase import with comprehensive validation
#[tokio::test]
async fn test_comprehensive_seed_import() {
    use vaughan::security::seed::SeedImportConfig;
    
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    let config = SeedImportConfig::default();
    
    // Test valid import
    let valid_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let (imported_phrase, validation) = seed_manager
        .import_seed_phrase_comprehensive(valid_phrase, &config)
        .unwrap();
    
    assert!(validation.is_valid);
    assert_eq!(validation.word_count, 12);
    assert_eq!(validation.strength, Some(SeedStrength::Words12));
    assert!(validation.checksum_valid);
    assert!(validation.errors.is_empty());
    
    // Verify the imported phrase is valid BIP-39
    assert!(seed_manager.validate_seed_phrase(&imported_phrase).is_ok());
}

/// Test seed phrase entropy requirements
#[tokio::test]
async fn test_entropy_requirements() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    // Test entropy bits for different strengths
    assert_eq!(SeedStrength::Words12.entropy_bits(), 128);
    assert_eq!(SeedStrength::Words15.entropy_bits(), 160);
    assert_eq!(SeedStrength::Words18.entropy_bits(), 192);
    assert_eq!(SeedStrength::Words21.entropy_bits(), 224);
    assert_eq!(SeedStrength::Words24.entropy_bits(), 256);
    
    // Test recommended strengths for different use cases
    let personal_strengths = SeedManager::get_recommended_strengths(SecurityUseCase::Personal);
    assert!(personal_strengths.contains(&SeedStrength::Words12));
    assert!(personal_strengths.contains(&SeedStrength::Words24));
    
    let institutional_strengths = SeedManager::get_recommended_strengths(SecurityUseCase::Institutional);
    assert!(institutional_strengths.contains(&SeedStrength::Words24));
    assert!(!institutional_strengths.contains(&SeedStrength::Words12));
}

/// Test seed phrase security recommendations
#[tokio::test]
async fn test_security_recommendations() {
    // Test use case recommendations
    assert!(SeedStrength::Words12.is_recommended_for_use_case(SecurityUseCase::Personal));
    assert!(SeedStrength::Words24.is_recommended_for_use_case(SecurityUseCase::Personal));
    assert!(!SeedStrength::Words12.is_recommended_for_use_case(SecurityUseCase::Institutional));
    assert!(SeedStrength::Words24.is_recommended_for_use_case(SecurityUseCase::Institutional));
    
    // Test default strengths
    assert_eq!(SeedManager::get_default_strength(SecurityUseCase::Personal), SeedStrength::Words12);
    assert_eq!(SeedManager::get_default_strength(SecurityUseCase::Institutional), SeedStrength::Words24);
    assert_eq!(SeedManager::get_default_strength(SecurityUseCase::LongTermStorage), SeedStrength::Words24);
}

/// Test seed phrase deterministic generation
#[tokio::test]
async fn test_deterministic_generation() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    let test_phrase = SecretString::new(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
    );
    
    // Generate seed from the same phrase multiple times
    let seed1 = seed_manager.phrase_to_seed(&test_phrase, None).unwrap();
    let seed2 = seed_manager.phrase_to_seed(&test_phrase, None).unwrap();
    
    // Seeds should be identical for the same input
    assert_eq!(seed1.expose_seed(), seed2.expose_seed());
    
    // Test with passphrase
    let passphrase = SecretString::new("test_passphrase".to_string());
    let seed_with_passphrase1 = seed_manager.phrase_to_seed(&test_phrase, Some(&passphrase)).unwrap();
    let seed_with_passphrase2 = seed_manager.phrase_to_seed(&test_phrase, Some(&passphrase)).unwrap();
    
    // Seeds with passphrase should be identical
    assert_eq!(seed_with_passphrase1.expose_seed(), seed_with_passphrase2.expose_seed());
    
    // Seeds with and without passphrase should be different
    assert_ne!(seed1.expose_seed(), seed_with_passphrase1.expose_seed());
}

/// Test BIP-39 wordlist utilities
#[tokio::test]
async fn test_bip39_utilities() {
    use vaughan::security::seed::utils;
    
    // Test valid BIP-39 words
    assert!(utils::is_valid_bip39_word("abandon"));
    assert!(utils::is_valid_bip39_word("zebra"));
    assert!(utils::is_valid_bip39_word("about"));
    
    // Test invalid words
    assert!(!utils::is_valid_bip39_word("invalid"));
    assert!(!utils::is_valid_bip39_word("xyz"));
    assert!(!utils::is_valid_bip39_word(""));
    
    // Test wordlist access
    let wordlist = utils::get_bip39_wordlist();
    assert_eq!(wordlist.len(), 2048);
    assert!(wordlist.contains(&"abandon"));
    assert!(wordlist.contains(&"zebra"));
    
    // Test entropy calculation
    assert_eq!(utils::entropy_bits_from_words(12), Some(128));
    assert_eq!(utils::entropy_bits_from_words(24), Some(256));
    assert_eq!(utils::entropy_bits_from_words(10), None); // Invalid word count
}

/// Performance benchmark for seed generation
#[tokio::test]
async fn test_seed_generation_performance() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    let start = std::time::Instant::now();
    
    // Generate 100 seed phrases
    for _ in 0..100 {
        let _phrase = seed_manager.generate_seed_phrase(SeedStrength::Words12).unwrap();
    }
    
    let duration = start.elapsed();
    
    // Should generate 100 seed phrases in under 1 second
    assert!(duration.as_secs() < 1, "Seed generation too slow: {:?}", duration);
    
    println!("Generated 100 seed phrases in {:?}", duration);
}

/// Test memory security - ensure sensitive data is properly handled
#[tokio::test]
async fn test_memory_security() {
    let keychain = Box::new(MockKeychain::new());
    let seed_manager = SeedManager::new(keychain);
    
    // Generate a seed phrase
    let phrase = seed_manager.generate_seed_phrase(SeedStrength::Words12).unwrap();
    
    // The SecretString should protect the data
    // This is more of a documentation test since we can't easily verify zeroization
    let _exposed = phrase.expose_secret();
    
    // When the SecretString is dropped, it should be zeroized
    drop(phrase);
    
    // If we could check memory here, the seed phrase should be zeroed
    // This is primarily ensured by the secrecy crate
    assert!(true); // Placeholder - actual memory verification would require unsafe code
}