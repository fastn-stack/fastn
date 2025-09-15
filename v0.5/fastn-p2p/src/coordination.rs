//! Task coordination helpers with strict singleton access control
//!
//! This module encapsulates ALL graceful access and fastn_net::get_stream usage
//! to ensure complete singleton access control.

use crate::client::CallError;

/// Global graceful shutdown coordinator (private to this module ONLY)
static GRACEFUL: std::sync::LazyLock<fastn_net::Graceful> =
    std::sync::LazyLock::new(fastn_net::Graceful::new);

/// Spawn a task with proper graceful shutdown coordination
///
/// This is the ONLY way to spawn tasks - ensures proper shutdown tracking.
pub fn spawn<F>(task: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    GRACEFUL.spawn(task)
}

/// Check for graceful shutdown signal
///
/// This is the ONLY way to check for cancellation.
pub async fn cancelled() {
    GRACEFUL.cancelled().await
}

/// Trigger graceful shutdown of all spawned tasks
///
/// This is used by the main macro to initiate shutdown after user main completes
/// or when signal handlers are triggered.
pub async fn shutdown() -> eyre::Result<()> {
    GRACEFUL.shutdown().await
}

/// Internal P2P call implementation with localized graceful access
///
/// This function contains the ONLY internal access to graceful for fastn_net compatibility.
/// All P2P calls go through this function to maintain singleton access control.
pub async fn internal_call<P, INPUT, OUTPUT, ERROR>(
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
    // Convert user protocol to fastn_net::Protocol::Generic
    let json_value =
        serde_json::to_value(&protocol).map_err(|e| CallError::Serialization { source: e })?;
    let net_protocol = fastn_net::Protocol::Generic(json_value);

    // Get endpoint for the sender
    let endpoint = fastn_net::get_endpoint(sender)
        .await
        .map_err(|source| CallError::Endpoint { source })?;

    // Establish P2P stream using singletons (graceful access localized to this module)
    let (mut send_stream, mut recv_stream) = fastn_net::get_stream(
        endpoint,
        net_protocol.into(),
        target,
        crate::pool(),
        GRACEFUL.clone(), // ONLY access to graceful singleton in entire codebase
    )
    .await
    .map_err(|source| CallError::Stream { source })?;

    // Serialize and send request
    let request_json =
        serde_json::to_string(&input).map_err(|source| CallError::Serialization { source })?;

    // Send JSON followed by newline
    send_stream
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| CallError::Send {
            source: eyre::Error::from(e),
        })?;
    send_stream
        .write_all(b"\n")
        .await
        .map_err(|e| CallError::Send {
            source: eyre::Error::from(e),
        })?;

    // Receive and deserialize response
    let response_json = fastn_net::next_string(&mut recv_stream)
        .await
        .map_err(|source| CallError::Receive { source })?;

    // Try to deserialize as success response first
    if let Ok(success_response) = serde_json::from_str::<OUTPUT>(&response_json) {
        return Ok(Ok(success_response));
    }

    // If that fails, try to deserialize as ERROR type
    if let Ok(error_response) = serde_json::from_str::<ERROR>(&response_json) {
        return Ok(Err(error_response));
    }

    // If both fail, it's a deserialization error
    Err(CallError::Deserialization {
        source: serde_json::Error::io(std::io::Error::other(format!(
            "Response doesn't match expected OUTPUT or ERROR types: {response_json}"
        ))),
    })
}
