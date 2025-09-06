//! Minimal fastn-net P2P receiver test
//!
//! Tests basic connection handling and stream acceptance

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize tracing with DEBUG level for fastn-net
    tracing_subscriber::fmt()
        .with_env_filter("fastn_net=trace,fastn_net_test=info")
        .init();

    // Get secret key from command line args or generate one
    let args: Vec<String> = std::env::args().collect();
    let receiver_key = if args.len() > 1 {
        // Use provided secret key
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
        // Generate new key
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

    // Create endpoint using fastn-net (same as other code)
    let endpoint = fastn_net::get_endpoint(receiver_key).await?;

    println!("📡 Receiver endpoint listening");
    println!("🎯 Waiting for connections...");

    let graceful = fastn_net::Graceful::new();

    // Handle incoming connections (minimal version)
    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                println!("🛑 Receiver shutting down");
                break;
            }

            Some(incoming) = endpoint.accept() => {
                // Spawn immediately without blocking main loop
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            match fastn_net::get_remote_id52(&conn).await {
                                Ok(peer_key) => {
                                    println!("🔗 Accepted connection from {}", peer_key.id52());
                                    println!("⚠️ Authorization disabled - accepting all connections");

                                    if let Err(e) = handle_connection(conn).await {
                                        eprintln!("❌ Connection handler error: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to get remote peer ID: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to accept connection: {}", e);
                        }
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_connection(conn: iroh::endpoint::Connection) -> eyre::Result<()> {
    println!("🔄 Starting connection handler");

    let conn = std::sync::Arc::new(conn);

    // Accept AccountToAccount streams concurrently
    loop {
        println!("⏳ Waiting for bidirectional stream...");

        match fastn_net::accept_bi(&conn, &[fastn_net::Protocol::Generic(serde_json::json!("Echo"))]).await {
            Ok((protocol, send, recv)) => {
                println!("✅ Accepted {:?} stream via fastn_net::accept_bi", protocol);

                // Spawn concurrent handler for this stream
                tokio::spawn(async move {
                    if let Err(e) = handle_stream(protocol, send, recv).await {
                        println!("❌ Stream handler error: {}", e);
                    }
                });

                // Continue accepting more streams immediately (concurrent)
                println!("🔄 Continuing to accept more streams concurrently");
            }
            Err(e) => {
                println!("❌ Failed to accept stream: {}", e);
                return Ok(());
            }
        }
    }
}

async fn handle_stream(
    protocol: fastn_net::Protocol,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
) -> eyre::Result<()> {
    println!("🧵 Stream handler started for {:?} protocol", protocol);

    // fastn_net::accept_bi already handled protocol negotiation and sent ACK

    // Read actual message
    let message = fastn_net::next_string(&mut recv).await?;
    println!("📨 Stream received message: {}", message);

    // Send response
    let response = "Message received successfully!";
    send.write_all(response.as_bytes()).await?;
    send.write_all(b"\n").await?;
    println!("📤 Stream sent response: {}", response);

    // Properly close the stream
    send.finish()
        .map_err(|e| eyre::anyhow!("Failed to finish send stream: {e}"))?;
    println!("🔚 Stream finished properly");

    Ok(())
}
