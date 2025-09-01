//! # HTTP Server Module
//!
//! Provides web access to accounts and rig management via HTTP.
//!
//! ## Routing Logic
//! - `<account-id52>.localhost` → Routes to account HTTP handler
//! - `<rig-id52>.localhost` → Routes to rig HTTP handler  
//! - `localhost` → Default rig management interface
//!
//! ## Features
//! - Subdomain-based routing for account isolation
//! - Account web interface for email management
//! - Rig web interface for system management

/// Start HTTP server for web-based account and rig access (following fastn/serve.rs pattern)
pub async fn start_http_server(
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    rig: fastn_rig::Rig,
    graceful: fastn_net::Graceful,
    port: Option<u16>,
) -> Result<(), fastn_rig::RunError> {
    // Create HTTP service state
    let app = HttpApp {
        account_manager,
        rig,
    };

    // Bind to localhost with automatic port selection if port is 0
    let listener = match port {
        Some(0) | None => {
            // Bind to any available port
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .map_err(|e| fastn_rig::RunError::ShutdownFailed {
                    source: Box::new(e),
                })?;

            let actual_port = listener
                .local_addr()
                .map_err(|e| fastn_rig::RunError::ShutdownFailed {
                    source: Box::new(e),
                })?
                .port();

            println!("🌐 HTTP server auto-selected port {actual_port}");
            tracing::info!("🌐 HTTP server bound to 127.0.0.1:{actual_port}");
            listener
        }
        Some(http_port) => {
            // Bind to specific port
            let bind_addr = format!("127.0.0.1:{http_port}");
            let listener = tokio::net::TcpListener::bind(&bind_addr)
                .await
                .map_err(|e| fastn_rig::RunError::ShutdownFailed {
                    source: Box::new(e),
                })?;

            println!("🌐 HTTP server listening on http://localhost:{http_port}");
            tracing::info!("🌐 HTTP server bound to {bind_addr}");
            listener
        }
    };

    // Spawn HTTP server task following fastn/serve.rs pattern
    graceful.spawn(async move {
        println!("🚀 HTTP server task started");

        loop {
            let (stream, _addr) = match listener.accept().await {
                Ok(stream) => stream,
                Err(e) => {
                    tracing::error!("Failed to accept HTTP connection: {e}");
                    continue;
                }
            };

            let app_clone = app.clone();
            tokio::task::spawn(async move {
                // Use hyper adapter for proper HTTP handling (following fastn/serve.rs)
                let io = hyper_util::rt::TokioIo::new(stream);

                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(
                        io,
                        hyper::service::service_fn(move |req| {
                            handle_request(req, app_clone.clone())
                        }),
                    )
                    .await
                {
                    tracing::warn!("HTTP connection error: {err:?}");
                }
            });
        }
    });

    Ok(())
}

/// HTTP application state
#[derive(Clone)]
struct HttpApp {
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    rig: fastn_rig::Rig,
}

/// Handle HTTP requests using hyper (following fastn/serve.rs pattern)
async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    app: HttpApp,
) -> Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, std::convert::Infallible> {
    println!("🌐 HTTP Request: {} {}", req.method(), req.uri());

    // Convert hyper request to our HttpRequest type
    let http_request = convert_hyper_request(&req);

    println!("🎯 Routing to: {} {}", http_request.host, http_request.path);

    // Route based on subdomain
    let response = route_request(&http_request, &app).await;

    // Convert our response to hyper response
    Ok(convert_to_hyper_response(response))
}

/// Convert hyper request to our HttpRequest type
fn convert_hyper_request(req: &hyper::Request<hyper::body::Incoming>) -> fastn_router::HttpRequest {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Extract host from headers
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost")
        .to_string();

    // Convert all headers
    let mut headers = std::collections::HashMap::new();
    for (key, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(key.to_string(), value_str.to_string());
        }
    }

    fastn_router::HttpRequest {
        method,
        path,
        host,
        headers,
    }
}

