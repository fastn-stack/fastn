impl fastn_rig::EndpointManager {
    /// Create a new EndpointManager with a message channel
    pub fn new(
        graceful: fastn_net::Graceful,
    ) -> (
        Self,
        tokio::sync::mpsc::Receiver<(String, fastn_rig::OwnerType, Vec<u8>)>,
    ) {
        let (message_tx, message_rx) = tokio::sync::mpsc::channel(1000);

        (
            Self {
                active: std::collections::HashMap::new(),
                message_tx,
                graceful,
            },
            message_rx,
        )
    }

    /// Bring an endpoint online
    pub async fn bring_online(
        &mut self,
        id52: String,
        secret_key: Vec<u8>,
        owner_type: fastn_rig::OwnerType,
        owner_path: std::path::PathBuf,
    ) -> eyre::Result<()> {
        if self.active.contains_key(&id52) {
            tracing::debug!("Endpoint {} already online", id52);
            return Ok(());
        }

        tracing::info!("Bringing endpoint {} online", id52);

        // Convert secret key bytes to Iroh SecretKey
        let secret_key_array: [u8; 32] = secret_key
            .as_slice()
            .try_into()
            .map_err(|_| eyre::eyre!("Secret key must be exactly 32 bytes"))?;
        let iroh_secret_key = iroh::SecretKey::from_bytes(&secret_key_array);

        // Create Iroh endpoint with this identity using proper ALPN
        let endpoint = iroh::Endpoint::builder()
            .secret_key(iroh_secret_key.clone())
            .alpns(vec![fastn_net::APNS_IDENTITY.into()])
            .relay_mode(iroh::RelayMode::Default)
            .discovery_n0()
            .discovery_local_network()
            .bind()
            .await?;

        // Get endpoint info for logging
        let node_id = endpoint.node_id();
        tracing::info!("Endpoint {} (node_id: {}) listening", id52, node_id);

        // Spawn the endpoint listener using graceful
        let id52_clone = id52.clone();
        let endpoint_clone = endpoint.clone();
        let owner_type_clone = owner_type.clone();
        let message_tx = self.message_tx.clone();
        let graceful_for_endpoint = self.graceful.clone();

        let handle = self.graceful.spawn(async move {
            endpoint_listener(
                id52_clone,
                endpoint_clone,
                owner_type_clone,
                message_tx,
                graceful_for_endpoint,
            )
            .await;
        });

        self.active.insert(
            id52.clone(),
            fastn_rig::EndpointHandle {
                secret_key,
                owner_type,
                owner_path,
                endpoint,
                handle,
            },
        );

        Ok(())
    }

    /// Take an endpoint offline
    pub async fn take_offline(&mut self, id52: &str) -> eyre::Result<()> {
        if let Some(handle) = self.active.remove(id52) {
            tracing::info!("Taking endpoint {} offline", id52);

            // Abort the task
            handle.handle.abort();

            // Close the endpoint
            handle.endpoint.close().await;
        }

        Ok(())
    }

    /// Get list of active endpoint IDs
    pub fn active_endpoints(&self) -> Vec<String> {
        self.active.keys().cloned().collect()
    }

    /// Check if an endpoint is online
    pub fn is_online(&self, id52: &str) -> bool {
        self.active.contains_key(id52)
    }

    /// Shutdown all endpoints
    pub async fn shutdown_all(&mut self) -> eyre::Result<()> {
        let endpoints: Vec<String> = self.active.keys().cloned().collect();

        for id52 in endpoints {
            self.take_offline(&id52).await?;
        }

        Ok(())
    }
}

/// Endpoint listener that uses fastn-net utilities for proper connection handling
async fn endpoint_listener(
    id52: String,
    endpoint: iroh::Endpoint,
    owner_type: fastn_rig::OwnerType,
    message_tx: tokio::sync::mpsc::Sender<(String, fastn_rig::OwnerType, Vec<u8>)>,
    graceful: fastn_net::Graceful,
) {
    tracing::debug!("Starting listener for endpoint {}", id52);

    // Create connection pool for reusing connections
    let peer_stream_senders = fastn_net::PeerStreamSenders::default();

    loop {
        tokio::select! {
            // Accept incoming connections
            conn = endpoint.accept() => {
                let conn = match conn {
                    Some(conn) => conn,
                    None => {
                        tracing::debug!("Endpoint {} closed", id52);
                        break;
                    }
                };

                // Spawn a task to handle the incoming connection
                let id52_clone = id52.clone();
                let owner_type_clone = owner_type.clone();
                let message_tx_clone = message_tx.clone();
                let peer_stream_senders_clone = peer_stream_senders.clone();
                let graceful_for_conn = graceful.clone();

                graceful.spawn(async move {
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("Failed to accept connection: {}", e);
                            return;
                        }
                    };

                    if let Err(e) = handle_connection(
                        id52_clone,
                        owner_type_clone,
                        conn,
                        message_tx_clone,
                        graceful_for_conn,
                        peer_stream_senders_clone,
                    ).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }

            // Handle graceful shutdown
            _ = graceful.cancelled() => {
                tracing::debug!("Shutting down listener for endpoint {}", id52);
                break;
            }
        }
    }
}

/// Handle an incoming connection using fastn-net protocol utilities
async fn handle_connection(
    endpoint_id52: String,
    _owner_type: fastn_rig::OwnerType,
    conn: iroh::endpoint::Connection,
    _message_tx: tokio::sync::mpsc::Sender<(String, fastn_rig::OwnerType, Vec<u8>)>,
    graceful: fastn_net::Graceful,
    _peer_stream_senders: fastn_net::PeerStreamSenders,
) -> eyre::Result<()> {
    // Get remote ID52
    let remote_id52 = fastn_net::get_remote_id52(&conn).await?;

    tracing::debug!(
        "Handling connection from {} to endpoint {}",
        remote_id52,
        endpoint_id52
    );

    // Handle multiple streams on this connection
    loop {
        tokio::select! {
            // Accept bidirectional streams with protocol negotiation
            _ = graceful.cancelled() => {
                tracing::debug!("Connection handler cancelled");
                break;
            }

            else => {
                // For now, accept any incoming bi-directional stream
                // In the future, we'll define our own protocol types for Account/Device/Rig messages
                match conn.accept_bi().await {
                    Ok((mut send, mut recv)) => {
                        // Read the message using fastn-net utilities
                        match fastn_net::next_string(&mut recv).await {
                            Ok(message_str) => {
                                // TODO: Process the message based on content and owner_type
                                // For now, just log it
                                tracing::info!(
                                    "Received message on {} from {}: {} bytes",
                                    endpoint_id52,
                                    remote_id52,
                                    message_str.len()
                                );

                                // Send acknowledgment
                                if let Err(e) = send.write_all(format!("{}\n", fastn_net::ACK).as_bytes()).await {
                                    tracing::error!("Failed to send ACK: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to read message: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept bi-stream: {}", e);
                        break;
                    }
                }
            }
        }
    }

    // Connection ended
    Ok(())
}
