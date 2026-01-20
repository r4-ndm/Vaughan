//! Wallet Message Types
//!
//! This module contains all message types and events for the wallet GUI.

use super::services::auto_balance_service::AutoBalanceMessage;
use super::wallet_types::{
    GasEstimation, GasSpeed, HistoryTab, ImportType, StatusMessageColor, TokenInfo, Transaction,
};
use crate::network::NetworkId;
use crate::security::SecureAccount;
use std::sync::Arc;

/// Main message enum for all wallet actions
#[derive(Debug, Clone)]
pub enum Message {
    RefreshBalance,
    BalanceFetched(Result<String, String>),
    // Smart polling messages
    SmartPollTick,
    BalanceChanged(String, String), // (old_balance, new_balance)
    UserActivity,
    // Internal refresh (doesn't show loading state)
    InternalRefreshBalance,
    NetworkSelected(NetworkId),
    ShowHistory,
    HideHistory,
    ShowTransactionHistory,
    HideTransactionHistory,
    HistoryTabSelected(HistoryTab),
    TransactionHistoryLoaded(Result<Vec<Transaction>, String>),
    ClearTransactionHistory,
    LogEntryCopied(Result<(), String>),
    ResetCopyFeedback,
    ClearLogs,
    ShowClearLogsConfirmation,
    HideClearLogsConfirmation,
    ConfirmClearLogs,
    CopyLogEntry(usize),            // Index of the log entry to copy
    CopyTransactionAddress(String), // Copy transaction address
    CopyTransactionHash(String),    // Copy transaction hash
    ShowSend,
    HideSend,
    SendAddressChanged(String),
    SendAmountChanged(String),
    SendGasLimitChanged(String),
    SendGasPriceChanged(String),
    // Advanced send options controls
    SendTxTypeChanged(String),         // "Legacy" or "EIP-1559"
    SendMaxFeeChanged(String),         // Gwei
    SendMaxPriorityFeeChanged(String), // Gwei
    SendNonceOverrideChanged(String),  // optional
    // Gas speed selection
    GasSpeedSelected(GasSpeed),
    ToggleAdvancedSendOptions,
    // Token selection messages for send dialog
    SendTokenSelected(String),
    SendCustomTokenAddressChanged(String),
    SendShowCustomTokenInput,
    SendPasteFromClipboard,
    SendPasteAddressFromClipboard, // Paste clipboard content to To Address field
    SendPasteAmountFromClipboard,  // Paste clipboard content to Amount field
    SendFromAccountSelected(String),
    // Balance token selection
    BalanceTokenSelected(String),     // Token selected for balance display
    BalanceTickerSelected(String),    // Ticker selected for balance display
    ShowBalanceAddToken,              // Show add token button for balance display
    TokenBalanceUpdateNeeded(String), // 🔧 FIX: Update token balance after custom token addition
    // Enhanced token management
    AddCustomToken(String),                      // Add custom token to available tokens list
    FetchTokenInfo(String),                      // Fetch token info from blockchain
    TokenInfoFetched(Result<TokenInfo, String>), // Result of token info fetch
    RemoveCustomToken(String),                   // Remove custom token from list
    HideCustomTokenInput,                        // Hide the custom token input dialog
    // Custom token screen navigation and management
    ShowCustomTokenScreen, // Show the dedicated custom token screen
    HideCustomTokenScreen, // Hide the custom token screen and return to main
    // Individual field changes for custom token creation
    CustomTokenAddressChanged(String),  // Contract address input
    CustomTokenNameChanged(String),     // Token name input
    CustomTokenSymbolChanged(String),   // Token symbol/ticker input
    CustomTokenDecimalsChanged(String), // Token decimals input
    // Manual token creation and auto-fetch
    CreateCustomTokenManually, // Create token with manually entered info
    AutoFetchTokenInfo,        // Fetch token info from contract address
    PasteTokenAddress,         // Paste address from clipboard
    // Custom tokens persistence
    LoadCustomTokens,                   // Load custom tokens from storage on startup
    CustomTokensLoaded(Vec<TokenInfo>), // Result of loading custom tokens
    SaveCustomTokens,                   // Save current custom tokens to storage

    // Security & Session Management
    ShowPasswordDialog {
        config: crate::gui::state::auth_state::PasswordDialogConfig,
    },
    HidePasswordDialog,
    PasswordInputChanged(secrecy::SecretString),
    NewPasswordInputChanged(secrecy::SecretString),
    ConfirmPasswordInputChanged(secrecy::SecretString),
    PasswordRememberChanged(bool), // Remember session checkbox
    SubmitPassword,
    ShowResetWalletConfirmation,
    HideResetWalletConfirmation,
    ConfirmResetWallet,
    WalletResetComplete,
    PasswordValidated(Result<secrecy::SecretString, crate::gui::state::auth_state::PasswordError>),
    SessionLocked,
    SessionUnlocked,

