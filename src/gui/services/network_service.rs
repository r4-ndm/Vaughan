//! Network management service
//!
//! Contains functions for network configuration and management

use crate::network::{NetworkConfig, NetworkId};

/// Add a custom network to the wallet
pub async fn add_custom_network(
    name: String,
    rpc_url: String,
    chain_id: u64,
    symbol: String,
    block_explorer: String,
) -> Result<String, String> {
    use crate::network::validation::{
        create_network_config, validate_custom_network_config, validate_network_endpoint,
    };
    use crate::security::create_keychain_interface;
    use crate::security::keystore::SecureKeystoreImpl;

    tracing::info!("Adding custom network: {} (Chain ID: {})", name, chain_id);

    // Step 1: Validate form inputs
    let form_validation =
        validate_custom_network_config(&name, &rpc_url, &chain_id.to_string(), &symbol, &block_explorer);

    if !form_validation.is_valid {
        let error_messages: Vec<String> = form_validation
            .issues
            .iter()
            .map(|issue| match issue {
                crate::network::validation::NetworkValidationIssue::EmptyField(field) => {
                    format!("{field} is required")
                }
                crate::network::validation::NetworkValidationIssue::InvalidUrl => {
                    "Invalid RPC URL format. Please use a valid HTTP/HTTPS URL".to_string()
                }
                crate::network::validation::NetworkValidationIssue::InvalidChainId => {
                    "Invalid Chain ID. Must be a valid number".to_string()
                }
                crate::network::validation::NetworkValidationIssue::InvalidBlockExplorer => {
                    "Invalid Block Explorer URL format".to_string()
                }
                crate::network::validation::NetworkValidationIssue::InsecureConnection => {
                    "HTTPS is required for security. HTTP is only allowed for localhost (127.0.0.1, localhost, [::1])"
                        .to_string()
                }
                _ => "Unknown validation error".to_string(),
            })
            .collect();

        return Err(format!("Validation failed: {}", error_messages.join(", ")));
    }

    // Step 2: Test network connectivity and chain ID
    match validate_network_endpoint(&rpc_url, chain_id).await {
        Ok(endpoint_validation) => {
            if !endpoint_validation.is_valid {
                let connectivity_errors: Vec<String> = endpoint_validation
                    .issues
                    .iter()
                    .map(|issue| match issue {
                        crate::network::validation::NetworkValidationIssue::NotResponding => {
                            "RPC endpoint is not responding. Please check the URL".to_string()
                        }
                        crate::network::validation::NetworkValidationIssue::InvalidResponse => {
                            "RPC endpoint returned invalid response".to_string()
                        }
                        crate::network::validation::NetworkValidationIssue::ChainIdMismatch { expected, actual } => {
                            format!("Chain ID mismatch: expected {expected}, got {actual}")
                        }
                        crate::network::validation::NetworkValidationIssue::HighLatency(ms) => {
                            format!("High network latency: {ms}ms")
                        }
                        _ => "Network connectivity issue".to_string(),
                    })
                    .collect();

                return Err(format!("Network test failed: {}", connectivity_errors.join(", ")));
            }

            if !endpoint_validation.chain_id_matches {
                return Err(format!(
                    "Chain ID verification failed. The RPC endpoint reports a different chain ID than expected ({chain_id})"
                ));
            }
        }
        Err(e) => {
            return Err(format!("Failed to test network endpoint: {e}"));
        }
    }

    // Step 3: Determine if network is a testnet based on common testnet chain IDs
    let is_testnet = matches!(
        chain_id,
        3 |    // Ropsten
        4 |    // Rinkeby
        5 |    // Goerli
        42 |   // Kovan
        943 |  // PulseChain Testnet v4
        11155111 | // Sepolia
        80001 |    // Polygon Mumbai
        97 |       // BSC Testnet
        421613 |   // Arbitrum Rinkeby
        420 |      // Optimism Goerli
        1337 // Local development
    ) || name.to_lowercase().contains("test")
        || name.to_lowercase().contains("testnet");

    // Step 4: Create network configuration
    let network_config = create_network_config(
        name.clone(),
        rpc_url.clone(),
        chain_id,
        symbol.clone(),
        block_explorer.clone(),
        is_testnet,
    );

    // Step 5: Store in keystore
    match create_keychain_interface() {
        Ok(keychain) => match SecureKeystoreImpl::new(keychain).await {
            Ok(mut keystore) => match keystore.add_custom_network(network_config.clone()).await {
                Ok(()) => {
                    let security_status = if rpc_url.starts_with("https://") {
                        "secure (HTTPS)"
                    } else {
                        "insecure (HTTP)"
                    };

                    let network_type = if is_testnet { "testnet" } else { "mainnet" };

                    Ok(format!(
                        "Successfully added {network_type} network '{name}' (Chain ID: {chain_id}, {security_status})"
                    ))
                }
                Err(e) => Err(format!("Failed to save network to keystore: {e}")),
            },
            Err(e) => Err(format!("Failed to initialize keystore: {e}")),
        },
        Err(e) => Err(format!("Failed to create keychain interface: {e}")),
    }
}

