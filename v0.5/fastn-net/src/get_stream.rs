/// Connection pool for P2P stream management.
///
/// Maintains a map of active peer connections indexed by (self ID52, remote ID52) pairs.
/// Each entry contains a channel for requesting new streams on that connection.
/// Connections are automatically removed when they break or become unhealthy.
///
/// This type is used to reuse existing P2P connections instead of creating new ones
/// for each request, improving performance and reducing connection overhead.
pub type PeerStreamSenders = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<(SelfID52, RemoteID52), StreamRequestSender>>,
>;

type Stream = (iroh::endpoint::SendStream, iroh::endpoint::RecvStream);
type StreamResult = eyre::Result<Stream>;
type ReplyChannel = tokio::sync::oneshot::Sender<StreamResult>;
type RemoteID52 = String;
type SelfID52 = String;

type StreamRequest = (crate::ProtocolHeader, ReplyChannel);

type StreamRequestSender = tokio::sync::mpsc::Sender<StreamRequest>;
type StreamRequestReceiver = tokio::sync::mpsc::Receiver<StreamRequest>;

/// Gets or creates a bidirectional stream to a remote peer.
///
/// This function manages P2P connections efficiently by:
/// 1. Reusing existing connections when available
/// 2. Creating new connections when needed
/// 3. Verifying connection health with protocol handshake
/// 4. Automatically reconnecting on failure
///
/// The function sends the protocol header and waits for acknowledgment
/// to ensure the stream is healthy before returning it.
///
/// # Arguments
///
/// * `self_endpoint` - Local Iroh endpoint
/// * `header` - Protocol header to negotiate
/// * `remote_node_id52` - ID52 of the target peer
/// * `peer_stream_senders` - Connection pool for reuse
/// * `graceful` - Graceful shutdown handle
///
/// # Returns
///
/// A tuple of (SendStream, RecvStream) ready for communication.
///
/// # Errors
///
/// Returns an error if connection fails or protocol negotiation times out.
#[tracing::instrument(skip_all)]
pub async fn get_stream(
    self_endpoint: iroh::Endpoint,
    header: crate::ProtocolHeader,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
    graceful: crate::Graceful,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    use eyre::WrapErr;

    tracing::trace!("get_stream: {header:?}");
    println!(
        "🔍 DEBUG get_stream: Starting stream request for {}",
        remote_node_id52
    );
    let stream_request_sender = get_stream_request_sender(
        self_endpoint,
        remote_node_id52,
        peer_stream_senders,
        graceful,
    )
    .await;
    println!("✅ DEBUG get_stream: Got stream_request_sender");
    tracing::trace!("got stream_request_sender");
    let (reply_channel, receiver) = tokio::sync::oneshot::channel();
    println!("🔗 DEBUG get_stream: Created oneshot channel");

    println!("📤 DEBUG get_stream: About to send stream request");
    stream_request_sender
        .send((header, reply_channel))
        .await
        .wrap_err_with(|| "failed to send on stream_request_sender")?;
    println!("✅ DEBUG get_stream: Stream request sent");

    tracing::trace!("sent stream request");

    println!("⏳ DEBUG get_stream: Waiting for stream reply");
    let r = receiver.await?;
    println!("✅ DEBUG get_stream: Received stream reply");

    tracing::trace!("got stream request reply");
    r
}

#[tracing::instrument(skip_all)]
async fn get_stream_request_sender(
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
    graceful: crate::Graceful,
) -> StreamRequestSender {
    println!(
        "🔍 DEBUG get_stream_request_sender: Starting for {}",
        remote_node_id52
    );
    // Convert iroh::PublicKey to ID52 string
    let self_id52 = data_encoding::BASE32_DNSSEC.encode(self_endpoint.node_id().as_bytes());
    println!(
        "🔍 DEBUG get_stream_request_sender: Self ID52: {}",
        self_id52
    );
    let mut senders = peer_stream_senders.lock().await;
    println!("🔍 DEBUG get_stream_request_sender: Got peer_stream_senders lock");

    if let Some(sender) = senders.get(&(self_id52.clone(), remote_node_id52.clone())) {
        return sender.clone();
    }

    // TODO: figure out if the mpsc::channel is the right size
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    senders.insert(
        (self_id52.clone(), remote_node_id52.clone()),
        sender.clone(),
    );
    drop(senders);

    let graceful_for_connection_manager = graceful.clone();
    println!(
        "🚀 DEBUG get_stream_request_sender: Spawning connection_manager task for {}",
        remote_node_id52
    );
    graceful.spawn(async move {
        println!(
            "📋 DEBUG connection_manager: Task started for {}",
            remote_node_id52
        );
        let result = connection_manager(
            receiver,
            self_endpoint,
            remote_node_id52.clone(),
            graceful_for_connection_manager,
        )
        .await;
        println!(
            "📋 DEBUG connection_manager: Task ended for {} with result: {:?}",
            remote_node_id52, result
        );

        // cleanup the peer_stream_senders map, so no future tasks will try to use this.
        let mut senders = peer_stream_senders.lock().await;
        senders.remove(&(self_id52.clone(), remote_node_id52));
    });

    sender
}

