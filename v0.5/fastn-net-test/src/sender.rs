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

    // Generate keys for sender
    let sender_key = fastn_id52::SecretKey::generate();
    let sender_id52 = sender_key.public_key().id52();
    println!("🔑 Sender ID52: {}", sender_id52);

    // Target receiver ID52 (will be provided as argument)
    let receiver_id52 = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: sender <receiver_id52>");
        std::process::exit(1);
    });

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

    // Use get_stream exactly like http_proxy.rs does
    let (mut send, mut recv) = fastn_net::get_stream(
        endpoint,
        fastn_net::Protocol::AccountToAccount.into(),
        receiver_id52.clone(),
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

    println!("🎉 fastn-net P2P communication successful!");

    Ok(())
}
