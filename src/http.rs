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

pub fn server_error_(msg: String) -> actix_web::HttpResponse {
    fpm::warning!("server error: {}", msg);
    actix_web::HttpResponse::InternalServerError().body(msg)
}

pub fn unauthorised_(msg: String) -> actix_web::HttpResponse {
    fpm::warning!("unauthorised: {}", msg);
    actix_web::HttpResponse::Unauthorized().body(msg)
}

pub fn not_found_(msg: String) -> actix_web::HttpResponse {
    fpm::warning!("page not found: {}", msg);
    actix_web::HttpResponse::NotFound().body(msg)
}

pub fn ok<V>(data: V) -> actix_web::HttpResponse
where
    V: actix_http::body::MessageBody + 'static,
{
    actix_web::HttpResponse::Ok().body(data)
}

pub fn ok_with_content_type<V, B>(data: B, content_type: V) -> actix_web::HttpResponse
where
    B: actix_http::body::MessageBody + 'static,
    V: actix_http::header::TryIntoHeaderValue,
{
    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(data)
}

#[derive(Debug, Clone)]
pub struct Request {
    pub req: actix_web::HttpRequest,
}

impl Request {
    pub fn from_actix(req: actix_web::HttpRequest) -> Self {
        Request { req }
    }

    pub fn method(&self) -> &str {
        self.req.method().as_str()
    }

    pub fn uri(&self) -> String {
        self.req.uri().to_string()
    }

    pub fn path(&self) -> &str {
        self.req.path()
    }

    pub fn url_data(&self, key: &str) -> &str {
        self.req.match_info().query(key)
    }

    pub fn cookies(&self) -> std::collections::HashMap<String, String> {
        self.req
            .cookies()
            .unwrap()
            .iter()
            .map(|c| (c.name().to_string(), c.value().to_string()))
            .collect()
    }

    pub fn cookie(&self, name: &str) -> Option<String> {
        self.req.cookie(name).map(|v| v.value().to_string())
    }

    pub fn host(&self) -> String {
        self.req.connection_info().host().to_string()
    }

    pub fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in self.req.headers() {
            headers.insert(key.clone(), value.clone());
        }
        headers
    }

    pub fn query(&self) -> fpm::Result<std::collections::HashMap<String, serde_json::Value>> {
        // TODO: Remove unwrap
        Ok(actix_web::web::Query::<std::collections::HashMap<String, serde_json::Value>>::from_query(
            self.req.query_string(),
        ).map_err(fpm::Error::QueryPayloadError)?.0)
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

    pub async fn from_reqwest(response: reqwest::Response) -> actix_web::HttpResponse {
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
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );
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

pub(crate) fn api_ok(data: impl serde::Serialize) -> actix_web::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize)]
    struct SuccessResponse<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let data = serde_json::to_string(&SuccessResponse {
        data,
        success: true,
    })?;

    Ok(ok_with_content_type(data, "application/json"))
}

pub(crate) fn api_error<T: Into<String>>(message: T) -> actix_web::Result<actix_web::HttpResponse> {
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
