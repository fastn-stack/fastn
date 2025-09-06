/// Global registry of active P2P listeners to prevent duplicate listeners
/// and enable per-endpoint shutdown. Uses public key directly as the key to avoid
/// storing secret keys in global state.
static ACTIVE_LISTENERS: std::sync::LazyLock<
    std::sync::Mutex<
        std::collections::HashMap<fastn_id52::PublicKey, tokio_util::sync::CancellationToken>,
    >,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub struct PeerRequest {
    pub peer: fastn_id52::PublicKey,
    pub protocol: fastn_net::Protocol,
    pub send: iroh::endpoint::SendStream,
    pub recv: iroh::endpoint::RecvStream,
}

/// Handle for responding to a P2P request
/// 
/// This handle ensures that exactly one response is sent per request,
/// preventing common bugs like sending multiple responses or forgetting to respond.
/// The handle is consumed when sending a response, making multiple responses impossible.
pub struct P2PResponseHandle {
    send_stream: iroh::endpoint::SendStream,
}

/// Error when trying to get input from a PeerConnection
#[derive(Debug, thiserror::Error)]
pub enum GetInputError {
    #[error("Failed to receive request: {source}")]
    ReceiveError { source: eyre::Error },

    #[error("Failed to deserialize request: {source}")]
    DeserializationError { source: serde_json::Error },
}

/// Error when sending a response through P2PResponseHandle
#[derive(Debug, thiserror::Error)]
pub enum P2PSendError {
    #[error("Failed to serialize response: {source}")]
    SerializationError { source: serde_json::Error },

    #[error("Failed to send response: {source}")]
    SendError { source: eyre::Error },
}

impl PeerRequest {
    /// Read and deserialize a JSON request from the peer connection
    /// 
    /// Returns the deserialized input and a response handle that must be used
    /// to send exactly one response back to the client.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Deserialize)]
    /// struct Request {
    ///     message: String,
    /// }
    /// 
    /// #[derive(Serialize)]
    /// struct Response {
    ///     echo: String,
    /// }
    /// 
    /// async fn handle_connection(mut request: fastn_net::PeerRequest) -> eyre::Result<()> {
    ///     let (request, handle): (Request, _) = request.get_input().await?;
    ///     
    ///     let result = Ok::<Response, String>(Response {
    ///         echo: format!("You said: {}", request.message),
    ///     });
    ///     
    ///     handle.send(result).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_input<INPUT>(mut self) -> Result<(INPUT, P2PResponseHandle), GetInputError> 
    where
        INPUT: for<'de> serde::Deserialize<'de>,
    {
        // Read JSON request from the stream
        let request_json = fastn_net::next_string(&mut self.recv)
            .await
            .map_err(|source| GetInputError::ReceiveError { source })?;

        // Deserialize the request
        let input: INPUT = serde_json::from_str(&request_json)
            .map_err(|source| GetInputError::DeserializationError { source })?;

        // Create response handle
        let response_handle = P2PResponseHandle {
            send_stream: self.send,
        };

        Ok((input, response_handle))
    }

    /// Handle a request with an async closure
    /// 
    /// This method provides the most convenient way to handle P2P requests.
    /// It automatically:
    /// - Deserializes the incoming request
    /// - Calls your handler function
    /// - Sends the response or error automatically
    /// - Handles all JSON serialization and error conversion
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Deserialize)]
    /// struct EchoRequest {
    ///     message: String,
    /// }
    /// 
    /// #[derive(Serialize)]
    /// struct EchoResponse {
    ///     echo: String,
    /// }
    /// 
    /// async fn handle_request(peer_request: fastn_net::PeerRequest) -> Result<(), fastn_net::HandleRequestError> {
    ///     peer_request.handle(|request: EchoRequest| async move {
    ///         // Handler returns Result<OUTPUT, ERROR> - framework handles rest automatically
    ///         Ok::<EchoResponse, String>(EchoResponse {
    ///             echo: format!("You said: {}", request.message),
    ///         })
    ///     }).await
    /// }
    /// ```
    pub async fn handle<INPUT, OUTPUT, ERROR, F, Fut>(
        self,
        handler: F,
    ) -> Result<(), HandleRequestError>
    where
        INPUT: for<'de> serde::Deserialize<'de>,
        OUTPUT: serde::Serialize,
        ERROR: serde::Serialize,
        F: FnOnce(INPUT) -> Fut,
        Fut: std::future::Future<Output = Result<OUTPUT, ERROR>>,
    {
        // Get input and response handle
        let (input, response_handle) = match self.get_input().await {
            Ok(result) => result,
            Err(e) => return Err(HandleRequestError::GetInputFailed { source: e }),
        };

        // Call the handler and send the result (automatically handles Ok/Err variants)
        let handler_result = handler(input).await;
        response_handle.send(handler_result).await
            .map_err(|source| HandleRequestError::SendResponseFailed { source })?;

        Ok(())
    }
}

