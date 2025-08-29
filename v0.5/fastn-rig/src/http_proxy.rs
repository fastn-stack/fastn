//! # HTTP to P2P Proxy
//!
//! Proxy HTTP requests to remote fastn peers (following kulfi/malai pattern).

/// Proxy HTTP request to remote peer over P2P (following kulfi http_to_peer pattern)
pub async fn http_to_peer(
    req: hyper::Request<hyper::body::Incoming>,
    target_id52: &str,
    our_endpoint: iroh::Endpoint,
    peer_stream_senders: &fastn_net::PeerStreamSenders,
    graceful: &fastn_net::Graceful,
) -> Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    println!("🌐 Proxying HTTP request to remote peer: {target_id52}");
    tracing::info!("Proxying HTTP request to peer: {target_id52}");
    
    // Get P2P stream to remote peer using fastn-net infrastructure
    let (mut send, mut recv) = fastn_net::get_stream(
        our_endpoint,
        fastn_net::Protocol::HttpProxy.into(),
        target_id52.to_string(),
        peer_stream_senders.clone(),
        graceful.clone(),
    ).await.map_err(|e| format!("Failed to get P2P stream to {target_id52}: {e}"))?;
    
    println!("🔗 P2P stream established to {target_id52}");
    
    // Send proxy data header (following kulfi pattern)
    let proxy_data = fastn_router::ProxyData::Http { 
        target_id52: target_id52.to_string() 
    };
    send.write_all(&serde_json::to_vec(&proxy_data)?).await?;
    send.write_all(b"\n").await?;
    
    // Convert hyper request to proxy request and send (following kulfi pattern)
    let (head, _body) = req.into_parts();
    let proxy_request = fastn_router::ProxyRequest::from(head);
    send.write_all(&serde_json::to_vec(&proxy_request)?).await?;
    send.write_all(b"\n").await?;
    
    println!("📤 Sent request to {target_id52}");
    
    // Wait for response from remote peer
    let response_json = fastn_net::next_string(&mut recv).await
        .map_err(|e| format!("Failed to receive response from {target_id52}: {e}"))?;
    
    let proxy_response: fastn_router::ProxyResponse = serde_json::from_str(&response_json)
        .map_err(|e| format!("Failed to parse response from {target_id52}: {e}"))?;
    
    println!("📨 Received response from {target_id52}: status {}", proxy_response.status);
    
    // Convert proxy response back to hyper response
    let mut builder = hyper::Response::builder().status(proxy_response.status);
    
    // Add headers from proxy response
    for (key, value) in proxy_response.headers {
        if let Ok(value_str) = String::from_utf8(value) {
            builder = builder.header(key, value_str);
        }
    }
    
    // TODO: Handle response body from proxy response
    let body = format!("Response from remote peer {target_id52} (body handling TODO)");
    
    Ok(builder
        .body(http_body_util::Full::new(hyper::body::Bytes::from(body)))
        .unwrap_or_else(|_| {
            hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(
                "Proxy Error"
            )))
        }))
}