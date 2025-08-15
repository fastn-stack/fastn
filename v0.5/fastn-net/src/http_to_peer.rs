/// Proxies an HTTP request to a remote peer over P2P.
///
/// Takes an incoming HTTP request and forwards it to a remote peer using
/// the Iroh P2P network. The response from the peer is returned.
///
/// # Arguments
///
/// * `header` - Protocol header containing metadata
/// * `req` - The HTTP request to proxy
/// * `self_endpoint` - Local Iroh endpoint
/// * `remote_node_id52` - ID52 of the target peer
/// * `peer_connections` - Connection pool for peer streams
/// * `graceful` - Graceful shutdown handle
///
/// # Errors
///
/// Returns an error if connection or proxying fails.
#[tracing::instrument(skip_all)]
pub async fn http_to_peer(
    header: crate::ProtocolHeader,
    req: hyper::Request<hyper::body::Incoming>,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: crate::PeerStreamSenders,
    graceful: crate::Graceful,
) -> crate::http::ProxyResult<eyre::Error> {
    use http_body_util::BodyExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, mut recv) = crate::get_stream(
        self_endpoint,
        header,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    let (head, mut body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all(b"\n").await?;

    tracing::info!("sent request header");

    while let Some(chunk) = body.frame().await {
        match chunk {
            Ok(v) => {
                let data = v
                    .data_ref()
                    .ok_or_else(|| eyre::anyhow!("chunk data is None"))?;
                tracing::trace!("sending chunk of size: {}", data.len());
                send.write_all(data).await?;
            }
            Err(e) => {
                tracing::error!("error reading chunk: {e:?}");
                return Err(eyre::anyhow!("read_chunk error: {e:?}"));
            }
        }
    }

    tracing::info!("sent body");

    let r: crate::http::Response = crate::next_json(&mut recv).await?;

    tracing::info!("got response header: {:?}", r);

    let stream = tokio_util::io::ReaderStream::new(recv);

    use futures_util::TryStreamExt;

    let stream_body = http_body_util::StreamBody::new(
        stream
            .map_ok(|b| {
                tracing::trace!("got chunk of size: {}", b.len());
                hyper::body::Frame::data(b)
            })
            .map_err(|e| {
                tracing::info!("error reading chunk: {e:?}");
                eyre::anyhow!("read_chunk error: {e:?}")
            }),
    );

    let boxed_body = http_body_util::BodyExt::boxed(stream_body);

    let mut res = hyper::Response::builder().status(hyper::http::StatusCode::from_u16(r.status)?);

    for (k, v) in r.headers {
        res = res.header(
            hyper::http::header::HeaderName::from_bytes(k.as_bytes())?,
            hyper::http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    let res = res.body(boxed_body)?;

    tracing::info!("all done");
    Ok(res)
}

/// Use http_to_peer unless you have a clear reason
/// Proxies an HTTP request to a remote peer (non-streaming version).
///
/// Similar to `http_to_peer` but buffers the entire request/response
/// instead of streaming. Better for small requests.
///
/// # Errors
///
/// Returns an error if connection or proxying fails.
pub async fn http_to_peer_non_streaming(
    header: crate::ProtocolHeader,
    req: hyper::Request<hyper::body::Bytes>,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: crate::PeerStreamSenders,
    graceful: crate::Graceful,
) -> crate::http::ProxyResult {
    use http_body_util::BodyExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, mut recv) = crate::get_stream(
        self_endpoint,
        header,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all(b"\n").await?;

    tracing::info!("sent request header");

    send.write_all(&body).await?;

    tracing::info!("sent body");

    let r: crate::http::Response = crate::next_json(&mut recv).await?;

    tracing::info!("got response header: {r:?}");

    let mut body = Vec::with_capacity(1024 * 4);

    tracing::trace!("reading body");

    while let Some(v) = match recv.read_chunk(1024 * 64, true).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!("error reading chunk: {e:?}");
            Err(eyre::anyhow!("read_chunk error: {e:?}"))
        }
    }? {
        body.extend_from_slice(&v.bytes);
        tracing::trace!(
            "reading body, partial: {}, new body size: {} bytes",
            v.bytes.len(),
            body.len()
        );
    }

    tracing::debug!("got {} bytes of body", body.len());

    let mut res = hyper::Response::new(
        http_body_util::Full::new(body.into())
            .map_err(|e| match e {})
            .boxed(),
    );
    *res.status_mut() = hyper::http::StatusCode::from_u16(r.status)?;
    for (k, v) in r.headers {
        res.headers_mut().insert(
            hyper::http::header::HeaderName::from_bytes(k.as_bytes())?,
            hyper::http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    tracing::info!("all done");
    Ok(res)
}