async fn connection_manager(
    mut receiver: StreamRequestReceiver,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
    graceful: crate::Graceful,
) {
    println!(
        "🔧 DEBUG connection_manager: Function started for {}",
        remote_node_id52
    );
    let e = match connection_manager_(
        &mut receiver,
        self_endpoint,
        remote_node_id52.clone(),
        graceful,
    )
    .await
    {
        Ok(()) => {
            tracing::info!("connection manager closed");
            return;
        }
        Err(e) => e,
    };

    // what is our error handling strategy?
    //
    // since an error has just occurred on our connection, it is best to cancel all concurrent
    // tasks that depend on this connection, and let the next task recreate the connection, this
    // way things are clean.
    //
    // we can try to keep the concurrent tasks open, and retry connection, but it increases the
    // complexity of implementation, and it is not worth it for now.
    //
    // also note that connection_manager() and it's caller, get_stream(), are called to create the
    // initial stream only, this error handling strategy will work for concurrent requests that are
    // waiting for the stream to be created. the tasks that already got the stream will not be
    // affected by this. tho, since something wrong has happened with the connection, they will
    // eventually fail too.
    tracing::error!("connection manager worker error: {e:?}");

    // once we close the receiver, any tasks that have gotten access to the corresponding sender
    // will fail when sending.
    receiver.close();

    // send an error to all the tasks that are waiting for stream for this receiver.
    while let Some((_protocol, reply_channel)) = receiver.recv().await {
        if reply_channel
            .send(Err(eyre::anyhow!("failed to create connection: {e:?}")))
            .is_err()
        {
            tracing::error!("failed to send error reply: {e:?}");
        }
    }
}

#[tracing::instrument(skip_all)]
async fn connection_manager_(
    receiver: &mut StreamRequestReceiver,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
    graceful: crate::Graceful,
) -> eyre::Result<()> {
    println!(
        "🔧 DEBUG connection_manager_: Starting main loop for {}",
        remote_node_id52
    );
    let conn = match self_endpoint
        .connect(
            {
                // Convert ID52 to iroh::NodeId
                use std::str::FromStr;
                let public_key = fastn_id52::PublicKey::from_str(&remote_node_id52)
                    .map_err(|e| eyre::anyhow!("{}", e))?;
                iroh::NodeId::from(iroh::PublicKey::from_bytes(&public_key.to_bytes())?)
            },
            crate::APNS_IDENTITY,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("🔍 DEBUG: Connection establishment failed: {e:?}");
            println!("🔍 DEBUG: Connection establishment failed, this could trigger CPU issues: {e:?}");
            return Err(eyre::anyhow!("failed to create connection: {e:?}"));
        }
    };

    let timeout = std::time::Duration::from_secs(12);
    let mut idle_counter = 0;

    loop {
        tracing::trace!("connection manager loop");

        if idle_counter > 4 {
            tracing::info!("🔍 DEBUG: Connection idle timeout after {} cycles (60s total)", idle_counter);
            println!("🔍 DEBUG: Connection manager hitting 60s idle timeout, breaking loop");
            // this ensures we keep a connection open only for 12 * 5 seconds = 1 min
            break;
        }

        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::info!("graceful shutdown");
                break;
            },
            _ = tokio::time::sleep(timeout) => {
                tracing::info!("🔍 DEBUG: Connection manager woken up after 12s (idle_counter: {})", idle_counter);
                println!("🔍 DEBUG: Connection manager 12s timeout, idle_counter: {}", idle_counter);
                if let Err(e) = crate::ping(&conn).await {
                    tracing::error!("🔍 DEBUG: Pinging failed: {e:?}");
                    println!("🔍 DEBUG: Pinging failed, breaking connection loop");
                    break;
                }
                idle_counter += 1;
                println!("🔍 DEBUG: Ping successful, idle_counter now: {}", idle_counter);
            },
            Some((header, reply_channel)) = receiver.recv() => {
                println!("📨 DEBUG connection_manager: Received stream request for {:?}, idle counter: {}", header, idle_counter);
                tracing::info!("connection: {header:?}, idle counter: {idle_counter}");
                idle_counter = 0;
                // is this a good idea to serialize this part? if 10 concurrent requests come in, we will
                // handle each one sequentially. the other alternative is to spawn a task for each request.
                // so which is better?
                //
                // in general, if we do it in parallel via spawning, we will have better throughput.
                //
                // and we are not worried about having too many concurrent tasks, tho iroh has a limit on
                // concurrent tasks[1], with a default of 100[2]. it is actually a todo to find out what
                // happens when we hit this limit, do they handle it by queueing the tasks, or do they
                // return an error. if they queue then we wont have to implement queue logic.
                //
                // [1]: https://docs.rs/iroh/0.34.1/iroh/endpoint/struct.TransportConfig.html#method.max_concurrent_bidi_streams
                // [2]: https://docs.rs/iroh-quinn-proto/0.13.0/src/iroh_quinn_proto/config/transport.rs.html#354
                //
                // but all that is besides the point, we are worried about resilience right now, not
                // throughput per se (throughput is secondary goal, resilience primary).
                //
                // say we have 10 concurrent requests and lets say if we spawned a task for each, what
                // happens in error case? say connection failed, the device switched from wifi to 4g, or
                // whatever? in the handler task, we are putting a timeout on the read. in the serial case
                // the first request will timeout, and all subsequent requests will get immediately an
                // error. its predictable, its clean.
                //
                // if the tasks were spawned, each will timeout independently.
                //
                // we can also no longer rely on this function, connection_manager_, returning an error for
                // them, so our connection_handler strategy will interfere, we would have read more requests
                // off of receiver.
                //
                // do note that this is not a clear winner problem, this is a tradeoff, we lose throughput,
                // as in best case scenario, 10 concurrent tasks will be better. we will have to revisit
                // this in future when we are performance optimising things.
                if let Err(e) = handle_request(&conn, header, reply_channel).await {
                    tracing::error!("failed to handle request: {e:?}");
                    // note: we are intentionally not calling conn.close(). why? so that if some existing
                    // stream is still open, if we explicitly call close on the connection, that stream will
                    // immediately fail as well, and we do not want that. we want to let the stream fail
                    // on its own, maybe it will work, maybe it will not.
                    return Err(e);
                }
                tracing::info!("handled connection");
            }
            else => {
                tracing::error!("failed to read from receiver");
                break
            },
        }
    }

    Ok(())
}

