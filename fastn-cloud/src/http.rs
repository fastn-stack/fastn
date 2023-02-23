const FASTN_HOST: &str = "http://127.0.0.1:8000";
// fastn-cw host

#[derive(thiserror::Error, Debug)]
pub enum PostError {
    #[error("ReqwestError: {}", _0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("HeadersError")]
    HeadersError(String),
}

pub(crate) async fn post_json<T: serde::de::DeserializeOwned, B: Into<reqwest::Body>>(
    url: &str,
    body: B,
    headers: &std::collections::HashMap<String, String>,
    query: &std::collections::HashMap<String, String>,
) -> Result<T, PostError> {
    let headers: Result<reqwest::header::HeaderMap, String> = headers
        .into_iter()
        .map(
            |(k, v)| -> Result<(reqwest::header::HeaderName, reqwest::header::HeaderValue), String> {
                let name = TryFrom::try_from(k).map_err(|e: reqwest::header::InvalidHeaderName| e.to_string())?;
                let value = TryFrom::try_from(v).map_err(|e:reqwest::header::InvalidHeaderValue| e.to_string())?;
                Ok((name, value))
            },
        )
        .collect();
    let hrs = headers.map_err(|e| PostError::HeadersError(e))?;
    Ok(reqwest::Client::new()
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "fastn")
        .headers(hrs)
        .query(query)
        .body(body)
        .send()
        .await?
        .json()
        .await?)
}
