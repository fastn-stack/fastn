//! fastn-p2p rshell - Remote shell client over P2P
//!
//! Usage: fastn-p2p-rshell [OPTIONS] [COMMAND]...
//!
//! Examples:
//!   fastn-p2p-rshell --target=<id52> ls -la
//!   fastn-p2p-rshell --private-key=<secret> --target=<id52> "echo hello"
//!   fastn-p2p-rshell (generates key and shows ID52)

use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "fastn-p2p-rshell")]
#[command(about = "Execute commands on remote machines over P2P")]
#[command(long_about = "
Remote shell client that executes commands on remote machines over secure P2P connections.
If no private key is provided, a new key will be generated and the ID52 will be displayed.
")]
struct Args {
    /// Target machine ID52 to connect to
    #[arg(help = "Target machine ID52 to connect to")]
    target: String,
    
    /// Private key for authentication (optional, generates if not provided)
    #[arg(long)]
    private_key: Option<String>,
    
    /// Command to execute on remote machine
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

/// Protocol for remote shell communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RemoteShellProtocol {
    Execute,
}

impl std::fmt::Display for RemoteShellProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteShellProtocol::Execute => write!(f, "remote-shell-execute"),
        }
    }
}

/// Request to execute a command on remote machine
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub command: Vec<String>,
}

/// Response from remote command execution
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Error responses for remote shell
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteShellError {
    CommandFailed {
        message: String,
        exit_code: Option<i32>,
    },
    ExecutionError {
        message: String,
    },
}

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    
    // Generate or parse private key
    let private_key = match args.private_key {
        Some(key_str) => {
            fastn_id52::SecretKey::from_str(&key_str)
                .map_err(|e| eyre::eyre!("Invalid private key: {}", e))?
        }
        None => {
            let key = fastn_id52::SecretKey::generate();
            println!("{}", "Generated new private key:".green().bold());
            println!("Private Key: {}", key.to_string().yellow());
            println!("Public ID52:  {}", key.id52().to_string().cyan().bold());
            println!();
            
            // Target is required, so we can proceed
            
            key
        }
    };
    
    // Parse target
    let target = fastn_id52::PublicKey::from_str(&args.target)
        .map_err(|e| eyre::eyre!("Invalid target ID52: {}", e))?;
    
    // Prepare command
    let command = if args.command.is_empty() {
        vec!["sh".to_string(), "-c".to_string(), "echo 'Hello from remote!'".to_string()]
    } else {
        args.command
    };
    
    println!("{}", "Connecting to remote machine...".blue());
    println!("Local ID52:  {}", private_key.id52().to_string().cyan());
    println!("Target ID52: {}", target.to_string().cyan());
    println!("Command:     {}", command.join(" ").yellow());
    println!();
    
    // Create execute request
    let request = ExecuteRequest { command };
    
    println!("{}", "🚀 Executing command over P2P...".green());
    
    // Make P2P call using fastn_p2p::client::call
    let result: Result<Result<ExecuteResponse, RemoteShellError>, fastn_p2p::client::CallError> = 
        fastn_p2p::client::call(
            private_key,
            target,
            RemoteShellProtocol::Execute,
            request
        ).await;
    
    match result {
        Ok(Ok(response)) => {
            // Success! Display output
            if !response.stdout.is_empty() {
                print!("{}", response.stdout);
            }
            if !response.stderr.is_empty() {
                eprint!("{}", response.stderr);
            }
            std::process::exit(response.exit_code);
        }
        Ok(Err(shell_error)) => {
            // Remote shell error
            match shell_error {
                RemoteShellError::CommandFailed { message, exit_code } => {
                    eprintln!("{}: {}", "Command failed".red(), message);
                    std::process::exit(exit_code.unwrap_or(1));
                }
                RemoteShellError::ExecutionError { message } => {
                    eprintln!("{}: {}", "Execution error".red(), message);
                    std::process::exit(1);
                }
            }
        }
        Err(call_error) => {
            // P2P communication error
            eprintln!("{}: {}", "P2P connection failed".red(), call_error);
            eprintln!();
            eprintln!("{}", "Possible causes:".yellow());
            eprintln!("• Target machine is not running remote access daemon");
            eprintln!("• Network connectivity issues");
            eprintln!("• Incorrect target ID52");
            eprintln!("• Firewall blocking P2P connections");
            std::process::exit(1);
        }
    }
}