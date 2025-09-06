/// Global registry of active P2P listeners to prevent duplicate listeners
/// and enable per-endpoint shutdown. Uses public key directly as the key to avoid
/// storing secret keys in global state.
static ACTIVE_LISTENERS: std::sync::LazyLock<
    std::sync::Mutex<
        std::collections::HashMap<fastn_id52::PublicKey, tokio_util::sync::CancellationToken>,
    >,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

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

/// Register a new listener in the global registry
/// 
/// Returns a cancellation token for the listener, or an error if already active.
#[allow(clippy::result_large_err)]  // PublicKey usage is intentional for type safety
pub(super) fn register_listener(
    public_key: fastn_id52::PublicKey,
) -> Result<tokio_util::sync::CancellationToken, ListenerAlreadyActiveError> {
    let mut listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");

    if listeners.contains_key(&public_key) {
        return Err(ListenerAlreadyActiveError { public_key });
    }

    let token = tokio_util::sync::CancellationToken::new();
    listeners.insert(public_key, token.clone());
    tracing::info!("Registered P2P listener for endpoint: {public_key}");
    Ok(token)
}

/// Remove a listener from the global registry  
/// 
/// This is called automatically when listeners shut down.
pub(super) fn unregister_listener(public_key: &fastn_id52::PublicKey) {
    let mut listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    
    listeners.remove(public_key);
    tracing::debug!("Removed endpoint {public_key} from active listeners registry");
}

/// Stop listening on a specific endpoint
///
/// This cancels the P2P listener for the given public key and removes it from
/// the global registry. Returns an error if no listener is active for this endpoint.
#[allow(clippy::result_large_err)]  // PublicKey usage is intentional for type safety
pub fn stop_listening(public_key: fastn_id52::PublicKey) -> Result<(), ListenerNotFoundError> {
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
pub fn is_listening(public_key: &fastn_id52::PublicKey) -> bool {
    let listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    listeners.contains_key(public_key)
}

/// Get the number of currently active listeners
pub fn active_listener_count() -> usize {
    let listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    listeners.len()
}

/// Get a list of all currently active listener public keys
pub fn active_listeners() -> Vec<fastn_id52::PublicKey> {
    let listeners = ACTIVE_LISTENERS
        .lock()
        .expect("Failed to acquire lock on ACTIVE_LISTENERS");
    listeners.keys().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_management() {
        let public_key1 = fastn_id52::SecretKey::generate().public_key();
        let public_key2 = fastn_id52::SecretKey::generate().public_key();

        // Initially no listeners
        assert_eq!(active_listener_count(), 0);
        assert!(!is_listening(&public_key1));
        assert!(!is_listening(&public_key2));
        assert!(active_listeners().is_empty());

        // Register first listener
        let token1 = register_listener(public_key1).unwrap();
        assert_eq!(active_listener_count(), 1);
        assert!(is_listening(&public_key1));
        assert!(!is_listening(&public_key2));
        assert_eq!(active_listeners(), vec![public_key1]);

        // Register second listener
        let _token2 = register_listener(public_key2).unwrap();
        assert_eq!(active_listener_count(), 2);
        assert!(is_listening(&public_key1));
        assert!(is_listening(&public_key2));

        let listeners = active_listeners();
        assert_eq!(listeners.len(), 2);
        assert!(listeners.contains(&public_key1));
        assert!(listeners.contains(&public_key2));

        // Try to register duplicate
        assert!(register_listener(public_key1).is_err());
        assert_eq!(active_listener_count(), 2);

        // Stop first listener
        assert!(stop_listening(public_key1).is_ok());
        assert_eq!(active_listener_count(), 1);
        assert!(!is_listening(&public_key1));
        assert!(is_listening(&public_key2));

        // Try to stop non-existent
        assert!(stop_listening(public_key1).is_err());

        // Clean up
        token1.cancel(); // This won't affect registry since already removed
        assert_eq!(active_listener_count(), 1);
        
        unregister_listener(&public_key2);
        assert_eq!(active_listener_count(), 0);
        assert!(active_listeners().is_empty());
    }
}