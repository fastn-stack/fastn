#[macro_export]
macro_rules! server_error {
    ($($t:tt)*) => {{
        fastn_core::http::server_error_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! unauthorised {
    ($($t:tt)*) => {{
        fastn_core::http::unauthorised_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! not_found {
    ($($t:tt)*) => {{
        fastn_core::http::not_found_(format!($($t)*) + "\n")
    }};
}

pub fn server_error_(msg: String) -> fastn_core::http::Response {
    fastn_core::warning!("server error: {}", msg);
    actix_web::HttpResponse::InternalServerError().body(msg)
}

pub fn unauthorised_(msg: String) -> fastn_core::http::Response {
    fastn_core::warning!("unauthorised: {}", msg);
    actix_web::HttpResponse::Unauthorized().body(msg)
}

pub fn not_found_(msg: String) -> fastn_core::http::Response {
    fastn_core::warning!("page not found: {}", msg);
    actix_web::HttpResponse::NotFound().body(msg)
}

impl actix_web::ResponseError for fastn_core::Error {}

pub type Response = actix_web::HttpResponse;
pub type StatusCode = actix_web::http::StatusCode;

pub fn ok(data: Vec<u8>) -> fastn_core::http::Response {
    actix_web::HttpResponse::Ok().body(data)
}

pub fn redirect(url: String) -> fastn_core::http::Response {
    actix_web::HttpResponse::PermanentRedirect()
        .insert_header(("LOCATION", url))
        .finish()
}

pub fn redirect_with_code(url: String, code: i32) -> fastn_core::http::Response {
    match code {
        301 => actix_web::HttpResponse::MovedPermanently(),
        302 => actix_web::HttpResponse::Found(),
        303 => actix_web::HttpResponse::SeeOther(),
        307 => actix_web::HttpResponse::TemporaryRedirect(),
        308 => actix_web::HttpResponse::PermanentRedirect(),
        _ => {
            fastn_core::warning!("invalid redirect code: {code}");
            actix_web::HttpResponse::PermanentRedirect()
        }
    }
    .insert_header(("LOCATION", url))
    .finish()
}

pub fn ok_with_content_type(
    data: Vec<u8>,
    content_type: mime_guess::Mime,
) -> fastn_core::http::Response {
    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(data)
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

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> fastn_core::Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }

    pub fn body_as_json(
        &self,
    ) -> fastn_core::Result<Option<std::collections::HashMap<String, serde_json::Value>>> {
        if self.body.is_empty() {
            return Ok(None);
        }
        if self.content_type() != Some(mime_guess::mime::APPLICATION_JSON) {
            return Err(fastn_core::Error::UsageError {
                message: fastn_core::warning!(
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
    }

    pub fn host(&self) -> String {
        self.host.to_string()
        // use std::borrow::Borrow;
        // self.headers
        //     .borrow()
        //     .get("host")
        //     .and_then(|v| v.to_str().ok())
        //     .unwrap_or("localhost")
        //     .to_string()
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    pub fn query(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.query
    }

    pub fn q<T>(&self, key: &str, default: T) -> fastn_core::Result<T>
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

    pub async fn from_reqwest(
        response: reqwest::Response,
        _package_name: &str,
    ) -> fastn_core::http::Response {
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

        // if status == actix_web::http::StatusCode::FOUND && false {
        //     response_builder.status(actix_web::http::StatusCode::OK);
        //     if let Some(location) = response.headers().get(actix_web::http::header::LOCATION) {
        //         let redirect = location.to_str().unwrap();
        //         let path = if redirect.trim_matches('/').is_empty() {
        //             format!("/-/{}/", package_name)
        //         } else {
        //             // if it contains query-params so url should not end with /
        //             if redirect.contains('?') {
        //                 format!("/-/{}/{}", package_name, redirect.trim_matches('/'))
        //             } else {
        //                 format!("/-/{}/{}/", package_name, redirect.trim_matches('/'))
        //             }
        //         };
        //         let t = serde_json::json!({"redirect": path.as_str()}).to_string();
        //         return response_builder.body(t);
        //     }
        // }

        let content = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(
                    fastn_core::Error::HttpError(e),
                ))
            }
        };
        response_builder.body(content)
    }
}

pub(crate) fn url_regex() -> regex::Regex {
    regex::Regex::new(
        r"((([A-Za-z]{3,9}:(?://)?)(?:[-;:&=\+\$,\w]+@)?[A-Za-z0-9.-]+|(?:www.|[-;:&=\+\$,\w]+@)[A-Za-z0-9.-]+)((?:/[\+~%/.\w_]*)?\??(?:[-\+=&;%@.\w_]*)\#?(?:[\w]*))?)"
    ).unwrap()
}

#[tracing::instrument]
pub(crate) async fn construct_url_and_get_str(url: &str) -> fastn_core::Result<String> {
    construct_url_and_return_response(url.to_string(), |f| async move {
        http_get_str(f.as_str()).await
    })
    .await
}

#[tracing::instrument]
pub(crate) async fn construct_url_and_get(url: &str) -> fastn_core::Result<Vec<u8>> {
    println!("http_download_by_id: {url}");
    construct_url_and_return_response(
        url.to_string(),
        |f| async move { http_get(f.as_str()).await },
    )
    .await
}

#[tracing::instrument(skip(f))]
pub(crate) async fn construct_url_and_return_response<T, F, D>(
    url: String,
    f: F,
) -> fastn_core::Result<D>
where
    F: FnOnce(String) -> T + Copy,
    T: futures::Future<Output = std::result::Result<D, fastn_core::Error>> + Send + 'static,
{
    let mut url = if url[1..].contains("://") || url.starts_with("//") {
        url
    } else {
        format!("https://{}", url)
    };

    if let Ok(package_proxy) = std::env::var("FASTN_PACKAGE_PROXY") {
        url = format!("{}/proxy/?url={}", package_proxy, url);
    }

    f(url).await
}

#[tracing::instrument]
pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> fastn_core::Result<T> {
    Ok(reqwest::Client::new()
        .get(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "fastn")
        .send()
        .await?
        .json::<T>()
        .await?)
}

#[tracing::instrument(skip(body))]
pub(crate) async fn post_json<T: serde::de::DeserializeOwned, B: Into<reqwest::Body>>(
    url: &str,
    body: B,
) -> fastn_core::Result<T> {
    Ok(reqwest::Client::new()
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "fastn")
        .body(body)
        .send()
        .await?
        .json()
        .await?)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn http_post_with_cookie(
    url: &str,
    cookie: Option<String>,
    headers: &std::collections::HashMap<String, String>,
    body: &str,
) -> fastn_core::Result<(fastn_core::Result<Vec<u8>>, Vec<String>)> {
    let mut resp_cookies = vec![];

    tracing::info!(url = url);
    let mut req_headers = reqwest::header::HeaderMap::new();
    req_headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fastn"),
    );
    if let Some(cookie) = cookie {
        req_headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(cookie.as_str()).unwrap(),
        );
    }

    for (key, value) in headers.iter() {
        req_headers.insert(
            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(value.as_str()).unwrap(),
        );
    }

    let c = reqwest::Client::builder()
        .default_headers(req_headers)
        .build()?;

    let res = c.post(url).body(body.to_string()).send().await?;
    res.headers().iter().for_each(|(k, v)| {
        if k.as_str().eq("set-cookie") {
            if let Ok(v) = v.to_str() {
                resp_cookies.push(v.to_string());
            }
        }
    });

    if !res.status().eq(&reqwest::StatusCode::OK) {
        let message = format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            res.status(),
            res.text().await
        );
        tracing::error!(url = url, msg = message);
        return Ok((
            Err(fastn_core::Error::APIResponseError(message)),
            resp_cookies,
        ));
    }
    tracing::info!(msg = "returning success", url = url);
    Ok((Ok(res.bytes().await?.into()), resp_cookies))
}

pub async fn http_get(url: &str) -> fastn_core::Result<Vec<u8>> {
    http_get_with_cookie(url, None, &std::collections::HashMap::new(), true)
        .await?
        .0
}

static NOT_FOUND_CACHE: once_cell::sync::Lazy<antidote::RwLock<std::collections::HashSet<String>>> =
    once_cell::sync::Lazy::new(|| antidote::RwLock::new(Default::default()));

#[tracing::instrument(skip_all)]
pub(crate) async fn http_get_with_cookie(
    url: &str,
    cookie: Option<String>,
    headers: &std::collections::HashMap<String, String>,
    use_cache: bool,
) -> fastn_core::Result<(fastn_core::Result<Vec<u8>>, Vec<String>)> {
    let mut cookies = vec![];

    if use_cache && NOT_FOUND_CACHE.read().contains(url) {
        return Ok((
            Err(fastn_core::Error::APIResponseError(
                "page not found, cached".to_string(),
            )),
            cookies,
        ));
    }

    tracing::info!(url = url);
    let mut req_headers = reqwest::header::HeaderMap::new();
    req_headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fastn"),
    );
    if let Some(cookie) = cookie {
        req_headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(cookie.as_str()).unwrap(),
        );
    }

    for (key, value) in headers.iter() {
        req_headers.insert(
            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(value.as_str()).unwrap(),
        );
    }

    let c = reqwest::Client::builder()
        .default_headers(req_headers)
        .build()?;

    // if we are not able to send the request, we could not have read the cookie
    let res = c.get(url).send().await?;

    res.headers().iter().for_each(|(k, v)| {
        if k.as_str().eq("set-cookie") {
            if let Ok(v) = v.to_str() {
                cookies.push(v.to_string());
            }
        }
    });

    if !res.status().eq(&reqwest::StatusCode::OK) {
        let message = format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            res.status(),
            res.text().await
        );
        tracing::error!(url = url, msg = message);

        if use_cache {
            NOT_FOUND_CACHE.write().insert(url.to_string());
        }

        return Ok((Err(fastn_core::Error::APIResponseError(message)), cookies));
    }
    tracing::info!(msg = "returning success", url = url);
    Ok((Ok(res.bytes().await?.into()), cookies))
}

// pub async fn http_get_with_type<T: serde::de::DeserializeOwned>(
//     url: url::Url,
//     headers: reqwest::header::HeaderMap,
//     query: &[(String, String)],
// ) -> fastn_core::Result<T> {
//     let c = reqwest::Client::builder()
//         .default_headers(headers)
//         .build()?;
//
//     let resp = c.get(url.to_string().as_str()).query(query).send().await?;
//     if !resp.status().eq(&reqwest::StatusCode::OK) {
//         return Err(fastn_core::Error::APIResponseError(format!(
//             "url: {}, response_status: {}, response: {:?}",
//             url,
//             resp.status(),
//             resp.text().await
//         )));
//     }
//     Ok(resp.json::<T>().await?)
// }

pub(crate) async fn http_get_str(url: &str) -> fastn_core::Result<String> {
    let url_f = format!("{:?}", url);
    match http_get(url).await {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| fastn_core::Error::UsageError {
            message: format!(
                "Cannot convert the response to string: URL: {:?}, ERROR: {}",
                url_f, e
            ),
        }),
        Err(e) => Err(e),
    }
}