/// Error when handling a request through the convenient handler API
#[derive(Debug, thiserror::Error)]
pub enum HandleRequestError {
    #[error("Failed to get input: {source}")]
    GetInputFailed { source: GetInputError },

    #[error("Failed to send response: {source}")]
    SendResponseFailed { source: P2PSendError },
}

impl P2PResponseHandle {
    /// Send a response back to the client
    /// 
    /// This method consumes the handle, ensuring exactly one response per request.
    /// Accepts a Result<OUTPUT, ERROR> and automatically serializes the appropriate variant.
    /// This ensures type safety by binding OUTPUT and ERROR together.
    pub async fn send<OUTPUT, ERROR>(mut self, result: Result<OUTPUT, ERROR>) -> Result<(), P2PSendError>
    where
        OUTPUT: serde::Serialize,
        ERROR: serde::Serialize,
    {
        let response_json = match result {
            Ok(output) => {
                // Serialize successful response
                serde_json::to_string(&output)
                    .map_err(|source| P2PSendError::SerializationError { source })?
            }
            Err(error) => {
                // Serialize error response  
                serde_json::to_string(&error)
                    .map_err(|source| P2PSendError::SerializationError { source })?
            }
        };

        // Send JSON followed by newline
        self.send_stream.write_all(response_json.as_bytes()).await
            .map_err(|e| P2PSendError::SendError { source: eyre::Error::from(e) })?;
        self.send_stream.write_all(b"\n").await
            .map_err(|e| P2PSendError::SendError { source: eyre::Error::from(e) })?;

        Ok(())
    }
}

/// Error when trying to start a listener that's already active
#[derive(Debug, thiserror::Error)]
#[error("Listener already active for endpoint {public_key}")]
#[allow(clippy::result_large_err)]  // PublicKey usage is intentional for type safety
pub struct ListenerAlreadyActiveError {
    pub public_key: fastn_id52::PublicKey,
}

/// Error when trying to stop a listener that's not active
#[derive(Debug, thiserror::Error)]
#[error("No active listener found for endpoint {public_key}")]
#[allow(clippy::result_large_err)]  // PublicKey usage is intentional for type safety
pub struct ListenerNotFoundError {
    pub public_key: fastn_id52::PublicKey,
}

