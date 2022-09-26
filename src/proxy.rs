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
pub(crate) async fn get_out(
    host: &str,
    req: fpm::http::Request,
    body: &[u8], // TODO: Not liking it, It should be fetched from request only
) -> fpm::Result<fpm::http::Response> {
    let headers = req.headers();
    // TODO: It should be part of fpm::Request::uri()
    let path = &req.req.uri().to_string()[1..];

    println!("proxy_request: {} {}", req.req.method(), path);

    let mut proxy_request = reqwest::Request::new(
        req.req.method().clone(),
        reqwest::Url::parse(format!("{}/{}?{}", host, path, req.req.query_string()).as_str())?,
    );

    *proxy_request.headers_mut() = headers;

    // TODO: Some extra headers, possibly Authentication header
    // Authentication header can come from system environment variable
    // env file path set in FPM.ftd file
    // We can get the data from request parameter
    // Flow will be
    // 1. Add Movie ftd page
    // 2. fpm will forward request to microservice and that service will redirect to /movie/?id=5
    // fpm will send back this response to browser
    // browser will request now /movie/?id=5
    // on this movie.ftd page we will call a processor: `request-data` which will give the
    // `id` from the request query parameter. Than, we will use processor: `http` to call http api
    // `/api/movie/?id=<id>` of movie-db service, this will happen while fpm is converting ftd code
    // to html, so all this happening on server side. So we can say server side rendering.

    proxy_request.headers_mut().insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );

    *proxy_request.body_mut() = Some(body.to_vec().into());

    Ok(fpm::http::ResponseBuilder::from_reqwest(CLIENT.execute(proxy_request).await?).await)
}
