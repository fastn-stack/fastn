//! Minimal fastn-p2p receiver test
//!
//! Tests the generic protocol system with meaningful protocol names

use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

/// Test protocol - meaningful names instead of Ping/Http!
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum TestProtocol {
    Echo,
}

impl std::fmt::Display for TestProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub from: String,
    pub to: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoResponse {
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug)]  
pub struct EchoError {
    pub error: String,
}

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
    println!("🔧 DEBUG: About to create protocols vec");
    let protocols = vec![TestProtocol::Echo];
    println!("🔧 DEBUG: About to call fastn_p2p::listen!");
    let mut stream = fastn_p2p::listen!(receiver_key, &protocols);
    println!("🔧 DEBUG: listen! returned successfully");

    println!("📡 fastn-p2p receiver listening on Echo protocol");
    println!("🎯 Waiting for connections...");
    
    println!("🔧 DEBUG: About to call stream.next().await");

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