    // Master Password Dialog for HD Wallet Authentication
    ShowMasterPasswordDialog(String), // Show dialog for specific account name
    HideMasterPasswordDialog,
    HDWalletPasswordChanged(String), // HD wallet password input changed (separate from wallet creation)
    MasterPasswordSubmit,            // User pressed enter or unlock button
    MasterPasswordCancel,            // User cancelled password entry
    MasterPasswordValidated(Result<secrecy::SecretString, String>), // Password validation result

    // Wallet Creation/Import Password (separate from HD wallet authentication)
    MasterPasswordChanged(String), // Master password input for wallet creation/import
    ExtendSession,
    ManualLock,
    SessionTimeoutCheck, // Periodic check for session timeout

    // Startup Authentication
    SeedAccountsChecked(bool),
    StartupAuthenticationRequired,
    StartupAuthenticationComplete,

    // Gas estimation and confirmation flow
    EstimateGas,
    GasEstimated(Result<GasEstimation, String>),
    ShowTransactionConfirmation,
    HideTransactionConfirmation,
    ConfirmTransaction,
    // Original transaction messages
    SubmitTransaction,
    TransactionSubmitted(Result<(String, Option<crate::gui::state::transaction_state::PendingTransaction>), String>),
    ShowReceive,
    ShowReceiveDialog,
    HideReceiveDialog,
    CopyToClipboard(String),
    ShowSettings,
    ShowDapps,
    ShowDappsDialog,
    HideDappsDialog,
    ShowDappsComingSoon,
    HideDappsComingSoon,
    // Transaction management
    ShowTransactionSpeed,
    CancelLastTransaction,
    // Enhanced transaction cancellation messages
    TransactionSubmittedForTracking(crate::gui::state::transaction_state::PendingTransaction),
    TransactionCancelled(Result<String, String>), // Result<tx_hash, error>
    TransactionConfirmed(String, String),         // (tx_hash, confirmation_status)
    ShowCancelConfirmation,
    HideCancelConfirmation,
    ConfirmCancelTransaction,
    UpdatePendingTransactionStatus(Result<Vec<crate::gui::state::transaction_state::PendingTransaction>, String>),
    // Cancellation progress updates
    CancellationProgressUpdate(crate::gui::state::transaction_state::CancellationProgress),
    // Real-time transaction monitoring
    TransactionMonitoringTick,
    // Seed phrase management
    ShowCreateWallet,
    HideCreateWallet,
    ShowImportWallet,
    ShowImportWalletFromSeed,
    HideImportWallet,
    ShowExportWallet,
    HideExportWallet,
    ExportSeedPhrase,
    ExportPrivateKey,
    ExportAccountSelected(String),
    SeedPhraseExported(Result<String, String>),
    PrivateKeyExported(Result<String, String>),
    // Inline export flow messages
    StartInlineExport(bool), // true for seed phrase, false for private key
    ExportPasswordChanged(String),
    SubmitInlineExport,
    CancelInlineExport,
    SeedPhraseChanged(String),
    PrivateKeyChanged(String),
    // Export copy functionality with clipboard security
    CopyExportedData(String),             // Copy seed phrase or private key to clipboard
    ExportDataCopied(Result<(), String>), // Result of export data copy
    ResetExportCopyFeedback,              // Clear export copy feedback
    ClearClipboardAfterDelay,             // Clear clipboard after 30 seconds for security