/// Edit an existing custom network
pub async fn edit_existing_network(
    id: String,
    name: String,
    rpc_url: String,
    chain_id: u64,
    symbol: String,
) -> Result<String, String> {
    use crate::network::validation::{
        create_network_config, validate_custom_network_config, validate_network_endpoint,
    };
    use crate::security::create_keychain_interface;
    use crate::security::keystore::SecureKeystoreImpl;

    tracing::info!("Editing custom network: {} (Chain ID: {})", name, chain_id);

    // Parse the network ID
    let parsed_chain_id = id.parse::<u64>().map_err(|e| format!("Invalid network ID: {e}"))?;

    // Step 1: Validate form inputs
    let form_validation = validate_custom_network_config(
        &name,
        &rpc_url,
        &chain_id.to_string(),
        &symbol,
        "", // Block explorer not required for editing, can be empty
    );

    if !form_validation.is_valid {
        let error_messages: Vec<String> = form_validation
            .issues
            .iter()
            .map(|issue| match issue {
                crate::network::validation::NetworkValidationIssue::EmptyField(field) => {
                    format!("{field} is required")
                }
                crate::network::validation::NetworkValidationIssue::InvalidUrl => {
                    "Invalid RPC URL format. Please use a valid HTTP/HTTPS URL".to_string()
                }
                crate::network::validation::NetworkValidationIssue::InvalidChainId => {
                    "Invalid Chain ID. Must be a valid number".to_string()
                }
                crate::network::validation::NetworkValidationIssue::InvalidBlockExplorer => {
                    "Invalid Block Explorer URL format".to_string()
                }
                crate::network::validation::NetworkValidationIssue::InsecureConnection => {
                    "HTTPS is required for security. HTTP is only allowed for localhost (127.0.0.1, localhost, [::1])"
                        .to_string()
                }
                _ => "Unknown validation error".to_string(),
            })
            .collect();

        return Err(format!("Validation failed: {}", error_messages.join(", ")));
    }

    // Step 2: Test network connectivity and chain ID if different from current
    if chain_id != parsed_chain_id {
        match validate_network_endpoint(&rpc_url, chain_id).await {
            Ok(endpoint_validation) => {
                if !endpoint_validation.is_valid {
                    let connectivity_errors: Vec<String> = endpoint_validation
                        .issues
                        .iter()
                        .map(|issue| match issue {
                            crate::network::validation::NetworkValidationIssue::NotResponding => {
                                "RPC endpoint is not responding. Please check the URL".to_string()
                            }
                            crate::network::validation::NetworkValidationIssue::InvalidResponse => {
                                "RPC endpoint returned invalid response".to_string()
                            }
                            crate::network::validation::NetworkValidationIssue::ChainIdMismatch {
                                expected,
                                actual,
                            } => {
                                format!("Chain ID mismatch: expected {expected}, got {actual}")
                            }
                            crate::network::validation::NetworkValidationIssue::HighLatency(ms) => {
                                format!("High network latency: {ms}ms")
                            }
                            _ => "Network connectivity issue".to_string(),
                        })
                        .collect();

                    return Err(format!("Network test failed: {}", connectivity_errors.join(", ")));
                }

                if !endpoint_validation.chain_id_matches {
                    return Err(format!("Chain ID verification failed. The RPC endpoint reports a different chain ID than expected ({chain_id})"));
                }
            }
            Err(e) => {
                return Err(format!("Failed to test network endpoint: {e}"));
            }
        }
    }

    // Step 3: Determine if network is a testnet
    let is_testnet = matches!(
        chain_id,
        3 |    // Ropsten
        4 |    // Rinkeby
        5 |    // Goerli
        42 |   // Kovan
        943 |  // PulseChain Testnet v4
        11155111 | // Sepolia
        80001 |    // Polygon Mumbai
        97 |       // BSC Testnet
        421613 |   // Arbitrum Rinkeby
        420 |      // Optimism Goerli
        1337 // Local development
    ) || name.to_lowercase().contains("test")
        || name.to_lowercase().contains("testnet");

    // Step 4: Create updated network configuration
    let network_config = create_network_config(
        name.clone(),
        rpc_url.clone(),
        chain_id,
        symbol.clone(),
        String::new(), // Block explorer not required for editing
        is_testnet,
    );

    // Step 5: Update in keystore
    match create_keychain_interface() {
        Ok(keychain) => match SecureKeystoreImpl::new(keychain).await {
            Ok(mut keystore) => match keystore.update_custom_network(network_config.clone()).await {
                Ok(()) => {
                    let security_status = if rpc_url.starts_with("https://") {
                        "secure (HTTPS)"
                    } else {
                        "insecure (HTTP)"
                    };

                    let network_type = if is_testnet { "testnet" } else { "mainnet" };

                    Ok(format!(
                        "Successfully updated {network_type} network '{name}' (Chain ID: {chain_id}, {security_status})"
                    ))
                }
                Err(e) => Err(format!("Failed to update network in keystore: {e}")),
            },
            Err(e) => Err(format!("Failed to initialize keystore: {e}")),
        },
        Err(e) => Err(format!("Failed to create keychain interface: {e}")),
    }
}

