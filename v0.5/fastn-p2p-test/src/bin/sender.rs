//! Minimal P2P Sender Example
//! 
//! Shows how to send a simple message using fastn-p2p in just a few lines.
//! 
//! Usage: sender <target_id52> [message]

use fastn_p2p_test::protocols::{EchoProtocol, EchoRequest, EchoResponse, EchoError, parse_id52, get_or_generate_key, init_logging};

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    init_logging();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <target_id52> [message]", args[0]);
        std::process::exit(1);
    }
    
    // Parse arguments
    let target = parse_id52(&args[1])?;
    let message = args.get(2).map(|s| s.clone()).unwrap_or_else(|| "Hello P2P!".to_string());
    let sender_key = get_or_generate_key(None)?;
    
    println!("Sending '{}' to {}", message, target);
    
    // Send message - this is the core fastn-p2p usage!
    let request = EchoRequest { message };
    let result: Result<EchoResponse, EchoError> = fastn_p2p::client::call(
        sender_key,
        target,
        EchoProtocol::Echo,
        request,
    ).await?;
    
    // Handle response
    match result {
        Ok(response) => println!("✅ Response: {}", response.echo),
        Err(error) => eprintln!("❌ Error: {}", error.error),
    }
    
    Ok(())
}