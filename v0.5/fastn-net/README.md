# fastn-net: Peer-to-Peer Networking Library

A Rust library for peer-to-peer communication built on top of iroh and QUIC, providing reliable message passing between peers with automatic connection management.

## Overview

fastn-net provides a high-level API for P2P communication while handling the complexity of:
- Connection establishment and management
- Protocol negotiation and stream coordination  
- Request-response patterns with proper ACK mechanisms
- Graceful error handling and connection lifecycle

## Quick Start

### Basic Sender

```rust
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Create endpoint with generated keys
    let sender_key = fastn_id52::SecretKey::generate();
    let endpoint = fastn_net::get_endpoint(sender_key).await?;
    
    // Set up coordination (required for proper connection management)
    let peer_stream_senders = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new()
    ));
    let graceful = fastn_net::Graceful::new();
    
    // Send message using get_stream
    let (mut send, mut recv) = fastn_net::get_stream(
        endpoint,
        fastn_net::Protocol::AccountToAccount.into(),
        "target_peer_id52".to_string(),
        peer_stream_senders,
        graceful,
    ).await?;
    
    // Send your message
    let message = serde_json::json!({"hello": "world"});
    let message_json = serde_json::to_string(&message)?;
    send.write_all(message_json.as_bytes()).await?;
    send.write_all(b"\n").await?;
    
    // Wait for response
    let response = fastn_net::next_string(&mut recv).await?;
    println!("Received: {}", response);
    
    Ok(())
}
```

### Basic Receiver

```rust
#[tokio::main] 
async fn main() -> eyre::Result<()> {
    // Create endpoint
    let receiver_key = fastn_id52::SecretKey::generate();
    let endpoint = fastn_net::get_endpoint(receiver_key).await?;
    
    let graceful = fastn_net::Graceful::new();
    
    // Accept connections (non-blocking main loop!)
    loop {
        tokio::select! {
            _ = graceful.cancelled() => break,
            
            Some(incoming) = endpoint.accept() => {
                // CRITICAL: Spawn immediately without I/O in main loop
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            if let Err(e) = handle_connection(conn).await {
                                eprintln!("Connection error: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Accept error: {}", e),
                    }
                });
            }
        }
    }
    Ok(())
}

async fn handle_connection(conn: iroh::endpoint::Connection) -> eyre::Result<()> {
    // Handle multiple concurrent streams on this connection
    loop {
        match fastn_net::accept_bi(&conn, &[fastn_net::Protocol::AccountToAccount]).await {
            Ok((protocol, mut send, mut recv)) => {
                // Spawn concurrent handler for each stream
                tokio::spawn(async move {
                    // Read message
                    let message = fastn_net::next_string(&mut recv).await?;
                    
                    // Send response
                    send.write_all(b"Response\n").await?;
                    send.finish()?;
                    
                    Ok::<(), eyre::Error>(())
                });
            }
            Err(e) => {
                println!("Accept stream error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
```

## Key Architecture Patterns

### 1. Non-Blocking Accept Loop ⚠️ CRITICAL

**❌ WRONG - Blocks main loop:**
```rust
Some(incoming) = endpoint.accept() => {
    let conn = incoming.await?;  // ← Blocks other connections!
    let peer_key = fastn_net::get_remote_id52(&conn).await?;  // ← More blocking!
    tokio::spawn(async move { handle_connection(conn).await });
}
```

**✅ CORRECT - Non-blocking:**
```rust
Some(incoming) = endpoint.accept() => {
    tokio::spawn(async move {  // ← Spawn immediately!
        let conn = incoming.await?;
        let peer_key = fastn_net::get_remote_id52(&conn).await?;
        handle_connection(conn).await
    });
}
```

**Why this matters:** The main accept loop must **never block** on I/O operations. Any blocking prevents accepting new connections concurrently.

### 2. Connection and Stream Lifecycle

- **One connection per peer pair** for efficiency
- **Multiple concurrent bidirectional streams** on each connection  
- **Each stream handled in separate task** for true concurrency
- **Proper stream finishing** with `send.finish()` when done

### 3. get_stream Coordination

The `peer_stream_senders` parameter is **not optional** - it provides:
- **Connection reuse** between multiple requests to same peer
- **Request coordination** to prevent connection conflicts
- **Proper cleanup** when connections fail

### 4. Protocol Negotiation

fastn-net handles protocol negotiation automatically:
1. **Sender** calls `get_stream()` with desired protocol
2. **Connection manager** opens bidirectional stream and sends protocol
3. **Receiver** uses `accept_bi()` with expected protocols  
4. **Automatic ACK** sent when protocol matches
5. **Stream returned** to both sides for data exchange

### 5. Error Handling and Resilience

**Connection managers** handle failures gracefully:
- **Automatic retry** with exponential backoff
- **Connection cleanup** when peers disconnect
- **Stream isolation** - one stream failure doesn't break others
- **Graceful shutdown** coordination

## Debugging Tips

### Enable Detailed Tracing

```rust
tracing_subscriber::fmt()
    .with_env_filter("fastn_net=trace")  
    .init();
```

This shows the complete fastn-net internal flow:
- Connection establishment steps
- Protocol negotiation details  
- Stream lifecycle events
- Error conditions and recovery

### Common Issues

1. **Accept loop blocking**: Always spawn tasks immediately in accept loop
2. **Missing peer_stream_senders**: Required for proper coordination
3. **Protocol mismatches**: Ensure sender and receiver use same protocols
4. **Stream leaks**: Always call `send.finish()` when done with streams

## Advanced Usage

### Multiple Protocols

```rust
// Receiver accepting multiple protocol types
match fastn_net::accept_bi(&conn, &[
    fastn_net::Protocol::AccountToAccount,
    fastn_net::Protocol::HttpProxy,  
    fastn_net::Protocol::DeviceToAccount,
]).await {
    Ok((protocol, send, recv)) => {
        match protocol {
            fastn_net::Protocol::AccountToAccount => handle_email(send, recv).await,
            fastn_net::Protocol::HttpProxy => handle_http(send, recv).await,
            // ... 
        }
    }
}
```

### Connection Health Monitoring

```rust
// The connection manager automatically pings peers and handles failures
// No manual health checking needed - fastn-net handles this internally
```

## Examples

See the `fastn-net-test/` directory for minimal working examples:
- `sender.rs` - Basic message sending
- `receiver.rs` - Connection and stream handling

These examples demonstrate the correct patterns for reliable P2P communication.

## Integration

fastn-net is designed to integrate with:
- **fastn-rig**: Endpoint management and routing
- **fastn-account**: Peer authorization and identity
- **fastn-mail**: Email delivery over P2P
- **fastn-router**: HTTP proxy and routing

The library handles the networking complexity while applications focus on business logic.