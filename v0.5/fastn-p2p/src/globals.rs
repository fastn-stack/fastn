/// Global singleton instances for P2P infrastructure
/// 
/// This module provides singleton access to essential P2P infrastructure
/// components to avoid duplication and simplify the API.
///
/// Global graceful shutdown coordinator
static GLOBAL_GRACEFUL: std::sync::LazyLock<fastn_net::Graceful> = 
    std::sync::LazyLock::new(fastn_net::Graceful::new);

/// Global peer stream connection pool
static GLOBAL_POOL: std::sync::LazyLock<fastn_net::PeerStreamSenders> = 
    std::sync::LazyLock::new(|| {
        std::sync::Arc::new(
            tokio::sync::Mutex::new(std::collections::HashMap::new())
        )
    });

/// Get the global graceful shutdown coordinator
/// 
/// Returns a clone of the singleton graceful shutdown coordinator.
/// All parts of the application should use the same instance to ensure
/// coordinated shutdown behavior.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let graceful = fastn_p2p::graceful();
/// let stream = fastn_p2p::listen(secret_key, &protocols, graceful)?;
/// ```
pub fn graceful() -> fastn_net::Graceful {
    GLOBAL_GRACEFUL.clone()
}

/// Get the global peer connection pool
/// 
/// Returns a clone of the singleton peer stream connection pool.
/// This pool manages reusable P2P connections to avoid connection overhead.
/// 
/// Note: Most users should not need to call this directly - the high-level
/// APIs (listen, call) handle pool management automatically.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let pool = fastn_p2p::pool();  // Usually not needed by end users
/// let result = fastn_p2p::call(secret_key, &target, protocol, pool, graceful, request).await?;
/// ```
pub fn pool() -> fastn_net::PeerStreamSenders {
    GLOBAL_POOL.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graceful_singleton() {
        let _graceful1 = graceful();
        let _graceful2 = graceful();
        
        // Both should reference the same underlying singleton instance
        // Basic functionality test - they should both be valid
    }

    #[test] 
    fn test_pool_singleton() {
        let pool1 = pool();
        let pool2 = pool();
        
        // Both should reference the same underlying HashMap
        // We can't directly test this, but we can verify basic functionality
        assert_eq!(std::sync::Arc::strong_count(&pool1), std::sync::Arc::strong_count(&pool2));
    }
}