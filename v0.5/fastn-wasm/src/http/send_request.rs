pub async fn send_request<S: Send>(
    mut caller: wasmtime::Caller<'_, S>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let r: ft_sys_shared::Request = fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;

    let mut headers = reqwest::header::HeaderMap::new();
    for (header_name, header_value) in r.headers {
        let header_name = reqwest::header::HeaderName::from_bytes(header_name.as_bytes())?;
        let header_value = reqwest::header::HeaderValue::from_bytes(header_value.as_slice())?;
        headers.insert(header_name, header_value);
    }
    let reqwest_response = if r.method.to_uppercase().eq("GET") {
        reqwest::Client::new().get(r.uri)
    } else {
        reqwest::Client::new().post(r.uri).body(r.body)
    }
    .headers(headers)
    .send()
    .await?;

    let mut response = http::Response::builder().status(reqwest_response.status());
    for (header_name, header_value) in reqwest_response.headers() {
        response = response.header(header_name, header_value);
    }
    let response = response.body(reqwest_response.bytes().await?)?;

    fastn_wasm::helpers::send_json(ft_sys_shared::Request::from(response), &mut caller).await
}
