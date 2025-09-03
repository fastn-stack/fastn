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
    _message_tx: tokio::sync::mpsc::Sender<fastn_rig::P2PMessage>,
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

    // Authorization disabled for first release - accept all connections
    tracing::info!(
        "✅ Connection accepted for peer {peer_key} to endpoint {our_endpoint} (no authorization)"
    );

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

    // Handle multiple streams on this connection (following receiver.rs pattern)
    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::debug!("Connection handler cancelled");
                break;
            }

            // Accept a bidirectional stream with protocol negotiation
            result = fastn_net::accept_bi(&conn, &expected_protocols) => {
                match result {
                    Ok((protocol, mut send, mut recv)) => {
                        tracing::info!("Accepted {protocol:?} stream from {peer_key} to {our_endpoint}");

                        match protocol {
                            fastn_net::Protocol::HttpProxy => {
                                // Handle incoming HTTP request from remote peer
                                tracing::info!("Handling HTTP proxy request from {} to {}", peer_key, our_endpoint);

                                if let Err(e) = handle_incoming_http_request(
                                    recv, send, &peer_key, &our_endpoint, &account_manager
                                ).await {
                                    tracing::error!("Failed to handle HTTP request: {}", e);
                                    break;
                                }
                            }
                            fastn_net::Protocol::AccountToAccount => {
                                // Handle email messages directly (synchronous processing for proper response)
                                match fastn_net::next_string(&mut recv).await {
                                    Ok(message_str) => {
                                        tracing::info!(
                                            "Received {protocol:?} message on {our_endpoint} from {peer_key}: {} bytes",
                                            message_str.len()
                                        );

                                        // Process the message directly instead of through channel
                                        let account_message = match serde_json::from_slice::<fastn_account::AccountToAccountMessage>(message_str.as_bytes()) {
                                            Ok(msg) => msg,
                                            Err(e) => {
                                                tracing::error!("Failed to deserialize account message: {}", e);
                                                let error_response = fastn_account::EmailDeliveryResponse {
                                                    email_id: "unknown".to_string(),
                                                    status: fastn_account::DeliveryStatus::Rejected {
                                                        reason: format!("Message deserialization failed: {}", e)
                                                    },
                                                };
                                                let json = serde_json::to_string(&error_response).unwrap_or_else(|_| "{}".to_string());
                                                if let Err(e) = send.write_all(format!("{}\n", json).as_bytes()).await {
                                                    tracing::error!("Failed to send error response: {}", e);
                                                }
                                                break;
                                            }
                                        };

                                        // Handle the message via AccountManager
                                        let result = account_manager
                                            .handle_account_message(&peer_key, &our_endpoint, account_message.clone())
                                            .await;

                                        // Get email ID from the message
                                        let email_id = match &account_message {
                                            fastn_account::AccountToAccountMessage::Email { envelope_from, .. } => {
                                                // Use envelope_from as identifier for now
                                                envelope_from.clone()
                                            }
                                        };

                                        // Send proper JSON response
                                        let response = match result {
                                            Ok(_) => {
                                                println!("✅ Account message handled successfully");
                                                fastn_account::EmailDeliveryResponse {
                                                    email_id: email_id.clone(),
                                                    status: fastn_account::DeliveryStatus::Accepted,
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to handle account message: {}", e);
                                                fastn_account::EmailDeliveryResponse {
                                                    email_id: email_id.clone(),
                                                    status: fastn_account::DeliveryStatus::Rejected {
                                                        reason: format!("Message handling failed: {}", e)
                                                    },
                                                }
                                            }
                                        };

                                        let json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
                                        if let Err(e) = send.write_all(format!("{}\n", json).as_bytes()).await {
                                            tracing::error!("Failed to send delivery response: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to read message: {}", e);
                                        break;
                                    }
                                }
                            }
                            _ => {
                                // Handle other protocols (ping, etc.)
                                match fastn_net::next_string(&mut recv).await {
                                    Ok(message_str) => {
                                        tracing::info!(
                                            "Received {protocol:?} message on {our_endpoint} from {peer_key}: {} bytes",
                                            message_str.len()
                                        );

                                        // Send acknowledgment for non-HTTP protocols
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

/// Handle incoming HTTP request from remote peer via P2P
async fn handle_incoming_http_request(
    mut recv: iroh::endpoint::RecvStream,
    mut send: iroh::endpoint::SendStream,
    requester: &fastn_id52::PublicKey,
    our_endpoint: &fastn_id52::PublicKey,
    account_manager: &std::sync::Arc<fastn_account::AccountManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Processing HTTP request from {} to {}",
        requester,
        our_endpoint
    );

    // Parse HTTP request from P2P stream (following fastn_net::peer_to_http pattern)
    let http_req: fastn_net::http::Request = fastn_net::next_json(&mut recv).await?;

    tracing::info!(
        "Received HTTP request: {} {}",
        http_req.method,
        http_req.uri
    );

    // Convert to our HttpRequest type
    let request = fastn_router::HttpRequest {
        method: http_req.method,
        path: http_req.uri,
        host: our_endpoint.id52(),
        headers: http_req
            .headers
            .into_iter()
            .map(|(k, v)| (k, String::from_utf8_lossy(&v).to_string()))
            .collect(),
    };

    // Route the request with requester context for permission control
    let response =
        route_http_with_requester(&request, Some(requester), our_endpoint, account_manager).await;

    // Convert response back to P2P format
    let http_response = fastn_net::http::Response {
        status: response.status,
        headers: response
            .headers
            .into_iter()
            .map(|(k, v)| (k, v.into_bytes()))
            .collect(),
    };

    // Send response back via P2P
    send.write_all(&serde_json::to_vec(&http_response)?).await?;
    send.write_all(b"\n").await?;

    // TODO: Handle response body - for now just send empty body
    tracing::info!("Sent HTTP response back to {}", requester);

    Ok(())
}

/// Route HTTP request with requester context (local or remote P2P)
async fn route_http_with_requester(
    request: &fastn_router::HttpRequest,
    requester: Option<&fastn_id52::PublicKey>,
    our_endpoint: &fastn_id52::PublicKey,
    account_manager: &std::sync::Arc<fastn_account::AccountManager>,
) -> fastn_router::HttpResponse {
    // Check if this endpoint belongs to an account
    if let Ok(account) = account_manager.find_account_by_alias(our_endpoint).await {
        return account
            .route_http(request, requester)
            .await
            .unwrap_or_else(|e| {
                fastn_router::HttpResponse::internal_error(format!("Account routing error: {e}"))
            });
    }

    // TODO: Check if this endpoint belongs to the rig
    // For now, return not found for rig endpoints accessed via P2P

    fastn_router::HttpResponse::not_found(format!(
        "Endpoint {} not found or not accessible via P2P",
        our_endpoint
    ))
}
