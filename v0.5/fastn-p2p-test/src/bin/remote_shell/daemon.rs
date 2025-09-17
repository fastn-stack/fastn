//! Remote Shell Daemon
//!
//! A high-quality reference implementation of a P2P remote shell server using fastn-p2p.
//! This daemon allows remote clients to execute commands over secure P2P connections.
//!
//! # Usage
//!
//! ```bash
//! # Generate a new key and start the daemon
//! remote-shell-daemon
//!
//! # Use an existing private key
//! remote-shell-daemon --private-key <secret>
//!
//! # Specify custom port (for future use)
//! remote-shell-daemon --port 8080
//! ```
//!
//! # Security Warning
//!
//! ⚠️ **SECURITY WARNING** ⚠️
//!
//! This daemon allows FULL COMMAND EXECUTION from remote machines!
//! Only share your ID52 with machines you completely trust!
//!
//! In production environments, implement additional security measures:
//! - Command whitelisting/blacklisting
//! - User privilege separation
//! - Resource limits and sandboxing
//! - Authentication and authorization
//! - Audit logging and monitoring
//!
//! # Architecture
//!
//! The daemon uses fastn-p2p's high-level APIs:
//! - `fastn_p2p::listen()` for accepting incoming connections
//! - `Request::handle()` for processing commands with automatic serialization
//! - `fastn_p2p::spawn()` for concurrent request handling
//!
//! # Error Handling
//!
//! The daemon implements comprehensive error handling:
//! - Network errors are logged and handled gracefully
//! - Command execution errors are captured and returned to clients
//! - Malformed requests are rejected with appropriate error messages
//! - System signals (Ctrl+C) trigger graceful shutdown

use clap::Parser;
use colored::*;
use fastn_p2p_test::remote_shell::{execute_command, ExecuteRequest, RemoteShellProtocol};
use futures_util::StreamExt;
use std::str::FromStr;
use tracing::{error, info, warn};

/// Command-line arguments for the remote shell daemon
#[derive(Parser)]
#[command(name = "remote-shell-daemon")]
#[command(about = "P2P Remote Shell Daemon - Allow remote command execution")]
#[command(long_about = "
Remote access daemon that allows other machines to execute commands over secure P2P connections.

⚠️  SECURITY WARNING ⚠️
This daemon allows FULL COMMAND EXECUTION from remote machines.
Only share your ID52 with machines you completely trust!

The daemon uses fastn-p2p for secure peer-to-peer communication and provides
comprehensive logging and error handling for production deployments.
")]
struct Args {
    /// Private key for authentication (optional, generates if not provided)
    ///
    /// If not provided, a new cryptographically secure private key will be generated
    /// and displayed. Store this key securely for persistent identity across restarts.
    #[arg(long, help = "Private key for authentication (generates new if not provided)")]
    private_key: Option<String>,

    /// Port to listen on (for future use with additional protocols)
    ///
    /// Currently unused but reserved for future multi-protocol support.
    #[arg(long, default_value = "8080", help = "Port for future protocol extensions")]
    port: u16,

    /// Enable verbose logging for debugging
    #[arg(short, long, help = "Enable verbose debug logging")]
    verbose: bool,
}

