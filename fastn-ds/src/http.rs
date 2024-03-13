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
                    fastn_ds::DSError::HttpError(e),
                ))
            }
        };
        response_builder.body(content)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Request {
    method: String,
    pub uri: String,
    pub path: String,
    query_string: String,
    cookies: std::collections::HashMap<String, String>,
    headers: reqwest::header::HeaderMap,
    query: std::collections::HashMap<String, serde_json::Value>,
    body: actix_web::web::Bytes,
    ip: Option<String>,
    scheme: String,
    host: String,
    pub connection_info: actix_web::dev::ConnectionInfo,
    // path_params: Vec<(String, )>
}

impl Request {
    //pub fn get_named_params() -> {}
    pub fn full_path(&self) -> String {
        if self.query_string.is_empty() {
            self.path.clone()
        } else {
            format!("{}?{}", self.path, self.query_string)
        }
    }

    pub fn from_actix(req: actix_web::HttpRequest, body: actix_web::web::Bytes) -> Self {
        let headers = {
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in req.headers() {
                headers.insert(key.clone(), value.clone());
            }
            headers
        };

        return Request {
            cookies: get_cookies(&headers),
            body,
            method: req.method().to_string(),
            uri: req.uri().to_string(),
            path: req.path().to_string(),
            query_string: req.query_string().to_string(),
            connection_info: req.connection_info().clone(),
            headers,
            query: {
                actix_web::web::Query::<std::collections::HashMap<String, serde_json::Value>>::from_query(
                    req.query_string(),
                ).unwrap().0
            },
            ip: req.peer_addr().map(|x| x.ip().to_string()),
            scheme: req.connection_info().scheme().to_string(),
            host: req.connection_info().host().to_string(),
        };

        fn get_cookies(
            headers: &reqwest::header::HeaderMap,
        ) -> std::collections::HashMap<String, String> {
            let mut cookies = std::collections::HashMap::new();
            if let Some(cookie) = headers.get("cookie") {
                if let Ok(cookie) = cookie.to_str() {
                    for cookie in cookie.split(';') {
                        let cookie = cookie.trim();
                        if let Some(index) = cookie.find('=') {
                            let (key, value) = cookie.split_at(index);
                            let key = key.trim();
                            let value = value.trim_start_matches('=').trim();
                            cookies.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
            cookies
        }
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.body)
    }

    pub fn body_as_json(
        &self,
    ) -> fastn_ds::DSResult<Option<std::collections::HashMap<String, serde_json::Value>>> {
        if self.body.is_empty() {
            return Ok(None);
        }
        if self.content_type() != Some(mime_guess::mime::APPLICATION_JSON) {
            return Err(fastn_ds::DSError::GenericError {
                message: format!(
                    "expected content type {}, got {:?}",
                    mime_guess::mime::APPLICATION_JSON,
                    self.content_type()
                ),
            });
        }
        Ok(Some(serde_json::from_slice(&self.body)?))
    }

    pub fn content_type(&self) -> Option<mime_guess::Mime> {
        self.headers
            .get(actix_web::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

    pub fn user_agent(&self) -> Option<String> {
        self.headers
            .get(actix_web::http::header::USER_AGENT)
            .and_then(|v| v.to_str().map(|v| v.to_string()).ok())
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn query_string(&self) -> &str {
        self.query_string.as_str()
    }

    pub fn cookies(&self) -> &std::collections::HashMap<String, String> {
        &self.cookies
    }

    pub fn cookies_string(&self) -> Option<String> {
        if self.cookies.is_empty() {
            return None;
        }
        Some(
            self.cookies()
                .iter()
                // TODO: check if extra escaping is needed
                .map(|(k, v)| format!("{}={}", k, v).replace(';', "%3B"))
                .collect::<Vec<_>>()
                .join(";"),
        )
    }

    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies().get(name).map(|v| v.to_string())
    }

    pub fn set_body(&mut self, body: actix_web::web::Bytes) {
        self.body = body;
    }

    pub fn set_cookies(&mut self, cookies: &std::collections::HashMap<String, String>) {
        self.cookies = cookies.clone();
    }

    pub fn insert_header(&mut self, name: reqwest::header::HeaderName, value: &'static str) {
        self.headers
            .insert(name, reqwest::header::HeaderValue::from_static(value));
    }

    pub fn set_method(&mut self, method: &str) {
        self.method = method.to_uppercase();
    }

    pub fn set_query_string(&mut self, query_string: &str) {
        self.query_string = query_string.to_string();
        self.query = actix_web::web::Query::<std::collections::HashMap<String, serde_json::Value>>::from_query(
            self.query_string.as_str(),
        ).unwrap().0;
    }

    pub fn host(&self) -> String {
        self.host.to_string()
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    pub fn query(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.query
    }

    pub fn q<T>(&self, key: &str, default: T) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = match self.query.get(key) {
            Some(v) => v,
            None => return Ok(default),
        };

        Ok(serde_json::from_value(value.clone())?)
    }

    pub fn get_ip(&self) -> Option<String> {
        self.ip.clone()
    }
    pub fn scheme(&self) -> String {
        self.scheme.to_string()
    }
}
