//! # fastn-mail CLI Binary
//!
//! Command-line interface for testing fastn email functionality.

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    let cli = fastn_mail::cli::Cli::parse();

    if let Err(e) = fastn_mail::cli::run_command(cli).await {
        eprintln!("❌ Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Build with --features cli");
    std::process::exit(1);
}
