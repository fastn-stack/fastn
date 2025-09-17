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
    
    // TODO: Implement actual P2P streaming connection
    // For now, just show what we would do
    println!("{}", "🚧 TODO: Implement P2P streaming connection".bright_red());
    println!("Would execute: {}", command.join(" "));
    
    // Simulate connection attempt
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    println!("{}", "✗ Not implemented yet - this is a prototype".red());
    println!("{}", "Next steps:".green());
    println!("  1. Implement client::connect() with actual iroh streaming");
    println!("  2. Create remote shell protocol");
    println!("  3. Wire up stdin/stdout/stderr streams");
    
    std::process::exit(1);
}