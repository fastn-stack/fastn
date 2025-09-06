//! Minimal fastn-p2p receiver test
//!
//! Tests the generic protocol system with meaningful protocol names

use futures_util::stream::StreamExt;
use fastn_p2p_test::{TestProtocol, EchoRequest, EchoResponse, EchoError};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("fastn_p2p=trace,fastn_p2p_test=info")
        .init();

    // Get secret key from command line args
    let args: Vec<String> = std::env::args().collect();
    let receiver_key = if args.len() > 1 {
        let secret_key_str = &args[1];
        match secret_key_str.parse::<fastn_id52::SecretKey>() {
            Ok(key) => {
                println!("🔑 Using provided secret key");
                key
            }
            Err(e) => {
                eprintln!("❌ Invalid secret key provided: {}", e);
                return Err(eyre::eyre!("Invalid secret key: {}", e));
            }
        }
    } else {
        println!("🔑 Generating new receiver key");
        fastn_id52::SecretKey::generate()
    };

    let receiver_id52 = receiver_key.public_key().id52();
    println!("🔑 Receiver ID52: {}", receiver_id52);

    // Output JSON for easy parsing in tests  
    let startup_info = serde_json::json!({
        "status": "started",
        "receiver_id52": receiver_id52,
        "secret_key": receiver_key.to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    println!(
        "📋 STARTUP: {}",
        serde_json::to_string(&startup_info).unwrap_or_default()
    );

    // Start listening using fastn-p2p
    let protocols = vec![TestProtocol::Echo];
    let mut stream = fastn_p2p::listen!(receiver_key, &protocols);

    println!("📡 fastn-p2p receiver listening on Echo protocol");
    println!("🎯 Waiting for connections...");

    // Handle exactly one connection (like fastn-net-test)
    if let Some(request_result) = stream.next().await {
        let request = request_result?;
        
        println!("🔗 Accepted connection from {}", request.peer().id52());
        println!("📨 Received {} protocol request", request.protocol);
        
        // Handle the echo request
        let result = request.handle(|req: EchoRequest| async move {
            println!("📨 Received message: {}", req.message);
            
            let response = EchoResponse {
                response: format!("Echo: {}", req.message),
            };
            
            Result::<EchoResponse, EchoError>::Ok(response)
        }).await;
        
        match result {
            Ok(_) => println!("✅ Request handled successfully"),
            Err(e) => eprintln!("❌ Request handling failed: {}", e),
        }
    }

    println!("🎯 fastn-p2p receiver test completed");
    Ok(())
}