/// Delete an existing custom network
pub async fn delete_existing_network(id: String) -> Result<String, String> {
    use crate::security::create_keychain_interface;
    use crate::security::keystore::SecureKeystoreImpl;

    tracing::info!("Deleting custom network: {}", id);

    // Parse the network ID
    let chain_id = id.parse::<u64>().map_err(|e| format!("Invalid network ID: {e}"))?;

    // Update keystore storage
    match create_keychain_interface() {
        Ok(keychain) => match SecureKeystoreImpl::new(keychain).await {
            Ok(mut keystore) => match keystore.remove_custom_network(NetworkId(chain_id)).await {
                Ok(()) => Ok(format!("Removed network with Chain ID {chain_id}")),
                Err(e) => Err(format!("Failed to remove network from keystore: {e}")),
            },
            Err(e) => Err(format!("Failed to initialize keystore: {e}")),
        },
        Err(e) => Err(format!("Failed to create keychain interface: {e}")),
    }
}

/// Load all available networks including custom ones from storage
pub async fn load_all_networks() -> Vec<NetworkConfig> {
    tracing::info!("Loading all networks including custom ones...");

    // Start with default networks
    let mut networks = vec![
        NetworkConfig {
            id: NetworkId(1),
            name: "Ethereum".to_string(),
            chain_id: 1,
            rpc_url: "https://ethereum.publicnode.com".to_string(),
            symbol: "ETH".to_string(),
            block_explorer_url: "https://etherscan.io".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(369),
            name: "PulseChain".to_string(),
            chain_id: 369,
            rpc_url: "https://rpc.pulsechain.com".to_string(),
            symbol: "PLS".to_string(),
            block_explorer_url: "https://scan.pulsechain.com".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(943),
            name: "PulseChain Testnet v4".to_string(),
            chain_id: 943,
            rpc_url: "https://rpc.v4.testnet.pulsechain.com".to_string(),
            symbol: "tPLS".to_string(),
            block_explorer_url: "https://scan.v4.testnet.pulsechain.com".to_string(),
            is_testnet: true,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(56),
            name: "BSC".to_string(),
            chain_id: 56,
            rpc_url: "https://bsc-dataseed1.binance.org".to_string(),
            symbol: "BNB".to_string(),
            block_explorer_url: "https://bscscan.com".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(137),
            name: "Polygon".to_string(),
            chain_id: 137,
            rpc_url: "https://polygon-rpc.com".to_string(),
            symbol: "MATIC".to_string(),
            block_explorer_url: "https://polygonscan.com".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(42161),
            name: "Arbitrum One".to_string(),
            chain_id: 42161,
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            symbol: "ETH".to_string(),
            block_explorer_url: "https://arbiscan.io".to_string(),
            is_testnet: false,
            is_custom: false,
        },
        NetworkConfig {
            id: NetworkId(10),
            name: "Optimism".to_string(),
            chain_id: 10,
            rpc_url: "https://mainnet.optimism.io".to_string(),
            symbol: "ETH".to_string(),
            block_explorer_url: "https://optimistic.etherscan.io".to_string(),
            is_testnet: false,
            is_custom: false,
        },
    ];

    // Load custom networks from storage using standardized path
    let mut networks_path = crate::security::keystore::storage::get_vaughan_dir();
    networks_path.push("custom_networks.json");

    if let Ok(content) = std::fs::read_to_string(&networks_path) {
        if let Ok(custom_networks) = serde_json::from_str::<Vec<NetworkConfig>>(&content) {
            tracing::info!("Loaded {} custom networks", custom_networks.len());
            networks.extend(custom_networks);
        }
    }

    tracing::info!("Total networks available: {}", networks.len());
    networks
}

