pub fn to_http_response(_r: reqwest::Response) -> http::Response<bytes::Bytes> {
    // http::Response::builder()
    //     .header("Content-Type", "application/json")
    //     .body(bytes::Bytes::from(
    //         serde_json::to_string(&self).unwrap(),
    //     ))
    //     .unwrap()
    todo!()
}