    ImportTypeSelected(ImportType),
    WalletNameChanged(String),
    ConfirmPasswordChanged(String),
    // Wallet password messages (separate from account passwords)
    WalletPasswordChanged(String),
    WalletPasswordSubmitted,
    WalletPasswordCancelled,
    WalletNewPasswordChanged(String),
    WalletPasswordConfirmChanged(String),
    WalletPasswordChangeSubmitted,
    WalletRememberSessionToggled(bool),
    ShowWalletUnlock,
    GenerateNewSeed,
    SeedGenerated(String),
    CreateWalletFromSeed,
    ImportWalletFromSeed,
    ImportWalletFromSeedDirect, // Import first address directly without discovery
    ImportWalletFromPrivateKey,
    // Multi-address management
    ShowAddressDiscovery,
    HideAddressDiscovery,
    DiscoverAddresses,
    AddressesDiscovered(Result<Vec<(String, String, bool)>, String>), // (address, derivation_path, has_activity)
    SelectAddressForImport(String, String),                           // (address, derivation_path)
    ImportSelectedAddresses,
    WalletCreated(Result<String, String>),
    // Seed strength selection
    SeedStrengthSelected(crate::security::SeedStrength),
    SeedAnalyzed(Result<crate::security::seed::SeedAnalysis, String>),
    // Custom network management
    ShowAddNetwork,
    HideAddNetwork,
    ShowHttpWarningDialog,
    AcceptHttpRisk,
    CancelHttpRisk,
    NetworkNameChanged(String),
    NetworkRpcUrlChanged(String),
    NetworkBlockExplorerChanged(String),
    NetworkChainIdChanged(String),
    NetworkSymbolChanged(String),
    AddCustomNetwork,
    NetworkAdded(Result<String, String>),
    // Network editing
    EditModeToggled(bool),
    ExistingNetworkSelected(Option<String>),
    LoadExistingNetwork,
    EditNetwork,
    NetworkUpdated(Result<String, String>),
    // Network deletion
    ShowDeleteNetworkConfirm,
    HideDeleteNetworkConfirm,
    ConfirmDeleteNetwork,
    NetworkDeleted(Result<String, String>),
    // Hardware wallet management
    ShowHardWallet,
    HideHardWallet,
    DetectHardwareWallets,
    HardwareWalletsDetected(Result<Vec<crate::security::hardware::HardwareWalletInfo>, String>),
    ConnectHardwareWallet(usize),
    HardwareWalletConnected(Result<String, String>),
    GetHardwareAddresses(usize),
    HardwareAddressesReceived(Result<Vec<alloy::primitives::Address>, String>),
    // Account management
    AccountSelected(String),
    AccountUnlocked(String),     // Account name
    AccountUnlockFailed(String), // Error message
    // Account password authentication (two-tier security)
    ShowAccountPasswordDialog(String), // Account ID
    HideAccountPasswordDialog,
    AccountPasswordChanged(String),
    SubmitAccountPassword,
    AccountPasswordSubmitted(String, String), // Account ID, Password
    AccountPasswordCancelled,
    AccountSessionUnlocked(String), // Account ID
    AccountSessionLocked(String),   // Account ID
    ShowDeleteAccount,
    HideDeleteAccount,
    ConfirmDeleteAccount,
    AccountDeleted(Result<String, String>),
    CopyAddress(String),
    LoadAccounts,
    AccountsLoaded(Result<Vec<SecureAccount>, String>),
    ResetCopyState,
    LoadNetworks,
    NetworksLoaded(Vec<crate::network::NetworkConfig>),
    // Wallet initialization
    WalletInitialized(Result<Arc<tokio::sync::RwLock<crate::wallet::Vaughan>>, String>),
    // Status message management
    SetStatusMessage(String, StatusMessageColor),
    ClearStatusMessage,
    StatusMessageTick,
    // Enhanced error handling and retry functionality
    RetryAccountLoading,
    RetryExportOperation,
    // Spinner animation
    SpinnerTick,
    // Theme management
    ThemeToggled,
    // Incoming transaction monitoring
    IncomingTransactionsChecked(Result<Vec<Transaction>, String>),
    // Manual test for incoming transactions (debugging)
    TestIncomingTransactions,
    // Price fetching messages
    ShowPriceInfo,
    HidePriceInfo,
    RefreshEthPrice,
    EthPriceFetched(Result<(f64, Option<String>), String>), // (price, 24h_change)
    PriceAutoRefreshTick,
    // Export data handling
    ExportDataReceived(Result<String, String>),
    PerformWalletExport(secrecy::SecretString),

    // Hardware wallet integration
    ScanHardwareWallets,
    RefreshHardwareWallets,
    ConnectToHardwareWallet(String), // device_id
    TestHardwareWallet,
    HardwareWalletsScanned(Result<Vec<crate::security::HardwareWalletInfo>, String>),

    // Dialog management
    ShowCreateDialog,
    HideCreateDialog,
    ShowImportDialog,
    HideImportDialog,
    ShowSettingsDialog,
    HideSettingsDialog,

    // Form field changes
    SendToAddressChanged(String),
    SendTokenChanged(String),
    CreateAccountNameChanged(String),
    ImportPrivateKeyChanged(String),
    ImportAccountNameChanged(String),

    // Account operations
    CreateAccount,
    AccountCreated(Result<String, String>),
    ImportAccount,
    AccountImported(Result<String, String>),
    DeleteAccount(String),

    // Balance and transaction management
    BalanceRefreshed(Result<String, String>),
    TokenBalancesRefreshed(Vec<(String, String)>), // Vec<(token_symbol, balance)>
    UpdateAccountBalance,
    RefreshTransactionHistory,
    TransactionHistoryRefreshed(Result<Vec<Transaction>, String>),

    // Activity tracking
    UpdateLastActivity,

    // Export account selection
    ToggleAccountDropdown,
    SelectExportAccount(String),

    // Export navigation
    BackToExportOptions,

    // Auto balance monitoring
    AutoBalanceUpdate(AutoBalanceMessage),
    StartAutoBalanceMonitoring,
    StopAutoBalanceMonitoring,
}
