//! Minimal fastn-net P2P sender test
//!
//! Tests basic get_stream functionality with minimal code

use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize tracing with DEBUG level for fastn-net
    tracing_subscriber::fmt()
        .with_env_filter("fastn_net=trace,fastn_net_test=info")
        .init();

    // Parse command line arguments: sender <sender_secret_key> <receiver_id52>
    let args: Vec<String> = std::env::args().collect();

    let (sender_key, receiver_id52) = if args.len() >= 3 {
        // Use provided sender secret key and receiver ID52
        let sender_secret_str = &args[1];
        let receiver_id52 = args[2].clone();

        let sender_key = match sender_secret_str.parse::<fastn_id52::SecretKey>() {
            Ok(key) => key,
            Err(e) => {
                eprintln!("❌ Invalid sender secret key: {}", e);
                std::process::exit(1);
            }
        };

        println!("🔑 Using provided sender secret key");
        (sender_key, receiver_id52)
    } else if args.len() == 2 {
        // Legacy mode: generate sender key, use provided receiver ID52
        let sender_key = fastn_id52::SecretKey::generate();
        let receiver_id52 = args[1].clone();

        println!("🔑 Generated new sender key (legacy mode)");
        (sender_key, receiver_id52)
    } else {
        eprintln!("Usage: sender <receiver_id52> OR sender <sender_secret_key> <receiver_id52>");
        std::process::exit(1);
    };

    let sender_id52 = sender_key.public_key().id52();
    println!("🔑 Sender ID52: {}", sender_id52);

    println!("🎯 Target receiver: {}", receiver_id52);

    // Create endpoint
    let endpoint = fastn_net::get_endpoint(sender_key).await?;
    println!("📡 Sender endpoint created");

    // Create peer stream senders coordination
    let peer_stream_senders = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    // Create graceful shutdown handler
    let graceful = fastn_net::Graceful::new();

    // Test message
    let test_message = serde_json::json!({
        "from": sender_id52,
        "to": receiver_id52,
        "message": "Hello from fastn-net test!",
        "timestamp": chrono::Utc::now().timestamp()
    });

    println!("📤 About to send test message via fastn-net get_stream");

    // Convert receiver_id52 string to PublicKey
    let receiver_public_key = receiver_id52.parse::<fastn_id52::PublicKey>()
        .map_err(|e| eyre::anyhow!("Invalid receiver_id52: {}", e))?;
    
    // Use get_stream exactly like http_proxy.rs does
    let (mut send, mut recv) = fastn_net::get_stream(
        endpoint,
        fastn_net::Protocol::AccountToAccount.into(),
        &receiver_public_key,
        peer_stream_senders.clone(),
        graceful.clone(),
    )
    .await
    .map_err(|e| eyre::anyhow!("Failed to get P2P stream to {receiver_id52}: {e}"))?;

    println!("✅ fastn-net stream established successfully");

    // Send test message
    let message_json = serde_json::to_string(&test_message)?;
    send.write_all(message_json.as_bytes()).await?;
    send.write_all(b"\n").await?;

    println!("📨 Test message sent, waiting for response...");

    // Wait for response
    let response = fastn_net::next_string(&mut recv).await?;
    println!("✅ Received response: {}", response);

    // Output JSON result for easy parsing in tests
    let result = serde_json::json!({
        "status": "success",
        "message": "P2P communication completed",
        "response_received": response,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    println!("📋 RESULT: {}", serde_json::to_string(&result)?);
    println!("🎉 fastn-net P2P communication successful!");

    Ok(())
}
