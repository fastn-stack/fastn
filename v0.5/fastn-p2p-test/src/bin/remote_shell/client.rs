//! Remote Shell Client
//!
//! A high-quality reference implementation of a P2P remote shell client using fastn-p2p.
//! This client connects to remote daemons and executes commands over secure P2P connections.
//!
//! # Usage
//!
//! ```bash
//! # Execute a simple command
//! remote-shell-client <target-id52> echo "Hello World"
//!
//! # Execute with custom private key
//! remote-shell-client --private-key <secret> <target-id52> ls -la
//!
//! # Get help
//! remote-shell-client --help
//! ```
//!
//! # Examples
//!
//! ```bash
//! # System information
//! remote-shell-client 12D3K...xyz whoami
//! remote-shell-client 12D3K...xyz uname -a
//! remote-shell-client 12D3K...xyz df -h
//!
//! # File operations
//! remote-shell-client 12D3K...xyz ls -la /tmp
//! remote-shell-client 12D3K...xyz cat /etc/hostname
//!
//! # Process information
//! remote-shell-client 12D3K...xyz ps aux
//! remote-shell-client 12D3K...xyz top -b -n1
//! ```
//!
//! # Architecture
//!
//! The client uses fastn-p2p's high-level APIs:
//! - `fastn_p2p::client::call()` for request/response communication
//! - Strong typing for protocol safety and error handling
//! - Automatic serialization and network management
//!
//! # Error Handling
//!
//! The client provides comprehensive error handling with user-friendly messages:
//! - Network connectivity issues with troubleshooting suggestions
//! - Command execution failures with proper exit codes
//! - Protocol errors with debugging information
//! - Input validation with helpful guidance

use clap::Parser;
use colored::*;
use fastn_p2p_test::remote_shell::{
    ExecuteRequest, ExecuteResponse, RemoteShellError, RemoteShellProtocol,
};
use std::str::FromStr;
use tracing::{error, info, warn};

/// Command-line arguments for the remote shell client
#[derive(Parser)]
#[command(name = "remote-shell-client")]
#[command(about = "P2P Remote Shell Client - Execute commands on remote machines")]
#[command(long_about = "
Remote shell client that executes commands on remote machines over secure P2P connections.

The client connects to a remote daemon using its ID52 identifier and executes
the specified command, returning all output and the exit code. If no private
key is provided, a new one will be generated for this session.

Exit codes are preserved from the remote command execution, allowing for
proper integration with scripts and automation tools.
")]
struct Args {
    /// Target machine ID52 to connect to
    ///
    /// This is the public key identifier of the remote daemon you want to connect to.
    /// The daemon will display its ID52 when it starts up.
    #[arg(help = "Target machine ID52 (get this from the remote daemon output)")]
    target: String,

    /// Private key for authentication (optional, generates if not provided)
    ///
    /// If not provided, a new key will be generated for this session.
    /// Use the same key for consistent identity across multiple commands.
    #[arg(long, help = "Private key for authentication (generates new if not provided)")]
    private_key: Option<String>,

    /// Command to execute on remote machine
    ///
    /// All remaining arguments are treated as the command and its arguments.
    /// Use quotes for commands that need to be treated as a single argument.
    #[arg(trailing_var_arg = true, help = "Command and arguments to execute remotely")]
    command: Vec<String>,

    /// Enable verbose logging for debugging
    #[arg(short, long, help = "Enable verbose debug logging")]
    verbose: bool,

    /// Connection timeout in seconds
    #[arg(long, default_value = "30", help = "Connection timeout in seconds")]
    timeout: u64,
}

/// Main client entry point
///
/// This function handles argument parsing, connection establishment,
/// command execution, and result presentation with comprehensive
/// error handling and user feedback.
#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    // Initialize logging based on verbosity level
    setup_logging(args.verbose);

    // Generate or parse private key with validation
    let private_key = initialize_private_key(args.private_key)?;

    // Parse and validate target ID52
    let target = parse_target_id52(&args.target)?;

    // Prepare command with defaults
    let command = prepare_command(args.command);

    // Display connection information
    display_connection_info(&private_key, &target, &command);

    // Execute command with timeout
    let result = execute_remote_command(private_key, target, command, args.timeout).await;

    // Handle result and exit with appropriate code
    handle_execution_result(result)
}