/// Route HTTP request based on subdomain
async fn route_request(
    request: &fastn_router::HttpRequest,
    app: &HttpApp,
) -> fastn_router::HttpResponse {
    println!("🎯 Routing: {} {}", request.host, request.path);

    // Extract ID52 from subdomain
    if let Some(id52) = extract_id52_from_host(&request.host) {
        println!("🔍 Extracted ID52: {id52}");

        // Check if this ID52 belongs to an account
        if let Ok(id52_key) = id52.parse::<fastn_id52::PublicKey>()
            && let Ok(account) = app.account_manager.find_account_by_alias(&id52_key).await
        {
            return account_route(&account, request).await;
        }

        // Check if this ID52 is the rig
        if app.rig.id52() == id52 {
            return rig_route(&app.rig, request).await;
        }

        // ID52 not found locally - attempt P2P proxy to remote peer
        println!("🌐 ID52 {id52} not local, attempting P2P proxy...");
        proxy_to_remote_peer(&id52, request, app).await
    } else {
        // Default rig interface
        rig_route(&app.rig, request).await
    }
}

/// Extract ID52 from hostname (e.g., "abc123.localhost" → "abc123")
fn extract_id52_from_host(host: &str) -> Option<String> {
    if host.ends_with(".localhost") {
        let id52 = host.strip_suffix(".localhost")?;
        if id52.len() == 52 {
            Some(id52.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Handle requests routed to an account
async fn account_route(
    account: &fastn_account::Account,
    request: &fastn_router::HttpRequest,
) -> fastn_router::HttpResponse {
    // For now, all requests are treated as local (None)
    // TODO: Implement P2P requester detection for remote browsing
    account.route_http(request, None).await.unwrap_or_else(|e| {
        fastn_router::HttpResponse::internal_error(format!("Account routing error: {e}"))
    })
}

/// Handle requests routed to the rig
async fn rig_route(
    rig: &fastn_rig::Rig,
    request: &fastn_router::HttpRequest,
) -> fastn_router::HttpResponse {
    // For now, all requests are treated as local (None)
    // TODO: Implement P2P requester detection for remote browsing
    rig.route_http(request, None).await.unwrap_or_else(|e| {
        fastn_router::HttpResponse::internal_error(format!("Rig routing error: {e}"))
    })
}

/// Convert our HttpResponse to hyper response
fn convert_to_hyper_response(
    response: fastn_router::HttpResponse,
) -> hyper::Response<http_body_util::Full<hyper::body::Bytes>> {
    let mut builder = hyper::Response::builder().status(response.status);

    // Add headers
    for (key, value) in response.headers {
        builder = builder.header(key, value);
    }

    builder
        .body(http_body_util::Full::new(hyper::body::Bytes::from(
            response.body,
        )))
        .unwrap_or_else(|_| {
            hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(
                "Internal Server Error",
            )))
        })
}

/// Proxy request to remote peer when ID52 is not local (following kulfi pattern)
async fn proxy_to_remote_peer(
    target_id52: &str,
    request: &fastn_router::HttpRequest,
    _app: &HttpApp,
) -> fastn_router::HttpResponse {
    println!("🚀 Attempting to proxy to remote peer: {target_id52}");

    // TODO: Get our endpoint and peer_stream_senders from app context
    // For now, return a placeholder response indicating P2P proxy would happen
    let body = format!(
        "🌐 P2P Proxy (Not Yet Implemented)\n\n\
        Target ID52: {target_id52}\n\
        Request: {} {}\n\
        Host: {}\n\n\
        This request would be proxied to remote peer {target_id52} via P2P.\n\n\
        Implementation needed:\n\
        - Get our iroh endpoint for P2P connection\n\
        - Use fastn_net::get_stream() with Protocol::HttpProxy\n\
        - Send HTTP request over P2P to remote peer\n\
        - Receive and return HTTP response from remote peer\n\n\
        Infrastructure ready:\n\
        - P2P connection management ✅\n\
        - HTTP request/response types ✅\n\
        - Protocol header with proxy data ✅\n\
        - Request serialization framework ✅",
        request.method, request.path, request.host
    );

    fastn_router::HttpResponse::ok(body)
}
