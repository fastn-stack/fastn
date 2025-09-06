
pub struct PeerRequest {
    pub peer: fastn_id52::PublicKey,
    pub protocol: fastn_net::Protocol,
    pub send: iroh::endpoint::SendStream,
    pub recv: iroh::endpoint::RecvStream,
}

/// Error when trying to get input from a PeerRequest
#[derive(Debug, thiserror::Error)]
pub enum GetInputError {
    #[error("Failed to receive request: {source}")]
    ReceiveError { source: eyre::Error },

    #[error("Failed to deserialize request: {source}")]
    DeserializationError { source: serde_json::Error },
}

/// Error when handling a request through the convenient handler API
#[derive(Debug, thiserror::Error)]
pub enum HandleRequestError {
    #[error("Failed to get input: {source}")]
    GetInputFailed { source: GetInputError },

    #[error("Failed to send response: {source}")]
    SendResponseFailed { source: fastn_p2p::SendError },
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
    /// async fn handle_connection(mut request: fastn_p2p::Request) -> eyre::Result<()> {
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
    pub async fn get_input<INPUT>(mut self) -> Result<(INPUT, fastn_p2p::ResponseHandle), GetInputError> 
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
        let response_handle = fastn_p2p::server::handle::P2PResponseHandle::new(self.send);

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
    /// async fn handle_request(peer_request: fastn_p2p::Request) -> Result<(), fastn_p2p::HandleRequestError> {
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