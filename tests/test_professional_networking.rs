use vaughan::network::professional::ProfessionalNetworkManager;
use vaughan::network::NetworkId;

#[tokio::test]
async fn test_professional_network_manager_basic_functionality() {
    // This test verifies that the professional network manager can be initialized
    // and basic operations work

    println!("🧪 Testing Professional Network Manager initialization...");

    match ProfessionalNetworkManager::new().await {
        Ok(manager) => {
            println!("✅ Professional Network Manager initialized successfully");

            // Test current network
            let current = manager.current_network().await;
            println!("📊 Current network: {}", current.0);
            assert_eq!(current, NetworkId(1), "Should default to Ethereum (chain ID 1)");

            // Test network switching
            let switch_result = manager.switch_network(NetworkId(137)).await;
            match switch_result {
                Ok(_) => {
                    println!("✅ Successfully switched to Polygon (137)");
                    let new_current = manager.current_network().await;
                    assert_eq!(new_current, NetworkId(137), "Should be Polygon now");
                }
                Err(e) => {
                    println!("❌ Failed to switch networks: {e}");
                    panic!("Network switch failed");
                }
            }

            // Test network statistics
            let stats = manager.get_network_statistics().await;
            println!("📊 Network statistics:");
            println!("  - Total requests: {}", stats.total_requests);
            println!("  - Successful requests: {}", stats.successful_requests);
            println!("  - Failed requests: {}", stats.failed_requests);

            println!("✅ All basic functionality tests passed!");
        }
        Err(e) => {
            println!("❌ Failed to initialize Professional Network Manager: {e}");
            panic!("Initialization failed: {e}");
        }
    }
}

#[tokio::test]
async fn test_network_endpoint_selection() {
    println!("🧪 Testing network endpoint selection...");

    let manager = ProfessionalNetworkManager::new()
        .await
        .expect("Failed to initialize manager");

    // Test getting best endpoint for different networks
    let networks = [
        (NetworkId(1), "Ethereum"),
        (NetworkId(137), "Polygon"),
        (NetworkId(56), "BSC"),
    ];

    for (network_id, name) in networks.iter() {
        match manager.get_best_endpoint(*network_id).await {
            Ok(endpoint) => {
                println!("✅ {name} best endpoint: {endpoint}");
                assert!(!endpoint.is_empty(), "Endpoint should not be empty");
                assert!(endpoint.starts_with("https://"), "Should be HTTPS URL");
            }
            Err(e) => {
                println!("❌ Failed to get endpoint for {name}: {e}");
                panic!("Endpoint selection failed");
            }
        }
    }

    println!("✅ Endpoint selection tests passed!");
}

#[tokio::test]
async fn test_unsupported_network_handling() {
    println!("🧪 Testing unsupported network handling...");

    let manager = ProfessionalNetworkManager::new()
        .await
        .expect("Failed to initialize manager");

    // Test switching to an unsupported network
    let unsupported_network = NetworkId(99999);
    let switch_result = manager.switch_network(unsupported_network).await;

    match switch_result {
        Ok(_) => {
            panic!("Should have failed for unsupported network");
        }
        Err(e) => {
            println!("✅ Correctly rejected unsupported network: {e}");
        }
    }

    // Test getting endpoint for unsupported network
    let endpoint_result = manager.get_best_endpoint(unsupported_network).await;

    match endpoint_result {
        Ok(_) => {
            panic!("Should have failed for unsupported network");
        }
        Err(e) => {
            println!("✅ Correctly rejected unsupported network endpoint request: {e}");
        }
    }

    println!("✅ Unsupported network handling tests passed!");
}
