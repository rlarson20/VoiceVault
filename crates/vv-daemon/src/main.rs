mod app;
mod cli;

use app::App;
use clap::Parser;
use cli::Args;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize tracing based on verbosity level
    let log_level = match args.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    // Build env filter: use user's level for VoiceVault, but suppress noisy external crates
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(format!("{},hf_hub=warn,indicatif=warn", log_level))
        });

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    info!("VoiceVault daemon starting");

    // Create and run application
    let app = App::new(args);

    if let Err(e) = app.run().await {
        error!("Application error: {:?}", e);
        std::process::exit(1);
    }
}
