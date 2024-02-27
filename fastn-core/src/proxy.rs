fn client_builder() -> reqwest::Client {
    // TODO: Connection Pool, It by default holds the connection pool internally
    reqwest::ClientBuilder::new()
        .http2_adaptive_window(true)
        .tcp_keepalive(std::time::Duration::new(150, 0))
        .tcp_nodelay(true)
        .connect_timeout(std::time::Duration::new(150, 0))
        .connection_verbose(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

static CLIENT: once_cell::sync::Lazy<std::sync::Arc<reqwest::Client>> =
    once_cell::sync::Lazy::new(|| std::sync::Arc::new(client_builder()));

// This method will connect client request to the out of the world
#[tracing::instrument(skip(req, extra_headers))]
pub(crate) async fn get_out(
    url: url::Url,
    req: &fastn_core::http::Request,
    extra_headers: &std::collections::HashMap<String, String>,
) -> fastn_core::Result<fastn_core::http::Response> {
    let headers = req.headers();

    let mut proxy_request = reqwest::Request::new(
        match req.method() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            "HEAD" => reqwest::Method::HEAD,
            "OPTIONS" => reqwest::Method::OPTIONS,
            "TRACE" => reqwest::Method::TRACE,
            "CONNECT" => reqwest::Method::CONNECT,
            _ => reqwest::Method::GET,
        },
        reqwest::Url::parse(
            format!(
                "{}/{}",
                url.as_str().trim_end_matches('/'),
                if req.query_string().is_empty() {
                    "".to_string()
                } else {
                    format!("?{}", req.query_string())
                }
            )
            .as_str(),
        )?,
    );

    *proxy_request.headers_mut() = headers.to_owned();

    for (header_key, header_value) in extra_headers {
        proxy_request.headers_mut().insert(
            reqwest::header::HeaderName::from_bytes(header_key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(header_value.as_str()).unwrap(),
        );
    }

    proxy_request.headers_mut().insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fastn"),
    );

    if let Some(ip) = req.get_ip_str() {
        proxy_request.headers_mut().insert(
            reqwest::header::FORWARDED,
            reqwest::header::HeaderValue::from_str(ip.as_str()).unwrap(),
        );
    }

    if let Some(cookies) = req.cookies_string() {
        proxy_request.headers_mut().insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(cookies.as_str()).unwrap(),
        );
    }

    for header in fastn_core::utils::ignore_headers() {
        proxy_request.headers_mut().remove(header);
    }

    *proxy_request.body_mut() = Some(req.body().to_vec().into());

    Ok(fastn_core::http::ResponseBuilder::from_reqwest(CLIENT.execute(proxy_request).await?).await)
}