async fn handle_request(
    conn: &iroh::endpoint::Connection,
    header: crate::ProtocolHeader,
    reply_channel: ReplyChannel,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    tracing::trace!("handling request: {header:?}");
    println!(
        "🔧 DEBUG handle_request: Handling stream request for protocol {:?}",
        header
    );

    println!("🔗 DEBUG handle_request: About to open bi-directional stream");
    let (mut send, mut recv) = match tokio::time::timeout(
        std::time::Duration::from_secs(10), // 10 second timeout for stream opening
        conn.open_bi(),
    )
    .await
    {
        Ok(Ok(v)) => {
            println!("✅ DEBUG handle_request: Successfully opened bi-directional stream");
            tracing::trace!("opened bi-stream");
            v
        }
        Ok(Err(e)) => {
            println!(
                "❌ DEBUG handle_request: Failed to open bi-directional stream: {:?}",
                e
            );
            tracing::error!("failed to open_bi: {e:?}");
            return Err(eyre::anyhow!("failed to open_bi: {e:?}"));
        }
        Err(_timeout) => {
            println!(
                "⏰ DEBUG handle_request: Timed out opening bi-directional stream after 10 seconds"
            );
            return Err(eyre::anyhow!("timed out opening bi-directional stream"));
        }
    };

    println!("📤 DEBUG handle_request: About to write protocol to stream");
    send.write_all(
        &serde_json::to_vec(&header.protocol)
            .wrap_err_with(|| format!("failed to serialize protocol: {:?}", header.protocol))?,
    )
    .await?;
    println!("✅ DEBUG handle_request: Successfully wrote protocol to stream");
    tracing::trace!("wrote protocol");

    println!("📤 DEBUG handle_request: About to write newline");
    send.write(b"\n")
        .await
        .wrap_err_with(|| "failed to write newline")?;
    println!("✅ DEBUG handle_request: Successfully wrote newline");

    tracing::trace!("wrote newline");

    if let Some(extra) = header.extra {
        send.write_all(extra.as_bytes()).await?;
        tracing::trace!("wrote protocol");

        send.write(b"\n")
            .await
            .wrap_err_with(|| "failed to write newline")?;
    }

    let msg = crate::next_string(&mut recv).await?;

    if msg != crate::ACK {
        tracing::error!("failed to read ack: {msg:?}");
        return Err(eyre::anyhow!("failed to read ack: {msg:?}"));
    }

    tracing::trace!("received ack");

    println!("📤 DEBUG handle_request: About to send stream reply");
    reply_channel.send(Ok((send, recv))).unwrap_or_else(|e| {
        println!(
            "❌ DEBUG handle_request: Failed to send stream reply: {:?}",
            e
        );
        tracing::error!("failed to send reply: {e:?}");
    });
    println!("✅ DEBUG handle_request: Stream reply sent successfully");

    tracing::trace!("handle_request done");

    Ok(())
}
