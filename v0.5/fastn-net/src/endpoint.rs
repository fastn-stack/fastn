pub struct PeerConnection {
    pub peer: fastn_id52::PublicKey,
    pub protocol: fastn_net::Protocol,
    pub send: iroh::endpoint::SendStream,
    pub recv: iroh::endpoint::RecvStream,
}

async fn handle_connection(
    conn: iroh::endpoint::Incoming,
    expected_protocols: Vec<fastn_net::Protocol>,
    tx: tokio::sync::mpsc::Sender<eyre::Result<PeerConnection>>,
) -> eyre::Result<()> {
    // Wait for the connection to be established
    let connection = conn.await?;
    
    // Get peer's ID52
    let peer_key = fastn_net::get_remote_id52(&connection).await?;
    
    tracing::debug!("Connection established with peer: {}", peer_key.id52());
    
    // Accept bi-directional streams on this connection using fastn_net utilities
    loop {
        let (protocol, send, recv) = match fastn_net::accept_bi(&connection, &expected_protocols).await {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Failed to accept bi-directional stream: {}", e);
                break;
            }
        };
        
        tracing::debug!(
            "Accepted {} connection from peer: {}", 
            protocol, peer_key.id52()
        );
        
        // Create PeerConnection and send it through the channel
        let peer_connection = PeerConnection {
            peer: peer_key.clone(),
            protocol,
            send,
            recv,
        };
        
        if let Err(_) = tx.send(Ok(peer_connection)).await {
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
) -> impl futures_core::stream::Stream<
    Item = eyre::Result<PeerConnection>,
> {
    let expected = expected.to_vec(); // Clone for move into async block
    async_stream::try_stream! {
        let endpoint = fastn_net::get_endpoint(secret_key.clone()).await?;
        let our_id52 = secret_key.id52();
        
        // Channel to receive PeerConnections from spawned connection handlers
        // Using 0-capacity channel to ensure backpressure - each connection blocks 
        // until the stream consumer is ready to receive it
        let (tx, mut rx) = tokio::sync::mpsc::channel(0);

        // Spawn connection acceptor task
        let acceptor_tx = tx.clone();
        let acceptor_graceful = graceful.clone();
        let acceptor_expected = expected.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    conn_result = endpoint.accept() => {
                        let conn = match conn_result {
                            Some(conn) => conn,
                            None => {
                                tracing::debug!("Endpoint {} closed", our_id52);
                                break;
                            }
                        };

                        // Spawn task to handle this specific connection
                        let handler_tx = acceptor_tx.clone();
                        let handler_expected = acceptor_expected.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(conn, handler_expected, handler_tx).await {
                                tracing::warn!("Connection handling failed: {}", e);
                            }
                        });
                    }

                    // Handle graceful shutdown
                    _ = acceptor_graceful.cancelled() => {
                        tracing::debug!("Shutting down listener for endpoint {}", our_id52);
                        break;
                    }
                }
            }
        });

        // Stream PeerConnections from the channel
        while let Some(peer_connection_result) = rx.recv().await {
            yield peer_connection_result?;
        }
    }
}
