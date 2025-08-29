impl fastn_rig::EndpointManager {
    /// Create a new EndpointManager with a message channel
    pub fn new(
        graceful: fastn_net::Graceful,
    ) -> (Self, tokio::sync::mpsc::Receiver<fastn_rig::P2PMessage>) {
        let (message_tx, message_rx) = tokio::sync::mpsc::channel(1000);

        (
            Self {
                active: std::collections::HashMap::new(),
                message_tx,
                graceful,
                peer_stream_senders: fastn_net::PeerStreamSenders::default(),
            },
            message_rx,
        )
    }

    /// Get connection pool for P2P stream reuse
    pub fn peer_stream_senders(&self) -> &fastn_net::PeerStreamSenders {
        &self.peer_stream_senders
    }

    /// Bring an endpoint online
    pub async fn bring_online(
        &mut self,
        id52: String,
        secret_key: Vec<u8>,
        owner_type: fastn_rig::OwnerType,
        owner_path: std::path::PathBuf,
        account_manager: std::sync::Arc<fastn_account::AccountManager>,
    ) -> Result<(), fastn_rig::EndpointError> {
        if self.active.contains_key(&id52) {
            return Err(fastn_rig::EndpointError::EndpointAlreadyOnline { id52 });
        }

        tracing::info!("Bringing endpoint {} online", id52);

        // Convert secret key bytes to Iroh SecretKey
        let secret_key_array: [u8; 32] = secret_key
            .as_slice()
            .try_into()
            .map_err(|_| fastn_rig::EndpointError::InvalidSecretKeyLength)?;
        let iroh_secret_key = iroh::SecretKey::from_bytes(&secret_key_array);

        // Create Iroh endpoint with this identity using proper ALPN
        let endpoint = iroh::Endpoint::builder()
            .secret_key(iroh_secret_key.clone())
            .alpns(vec![fastn_net::APNS_IDENTITY.into()])
            .relay_mode(iroh::RelayMode::Default)
            .discovery_n0()
            .discovery_local_network()
            .bind()
            .await
            .map_err(|e| fastn_rig::EndpointError::IrohEndpointCreationFailed {
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        // Get endpoint info for logging
        let node_id = endpoint.node_id();
        tracing::info!("Endpoint {} (node_id: {}) listening", id52, node_id);

        // Spawn the endpoint listener using graceful
        let endpoint_clone = endpoint.clone();
        let owner_type_clone = owner_type.clone();
        let message_tx = self.message_tx.clone();
        let graceful_for_endpoint = self.graceful.clone();

        let our_endpoint_key: fastn_id52::PublicKey =
            id52.parse()
                .map_err(|_| fastn_rig::EndpointError::IrohEndpointCreationFailed {
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid ID52",
                    )) as Box<dyn std::error::Error + Send + Sync>,
                })?;

        let handle = self.graceful.spawn(async move {
            endpoint_listener(
                our_endpoint_key,
                endpoint_clone,
                owner_type_clone,
                message_tx,
                graceful_for_endpoint,
                account_manager,
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
    pub async fn take_offline(&mut self, id52: &str) -> Result<(), fastn_rig::EndpointError> {
        if let Some(handle) = self.active.remove(id52) {
            tracing::info!("Taking endpoint {} offline", id52);

            // Close the endpoint - this will cause accept() to return None
            // and the listener task will exit gracefully
            handle.endpoint.close().await;

            // Wait for the task to finish with a timeout
            match tokio::time::timeout(std::time::Duration::from_secs(5), handle.handle).await {
                Ok(Ok(())) => tracing::debug!("Endpoint {} stopped gracefully", id52),
                Ok(Err(e)) => tracing::warn!("Endpoint {} task error: {}", id52, e),
                Err(_) => {
                    tracing::warn!("Endpoint {} shutdown timed out", id52);
                    // Task didn't finish in time, but since we closed the endpoint,
                    // it should eventually stop on its own
                }
            }
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
    pub async fn shutdown_all(&mut self) -> Result<(), fastn_rig::EndpointError> {
        let endpoints: Vec<String> = self.active.keys().cloned().collect();

        for id52 in endpoints {
            self.take_offline(&id52).await?;
        }

        Ok(())
    }
}

/// Endpoint listener that uses fastn-net utilities for proper connection handling
async fn endpoint_listener(
    our_endpoint: fastn_id52::PublicKey,
    endpoint: iroh::Endpoint,
    owner_type: fastn_rig::OwnerType,
    message_tx: tokio::sync::mpsc::Sender<fastn_rig::P2PMessage>,
    graceful: fastn_net::Graceful,
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
) {
    tracing::debug!("Starting listener for endpoint {our_endpoint}");

    loop {
        tokio::select! {
            // Accept incoming connections
            conn = endpoint.accept() => {
                let conn = match conn {
                    Some(conn) => conn,
                    None => {
                        tracing::debug!("Endpoint {} closed", our_endpoint);
                        break;
                    }
                };

                // Spawn a task to handle the incoming connection
                let owner_type_clone = owner_type.clone();
                let message_tx_clone = message_tx.clone();
                let graceful_for_conn = graceful.clone();
                let account_manager_clone = account_manager.clone();

                graceful.spawn(async move {
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("Failed to accept connection: {}", e);
                            return;
                        }
                    };

                    if let Err(e) = handle_connection(
                        our_endpoint,
                        owner_type_clone,
                        conn,
                        message_tx_clone,
                        graceful_for_conn,
                        account_manager_clone,
                    ).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }

            // Handle graceful shutdown
            _ = graceful.cancelled() => {
                tracing::debug!("Shutting down listener for endpoint {our_endpoint}");
                break;
            }
        }
    }
}

/// Handle an incoming connection using fastn-net protocol utilities
async fn handle_connection(
    our_endpoint: fastn_id52::PublicKey,
    owner_type: fastn_rig::OwnerType,
    conn: iroh::endpoint::Connection,
    message_tx: tokio::sync::mpsc::Sender<fastn_rig::P2PMessage>,
    graceful: fastn_net::Graceful,
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
) -> Result<(), fastn_rig::EndpointError> {
    // Get remote peer's PublicKey
    let peer_key = fastn_net::get_remote_id52(&conn).await.map_err(|e| {
        fastn_rig::EndpointError::ConnectionHandlingFailed {
            source: Box::new(std::io::Error::other(format!(
                "Failed to get remote ID52: {e}"
            ))) as Box<dyn std::error::Error + Send + Sync>,
        }
    })?;

    tracing::debug!("Handling connection from {peer_key} to endpoint {our_endpoint}");

    // Perform connection authorization directly
    tracing::debug!("Authorizing connection from {peer_key} to {our_endpoint}");

    // Find the account that owns this endpoint
    let account = match account_manager.find_account_by_alias(&our_endpoint).await {
        Ok(account) => account,
        Err(e) => {
            tracing::warn!("No account found for endpoint {our_endpoint}: {e}");
            return Ok(()); // Close connection silently
        }
    };

    // Authorize the connection
    let authorized = match account.authorize_connection(&peer_key, &our_endpoint).await {
        Ok(auth) => auth,
        Err(e) => {
            tracing::error!("Authorization failed for peer {peer_key}: {e}");
            return Ok(()); // Close connection
        }
    };

    if !authorized {
        tracing::warn!("Connection rejected for peer {peer_key} to endpoint {our_endpoint}");
        return Ok(()); // Close connection immediately
    }

    tracing::info!("✅ Connection authorized for peer {peer_key} to endpoint {our_endpoint}");

    // Handle multiple streams on this connection
    loop {
        tokio::select! {
            // Accept bidirectional streams with protocol negotiation
            _ = graceful.cancelled() => {
                tracing::debug!("Connection handler cancelled");
                break;
            }

            else => {
                // Determine expected protocols based on owner type
                let expected_protocols: Vec<fastn_net::Protocol> = match owner_type {
                    fastn_rig::OwnerType::Account => {
                        // Accounts can receive from Devices or other Accounts
                        vec![
                            fastn_net::Protocol::DeviceToAccount,
                            fastn_net::Protocol::AccountToAccount,
                        ]
                    }
                    fastn_rig::OwnerType::Device => {
                        // Devices receive from Accounts
                        vec![fastn_net::Protocol::AccountToDevice]
                    }
                    fastn_rig::OwnerType::Rig => {
                        // Rig receives control messages
                        vec![fastn_net::Protocol::RigControl]
                    }
                };

                // Accept a bidirectional stream with protocol negotiation
                match fastn_net::accept_bi(&conn, &expected_protocols).await {
                    Ok((protocol, mut send, mut recv)) => {
                        // Read the actual message content
                        match fastn_net::next_string(&mut recv).await {
                            Ok(message_str) => {
                                // TODO: Parse and process the message based on protocol type
                                // For now, just log it
                                tracing::info!(
                                    "Received {protocol:?} message on {our_endpoint} from {peer_key}: {} bytes",
                                    message_str.len()
                                );

                                // Send the message through the channel for processing
                                if let Err(e) = message_tx
                                    .send(fastn_rig::P2PMessage {
                                        our_endpoint,
                                        owner_type: owner_type.clone(),
                                        peer_id52: peer_key,
                                        message: message_str.into_bytes(),
                                    })
                                    .await
                                {
                                    tracing::error!("Failed to send message to channel: {}", e);
                                }

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
                        // This might happen if the protocol doesn't match
                        // Try to accept as a different protocol or handle ping
                        tracing::debug!("Protocol mismatch or error: {}", e);
                        // The accept_bi function already handles Ping internally
                        // For now, we'll continue to the next iteration
                        continue;
                    }
                }
            }
        }
    }

    // Connection ended
    Ok(())
}
