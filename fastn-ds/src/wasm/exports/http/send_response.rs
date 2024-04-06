pub async fn send_response(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    let r = fastn_ds::wasm::helpers::get_json(ptr, len, &mut caller).await?;
    caller
        .data_mut()
        .store_response(fastn_ds::wasm::Response::Http(r));
    Ok(())
}

impl fastn_ds::wasm::Store {
    pub fn store_response(&mut self, r: fastn_ds::wasm::Response) {
        self.response = Some(r);
    }
}
