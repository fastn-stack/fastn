/// Error type for call function
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("Failed to establish P2P stream: {source}")]
    EndpointError { source: eyre::Error },

    #[error("Failed to establish P2P stream: {source}")]
    StreamError { source: eyre::Error },

    #[error("Failed to serialize request: {source}")]
    SerializationError { source: serde_json::Error },

    #[error("Failed to send request: {source}")]
    SendError { source: eyre::Error },

    #[error("Failed to receive response: {source}")]
    ReceiveError { source: eyre::Error },

    #[error("Failed to deserialize response: {source}")]
    DeserializationError { source: serde_json::Error },
}

/// High-level P2P request-response function using JSON serialization
///
/// This function provides a convenient way to make P2P calls without dealing with
/// streams directly. It internally uses get_stream() but handles all the serialization,
/// sending, receiving, and deserialization automatically.
///
/// # Type Parameters
///
/// * `INPUT` - The request type (must implement Serialize)
/// * `OUTPUT` - The response type (must implement DeserializeOwned)
/// * `ERROR` - The error type (must implement DeserializeOwned)
///
/// # Arguments
///
/// * `sender_secret_key` - Secret key of the calling endpoint
/// * `target_public_key` - Public key of the target endpoint
/// * `protocol` - Protocol to use for the communication
/// * `peer_stream_senders` - Connection pool for stream management
/// * `graceful` - Graceful shutdown handle
/// * `input` - The request data to send
///
/// # Example
///
/// ```rust,no_run
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize)]
/// struct PingRequest {
///     message: String,
/// }
///
/// #[derive(Deserialize)]
/// struct PingResponse {
///     echo: String,
///     timestamp: u64,
/// }
///
/// async fn example() -> eyre::Result<()> {
///     let sender_secret_key = fastn_id52::SecretKey::generate();
///     let target_public_key = fastn_id52::SecretKey::generate().public_key();
///     let peer_stream_senders = std::sync::Arc::new(
///         tokio::sync::Mutex::new(std::collections::HashMap::new())
///     );
///     let graceful = fastn_net::Graceful::new();
///     
///     let request = PingRequest {
///         message: "Hello!".to_string(),
///     };
///
///     let result: Result<PingResponse, String> = fastn_p2p::call(
///         sender_secret_key,
///         &target_public_key,
///         fastn_net::Protocol::Ping,
///         peer_stream_senders,
///         graceful,
///         request,
///     ).await?;
///     
///     match result {
///         Ok(response) => println!("Got response: {}", response.echo),
///         Err(error) => println!("Got error: {}", error),
///     }
///     Ok(())
/// }
/// ```
pub async fn call<INPUT, OUTPUT, ERROR>(
    sender: fastn_id52::SecretKey,
    target: &fastn_id52::PublicKey,
    protocol: fastn_net::Protocol,
    peer_stream_senders: fastn_net::PeerStreamSenders,
    graceful: fastn_net::Graceful,
    input: INPUT,
) -> Result<Result<OUTPUT, ERROR>, CallError>
where
    INPUT: serde::Serialize,
    OUTPUT: for<'de> serde::Deserialize<'de>,
    ERROR: for<'de> serde::Deserialize<'de>,
{
    // Get endpoint for the sender
    let endpoint = fastn_net::get_endpoint(sender)
        .await
        .map_err(|source| CallError::EndpointError { source })?;

    // Establish P2P stream
    let (mut send_stream, mut recv_stream) = fastn_net::get_stream(
        endpoint,
        protocol.into(),
        target,
        peer_stream_senders,
        graceful,
    )
    .await
    .map_err(|source| CallError::StreamError { source })?;

    // Serialize and send request
    let request_json = serde_json::to_string(&input)
        .map_err(|source| CallError::SerializationError { source })?;

    // Send JSON followed by newline
    send_stream
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| CallError::SendError {
            source: eyre::Error::from(e),
        })?;
    send_stream
        .write_all(b"\n")
        .await
        .map_err(|e| CallError::SendError {
            source: eyre::Error::from(e),
        })?;

    // Receive and deserialize response
    let response_json = fastn_net::next_string(&mut recv_stream)
        .await
        .map_err(|source| CallError::ReceiveError { source })?;

    // Try to deserialize as success response first
    if let Ok(success_response) = serde_json::from_str::<OUTPUT>(&response_json) {
        return Ok(Ok(success_response));
    }

    // If that fails, try to deserialize as ERROR type
    if let Ok(error_response) = serde_json::from_str::<ERROR>(&response_json) {
        return Ok(Err(error_response));
    }

    // If both fail, it's a deserialization error
    Err(CallError::DeserializationError {
        source: serde_json::Error::io(std::io::Error::other(
            format!("Response doesn't match expected OUTPUT or ERROR types: {}", response_json)
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug, PartialEq)]
    struct TestRequest {
        message: String,
        id: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestResponse {
        echo: String,
        processed_id: u32,
    }

    #[tokio::test]
    async fn test_p2p_call_error_types() {
        // Test that error types are properly defined and can be created
        let io_error = std::io::Error::other("test");
        let serialization_err = CallError::SerializationError {
            source: serde_json::Error::io(io_error),
        };

        assert!(
            serialization_err
                .to_string()
                .contains("Failed to serialize request")
        );

        let io_error2 = std::io::Error::other("test");
        let deserialization_err = CallError::DeserializationError {
            source: serde_json::Error::io(io_error2),
        };

        assert!(
            deserialization_err
                .to_string()
                .contains("Failed to deserialize response")
        );
    }

    #[test]
    fn test_request_response_serialization() {
        let request = TestRequest {
            message: "test".to_string(),
            id: 42,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("42"));

        let response = TestResponse {
            echo: "test response".to_string(),
            processed_id: 43,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TestResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}