/// Set up logging configuration based on verbosity level
fn setup_logging(verbose: bool) {
    let filter = if verbose {
        "debug,fastn_p2p=trace,fastn_net=trace"
    } else {
        "warn,fastn_p2p=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    if verbose {
        info!("Debug logging enabled");
    }
}

/// Initialize private key from argument or generate new one
fn initialize_private_key(private_key_arg: Option<String>) -> eyre::Result<fastn_id52::SecretKey> {
    match private_key_arg {
        Some(key_str) => {
            fastn_id52::SecretKey::from_str(&key_str).map_err(|e| {
                eyre::eyre!(
                    "Invalid private key format: {}. Expected base58-encoded key.",
                    e
                )
            })
        }
        None => {
            let key = fastn_id52::SecretKey::generate();
            if atty::is(atty::Stream::Stderr) {
                eprintln!("{}", "🔑 Generated temporary private key for this session".dimmed());
                eprintln!("   Your ID52: {}", key.id52().to_string().bright_black());
                eprintln!();
            }
            info!(local_id52 = %key.id52(), "Generated session private key");
            Ok(key)
        }
    }
}

/// Parse and validate target ID52
fn parse_target_id52(target_str: &str) -> eyre::Result<fastn_id52::PublicKey> {
    fastn_id52::PublicKey::from_str(target_str).map_err(|e| {
        eyre::eyre!(
            "Invalid target ID52 '{}': {}. Check that you copied the ID52 correctly from the daemon output.",
            target_str,
            e
        )
    })
}

/// Prepare command with sensible defaults
fn prepare_command(mut command: Vec<String>) -> Vec<String> {
    if command.is_empty() {
        // Default to a simple info command if none provided
        warn!("No command specified, using default 'whoami'");
        vec!["whoami".to_string()]
    } else {
        // Clean up command arguments
        command.retain(|arg| !arg.is_empty());
        command
    }
}

/// Display connection information for user feedback
fn display_connection_info(
    private_key: &fastn_id52::SecretKey,
    target: &fastn_id52::PublicKey,
    command: &[String],
) {
    if atty::is(atty::Stream::Stderr) {
        eprintln!("{}", "🚀 Connecting to remote machine...".blue());
        eprintln!("   Local ID52:  {}", private_key.id52().to_string().cyan());
        eprintln!("   Target ID52: {}", target.to_string().cyan());
        eprintln!("   Command:     {}", command.join(" ").yellow());
        eprintln!();
    }

    info!(
        local_id52 = %private_key.id52(),
        target_id52 = %target,
        command = ?command,
        "Initiating remote command execution"
    );
}

/// Execute remote command with timeout and error handling
async fn execute_remote_command(
    private_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey,
    command: Vec<String>,
    timeout_secs: u64,
) -> Result<ExecuteResponse, ClientError> {
    // Create execute request
    let request = ExecuteRequest { command };

    if atty::is(atty::Stream::Stderr) {
        eprintln!("{}", "📡 Executing command over P2P...".green());
    }

    // Execute with timeout
    let call_future = fastn_p2p::client::call::<_, _, ExecuteResponse, RemoteShellError>(
        private_key,
        target,
        RemoteShellProtocol::Execute,
        request,
    );

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        call_future,
    )
    .await;

    match result {
        Ok(call_result) => {
            info!("P2P call completed");
            match call_result {
                Ok(Ok(response)) => {
                    info!(
                        exit_code = response.exit_code,
                        stdout_len = response.stdout.len(),
                        stderr_len = response.stderr.len(),
                        "Command executed successfully"
                    );
                    Ok(response)
                }
                Ok(Err(shell_error)) => {
                    error!(error = ?shell_error, "Remote shell error");
                    Err(ClientError::RemoteShellError(shell_error))
                }
                Err(call_error) => {
                    error!(error = %call_error, "P2P call failed");
                    Err(ClientError::P2PError(call_error))
                }
            }
        }
        Err(_) => {
            error!(timeout_secs = timeout_secs, "Command execution timed out");
            Err(ClientError::Timeout(timeout_secs))
        }
    }
}