pub(crate) fn api_ok(
    data: impl serde::Serialize,
) -> fastn_core::Result<fastn_core::http::Response> {
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

/// construct an error response with `message`
/// and `status_code`. Use 500 if `status_code` is None
pub(crate) fn api_error<T: Into<String>>(
    message: T,
    status_code: Option<actix_web::http::StatusCode>,
) -> fastn_core::Result<fastn_core::http::Response> {
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
        .status(status_code.unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR))
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

pub async fn get_api<T: serde::de::DeserializeOwned>(
    url: impl AsRef<str>,
    bearer_token: &str,
) -> fastn_core::Result<T> {
    let response = reqwest::Client::new()
        .get(url.as_ref())
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{} {}", "Bearer", bearer_token),
        )
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fastn"),
        )
        .send()
        .await?;

    if !response.status().eq(&reqwest::StatusCode::OK) {
        return Err(fastn_core::Error::APIResponseError(format!(
            "fastn-API-ERROR: {}, Error: {}",
            url.as_ref(),
            response.text().await?
        )));
    }

    Ok(response.json().await?)
}

pub async fn github_graphql<T: serde::de::DeserializeOwned>(
    query: &str,
    token: &str,
) -> fastn_core::Result<T> {
    let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    map.insert("query", query);

    let response = reqwest::Client::new()
        .post("https://api.github.com/graphql")
        .json(&map)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{} {}", "Bearer", token),
        )
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fastn"),
        )
        .send()
        .await?;
    if !response.status().eq(&reqwest::StatusCode::OK) {
        return Err(fastn_core::Error::APIResponseError(format!(
            "GitHub API ERROR: {}",
            response.status()
        )));
    }
    let return_obj = response.json::<T>().await?;

    Ok(return_obj)
}

