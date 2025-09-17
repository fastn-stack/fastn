/// Server-side streaming session (handles both RPC and streaming)
pub struct Session<PROTOCOL> {
    /// Protocol negotiated with client
    pub protocol: PROTOCOL,
    /// Stream to client (stdout)
    pub send: iroh::endpoint::SendStream,
    /// Stream from client (stdin)  
    pub recv: iroh::endpoint::RecvStream,
    /// Peer's public key
    peer: fastn_id52::PublicKey,
    /// Context for this session (integration with fastn-context)
    context: std::sync::Arc<fastn_context::Context>,
}

impl<PROTOCOL> Session<PROTOCOL> {
    /// Get the peer's public key
    pub fn peer(&self) -> &fastn_id52::PublicKey {
        &self.peer
    }
    
    /// Get the context for this session
    pub fn context(&self) -> &std::sync::Arc<fastn_context::Context> {
        &self.context
    }
    
    /// Convert to Request for RPC handling (consumes Session)
    pub fn into_request(self) -> super::request::Request<PROTOCOL> {
        // TODO: Convert Session to Request for RPC pattern
        todo!("Convert Session to Request for RPC handling")
    }
    
    /// Open unidirectional stream back to client (e.g., stderr)
    pub async fn open_uni(&mut self) -> Result<iroh::endpoint::SendStream, crate::client::ConnectionError> {
        // TODO: Open unidirectional stream to client
        todo!("Open unidirectional stream back to client")
    }
    
    /// Open bidirectional stream back to client
    pub async fn open_bi(&mut self) -> Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream), crate::client::ConnectionError> {
        // TODO: Open bidirectional stream to client
        todo!("Open bidirectional stream back to client")
    }
}

/// Create a new Session (used internally by listener)
pub(crate) fn create_session<PROTOCOL>(
    protocol: PROTOCOL,
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
    peer: fastn_id52::PublicKey,
    parent_context: &std::sync::Arc<fastn_context::Context>,
) -> Session<PROTOCOL> {
    // Use parent context for now (can create child context later)
    Session {
        protocol,
        send,
        recv,
        peer,
        context: parent_context.clone(),
    }
}