#[macro_export]
macro_rules! server_error {
    ($($t:tt)*) => {{
        fpm::http::server_error_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! unauthorised {
    ($($t:tt)*) => {{
        fpm::http::unauthorised_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! not_found {
    ($($t:tt)*) => {{
        fpm::http::not_found_(format!($($t)*))
    }};
}

pub fn server_error_(msg: String) -> fpm::http::Response {
    fpm::warning!("server error: {}", msg);
    actix_web::HttpResponse::InternalServerError().body(msg)
}

pub fn unauthorised_(msg: String) -> fpm::http::Response {
    fpm::warning!("unauthorised: {}", msg);
    actix_web::HttpResponse::Unauthorized().body(msg)
}

pub fn not_found_(msg: String) -> fpm::http::Response {
    fpm::warning!("page not found: {}", msg);
    actix_web::HttpResponse::NotFound().body(msg)
}

impl actix_web::ResponseError for fpm::Error {}

pub type Response = actix_web::HttpResponse;

pub fn ok(data: Vec<u8>) -> fpm::http::Response {
    actix_web::HttpResponse::Ok().body(data)
}

pub fn ok_with_content_type(data: Vec<u8>, content_type: mime_guess::Mime) -> fpm::http::Response {
    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(data)
}

#[derive(Debug, Clone)]
pub struct Request {
    method: String,
    uri: String,
    path: String,
    query_string: String,
    cookies: std::collections::HashMap<String, String>,
    headers: reqwest::header::HeaderMap,
    query: std::collections::HashMap<String, serde_json::Value>,
    body: actix_web::web::Bytes,
    ip: Option<String>,
    // path_params: Vec<(String, )>
}

impl Request {
    //pub fn get_named_params() -> {}
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
            headers,
            query: {
                actix_web::web::Query::<std::collections::HashMap<String, serde_json::Value>>::from_query(
                    req.query_string(),
                ).unwrap().0
            },
            ip: req.peer_addr().map(|x| x.ip().to_string()),
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

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> fpm::Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }

    pub fn body_as_json(
        &self,
    ) -> fpm::Result<Option<std::collections::HashMap<String, serde_json::Value>>> {
        if self.body.is_empty() {
            return Ok(None);
        }
        if self.content_type() != Some(mime_guess::mime::APPLICATION_JSON) {
            return Err(fpm::Error::UsageError {
                message: fpm::warning!(
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
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
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

    pub fn host(&self) -> String {
        use std::borrow::Borrow;

        self.headers
            .borrow()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost")
            .to_string()
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    pub fn query(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.query
    }

    pub fn get_ip(&self) -> Option<String> {
        self.ip.clone()
    }
}

pub(crate) struct ResponseBuilder {
    // headers: std::collections::HashMap<String, String>,
    // code: u16,
    // remaining
}

// We will no do stream, data is going to less from services
impl ResponseBuilder {
    // chain implementation
    // .build
    // response from string, json, bytes etc

    pub async fn from_reqwest(response: reqwest::Response) -> fpm::http::Response {
        let status = response.status();

        let mut response_builder = actix_web::HttpResponse::build(status);
        // TODO
        // *resp.extensions_mut() = response.extensions().clone();

        // Remove `Connection` as per
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
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
                    fpm::Error::HttpError(e),
                ))
            }
        };
        println!("Response {:?}", String::from_utf8(content.to_vec()));
        response_builder.body(content)
    }
}

pub(crate) fn url_regex() -> regex::Regex {
    regex::Regex::new(
        r#"((([A-Za-z]{3,9}:(?://)?)(?:[-;:&=\+\$,\w]+@)?[A-Za-z0-9.-]+|(?:www.|[-;:&=\+\$,\w]+@)[A-Za-z0-9.-]+)((?:/[\+~%/.\w_]*)?\??(?:[-\+=&;%@.\w_]*)\#?(?:[\w]*))?)"#
    ).unwrap()
}

pub(crate) async fn construct_url_and_get_str(url: &str) -> fpm::Result<String> {
    construct_url_and_return_response(url.to_string(), |f| async move {
        http_get_str(f.as_str()).await
    })
    .await
}

pub(crate) async fn construct_url_and_get(url: &str) -> fpm::Result<Vec<u8>> {
    construct_url_and_return_response(
        url.to_string(),
        |f| async move { http_get(f.as_str()).await },
    )
    .await
}

pub(crate) async fn construct_url_and_return_response<T, F, D>(url: String, f: F) -> fpm::Result<D>
where
    F: FnOnce(String) -> T + Copy,
    T: futures::Future<Output = std::result::Result<D, fpm::Error>> + Send + 'static,
{
    if url[1..].contains("://") || url.starts_with("//") {
        f(url).await
    } else if let Ok(response) = f(format!("https://{}", url)).await {
        Ok(response)
    } else {
        f(format!("http://{}", url)).await
    }
}

pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> fpm::Result<T> {
    Ok(reqwest::Client::new()
        .get(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "fpm")
        .send()
        .await?
        .json::<T>()
        .await?)
}

pub(crate) async fn post_json<T: serde::de::DeserializeOwned, B: Into<reqwest::Body>>(
    url: &str,
    body: B,
) -> fpm::Result<T> {
    Ok(reqwest::Client::new()
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "fpm")
        .body(body)
        .send()
        .await?
        .json()
        .await?)
}

pub(crate) async fn http_get(url: &str) -> fpm::Result<Vec<u8>> {
    http_get_with_cookie(url, None).await
}

pub(crate) async fn http_get_with_cookie(
    url: &str,
    cookie: Option<String>,
) -> fpm::Result<Vec<u8>> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );
    if let Some(cookie) = cookie {
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(cookie.as_str()).unwrap(),
        );
    }

    let c = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let res = c.get(url).send().await?;
    if !res.status().eq(&reqwest::StatusCode::OK) {
        return Err(fpm::Error::APIResponseError(format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            res.status(),
            res.text().await
        )));
    }
    Ok(res.bytes().await?.into())
}

pub async fn http_get_with_type<T: serde::de::DeserializeOwned>(
    url: url::Url,
    headers: reqwest::header::HeaderMap,
    query: &[(String, String)],
) -> fpm::Result<T> {
    let c = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let resp = c.get(url.to_string().as_str()).query(query).send().await?;
    if !resp.status().eq(&reqwest::StatusCode::OK) {
        return Err(fpm::Error::APIResponseError(format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            resp.status(),
            resp.text().await
        )));
    }
    Ok(resp.json::<T>().await?)
}

pub(crate) async fn http_get_str(url: &str) -> fpm::Result<String> {
    let url_f = format!("{:?}", url);
    match http_get(url).await {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| fpm::Error::UsageError {
            message: format!(
                "Cannot convert the response to string: URL: {:?}, ERROR: {}",
                url_f, e
            ),
        }),
        Err(e) => Err(e),
    }
}

