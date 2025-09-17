//! Minimal P2P Receiver Example
//! 
//! Shows how to listen for P2P messages using fastn-p2p in just a few lines.
//! 
//! Usage: receiver [private_key]

use fastn_p2p_test::protocols::{EchoProtocol, EchoRequest, EchoResponse, EchoError, get_or_generate_key, init_logging};
use futures_util::StreamExt;

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    init_logging();
    
    // Get or generate key
    let args: Vec<String> = std::env::args().collect();
    let key_str = args.get(1).map(|s| s.as_str());
    let receiver_key = get_or_generate_key(key_str)?;
    
    println!("Listening on ID52: {}", receiver_key.id52());
    println!("Connect with: sender {} <message>", receiver_key.id52());
    
    // Start listening - this is the core fastn-p2p usage!
    let protocols = [EchoProtocol::Echo];
    let mut stream = fastn_p2p::listen!(receiver_key, &protocols);
    
    // Handle incoming messages
    while let Some(request) = stream.next().await {
        let request = request?;
        println!("📨 Message from {}", request.peer().id52());
        
        // Process request using the convenient handle method
        request.handle(|req: EchoRequest| async move {
            println!("   Content: {}", req.message);
            Ok::<EchoResponse, EchoError>(EchoResponse {
                echo: format!("Echo: {}", req.message),
            })
        }).await?;
        
        println!("✅ Response sent");
    }
    
    Ok(())
}