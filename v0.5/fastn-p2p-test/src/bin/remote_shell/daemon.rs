//! Minimal Remote Shell Daemon
//! 
//! Accept P2P connections and execute commands.
//! 
//! ⚠️  SECURITY WARNING: This allows full command execution from remote machines!
//! Only share your ID52 with completely trusted systems.
//! 
//! Usage: remote-shell-daemon [--private-key <key>]

use clap::Parser;
use fastn_p2p_test::{
    protocols::get_or_generate_key,
    remote_shell::{RemoteShellProtocol, execute_command},
};
use futures_util::StreamExt;

#[derive(Parser)]
#[command(about = "P2P Remote Shell Daemon")]
struct Args {
    /// Private key (optional, generates if not provided)
    #[arg(long)]
    private_key: Option<String>,
}

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    
    // Get or generate key
    let private_key = get_or_generate_key(args.private_key.as_deref())?;
    
    println!("⚠️  SECURITY WARNING: This daemon allows FULL COMMAND EXECUTION!");
    println!("🔑 Your ID52: {}", private_key.id52());
    println!("🚀 Connect with: remote-shell-client {} <command>", private_key.id52());
    println!("📡 Listening for P2P connections...");
    
    // Start listening - this is the core fastn-p2p usage!
    let protocols = [RemoteShellProtocol::Execute];
    let mut stream = fastn_p2p::listen!(private_key, &protocols);
    
    // Handle incoming requests
    while let Some(request) = stream.next().await {
        let request = request?;
        println!("📨 Command from {}", request.peer().id52());
        
        // Execute command using the convenient handle method
        let result = request.handle(execute_command).await;
        
        match result {
            Ok(()) => println!("✅ Command completed"),
            Err(e) => eprintln!("❌ Request failed: {}", e),
        }
    }
    
    Ok(())
}