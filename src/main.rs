use tracing::info;
use vaughan::gui::launcher;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Vaughan - Multi-EVM DeFi Wallet with Iced GUI");

    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check if user wants the simple wallet interface
    if args.len() > 1 && args[1] == "--simple" {
        info!("Launching simple wallet GUI interface");
        return launcher::launch_working_gui();
    }

    // Default: Launch the main wallet
    info!("Launching Main Wallet application");

    launcher::launch_working_gui()
}
