async fn handle_connection<P>(
    conn: iroh::endpoint::Incoming,
    expected_protocols: Vec<P>,
    tx: tokio::sync::mpsc::Sender<eyre::Result<fastn_p2p::Request<P>>>,
) -> eyre::Result<()> 
where
    P: serde::Serialize + 
       for<'de> serde::Deserialize<'de> + 
       Clone + 
       PartialEq + 
       std::fmt::Display +
       std::fmt::Debug +
       Send + 
       Sync + 
       'static,
{
    // Wait for the connection to be established
    let connection = conn.await?;

    // Get peer's ID52
    let peer_key = fastn_net::get_remote_id52(&connection).await?;

    tracing::debug!("Connection established with peer: {}", peer_key.id52());

    // Convert user protocols to fastn_net::Protocol::Generic for network transmission
    let net_protocols: Vec<fastn_net::Protocol> = expected_protocols
        .iter()
        .map(|p| {
            let json_value = serde_json::to_value(p)
                .expect("Protocol should be serializable");
            fastn_net::Protocol::Generic(json_value)
        })
        .collect();

    // Accept bi-directional streams on this connection using fastn_net utilities
    loop {
        let (net_protocol, send, recv) =
            match fastn_net::accept_bi(&connection, &net_protocols).await {
                Ok(result) => result,
                Err(e) => {
                    tracing::warn!("Failed to accept bi-directional stream: {e}");
                    break;
                }
            };

        // Convert back from fastn_net::Protocol::Generic to user protocol
        let user_protocol = match net_protocol {
            fastn_net::Protocol::Generic(json_value) => {
                match serde_json::from_value::<P>(json_value) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("Failed to deserialize user protocol: {e}");
                        continue;
                    }
                }
            }
            other => {
                tracing::warn!("Expected Generic protocol, got: {:?}", other);
                continue;
            }
        };

        tracing::debug!("Accepted {user_protocol} connection from peer: {peer_key}");

        // Create PeerRequest and send it through the channel
        let peer_request = fastn_p2p::server::request::Request::new(
            peer_key,
            user_protocol,
            send,
            recv,
        );

        if (tx.send(Ok(peer_request)).await).is_err() {
            // Channel receiver has been dropped, stop processing
            tracing::debug!("Channel receiver dropped, stopping connection handling");
            break;
        }
    }

    Ok(())
}


fn listen_generic<P>(
    secret_key: fastn_id52::SecretKey,
    expected: &[P],
    graceful: fastn_net::Graceful,
) -> Result<
    impl futures_core::stream::Stream<Item = eyre::Result<fastn_p2p::Request<P>>>,
    fastn_p2p::ListenerAlreadyActiveError,
>
where
    P: serde::Serialize + 
       for<'de> serde::Deserialize<'de> + 
       Clone + 
       PartialEq + 
       std::fmt::Display +
       std::fmt::Debug +
       Send + 
       Sync + 
       'static,
{
    let public_key = secret_key.public_key();

    // Check if already listening and register this endpoint
    let endpoint_cancellation_token = fastn_p2p::server::management::register_listener(public_key)?;

    let expected = expected.to_vec(); // Clone for move into async block

    Ok(async_stream::try_stream! {
        println!("🔧 DEBUG: About to call fastn_net::get_endpoint");
        let endpoint = fastn_net::get_endpoint(secret_key.clone()).await?;
        println!("🔧 DEBUG: Successfully created endpoint");

        // Channel to receive PeerRequests from spawned connection handlers
        // Using 1-capacity channel for minimal buffering with backpressure
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        println!("🔧 DEBUG: Created channel");

        // Spawn connection acceptor task
        let acceptor_tx = tx.clone();
        let acceptor_graceful = graceful.clone();
        let acceptor_endpoint_cancellation = endpoint_cancellation_token.clone();
        let acceptor_expected = expected.clone();
        let acceptor_public_key = public_key;

        tokio::spawn(async move {
            println!("🔧 DEBUG: Started connection acceptor task");
            loop {
                println!("🔧 DEBUG: Waiting for endpoint.accept()");
                tokio::select! {
                    conn_result = endpoint.accept() => {
                        println!("🔧 DEBUG: endpoint.accept() returned");
                        let conn = match conn_result {
                            Some(conn) => conn,
                            None => {
                                tracing::debug!("Endpoint {public_key} closed");
                                break;
                            }
                        };

                        // Spawn task to handle this specific connection
                        let handler_tx = acceptor_tx.clone();
                        let handler_expected = acceptor_expected.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection::<P>(conn, handler_expected, handler_tx).await {
                                tracing::warn!("Connection handling failed: {e}");
                            }
                        });
                    }

                    // Handle global graceful shutdown
                    _ = acceptor_graceful.cancelled() => {
                        tracing::debug!("Global shutdown: stopping listener for endpoint {public_key}");
                        break;
                    }

                    // Handle endpoint-specific cancellation
                    _ = acceptor_endpoint_cancellation.cancelled() => {
                        tracing::debug!("Endpoint-specific shutdown: stopping listener for endpoint {public_key}");
                        break;
                    }
                }
            }

            // Clean up: remove from global registry when task ends
            fastn_p2p::server::management::unregister_listener(&acceptor_public_key);
        });

        // Stream PeerRequests from the channel
        println!("🔧 DEBUG: About to start streaming from channel");
        while let Some(peer_request_result) = rx.recv().await {
            println!("🔧 DEBUG: Received peer request from channel");
            yield peer_request_result?;
        }
        println!("🔧 DEBUG: Channel stream ended");

        // Clean up: ensure removal from registry when stream ends
        fastn_p2p::server::management::unregister_listener(&public_key);
    })
}

/// Start listening for P2P requests using global graceful shutdown
/// 
/// This is the main function end users should use. It automatically uses
/// the global graceful shutdown coordinator.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use futures_util::stream::StreamExt;
/// 
/// async fn server_example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut stream = fastn_p2p::listen!(secret_key, &protocols)?;
///     let mut stream = std::pin::pin!(stream);
///     
///     while let Some(request) = stream.next().await {
///         let request = request?;
///         match request.protocol {
///             fastn_p2p::Protocol::Ping => {
///                 // Handle ping requests...
///             }
///             _ => { /* other protocols */ }
///         }
///     }
///     Ok(())
/// }
/// ```
pub fn listen<P>(
    secret_key: fastn_id52::SecretKey,
    expected: &[P],
) -> Result<
    impl futures_core::stream::Stream<Item = eyre::Result<fastn_p2p::Request<P>>>,
    fastn_p2p::ListenerAlreadyActiveError,
>
where
    P: serde::Serialize + 
       for<'de> serde::Deserialize<'de> + 
       Clone + 
       PartialEq + 
       std::fmt::Display +
       std::fmt::Debug +
       Send + 
       Sync + 
       'static,
{
    listen_generic(secret_key, expected, crate::graceful())
}
