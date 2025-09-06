/// Handle for responding to a P2P request
/// 
/// This handle ensures that exactly one response is sent per request,
/// preventing common bugs like sending multiple responses or forgetting to respond.
/// The handle is consumed when sending a response, making multiple responses impossible.
pub struct P2PResponseHandle {
    send_stream: iroh::endpoint::SendStream,
}

/// Error when sending a response through P2PResponseHandle
#[derive(Debug, thiserror::Error)]
pub enum P2PSendError {
    #[error("Failed to serialize response: {source}")]
    SerializationError { source: serde_json::Error },

    #[error("Failed to send response: {source}")]
    SendError { source: eyre::Error },
}

impl P2PResponseHandle {
    /// Create a new response handle from a send stream
    pub(crate) fn new(send_stream: iroh::endpoint::SendStream) -> Self {
        Self { send_stream }
    }

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