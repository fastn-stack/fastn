// Functions that work with iroh types

/// Gets the ID52 string of the remote peer from a connection.
///
/// Extracts the remote node's public key and converts it to ID52 format
/// (52-character BASE32_DNSSEC encoding).
///
/// # Errors
///
/// Returns an error if the remote node ID cannot be read from the connection.
pub async fn get_remote_id52(conn: &iroh::endpoint::Connection) -> eyre::Result<String> {
    let remote_node_id = match conn.remote_node_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("could not read remote node id: {e}, closing connection");
            // TODO: is this how we close the connection in error cases or do we send some error
            //       and wait for other side to close the connection?
            let e2 = conn.closed().await;
            tracing::info!("connection closed: {e2}");
            // TODO: send another error_code to indicate bad remote node id?
            conn.close(0u8.into(), &[]);
            return Err(eyre::anyhow!("could not read remote node id: {e}"));
        }
    };

    // Convert iroh::PublicKey to ID52 string
    let bytes = remote_node_id.as_bytes();
    Ok(data_encoding::BASE32_DNSSEC.encode(bytes))
}

async fn ack(send: &mut iroh::endpoint::SendStream) -> eyre::Result<()> {
    tracing::trace!("sending ack");
    send.write_all(format!("{}\n", crate::ACK).as_bytes())
        .await?;
    tracing::trace!("sent ack");
    Ok(())
}

/// Accepts an incoming bidirectional stream with the expected protocol.
///
/// Continuously accepts incoming streams until one matches the expected protocol.
/// Automatically handles and responds to ping messages.
///
/// # Errors
///
/// Returns an error if a non-ping stream has unexpected protocol.
pub async fn accept_bi(
    conn: &iroh::endpoint::Connection,
    expected: crate::Protocol,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    loop {
        tracing::trace!("accepting bidirectional stream");
        match accept_bi_(conn).await? {
            (mut send, _recv, crate::Protocol::Ping) => {
                tracing::trace!("got ping");
                tracing::trace!("sending PONG");
                send.write_all(crate::PONG)
                    .await
                    .inspect_err(|e| tracing::error!("failed to write PONG: {e:?}"))?;
                tracing::trace!("sent PONG");
            }
            (s, r, found) => {
                tracing::trace!("got bidirectional stream: {found:?}");
                if found != expected {
                    return Err(eyre::anyhow!("expected: {expected:?}, got {found:?}"));
                }
                return Ok((s, r));
            }
        }
    }
}

/// Accepts an incoming bidirectional stream and reads additional data.
///
/// Like `accept_bi` but also reads and deserializes the next JSON message
/// from the stream after protocol negotiation.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize from the stream
///
/// # Errors
///
/// Returns an error if protocol doesn't match or deserialization fails.
pub async fn accept_bi_with<T: serde::de::DeserializeOwned>(
    conn: &iroh::endpoint::Connection,
    expected: crate::Protocol,
) -> eyre::Result<(T, iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let (send, mut recv) = accept_bi(conn, expected).await?;
    let next = next_json(&mut recv)
        .await
        .inspect_err(|e| tracing::error!("failed to read next message: {e}"))?;

    Ok((next, send, recv))
}

async fn accept_bi_(
    conn: &iroh::endpoint::Connection,
) -> eyre::Result<(
    iroh::endpoint::SendStream,
    iroh::endpoint::RecvStream,
    crate::Protocol,
)> {
    tracing::trace!("accept_bi_ called");
    let (mut send, mut recv) = conn.accept_bi().await?;
    tracing::trace!("accept_bi_ got send and recv");

    let msg: crate::Protocol = next_json(&mut recv)
        .await
        .inspect_err(|e| tracing::error!("failed to read next message: {e}"))?;

    tracing::trace!("msg: {msg:?}");

    ack(&mut send).await?;

    tracing::trace!("ack sent");
    Ok((send, recv, msg))
}

/// Reads a newline-terminated JSON message from a stream.
///
/// Reads bytes until a newline character is encountered, then deserializes
/// the buffer as JSON into the specified type.
///
/// # Errors
///
/// Returns an error if:
/// - Connection is closed while reading
/// - JSON deserialization fails
pub async fn next_json<T: serde::de::DeserializeOwned>(
    recv: &mut iroh::endpoint::RecvStream,
) -> eyre::Result<T> {
    // NOTE: the capacity is just a guess to avoid reallocations
    let mut buffer = Vec::with_capacity(1024);

    loop {
        let mut byte = [0u8];
        let n = recv.read(&mut byte).await?;

        if n == Some(0) || n.is_none() {
            return Err(eyre::anyhow!(
                "connection closed while reading response header"
            ));
        }

        if byte[0] == b'\n' {
            break;
        } else {
            buffer.push(byte[0]);
        }
    }

    Ok(serde_json::from_slice(&buffer)?)
}

/// Reads a newline-terminated string from a stream.
///
/// Reads bytes until a newline character is encountered and returns
/// the result as a UTF-8 string.
///
/// # Errors
///
/// Returns an error if:
/// - Connection is closed while reading
/// - Bytes are not valid UTF-8
pub async fn next_string(recv: &mut iroh::endpoint::RecvStream) -> eyre::Result<String> {
    // NOTE: the capacity is just a guess to avoid reallocations
    let mut buffer = Vec::with_capacity(1024);

    loop {
        let mut byte = [0u8];
        let n = recv.read(&mut byte).await?;

        if n == Some(0) || n.is_none() {
            return Err(eyre::anyhow!(
                "connection closed while reading response header"
            ));
        }

        if byte[0] == b'\n' {
            break;
        } else {
            buffer.push(byte[0]);
        }
    }

    String::from_utf8(buffer).map_err(|e| eyre::anyhow!("failed to convert bytes to string: {e}"))
}

/// Returns a global singleton Iroh endpoint.
///
/// Creates the endpoint on first call and returns the same instance
/// on subsequent calls. Configured with:
/// - Local network discovery
/// - N0 discovery (DHT-based)
/// - ALPN: `/fastn/identity/0.1`
///
/// # Panics
///
/// Panics if endpoint creation fails.
pub async fn global_iroh_endpoint() -> iroh::Endpoint {
    async fn new_iroh_endpoint() -> iroh::Endpoint {
        // TODO: read secret key from ENV VAR
        iroh::Endpoint::builder()
            .discovery_n0()
            .discovery_local_network()
            .alpns(vec![crate::APNS_IDENTITY.into()])
            .bind()
            .await
            .expect("failed to create iroh Endpoint")
    }

    static IROH_ENDPOINT: tokio::sync::OnceCell<iroh::Endpoint> =
        tokio::sync::OnceCell::const_new();
    IROH_ENDPOINT.get_or_init(new_iroh_endpoint).await.clone()
}