/// Handle execution result and exit with appropriate code
fn handle_execution_result(result: Result<ExecuteResponse, ClientError>) -> ! {
    match result {
        Ok(response) => {
            // Success! Display output exactly as received
            if !response.stdout.is_empty() {
                print!("{}", response.stdout);
            }
            if !response.stderr.is_empty() {
                eprint!("{}", response.stderr);
            }

            if atty::is(atty::Stream::Stderr) && response.exit_code != 0 {
                eprintln!(
                    "{}",
                    format!("Command exited with code {}", response.exit_code).bright_black()
                );
            }

            info!(exit_code = response.exit_code, "Client execution completed");
            std::process::exit(response.exit_code);
        }
        Err(error) => {
            handle_error(error);
        }
    }
}

/// Handle different types of client errors with appropriate user feedback
fn handle_error(error: ClientError) -> ! {
    match error {
        ClientError::RemoteShellError(shell_error) => {
            match shell_error {
                RemoteShellError::CommandFailed { message, exit_code } => {
                    eprintln!("{}: {}", "Remote command failed".red(), message);
                    let code = exit_code.unwrap_or(1);
                    error!(exit_code = code, error = %message, "Remote command failed");
                    std::process::exit(code);
                }
                RemoteShellError::ExecutionError { message } => {
                    eprintln!("{}: {}", "Remote execution error".red(), message);
                    eprintln!();
                    eprintln!("{}", "This usually indicates:".yellow());
                    eprintln!("• Command not found on remote system");
                    eprintln!("• Permission denied");
                    eprintln!("• System resource constraints");
                    error!(error = %message, "Remote execution error");
                    std::process::exit(1);
                }
            }
        }
        ClientError::P2PError(p2p_error) => {
            eprintln!("{}: {}", "P2P connection failed".red(), p2p_error);
            eprintln!();
            eprintln!("{}", "Possible causes:".yellow());
            eprintln!("• Target machine is not running the remote shell daemon");
            eprintln!("• Network connectivity issues or firewall blocking");
            eprintln!("• Incorrect target ID52 (check daemon output)");
            eprintln!("• Target machine is behind NAT without hole punching");
            eprintln!();
            eprintln!("{}", "Troubleshooting:".cyan());
            eprintln!("• Verify the daemon is running: check daemon output");
            eprintln!("• Test network connectivity between machines");
            eprintln!("• Ensure both machines can reach the internet");
            eprintln!("• Try from the same local network first");
            
            error!(error = %p2p_error, "P2P connection failed");
            std::process::exit(2);
        }
        ClientError::Timeout(timeout_secs) => {
            eprintln!(
                "{}: Command timed out after {} seconds",
                "Connection timeout".red(),
                timeout_secs
            );
            eprintln!();
            eprintln!("{}", "This might indicate:".yellow());
            eprintln!("• Slow network connection");
            eprintln!("• Long-running command on remote system");
            eprintln!("• Network partitioning or packet loss");
            eprintln!();
            eprintln!("{}", "Try:".cyan());
            eprintln!("• Increasing timeout with --timeout <seconds>");
            eprintln!("• Testing with a simpler command first");
            eprintln!("• Checking network connectivity");
            
            error!(timeout_secs = timeout_secs, "Command execution timed out");
            std::process::exit(3);
        }
    }
}

/// Client-specific error types for better error handling
#[derive(Debug)]
enum ClientError {
    RemoteShellError(RemoteShellError),
    P2PError(fastn_p2p::client::CallError),
    Timeout(u64),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::RemoteShellError(e) => write!(f, "Remote shell error: {}", e),
            ClientError::P2PError(e) => write!(f, "P2P error: {}", e),
            ClientError::Timeout(secs) => write!(f, "Timeout after {} seconds", secs),
        }
    }
}

impl std::error::Error for ClientError {}