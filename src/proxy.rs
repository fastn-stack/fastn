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

lazy_static! {
    static ref CLIENT: std::sync::Arc<reqwest::Client> = std::sync::Arc::new(client_builder());
}

// This method will connect client request to out of the world
pub(crate) async fn get_out(
    host: &str,
    req: fpm::http::Request<'_>,
    body: actix_web::web::Bytes, // TODO: Not liking it, It should be fetched from request only
) -> actix_web::HttpResponse {
    let headers = req.headers();
    // TODO: It should be part of fpm::Request::uri()
    let path = &req.req.uri().to_string()[1..];

    println!("proxy_request: {} {}", req.req.method(), path);

    let mut proxy_request = reqwest::Request::new(
        req.req.method().clone(),
        reqwest::Url::parse(format!("{}/{}?{}", host, path, req.req.query_string()).as_str())
            .unwrap(),
    );

    *proxy_request.headers_mut() = headers;
    *proxy_request.body_mut() = Some(body.to_vec().into());

    match CLIENT.execute(proxy_request).await {
        Ok(response) => {
            println!("success response from service");
            fpm::http::ResponseBuilder::from_reqwest(response).await
        }
        Err(e) => actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(
            fpm::Error::HttpError(e),
        )),
    }
}
