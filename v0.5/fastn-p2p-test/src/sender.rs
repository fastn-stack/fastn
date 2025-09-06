//! Minimal fastn-p2p sender test
//!
//! Tests the generic protocol system by sending meaningful protocol requests

use fastn_p2p_test::{TestProtocol, EchoRequest, EchoResponse, EchoError};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("fastn_p2p=trace,fastn_p2p_test=info")
        .init();

    // Parse command line arguments: sender <sender_secret_key> <receiver_id52>
    let args: Vec<String> = std::env::args().collect();

    let (sender_key, receiver_id52) = if args.len() >= 3 {
        let sender_secret_str = &args[1];
        let receiver_id52 = args[2].clone();

        let sender_key = match sender_secret_str.parse::<fastn_id52::SecretKey>() {
            Ok(key) => key,
            Err(e) => {
                eprintln!("❌ Invalid sender secret key: {}", e);
                std::process::exit(1);
            }
        };

        (sender_key, receiver_id52)
    } else {
        eprintln!("❌ Usage: sender <sender_secret_key> <receiver_id52>");
        std::process::exit(1);
    };

    let sender_id52 = sender_key.public_key().id52();
    println!("🔑 Sender ID52: {}", sender_id52);
    println!("🎯 Target ID52: {}", receiver_id52);

    // Convert receiver ID52 to public key
    let receiver_public_key = receiver_id52.parse::<fastn_id52::PublicKey>()
        .map_err(|e| eyre::eyre!("Invalid receiver_id52: {}", e))?;

    println!("📤 Sending test message via fastn-p2p");

    // Create test request
    let request = EchoRequest {
        from: sender_id52,
        to: receiver_id52,
        message: "Hello from fastn-p2p test!".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Send using fastn-p2p call with meaningful protocol name
    let result: Result<EchoResponse, EchoError> = fastn_p2p::call(
        sender_key,
        &receiver_public_key,
        TestProtocol::Echo,
        request,
    ).await?;

    match result {
        Ok(response) => {
            println!("✅ Received response: {}", response.response);
            
            // Output JSON result for test parsing
            let result_json = serde_json::json!({
                "status": "success",
                "response": response.response,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            println!("📋 RESULT: {}", serde_json::to_string(&result_json)?);
        }
        Err(error) => {
            eprintln!("❌ Received error: {}", error.error);
            
            let error_json = serde_json::json!({
                "status": "error", 
                "error": error.error,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            println!("📋 RESULT: {}", serde_json::to_string(&error_json)?);
        }
    }

    println!("🎯 fastn-p2p sender test completed");
    Ok(())
}