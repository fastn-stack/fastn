use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use tokio_util::sync::CancellationToken;

/// Global registry of active P2P listeners to prevent duplicate listeners
/// and enable per-endpoint shutdown. Uses public key directly as the key to avoid
/// storing secret keys in global state.
static ACTIVE_LISTENERS: LazyLock<Mutex<HashMap<fastn_id52::PublicKey, CancellationToken>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct PeerConnection {
    pub peer: fastn_id52::PublicKey,
    pub protocol: fastn_net::Protocol,
    pub send: iroh::endpoint::SendStream,
    pub recv: iroh::endpoint::RecvStream,
}

/// Error when trying to start a listener that's already active
#[derive(Debug, thiserror::Error)]
#[error("Listener already active for endpoint {public_key_id52}")]
pub struct ListenerAlreadyActiveError {
    pub public_key_id52: String,
}

/// Error when trying to stop a listener that's not active
#[derive(Debug, thiserror::Error)]  
#[error("No active listener found for endpoint {public_key_id52}")]
pub struct ListenerNotFoundError {
    pub public_key_id52: String,
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
            peer: peer_key,
            protocol,
            send,
            recv,
        };
        
        if (tx.send(Ok(peer_connection)).await).is_err() {
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
pub fn p2p_stop_listening(public_key: &fastn_id52::PublicKey) -> Result<(), ListenerNotFoundError> {
    let public_key_id52 = public_key.id52();
    
    let mut listeners = ACTIVE_LISTENERS.lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    
    if let Some(cancellation_token) = listeners.remove(public_key) {
        tracing::info!("Stopping P2P listener for endpoint: {}", public_key_id52);
        cancellation_token.cancel();
        Ok(())
    } else {
        Err(ListenerNotFoundError { public_key_id52 })
    }
}

/// Check if a P2P listener is currently active for the given endpoint
pub fn is_p2p_listening(public_key: &fastn_id52::PublicKey) -> bool {
    let listeners = ACTIVE_LISTENERS.lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    listeners.contains_key(public_key)
}

pub fn p2p_listen(
    secret_key: fastn_id52::SecretKey,
    expected: &[fastn_net::Protocol],
    graceful: fastn_net::Graceful,
) -> Result<impl futures_core::stream::Stream<Item = eyre::Result<PeerConnection>>, ListenerAlreadyActiveError> {
    let public_key = secret_key.public_key();
    let public_key_id52 = public_key.id52();
    
    // Check if already listening and register this endpoint (using public key)
    let endpoint_cancellation_token = {
        let mut listeners = ACTIVE_LISTENERS.lock()
            .expect("Failed to acquire lock on ACTIVE_LISTENERS");
        
        if listeners.contains_key(&public_key) {
            return Err(ListenerAlreadyActiveError { public_key_id52 });
        }
        
        let token = CancellationToken::new();
        listeners.insert(public_key, token.clone());
        tracing::info!("Registered P2P listener for endpoint: {}", public_key_id52);
        token
    };
    
    let expected = expected.to_vec(); // Clone for move into async block
    
    Ok(async_stream::try_stream! {
        let endpoint = fastn_net::get_endpoint(secret_key.clone()).await?;
        
        // Channel to receive PeerConnections from spawned connection handlers
        // Using 0-capacity channel to ensure backpressure - each connection blocks 
        // until the stream consumer is ready to receive it
        let (tx, mut rx) = tokio::sync::mpsc::channel(0);

        // Spawn connection acceptor task
        let acceptor_tx = tx.clone();
        let acceptor_graceful = graceful.clone();
        let acceptor_endpoint_cancellation = endpoint_cancellation_token.clone();
        let acceptor_expected = expected.clone();
        let acceptor_public_key_id52 = public_key_id52.clone();
        let acceptor_public_key = public_key;
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    conn_result = endpoint.accept() => {
                        let conn = match conn_result {
                            Some(conn) => conn,
                            None => {
                                tracing::debug!("Endpoint {} closed", acceptor_public_key_id52);
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

                    // Handle global graceful shutdown
                    _ = acceptor_graceful.cancelled() => {
                        tracing::debug!("Global shutdown: stopping listener for endpoint {}", acceptor_public_key_id52);
                        break;
                    }
                    
                    // Handle endpoint-specific cancellation
                    _ = acceptor_endpoint_cancellation.cancelled() => {
                        tracing::debug!("Endpoint-specific shutdown: stopping listener for endpoint {}", acceptor_public_key_id52);
                        break;
                    }
                }
            }
            
            // Clean up: remove from global registry when task ends
            {
                let mut listeners = ACTIVE_LISTENERS.lock()
                    .expect("Failed to acquire lock on ACTIVE_LISTENERS");
                listeners.remove(&acceptor_public_key);
                tracing::debug!("Removed endpoint {} from active listeners registry", acceptor_public_key_id52);
            }
        });

        // Stream PeerConnections from the channel
        while let Some(peer_connection_result) = rx.recv().await {
            yield peer_connection_result?;
        }
        
        // Clean up: ensure removal from registry when stream ends
        {
            let mut listeners = ACTIVE_LISTENERS.lock()
                .expect("Failed to acquire lock on ACTIVE_LISTENERS");
            listeners.remove(&public_key);
            tracing::debug!("Stream ended: removed endpoint {} from active listeners registry", public_key_id52);
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
        assert!(p2p_stop_listening(&public_key).is_err());
        
        // Starting listener should succeed
        let graceful = fastn_net::Graceful::new();
        let stream_result = p2p_listen(
            secret_key.clone(), 
            &[fastn_net::Protocol::Ping], 
            graceful
        );
        assert!(stream_result.is_ok());
        
        // Should now be listening
        assert!(is_p2p_listening(&public_key));
        
        // Starting same listener again should fail
        let graceful2 = fastn_net::Graceful::new();
        let duplicate_result = p2p_listen(
            secret_key.clone(), 
            &[fastn_net::Protocol::Ping], 
            graceful2
        );
        assert!(duplicate_result.is_err());
        if let Err(e) = duplicate_result {
            assert!(matches!(e, ListenerAlreadyActiveError { .. }));
        }
        
        // Stopping should succeed
        assert!(p2p_stop_listening(&public_key).is_ok());
        
        // Should no longer be listening
        assert!(!is_p2p_listening(&public_key));
        
        // Stopping again should fail
        assert!(p2p_stop_listening(&public_key).is_err());
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
        let stream_result1 = p2p_listen(
            secret_key1.clone(),
            &[fastn_net::Protocol::Ping],
            graceful1,
        );
        assert!(stream_result1.is_ok());
        
        // First should be listening, second should not
        assert!(is_p2p_listening(&public_key1));
        assert!(!is_p2p_listening(&public_key2));
        
        // Second endpoint should also be able to start
        let graceful2 = fastn_net::Graceful::new();
        let stream_result2 = p2p_listen(
            secret_key2.clone(),
            &[fastn_net::Protocol::Ping], 
            graceful2,
        );
        assert!(stream_result2.is_ok());
        
        // Both should now be listening
        assert!(is_p2p_listening(&public_key1));
        assert!(is_p2p_listening(&public_key2));
        
        // Should be able to stop first without affecting second
        assert!(p2p_stop_listening(&public_key1).is_ok());
        assert!(!is_p2p_listening(&public_key1));
        assert!(is_p2p_listening(&public_key2));
        
        // Should be able to stop second
        assert!(p2p_stop_listening(&public_key2).is_ok());
        assert!(!is_p2p_listening(&public_key2));
    }
}