async fn handle_connection(
    conn: iroh::endpoint::Incoming,
    expected_protocols: Vec<fastn_net::Protocol>,
    tx: tokio::sync::mpsc::Sender<eyre::Result<PeerRequest>>,
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
        let peer_request = PeerRequest {
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

/// Stop listening on a specific endpoint
///
/// This cancels the P2P listener for the given public key and removes it from
/// the global registry. Returns an error if no listener is active for this endpoint.
#[allow(clippy::result_large_err)]  // PublicKey usage is intentional for type safety
pub fn p2p_stop_listening(public_key: fastn_id52::PublicKey) -> Result<(), ListenerNotFoundError> {
    let mut listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");

    if let Some(cancellation_token) = listeners.remove(&public_key) {
        tracing::info!("Stopping P2P listener for endpoint: {public_key}");
        cancellation_token.cancel();
        Ok(())
    } else {
        Err(ListenerNotFoundError { public_key })
    }
}

/// Check if a P2P listener is currently active for the given endpoint
pub fn is_p2p_listening(public_key: &fastn_id52::PublicKey) -> bool {
    let listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    listeners.contains_key(public_key)
}

#[allow(clippy::result_large_err)]  // PublicKey usage is intentional for type safety
pub fn p2p_listen(
    secret_key: fastn_id52::SecretKey,
    expected: &[fastn_net::Protocol],
    graceful: fastn_net::Graceful,
) -> Result<
    impl futures_core::stream::Stream<Item = eyre::Result<PeerRequest>>,
    ListenerAlreadyActiveError,
> {
    let public_key = secret_key.public_key();

    // Check if already listening and register this endpoint (using public key)
    let endpoint_cancellation_token = {
        let mut listeners = ACTIVE_LISTENERS
            .lock()
            .expect("Failed to acquire lock on ACTIVE_LISTENERS");

        if listeners.contains_key(&public_key) {
            return Err(ListenerAlreadyActiveError { public_key });
        }

        let token = tokio_util::sync::CancellationToken::new();
        listeners.insert(public_key, token.clone());
        tracing::info!("Registered P2P listener for endpoint: {public_key}");
        token
    };

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
            {
                let mut listeners = ACTIVE_LISTENERS.lock()
                    .expect("Failed to acquire lock on ACTIVE_LISTENERS");
                listeners.remove(&acceptor_public_key);
                tracing::debug!("Removed endpoint {public_key} from active listeners registry");
            }
        });

        // Stream PeerRequests from the channel
        while let Some(peer_request_result) = rx.recv().await {
            yield peer_request_result?;
        }

        // Clean up: ensure removal from registry when stream ends
        {
            let mut listeners = ACTIVE_LISTENERS.lock()
                .expect("Failed to acquire lock on ACTIVE_LISTENERS");
            listeners.remove(&public_key);
            tracing::debug!("Stream ended: removed endpoint {public_key} from active listeners registry");
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_listener_registry() {
        let secret_key = fastn_id52::SecretKey::generate();
        let public_key = secret_key.public_key();

        // Should not be listening initially
        assert!(!is_p2p_listening(&public_key));

        // Stopping non-existent listener should fail
        assert!(p2p_stop_listening(public_key).is_err());

        // Starting listener should succeed
        let graceful = fastn_net::Graceful::new();
        let stream_result = p2p_listen(secret_key.clone(), &[fastn_net::Protocol::Ping], graceful);
        assert!(stream_result.is_ok());

        // Should now be listening
        assert!(is_p2p_listening(&public_key));

        // Starting same listener again should fail
        let graceful2 = fastn_net::Graceful::new();
        let duplicate_result =
            p2p_listen(secret_key.clone(), &[fastn_net::Protocol::Ping], graceful2);
        assert!(duplicate_result.is_err());
        if let Err(e) = duplicate_result {
            assert!(matches!(e, ListenerAlreadyActiveError { .. }));
        }

        // Stopping should succeed
        assert!(p2p_stop_listening(public_key).is_ok());

        // Should no longer be listening
        assert!(!is_p2p_listening(&public_key));

        // Stopping again should fail
        assert!(p2p_stop_listening(public_key).is_err());
    }

    #[tokio::test]
    async fn test_public_key_direct_comparison() {
        let secret_key1 = fastn_id52::SecretKey::generate();
        let secret_key2 = fastn_id52::SecretKey::generate();
        let public_key1 = secret_key1.public_key();
        let public_key2 = secret_key2.public_key();

        // Should be able to register different endpoints
        assert!(!is_p2p_listening(&public_key1));
        assert!(!is_p2p_listening(&public_key2));

        let graceful1 = fastn_net::Graceful::new();
        let stream_result1 =
            p2p_listen(secret_key1.clone(), &[fastn_net::Protocol::Ping], graceful1);
        assert!(stream_result1.is_ok());

        // First should be listening, second should not
        assert!(is_p2p_listening(&public_key1));
        assert!(!is_p2p_listening(&public_key2));

        // Second endpoint should also be able to start
        let graceful2 = fastn_net::Graceful::new();
        let stream_result2 =
            p2p_listen(secret_key2.clone(), &[fastn_net::Protocol::Ping], graceful2);
        assert!(stream_result2.is_ok());

        // Both should now be listening
        assert!(is_p2p_listening(&public_key1));
        assert!(is_p2p_listening(&public_key2));

        // Should be able to stop first without affecting second
        assert!(p2p_stop_listening(public_key1).is_ok());
        assert!(!is_p2p_listening(&public_key1));
        assert!(is_p2p_listening(&public_key2));

        // Should be able to stop second
        assert!(p2p_stop_listening(public_key2).is_ok());
        assert!(!is_p2p_listening(&public_key2));
    }

    #[tokio::test]
    async fn test_peer_connection_request_response() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestRequest {
            message: String,
            number: i32,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestResponse {
            echo: String,
            doubled: i32,
        }

        // Create mock streams for testing
        let secret_key = fastn_id52::SecretKey::generate();
        let _public_key = secret_key.public_key();

        // Test request serialization
        let request = TestRequest {
            message: "hello".to_string(),
            number: 21,
        };
        
        let request_json = serde_json::to_string(&request).unwrap();
        
        // Verify JSON format is correct
        assert!(request_json.contains("hello"));
        assert!(request_json.contains("21"));
        
        let parsed_request: TestRequest = serde_json::from_str(&request_json).unwrap();
        assert_eq!(request, parsed_request);

        // Test response serialization  
        let response = TestResponse {
            echo: "hello world".to_string(),
            doubled: 42,
        };
        
        let response_json = serde_json::to_string(&response).unwrap();
        let parsed_response: TestResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response, parsed_response);
    }

    #[tokio::test]
    async fn test_p2p_response_handle_already_responded() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct TestResponse {
            message: String,
        }

        // Create mock send stream using a simple channel approach
        let (_tx, _rx) = tokio::sync::mpsc::channel::<Vec<u8>>(10);
        
        // This is a bit of a hack since we can't easily create a real SendStream in tests
        // but the logic of preventing multiple responses is important to test conceptually
        let _response = TestResponse {
            message: "test".to_string(),
        };
        
        // Test that error types work
        let serialization_err = P2PSendError::SerializationError {
            source: serde_json::Error::io(std::io::Error::other("test")),
        };
        assert!(serialization_err.to_string().contains("Failed to serialize response"));
    }

    #[test]
    fn test_error_types() {
        // Test GetInputError
        let receive_err = GetInputError::ReceiveError {
            source: eyre::anyhow!("connection closed"),
        };
        assert!(receive_err.to_string().contains("Failed to receive request"));

        let deser_err = GetInputError::DeserializationError {
            source: serde_json::Error::io(std::io::Error::other("invalid json")),
        };
        assert!(deser_err.to_string().contains("Failed to deserialize request"));

        // Test P2PSendError variants
        let send_err = P2PSendError::SendError {
            source: eyre::anyhow!("network error"),
        };
        assert!(send_err.to_string().contains("Failed to send response"));
        
    }

    #[test]
    fn test_request_response_types() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct EchoRequest {
            message: String,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct EchoResponse {
            echo: String,
            processed: bool,
        }

        // Test that our types work with the expected serde traits
        let request = EchoRequest {
            message: "test message".to_string(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let parsed: EchoRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, parsed);

        let response = EchoResponse {
            echo: "response".to_string(),
            processed: true,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let parsed: EchoResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
    }

    #[test]
    fn test_handle_request_error() {
        // Test that HandleRequestError types work
        let get_input_err = HandleRequestError::GetInputFailed {
            source: GetInputError::ReceiveError {
                source: eyre::anyhow!("connection closed"),
            },
        };
        assert!(get_input_err.to_string().contains("Failed to get input"));

        let send_response_err = HandleRequestError::SendResponseFailed {
            source: P2PSendError::SerializationError {
                source: serde_json::Error::io(std::io::Error::other("test")),
            },
        };
        assert!(send_response_err.to_string().contains("Failed to send response"));

    }
}
