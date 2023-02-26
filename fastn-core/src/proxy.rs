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
#[tracing::instrument(skip_all)]
pub(crate) async fn get_out(
    host: &str,
    req: fastn_core::http::Request,
    path: &str,
    package_name: &str,
    req_headers: &std::collections::HashMap<String, String>,
) -> fastn_core::Result<fastn_core::http::Response> {
    let headers = req.headers();
    // TODO: It should be part of fastn_core::Request::uri()
    // let path = &req.uri().to_string()[1..];

    tracing::info!("proxy_request: {} {} {}", req.method(), path, host);

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
                "{}/{}{}",
                host.trim_end_matches('/'),
                path.trim_start_matches('/'),
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

    // TODO: Some extra headers, possibly Authentication header
    // Authentication header can come from system environment variable
    // env file path set in FASTN.ftd file
    // We can get the data from request parameter
    // Flow will be
    // 1. Add Movie ftd page
    // 2. fastn will forward request to microservice and that service will redirect to /movie/?id=5
    // fastn will send back this response to browser
    // browser will request now /movie/?id=5
    // on this movie.ftd page we will call a processor: `request-data` which will give the
    // `id` from the request query parameter. Than, we will use processor: `http` to call http api
    // `/api/movie/?id=<id>` of movie-db service, this will happen while fastn is converting ftd code
    // to html, so all this happening on server side. So we can say server side rendering.

    // headers

    for (header_key, header_value) in req_headers {
        proxy_request.headers_mut().insert(
            reqwest::header::HeaderName::from_bytes(header_key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(header_value.as_str()).unwrap(),
        );
    }

    proxy_request.headers_mut().insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fastn"),
    );

    if let Some(ip) = req.get_ip() {
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

    Ok(fastn_core::http::ResponseBuilder::from_reqwest(
        CLIENT.execute(proxy_request).await?,
        package_name,
    )
    .await)
}
