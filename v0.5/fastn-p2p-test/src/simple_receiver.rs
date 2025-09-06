//! Simplified fastn-p2p receiver that mimics fastn-net-test pattern exactly
//! This will help us isolate whether the issue is in the complex fastn-p2p architecture

use serde::{Deserialize, Serialize};

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
    pub message: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("fastn_p2p=trace,fastn_p2p_test=info")
        .init();

    let receiver_key = fastn_id52::SecretKey::generate();
    let receiver_id52 = receiver_key.public_key().id52();
    
    println!("🔑 Simple Receiver ID52: {}", receiver_id52);

    // DIRECT fastn-net usage (like fastn-net-test) - NO fastn-p2p abstractions
    println!("🔧 DEBUG: About to call fastn_net::get_endpoint directly");
    let endpoint = fastn_net::get_endpoint(receiver_key).await?;
    println!("🔧 DEBUG: Successfully created endpoint directly");

    println!("📡 Simple receiver listening directly on fastn-net");
    println!("🎯 Waiting for connections...");

    // Simple direct accept loop (like fastn-net-test)
    loop {
        println!("🔧 DEBUG: Calling endpoint.accept() directly");
        if let Some(conn) = endpoint.accept().await {
            println!("🔗 Accepted connection directly!");
            
            // Convert user protocol to fastn_net::Protocol::Generic for test
            let test_proto = TestProtocol::Echo;
            let json_value = serde_json::to_value(&test_proto)?;
            let net_protocols = vec![fastn_net::Protocol::Generic(json_value)];
            
            println!("🔧 DEBUG: About to call fastn_net::accept_bi");
            let (_protocol, mut _send, mut _recv) = fastn_net::accept_bi(&conn, &net_protocols).await?;
            println!("✅ accept_bi succeeded!");
            
            break; // Handle one connection
        }
    }

    println!("🎯 Simple receiver test completed");
    Ok(())
}