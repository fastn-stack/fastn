//! fastn-p2p remote-access-daemon - Remote shell server over P2P
//!
//! Usage: fastn-p2p-remote-access-daemon [OPTIONS]
//!
//! Examples:
//!   fastn-p2p-remote-access-daemon
//!   fastn-p2p-remote-access-daemon --private-key=<secret>

use clap::Parser;
use colored::*;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio::process::Command;

#[derive(Parser)]
#[command(name = "fastn-p2p-remote-access-daemon")]
#[command(about = "Allow remote command execution over P2P")]
#[command(long_about = "
Remote access daemon that allows other machines to execute commands over secure P2P connections.

⚠️  SECURITY WARNING ⚠️
This daemon allows FULL COMMAND EXECUTION from remote machines.
Only share your ID52 with machines you completely trust!
")]
struct Args {
    /// Private key for authentication (optional, generates if not provided)
    #[arg(long)]
    private_key: Option<String>,
    
    /// Port to listen on (optional, for future use)
    #[arg(long, default_value = "8080")]
    port: u16,
}

/// Protocol for remote shell communication (shared with rshell client)
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
            key
        }
    };
    
    // Display security warning
    println!();
    println!("{}", "⚠️  SECURITY WARNING ⚠️".red().bold());
    println!("{}", "This daemon allows FULL COMMAND EXECUTION from remote machines!".red());
    println!("{}", "Only share your ID52 with machines you completely trust!".red());
    println!();
    
    // Display connection info
    println!("{}", "Remote Access Daemon Starting...".green().bold());
    println!("Your ID52: {}", private_key.id52().to_string().cyan().bold());
    println!("Port:      {}", args.port.to_string().yellow());
    println!();
    println!("{}", "Clients can connect using:".bright_blue());
    println!("  fastn-p2p-rshell {} <command>", private_key.id52());
    println!();
    
    // Start P2P listener
    println!("{}", "🚀 Starting P2P listener...".green());
    let protocols = [RemoteShellProtocol::Execute];
    
    let stream = fastn_p2p::listen(private_key, &protocols)
        .map_err(|e| eyre::eyre!("Failed to start listener: {}", e))?;
    let mut stream = std::pin::pin!(stream);
    
    println!("{}", "✅ Remote access daemon ready!".green().bold());
    println!("{}", "Listening for incoming connections...".blue());
    
    // Handle incoming requests
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("{}", "Received Ctrl+C".yellow());
        }
        _ = async {
            while let Some(request_result) = stream.next().await {
                match request_result {
                    Ok(request) => {
                        println!("{} from {}", 
                                "📨 Received request".green(), 
                                request.peer().to_string().cyan());
                        
                        // Spawn task to handle request
                        fastn_p2p::spawn(handle_request(request));
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Request handling error".red(), e);
                    }
                }
            }
        } => {
            println!("{}", "Request stream ended".yellow());
        }
    }
    
    println!("{}", "Remote access daemon shutting down".green());
    Ok(())
}

async fn handle_request(request: fastn_p2p::Request<RemoteShellProtocol>) -> eyre::Result<()> {
    match request.protocol() {
        RemoteShellProtocol::Execute => {
            println!("{}", "🔧 Handling execute request".blue());
            
            // Use the convenient handle method that manages everything automatically
            request.handle(|execute_request: ExecuteRequest| async move {
                println!("📋 Command: {}", execute_request.command.join(" ").yellow());
                
                // Execute the command and return the result
                execute_command(execute_request.command).await
            }).await
            .map_err(|e| eyre::eyre!("Failed to handle request: {}", e))?;
        }
    }
    
    Ok(())
}

async fn execute_command(command: Vec<String>) -> Result<ExecuteResponse, RemoteShellError> {
    if command.is_empty() {
        return Err(RemoteShellError::ExecutionError {
            message: "Empty command".to_string(),
        });
    }
    
    let program = &command[0];
    let args = &command[1..];
    
    println!("🚀 Executing: {} {}", program, args.join(" "));
    
    // Execute the command
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| RemoteShellError::ExecutionError {
            message: format!("Failed to execute command: {}", e),
        })?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    
    // Log output for daemon operator
    if !stdout.is_empty() {
        println!("📤 stdout: {}", stdout.trim());
    }
    if !stderr.is_empty() {
        println!("📤 stderr: {}", stderr.trim());
    }
    
    println!("✅ Command completed with exit code: {}", exit_code);
    
    Ok(ExecuteResponse {
        exit_code,
        stdout,
        stderr,
    })
}