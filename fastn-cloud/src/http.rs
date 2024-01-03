#[allow(dead_code)]
const FASTN_CW_HOST: &str = "http://127.0.0.1:3001";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ReqwestError: {}", _0)]
    Reqwest(#[from] reqwest::Error),
    #[error("HeadersError: {}", _0)]
    Headers(String),
    #[error("ResponseParseError: {0}")]
    ResponseParse(#[from] serde_json::Error),
    #[error("ResponseError : {msg}")]
    HttpResponse { msg: String },
}

#[derive(serde::Deserialize, Debug)]
struct ApiResponse {
    success: bool,
    #[serde(default)]
    data: serde_json::Value,
    #[serde(default)]
    msg: serde_json::Value,
}

pub(crate) async fn request<T: serde::de::DeserializeOwned, B: Into<reqwest::Body>>(
    method: reqwest::Method,
    url: &str,
    body: B,
    headers: &std::collections::HashMap<String, String>,
    query: &std::collections::HashMap<String, String>,
) -> Result<T, Error> {
    let url = format!("{}{}", FASTN_CW_HOST, url);

    let resp: ApiResponse = reqwest::Client::new()
        .request(method, url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "fastn")
        .headers(map_to_headers(headers)?)
        .query(query)
        .body(body)
        .send()
        .await?
        .json()
        .await?;

    println!("{:?}", resp);

    // TODO: Handle The errors and different statuses

    if resp.success {
        Ok(serde_json::from_value(resp.data)?)
    } else {
        println!("Response Error: {}", &resp.msg);
        Err(Error::HttpResponse {
            msg: resp.msg.to_string(),
        })
    }
}

fn map_to_headers(
    headers: &std::collections::HashMap<String, String>,
) -> Result<reqwest::header::HeaderMap, Error> {
    let headers: Result<reqwest::header::HeaderMap, String> = headers
        .iter()
        .map(
            |(k, v)| -> Result<(reqwest::header::HeaderName, reqwest::header::HeaderValue), String> {
                let name = TryFrom::try_from(k).map_err(|e: reqwest::header::InvalidHeaderName| e.to_string())?;
                let value = TryFrom::try_from(v).map_err(|e: reqwest::header::InvalidHeaderValue| e.to_string())?;
                Ok((name, value))
            },
        )
        .collect();
    headers.map_err(Error::Headers)
}
