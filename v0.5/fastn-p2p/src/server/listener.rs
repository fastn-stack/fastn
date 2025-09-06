async fn handle_connection(
    conn: iroh::endpoint::Incoming,
    expected_protocols: Vec<fastn_net::Protocol>,
    tx: tokio::sync::mpsc::Sender<eyre::Result<fastn_p2p::Request>>,
) -> eyre::Result<()> {
    // Wait for the connection to be established
    let connection = conn.await?;

    // Get peer's ID52
    let peer_key = fastn_net::get_remote_id52(&connection).await?;

    tracing::debug!("Connection established with peer: {}", peer_key.id52());

    // Accept bi-directional streams on this connection using fastn_net utilities
    loop {
        let (protocol, send, recv) =
            match fastn_net::accept_bi(&connection, &expected_protocols).await {
                Ok(result) => result,
                Err(e) => {
                    tracing::warn!("Failed to accept bi-directional stream: {e}");
                    break;
                }
            };

        tracing::debug!("Accepted {protocol} connection from peer: {peer_key}");

        // Create PeerRequest and send it through the channel
        let peer_request = fastn_p2p::Request {
            peer: peer_key,
            protocol,
            send,
            recv,
        };

        if (tx.send(Ok(peer_request)).await).is_err() {
            // Channel receiver has been dropped, stop processing
            tracing::debug!("Channel receiver dropped, stopping connection handling");
            break;
        }
    }

    Ok(())
}


pub fn p2p_listen(
    secret_key: fastn_id52::SecretKey,
    expected: &[fastn_net::Protocol],
    graceful: fastn_net::Graceful,
) -> Result<
    impl futures_core::stream::Stream<Item = eyre::Result<fastn_p2p::Request>>,
    fastn_p2p::ListenerAlreadyActiveError,
> {
    let public_key = secret_key.public_key();

    // Check if already listening and register this endpoint
    let endpoint_cancellation_token = fastn_p2p::server::management::register_listener(public_key)?;

    let expected = expected.to_vec(); // Clone for move into async block

    Ok(async_stream::try_stream! {
        let endpoint = fastn_net::get_endpoint(secret_key.clone()).await?;

        // Channel to receive PeerRequests from spawned connection handlers
        // Using 0-capacity channel to ensure backpressure - each connection blocks
        // until the stream consumer is ready to receive it
        let (tx, mut rx) = tokio::sync::mpsc::channel(0);

        // Spawn connection acceptor task
        let acceptor_tx = tx.clone();
        let acceptor_graceful = graceful.clone();
        let acceptor_endpoint_cancellation = endpoint_cancellation_token.clone();
        let acceptor_expected = expected.clone();
        let acceptor_public_key = public_key;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    conn_result = endpoint.accept() => {
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
                            if let Err(e) = handle_connection(conn, handler_expected, handler_tx).await {
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
        while let Some(peer_request_result) = rx.recv().await {
            yield peer_request_result?;
        }

        // Clean up: ensure removal from registry when stream ends
        fastn_p2p::server::management::unregister_listener(&public_key);
    })
}