pub async fn to_http_response(
    r: reqwest::Response,
) -> Result<http::Response<bytes::Bytes>, reqwest::Error> {
    let mut b = http::Response::builder().status(r.status().as_u16());
    for (k, v) in r.headers().iter() {
        b = b.header(k.as_str(), v.as_bytes());
    }
    Ok(b.body(r.bytes().await?).unwrap()) // unwrap is okay
}