/// Main daemon entry point
///
/// This function sets up the P2P listener, handles incoming connections,
/// and manages graceful shutdown. It demonstrates proper use of fastn-p2p
/// APIs and error handling patterns.
#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    // Initialize logging based on verbosity level
    setup_logging(args.verbose);

    // Generate or parse private key with proper error handling
    let private_key = match args.private_key {
        Some(key_str) => {
            fastn_id52::SecretKey::from_str(&key_str).map_err(|e| {
                eyre::eyre!("Invalid private key format: {}. Expected base58-encoded key.", e)
            })?
        }
        None => {
            let key = fastn_id52::SecretKey::generate();
            display_new_key_info(&key);
            key
        }
    };

    // Display comprehensive security warning
    display_security_warning();

    // Show connection information for users
    display_connection_info(&private_key, args.port);

    // Start the P2P listener with proper error handling
    let result = start_daemon(private_key).await;

    // Handle shutdown gracefully
    match result {
        Ok(()) => {
            println!("{}", "✅ Remote access daemon shut down cleanly".green());
        }
        Err(e) => {
            eprintln!("{}: {}", "❌ Daemon error".red(), e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Set up logging configuration based on verbosity level
fn setup_logging(verbose: bool) {
    let filter = if verbose {
        "debug,fastn_p2p=trace,fastn_net=trace"
    } else {
        "info,fastn_p2p=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    if verbose {
        info!("Debug logging enabled");
    }
}

/// Display information about a newly generated private key
fn display_new_key_info(key: &fastn_id52::SecretKey) {
    println!();
    println!("{}", "🔑 Generated new private key".green().bold());
    println!();
    println!("{}", "IMPORTANT: Save this private key securely!".yellow().bold());
    println!("Private Key: {}", key.to_string().cyan());
    println!("Public ID52:  {}", key.id52().to_string().bright_cyan().bold());
    println!();
    println!("{}", "💡 Tip: Use --private-key to reuse this identity".dimmed());
    println!();
}

/// Display comprehensive security warning
fn display_security_warning() {
    println!("{}", "⚠️  SECURITY WARNING ⚠️".red().bold());
    println!("{}", "┌─────────────────────────────────────────────────────────────┐".red());
    println!("{}", "│  This daemon allows FULL COMMAND EXECUTION from remote     │".red());
    println!("{}", "│  machines. Only share your ID52 with completely trusted    │".red());
    println!("{}", "│  systems. Consider implementing additional security        │".red());
    println!("{}", "│  measures for production deployments.                      │".red());
    println!("{}", "└─────────────────────────────────────────────────────────────┘".red());
    println!();
}

/// Display connection and usage information
fn display_connection_info(private_key: &fastn_id52::SecretKey, port: u16) {
    println!("{}", "🚀 Remote Access Daemon Starting...".green().bold());
    println!();
    println!("📋 Configuration:");
    println!("   Your ID52: {}", private_key.id52().to_string().cyan().bold());
    println!("   Port:      {}", port.to_string().yellow());
    println!();
    println!("{}", "📡 Client Connection Info:".bright_blue().bold());
    println!("   Command: {}", format!("remote-shell-client {} <command>", private_key.id52()).cyan());
    println!();
    println!("   Examples:");
    println!("   {}", format!("remote-shell-client {} echo 'Hello World'", private_key.id52()).dimmed());
    println!("   {}", format!("remote-shell-client {} ls -la", private_key.id52()).dimmed());
    println!("   {}", format!("remote-shell-client {} whoami", private_key.id52()).dimmed());
    println!();
}

/// Start the main daemon loop
///
/// This function encapsulates the core daemon logic:
/// 1. Initialize P2P listener
/// 2. Accept and handle incoming connections
/// 3. Manage graceful shutdown
async fn start_daemon(private_key: fastn_id52::SecretKey) -> eyre::Result<()> {
    // Define supported protocols
    let protocols = [RemoteShellProtocol::Execute];

    println!("{}", "🔧 Initializing P2P listener...".blue());

    // Start P2P listener with comprehensive error handling
    let stream = fastn_p2p::listen(private_key, &protocols).map_err(|e| {
        error!(error = %e, "Failed to start P2P listener");
        eyre::eyre!("Failed to start P2P listener: {}. Check network configuration and port availability.", e)
    })?;

    let mut stream = std::pin::pin!(stream);

    println!("{}", "✅ P2P listener ready!".green().bold());
    println!("{}", "🔍 Waiting for incoming connections...".blue());
    println!("{}", "Press Ctrl+C to stop".dimmed());
    println!();

    // Main event loop with graceful shutdown handling
    tokio::select! {
        // Handle Ctrl+C gracefully
        result = tokio::signal::ctrl_c() => {
            match result {
                Ok(()) => {
                    println!();
                    println!("{}", "📡 Received shutdown signal (Ctrl+C)".yellow());
                    info!("Graceful shutdown initiated by user");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to listen for Ctrl+C signal");
                }
            }
        }
        
        // Main request processing loop
        () = async {
            let mut connection_count = 0u64;
            
            while let Some(request_result) = stream.next().await {
                match request_result {
                    Ok(request) => {
                        connection_count += 1;
                        
                        let peer_id = request.peer().to_string();
                        println!(
                            "{} #{} from {}",
                            "📨 New connection".green(),
                            connection_count.to_string().cyan(),
                            peer_id.bright_cyan()
                        );
                        
                        info!(
                            connection_id = connection_count,
                            peer = %peer_id,
                            protocol = ?request.protocol(),
                            "Accepted new P2P connection"
                        );

                        // Spawn concurrent task to handle the request
                        // This allows the daemon to handle multiple requests simultaneously
                        fastn_p2p::spawn(handle_connection(request, connection_count));
                    }
                    Err(e) => {
                        // Log network errors but continue accepting connections
                        error!(error = %e, "Failed to accept P2P connection");
                        eprintln!("{}: {}", "⚠️  Connection error".yellow(), e);
                    }
                }
            }
            
            info!("P2P stream ended, shutting down");
            println!("{}", "📡 P2P stream ended".yellow());
        } => {}
    }

    info!("Daemon shutdown complete");
    Ok(())
}

/// Handle a single P2P connection
///
/// This function processes an individual remote shell request using fastn-p2p's
/// convenient `handle` method, which automatically manages serialization and
/// error handling.
///
/// # Arguments
///
/// * `request` - The incoming P2P request
/// * `connection_id` - Unique identifier for this connection (for logging)
async fn handle_connection(
    request: fastn_p2p::Request<RemoteShellProtocol>,
    connection_id: u64,
) -> eyre::Result<()> {
    let peer_id = request.peer().to_string();
    
    info!(
        connection_id = connection_id,
        peer = %peer_id,
        "Processing request"
    );

    match request.protocol() {
        RemoteShellProtocol::Execute => {
            println!(
                "{} #{} ({})",
                "🔧 Processing execute request".blue(),
                connection_id.to_string().cyan(),
                peer_id.bright_black()
            );

            // Use fastn-p2p's convenient handle method for automatic serialization
            let result = request
                .handle(|execute_request: ExecuteRequest| async move {
                    info!(
                        connection_id = connection_id,
                        command = ?execute_request.command,
                        "Executing command"
                    );

                    println!(
                        "📋 Command #{}: {}",
                        connection_id.to_string().cyan(),
                        execute_request.command.join(" ").yellow()
                    );

                    // Execute the command using our secure execution engine
                    execute_command(execute_request).await
                })
                .await;

            match result {
                Ok(()) => {
                    println!(
                        "{} #{} ✅",
                        "✅ Request completed".green(),
                        connection_id.to_string().cyan()
                    );
                    
                    info!(
                        connection_id = connection_id,
                        peer = %peer_id,
                        "Request completed successfully"
                    );
                }
                Err(e) => {
                    error!(
                        connection_id = connection_id,
                        peer = %peer_id,
                        error = %e,
                        "Request processing failed"
                    );
                    
                    eprintln!(
                        "{} #{}: {}",
                        "❌ Request failed".red(),
                        connection_id.to_string().cyan(),
                        e
                    );
                }
            }
        }
    }

    Ok(())
}