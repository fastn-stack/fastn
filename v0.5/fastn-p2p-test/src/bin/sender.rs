//! Minimal fastn-p2p sender test
//!
//! Tests the generic protocol system by sending meaningful protocol requests

use serde::{Deserialize, Serialize};

/// Test protocol - meaningful names!
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

#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    println!("🔧 DEBUG SENDER: Starting main function");

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("fastn_p2p=trace,fastn_p2p_test=info")
        .init();

    println!("🔧 DEBUG SENDER: Tracing initialized");

    // Parse command line arguments: sender <sender_secret_key> <receiver_id52>
    let args: Vec<String> = std::env::args().collect();

    let (sender_key, receiver_id52) = if args.len() >= 3 {
        let sender_secret_str = &args[1];
        let receiver_id52 = args[2].clone();

        let sender_key = match sender_secret_str.parse::<fastn_id52::SecretKey>() {
            Ok(key) => key,
            Err(e) => {
                eprintln!("❌ Invalid sender secret key: {e}");
                std::process::exit(1);
            }
        };

        (sender_key, receiver_id52)
    } else {
        eprintln!("❌ Usage: sender <sender_secret_key> <receiver_id52>");
        std::process::exit(1);
    };

    let sender_id52 = sender_key.public_key().id52();
    println!("🔑 Sender ID52: {sender_id52}");
    println!("🎯 Target ID52: {receiver_id52}");

    // Convert receiver ID52 to public key
    let receiver_public_key = receiver_id52
        .parse::<fastn_id52::PublicKey>()
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
    println!("🔧 DEBUG: About to call fastn_p2p::call");
    let result: Result<EchoResponse, EchoError> = fastn_p2p::client::call(
        sender_key,
        receiver_public_key,
        TestProtocol::Echo,
        request,
    )
    .await
    .map_err(|e| {
        eprintln!("❌ fastn_p2p::call failed: {e}");
        e
    })?;
    println!("🔧 DEBUG: fastn_p2p::call completed successfully");

    println!("🔧 DEBUG: About to match result");
    match result {
        Ok(response) => {
            println!("🔧 DEBUG: Got Ok response");
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
            println!("🔧 DEBUG: Got Err response");
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
