/// Error type for call function
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("Failed to establish P2P stream: {source}")]
    Endpoint { source: eyre::Error },

    #[error("Failed to establish P2P stream: {source}")]
    Stream { source: eyre::Error },

    #[error("Failed to serialize request: {source}")]
    Serialization { source: serde_json::Error },

    #[error("Failed to send request: {source}")]
    Send { source: eyre::Error },

    #[error("Failed to receive response: {source}")]
    Receive { source: eyre::Error },

    #[error("Failed to deserialize response: {source}")]
    Deserialization { source: serde_json::Error },
}

/// Make a P2P call using global singletons
///
/// This is the main function end users should use. It automatically uses
/// the global connection pool and graceful shutdown coordinator.
///
/// # Example
///
/// ```rust,ignore
/// let result: Result<MyResponse, MyError> = fastn_p2p::call(
///     secret_key, &target, protocol, request
/// ).await?;
/// ```
pub async fn call<P, INPUT, OUTPUT, ERROR>(
    sender: fastn_id52::SecretKey,
    target: &fastn_id52::PublicKey,
    protocol: P,
    input: INPUT,
) -> Result<Result<OUTPUT, ERROR>, CallError>
where
    P: serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Clone
        + PartialEq
        + std::fmt::Display
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
    INPUT: serde::Serialize,
    OUTPUT: for<'de> serde::Deserialize<'de>,
    ERROR: for<'de> serde::Deserialize<'de>,
{
    // Delegate to coordination module which has strict singleton access control
    crate::coordination::internal_call(sender, target, protocol, input).await
}
