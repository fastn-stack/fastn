impl fastn_rig::EndpointManager {
    /// Create a new EndpointManager with a message channel
    pub fn new() -> (
        Self,
        tokio::sync::mpsc::Receiver<(String, fastn_rig::OwnerType, Vec<u8>)>,
    ) {
        let (message_tx, message_rx) = tokio::sync::mpsc::channel(1000);

        (
            Self {
                active: std::collections::HashMap::new(),
                message_tx,
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
        // Note: fastn_id52 uses ed25519, Iroh also uses ed25519, so we can convert
        // secret_key is a Vec<u8> that should contain exactly 32 bytes
        let secret_key_array: [u8; 32] = secret_key
            .as_slice()
            .try_into()
            .map_err(|_| eyre::eyre!("Secret key must be exactly 32 bytes"))?;
        let iroh_secret_key = iroh::SecretKey::from_bytes(&secret_key_array);

        // Create Iroh endpoint with this identity
        let endpoint = iroh::Endpoint::builder()
            .secret_key(iroh_secret_key.clone())
            .alpns(vec![b"fastn/email/0".to_vec()])
            .relay_mode(iroh::RelayMode::Default)
            .bind()
            .await?;

        // Get endpoint info for logging
        let node_id = endpoint.node_id();
        let _addrs = endpoint.direct_addresses(); // Will use this when we need to display addresses
        tracing::info!("Endpoint {} (node_id: {}) listening", id52, node_id);

        // Setup listener task
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let id52_clone = id52.clone();
        let endpoint_clone = endpoint.clone();
        let message_tx = self.message_tx.clone();

        let owner_type_clone = owner_type.clone();
        let handle = tokio::spawn(async move {
            endpoint_listener(
                id52_clone,
                endpoint_clone,
                owner_type_clone,
                message_tx,
                shutdown_rx,
            )
            .await;
        });

        self.active.insert(
            id52,
            fastn_rig::EndpointHandle {
                secret_key,
                owner_type,
                owner_path,
                endpoint,
                shutdown_tx,
                handle,
            },
        );

        Ok(())
    }

    /// Take an endpoint offline
    pub async fn take_offline(&mut self, id52: &str) -> eyre::Result<()> {
        if let Some(handle) = self.active.remove(id52) {
            tracing::info!("Taking endpoint {} offline", id52);

            // Signal shutdown
            let _ = handle.shutdown_tx.send(());

            // Wait with timeout
            match tokio::time::timeout(std::time::Duration::from_secs(5), handle.handle).await {
                Ok(_) => tracing::debug!("Endpoint {} stopped gracefully", id52),
                Err(_) => tracing::warn!("Endpoint {} shutdown timed out", id52),
            }

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

/// Generic endpoint listener - forwards messages to the channel
async fn endpoint_listener(
    id52: String,
    endpoint: iroh::Endpoint,
    owner_type: fastn_rig::OwnerType,
    message_tx: tokio::sync::mpsc::Sender<(String, fastn_rig::OwnerType, Vec<u8>)>,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    tracing::debug!("Starting listener for endpoint {}", id52);

    loop {
        tokio::select! {
            // Accept incoming connections
            incoming = endpoint.accept() => {
                match incoming {
                    Some(incoming) => {
                        match incoming.await {
                            Ok(conn) => {
                                tracing::info!("Accepted connection on endpoint {}", id52);

                                // Spawn a task to handle this connection
                                let id52_clone = id52.clone();
                                let owner_type_clone = owner_type.clone();
                                let message_tx_clone = message_tx.clone();

                                tokio::spawn(async move {
                                    handle_connection(id52_clone, owner_type_clone, conn, message_tx_clone).await;
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    None => {
                        tracing::debug!("Endpoint {} closed", id52);
                        break;
                    }
                }
            }

            // Handle shutdown signal
            _ = &mut shutdown_rx => {
                tracing::debug!("Shutting down listener for endpoint {}", id52);
                break;
            }
        }
    }
}

/// Handle an incoming connection
async fn handle_connection(
    endpoint_id52: String,
    owner_type: fastn_rig::OwnerType,
    conn: iroh::endpoint::Connection,
    message_tx: tokio::sync::mpsc::Sender<(String, fastn_rig::OwnerType, Vec<u8>)>,
) {
    tracing::debug!("Handling connection for endpoint {}", endpoint_id52);

    // Accept a bi-directional stream
    match conn.accept_bi().await {
        Ok((mut send, mut recv)) => {
            // Read message from the stream
            match recv.read_chunk(1024 * 1024, true).await {
                Ok(Some(chunk)) => {
                    let buffer = chunk.bytes.to_vec();

                    // Send to message channel
                    if let Err(e) = message_tx
                        .send((endpoint_id52.clone(), owner_type.clone(), buffer))
                        .await
                    {
                        tracing::error!("Failed to send message to channel: {}", e);
                    }

                    // Send acknowledgment
                    if let Err(e) = send.write_all(b"ACK").await {
                        tracing::error!("Failed to send ACK: {}", e);
                    }
                }
                Ok(None) => {
                    tracing::debug!("Stream closed");
                }
                Err(e) => {
                    tracing::error!("Failed to read chunk: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to accept bi-stream: {}", e);
        }
    }
}
