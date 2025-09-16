/// P2P streaming connections with three-stream protocol
///
/// This module provides networking APIs for establishing streaming P2P 
/// connections with stdin/stdout/stderr-like semantics.

/// A P2P streaming session with main bidirectional stream and optional side channels
///
/// Simple abstraction: you get the main streams, create side channels on demand.
/// Protocol-specific communication happens over the available streams.
pub struct Session<PROTOCOL> {
    /// The protocol data negotiated for this session
    pub protocol: PROTOCOL,
    /// Input stream: Client → Server
    pub stdin: iroh::endpoint::RecvStream,
    /// Output stream: Server → Client 
    pub stdout: iroh::endpoint::SendStream,
    // TODO: Store underlying iroh connection for side channel creation
    // iroh_connection: iroh::Connection,
}

impl<PROTOCOL> Session<PROTOCOL> {
    /// Open unidirectional stream (matches iroh terminology)
    pub async fn open_uni(&mut self) -> Result<iroh::endpoint::SendStream, ConnectionError> {
        // TODO: Use iroh_connection.open_uni()
        todo!("Open unidirectional stream")
    }
    
    /// Open bidirectional stream (matches iroh terminology)
    pub async fn open_bi(&mut self) -> Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream), ConnectionError> {
        // TODO: Use iroh_connection.open_bi()
        todo!("Open bidirectional stream")
    }
    
    /// Accept incoming unidirectional stream (matches iroh terminology)
    pub async fn accept_uni(&mut self) -> Result<iroh::endpoint::RecvStream, ConnectionError> {
        // TODO: Use iroh_connection.accept_uni()
        todo!("Accept unidirectional stream")
    }
    
    /// Accept incoming bidirectional stream (matches iroh terminology)
    pub async fn accept_bi(&mut self) -> Result<(iroh::endpoint::RecvStream, iroh::endpoint::SendStream), ConnectionError> {
        // TODO: Use iroh_connection.accept_bi()
        todo!("Accept bidirectional stream")
    }
}

/// Errors related to P2P connections
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Failed to establish command connection: {source}")]
    Connection { source: eyre::Error },
    
    #[error("Failed to send command: {source}")]
    Send { source: eyre::Error },
    
    #[error("Failed to receive from command: {source}")]
    Receive { source: eyre::Error },
    
    #[error("Command serialization error: {source}")]
    Serialization { source: serde_json::Error },
    
    #[error("Exit code protocol error: {message}")]
    ExitProtocol { message: String },
}

/// Establish a streaming P2P connection (client-side)
///
/// Returns a Connection<PROTOCOL> with direct access to stdin/stdout/stderr streams.
/// You handle the streams however you want - no magic abstractions.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(serde::Serialize, serde::Deserialize, Debug)]
/// struct RemoteShellProtocol {
///     shell: String,
///     args: Vec<String>,
/// }
/// 
/// let mut conn = fastn_p2p::connect(our_key, target_key, RemoteShellProtocol { 
///     shell: "bash".to_string(),
///     args: vec![],
/// }).await?;
/// 
/// // Use the streams directly - no lookups needed
/// tokio::spawn(pipe_stdout(conn.stdout));
/// tokio::spawn(pipe_stderr(conn.stderr));
/// ```
pub async fn connect<PROTOCOL>(
    our_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey, 
    protocol: PROTOCOL
) -> Result<Session<PROTOCOL>, ConnectionError>
where
    PROTOCOL: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    // TODO: Implement streaming connection over P2P
    todo!("Connect to {target} with protocol {protocol:?} using {}", our_key.id52())
}