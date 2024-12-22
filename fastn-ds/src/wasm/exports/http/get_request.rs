pub async fn get_request(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store>,
) -> wasmtime::Result<i32> {
    let req = caller.data().to_http();
    fastn_wasm::helpers::send_json(req, &mut caller).await
}
