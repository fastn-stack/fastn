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

/// Start HTTP server for web-based account and rig access
pub async fn start_http_server(
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    rig: fastn_rig::Rig,
    graceful: fastn_net::Graceful,
) -> Result<(), fastn_rig::RunError> {
    println!("🌐 Starting HTTP server on port 8000...");
    tracing::info!("🌐 Starting HTTP server for web access");

    // Create HTTP service with routing
    let app = create_app(account_manager, rig);

    // Bind to localhost:8000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .map_err(|e| fastn_rig::RunError::ShutdownFailed {
            source: Box::new(e),
        })?;

    println!("🌐 HTTP server listening on http://localhost:8000");
    tracing::info!("🌐 HTTP server bound to 127.0.0.1:8000");

    // Spawn HTTP server task
    let graceful_clone = graceful.clone();
    graceful.spawn(async move {
        println!("🚀 HTTP server task started");

        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            println!("🔗 HTTP connection from {}", addr);
                            let app_clone = app.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, app_clone).await {
                                    tracing::warn!("HTTP connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to accept HTTP connection: {}", e);
                        }
                    }
                }

                // Graceful shutdown
                _ = graceful_clone.cancelled() => {
                    println!("🛑 HTTP server shutting down");
                    break;
                }
            }
        }

        println!("🏁 HTTP server task finished");
    });

    Ok(())
}

/// HTTP application state
#[derive(Clone)]
struct HttpApp {
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    rig: fastn_rig::Rig,
}

/// Create HTTP application with routing
fn create_app(
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    rig: fastn_rig::Rig,
) -> HttpApp {
    HttpApp {
        account_manager,
        rig,
    }
}

/// Handle individual HTTP connection
async fn handle_connection(
    stream: tokio::net::TcpStream,
    app: HttpApp,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse HTTP request
    let mut buffer = [0; 1024];
    stream.readable().await?;
    let n = stream.try_read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // Convert hyper request to our HttpRequest type
    let http_request = convert_hyper_request(&req);

    println!(
        "🌐 HTTP Request: {} {}",
        http_request.host, http_request.path
    );

    // Route based on subdomain
    let response = route_request(&http_request, &app).await;

    // Send response
    stream.writable().await?;
    let response_string = response.to_http_string();
    let _bytes_written = stream.try_write(response_string.as_bytes())?;

    Ok(())
}

/// Convert hyper request to our HttpRequest type
fn convert_hyper_request(req: &hyper::Request<hyper::body::Incoming>) -> fastn_router::HttpRequest {
    let mut host = "localhost".to_string();
    let mut path = "/".to_string();
    let mut method = "GET".to_string();
    let mut headers = std::collections::HashMap::new();

    for line in request.lines() {
        if line.starts_with("GET ")
            || line.starts_with("POST ")
            || line.starts_with("PUT ")
            || line.starts_with("DELETE ")
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                method = parts[0].to_string();
                path = parts[1].to_string();
            }
        } else if line.contains(':') {
            let mut split = line.splitn(2, ':');
            if let (Some(key), Some(value)) = (split.next(), split.next()) {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();

                if key == "host" {
                    host = value.clone();
                }
                headers.insert(key, value);
            }
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

        // ID52 not found
        fastn_router::HttpResponse::not_found(format!("ID52 {} not found", id52))
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
