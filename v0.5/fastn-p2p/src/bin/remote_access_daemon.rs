//! fastn-p2p remote-access-daemon - Remote shell server over P2P
//!
//! Usage: fastn-p2p-remote-access-daemon [OPTIONS]
//!
//! Examples:
//!   fastn-p2p-remote-access-daemon
//!   fastn-p2p-remote-access-daemon --private-key=<secret>

use clap::Parser;
use colored::*;
use std::str::FromStr;

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
    
    // TODO: Implement actual P2P server
    println!("{}", "🚧 TODO: Implement P2P streaming server".bright_red());
    println!("{}", "Listening for incoming connections...".blue());
    
    // Simulate server running
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("{}", "Received Ctrl+C".yellow());
        }
        _ = simulate_server() => {
            println!("{}", "Server simulation ended".yellow());
        }
    }
    
    println!("{}", "Remote access daemon shutting down".green());
    Ok(())
}

async fn simulate_server() {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("{}", "🔍 Waiting for connections... (not implemented yet)".dimmed());
        println!("{}", "Next steps:".green());
        println!("  1. Implement server::listen() with actual iroh streaming");
        println!("  2. Create remote shell protocol handler");
        println!("  3. Execute commands and stream output back");
        println!();
    }
}