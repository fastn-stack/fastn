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

pub static CLIENT: once_cell::sync::Lazy<std::sync::Arc<reqwest::Client>> =
    once_cell::sync::Lazy::new(|| std::sync::Arc::new(client_builder()));

pub static NOT_FOUND_CACHE: once_cell::sync::Lazy<
    antidote::RwLock<std::collections::HashSet<String>>,
> = once_cell::sync::Lazy::new(|| antidote::RwLock::new(Default::default()));

pub(crate) struct ResponseBuilder {}

impl ResponseBuilder {
    pub async fn from_reqwest(response: reqwest::Response) -> fastn_ds::HttpResponse {
        let status = response.status();

        // Remove `Connection` as per
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
        let mut response_builder = actix_web::HttpResponse::build(status);
        for header in response
            .headers()
            .iter()
            .filter(|(h, _)| *h != "connection")
        {
            response_builder.insert_header(header);
        }

        let content = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(
                    fastn_ds::HttpError::ReqwestError(e),
                ))
            }
        };
        response_builder.body(content)
    }
}