/// construct `ftd.http` consumable error responses
/// https://github.com/fastn-stack/fastn/blob/7f0b79a/fastn-js/js/ftd.js#L218C45-L229
/// ```json
/// {
///     "data": null,
///     "errors": {
///         key: String,
///         key2: String,
///         ...
///     }
/// }
/// ```
pub async fn user_err(
    errors: Vec<(&str, &str)>,
    status_code: fastn_core::http::StatusCode,
) -> fastn_core::Result<fastn_core::http::Response> {
    let mut json_error = serde_json::Map::new();

    for (k, v) in errors {
        json_error.insert(k.to_string(), serde_json::Value::String(v.to_string()));
    }

    let resp = serde_json::json!({
        "data": null,
        "errors": json_error,
    });

    Ok(actix_web::HttpResponse::Ok()
        .status(status_code)
        .content_type(actix_web::http::header::ContentType::json())
        .body(serde_json::to_string(&resp)?))
}

#[cfg(test)]
mod test {
    use actix_web::body::MessageBody;

    #[tokio::test]
    async fn user_err() -> fastn_core::Result<()> {
        let user_err = "invalid email";
        let token_err = "no key expected with name token";
        let errors = vec![("user", user_err), ("token", token_err)];

        let res =
            fastn_core::http::user_err(errors, fastn_core::http::StatusCode::BAD_REQUEST).await?;

        assert_eq!(res.status(), fastn_core::http::StatusCode::BAD_REQUEST);

        #[derive(serde::Deserialize)]
        struct Errors {
            user: String,
            token: String,
        }

        #[derive(serde::Deserialize)]
        struct TestErrorResponse {
            data: Option<String>,
            errors: Errors,
        }

        let bytes = res.into_body().try_into_bytes().unwrap();

        let body: TestErrorResponse = serde_json::from_slice(&bytes)?;

        assert_eq!(body.data, None);
        assert_eq!(body.errors.user, user_err);
        assert_eq!(body.errors.token, token_err);

        Ok(())
    }
}