pub(crate) fn api_ok(data: impl serde::Serialize) -> fpm::Result<fpm::http::Response> {
    #[derive(serde::Serialize)]
    struct SuccessResponse<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let data = serde_json::to_vec(&SuccessResponse {
        data,
        success: true,
    })?;

    Ok(ok_with_content_type(
        data,
        mime_guess::mime::APPLICATION_JSON,
    ))
}

pub(crate) fn api_error<T: Into<String>>(message: T) -> fpm::Result<fpm::http::Response> {
    #[derive(serde::Serialize, Debug)]
    struct ErrorResponse {
        message: String,
        success: bool,
    }

    let resp = ErrorResponse {
        message: message.into(),
        success: false,
    };

    Ok(actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::json())
        .status(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(serde_json::to_string(&resp)?))
}

pub(crate) fn get_available_port(
    port: Option<u16>,
    bind_address: &str,
) -> Option<std::net::TcpListener> {
    let available_listener =
        |port: u16, bind_address: &str| std::net::TcpListener::bind((bind_address, port));

    if let Some(port) = port {
        return match available_listener(port, bind_address) {
            Ok(l) => Some(l),
            Err(_) => None,
        };
    }

    for x in 8000..9000 {
        match available_listener(x, bind_address) {
            Ok(l) => return Some(l),
            Err(_) => continue,
        }
    }
    None
}