/// Save networks to persistent storage
pub async fn save_networks_to_storage(networks: Vec<NetworkConfig>) -> Result<(), String> {
    use crate::security::keychain::OSKeychain;
    use crate::security::keystore::SecureKeystoreImpl;

    tracing::info!("Saving networks to storage...");

    // Create keychain and keystore
    match OSKeychain::new(crate::security::SERVICE_NAME_PRIVATE_KEYS.to_string()) {
        Ok(keychain) => {
            match SecureKeystoreImpl::new(Box::new(keychain)).await {
                Ok(mut keystore) => {
                    // Clear existing custom networks and add all custom ones from the list
                    let custom_networks: Vec<NetworkConfig> = networks.into_iter().filter(|n| n.is_custom).collect();

                    for network in custom_networks {
                        // Try to add or update the network
                        match keystore.add_custom_network(network.clone()).await {
                            Ok(()) => {
                                tracing::info!(
                                    "Saved custom network: {} (Chain ID: {})",
                                    network.name,
                                    network.chain_id
                                );
                            }
                            Err(e) => {
                                // If network already exists, try to update it
                                if e.to_string().contains("already exists") {
                                    match keystore.update_custom_network(network.clone()).await {
                                        Ok(()) => {
                                            tracing::info!(
                                                "Updated custom network: {} (Chain ID: {})",
                                                network.name,
                                                network.chain_id
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to update network {}: {}", network.name, e);
                                        }
                                    }
                                } else {
                                    tracing::error!("Failed to save network {}: {}", network.name, e);
                                }
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(format!("Failed to initialize keystore: {e}")),
            }
        }
        Err(e) => Err(format!("Failed to initialize keychain: {e}")),
    }
}
