/// Response message sent back after receiving a ping.
pub const PONG: &[u8] = b"pong\n";
pub const ACK_PONG: &[u8] = b"ack\npong\n";

/// Sends a ping message to test connectivity with a peer.
///
/// Opens a bidirectional stream, sends a `Protocol::Ping` message,
/// and waits for a PONG response. Used for connection health checks.
///
/// # Errors
///
/// Returns an error if:
/// - Failed to open bidirectional stream
/// - Failed to send ping message
/// - Failed to receive or incorrect pong response
pub async fn ping(conn: &iroh::endpoint::Connection) -> eyre::Result<()> {
    tracing::info!("ping called");
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
    tracing::info!("got bi, sending ping");
    send_stream
        .write_all(&serde_json::to_vec(&crate::Protocol::Ping)?)
        .await?;
    tracing::info!("sent ping, sending newline");
    send_stream.write_all("\n".as_bytes()).await?;
    tracing::info!("newline sent, waiting for reply");
    let msg = recv_stream
        .read_to_end(1000)
        .await
        .inspect_err(|e| tracing::error!("failed to read: {e}"))?;
    tracing::info!("got {:?}, {PONG:?}", str::from_utf8(&msg));
    if msg != ACK_PONG {
        return Err(eyre::anyhow!("expected {PONG:?}, got {msg:?}"));
    }
    tracing::info!("got reply, finishing stream");
    send_stream.finish()?;
    tracing::info!("finished stream");
    Ok(())
}
